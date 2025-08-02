use eframe::egui;

pub mod upload;
pub mod localencdec;
pub mod wallet;
pub mod login;
pub mod list;
pub mod link;
pub mod utils;

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
    pub link_state: link::CreateLinkState,
    pub api_endpoint: String, // Endpoint
    pub login_state: login::LoginState,
    pub list_uploads_state: list::ListUploadsState,
}

impl Default for PipeGuiApp {
    fn default() -> Self {
        Self {
            selected_panel: 0,
            upload_panel: upload::UploadPanelState::default(),
            upload_download: UploadDownloadState::default(),
            localencdec_state: localencdec::LocalEncDecState::default(),
            wallet_panel: wallet::WalletPanelState::default(),
            link_state: link::CreateLinkState::default(),
            api_endpoint: "https://us-west-00-firestarter.pipenetwork.com".to_string(),
            login_state: login::LoginState::default(),
            list_uploads_state: list::ListUploadsState::default(),
        }
    }
}

pub fn run_gui() {
    let mut options = eframe::NativeOptions::default();
    
    // Set window properties
    options.viewport.inner_size = Some([1200.0, 800.0].into());
    options.viewport.min_inner_size = Some([800.0, 600.0].into());
    options.viewport.resizable = Some(true);
    
    // Set window title and icon
    options.viewport.title = Some("Pipe Network - Firestarter Storage".to_string());
    
    // Try to load icon from embedded data or file
    #[cfg(windows)]
    {
        // Window icon will be handled by winres in build.rs
    }
    
    let _ = eframe::run_native(
        "Pipe Network - Firestarter Storage",
        options,
        Box::new(|cc| {
            // Configure visuals
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            
            // Set application icon if available
            #[cfg(feature = "gui")]
            {
                // Icon will be embedded via build.rs
            }
            
            Box::new(PipeGuiApp::default())
        }),
    );
}

impl eframe::App for PipeGuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Logo/Brand section
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        // Fire emoji as logo placeholder
                        ui.heading("🔥");
                        ui.vertical(|ui| {
                            ui.heading("Pipe Network");
                            ui.small("Firestarter Storage");
                        });
                    });
                });
                
                ui.separator();
                
                // API Endpoint section
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Reset Endpoint").clicked() {
                        self.api_endpoint = "https://us-west-00-firestarter.pipenetwork.com".to_string();
                    }
                    ui.add_sized([300.0, 20.0], egui::TextEdit::singleline(&mut self.api_endpoint).hint_text("API Endpoint"));
                    ui.label("Endpoint:");
                });
            });
        });

        if self.login_state.logged_in && self.login_state.has_valid_credentials {
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
            if !self.login_state.logged_in || !self.login_state.has_valid_credentials {
                ui.centered_and_justified(|ui| {
                    login::login_panel(ui, &mut self.login_state, &self.api_endpoint);
                });
                // Check login status
                let status = self.login_state.status.lock().unwrap().clone();
                if status.contains("✅ Login successful!") && self.login_state.has_valid_credentials {
                    self.login_state.logged_in = true;
                }
            } else {
                match self.selected_panel {
                    0 => upload::upload_download_panel(
                        ui,
                        &mut self.upload_panel,
                        &mut self.upload_download,
                        &self.api_endpoint,
                        &mut self.list_uploads_state,
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
                            ui.heading("Key Management (soon)");
                        });
                    }
                    4 => link::create_link_panel(
                        ui,
                        &mut self.link_state,
                        &self.api_endpoint,
                        &self.list_uploads_state.uploads,
                    ),
                    5 => {
                        ui.vertical_centered(|ui| {
                            ui.heading("Update Firestarter (soon)");
                        });
                    }
                    _ => {
                        ui.vertical_centered(|ui| {
                            ui.heading("");
                        });
                    }
                }
            }
        });
    }
}