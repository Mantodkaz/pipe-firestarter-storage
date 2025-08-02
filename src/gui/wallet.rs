use eframe::egui;
use std::sync::{Arc, Mutex};
use std::thread;
use std::process::Command;
use super::utils::get_current_executable_path;

#[derive(Default)]
pub struct WalletPanelState {
    pub sol_address: Arc<Mutex<String>>,
    pub sol_balance: Arc<Mutex<String>>,
    pub pipe_balance: Arc<Mutex<String>>,
    pub pipe_mint: Arc<Mutex<String>>,
    pub usage_report: Arc<Mutex<String>>,
    pub selected_period: String,
    pub is_loading: Arc<Mutex<bool>>,
    pub first_open: bool,
    pub swap_sol_amount: String,
    pub withdraw_sol_amount: String,
    pub withdraw_sol_pubkey: String,
    pub withdraw_pipe_amount: String,
    pub withdraw_pipe_pubkey: String,
    pub last_action_status: Arc<Mutex<String>>,
    pub withdraw_mode: WithdrawMode,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum WithdrawMode {
    SwapSolForPipe,
    WithdrawSol,
    WithdrawPipe,
}

impl Default for WithdrawMode {
    fn default() -> Self {
        WithdrawMode::SwapSolForPipe
    }
}

const PIPE_MINT: &str = "35mhJor7qTD212YXdLkB8sRzTbaYRXmTzHTCFSDP5voJ"; // pipe SPL (devnet)

fn filter_numeric(input: &mut String) {
    let filtered: String = input.chars()
        .filter(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    *input = filtered;
}

impl WalletPanelState {
    pub fn refresh_wallet_with_api(&mut self, api_endpoint: &str) {
        let sol_address = self.sol_address.clone();
        let sol_balance = self.sol_balance.clone();
        let pipe_balance = self.pipe_balance.clone();
        let pipe_mint = self.pipe_mint.clone();
        let is_loading = self.is_loading.clone();
        let api = api_endpoint.to_string();
        {
            let mut loading = is_loading.lock().unwrap();
            *loading = true;
        }
        thread::spawn(move || {
            let current_exe = get_current_executable_path();
            let sol_output = Command::new(&current_exe)
                .arg("check-sol")
                .arg("--api").arg(&api)
                .output();
            let mut address = String::from("-");
            let mut sol = String::from("-");
            if let Ok(out) = sol_output {
                let text = String::from_utf8_lossy(&out.stdout);
                for line in text.lines() {
                    if line.starts_with("Pubkey:") {
                        address = line["Pubkey:".len()..].trim().to_string();
                    }
                    if line.starts_with("SOL:") {
                        sol = line["SOL:".len()..].trim().to_string();
                    }
                }
            }
            *sol_address.lock().unwrap() = address.clone();
            *sol_balance.lock().unwrap() = sol.clone();
            let pipe_output = Command::new(&current_exe)
                .arg("check-token")
                .arg("--api").arg(&api)
                .output();
            let mut pipe = String::from("-");
            let mut mint = String::from(PIPE_MINT);
            if let Ok(out) = pipe_output {
                let text = String::from_utf8_lossy(&out.stdout);
                for line in text.lines() {
                    if line.starts_with("UI:") {
                        pipe = line["UI:".len()..].trim().to_string();
                    }
                    if line.starts_with("Mint:") {
                        mint = line["Mint:".len()..].trim().to_string();
                    }
                }
            }
            *pipe_balance.lock().unwrap() = pipe.clone();
            *pipe_mint.lock().unwrap() = mint.clone();
            let mut loading = is_loading.lock().unwrap();
            *loading = false;
        });
    }
    pub fn refresh_usage_with_api(&mut self, api_endpoint: &str) {
        let usage_report = self.usage_report.clone();
        let period = self.selected_period.clone();
        let api = api_endpoint.to_string();
        thread::spawn(move || {
            let current_exe = get_current_executable_path();
            let usage_output = Command::new(&current_exe)
                .arg("token-usage")
                .arg("-p")
                .arg(&period)
                .arg("--api").arg(&api)
                .output();
            let mut usage = String::from("No usage data.");
            if let Ok(out) = usage_output {
                let text = String::from_utf8_lossy(&out.stdout);
                let filtered: Vec<&str> = text
                    .lines()
                    .filter(|line| {
                        !line.trim().is_empty() &&
                        !line.contains("Token expired") &&
                        !line.contains("Credentials saved") &&
                        !line.contains("Token refreshed") &&
                        !line.contains("Token Usage Report")
                    })
                    .collect();
                usage = filtered.join("\n");
            }
            *usage_report.lock().unwrap() = usage;
        });
    }
    pub fn swap_sol_for_pipe_with_api(&mut self, api_endpoint: &str) {
        let amount = self.swap_sol_amount.clone();
        let status = self.last_action_status.clone();
        let api = api_endpoint.to_string();
        thread::spawn(move || {
            let current_exe = get_current_executable_path();
            let output = Command::new(&current_exe)
                .arg("swap-sol-for-pipe")
                .arg(&amount)
                .arg("--api").arg(&api)
                .output();
            let mut result = String::from("Swap failed.");
            if let Ok(out) = output {
                let text = String::from_utf8_lossy(&out.stdout);
                result = text.trim().to_string();
            }
            *status.lock().unwrap() = result;
        });
    }
    pub fn withdraw_sol_with_api(&mut self, api_endpoint: &str) {
        let amount = self.withdraw_sol_amount.clone();
        let to_pubkey = self.withdraw_sol_pubkey.clone();
        let status = self.last_action_status.clone();
        let api = api_endpoint.to_string();
        thread::spawn(move || {
            let current_exe = get_current_executable_path();
            let output = Command::new(&current_exe)
                .arg("withdraw-sol")
                .arg(&amount)
                .arg(&to_pubkey)
                .arg("--api").arg(&api)
                .output();
            let mut result = String::from("Withdraw SOL failed.");
            if let Ok(out) = output {
                let text = String::from_utf8_lossy(&out.stdout);
                result = text.trim().to_string();
            }
            *status.lock().unwrap() = result;
        });
    }
    pub fn withdraw_pipe_with_api(&mut self, api_endpoint: &str) {
        let mint = PIPE_MINT.to_string();
        let amount = self.withdraw_pipe_amount.clone();
        let to_pubkey = self.withdraw_pipe_pubkey.clone();
        let status = self.last_action_status.clone();
        let api = api_endpoint.to_string();
        thread::spawn(move || {
            let current_exe = get_current_executable_path();
            let output = Command::new(&current_exe)
                .arg("withdraw-custom-token")
                .arg(&mint)
                .arg(&amount)
                .arg(&to_pubkey)
                .arg("--api").arg(&api)
                .output();
            let mut result = String::from("Withdraw PIPE failed.");
            if let Ok(out) = output {
                let text = String::from_utf8_lossy(&out.stdout);
                result = text.trim().to_string();
            }
            *status.lock().unwrap() = result;
        });
    }
}

pub fn wallet_panel(ui: &mut egui::Ui, panel_state: &mut WalletPanelState, api_endpoint: &str) {
    if !panel_state.first_open {
        panel_state.selected_period = "30d".to_string();
        panel_state.refresh_wallet_with_api(api_endpoint);
        panel_state.refresh_usage_with_api(api_endpoint);
        panel_state.first_open = true;
    }

    ui.add_space(8.0);
    ui.columns(2, |columns| {
        // Kiri: Wallet Info + Swap & Withdraw
        columns[0].set_min_width(340.0);
        columns[0].group(|ui| {
            ui.vertical(|ui| {
                ui.heading(egui::RichText::new("Wallet Info").size(22.0).strong());
                ui.add_space(8.0);

                ui.label("Solana Address:");
                ui.label(
                    egui::RichText::new(panel_state.sol_address.lock().unwrap().as_str())
                        .monospace()
                        .color(egui::Color32::from_rgb(180, 220, 255)),
                );
                ui.add_space(4.0);

                ui.label("SOL Balance:");
                ui.label(
                    egui::RichText::new(panel_state.sol_balance.lock().unwrap().as_str())
                        .monospace()
                        .color(egui::Color32::from_rgb(255, 220, 180)),
                );
                ui.add_space(4.0);

                ui.label("PIPE Balance:");
                ui.label(
                    egui::RichText::new(panel_state.pipe_balance.lock().unwrap().as_str())
                        .monospace()
                        .color(egui::Color32::from_rgb(180, 255, 200)),
                );
                ui.add_space(4.0);

                ui.label("PIPE Token:");
                ui.label(
                    egui::RichText::new(panel_state.pipe_mint.lock().unwrap().as_str())
                        .monospace()
                        .color(egui::Color32::from_rgb(200, 200, 255)),
                );
                ui.add_space(10.0);

                if *panel_state.is_loading.lock().unwrap() {
                    ui.spinner();
                }
                if ui.button("Refresh Wallet Info").clicked() {
                    panel_state.refresh_wallet_with_api(api_endpoint);
                }

                ui.separator();
                ui.heading("Swap & Withdraw");
                ui.add_space(6.0);

                ui.horizontal(|ui| {
                    ui.radio_value(&mut panel_state.withdraw_mode, WithdrawMode::SwapSolForPipe, "Swap SOL for PIPE");
                    ui.radio_value(&mut panel_state.withdraw_mode, WithdrawMode::WithdrawSol, "Withdraw SOL");
                    ui.radio_value(&mut panel_state.withdraw_mode, WithdrawMode::WithdrawPipe, "Withdraw PIPE");
                });

                ui.add_space(8.0);

                match panel_state.withdraw_mode {
                    WithdrawMode::SwapSolForPipe => {
                        ui.horizontal(|ui| {
                            ui.label("Amount SOL:");
                            let response = ui.add(
                                egui::TextEdit::singleline(&mut panel_state.swap_sol_amount)
                                    .desired_width(60.0)
                            );
                            if response.changed() {
                                filter_numeric(&mut panel_state.swap_sol_amount);
                            }
                            if ui.button("Swap").clicked() {
                                panel_state.swap_sol_for_pipe_with_api(api_endpoint);
                                panel_state.refresh_wallet_with_api(api_endpoint);
                            }
                        });
                    }
                    WithdrawMode::WithdrawSol => {
                        ui.horizontal(|ui| {
                            ui.label("Amount SOL:");
                            let response = ui.add(
                                egui::TextEdit::singleline(&mut panel_state.withdraw_sol_amount)
                                    .desired_width(60.0)
                            );
                            if response.changed() {
                                filter_numeric(&mut panel_state.withdraw_sol_amount);
                            }
                            ui.label("To Pubkey:");
                            ui.add(
                                egui::TextEdit::singleline(&mut panel_state.withdraw_sol_pubkey)
                                    .desired_width(100.0)
                            );
                            if ui.button("Withdraw SOL").clicked() {
                                panel_state.withdraw_sol_with_api(api_endpoint);
                                panel_state.refresh_wallet_with_api(api_endpoint);
                            }
                        });
                    }
                    WithdrawMode::WithdrawPipe => {
                        ui.horizontal(|ui| {
                            ui.label("Amount PIPE:");
                            let response = ui.add(
                                egui::TextEdit::singleline(&mut panel_state.withdraw_pipe_amount)
                                    .desired_width(60.0)
                            );
                            if response.changed() {
                                filter_numeric(&mut panel_state.withdraw_pipe_amount);
                            }
                            ui.label("To Pubkey:");
                            ui.add(
                                egui::TextEdit::singleline(&mut panel_state.withdraw_pipe_pubkey)
                                    .desired_width(100.0)
                            );
                            if ui.button("Withdraw PIPE").clicked() {
                                panel_state.withdraw_pipe_with_api(api_endpoint);
                                panel_state.refresh_wallet_with_api(api_endpoint);
                            }
                        });
                    }
                }

                ui.add_space(8.0);
                let status = panel_state.last_action_status.lock().unwrap();
                if !status.is_empty() {
                    ui.label(egui::RichText::new(status.as_str()).color(egui::Color32::YELLOW));
                }
            });
        });

        // Usage Report
        columns[1].set_min_width(340.0);
        columns[1].group(|ui| {
            ui.set_min_width(340.0);
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading(
                    egui::RichText::new("Token Usage Report")
                        .size(22.0)
                        .strong()
                        .color(egui::Color32::from_rgb(180, 220, 255)),
                );
                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Token Usage Period:").strong());
                    let periods = ["7d", "30d", "90d", "365d", "all"];
                    for &p in &periods {
                        if ui.radio_value(&mut panel_state.selected_period, p.to_string(), p).changed() {
                            panel_state.refresh_usage_with_api(api_endpoint);
                        }
                    }
                });

                ui.add_space(8.0);

                let usage = panel_state.usage_report.lock().unwrap();
                render_usage_report(ui, usage.as_str());
            });
        });
    });
}

// render usage report
fn render_usage_report(ui: &mut egui::Ui, usage: &str) {
    let lines: Vec<&str> = usage.lines().collect();
    let mut section = String::new();
    let mut section_lines = vec![];
    let mut first_section = true;

    for line in lines {
        if line.contains("Storage (Uploads):") {
            if !section_lines.is_empty() {
                usage_section_card(ui, &section, &section_lines, first_section);
                first_section = false;
            }
            section = "📦 Storage (Uploads)".to_string();
            section_lines.clear();
        } else if line.contains("Bandwidth (Downloads):") {
            if !section_lines.is_empty() {
                usage_section_card(ui, &section, &section_lines, first_section);
                first_section = false;
            }
            section = "🌐 Bandwidth (Downloads)".to_string();
            section_lines.clear();
        } else if line.contains("Total:") {
            if !section_lines.is_empty() {
                usage_section_card(ui, &section, &section_lines, first_section);
                first_section = false;
            }
            section = "💰 Total".to_string();
            section_lines.clear();
        } else if !line.trim().is_empty() {
            section_lines.push(line);
        }
    }
    if !section_lines.is_empty() {
        usage_section_card(ui, &section, &section_lines, first_section);
    }
}

fn usage_section_card(ui: &mut egui::Ui, section: &str, lines: &[&str], first: bool) {
    if !first {
        ui.add_space(6.0);
    }
    ui.group(|ui| {
        ui.label(egui::RichText::new(section).strong().size(16.0));
        for l in lines {
            highlight_usage_line(ui, l);
        }
    });
}

fn highlight_usage_line(ui: &mut egui::Ui, line: &str) {
    if line.contains("PIPE") {
        let parts: Vec<&str> = line.split("PIPE").collect();
        if parts.len() > 1 {
            let before = parts[0];
            ui.horizontal(|ui| {
                ui.label(before);
                ui.label(
                    egui::RichText::new("PIPE")
                        .color(egui::Color32::from_rgb(180, 255, 200))
                        .strong(),
                );
            });
        } else {
            ui.label(line);
        }
    } else if line.contains("GB") {
        let parts: Vec<&str> = line.split("GB").collect();
        if parts.len() > 1 {
            let before = parts[0];
            ui.horizontal(|ui| {
                ui.label(before);
                ui.label(
                    egui::RichText::new("GB")
                        .color(egui::Color32::from_rgb(255, 220, 180))
                        .strong(),
                );
            });
        } else {
            ui.label(line);
        }
    } else if line.contains("Burned:") {
        ui.label(
            egui::RichText::new(line)
                .color(egui::Color32::from_rgb(255, 180, 180))
                .strong(),
        );
    } else if line.contains("Treasury:") {
        ui.label(
            egui::RichText::new(line)
                .color(egui::Color32::from_rgb(180, 180, 255)),
        );
    } else {
        ui.label(line);
    }
}