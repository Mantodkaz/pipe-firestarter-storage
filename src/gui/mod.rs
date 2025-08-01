use eframe::egui;

pub mod upload;
pub mod localencdec;
pub mod wallet;
pub mod login;
// Panel lain bisa ditambahkan seperti: pub mod encryption; pub mod wallet; dst.

#[derive(Default)]
pub struct UploadDownloadState {
    pub local_path: String,
    pub remote_name: String,
    pub mode: usize, // 0 = Upload, 1 = Download
    pub legacy: bool,
    pub save_as: String,
}

pub struct PipeGuiApp {
    pub selected_panel: usize, // 0 = Upload/Download, 1 = Encrypt/Decrypt, 2 = Wallet, 3 = Key Management, 4 = Create Link, 5 = Update Firestarter
    pub upload_panel: upload::UploadPanelState,
    pub upload_download: UploadDownloadState,
    pub localencdec_state: localencdec::LocalEncDecState,
    pub wallet_panel: wallet::WalletPanelState,
    pub api_endpoint: String, // Endpoint API Pipe Network
    pub login_state: login::LoginState,
}

impl Default for PipeGuiApp {
    fn default() -> Self {
        Self {
            selected_panel: 0,
            upload_panel: upload::UploadPanelState::default(),
            upload_download: UploadDownloadState::default(),
            localencdec_state: localencdec::LocalEncDecState::default(),
            wallet_panel: wallet::WalletPanelState::default(),
            api_endpoint: "https://us-west-00-firestarter.pipenetwork.com".to_string(),
            login_state: login::LoginState::default(),
        }
    }
}

pub fn run_gui() {
    let options = eframe::NativeOptions::default();
    let _ = eframe::run_native(
        "Pipe Network",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Box::new(PipeGuiApp::default())
        }),
    );
}

impl eframe::App for PipeGuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Firestarter Storage");
                ui.separator();
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Endpoint input di kanan, label di kiri
                    ui.text_edit_singleline(&mut self.api_endpoint);
                    if ui.button("Reset to Default").clicked() {
                        self.api_endpoint = "https://us-west-00-firestarter.pipenetwork.com".to_string();
                    }
                });
                // Tambahkan logo, versi, dsb di sini jika perlu
            });
        });

        if self.login_state.logged_in {
            egui::SidePanel::left("sidebar").show(ctx, |ui| {
                ui.heading("Menu");
                ui.separator();
                if ui.selectable_label(self.selected_panel == 0, "Upload/Download").clicked() {
                    self.selected_panel = 0;
                }
                if ui.selectable_label(self.selected_panel == 1, "Encrypt/Decrypt").clicked() {
                    self.selected_panel = 1;
                }
                if ui.selectable_label(self.selected_panel == 2, "Wallet").clicked() {
                    self.selected_panel = 2;
                }
                if ui.selectable_label(self.selected_panel == 3, "Key Management").clicked() {
                    self.selected_panel = 3;
                }
                if ui.selectable_label(self.selected_panel == 4, "Create Link").clicked() {
                    self.selected_panel = 4;
                }
                if ui.selectable_label(self.selected_panel == 5, "Update Firestarter").clicked() {
                    self.selected_panel = 5;
                }
                // another menu items
            });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            if !self.login_state.logged_in {
                ui.centered_and_justified(|ui| {
                    login::login_panel(ui, &mut self.login_state, &self.api_endpoint);
                });
                // Check login status from status string
                let status = self.login_state.status.lock().unwrap().clone();
                if status.contains("✅ Login successful!") {
                    self.login_state.logged_in = true;
                }
            } else {
                match self.selected_panel {
                    0 => upload::upload_download_panel(
                        ui,
                        &mut self.upload_panel,
                        &mut self.upload_download,
                        &self.api_endpoint,
                    ),
                    1 => localencdec::local_encdec_panel(
                        ui,
                        &mut self.localencdec_state,
                        &self.api_endpoint,
                    ),
                    2 => wallet::wallet_panel(
                        ui,
                        &mut self.wallet_panel,
                        &self.api_endpoint,
                    ),
                    3 => {
                        ui.vertical_centered(|ui| {
                            ui.heading("Key Management Panel (coming soon)");
                        });
                    }
                    4 => {
                        ui.vertical_centered(|ui| {
                            ui.heading("Create Link Panel (coming soon)");
                        });
                    }
                    5 => {
                        ui.vertical_centered(|ui| {
                            ui.heading("Update Firestarter Panel (coming soon)");
                        });
                    }
                    _ => {
                        ui.vertical_centered(|ui| {
                            ui.heading("Pilih panel di sidebar");
                        });
                    }
                }
            }
        });
    }
}