use eframe::egui;
use std::sync::{Arc, Mutex};
use std::thread;
use std::process::{Command, Stdio};
use std::io::Read;
use crate::gui::list::UploadRecord;
use super::utils::get_current_executable_path;

// file size
fn format_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = size as f64;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{} {}", size as u64, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

pub struct CreateLinkState {
    pub remote_filename: String,
    pub title: String,
    pub description: String,
    pub status: Arc<Mutex<String>>,
    pub is_processing: bool,
    pub processing_flag: Arc<Mutex<bool>>,
    pub generated_links: Arc<Mutex<Option<GeneratedLinks>>>,
    pub copy_feedback: Arc<Mutex<String>>,
}

#[derive(Debug, Clone)]
pub struct GeneratedLinks {
    pub direct_link: String,
    pub social_media_link: String,
    pub download_hash: String,
}

impl Default for CreateLinkState {
    fn default() -> Self {
        Self {
            remote_filename: String::new(),
            title: String::new(),
            description: String::new(),
            status: Arc::new(Mutex::new(String::new())),
            is_processing: false,
            processing_flag: Arc::new(Mutex::new(false)),
            generated_links: Arc::new(Mutex::new(None)),
            copy_feedback: Arc::new(Mutex::new(String::new())),
        }
    }
}

pub fn create_link_panel(ui: &mut egui::Ui, state: &mut CreateLinkState, api_endpoint: &str, uploads: &Arc<Mutex<Vec<UploadRecord>>>) {
    {
        let flag = state.processing_flag.lock().unwrap();
        state.is_processing = *flag;
    }
    
    ui.ctx().request_repaint();

    // scrollscroll
    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("🔗 Create Public Link");
                ui.separator();
                ui.add_space(8.0);
            });

            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.add_space(8.0);
                    
                    // Remote filename input
                    ui.horizontal(|ui| {
                        ui.label("📁 Remote filename:");
                        ui.add(egui::TextEdit::singleline(&mut state.remote_filename)
                            .desired_width(300.0)
                            .hint_text("Type to search/filter files, or click from list below"));
                    });
                    
                    ui.add_space(8.0);
                    
                    // List uploads 4 easy selection
                    ui.group(|ui| {
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new("📋 Available Remote Files:").size(12.0).strong());
                            ui.add_space(4.0);
                            
                            egui::ScrollArea::vertical()
                                .max_height(150.0)
                                .auto_shrink([false; 2])
                                .show(ui, |ui| {
                                    let uploads_lock = uploads.lock().unwrap();
                                    
                                    if uploads_lock.is_empty() {
                                        ui.label(egui::RichText::new("No uploads found. Please refresh the list first.").color(egui::Color32::GRAY));
                                    } else {
                                        // Filter uploads based on remote filename
                                        let search_term = state.remote_filename.to_lowercase();
                                        let filtered_uploads: Vec<_> = if search_term.is_empty() {
                                            uploads_lock.iter().collect()
                                        } else {
                                            uploads_lock.iter()
                                                .filter(|record| record.remote_path.to_lowercase().contains(&search_term))
                                                .collect()
                                        };
                                        
                                        if filtered_uploads.is_empty() && !search_term.is_empty() {
                                            ui.label(egui::RichText::new(format!("No files matching '{}'", state.remote_filename)).color(egui::Color32::YELLOW));
                                        } else {
                                            for (index, record) in filtered_uploads.iter().enumerate() {
                                                ui.horizontal(|ui| {
                                                    // Show file icon based on type
                                                    let icon = match record.remote_path.split('.').last().unwrap_or("").to_lowercase().as_str() {
                                                        "mp4" | "avi" | "mkv" | "mov" | "wmv" | "flv" | "webm" => "🎬",
                                                        "mp3" | "wav" | "flac" | "aac" | "ogg" | "m4a" => "🎵",
                                                        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "svg" | "webp" => "🖼",
                                                        "pdf" => "📄",
                                                        "txt" | "md" | "doc" | "docx" => "📝",
                                                        "zip" | "rar" | "7z" | "tar" | "gz" => "📦",
                                                        _ => "📄"
                                                    };
                                                    
                                                    // Clickable
                                                    if ui.selectable_label(false, format!("{} {}", icon, record.remote_path)).clicked() {
                                                        state.remote_filename = record.remote_path.clone();
                                                    }
                                                    
                                                    // file size
                                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                        ui.label(egui::RichText::new(format_size(record.file_size)).color(egui::Color32::GRAY).size(10.0));
                                                    });
                                                });

                                                if index < filtered_uploads.len() - 1 {
                                                    ui.separator();
                                                }
                                            }
                                        }
                                    }
                                });
                        });
                    });
                    
                    ui.add_space(8.0);
                    
                    // Title input
                    ui.horizontal(|ui| {
                        ui.label("🏷 Title (optional):");
                        ui.add(egui::TextEdit::singleline(&mut state.title)
                            .desired_width(300.0)
                            .hint_text("Custom title for social media preview"));
                    });
                    
                    ui.add_space(8.0);
                    
                    // Description
                    ui.vertical(|ui| {
                        ui.label("📝 Description (optional):");
                        ui.add(egui::TextEdit::multiline(&mut state.description)
                            .desired_rows(3)
                            .desired_width(ui.available_width() - 20.0)
                            .hint_text("Custom description for social media preview"));
                    });
                    
                    ui.add_space(12.0);
                    ui.separator();
                    ui.add_space(8.0);
                    
                    // "Create Link" button
                    let uploads_lock = uploads.lock().unwrap();
                    let remote_filename_exists = uploads_lock.iter()
                        .any(|record| record.remote_path == state.remote_filename.trim());
                    
                    let button_enabled = !state.is_processing 
                        && !state.remote_filename.trim().is_empty() 
                        && remote_filename_exists;
                    
                    ui.horizontal(|ui| {
                        let button = ui.add_enabled(button_enabled, egui::Button::new("🔗 Create Public Link"));
                        
                        if !state.remote_filename.trim().is_empty() && !remote_filename_exists {
                            ui.label(egui::RichText::new("⚠ File not found in uploads").color(egui::Color32::RED));
                        }
                        
                        if button.clicked() {
                    {
                        let mut status = state.status.lock().unwrap();
                        *status = "Creating public link...".to_string();
                    }
                    {
                        let mut link = state.generated_links.lock().unwrap();
                        *link = None;
                    }
                    {
                        let mut flag = state.processing_flag.lock().unwrap();
                        *flag = true;
                    }
                    state.is_processing = true;
                    let status = state.status.clone();
                    let processing_flag = state.processing_flag.clone();
                    let generated_links = state.generated_links.clone();
                    let remote_filename = state.remote_filename.trim().to_string();
                    let title = state.title.trim().to_string();
                    let description = state.description.trim().to_string();
                    let api_endpoint = api_endpoint.to_string();
                    
                    thread::spawn(move || {
                        let mut args = vec!["create-public-link", &remote_filename];
                        
                        // title flag
                        let title_arg;
                        if !title.is_empty() {
                            args.push("--title");
                            title_arg = title;
                            args.push(&title_arg);
                        }
                        
                        // desc flag
                        let description_arg;
                        if !description.is_empty() {
                            args.push("--description");
                            description_arg = description;
                            args.push(&description_arg);
                        }
                        
                        // endpoint flag
                        args.push("--api");
                        args.push(&api_endpoint);
                        
                        // Ex
                        let current_exe = get_current_executable_path();
                        let mut cmd = Command::new(&current_exe);
                        cmd.args(&args)
                            .stdout(Stdio::piped())
                            .stderr(Stdio::piped());
                        
                        match cmd.spawn() {
                            Ok(mut child) => {
                                let mut stdout_output = String::new();
                                let mut stderr_output = String::new();
                                
                                // stdout
                                if let Some(stdout) = child.stdout.take() {
                                    let mut stdout_reader = stdout;
                                    let _ = stdout_reader.read_to_string(&mut stdout_output);
                                }
                                
                                // stderr
                                if let Some(stderr) = child.stderr.take() {
                                    let mut stderr_reader = stderr;
                                    let _ = stderr_reader.read_to_string(&mut stderr_output);
                                }
                                
                                // process
                                match child.wait() {
                                    Ok(exit_status) => {
                                        if exit_status.success() {
                                            let combined_output = format!("{}\n{}", stdout_output, stderr_output);
                                            let mut direct_link = None;
                                            let mut social_media_link = None;
                                            let mut download_hash = None;
                                            for line in combined_output.lines() {
                                                let line = line.trim();
                                                
                                                // Look for direct link
                                                if line.starts_with("https://") && line.contains("publicDownload") && !line.contains("preview=true") {
                                                    direct_link = Some(line.to_string());
                                                }
                                                
                                                // Look for social media link  
                                                if line.starts_with("https://") && line.contains("publicDownload") && line.contains("preview=true") {
                                                    social_media_link = Some(line.to_string());
                                                }
                                                
                                                // Extract hash from publicDownload line
                                                if line.contains("publicDownload?hash=") {
                                                    if let Some(hash_start) = line.find("hash=") {
                                                        let hash_part = &line[hash_start + 5..];
                                                        let clean_hash = if let Some(hash_end) = hash_part.find(|c: char| c.is_whitespace() || c == '&' || c == '`' || c == '"' || c == '\'' || c == ')' || c == ']') {
                                                            hash_part[..hash_end].to_string()
                                                        } else {
                                                            hash_part.to_string()
                                                        };

                                                        let cleaned = clean_hash.trim_end_matches(&['`', '"', '\'', ')', ']', '\n', '\r'][..]).to_string();
                                                        if !cleaned.is_empty() {
                                                            download_hash = Some(cleaned);
                                                        }
                                                    }
                                                }
                                            }
                                            
                                            if let (Some(direct), Some(social)) = (direct_link, social_media_link) {
                                                let links = GeneratedLinks {
                                                    direct_link: direct,
                                                    social_media_link: social,
                                                    download_hash: download_hash.unwrap_or_default(),
                                                };
                                                
                                                {
                                                    let mut generated = generated_links.lock().unwrap();
                                                    *generated = Some(links);
                                                }
                                                {
                                                    let mut status_lock = status.lock().unwrap();
                                                    *status_lock = format!("✅ Public link created successfully!\n\n{}", combined_output);
                                                }
                                            } else {
                                                // Fallback: try to find any link
                                                let mut found_any_link = false;
                                                for line in combined_output.lines() {
                                                    if line.contains("https://") && line.contains("pipenetwork.com") {
                                                        if let Some(start) = line.find("https://") {
                                                            let url_part = &line[start..];
                                                            if let Some(end) = url_part.find(char::is_whitespace) {
                                                                let link = url_part[..end].to_string();
                                                                let links = GeneratedLinks {
                                                                    direct_link: link.clone(),
                                                                    social_media_link: link,
                                                                    download_hash: download_hash.unwrap_or_default(),
                                                                };
                                                                
                                                                {
                                                                    let mut generated = generated_links.lock().unwrap();
                                                                    *generated = Some(links);
                                                                }
                                                                found_any_link = true;
                                                                break;
                                                            }
                                                        }
                                                    }
                                                }
                                                
                                                {
                                                    let mut status_lock = status.lock().unwrap();
                                                    if found_any_link {
                                                        *status_lock = format!("✅ Public link created successfully!\n\n{}", combined_output);
                                                    } else {
                                                        *status_lock = format!("✅ Command executed successfully!\n\n{}", combined_output);
                                                    }
                                                }
                                            }
                                        } else {
                                            {
                                                let mut status_lock = status.lock().unwrap();
                                                *status_lock = format!("❌ Failed to create public link:\n\n{}\n{}", stdout_output, stderr_output);
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        {
                                            let mut status_lock = status.lock().unwrap();
                                            *status_lock = format!("❌ Failed to wait for command: {}", e);
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                {
                                    let mut status_lock = status.lock().unwrap();
                                    *status_lock = format!("❌ Failed to run pipe command: {}", e);
                                }
                            }
                        }
                        
                        // Reset processing flag
                        {
                            let mut flag = processing_flag.lock().unwrap();
                            *flag = false;
                        }
                    });
                        }
                        
                        if state.is_processing {
                            ui.spinner();
                            ui.label("Creating link...");
                        }
                    });
                });
            });
            
            ui.add_space(8.0);
            
            {
                let generated_links = state.generated_links.lock().unwrap();
                if let Some(ref links) = *generated_links {
                    ui.separator();
                    ui.add_space(8.0);
                    
                    ui.group(|ui| {
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new("🎉 Generated Public Links:").size(14.0).strong().color(egui::Color32::GREEN));
                            ui.add_space(8.0);
                            
                            // Direct link
                            ui.label(egui::RichText::new("📥 Direct link (for downloads/playback):").size(12.0).strong());
                            ui.horizontal(|ui| {
                                ui.add(egui::TextEdit::singleline(&mut links.direct_link.clone())
                                    .desired_width(ui.available_width() - 160.0)
                                    .interactive(false)
                                    .frame(true)
                                    .font(egui::TextStyle::Monospace));
                                
                                if ui.button("📋 Copy Direct").clicked() {
                                    ui.output_mut(|o| o.copied_text = links.direct_link.clone());
                                    let mut feedback = state.copy_feedback.lock().unwrap();
                                    *feedback = "direct".to_string();
                                    
                                    // Clear feedback after 3 seconds
                                    let feedback_copy = state.copy_feedback.clone();
                                    thread::spawn(move || {
                                        thread::sleep(std::time::Duration::from_secs(3));
                                        let mut feedback = feedback_copy.lock().unwrap();
                                        if *feedback == "direct" {
                                            *feedback = String::new();
                                        }
                                    });
                                }
                            });
                            
                            {
                                let feedback = state.copy_feedback.lock().unwrap();
                                if *feedback == "direct" {
                                    ui.label(egui::RichText::new("✅ Direct link copied to clipboard!").color(egui::Color32::GREEN).size(10.0));
                                }
                            }
                            
                            ui.add_space(8.0);
                            
                            // Social media link
                            ui.label(egui::RichText::new("📱 Social media link (for sharing):").size(12.0).strong());
                            ui.horizontal(|ui| {
                                ui.add(egui::TextEdit::singleline(&mut links.social_media_link.clone())
                                    .desired_width(ui.available_width() - 160.0)
                                    .interactive(false)
                                    .frame(true)
                                    .font(egui::TextStyle::Monospace));
                                
                                if ui.button("📋 Copy Social").clicked() {
                                    ui.output_mut(|o| o.copied_text = links.social_media_link.clone());
                                    let mut feedback = state.copy_feedback.lock().unwrap();
                                    *feedback = "social".to_string();
                                    let feedback_copy = state.copy_feedback.clone();
                                    thread::spawn(move || {
                                        thread::sleep(std::time::Duration::from_secs(3));
                                        let mut feedback = feedback_copy.lock().unwrap();
                                        if *feedback == "social" {
                                            *feedback = String::new();
                                        }
                                    });
                                }
                            });
                            
                            // Show copy feedback for social media link
                            {
                                let feedback = state.copy_feedback.lock().unwrap();
                                if *feedback == "social" {
                                    ui.label(egui::RichText::new("✅ Social media link copied to clipboard!").color(egui::Color32::GREEN).size(10.0));
                                }
                            }
                            
                            // Show hash
                            if !links.download_hash.is_empty() {
                                ui.add_space(8.0);
                                ui.label(egui::RichText::new("🔑 Download hash:").size(12.0).strong());
                                ui.horizontal(|ui| {
                                    ui.add(egui::TextEdit::singleline(&mut links.download_hash.clone())
                                        .desired_width(ui.available_width() - 160.0)
                                        .interactive(false)
                                        .frame(true)
                                        .font(egui::TextStyle::Monospace));
                                    
                                    if ui.button("📋 Copy Hash").clicked() {
                                        ui.output_mut(|o| o.copied_text = links.download_hash.clone());
                                        let mut feedback = state.copy_feedback.lock().unwrap();
                                        *feedback = "hash".to_string();
                                        let feedback_copy = state.copy_feedback.clone();
                                        thread::spawn(move || {
                                            thread::sleep(std::time::Duration::from_secs(3));
                                            let mut feedback = feedback_copy.lock().unwrap();
                                            if *feedback == "hash" {
                                                *feedback = String::new();
                                            }
                                        });
                                    }
                                });
                                {
                                    let feedback = state.copy_feedback.lock().unwrap();
                                    if *feedback == "hash" {
                                        ui.label(egui::RichText::new("✅ Hash copied to clipboard!").color(egui::Color32::GREEN).size(10.0));
                                    }
                                }
                            }
                        });
                    });
                    
                    ui.add_space(8.0);
                }
            }
            ui.add_space(8.0);
        });
}
