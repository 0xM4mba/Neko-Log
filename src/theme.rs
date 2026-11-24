use egui::{Color32, Stroke, TextStyle, FontId, FontFamily};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    SakuraCyberpunk,
    TokyoNeon,
    MatchaMaid,
    MidnightNeko,
}

impl ThemeMode {
    pub const ALL: [ThemeMode; 4] = [
        ThemeMode::SakuraCyberpunk,
        ThemeMode::TokyoNeon,
        ThemeMode::MatchaMaid,
        ThemeMode::MidnightNeko,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            ThemeMode::SakuraCyberpunk => "🌸 Sakura Cyberpunk",
            ThemeMode::TokyoNeon => "⚡ Tokyo Neon",
            ThemeMode::MatchaMaid => "🍵 Matcha Maid",
            ThemeMode::MidnightNeko => "🐾 Midnight Neko",
        }
    }
}

pub struct AnimePalette {
    pub bg_dark: Color32,
    pub bg_card: Color32,
    pub bg_card_hover: Color32,
    pub bg_panel: Color32,
    pub text_primary: Color32,
    pub text_muted: Color32,
    pub accent_pink: Color32,
    pub accent_cyan: Color32,
    pub accent_purple: Color32,
    pub accent_yellow: Color32,
    pub accent_green: Color32,
    pub accent_red: Color32,
    #[allow(dead_code)]
    pub border_glow: Color32,
}

impl ThemeMode {
    pub fn palette(&self) -> AnimePalette {
        match self {
            ThemeMode::SakuraCyberpunk => AnimePalette {
                bg_dark: Color32::from_rgb(26, 21, 38),
                bg_card: Color32::from_rgb(38, 31, 56),
                bg_card_hover: Color32::from_rgb(52, 42, 77),
                bg_panel: Color32::from_rgb(32, 26, 47),
                text_primary: Color32::from_rgb(248, 248, 242),
                text_muted: Color32::from_rgb(170, 155, 195),
                accent_pink: Color32::from_rgb(255, 121, 198),
                accent_cyan: Color32::from_rgb(139, 233, 253),
                accent_purple: Color32::from_rgb(189, 147, 249),
                accent_yellow: Color32::from_rgb(241, 250, 140),
                accent_green: Color32::from_rgb(80, 250, 123),
                accent_red: Color32::from_rgb(255, 85, 85),
                border_glow: Color32::from_rgba_premultiplied(255, 121, 198, 120),
            },
            ThemeMode::TokyoNeon => AnimePalette {
                bg_dark: Color32::from_rgb(15, 14, 26),
                bg_card: Color32::from_rgb(25, 23, 44),
                bg_card_hover: Color32::from_rgb(40, 36, 70),
                bg_panel: Color32::from_rgb(20, 18, 35),
                text_primary: Color32::from_rgb(255, 255, 255),
                text_muted: Color32::from_rgb(140, 145, 175),
                accent_pink: Color32::from_rgb(255, 42, 133),
                accent_cyan: Color32::from_rgb(0, 240, 255),
                accent_purple: Color32::from_rgb(160, 80, 255),
                accent_yellow: Color32::from_rgb(255, 230, 0),
                accent_green: Color32::from_rgb(0, 255, 150),
                accent_red: Color32::from_rgb(255, 60, 90),
                border_glow: Color32::from_rgba_premultiplied(0, 240, 255, 140),
            },
            ThemeMode::MatchaMaid => AnimePalette {
                bg_dark: Color32::from_rgb(22, 32, 26),
                bg_card: Color32::from_rgb(32, 46, 38),
                bg_card_hover: Color32::from_rgb(44, 64, 52),
                bg_panel: Color32::from_rgb(27, 39, 32),
                text_primary: Color32::from_rgb(240, 248, 242),
                text_muted: Color32::from_rgb(155, 185, 165),
                accent_pink: Color32::from_rgb(255, 170, 200),
                accent_cyan: Color32::from_rgb(140, 230, 220),
                accent_purple: Color32::from_rgb(180, 160, 230),
                accent_yellow: Color32::from_rgb(245, 235, 145),
                accent_green: Color32::from_rgb(115, 230, 145),
                accent_red: Color32::from_rgb(255, 110, 110),
                border_glow: Color32::from_rgba_premultiplied(115, 230, 145, 130),
            },
            ThemeMode::MidnightNeko => AnimePalette {
                bg_dark: Color32::from_rgb(18, 18, 28),
                bg_card: Color32::from_rgb(28, 28, 44),
                bg_card_hover: Color32::from_rgb(42, 42, 66),
                bg_panel: Color32::from_rgb(23, 23, 36),
                text_primary: Color32::from_rgb(245, 245, 250),
                text_muted: Color32::from_rgb(160, 160, 190),
                accent_pink: Color32::from_rgb(255, 140, 190),
                accent_cyan: Color32::from_rgb(120, 210, 255),
                accent_purple: Color32::from_rgb(200, 160, 255),
                accent_yellow: Color32::from_rgb(255, 220, 130),
                accent_green: Color32::from_rgb(120, 240, 170),
                accent_red: Color32::from_rgb(255, 95, 95),
                border_glow: Color32::from_rgba_premultiplied(200, 160, 255, 130),
            },
        }
    }
}

pub fn configure_egui_style(ctx: &egui::Context, theme: ThemeMode) {
    let p = theme.palette();
    let mut visuals = egui::Visuals::dark();

    visuals.override_text_color = Some(p.text_primary);
    visuals.panel_fill = p.bg_panel;
    visuals.window_fill = p.bg_dark;
    visuals.faint_bg_color = p.bg_card;
    visuals.extreme_bg_color = p.bg_dark;

    visuals.widgets.noninteractive.bg_fill = p.bg_card;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(p.accent_purple.r(), p.accent_purple.g(), p.accent_purple.b(), 60));
    visuals.widgets.noninteractive.rounding = egui::Rounding::same(8.0);

    visuals.widgets.inactive.bg_fill = p.bg_card;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(p.accent_pink.r(), p.accent_pink.g(), p.accent_pink.b(), 70));
    visuals.widgets.inactive.rounding = egui::Rounding::same(8.0);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, p.text_primary);

    visuals.widgets.hovered.bg_fill = p.bg_card_hover;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.5_f32, p.accent_pink);
    visuals.widgets.hovered.rounding = egui::Rounding::same(8.0);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.2_f32, p.accent_cyan);

    visuals.widgets.active.bg_fill = p.accent_purple;
    visuals.widgets.active.bg_stroke = Stroke::new(2.0_f32, p.accent_cyan);
    visuals.widgets.active.rounding = egui::Rounding::same(8.0);
    visuals.widgets.active.fg_stroke = Stroke::new(1.5_f32, Color32::WHITE);

    visuals.selection.bg_fill = Color32::from_rgba_premultiplied(p.accent_pink.r(), p.accent_pink.g(), p.accent_pink.b(), 90);
    visuals.selection.stroke = Stroke::new(1.0_f32, p.accent_cyan);

    ctx.set_visuals(visuals);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.button_padding = egui::vec2(10.0, 6.0);
    style.text_styles.insert(
        TextStyle::Heading,
        FontId::new(20.0, FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Body,
        FontId::new(13.5, FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Monospace,
        FontId::new(12.5, FontFamily::Monospace),
    );
    style.text_styles.insert(
        TextStyle::Button,
        FontId::new(13.0, FontFamily::Proportional),
    );
    ctx.set_style(style);
}
