use eframe::egui;
use std::sync::{Arc, Mutex};

pub struct LoginState {
    pub show_password: bool,
    pub username: String,
    pub password: String,
    pub status: Arc<Mutex<String>>,
    pub is_loading: Arc<Mutex<bool>>,
    pub logged_in: bool,
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
        }
    }
}

pub fn login_panel(ui: &mut egui::Ui, state: &mut LoginState, api_endpoint: &str) {
    ui.vertical_centered(|ui| {
        ui.add_space(48.0);
        ui.centered_and_justified(|ui| {
            ui.group(|ui| {
                ui.set_min_width(360.0);
                ui.vertical_centered(|ui| {
                    ui.heading(egui::RichText::new("🔑 Login to Pipe Network").size(24.0).strong());
                    ui.add_space(32.0);

                    // Username row
                    ui.horizontal(|ui| {
                        ui.label("Username :");
                        ui.add(
                            egui::TextEdit::singleline(&mut state.username)
                                .hint_text("Enter your username")
                                .desired_width(200.0),
                        );
                    });
                    ui.add_space(16.0);

                    // Password row
                    ui.horizontal(|ui| {
                        ui.label("Password :");
                        let password_edit = egui::TextEdit::singleline(&mut state.password)
                            .password(!state.show_password)
                            .hint_text("Enter your password")
                            .desired_width(200.0);
                        ui.add(password_edit);
                        if ui.button(if state.show_password { "🐵" } else { "🙈" }).clicked() {
                            state.show_password = !state.show_password;
                        }
                    });
                    ui.add_space(24.0);

                    // Login button
                    let is_loading = *state.is_loading.lock().unwrap();
                    let can_login = !is_loading && !state.username.is_empty() && !state.password.is_empty();
                    ui.horizontal(|ui| {
                        ui.add_space(60.0);
                        if ui
                            .add_enabled(
                                can_login,
                                egui::Button::new("🔓 Login")
                                    .min_size(egui::vec2(120.0, 32.0))
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
                                let output = std::process::Command::new("pipe")
                                    .arg("login")
                                    .arg(&username)
                                    .arg("--api")
                                    .arg(&api)
                                    .arg("--password")
                                    .arg(&password)
                                    .output();

                                let result = match output {
                                    Ok(out) => {
                                        if out.status.success() {
                                            "✅ Login successful!".to_string()
                                        } else {
                                            format!("❌ Login failed: {}", String::from_utf8_lossy(&out.stderr))
                                        }
                                    }
                                    Err(e) => format!("❌ Failed to run CLI: {}", e),
                                };

                                let mut status_lock = status.lock().unwrap();
                                *status_lock = result;

                                let mut loading = is_loading.lock().unwrap();
                                *loading = false;
                            });
                        }
                    });

                    ui.add_space(16.0);
                    let status = state.status.lock().unwrap().clone();
                    if !status.is_empty() {
                        ui.label(egui::RichText::new(status).size(16.0));
                    }
                });
            });
        });
    });
}
