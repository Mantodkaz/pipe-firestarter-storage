use eframe::egui;
use std::sync::{Arc, Mutex};
use std::thread;
use std::path::PathBuf;
use std::env;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadRecord {
    pub local_path: String,
    pub remote_path: String,
    pub status: String,
    pub message: String,
    pub blake3_hash: String,
    pub file_size: u64,
    pub timestamp: DateTime<Utc>,
}

pub struct ListUploadsState {
    pub uploads: Arc<Mutex<Vec<UploadRecord>>>,
    pub last_refresh: std::time::Instant,
    pub search: String,
    pub is_loading: Arc<Mutex<bool>>,
}

impl Default for ListUploadsState {
    fn default() -> Self {
        Self {
            uploads: Arc::new(Mutex::new(Vec::new())),
            last_refresh: std::time::Instant::now(),
            search: String::new(),
            is_loading: Arc::new(Mutex::new(true)),
        }
    }
}

impl ListUploadsState {
    pub fn refresh_if_needed(&mut self) {
        let is_loading = {
            let loading = self.is_loading.lock().unwrap();
            *loading
        };
        let refresh_needed = self.last_refresh.elapsed().as_secs() > 10 || is_loading;
        
        if refresh_needed {
            let uploads = self.uploads.clone();
            let is_loading = self.is_loading.clone();
            self.last_refresh = std::time::Instant::now();
            
            thread::spawn(move || {
                {
                    let mut loading = is_loading.lock().unwrap();
                    *loading = true;
                }
                
                let result = load_uploads_from_file();
                
                {
                    let mut uploads_list = uploads.lock().unwrap();
                    *uploads_list = result;
                }
                
                {
                    let mut loading = is_loading.lock().unwrap();
                    *loading = false;
                }
            });
        }
    }

    pub fn force_refresh(&mut self) {
        self.last_refresh = std::time::Instant::now() - std::time::Duration::from_secs(11);
        let mut loading = self.is_loading.lock().unwrap();
        *loading = true;
    }
}

fn get_uploads_file_path() -> PathBuf {
    let home_dir = env::var("USERPROFILE")
        .or_else(|_| env::var("HOME"))
        .unwrap_or_else(|_| ".".to_string());
    
    PathBuf::from(home_dir).join(".pipe-cli-uploads.json")
}

fn load_uploads_from_file() -> Vec<UploadRecord> {
    let file_path = get_uploads_file_path();
    
    if !file_path.exists() {
        return Vec::new();
    }
    
    match std::fs::read_to_string(&file_path) {
        Ok(content) => {
            let mut records = Vec::new();
            
            for line in content.lines() {
                if line.trim().is_empty() {
                    continue;
                }
                
                match serde_json::from_str::<UploadRecord>(line) {
                    Ok(record) => records.push(record),
                    Err(e) => {
                        eprintln!("Failed to parse upload record: {} - Line: {}", e, line);
                    }
                }
            }
            // Sort by timestamp (newest first)
            records.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
            records
        }
        Err(e) => {
            eprintln!("Failed to read uploads file: {}", e);
            Vec::new()
        }
    }
}

fn format_file_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size_f = size as f64;
    let mut unit_index = 0;
    
    while size_f >= 1024.0 && unit_index < UNITS.len() - 1 {
        size_f /= 1024.0;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{} {}", size, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size_f, UNITS[unit_index])
    }
}

fn format_timestamp(timestamp: &DateTime<Utc>) -> String {
    timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}

pub fn render_uploads_list(ui: &mut egui::Ui, list_state: &mut ListUploadsState) {
    ui.vertical(|ui| {
        ui.add_space(8.0);
        ui.label(egui::RichText::new("📋 List Uploads").size(18.0).strong());
        ui.separator();
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label("🔍 Search:");
                ui.add(egui::TextEdit::singleline(&mut list_state.search).desired_width(120.0));
                if ui.button("🔄 Refresh").clicked() {
                    list_state.force_refresh();
                }
            });
            ui.horizontal(|ui| {
                ui.label("Sort by:");
                // newest to oldest
                if !ui.ctx().memory(|mem| mem.data.get_temp::<usize>("sort_order".into())).is_some() {
                    ui.ctx().memory_mut(|mem| mem.data.insert_temp("sort_order".into(), 0usize));
                }
                let mut sort_order = ui.ctx().memory(|mem| mem.data.get_temp::<usize>("sort_order".into())).unwrap_or(0);
                egui::ComboBox::from_id_source("sort_order_combo")
                    .selected_text(if sort_order == 0 { "Newest" } else { "Oldest" })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut sort_order, 0, "Newest");
                        ui.selectable_value(&mut sort_order, 1, "Oldest");
                    });
                ui.ctx().memory_mut(|mem| mem.data.insert_temp("sort_order".into(), sort_order));
            });
        });
        ui.separator();
        
        let is_loading = {
            let loading = list_state.is_loading.lock().unwrap();
            *loading
        };
        
        egui::CollapsingHeader::new("📋 List Uploads")
            .default_open(true)
            .show(ui, |ui| {
                egui::Frame::none()
                    .inner_margin(egui::Margin::same(4.0))
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical()
                            .auto_shrink([false; 2])
                            .max_height(420.0)
                            .show(ui, |ui| {
                                if is_loading {
                                    ui.vertical_centered(|ui| {
                                        ui.label(egui::RichText::new("⏳ Loading uploads...").color(egui::Color32::LIGHT_BLUE).size(13.0));
                                    });
                                    return;
                                }
                                
                                let uploads = list_state.uploads.lock().unwrap().clone();
                                let search = list_state.search.to_lowercase();
                                let original_search = list_state.search.clone();
                                
                                if uploads.is_empty() {
                                    ui.vertical_centered(|ui| {
                                        ui.label(egui::RichText::new("No uploads found.").color(egui::Color32::GRAY).size(13.0));
                                    });
                                    return;
                                }
                                
                                let mut filtered_uploads: Vec<_> = uploads.iter().enumerate().collect();
                                
                                // Filter by search
                                if !search.is_empty() {
                                    filtered_uploads.retain(|(_, record)| {
                                        record.local_path.to_lowercase().contains(&search) ||
                                        record.remote_path.to_lowercase().contains(&search) ||
                                        record.status.to_lowercase().contains(&search) ||
                                        record.message.to_lowercase().contains(&search) ||
                                        record.blake3_hash.contains(&original_search)
                                    });
                                }
                                
                                // Sort by order
                                let sort_order = ui.ctx().memory(|mem| mem.data.get_temp::<usize>("sort_order".into())).unwrap_or(0);
                                if sort_order == 1 {
                                    filtered_uploads.reverse(); // oldest to newest
                                }
                                
                                if filtered_uploads.is_empty() {
                                    ui.vertical_centered(|ui| {
                                        ui.label(egui::RichText::new("No uploads match search criteria.").color(egui::Color32::GRAY).size(13.0));
                                    });
                                    return;
                                }
                                
                                for (original_idx, record) in filtered_uploads {
                                    egui::Frame::group(ui.style())
                                        .inner_margin(egui::Margin::symmetric(12.0, 10.0))
                                        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(180, 180, 180)))
                                        .show(ui, |ui: &mut egui::Ui| {
                                            ui.set_min_width(ui.available_width());
                                            ui.vertical(|ui| {
                                                // Header
                                                ui.horizontal(|ui| {
                                                    ui.label(egui::RichText::new("📁 Remote: ").size(13.0).color(egui::Color32::LIGHT_GRAY));
                                                    highlight_label_bright(ui, &record.remote_path, &list_state.search);
                                                });
                                                
                                                // Local_path
                                                ui.horizontal(|ui| {
                                                    ui.label(egui::RichText::new("Local: ").size(12.0).color(egui::Color32::LIGHT_GRAY));
                                                    highlight_label_path(ui, &record.local_path, &list_state.search);
                                                });
                                                let (status_icon, status_color) = if record.status.to_lowercase().contains("success") {
                                                    ("✅", egui::Color32::GREEN)
                                                } else if record.status.to_lowercase().contains("failed") || record.status.to_lowercase().contains("error") {
                                                    ("❌", egui::Color32::RED)
                                                } else {
                                                    ("⏳", egui::Color32::YELLOW)
                                                };
                                                ui.horizontal(|ui| {
                                                    ui.label(egui::RichText::new(status_icon).color(status_color).size(12.0));
                                                    ui.label(egui::RichText::new(format!("Status: {}", record.status)).color(status_color).strong().size(12.0));
                                                });
                                                
                                                // File size & timestamp
                                                ui.horizontal(|ui| {
                                                    ui.label(egui::RichText::new("Size: ").size(11.0).color(egui::Color32::LIGHT_GRAY));
                                                    ui.label(egui::RichText::new(format_file_size(record.file_size)).size(11.0).color(egui::Color32::WHITE));
                                                    ui.separator();
                                                    ui.label(egui::RichText::new("Time: ").size(11.0).color(egui::Color32::LIGHT_GRAY));
                                                    ui.label(egui::RichText::new(format_timestamp(&record.timestamp)).size(11.0).color(egui::Color32::WHITE));
                                                });

                                                if !record.message.is_empty() {
                                                    ui.horizontal(|ui| {
                                                        ui.label(egui::RichText::new("Message: ").size(11.0).color(egui::Color32::LIGHT_GRAY));
                                                        highlight_label_small(ui, &record.message, &list_state.search);
                                                    });
                                                }
                                                
                                                // Blake3 hash
                                                ui.horizontal(|ui| {
                                                    ui.label(egui::RichText::new("Hash: ").size(11.0).color(egui::Color32::LIGHT_GRAY));
                                                    let short_hash = if record.blake3_hash.len() > 16 {
                                                        format!("{}...{}", &record.blake3_hash[..8], &record.blake3_hash[record.blake3_hash.len()-8..])
                                                    } else {
                                                        record.blake3_hash.clone()
                                                    };
                                                    highlight_label_hash(ui, &short_hash, &original_search, &record.blake3_hash);
                                                    
                                                    if ui.small_button("📋").on_hover_text("Copy full hash").clicked() {
                                                        ui.output_mut(|o| o.copied_text = record.blake3_hash.clone());
                                                        let key = egui::Id::new(format!("copy_notification_{}", original_idx));
                                                        ui.ctx().memory_mut(|mem| {
                                                            mem.data.insert_temp(key, std::time::Instant::now());
                                                        });
                                                    }
                                                    
                                                    // copy notification
                                                    let key = egui::Id::new(format!("copy_notification_{}", original_idx));
                                                    if let Some(copy_time) = ui.ctx().memory(|mem| mem.data.get_temp::<std::time::Instant>(key)) {
                                                        if copy_time.elapsed().as_secs() < 2 {
                                                            ui.label(egui::RichText::new("✅ Copied!").size(10.0).color(egui::Color32::GREEN));
                                                            ui.ctx().request_repaint();
                                                        }
                                                    }
                                                });
                                            });
                                        });
                                    ui.add_space(8.0);
                                }
                            });
                    });
            });
        ui.separator();
        ui.add_space(8.0);
    });
}

// highlight search
fn highlight_label_bright(ui: &mut egui::Ui, text: &str, search: &str) {
    if search.is_empty() {
        ui.label(egui::RichText::new(text).size(13.0).strong().color(egui::Color32::WHITE));
        return;
    }
    let search_lower = search.to_lowercase();
    let text_lower = text.to_lowercase();
    if let Some(idx) = text_lower.find(&search_lower) {
        let before = &text[..idx];
        let matched = &text[idx..idx + search.len()];
        let after = &text[idx + search.len()..];
        // settings
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(before).size(13.0).strong().color(egui::Color32::WHITE));
            ui.label(
                egui::RichText::new(matched)
                    .size(13.0)
                    .strong()
                    .background_color(egui::Color32::from_rgb(255, 200, 100))
                    .color(egui::Color32::DARK_GRAY)
            );
            ui.label(egui::RichText::new(after).size(13.0).strong().color(egui::Color32::WHITE));
        });
    } else {
        ui.label(egui::RichText::new(text).size(13.0).strong().color(egui::Color32::WHITE));
    }
}

// highlight search for file paths 
fn highlight_label_path(ui: &mut egui::Ui, text: &str, search: &str) {
    if search.is_empty() {
        ui.label(egui::RichText::new(text).size(11.0).family(egui::FontFamily::Monospace).color(egui::Color32::LIGHT_BLUE));
        return;
    }
    let search_lower = search.to_lowercase();
    let text_lower = text.to_lowercase();
    if let Some(idx) = text_lower.find(&search_lower) {
        let before = &text[..idx];
        let matched = &text[idx..idx + search.len()];
        let after = &text[idx + search.len()..];
        // settings
        ui.horizontal_wrapped(|ui| {
            ui.label(egui::RichText::new(before).size(11.0).family(egui::FontFamily::Monospace).color(egui::Color32::LIGHT_BLUE));
            ui.label(
                egui::RichText::new(matched)
                    .size(11.0)
                    .family(egui::FontFamily::Monospace)
                    .background_color(egui::Color32::from_rgb(200, 200, 120))
                    .color(egui::Color32::DARK_GRAY)
            );
            ui.label(egui::RichText::new(after).size(11.0).family(egui::FontFamily::Monospace).color(egui::Color32::LIGHT_BLUE));
        });
    } else {
        ui.label(egui::RichText::new(text).size(11.0).family(egui::FontFamily::Monospace).color(egui::Color32::LIGHT_BLUE));
    }
}

// highlight search small text
fn highlight_label_small(ui: &mut egui::Ui, text: &str, search: &str) {
    if search.is_empty() {
        ui.label(egui::RichText::new(text).size(11.0).color(egui::Color32::LIGHT_BLUE));
        return;
    }
    let search_lower = search.to_lowercase();
    let text_lower = text.to_lowercase();
    if let Some(idx) = text_lower.find(&search_lower) {
        let before = &text[..idx];
        let matched = &text[idx..idx + search.len()];
        let after = &text[idx + search.len()..];
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(before).size(11.0).color(egui::Color32::LIGHT_BLUE));
            ui.label(
                egui::RichText::new(matched)
                    .size(11.0)
                    .background_color(egui::Color32::from_rgb(200, 200, 120))
                    .color(egui::Color32::DARK_GRAY)
            );
            ui.label(egui::RichText::new(after).size(11.0).color(egui::Color32::LIGHT_BLUE));
        });
    } else {
        ui.label(egui::RichText::new(text).size(11.0).color(egui::Color32::LIGHT_BLUE));
    }
}

// highlight search for hash
fn highlight_label_hash(ui: &mut egui::Ui, display_text: &str, search: &str, full_hash: &str) {
    if search.is_empty() {
        ui.label(egui::RichText::new(display_text)
            .size(11.0)
            .family(egui::FontFamily::Monospace)
            .color(egui::Color32::LIGHT_BLUE));
        return;
    }
    
    // Hash search is case-sensitive for precision
    if full_hash.contains(search) {
        ui.label(
            egui::RichText::new(display_text)
                .size(11.0)
                .family(egui::FontFamily::Monospace)
                .background_color(egui::Color32::from_rgb(255, 200, 100))
                .color(egui::Color32::DARK_GRAY)
        );
    } else {
        ui.label(egui::RichText::new(display_text)
            .size(11.0)
            .family(egui::FontFamily::Monospace)
            .color(egui::Color32::LIGHT_BLUE));
    }
}
