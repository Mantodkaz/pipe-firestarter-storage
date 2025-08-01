use eframe::egui;
use std::thread;
use std::sync::{Arc, Mutex};
use super::UploadDownloadState;
use rfd::FileDialog;
use std::path::Path;
use std::process::Command;

pub struct UploadPanelState {
    pub selected_tier: usize, // 0: normal, 1: priority, 2: premium, 3: ultra, 4: enterprise
    pub status_raw: Arc<Mutex<String>>, 
    pub list_uploads: Arc<Mutex<String>>,
    pub last_refresh: std::time::Instant,
    pub search: String,
    pub encrypt: bool,
    pub decrypt: bool,
    pub password: String,
    pub is_processing: bool,
    pub processing_flag: Arc<Mutex<bool>>,
    pub upload_success: bool,
}

impl Default for UploadPanelState {
    fn default() -> Self {
        Self {
            status_raw: Arc::new(Mutex::new(String::new())),
            list_uploads: Arc::new(Mutex::new(String::from("Loading..."))),
            last_refresh: std::time::Instant::now(),
            search: String::new(),
            encrypt: false,
            decrypt: false,
            password: String::new(),
            is_processing: false,
            processing_flag: Arc::new(Mutex::new(false)),
            upload_success: false,
            selected_tier: 0,
        }
    }
}

pub fn upload_download_panel(
    ui: &mut egui::Ui,
    panel_state: &mut UploadPanelState,
    state: &mut UploadDownloadState,
) {
    // Sync processing flag for UI
    {
        let flag = panel_state.processing_flag.lock().unwrap();
        panel_state.is_processing = *flag;
    }
    // Force repaint every frame for real-time status/progress
    ui.ctx().request_repaint();

    // Refresh uploads list if needed or if still loading
    let uploads_status = panel_state.list_uploads.lock().unwrap().clone();
    let refresh_needed = panel_state.last_refresh.elapsed().as_secs() > 10 || uploads_status.trim() == "Loading...";
    if refresh_needed {
        let list_uploads = panel_state.list_uploads.clone();
        panel_state.last_refresh = std::time::Instant::now();
        thread::spawn(move || {
            let output = Command::new("pipe")
                .arg("list-uploads")
                .output();
            let result = match output {
                Ok(out) => {
                    if out.status.success() {
                        String::from_utf8_lossy(&out.stdout).to_string()
                    } else {
                        format!("Failed to list uploads:\n{}", String::from_utf8_lossy(&out.stderr))
                    }
                }
                Err(e) => format!("Failed to run CLI: {}", e),
            };
            let mut uploads = list_uploads.lock().unwrap();
            *uploads = result;
        });
    }

    ui.vertical_centered(|ui| {
        ui.heading("📤 Pipe Upload/Download Panel");
        ui.separator();
    });

    ui.columns(2, |columns| {
        // Left column: upload/download
        let ui = &mut columns[0];
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.label("Mode:");
                let modes = ["Upload", "Download"];
                ui.radio_value(&mut state.mode, 0, modes[0]);
                ui.radio_value(&mut state.mode, 1, modes[1]);

                ui.separator();

                if state.mode == 0 {
                    ui.label("Tier:");
                    let tier_names = ["normal", "priority", "premium", "ultra", "enterprise"];
                    egui::ComboBox::from_id_source("tier_combo")
                        .selected_text(tier_names[panel_state.selected_tier])
                        .show_ui(ui, |ui| {
                            for (i, name) in tier_names.iter().enumerate() {
                                ui.selectable_value(&mut panel_state.selected_tier, i, *name);
                            }
                        });
                    ui.label("Local file:");
                    ui.text_edit_singleline(&mut state.local_path);
                    if ui.button("📁 Choose File...").clicked() {
                        if let Some(path) = FileDialog::new().pick_file() {
                            state.local_path = path.display().to_string();
                            if let Some(filename) = Path::new(&state.local_path)
                                .file_name()
                                .map(|f| f.to_string_lossy().to_string())
                            {
                                state.remote_name = filename;
                            }
                        }
                    }
                } else {
                    ui.label("Remote file:");
                    ui.text_edit_singleline(&mut state.remote_name);
                }

                ui.separator();

                if state.mode == 0 {
                    ui.label("Remote name:");
                    ui.text_edit_singleline(&mut state.remote_name);
                    ui.checkbox(&mut panel_state.encrypt, "Encrypt");
                } else {
                    ui.label("Save as:");
                    ui.text_edit_singleline(&mut state.save_as);
                    if ui.button("📂 Choose Folder...").clicked() {
                        if let Some(folder) = FileDialog::new().pick_folder() {
                            let filename = Path::new(&state.remote_name)
                                .file_name()
                                .map(|f| f.to_string_lossy().to_string())
                                .unwrap_or_else(|| "filename.extension".to_string());
                            let full_path = folder.join(filename);
                            state.save_as = full_path.display().to_string();
                        }
                    }
                    ui.checkbox(&mut panel_state.decrypt, "Decrypt");
                }

                // Password box hanya muncul jika:
                // - mode upload dan encrypt dicentang
                // - mode download dan decrypt dicentang
                if (state.mode == 0 && panel_state.encrypt) || (state.mode == 1 && panel_state.decrypt) {
                    ui.label("Password:");
                    ui.add(egui::TextEdit::singleline(&mut panel_state.password).password(true));
                }

                if state.mode == 1 {
                    ui.checkbox(&mut state.legacy, "Legacy mode");
                }

                ui.separator();

                let button_label = if state.mode == 0 { "⬆️ Upload" } else { "⬇️ Download" };
                let is_disabled = panel_state.is_processing;
                if ui.add_enabled(!is_disabled, egui::Button::new(button_label)).clicked() {
                    // Set status_raw awal jika upload
                    {
                        let mut status_raw = panel_state.status_raw.lock().unwrap();
                        if state.mode == 0 {
                            *status_raw = format!(
                                "Uploading\nLocal file: {}\nRemote file: {}\n\n",
                                state.local_path,
                                state.remote_name
                            );
                        } else {
                            *status_raw = String::new();
                        }
                    }
                    {
                        let mut flag = panel_state.processing_flag.lock().unwrap();
                        *flag = true;
                    }
                    panel_state.is_processing = true;

                    // Prepare CLI arguments (clone all needed data)
                    let status_raw = panel_state.status_raw.clone();
                    let local_path = state.local_path.clone();
                    let remote_name = state.remote_name.clone();
                    let save_as = state.save_as.clone();
                    let password = panel_state.password.clone();
                    let encrypt = panel_state.encrypt;
                    let decrypt = panel_state.decrypt;
                    let legacy = state.legacy;
                    let selected_tier = panel_state.selected_tier;
                    let mode = state.mode;

                    thread::spawn(move || {
                        use std::process::{Command, Stdio};
                        use std::io::{Read};

                        let tier_names = ["normal", "priority", "premium", "ultra", "enterprise"];
                        let mut args = if mode == 0 {
                            let mut v = vec!["upload-file", &local_path, &remote_name];
                            v.push("--tier");
                            v.push(tier_names[selected_tier]);
                            v
                        } else {
                            vec!["download-file", &remote_name, &save_as]
                        };
                        if mode == 0 && encrypt {
                            args.push("--encrypt");
                            args.push("--password");
                            args.push(&password);
                        }
                        if mode == 1 && decrypt {
                            args.push("--decrypt");
                            args.push("--password");
                            args.push(&password);
                        }
                        if mode == 1 && legacy {
                            args.push("--legacy");
                        }
                        // Always add --gui-style for GUI progress info
                        args.push("--gui-style");

                        let mut cmd = Command::new("pipe");
                        cmd.args(&args)
                            .stdout(Stdio::piped())
                            .stderr(Stdio::piped());

                        let mut child = match cmd.spawn() {
                            Ok(child) => child,
                            Err(e) => {
                                let mut status = status_raw.lock().unwrap();
                                status.push_str(&format!("❌ Failed to run CLI: {}\n", e));
                                return;
                            }
                        };

                        // Stream stdout as bytes (progress bar, etc)
                        let mut stdout = child.stdout.take().unwrap();
                        let mut stderr = child.stderr.take().unwrap();
                        let mut buf = [0u8; 4096];
                        let mut partial = String::new();
                        loop {
                            match stdout.read(&mut buf) {
                                Ok(0) => break,
                                Ok(n) => {
                                    let chunk = &buf[..n];
                                    let s = String::from_utf8_lossy(chunk);
                                    partial.push_str(&s);
                                    // Split by \r and \n
                                    let mut lines: Vec<&str> = partial.split(|c| c == '\n' || c == '\r').collect();
                                    let keep_partial = if !partial.ends_with('\n') && !partial.ends_with('\r') {
                                        lines.pop().unwrap_or_default().to_string()
                                    } else {
                                        String::new()
                                    };
                                    for line in &lines {
                                        let mut status = status_raw.lock().unwrap();
                                        if let Some(progress) = extract_progress_info(line) {
                                            let mut lines_vec: Vec<&str> = status.lines().collect();
                                            if mode == 0 {
                                                // Untuk upload, hapus semua baris yang prefix-nya [PROGRESS] atau Progress:
                                                lines_vec.retain(|l| {
                                                    let l = l.trim_start();
                                                    !l.starts_with("[PROGRESS]") && !l.starts_with("Progress:")
                                                });
                                            } else if mode == 1 {
                                                // Untuk download, hapus semua baris [PROGRESS]
                                                lines_vec.retain(|l| !l.starts_with("[PROGRESS]"));
                                            }
                                            let mut new_status = lines_vec.join("\n");
                                            if !new_status.is_empty() {
                                                new_status.push('\n');
                                            }
                                            new_status.push_str(&format!("[PROGRESS] {}", progress));
                                            *status = new_status;
                                        } else {
                                            // Jika line mengandung "PROGRESS:" tapi tidak terparse, tetap normalisasi ke [PROGRESS]
                                            if line.trim().starts_with("PROGRESS:") {
                                                let content = line.trim().trim_start_matches("PROGRESS:").trim();
                                                // Hapus semua baris progress lama juga
                                                let mut lines_vec: Vec<&str> = status.lines().collect();
                                                lines_vec.retain(|l| {
                                                    let l = l.trim_start();
                                                    !l.starts_with("[PROGRESS]") && !l.starts_with("Progress:")
                                                });
                                                let mut new_status = lines_vec.join("\n");
                                                if !new_status.is_empty() {
                                                    new_status.push('\n');
                                                }
                                                new_status.push_str(&format!("[PROGRESS] {}\n", content));
                                                *status = new_status;
                                            } else {
                                                status.push_str(line);
                                                status.push('\n');
                                            }
                                        }
                                    }
                                    partial = keep_partial;
                                }
                                Err(_) => break,
                            }
                        }
                        // Stream stderr as bytes
                        let mut partial_err = String::new();
                        loop {
                            match stderr.read(&mut buf) {
                                Ok(0) => break,
                                Ok(n) => {
                                    let chunk = &buf[..n];
                                    let s = String::from_utf8_lossy(chunk);
                                    partial_err.push_str(&s);
                                    let mut lines: Vec<&str> = partial_err.split(|c| c == '\n' || c == '\r').collect();
                                    let keep_partial = if !partial_err.ends_with('\n') && !partial_err.ends_with('\r') {
                                        lines.pop().unwrap_or_default().to_string()
                                    } else {
                                        String::new()
                                    };
                                    for line in &lines {
                                        let mut status = status_raw.lock().unwrap();
                                        status.push_str(line);
                                        status.push('\n');
                                    }
                                    partial_err = keep_partial;
                                }
                                Err(_) => break,
                            }
                        }
                    });

                    // Done, reset processing flag
                    {
                        let mut flag = panel_state.processing_flag.lock().unwrap();
                        *flag = false;
                    }
                    panel_state.is_processing = false;
                }

                // Poll upload status and show success if found in uploads list
                // Removed: do not add any upload.rs-generated status output, only show CLI output

                ui.separator();
                // Status output: show raw CLI output (with progress), same for upload & download
                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        let status_raw = panel_state.status_raw.lock().unwrap().clone();
                        let lines = status_raw.lines();
                        let mut first = true;
                        for line in lines {
                            if first {
                                ui.label(
                                    egui::RichText::new("📦 Status:").strong().color(egui::Color32::from_rgb(120, 220, 220))
                                );
                                first = false;
                            }
                            if line.trim().is_empty() { continue; }
                            if line.starts_with("[PROGRESS]") {
                                let progress = line.trim_start_matches("[PROGRESS]").trim();
                                ui.label(
                                    egui::RichText::new(format!("Progress: {}", progress))
                                        .color(egui::Color32::from_rgb(220, 200, 80))
                                        .strong()
                                );
                            } else {
                                let styled = if line.to_lowercase().contains("success") || line.to_lowercase().contains("completed") {
                                    egui::RichText::new(format!("✅ {}", line)).color(egui::Color32::GREEN)
                                } else if line.to_lowercase().contains("failed") || line.to_lowercase().contains("error") {
                                    egui::RichText::new(format!("❌ {}", line)).color(egui::Color32::RED)
                                } else {
                                    egui::RichText::new(line).color(egui::Color32::LIGHT_GRAY)
                                };
                                ui.label(styled);
                            }
                        }
                    });

// Helper: extract progress info from CLI output line
fn extract_progress_info(line: &str) -> Option<String> {
    // Support both "[PROGRESS] ..." and "PROGRESS: ..."
    let line = line.trim();
    let line = if line.starts_with("PROGRESS:") {
        &line["PROGRESS:".len()..]
    } else if line.starts_with("[PROGRESS]") {
        &line["[PROGRESS]".len()..]
    } else {
        line
    };
    // Regex: match full progress info first, fallback to percent only
    let re_full = regex::Regex::new(r"(\d+(\.\d+)?\s*(KiB|MiB|GiB|B)\s*/\s*\d+(\.\d+)?\s*(KiB|MiB|GiB|B)\s*\(\d+%?,?\s*\d+s?\))").ok()?;
    let re_percent = regex::Regex::new(r"(\d{1,3}%\s*)").ok()?;
    if let Some(cap) = re_full.captures(line) {
        cap.get(1).map(|m| m.as_str().trim().to_string())
    } else if let Some(cap) = re_percent.captures(line) {
        cap.get(1).map(|m| m.as_str().trim().to_string())
    } else {
        None
    }
}
            });
        });

        // Right column (list uploads, collapsible, responsive, scroll, search)
        let ui = &mut columns[1];
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.add_space(8.0);
                egui::CollapsingHeader::new("📋 List Uploads")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.separator();
                        ui.horizontal(|ui| {
                            ui.label("🔍 Search:");
                            ui.text_edit_singleline(&mut panel_state.search);
                            if ui.button("🔄 Refresh").clicked() {
                                panel_state.last_refresh = std::time::Instant::now() - std::time::Duration::from_secs(11);
                                // Reset upload_success so status can be polled again
                                panel_state.upload_success = false;
                            }
                        });
                        ui.separator();
                        egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
                            let uploads = panel_state.list_uploads.lock().unwrap().clone();
                            let search = panel_state.search.to_lowercase();
                            if uploads.trim() == "Loading..." {
                                ui.vertical_centered(|ui| {
                                    ui.label(egui::RichText::new("⏳ Loading uploads...").color(egui::Color32::LIGHT_BLUE).size(13.0));
                                });
                                return;
                            }
                            let mut found = false;
                            for (idx, line) in uploads.lines().enumerate() {
                                if !search.is_empty() && !line.to_lowercase().contains(&search) {
                                    continue;
                                }
                                let line = if let Some(pos) = line.find(':') {
                                    line[pos + 1..].trim()
                                } else {
                                    line.trim()
                                };
                                if line.is_empty() {
                                    continue;
                                }
                                found = true;
                                let mut local = String::new();
                                let mut remote = String::new();
                                let mut status = String::new();
                                let mut msg = String::new();
                                for part in line.split(',') {
                                    let part = part.trim();
                                    if part.starts_with("local=") {
                                        local = part.trim_start_matches("local=").trim_matches('\'').to_string();
                                    } else if part.starts_with("remote=") {
                                        remote = part.trim_start_matches("remote=").trim_matches('\'').to_string();
                                    } else if part.starts_with("status=") {
                                        status = part.trim_start_matches("status=").trim_matches('\'').to_string();
                                    } else if part.starts_with("msg=") {
                                        msg = part.trim_start_matches("msg=").trim_matches('\'').to_string();
                                    }
                                }
                                // Card style
                                ui.group(|ui| {
                                    ui.vertical(|ui| {
                                        // Highlight search
                                        ui.horizontal(|ui| {
                                            ui.label(egui::RichText::new(format!("{}: ", idx + 1)).strong().size(13.0));
                                            highlight_label(ui, &local, &panel_state.search);
                                        });
                                        ui.horizontal(|ui| {
                                            ui.label(egui::RichText::new("Remote: ").size(12.0));
                                            highlight_label(ui, &remote, &panel_state.search);
                                        });
                                        let (status_icon, status_color) = if status.to_lowercase().contains("success") || status.to_lowercase().contains("completed") {
                                            ("✅", egui::Color32::GREEN)
                                        } else if status.to_lowercase().contains("failed") || status.to_lowercase().contains("error") {
                                            ("❌", egui::Color32::RED)
                                        } else {
                                            ("⏳", egui::Color32::YELLOW)
                                        };
                                        ui.horizontal(|ui| {
                                            ui.label(egui::RichText::new(status_icon).color(status_color).size(12.0));
                                            ui.label(egui::RichText::new(format!("Status: {}", status)).color(status_color).strong().size(12.0));
                                        });
                                        if !msg.is_empty() {
                                            ui.label(egui::RichText::new(format!("Msg: {}", msg)).color(egui::Color32::LIGHT_BLUE).size(11.0));
                                        }
                                    });
                                });
                                ui.add_space(12.0);
                            }
                            if !found {
                                ui.vertical_centered(|ui| {
                                    ui.label(egui::RichText::new("No uploads found.").color(egui::Color32::GRAY).size(13.0));
                                });
                            }
                        });
                        ui.separator();
                    });
                ui.add_space(8.0);
            });
        });
    });
}

// highlight search
fn highlight_label(ui: &mut egui::Ui, text: &str, search: &str) {
    if search.is_empty() {
        ui.code(text);
        return;
    }
    let search_lower = search.to_lowercase();
    let text_lower = text.to_lowercase();
    if let Some(idx) = text_lower.find(&search_lower) {
        let before = &text[..idx];
        let matched = &text[idx..idx + search.len()];
        let after = &text[idx + search.len()..];
        ui.horizontal_wrapped(|ui| {
            ui.code(before);
            ui.label(
                egui::RichText::new(matched)
                    .background_color(egui::Color32::from_rgb(220, 220, 180))
                    .color(egui::Color32::BLACK)
                    .strong()
            );
            ui.code(after);
        });
    } else {
        ui.code(text);
    }
}