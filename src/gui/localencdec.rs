use eframe::egui;
use std::process::Command;
use rfd::FileDialog;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;

pub struct LocalEncDecState {
    pub input_path: String,
    pub output_folder: String,
    pub password: String,
    pub output_filename: String,
    pub status_raw: Arc<Mutex<String>>,
    pub mode: usize, // 0 = Encrypt, 1 = Decrypt
    pub is_processing: bool,
    pub last_output: Option<String>,
    pub process_success: bool,
}

impl Default for LocalEncDecState {
    fn default() -> Self {
        Self {
            input_path: String::new(),
            output_folder: String::new(),
            password: String::new(),
            output_filename: String::new(),
            status_raw: Arc::new(Mutex::new(String::new())),
            mode: 0,
            is_processing: false,
            last_output: None,
            process_success: false,
        }
    }
}

pub fn local_encdec_panel(ui: &mut egui::Ui, state: &mut LocalEncDecState) {
    ui.vertical_centered(|ui| {
        ui.heading("🔒 Local Encrypt/Decrypt");
        ui.separator();
    });

    ui.horizontal(|ui| {
        let modes = ["Encrypt", "Decrypt"];
        ui.radio_value(&mut state.mode, 0, modes[0]);
        ui.radio_value(&mut state.mode, 1, modes[1]);
    });

    ui.separator();

    ui.horizontal(|ui| {
        ui.label("Input file:");
        ui.text_edit_singleline(&mut state.input_path);
        if ui.button("📁 Choose File...").clicked() {
            if let Some(path) = FileDialog::new().pick_file() {
                state.input_path = path.display().to_string();
                let input_filename = Path::new(&state.input_path)
                    .file_name()
                    .map(|f| f.to_string_lossy().to_string())
                    .unwrap_or_default();
                state.output_filename = if state.mode == 0 {
                    format!("{}.enc", input_filename)
                } else if input_filename.ends_with(".enc") {
                    input_filename.trim_end_matches(".enc").to_string()
                } else {
                    format!("decrypted_{}", input_filename)
                };
            }
        }
    });

    ui.horizontal(|ui| {
        ui.label("Output folder:");
        ui.text_edit_singleline(&mut state.output_folder);
        if ui.button("📂 Choose Folder...").clicked() {
            if let Some(folder) = FileDialog::new().pick_folder() {
                state.output_folder = folder.display().to_string();
            }
        }
        ui.label("Output filename:");
        ui.text_edit_singleline(&mut state.output_filename);
    });

    ui.separator();

    ui.label("Password:");
    ui.add(egui::TextEdit::singleline(&mut state.password).password(true));

    ui.separator();

    let button_label = if state.mode == 0 { "🔐 Encrypt" } else { "🔓 Decrypt" };

    let can_process = !state.is_processing
        && !state.input_path.is_empty()
        && !state.output_folder.is_empty()
        && !state.output_filename.is_empty()
        && !state.password.is_empty();

    let button = ui.add_enabled(can_process, egui::Button::new(button_label));

    if button.clicked() && !state.is_processing {
        if state.input_path.is_empty() || state.output_folder.is_empty() || state.output_filename.is_empty() || state.password.is_empty() {
            let mut status_raw = state.status_raw.lock().unwrap();
            *status_raw = "Input file, output folder, output filename, and password are required.\n".to_string();
            return;
        }

        state.is_processing = true;
        state.process_success = false;
        state.last_output = Some(Path::new(&state.output_folder).join(&state.output_filename).display().to_string());

        {
            let mut status_raw = state.status_raw.lock().unwrap();
            *status_raw = format!(
                "Processing...\nInput file: {}\nOutput file: {}\n",
                state.input_path,
                state.last_output.as_ref().unwrap()
            );
        }

        let status_raw = state.status_raw.clone();
        let input_path = state.input_path.clone();
        let output_folder = state.output_folder.clone();
        let output_filename = state.output_filename.clone();
        let password = state.password.clone();
        let mode = state.mode;

        thread::spawn(move || {
            let output_path = Path::new(&output_folder).join(&output_filename).display().to_string();

            let cmd = if mode == 0 {
                vec![
                    "encrypt-local",
                    &input_path,
                    &output_path,
                    "--password",
                    &password,
                ]
            } else {
                vec![
                    "decrypt-local",
                    &input_path,
                    &output_path,
                    "--password",
                    &password,
                ]
            };

            let output = Command::new("pipe")
                .args(&cmd)
                .output();

            let mut status = status_raw.lock().unwrap();
            match output {
                Ok(out) => {
                    if out.status.success() {
                        status.push_str("✅ Process completed successfully!\n");
                        status.push_str("Check your output file for results.\n");
                    } else {
                        status.push_str(&format!("❌ Process failed: {}\n", String::from_utf8_lossy(&out.stderr)));
                    }
                }
                Err(e) => {
                    status.push_str(&format!("❌ Failed to run CLI: {}\n", e));
                }
            }
        });
    }

    // Polling status: show success only after process done
    if state.is_processing {
        let status = state.status_raw.lock().unwrap().clone();
        if status.contains("✅ Process completed successfully!") || status.contains("❌") {
            state.is_processing = false;
            state.process_success = status.contains("✅");
        }
    }

    ui.separator();
    // Responsive status output
    {
        let status_raw = state.status_raw.lock().unwrap().clone();
        let lines = status_raw.lines();
        let mut first = true;
        for line in lines {
            if first {
                ui.label(
                    egui::RichText::new("📦 Status:").strong().color(egui::Color32::from_rgb(120, 220, 220))
                );
                first = false;
            }
            let styled = if line.contains("✅") {
                egui::RichText::new(line).color(egui::Color32::GREEN)
            } else if line.contains("❌") {
                egui::RichText::new(line).color(egui::Color32::RED)
            } else {
                egui::RichText::new(line).color(egui::Color32::LIGHT_GRAY)
            };
            ui.label(styled);
        }
    }
}