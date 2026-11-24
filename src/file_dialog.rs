use egui::{Context, Vec2, Window};
use std::fs;
use std::path::{Path, PathBuf};
use crate::theme::AnimePalette;

pub struct AnimeFileDialog {
    pub is_open: bool,
    pub current_dir: PathBuf,
    pub selected_file: Option<PathBuf>,
    pub search_query: String,
    pub error_msg: Option<String>,
}

impl AnimeFileDialog {
    pub fn new() -> Self {
        let initial_dir = if Path::new("/var/log").exists() {
            PathBuf::from("/var/log")
        } else {
            std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"))
        };

        Self {
            is_open: false,
            current_dir: initial_dir,
            selected_file: None,
            search_query: String::new(),
            error_msg: None,
        }
    }

    pub fn open(&mut self) {
        self.is_open = true;
        self.selected_file = None;
        self.error_msg = None;
    }

    pub fn render(&mut self, ctx: &Context, p: &AnimePalette) -> Option<PathBuf> {
        if !self.is_open {
            return None;
        }

        let mut chosen_path: Option<PathBuf> = None;
        let mut close_dialog = false;

        Window::new("🌸 Select Log File // ログファイル選択")
            .collapsible(false)
            .resizable(true)
            .default_size(Vec2::new(580.0, 440.0))
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                // Top Bookmarks & Navigation
                ui.horizontal(|ui| {
                    if ui.button("⬆ Up").clicked() {
                        if let Some(parent) = self.current_dir.parent() {
                            self.current_dir = parent.to_path_buf();
                            self.error_msg = None;
                        }
                    }

                    if ui.button("📁 /var/log").clicked() {
                        self.current_dir = PathBuf::from("/var/log");
                        self.error_msg = None;
                    }

                    if let Ok(home) = std::env::var("HOME") {
                        if ui.button("🏠 Home").clicked() {
                            self.current_dir = PathBuf::from(home);
                            self.error_msg = None;
                        }
                    }

                    if ui.button("⚡ /tmp").clicked() {
                        self.current_dir = PathBuf::from("/tmp");
                        self.error_msg = None;
                    }
                });

                ui.separator();

                // Current Path Display
                ui.horizontal(|ui| {
                    ui.label("Path: ");
                    ui.colored_label(p.accent_cyan, self.current_dir.to_string_lossy());
                });

                // Search Filter
                ui.horizontal(|ui| {
                    ui.label("🔍 Filter:");
                    ui.text_edit_singleline(&mut self.search_query);
                });

                if let Some(ref err) = self.error_msg {
                    ui.colored_label(p.accent_red, format!("⚠️ {}", err));
                }

                ui.separator();

                // File & Directory List
                egui::ScrollArea::vertical().max_height(250.0).show(ui, |ui| {
                    match fs::read_dir(&self.current_dir) {
                        Ok(entries) => {
                            let mut dirs = Vec::new();
                            let mut files = Vec::new();

                            for entry in entries.flatten() {
                                let path = entry.path();
                                let file_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();

                                if !self.search_query.is_empty()
                                    && !file_name.to_lowercase().contains(&self.search_query.to_lowercase())
                                {
                                    continue;
                                }

                                if path.is_dir() {
                                    dirs.push((file_name, path));
                                } else {
                                    let size_str = if let Ok(meta) = entry.metadata() {
                                        format_size(meta.len())
                                    } else {
                                        "?".to_string()
                                    };
                                    files.push((file_name, path, size_str));
                                }
                            }

                            dirs.sort_by(|a, b| a.0.cmp(&b.0));
                            files.sort_by(|a, b| a.0.cmp(&b.0));

                            // Show Directories first
                            for (name, path) in dirs {
                                ui.horizontal(|ui| {
                                    if ui.button(format!("📁 {}", name)).clicked() {
                                        self.current_dir = path;
                                        self.error_msg = None;
                                    }
                                });
                            }

                            // Show Files
                            for (name, path, size) in files {
                                let is_selected = self.selected_file.as_ref() == Some(&path);
                                ui.horizontal(|ui| {
                                    let label_text = format!("📄 {:<35} ({})", name, size);
                                    let _label_color = if is_selected {
                                        p.accent_pink
                                    } else if name.ends_with(".log") {
                                        p.accent_green
                                    } else {
                                        p.text_primary
                                    };

                                    let resp = ui.selectable_label(is_selected, label_text);
                                    if resp.clicked() {
                                        self.selected_file = Some(path.clone());
                                    }
                                    if resp.double_clicked() {
                                        chosen_path = Some(path);
                                        close_dialog = true;
                                    }
                                });
                            }
                        }
                        Err(e) => {
                            ui.colored_label(
                                p.accent_red,
                                format!("Cannot read directory: {} (permission denied)", e),
                            );
                        }
                    }
                });

                ui.separator();

                // Bottom confirmation buttons
                ui.horizontal(|ui| {
                    if let Some(ref sel) = self.selected_file {
                        ui.label(format!("Selected: {}", sel.file_name().unwrap_or_default().to_string_lossy()));
                    } else {
                        ui.colored_label(p.text_muted, "No file selected");
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("❌ Cancel").clicked() {
                            close_dialog = true;
                        }

                        let can_open = self.selected_file.is_some();
                        if ui.add_enabled(can_open, egui::Button::new("🌸 Open Log")).clicked() {
                            chosen_path = self.selected_file.take();
                            close_dialog = true;
                        }
                    });
                });
            });

        if close_dialog {
            self.is_open = false;
        }

        chosen_path
    }
}

fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}
