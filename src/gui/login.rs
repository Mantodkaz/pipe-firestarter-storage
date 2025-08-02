use eframe::egui;
use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use std::env;
use super::utils::get_current_executable_path;

pub struct LoginState {
    pub show_password: bool,
    pub username: String,
    pub password: String,
    pub status: Arc<Mutex<String>>,
    pub is_loading: Arc<Mutex<bool>>,
    pub logged_in: bool,
    pub has_valid_credentials: bool,
    pub show_import_options: bool,
    pub json_text: String,
    pub show_new_user_form: bool,
    pub new_user_username: String,
    pub new_user_password: String,
    pub new_user_confirm_password: String,
}

impl Default for LoginState {
    fn default() -> Self {
        Self {
            username: String::new(),
            password: String::new(),
            status: Arc::new(Mutex::new(String::new())),
            is_loading: Arc::new(Mutex::new(false)),
            logged_in: false,
            show_password: false,
            has_valid_credentials: false,
            show_import_options: false,
            json_text: String::new(),
            show_new_user_form: false,
            new_user_username: String::new(),
            new_user_password: String::new(),
            new_user_confirm_password: String::new(),
        }
    }
}

pub fn login_panel(ui: &mut egui::Ui, state: &mut LoginState, api_endpoint: &str) {
    let available_rect = ui.available_rect_before_wrap();
    let available_height = available_rect.height();
    let available_width = available_rect.width();
    let base_spacing = (available_height * 0.02).max(4.0).min(16.0);
    let header_spacing = (available_height * 0.04).max(8.0).min(32.0);
    let group_width = (available_width * 0.8).max(300.0).min(500.0);
    
    ui.vertical_centered(|ui| {
        ui.add_space(base_spacing);
        ui.centered_and_justified(|ui| {
            ui.group(|ui| {
                ui.set_min_width(group_width);
                ui.set_max_width(group_width);
                ui.vertical_centered(|ui| {
                    // header
                    let header_size = (available_height * 0.03).max(16.0).min(24.0);
                    ui.heading(egui::RichText::new("🔑 Login to Pipe Network").size(header_size).strong());
                    ui.add_space(header_spacing);

                    // Check for existing credentials
                    let config_path = get_pipe_config_path();
                    let config_exists = config_path.exists();
                    // Update credentials status
                    state.has_valid_credentials = config_exists && validate_credentials(&config_path);
                    let input_width = (group_width * 0.6).max(150.0).min(250.0);
                    
                    // Username
                    ui.horizontal(|ui| {
                        ui.label("Username :");
                        ui.add(
                            egui::TextEdit::singleline(&mut state.username)
                                .hint_text("Enter your username")
                                .desired_width(input_width),
                        );
                    });
                    ui.add_space(base_spacing);

                    // Password
                    ui.horizontal(|ui| {
                        ui.label("Password :");
                        let password_edit = egui::TextEdit::singleline(&mut state.password)
                            .password(!state.show_password)
                            .hint_text("Enter your password")
                            .desired_width(input_width);
                        ui.add(password_edit);
                        if ui.button(if state.show_password { "🐵" } else { "🙈" }).clicked() {
                            state.show_password = !state.show_password;
                        }
                    });
                    ui.add_space(header_spacing);

                    // Login button
                    let is_loading = *state.is_loading.lock().unwrap();
                    let can_login = !is_loading && !state.username.is_empty() && !state.password.is_empty();
                    ui.horizontal(|ui| {
                        ui.add_space((group_width * 0.1).max(10.0).min(30.0));
                        let button_size = egui::vec2((group_width * 0.25).max(80.0).min(100.0), 28.0);
                        
                        // Login button
                        if ui
                            .add_enabled(
                                can_login,
                                egui::Button::new("🔓 Login")
                                    .min_size(button_size)
                                    .frame(true),
                            )
                            .clicked()
                        {
                            {
                                let mut loading = state.is_loading.lock().unwrap();
                                *loading = true;
                            }

                            let username = state.username.clone();
                            let password = state.password.clone();
                            let status = state.status.clone();
                            let api = api_endpoint.to_string();
                            let is_loading = state.is_loading.clone();

                            std::thread::spawn(move || {
                                // cek
                                let config_path = get_pipe_config_path();
                                let config_existed_before = config_path.exists();
                                
                                let result = {
                                    let current_exe = get_current_executable_path();
                                    let output = std::process::Command::new(&current_exe)
                                        .arg("login")
                                        .arg(&username)
                                        .arg("--api")
                                        .arg(&api)
                                        .arg("--password")
                                        .arg(&password)
                                        .output();

                                    match output {
                                        Ok(out) => {
                                            if out.status.success() {
                                                let config_exists_now = config_path.exists();
                                                let has_valid_creds = config_exists_now && validate_credentials(&config_path);
                                                
                                                if has_valid_creds {
                                                    if !config_existed_before {
                                                        "✅ Login successful! Valid credentials file created.".to_string()
                                                    } else {
                                                        "✅ Login successful! Credentials updated.".to_string()
                                                    }
                                                } else if config_exists_now {
                                                    "⚠ Login succeeded but credentials are incomplete. You may need legacy credentials (user_id and user_app_key).".to_string()
                                                } else {
                                                    format!(
                                                        "Login succeeded, but no credentials file.\n
                                                        No pipe CLI configuration found at: {}\n
                                                        ❌ You cannot use features without valid credentials.",
                                                        config_path.display()
                                                    )
                                                }
                                            } else {
                                                format!("❌ Login failed: {}", String::from_utf8_lossy(&out.stderr))
                                            }
                                        }
                                        Err(e) => format!("❌ Failed to run CLI: {}", e),
                                    }
                                };

                                let mut status_lock = status.lock().unwrap();
                                *status_lock = result;
                                let mut loading = is_loading.lock().unwrap();
                                *loading = false;
                            });
                        }
                        
                        ui.add_space((group_width * 0.05).max(5.0).min(15.0));
                        
                        // New User
                        if ui
                            .add_enabled(
                                !is_loading,
                                egui::Button::new("👤 Create New User")
                                    .min_size(button_size)
                                    .frame(true),
                            )
                            .clicked()
                        {
                            state.show_new_user_form = !state.show_new_user_form;
                            if state.show_new_user_form {
                                state.new_user_username.clear();
                                state.new_user_password.clear();
                                state.new_user_confirm_password.clear();
                            }
                        }
                    });

                    // New User Form
                    if state.show_new_user_form {
                        ui.add_space(header_spacing);
                        ui.separator();
                        ui.add_space(base_spacing);
                        
                        ui.vertical_centered(|ui| {
                            ui.heading(egui::RichText::new("👤 Create New User").size((available_height * 0.025).max(14.0).min(18.0)));
                        });
                        ui.add_space(base_spacing);
                        
                        // New User Username
                        ui.horizontal(|ui| {
                            ui.label("Username :");
                            ui.add(
                                egui::TextEdit::singleline(&mut state.new_user_username)
                                    .hint_text("Enter desired username")
                                    .desired_width(input_width),
                            );
                        });
                        ui.add_space(base_spacing);

                        // New User pass
                        ui.horizontal(|ui| {
                            ui.label("Password :");
                            ui.add(
                                egui::TextEdit::singleline(&mut state.new_user_password)
                                    .password(true)
                                    .hint_text("Enter password")
                                    .desired_width(input_width),
                            );
                        });
                        ui.add_space(base_spacing);

                        // Confirm pass
                        ui.horizontal(|ui| {
                            ui.label("Confirm  :");
                            ui.add(
                                egui::TextEdit::singleline(&mut state.new_user_confirm_password)
                                    .password(true)
                                    .hint_text("Confirm password")
                                    .desired_width(input_width),
                            );
                        });
                        ui.add_space(base_spacing);

                        // Password validation
                        if !state.new_user_password.is_empty() && !state.new_user_confirm_password.is_empty() {
                            if state.new_user_password != state.new_user_confirm_password {
                                ui.vertical_centered(|ui| {
                                    ui.label(egui::RichText::new("❌ Passwords do not match").color(egui::Color32::RED).size(12.0));
                                });
                                ui.add_space(base_spacing);
                            }
                        }
                        let can_create = !is_loading && 
                                       !state.new_user_username.is_empty() && 
                                       !state.new_user_password.is_empty() && 
                                       !state.new_user_confirm_password.is_empty() &&
                                       state.new_user_password == state.new_user_confirm_password;
                        
                        ui.horizontal(|ui| {
                            ui.add_space((group_width * 0.1).max(10.0).min(30.0));
                            let button_size = egui::vec2((group_width * 0.25).max(80.0).min(100.0), 28.0);
                            
                            if ui
                                .add_enabled(
                                    can_create,
                                    egui::Button::new("✨ Create User")
                                        .min_size(button_size)
                                        .frame(true),
                                )
                                .clicked()
                            {
                                {
                                    let mut loading = state.is_loading.lock().unwrap();
                                    *loading = true;
                                }

                                let username = state.new_user_username.clone();
                                let password = state.new_user_password.clone();
                                let status = state.status.clone();
                                let api = api_endpoint.to_string();
                                let is_loading = state.is_loading.clone();

                                std::thread::spawn(move || {
                                    let result = {
                                        let current_exe = get_current_executable_path();
                                        let output = std::process::Command::new(&current_exe)
                                            .arg("new-user")
                                            .arg(&username)
                                            .arg("--api")
                                            .arg(&api)
                                            .arg("--password")
                                            .arg(&password)
                                            .output();

                                        match output {
                                            Ok(out) => {
                                                if out.status.success() {
                                                    "✅ User created successfully! You can now login.".to_string()
                                                } else {
                                                    format!("❌ Failed to create user: {}", String::from_utf8_lossy(&out.stderr))
                                                }
                                            }
                                            Err(e) => format!("❌ Failed to run CLI: {}", e),
                                        }
                                    };

                                    let mut status_lock = status.lock().unwrap();
                                    *status_lock = result;

                                    let mut loading = is_loading.lock().unwrap();
                                    *loading = false;
                                });
                            }
                            
                            ui.add_space((group_width * 0.05).max(5.0).min(15.0));
                            
                            if ui
                                .add_enabled(
                                    !is_loading,
                                    egui::Button::new("❌ Cancel")
                                        .min_size(button_size)
                                        .frame(true),
                                )
                                .clicked()
                            {
                                state.show_new_user_form = false;
                                state.new_user_username.clear();
                                state.new_user_password.clear();
                                state.new_user_confirm_password.clear();
                            }
                        });
                        ui.add_space(base_spacing);
                    }

                    ui.add_space(base_spacing);
                    let status = state.status.lock().unwrap().clone();
                    if !status.is_empty() {
                        let status_size = (available_height * 0.02).max(12.0).min(16.0);
                        ui.vertical_centered(|ui| {
                            ui.label(egui::RichText::new(&status).size(status_size));
                        });
                        
                        // no json file
                        if status.contains("Login succeeded, but no credentials file") || 
                           status.contains("⚠ Login succeeded but no credentials file was created.") ||
                           status.contains("You cannot use features without valid credentials") ||
                           status.contains("No pipe CLI configuration found") {
                            state.show_import_options = true;
                        }

                        if state.show_import_options && !state.has_valid_credentials {
                            if status.contains("✅ Credentials imported successfully!") || 
                               status.contains("✅ Credentials saved successfully!") {
                                state.show_import_options = false;
                            }
                        }
                        
                        if status.contains("✅ User created successfully!") {
                            state.show_new_user_form = false;
                            state.new_user_username.clear();
                            state.new_user_password.clear();
                            state.new_user_confirm_password.clear();
                        }
                    }

                    if state.show_import_options {
                        ui.add_space(base_spacing);
                        ui.separator();
                        ui.add_space(base_spacing);
                        
                        let import_header_size = (available_height * 0.025).max(14.0).min(18.0);
                        ui.vertical_centered(|ui| {
                            ui.heading(egui::RichText::new("📁 Import Credentials").size(import_header_size));
                        });
                        ui.add_space(base_spacing);
                        
                        // Compact layout import options
                        egui::ScrollArea::vertical()
                            .max_height(available_height * 0.4)
                            .show(ui, |ui| {
                                // Option 1: Import from file
                                ui.vertical_centered(|ui| {
                                    let button_size = egui::vec2((group_width * 0.5).max(150.0).min(200.0), 24.0);
                                    if ui.add(egui::Button::new("📂 Import .pipe-cli.json file").min_size(button_size)).clicked() {
                                        if let Some(path) = rfd::FileDialog::new()
                                            .add_filter("JSON files", &["json"])
                                            .set_file_name(".pipe-cli.json")
                                            .pick_file()
                                        {
                                            match std::fs::read_to_string(&path) {
                                                Ok(content) => {
                                                    if validate_json_credentials(&content) {
                                                        if save_credentials_file(&content) {
                                                            let mut status_lock = state.status.lock().unwrap();
                                                            *status_lock = "✅ Credentials imported successfully!".to_string();
                                                            state.show_import_options = false;
                                                            state.has_valid_credentials = true;
                                                        } else {
                                                            let mut status_lock = state.status.lock().unwrap();
                                                            *status_lock = "❌ Failed to save credentials file.".to_string();
                                                        }
                                                    } else {
                                                        let mut status_lock = state.status.lock().unwrap();
                                                        *status_lock = "❌ Invalid Credentials File.".to_string();
                                                    }
                                                }
                                                Err(_) => {
                                                    let mut status_lock = state.status.lock().unwrap();
                                                    *status_lock = "❌ Failed to read the selected file.".to_string();
                                                }
                                            }
                                        }
                                    }
                                });
                                
                                ui.add_space(base_spacing);
                                ui.vertical_centered(|ui| {
                                    ui.label("Or paste\nJSON content:");
                                });
                                
                                // paste options
                                let text_height = (available_height * 0.2).max(80.0).min(120.0);
                                ui.add(
                                    egui::TextEdit::multiline(&mut state.json_text)
                                        .hint_text("Paste your .pipe-cli.json content here...")
                                        .desired_rows((text_height / 20.0) as usize)
                                        .desired_width(group_width - 20.0)
                                        .code_editor()
                                );
                                
                                ui.add_space(base_spacing);
                                ui.vertical_centered(|ui| {
                                    ui.horizontal(|ui| {
                                        let button_size = egui::vec2(100.0, 24.0);
                                        if ui.add(egui::Button::new("💾 Save").min_size(button_size)).clicked() {
                                        if !state.json_text.trim().is_empty() {
                                            if validate_json_credentials(&state.json_text) {
                                                if save_credentials_file(&state.json_text) {
                                                    let mut status_lock = state.status.lock().unwrap();
                                                    *status_lock = "✅ Credentials saved successfully!".to_string();
                                                    state.show_import_options = false;
                                                    state.has_valid_credentials = true;
                                                    state.json_text.clear();
                                                } else {
                                                    let mut status_lock = state.status.lock().unwrap();
                                                    *status_lock = "❌ Failed to save credentials file.".to_string();
                                                }
                                            } else {
                                                let mut status_lock = state.status.lock().unwrap();
                                                *status_lock = "❌ Invalid JSON format or missing required fields (user_id, user_app_key).".to_string();
                                            }
                                        }
                                    }
                                    
                                    if ui.add(egui::Button::new("❌ Cancel").min_size(button_size)).clicked() {
                                        state.show_import_options = false;
                                        state.json_text.clear();
                                    }
                                });
                                });
                            });
                    }
                });
            });
        });
    });
}

// test
fn get_pipe_config_path() -> PathBuf {
    let home_dir = env::var("USERPROFILE")
        .or_else(|_| env::var("HOME"))
        .unwrap_or_else(|_| ".".to_string());
    
    PathBuf::from(home_dir).join(".pipe-cli.json")
}

fn validate_credentials(config_path: &PathBuf) -> bool {
    if !config_path.exists() {
        return false;
    }
    
    match std::fs::read_to_string(config_path) {
        Ok(content) => validate_json_credentials(&content),
        Err(_) => false,
    }
}

fn validate_json_credentials(json_content: &str) -> bool {
    match serde_json::from_str::<serde_json::Value>(json_content) {
        Ok(json) => {
            let has_user_id = json.get("user_id").is_some_and(|v| v.is_string() && !v.as_str().unwrap().is_empty());
            let has_user_app_key = json.get("user_app_key").is_some_and(|v| v.is_string() && !v.as_str().unwrap().is_empty());
            has_user_id && has_user_app_key
        }
        Err(_) => false,
    }
}

fn save_credentials_file(json_content: &str) -> bool {
    let config_path = get_pipe_config_path();
    if let Some(parent) = config_path.parent() {
        if let Err(_) = std::fs::create_dir_all(parent) {
            return false;
        }
    }
    
    // Save JSON content
    match std::fs::write(&config_path, json_content) {
        Ok(_) => true,
        Err(_) => false,
    }
}
