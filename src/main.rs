mod app;
mod file_dialog;
mod log_entry;
mod log_source;
mod mascot;
mod particles;
mod theme;
mod updater;

use app::NekoLogApp;
use egui::Vec2;
use egui::IconData;

fn main() -> eframe::Result<()> {
    // Generate a cute 32x32 RGBA anime cat paw icon
    let icon = generate_anime_app_icon();

    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_app_id("nekolog")
            .with_title("🌸 NekoLog // ログニャン - Anime Linux Log Viewer")
            .with_inner_size(Vec2::new(1220.0, 780.0))
            .with_min_inner_size(Vec2::new(720.0, 460.0))
            .with_icon(icon),
        ..Default::default()
    };

    eframe::run_native(
        "NekoLog",
        native_options,
        Box::new(|cc| Ok(Box::new(NekoLogApp::new(cc)))),
    )
}

fn generate_anime_app_icon() -> IconData {
    let width = 32;
    let height = 32;
    let mut rgba = vec![0u8; (width * height * 4) as usize];

    for y in 0..height {
        for x in 0..width {
            let idx = ((y * width + x) * 4) as usize;
            let fx = x as f32;
            let fy = y as f32;

            // Distance from center (16, 17) for main paw pad
            let dx = fx - 16.0;
            let dy = fy - 17.0;
            let main_dist = (dx * dx + dy * dy).sqrt();

            // Toe bean 1 (top left)
            let d_toe1 = ((fx - 10.0).powi(2) + (fy - 8.0).powi(2)).sqrt();
            // Toe bean 2 (top mid-left)
            let d_toe2 = ((fx - 14.0).powi(2) + (fy - 6.0).powi(2)).sqrt();
            // Toe bean 3 (top mid-right)
            let d_toe3 = ((fx - 18.0).powi(2) + (fy - 6.0).powi(2)).sqrt();
            // Toe bean 4 (top right)
            let d_toe4 = ((fx - 22.0).powi(2) + (fy - 8.0).powi(2)).sqrt();

            if main_dist <= 7.5 {
                // Main Pink Paw Pad
                rgba[idx] = 255;     // R
                rgba[idx + 1] = 120; // G
                rgba[idx + 2] = 180; // B
                rgba[idx + 3] = 255; // A
            } else if d_toe1 <= 2.5 || d_toe2 <= 2.8 || d_toe3 <= 2.8 || d_toe4 <= 2.5 {
                // Toe Beans
                rgba[idx] = 255;
                rgba[idx + 1] = 140;
                rgba[idx + 2] = 195;
                rgba[idx + 3] = 255;
            } else {
                // Circular dark purple badge background
                let badge_dist = ((fx - 16.0).powi(2) + (fy - 16.0).powi(2)).sqrt();
                if badge_dist <= 15.0 {
                    rgba[idx] = 38;
                    rgba[idx + 1] = 31;
                    rgba[idx + 2] = 56;
                    rgba[idx + 3] = 230;
                }
            }
        }
    }

    IconData {
        rgba,
        width,
        height,
    }
}
