use eframe::egui;
use std::thread;
use std::sync::{Arc, Mutex};
use super::{UploadDownloadState, list};
use rfd::FileDialog;
use std::path::Path;
use super::utils::get_current_executable_path;

pub struct UploadPanelState {
    pub tier_pricing_output: Arc<Mutex<Option<String>>>,
    pub show_tier_popup: bool,
    pub selected_tier: usize, // 0: normal, 1: priority, 2: premium, 3: ultra, 4: enterprise
    pub status_upload: Arc<Mutex<String>>,
    pub status_download: Arc<Mutex<String>>,
    pub encrypt: bool,
    pub decrypt: bool,
    pub password: String,
    pub is_processing: bool,
    pub processing_flag: Arc<Mutex<bool>>,
}

impl Default for UploadPanelState {
    fn default() -> Self {
        Self {
            status_upload: Arc::new(Mutex::new(String::new())),
            status_download: Arc::new(Mutex::new(String::new())),
            encrypt: false,
            decrypt: false,
            password: String::new(),
            is_processing: false,
            processing_flag: Arc::new(Mutex::new(false)),
            selected_tier: 0,
            show_tier_popup: false,
            tier_pricing_output: Arc::new(Mutex::new(None)),
        }
    }
}

pub fn upload_download_panel(
    ui: &mut egui::Ui,
    panel_state: &mut UploadPanelState,
    state: &mut UploadDownloadState,
    api_endpoint: &str,
    list_uploads_state: &mut list::ListUploadsState,
) {
    {
        let flag = panel_state.processing_flag.lock().unwrap();
        panel_state.is_processing = *flag;
    }
    ui.ctx().request_repaint();
    list_uploads_state.refresh_if_needed();
    ui.vertical_centered(|ui| {
        ui.heading("📤 Pipe Upload/Download Panel");
        ui.separator();
    });

    ui.columns(2, |columns| {
        // Left (upload/download)
        let ui = &mut columns[0];
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.label("Mode:");
                let modes = ["Upload", "Download"];
                let old_mode = state.mode;
                ui.radio_value(&mut state.mode, 0, modes[0]);
                ui.radio_value(&mut state.mode, 1, modes[1]);
                if old_mode != state.mode && state.mode == 1 && state.save_as.is_empty() {
                    if !state.remote_name.is_empty() {
                        state.save_as = state.remote_name.clone();
                    }
                }

                ui.separator();

                if state.mode == 0 {
                    ui.horizontal(|ui| {
                        ui.label("Tier:");
                        let tier_names = ["normal", "priority", "premium", "ultra", "enterprise"];
                        egui::ComboBox::from_id_source("tier_combo")
                            .selected_text(tier_names[panel_state.selected_tier])
                            .show_ui(ui, |ui| {
                                for (i, name) in tier_names.iter().enumerate() {
                                    ui.selectable_value(&mut panel_state.selected_tier, i, *name);
                                }
                            });
                        // ingfo tier
                        if ui.button("❓").on_hover_text("Show tier pricing info").clicked() {
                            panel_state.show_tier_popup = true;
                        }
                    });
                    if panel_state.show_tier_popup {
                        if panel_state.tier_pricing_output.lock().unwrap().is_none() {
                            let output_ref = panel_state.tier_pricing_output.clone();
                            std::thread::spawn(move || {
                                let current_exe = get_current_executable_path();
                                let pricing_output = match std::process::Command::new(&current_exe)
                                    .arg("get-tier-pricing")
                                    .output() {
                                    Ok(out) => {
                                        if out.status.success() {
                                            String::from_utf8_lossy(&out.stdout).to_string()
                                        } else {
                                            format!("Failed to get pricing:\n{}", String::from_utf8_lossy(&out.stderr))
                                        }
                                    }
                                    Err(e) => format!("Failed to run CLI: {}", e),
                                };
                                let mut output_lock = output_ref.lock().unwrap();
                                *output_lock = Some(pricing_output);
                            });
                        }
                        egui::Window::new("Tier Pricing Info")
                            .collapsible(false)
                            .resizable(true)
                            .default_size([650.0, 420.0])
                            .show(ui.ctx(), |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new("📊 Upload Tier Pricing").size(22.0).strong());
                                });
                                ui.separator();
                                let pricing_output = panel_state.tier_pricing_output.lock().unwrap().clone();
                                if let Some(pricing_output) = pricing_output {
                                    ui.add(
                                        egui::TextEdit::multiline(&mut pricing_output.clone())
                                            .font(egui::TextStyle::Monospace)
                                            .desired_rows(16)
                                            .code_editor()
                                            .interactive(false)
                                            .text_color(egui::Color32::WHITE)
                                            .frame(false)
                                            .margin(egui::Vec2::splat(4.0))
                                            .layouter(&mut |ui, text, _wrap_width| {
                                                ui.fonts(|fonts| fonts.layout_no_wrap(
                                                    text.to_string(),
                                                    egui::FontId::monospace(16.0),
                                                    egui::Color32::WHITE
                                                ))
                                            })
                                    );
                                    ui.add_space(8.0);
                                   // ui.label(egui::RichText::new("Note: Current price adjusts based on demand for Priority and Premium tiers.").size(14.0).color(egui::Color32::LIGHT_BLUE));
                                } else {
                                    ui.label(egui::RichText::new("Loading pricing info...").size(16.0).color(egui::Color32::YELLOW));
                                }
                                ui.add_space(8.0);
                                if ui.button("Close").clicked() {
                                    panel_state.show_tier_popup = false;
                                    *panel_state.tier_pricing_output.lock().unwrap() = None;
                                }
                            });
                    }
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
                    ui.small("(Enter filename only to save to Downloads folder, or full path)");
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

                // bug fixes (password box only appears if encrypt/decrypt is checked)
                if (state.mode == 0 && panel_state.encrypt) || (state.mode == 1 && panel_state.decrypt) {
                    ui.label("Password:");
                    ui.add(egui::TextEdit::singleline(&mut panel_state.password).password(true));
                }

                if state.mode == 1 {
                    ui.checkbox(&mut state.legacy, "Legacy mode");
                }

                ui.separator();

                let button_label = if state.mode == 0 { "⬆ Upload" } else { "⬇ Download" };
                let password_required = (state.mode == 0 && panel_state.encrypt) || (state.mode == 1 && panel_state.decrypt);
                let password_empty = panel_state.password.trim().is_empty();
                let is_disabled = panel_state.is_processing || (password_required && password_empty);
                if ui.add_enabled(!is_disabled, egui::Button::new(button_label)).clicked() {
                    // Set status
                    if state.mode == 0 {
                        let mut status = panel_state.status_upload.lock().unwrap();
                        *status = format!(
                            "Uploading\nLocal file: {}\nRemote file: {}\n\n",
                            state.local_path,
                            state.remote_name
                        );
                    } else {
                        let mut status = panel_state.status_download.lock().unwrap();
                        *status = String::new();
                    }
                    {
                        let mut flag = panel_state.processing_flag.lock().unwrap();
                        *flag = true;
                    }
                    panel_state.is_processing = true;

                    // Prepare
                    let status_upload = panel_state.status_upload.clone();
                    let status_download = panel_state.status_download.clone();
                    let local_path = state.local_path.clone();
                    let remote_name = state.remote_name.clone();
                    let save_as = state.save_as.clone();
                    let password = panel_state.password.clone();
                    let encrypt = panel_state.encrypt;
                    let decrypt = panel_state.decrypt;
                    let legacy = state.legacy;
                    let selected_tier = panel_state.selected_tier;
                    let mode = state.mode;
                    let api_endpoint = api_endpoint.to_string();

                    thread::spawn(move || {
                        use std::process::{Command, Stdio};
                        use std::io::{Read};

                        let tier_names = ["normal", "priority", "premium", "ultra", "enterprise"];
                        
                        // Handle save_as path
                        let save_path = if mode == 1 {
                            if Path::new(&save_as).is_absolute() {
                                save_as.clone()
                            } else {
                                let downloads_dir = std::env::var("USERPROFILE")
                                    .map(|home| Path::new(&home).join("Downloads"))
                                    .unwrap_or_else(|_| Path::new(".").join("Downloads").to_path_buf());
                                
                                downloads_dir.join(&save_as).display().to_string()
                            }
                        } else {
                            String::new() // Not used for upload
                        };
                        
                        let mut args = if mode == 0 {
                            let mut v = vec!["upload-file", &local_path, &remote_name];
                            v.push("--tier");
                            v.push(tier_names[selected_tier]);
                            v
                        } else {
                            vec!["download-file", &remote_name, &save_path]
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

                        args.push("--api");
                        args.push(&api_endpoint);
                        args.push("--gui-style");

                        let current_exe = get_current_executable_path();
                        let mut cmd = Command::new(&current_exe);
                        cmd.args(&args)
                            .stdout(Stdio::piped())
                            .stderr(Stdio::piped());

                        let mut child = match cmd.spawn() {
                            Ok(child) => child,
                            Err(e) => {
                                if mode == 0 {
                                    let mut status = status_upload.lock().unwrap();
                                    status.push_str(&format!("❌ Failed to run CLI: {}\n", e));
                                } else {
                                    let mut status = status_download.lock().unwrap();
                                    status.push_str(&format!("❌ Failed to run CLI: {}\n", e));
                                }
                                return;
                            }
                        };

                        // Stream stdout as bytes
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
                                    let mut lines: Vec<&str> = partial.split(|c| c == '\n' || c == '\r').collect();
                                    let keep_partial = if !partial.ends_with('\n') && !partial.ends_with('\r') {
                                        lines.pop().unwrap_or_default().to_string()
                                    } else {
                                        String::new()
                                    };
                                    for line in &lines {
                                        if mode == 0 {
                                            let mut status = status_upload.lock().unwrap();
                                            if let Some(progress) = extract_progress_info(line) {
                                                let mut lines_vec: Vec<&str> = status.lines().collect();
                                                lines_vec.retain(|l| {
                                                    let l = l.trim_start();
                                                    !l.starts_with("[PROGRESS]") && !l.starts_with("Progress:")
                                                });
                                                let mut new_status = lines_vec.join("\n");
                                                if !new_status.is_empty() {
                                                    new_status.push('\n');
                                                }
                                                new_status.push_str(&format!("[PROGRESS] {}", progress));
                                                *status = new_status;
                                            } else {
                                                if line.trim().starts_with("PROGRESS:") {
                                                    let content = line.trim().trim_start_matches("PROGRESS:").trim();
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
                                        } else {
                                            let mut status = status_download.lock().unwrap();
                                            if let Some(progress) = extract_progress_info(line) {
                                                let mut lines_vec: Vec<&str> = status.lines().collect();
                                                lines_vec.retain(|l| !l.starts_with("[PROGRESS]"));
                                                let mut new_status = lines_vec.join("\n");
                                                if !new_status.is_empty() {
                                                    new_status.push('\n');
                                                }
                                                new_status.push_str(&format!("[PROGRESS] {}", progress));
                                                *status = new_status;
                                            } else {
                                                if line.trim().starts_with("PROGRESS:") {
                                                    let content = line.trim().trim_start_matches("PROGRESS:").trim();
                                                    let mut lines_vec: Vec<&str> = status.lines().collect();
                                                    lines_vec.retain(|l| !l.starts_with("[PROGRESS]"));
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
                                        if mode == 0 {
                                            let mut status = status_upload.lock().unwrap();
                                            status.push_str(line);
                                            status.push('\n');
                                        } else {
                                            let mut status = status_download.lock().unwrap();
                                            status.push_str(line);
                                            status.push('\n');
                                        }
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

                ui.separator();
                // raw output
                egui::Frame::none()
                    .inner_margin(egui::Margin::same(4.0))
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical()
                            .auto_shrink([false; 2])
                            .stick_to_bottom(true)
                            .max_height(180.0)
                            .show(ui, |ui| {
                                let status = if state.mode == 0 {
                                    panel_state.status_upload.lock().unwrap().clone()
                                } else {
                                    panel_state.status_download.lock().unwrap().clone()
                                };
                                let lines = status.lines();
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
                    });

// extract progress info from CLI output line
fn extract_progress_info(line: &str) -> Option<String> {
    // "[PROGRESS]" "PROGRESS:"
    let line = line.trim();
    let line = if line.starts_with("PROGRESS:") {
        &line["PROGRESS:".len()..]
    } else if line.starts_with("[PROGRESS]") {
        &line["[PROGRESS]".len()..]
    } else {
        line
    };
    // test progress
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

        // Right column uploads list
        let ui = &mut columns[1];
        ui.group(|ui| {
            list::render_uploads_list(ui, list_uploads_state);
        });
    });
}