use egui::{
    vec2, Align, CentralPanel, Color32, Context, FontId, Layout,
    Pos2, RichText, Rounding, ScrollArea, Stroke, TextEdit, TextStyle, TopBottomPanel,
};
use regex::Regex;
use std::collections::HashSet;
use std::path::PathBuf;
use std::time::Instant;

use crate::file_dialog::AnimeFileDialog;
use crate::log_entry::{LogLevel, LogEntry};
use crate::log_source::{LogSourceManager, SourceKind};
use crate::mascot::AnimeMascot;
use crate::particles::ParticleSystem;
use crate::theme::{configure_egui_style, ThemeMode};

pub struct NekoLogApp {
    // Theme & Visuals
    theme_mode: ThemeMode,
    particles: ParticleSystem,
    mascot: AnimeMascot,

    // Logs & Filter state
    entries: Vec<LogEntry>,
    filtered_indices: Vec<usize>,
    selected_entry_idx: Option<usize>,
    search_query: String,
    use_regex: bool,
    level_filters: HashSet<LogLevel>,
    selected_service_filter: Option<String>,
    auto_scroll_to_bottom: bool,

    // Log Source
    log_mgr: LogSourceManager,
    discovered_logs: Vec<PathBuf>,
    file_dialog: AnimeFileDialog,

    // Statistics & Timing
    error_count: usize,
    warn_count: usize,
    last_frame_time: Instant,
    total_time: f64,
    notification: Option<(String, f32)>, // (message, remaining_seconds)
    lines_per_second: f32,
    lines_received_in_second: usize,
    second_timer: f32,
}

impl NekoLogApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let theme_mode = ThemeMode::SakuraCyberpunk;
        configure_egui_style(&cc.egui_ctx, theme_mode);

        let mut level_filters = HashSet::new();
        level_filters.insert(LogLevel::Fatal);
        level_filters.insert(LogLevel::Critical);
        level_filters.insert(LogLevel::Error);
        level_filters.insert(LogLevel::Warn);
        level_filters.insert(LogLevel::Notice);
        level_filters.insert(LogLevel::Info);
        level_filters.insert(LogLevel::Debug);
        level_filters.insert(LogLevel::Trace);
        level_filters.insert(LogLevel::Unknown);

        let discovered_logs = LogSourceManager::discover_system_logs();
        let mut log_mgr = LogSourceManager::new();

        // Default to journalctl if available, or first readable log, or anime demo
        let initial_source = SourceKind::JournalAll;
        log_mgr.switch_source(initial_source, true);

        Self {
            theme_mode,
            particles: ParticleSystem::new(),
            mascot: AnimeMascot::new(),

            entries: Vec::new(),
            filtered_indices: Vec::new(),
            selected_entry_idx: None,
            search_query: String::new(),
            use_regex: false,
            level_filters,
            selected_service_filter: None,
            auto_scroll_to_bottom: true,

            log_mgr,
            discovered_logs,
            file_dialog: AnimeFileDialog::new(),

            error_count: 0,
            warn_count: 0,
            last_frame_time: Instant::now(),
            total_time: 0.0,
            notification: Some(("🌸 Welcome to NekoLog! Streaming live system logs nya~ ✨".into(), 5.0)),
            lines_per_second: 0.0,
            lines_received_in_second: 0,
            second_timer: 0.0,
        }
    }

    fn recompute_filters(&mut self) {
        let regex_opt = if self.use_regex && !self.search_query.is_empty() {
            Regex::new(&format!("(?i){}", self.search_query)).ok()
        } else {
            None
        };

        let query_lower = self.search_query.to_lowercase();

        self.filtered_indices.clear();
        for (idx, entry) in self.entries.iter().enumerate() {
            // Level filter
            if !self.level_filters.contains(&entry.level) {
                continue;
            }

            // Service filter
            if let Some(ref svc) = self.selected_service_filter {
                if !entry.service.eq_ignore_ascii_case(svc) {
                    continue;
                }
            }

            // Search query filter
            if !self.search_query.is_empty() {
                if let Some(ref re) = regex_opt {
                    if !re.is_match(&entry.message) && !re.is_match(&entry.service) {
                        continue;
                    }
                } else {
                    let msg_match = entry.message.to_lowercase().contains(&query_lower);
                    let svc_match = entry.service.to_lowercase().contains(&query_lower);
                    let raw_match = entry.raw.to_lowercase().contains(&query_lower);
                    if !msg_match && !svc_match && !raw_match {
                        continue;
                    }
                }
            }

            self.filtered_indices.push(idx);
        }
    }

    fn export_logs_json(&mut self) {
        let logs_to_export: Vec<&LogEntry> = self
            .filtered_indices
            .iter()
            .filter_map(|&idx| self.entries.get(idx))
            .collect();

        if let Ok(json_str) = serde_json::to_string_pretty(&logs_to_export) {
            let export_path = "nekolog_export.json";
            if std::fs::write(export_path, json_str).is_ok() {
                self.notification = Some((
                    format!("🌸 Exported {} logs to {}! ✨", logs_to_export.len(), export_path),
                    5.0,
                ));
            } else {
                self.notification = Some(("⚠️ Failed to write export file".into(), 4.0));
            }
        }
    }

    fn export_logs_text(&mut self) {
        let mut out = String::new();
        for &idx in &self.filtered_indices {
            if let Some(entry) = self.entries.get(idx) {
                out.push_str(&entry.raw);
                out.push('\n');
            }
        }

        let export_path = "nekolog_export.txt";
        if std::fs::write(export_path, out).is_ok() {
            self.notification = Some((
                format!("📄 Exported {} lines to {}! ✨", self.filtered_indices.len(), export_path),
                5.0,
            ));
        } else {
            self.notification = Some(("⚠️ Failed to write export file".into(), 4.0));
        }
    }
}

impl eframe::App for NekoLogApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        let now = Instant::now();
        let dt = (now - self.last_frame_time).as_secs_f32().min(0.1);
        self.last_frame_time = now;
        self.total_time += dt as f64;

        // Animate particles & background
        self.particles.update(dt, ctx.screen_rect(), self.total_time);

        // Update notification timer
        if let Some((_, ref mut timer)) = self.notification {
            *timer -= dt;
            if *timer <= 0.0 {
                self.notification = None;
            }
        }

        // Velocity tracking
        self.second_timer += dt;
        if self.second_timer >= 1.0 {
            self.lines_per_second = self.lines_received_in_second as f32 / self.second_timer;
            self.lines_received_in_second = 0;
            self.second_timer = 0.0;
        }

        // Poll new log entries from background worker
        let new_entries = self.log_mgr.poll_new_entries();
        if !new_entries.is_empty() {
            self.lines_received_in_second += new_entries.len();
            for entry in new_entries {
                if matches!(entry.level, LogLevel::Error | LogLevel::Critical | LogLevel::Fatal) {
                    self.error_count += 1;
                } else if entry.level == LogLevel::Warn {
                    self.warn_count += 1;
                }
                self.entries.push(entry);
            }
            self.recompute_filters();
        }

        // Update mascot state
        self.mascot.update(dt, self.error_count, self.warn_count, self.entries.len());

        let p = self.theme_mode.palette();

        // 1. TOP HEADER PANEL (Responsive multi-row layout without overlapping)
        TopBottomPanel::top("top_header_panel")
            .frame(egui::Frame::none().fill(p.bg_panel).inner_margin(egui::Margin::symmetric(14.0, 8.0)))
            .show(ctx, |ui| {
                // Row 1: Brand & Visual Theme Controls
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("🌸 NEKO-LOG // ログニャン")
                            .font(FontId::proportional(17.0))
                            .color(p.accent_pink)
                            .strong(),
                    );

                    // Show subtitle only if width is sufficient to avoid crowding
                    if ui.available_width() > 420.0 {
                        ui.label(
                            RichText::new("Linux System Log Inspector")
                                .font(FontId::proportional(11.5))
                                .color(p.text_muted),
                        );
                    }

                    // Push theme and particle controls to the right if space allows
                    let right_controls_width = 300.0;
                    if ui.available_width() > right_controls_width {
                        ui.add_space(ui.available_width() - right_controls_width);
                    }

                    ui.horizontal(|ui| {
                        // Theme Mode Selector
                        egui::ComboBox::from_id_salt("theme_selector")
                            .selected_text(self.theme_mode.name())
                            .show_ui(ui, |ui| {
                                for theme in ThemeMode::ALL {
                                    if ui.selectable_value(&mut self.theme_mode, theme, theme.name()).clicked() {
                                        configure_egui_style(ctx, self.theme_mode);
                                    }
                                }
                            });

                        // Particle Toggle
                        let petal_label = if self.particles.enabled {
                            "🌸 Petals: ON"
                        } else {
                            "🌸 Petals: OFF"
                        };
                        if ui.button(petal_label).clicked() {
                            self.particles.enabled = !self.particles.enabled;
                        }
                    });
                });

                ui.add_space(5.0);

                // Row 2: Stream Controls & Export Actions (horizontal_wrapped prevents ANY collision/overlap on resize!)
                ui.horizontal_wrapped(|ui| {
                    // Animated Live Status Beacon
                    let is_streaming = self.log_mgr.is_streaming;
                    let pulse = ((self.total_time * 4.0).sin().abs()) as f32;
                    let beacon_color = if is_streaming {
                        Color32::from_rgb(
                            (80.0 + 175.0 * pulse) as u8,
                            250,
                            (120.0 + 130.0 * pulse) as u8,
                        )
                    } else {
                        Color32::from_rgb(255, 120, 120)
                    };

                    let status_badge = if is_streaming {
                        "● LIVE STREAMING"
                    } else {
                        "■ PAUSED"
                    };
                    ui.label(
                        RichText::new(status_badge)
                            .font(FontId::proportional(11.5))
                            .color(beacon_color)
                            .strong(),
                    );

                    // Live Follow Toggle Button
                    let follow_label = if self.log_mgr.is_streaming {
                        "⏸ Pause Stream"
                    } else {
                        "▶ Follow Live"
                    };
                    if ui.button(follow_label).clicked() {
                        let new_state = !self.log_mgr.is_streaming;
                        let src = self.log_mgr.current_source.clone();
                        self.log_mgr.switch_source(src, new_state);
                    }

                    // Refresh Button
                    if ui.button("🔄 Reload").clicked() {
                        let src = self.log_mgr.current_source.clone();
                        let stream = self.log_mgr.is_streaming;
                        self.entries.clear();
                        self.error_count = 0;
                        self.warn_count = 0;
                        self.log_mgr.switch_source(src, stream);
                    }

                    // Clear Button
                    if ui.button("🧹 Clear").clicked() {
                        self.entries.clear();
                        self.filtered_indices.clear();
                        self.error_count = 0;
                        self.warn_count = 0;
                        self.selected_entry_idx = None;
                    }

                    ui.separator();

                    // Export Options
                    if ui.button("💾 Export JSON").clicked() {
                        self.export_logs_json();
                    }
                    if ui.button("📄 Export TXT").clicked() {
                        self.export_logs_text();
                    }
                });
            });

        // 2. BOTTOM STATUS BAR (Wrapped to prevent overlap on small screens)
        TopBottomPanel::bottom("bottom_status_panel")
            .frame(egui::Frame::none().fill(p.bg_panel).inner_margin(egui::Margin::symmetric(14.0, 6.0)))
            .show(ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        RichText::new(format!("📊 Total: {} lines", self.entries.len()))
                            .font(FontId::proportional(12.0))
                            .color(p.text_primary),
                    );

                    ui.separator();

                    ui.label(
                        RichText::new(format!("🔍 Showing: {} lines", self.filtered_indices.len()))
                            .font(FontId::proportional(12.0))
                            .color(p.accent_cyan),
                    );

                    ui.separator();

                    ui.label(
                        RichText::new(format!("🔥 Errors: {}", self.error_count))
                            .font(FontId::proportional(12.0))
                            .color(if self.error_count > 0 { p.accent_red } else { p.text_muted }),
                    );

                    ui.separator();

                    ui.label(
                        RichText::new(format!("⚡ Warnings: {}", self.warn_count))
                            .font(FontId::proportional(12.0))
                            .color(if self.warn_count > 0 { p.accent_yellow } else { p.text_muted }),
                    );

                    ui.separator();

                    ui.label(
                        RichText::new(format!("⚡ Rate: {:.1} lines/s", self.lines_per_second))
                            .font(FontId::proportional(12.0))
                            .color(p.accent_green),
                    );

                    // Toast Notification Banner
                    if let Some((ref msg, _)) = self.notification {
                        ui.separator();
                        ui.label(
                            RichText::new(msg)
                                .font(FontId::proportional(12.0))
                                .color(p.accent_pink)
                                .strong(),
                        );
                    }

                    ui.separator();

                    let kaomoji = if self.error_count > 0 {
                        "(ノ°Д°）ノ [Critical Alert!]"
                    } else if self.warn_count > 0 {
                        "(；^ω^) [Checking Warnings]"
                    } else {
                        "(✿◠‿◠) [All Nominal Nya~]"
                    };
                    ui.label(
                        RichText::new(kaomoji)
                            .font(FontId::proportional(12.0))
                            .color(p.accent_purple),
                    );

                    ui.separator();

                    ui.label(
                        RichText::new(format!("Source: {}", self.log_mgr.current_source.display_name()))
                            .font(FontId::proportional(11.5))
                            .color(p.text_muted),
                    );
                });
            });

        // 3. LEFT SIDEBAR PANEL (Sources, Filters, and Mascot)
        egui::SidePanel::left("left_sidebar_panel")
            .default_width(310.0)
            .width_range(270.0..=400.0)
            .frame(egui::Frame::none().fill(p.bg_dark).inner_margin(egui::Margin::symmetric(12.0, 10.0)))
            .show(ctx, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    // A) Mascot Card Widget
                    self.mascot.render(
                        ui,
                        &p,
                        self.error_count,
                        self.warn_count,
                        self.total_time,
                        dt,
                    );

                    ui.add_space(10.0);
                    ui.separator();

                    // B) Quick Level Filter Badges
                    ui.label(
                        RichText::new("🎀 LOG SEVERITY FILTERS")
                            .font(FontId::proportional(12.5))
                            .color(p.accent_purple)
                            .strong(),
                    );

                    ui.horizontal(|ui| {
                        if ui.button("Select All").clicked() {
                            for lvl in [
                                LogLevel::Fatal, LogLevel::Critical, LogLevel::Error,
                                LogLevel::Warn, LogLevel::Notice, LogLevel::Info,
                                LogLevel::Debug, LogLevel::Trace, LogLevel::Unknown,
                            ] {
                                self.level_filters.insert(lvl);
                            }
                            self.recompute_filters();
                        }
                        if ui.button("🔥 Only Errors").clicked() {
                            self.level_filters.clear();
                            self.level_filters.insert(LogLevel::Error);
                            self.level_filters.insert(LogLevel::Critical);
                            self.level_filters.insert(LogLevel::Fatal);
                            self.recompute_filters();
                        }
                        if ui.button("⚡ Warn+Error").clicked() {
                            self.level_filters.clear();
                            self.level_filters.insert(LogLevel::Warn);
                            self.level_filters.insert(LogLevel::Error);
                            self.level_filters.insert(LogLevel::Critical);
                            self.level_filters.insert(LogLevel::Fatal);
                            self.recompute_filters();
                        }
                    });

                    ui.add_space(4.0);

                    // Level checkboxes with kawaii anime badges
                    let level_list = [
                        (LogLevel::Error, "🔥 [ERROR / CRIT]", p.accent_red),
                        (LogLevel::Warn, "⚡ [WARN / ALERT]", p.accent_yellow),
                        (LogLevel::Notice, "✨ [NOTICE / START]", p.accent_green),
                        (LogLevel::Info, "🌸 [INFO / NOMINAL]", p.accent_cyan),
                        (LogLevel::Debug, "🐾 [DEBUG TRACE]", p.accent_purple),
                    ];

                    for (lvl, label, color) in level_list {
                        let mut checked = self.level_filters.contains(&lvl);
                        ui.horizontal(|ui| {
                            if ui.checkbox(&mut checked, RichText::new(label).color(color)).changed() {
                                if checked {
                                    self.level_filters.insert(lvl);
                                    if lvl == LogLevel::Error {
                                        self.level_filters.insert(LogLevel::Critical);
                                        self.level_filters.insert(LogLevel::Fatal);
                                    }
                                } else {
                                    self.level_filters.remove(&lvl);
                                    if lvl == LogLevel::Error {
                                        self.level_filters.remove(&LogLevel::Critical);
                                        self.level_filters.remove(&LogLevel::Fatal);
                                    }
                                }
                                self.recompute_filters();
                            }
                        });
                    }

                    ui.add_space(10.0);
                    ui.separator();

                    // C) Log Sources Tree
                    ui.label(
                        RichText::new("📁 LOG SOURCES")
                            .font(FontId::proportional(12.5))
                            .color(p.accent_purple)
                            .strong(),
                    );

                    // Systemd Journal sources
                    ui.collapsing("📜 Systemd Journal (journalctl)", |ui| {
                        let sources = [
                            (SourceKind::JournalAll, "📜 All System Logs"),
                            (SourceKind::JournalErrors, "🔥 Errors Only (-p err)"),
                            (SourceKind::JournalCurrentBoot, "🚀 Current Boot (-b)"),
                            (SourceKind::JournalKernel, "🐧 Kernel Buffer (dmesg)"),
                        ];

                        for (src, title) in sources {
                            let is_active = self.log_mgr.current_source == src;
                            if ui.selectable_label(is_active, title).clicked() && !is_active {
                                self.entries.clear();
                                self.error_count = 0;
                                self.warn_count = 0;
                                self.selected_entry_idx = None;
                                self.log_mgr.switch_source(src, true);
                            }
                        }
                    });

                    // Discovered /var/log/ files
                    ui.collapsing("📄 Linux Log Files (/var/log/)", |ui| {
                        if self.discovered_logs.is_empty() {
                            ui.colored_label(p.text_muted, "No standard /var/log readable without root");
                        } else {
                            for path in &self.discovered_logs {
                                let src = SourceKind::SystemFile(path.clone());
                                let is_active = self.log_mgr.current_source == src;
                                let name = path.file_name().unwrap_or_default().to_string_lossy();
                                if ui.selectable_label(is_active, format!("📄 {}", name)).clicked() && !is_active {
                                    self.entries.clear();
                                    self.error_count = 0;
                                    self.warn_count = 0;
                                    self.selected_entry_idx = None;
                                    self.log_mgr.switch_source(src, true);
                                }
                            }
                        }
                    });

                    // Custom file button
                    ui.horizontal(|ui| {
                        if ui.button("📂 Open Custom Log File...").clicked() {
                            self.file_dialog.open();
                        }
                    });

                    // Anime Demo Logs (Always available for testing & fun!)
                    let demo_src = SourceKind::AnimeDemo;
                    let is_demo_active = self.log_mgr.current_source == demo_src;
                    if ui.selectable_label(is_demo_active, "✨ NekoOS Kawaii Demo Logs").clicked() && !is_demo_active {
                        self.entries.clear();
                        self.error_count = 0;
                        self.warn_count = 0;
                        self.selected_entry_idx = None;
                        self.log_mgr.switch_source(demo_src, true);
                    }

                    ui.add_space(10.0);
                    ui.separator();

                    // D) Service Filter (Top Services)
                    ui.label(
                        RichText::new("🍙 ACTIVE UNITS / SERVICES")
                            .font(FontId::proportional(12.5))
                            .color(p.accent_purple)
                            .strong(),
                    );

                    if self.selected_service_filter.is_some() {
                        if ui.button("✖ Clear Service Filter").clicked() {
                            self.selected_service_filter = None;
                            self.recompute_filters();
                        }
                    }

                    // Extract unique services from recent entries
                    let mut top_services: Vec<String> = self
                        .entries
                        .iter()
                        .take(500)
                        .map(|e| e.service.clone())
                        .filter(|s| !s.is_empty())
                        .collect();
                    top_services.sort();
                    top_services.dedup();

                    for svc in top_services.iter().take(12) {
                        let is_sel = self.selected_service_filter.as_ref() == Some(svc);
                        if ui.selectable_label(is_sel, format!("🍙 {}", svc)).clicked() {
                            if is_sel {
                                self.selected_service_filter = None;
                            } else {
                                self.selected_service_filter = Some(svc.clone());
                            }
                            self.recompute_filters();
                        }
                    }
                });
            });

        // 4. MAIN LOG TABLE PANEL
        CentralPanel::default()
            .frame(egui::Frame::none().fill(p.bg_dark).inner_margin(egui::Margin::symmetric(14.0, 10.0)))
            .show(ctx, |ui| {
                // Background Sakura Particle Canvas
                let bg_rect = ui.available_rect_before_wrap();
                let painter = ui.painter_at(bg_rect);
                self.particles.paint(&painter);

                // Search & Control Bar (Wrapped to prevent overlap on window resize)
                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        RichText::new("🔍 Search:")
                            .font(FontId::proportional(13.0))
                            .color(p.accent_pink)
                            .strong(),
                    );

                    let search_resp = ui.add(
                        TextEdit::singleline(&mut self.search_query)
                            .hint_text("Filter keywords, IP, process, error messages...")
                            .desired_width(220.0),
                    );
                    if search_resp.changed() {
                        self.recompute_filters();
                    }

                    if ui.checkbox(&mut self.use_regex, ".* Regex").changed() {
                        self.recompute_filters();
                    }

                    if !self.search_query.is_empty() {
                        if ui.button("✖ Clear").clicked() {
                            self.search_query.clear();
                            self.recompute_filters();
                        }

                        let match_badge = format!("🌸 {} matched", self.filtered_indices.len());
                        ui.label(
                            RichText::new(match_badge)
                                .font(FontId::proportional(12.0))
                                .color(p.accent_green),
                        );
                    }

                    ui.separator();
                    ui.checkbox(&mut self.auto_scroll_to_bottom, "📌 Auto-Scroll Bottom");
                });

                ui.add_space(6.0);

                // Table Header Row
                ui.horizontal(|ui| {
                    ui.colored_label(p.accent_purple, "  # Line  ");
                    ui.colored_label(p.accent_pink, " Severity ");
                    ui.colored_label(p.accent_cyan, " Timestamp ");
                    ui.colored_label(p.accent_yellow, " Unit / Source ");
                    ui.colored_label(p.text_primary, " Message ");
                });

                ui.separator();

                // Virtualized Scroll Area for log lines
                let row_height = 24.0;
                let num_rows = self.filtered_indices.len();

                let mut clicked_entry_idx: Option<usize> = None;

                let scroll_area = ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .stick_to_bottom(self.auto_scroll_to_bottom);

                scroll_area.show_rows(ui, row_height, num_rows, |ui, row_range| {
                    for row_idx in row_range {
                        let entry_idx = self.filtered_indices[row_idx];
                        if let Some(entry) = self.entries.get(entry_idx) {
                            let is_selected = self.selected_entry_idx == Some(entry_idx);
                            let level_color = entry.level.color(&p);
                            let bg_tint = if is_selected {
                                Color32::from_rgba_premultiplied(p.accent_pink.r(), p.accent_pink.g(), p.accent_pink.b(), 60)
                            } else if matches!(entry.level, LogLevel::Error | LogLevel::Critical | LogLevel::Fatal) {
                                Color32::from_rgba_premultiplied(255, 60, 60, 28)
                            } else if entry.level == LogLevel::Warn {
                                Color32::from_rgba_premultiplied(255, 200, 60, 22)
                            } else {
                                Color32::TRANSPARENT
                            };

                            let (rect, row_resp) = ui.allocate_exact_size(
                                vec2(ui.available_width(), row_height),
                                egui::Sense::click(),
                            );

                            if row_resp.clicked() {
                                clicked_entry_idx = Some(entry_idx);
                            }

                            let row_painter = ui.painter_at(rect);
                            if bg_tint != Color32::TRANSPARENT {
                                row_painter.rect_filled(rect, Rounding::same(4.0), bg_tint);
                            }

                            if is_selected {
                                row_painter.rect_stroke(rect, Rounding::same(4.0), Stroke::new(1.0_f32, p.accent_pink));
                            }

                            // Render row contents
                            let y = rect.center().y;

                            // 1. Line number
                            let line_str = format!("{:>6}", entry.line_number);
                            row_painter.text(
                                Pos2::new(rect.left() + 6.0, y),
                                egui::Align2::LEFT_CENTER,
                                line_str,
                                FontId::monospace(11.5),
                                p.text_muted,
                            );

                            // 2. Anime Badge
                            let badge = entry.level.anime_badge();
                            row_painter.text(
                                Pos2::new(rect.left() + 75.0, y),
                                egui::Align2::LEFT_CENTER,
                                badge,
                                FontId::proportional(11.5),
                                level_color,
                            );

                            // 3. Timestamp
                            let ts_str = if entry.timestamp.len() > 19 {
                                &entry.timestamp[..19]
                            } else {
                                &entry.timestamp
                            };
                            row_painter.text(
                                Pos2::new(rect.left() + 170.0, y),
                                egui::Align2::LEFT_CENTER,
                                ts_str,
                                FontId::monospace(11.0),
                                p.accent_cyan,
                            );

                            // 4. Service / Unit
                            let svc_str = if entry.service.len() > 18 {
                                format!("{}...", &entry.service[..15])
                            } else {
                                entry.service.clone()
                            };
                            row_painter.text(
                                Pos2::new(rect.left() + 310.0, y),
                                egui::Align2::LEFT_CENTER,
                                svc_str,
                                FontId::proportional(11.5),
                                p.accent_yellow,
                            );

                            // 5. Message
                            let msg_clip_x = rect.left() + 450.0;
                            let msg_str = if entry.message.len() > 140 {
                                format!("{}...", &entry.message[..137])
                            } else {
                                entry.message.clone()
                            };
                            row_painter.text(
                                Pos2::new(msg_clip_x, y),
                                egui::Align2::LEFT_CENTER,
                                msg_str,
                                FontId::monospace(12.0),
                                p.text_primary,
                            );
                        }
                    }
                });

                if let Some(idx) = clicked_entry_idx {
                    self.selected_entry_idx = Some(idx);
                }

                // 5. INSPECTOR DRAWER (When a log entry is selected)
                if let Some(sel_idx) = self.selected_entry_idx {
                    if let Some(entry) = self.entries.get(sel_idx) {
                        ui.separator();
                        ui.push_id("inspector_drawer", |ui| {
                            egui::Frame::none()
                                .fill(p.bg_card)
                                .rounding(Rounding::same(8.0))
                                .stroke(Stroke::new(1.0_f32, p.accent_pink))
                                .inner_margin(egui::Margin::symmetric(12.0, 10.0))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            RichText::new("🔍 LOG ENTRY INSPECTOR")
                                                .font(FontId::proportional(13.0))
                                                .color(p.accent_pink)
                                                .strong(),
                                        );

                                        let badge_str = format!("{}  Kaomoji: {}", entry.level.anime_badge(), entry.level.anime_kaomoji());
                                        ui.colored_label(entry.level.color(&p), badge_str);

                                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                            if ui.button("✖ Close Inspector").clicked() {
                                                self.selected_entry_idx = None;
                                            }

                                            if ui.button("📋 Copy Raw").clicked() {
                                                ui.output_mut(|o| o.copied_text = entry.raw.clone());
                                                self.notification = Some(("🌸 Copied log line to clipboard! ✨".into(), 3.0));
                                            }
                                        });
                                    });

                                    ui.horizontal(|ui| {
                                        ui.label(format!("Line: #{}", entry.line_number));
                                        ui.separator();
                                        ui.label(format!("Timestamp: {}", entry.timestamp));
                                        ui.separator();
                                        ui.label(format!("Unit/Service: {}", entry.service));
                                    });

                                    ui.add_space(4.0);

                                    // Mascot advice / diagnostic speech
                                    let anime_analysis = match entry.level {
                                        LogLevel::Fatal | LogLevel::Critical => {
                                            "💥 (ノ°Д°）ノ Neko-chan says: \"Eeeek! A critical crash or kernel panic occurred! Verify daemon dependencies and hardware memory immediately!\""
                                        }
                                        LogLevel::Error => {
                                            "🔥 (ノ°Д°）ノ Neko-chan says: \"Senpai! This service failed or returned an error status! Check permissions, socket files, or configuration syntax.\""
                                        }
                                        LogLevel::Warn => {
                                            "⚡ (；^ω^) Neko-chan says: \"Warning detected! It's not crashing yet, but keep an eye on resource consumption or deprecation warnings!\""
                                        }
                                        _ => {
                                            "🌸 (✿◠‿◠) Neko-chan says: \"Standard operating message! Everything looks healthy and fluffy nya~\""
                                        }
                                    };
                                    ui.colored_label(entry.level.color(&p), anime_analysis);

                                    ui.add_space(4.0);

                                    // Full message box
                                    ScrollArea::vertical().max_height(90.0).show(ui, |ui| {
                                        ui.add(
                                            TextEdit::multiline(&mut entry.raw.as_str())
                                                .font(TextStyle::Monospace)
                                                .desired_width(ui.available_width()),
                                        );
                                    });
                                });
                        });
                    }
                }
            });

        // 6. RENDER FILE DIALOG MODAL (If open)
        if let Some(chosen) = self.file_dialog.render(ctx, &p) {
            let src = SourceKind::CustomFile(chosen);
            self.entries.clear();
            self.error_count = 0;
            self.warn_count = 0;
            self.selected_entry_idx = None;
            self.log_mgr.switch_source(src, true);
        }

        // Request repaint for smooth 60 FPS animations (mascot blinking, breathing, tail sway, sakura particles)
        ctx.request_repaint();
    }
}
