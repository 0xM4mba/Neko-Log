use egui::{Color32, FontId, Pos2, Rect, Response, Rounding, Sense, Shape, Stroke, Ui, Vec2};
use crate::theme::AnimePalette;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MascotMood {
    Peaceful,   // 0 errors, few or no warnings -> happy, sparkling eyes
    Alert,      // Some warnings -> sweat drop, alert ears
    Panic,      // Errors found -> tears / flame aura, wide eyes
    Loved,      // User petted her -> purring, blush, hearts
}

#[derive(Clone)]
pub struct FloatingHeart {
    pub pos: Pos2,
    pub velocity: Vec2,
    pub size: f32,
    pub alpha: f32,
    pub life: f32,
}

pub struct AnimeMascot {
    pub pet_count: u32,
    pub hearts: Vec<FloatingHeart>,
    pub blink_timer: f32,
    pub is_blinking: bool,
    pub speech_text: String,
    pub speech_timer: f32,
    pub pet_reaction_timer: f32,
}

impl AnimeMascot {
    pub fn new() -> Self {
        Self {
            pet_count: 0,
            hearts: Vec::new(),
            blink_timer: 2.5,
            is_blinking: false,
            speech_text: "Konnichiwa! I am Neko-chan! Let's inspect your Linux logs nya~ (★ω★)".into(),
            speech_timer: 10.0,
            pet_reaction_timer: 0.0,
        }
    }

    pub fn update_mood(&mut self, error_count: usize, warn_count: usize, dt: f32) -> MascotMood {
        // Decrease pet reaction timer
        if self.pet_reaction_timer > 0.0 {
            self.pet_reaction_timer -= dt;
            return MascotMood::Loved;
        }

        if error_count > 0 {
            MascotMood::Panic
        } else if warn_count > 0 {
            MascotMood::Alert
        } else {
            MascotMood::Peaceful
        }
    }

    pub fn update(&mut self, dt: f32, error_count: usize, warn_count: usize, log_total: usize) {
        // Blinking cycle
        self.blink_timer -= dt;
        if self.blink_timer <= 0.0 {
            if self.is_blinking {
                self.is_blinking = false;
                self.blink_timer = 2.0 + (self.pet_count as f32 * 0.1).sin().abs() * 3.0;
            } else {
                self.is_blinking = true;
                self.blink_timer = 0.15; // Quick blink
            }
        }

        // Update floating hearts
        for heart in &mut self.hearts {
            heart.pos.x += heart.velocity.x * dt;
            heart.pos.y += heart.velocity.y * dt;
            heart.alpha -= dt * 0.6;
            heart.life += dt;
        }
        self.hearts.retain(|h| h.alpha > 0.05);

        // Dynamic anime speech updates
        self.speech_timer -= dt;
        if self.speech_timer <= 0.0 {
            self.speech_timer = 8.0;
            let mood = self.update_mood(error_count, warn_count, 0.0);
            self.speech_text = match mood {
                MascotMood::Loved => {
                    let quotes = [
                        "Purr purr~ Headpats feel so warm, Senpai! (///ω///)",
                        "Nyaa~ You're the best Linux master ever! (* >ω<)",
                        "Heartbeat stable! All kernel threads synced! ♡",
                    ];
                    quotes[(self.pet_count as usize) % quotes.len()].to_string()
                }
                MascotMood::Panic => {
                    let quotes = [
                        format!("H-Hyaaa! {} errors detected! Senpai, taskete! ヽ(ﾟДﾟ)ﾉ", error_count),
                        "Fire in the systemd units! Check the red logs quickly!".into(),
                        "Kernel or service throwing exceptions! Let's triage it!".into(),
                    ];
                    quotes[(self.pet_count as usize) % quotes.len()].to_string()
                }
                MascotMood::Alert => {
                    let quotes = [
                        format!("Hmm? {} warnings spotted. Nothing fatal yet nya~ (；^ω^)", warn_count),
                        "Careful, some daemons are grumbling in the logs!".into(),
                        "Sensors tingling! A few warnings need your attention, Senpai.".into(),
                    ];
                    quotes[(self.pet_count as usize) % quotes.len()].to_string()
                }
                MascotMood::Peaceful => {
                    let quotes = [
                        format!("Everything is peaceful nya~ {} logs parsed cleanly! (✿◠‿◠)", log_total),
                        "Sugoi! System memory and daemon journals are looking kawaii! ✨".into(),
                        "Log stream is flowing like a calm anime spring stream~ 🌸".into(),
                        "Click me for headpats anytime, Senpai! ( =^･ω･^= )".into(),
                    ];
                    quotes[(self.pet_count as usize) % quotes.len()].to_string()
                }
            };
        }
    }

    pub fn pet(&mut self, center: Pos2) {
        self.pet_count += 1;
        self.pet_reaction_timer = 3.5;
        self.speech_text = match self.pet_count % 4 {
            0 => "Purr purr~ (///ω///) More headpats please, Senpai!".into(),
            1 => "Kyaa! My ears twitched! ( > ‿ < ) You're so sweet!".into(),
            2 => "Nyaa! System stability boosted by +100% love! ♡".into(),
            _ => "Hehe~ Cat-girl log analyst at your command! ✨".into(),
        };

        // Spawn 3 floating hearts
        for i in 0..4 {
            let fi = i as f32;
            self.hearts.push(FloatingHeart {
                pos: Pos2::new(center.x - 20.0 + fi * 12.0, center.y - 30.0),
                velocity: Vec2::new(-15.0 + fi * 10.0, -35.0 - fi * 8.0),
                size: 8.0 + fi * 2.0,
                alpha: 1.0,
                life: 0.0,
            });
        }
    }

    pub fn render(
        &mut self,
        ui: &mut Ui,
        p: &AnimePalette,
        error_count: usize,
        warn_count: usize,
        time: f64,
        dt: f32,
    ) -> Response {
        let mood = self.update_mood(error_count, warn_count, dt);

        let size = Vec2::new(ui.available_width().max(200.0), 175.0);
        let (rect, response) = ui.allocate_exact_size(size, Sense::click());

        if response.clicked() {
            let center = Pos2::new(rect.center().x, rect.center().y - 10.0);
            self.pet(center);
        }

        let painter = ui.painter_at(rect);

        // 1. Card Container with glowing gradient border
        let glow_color = match mood {
            MascotMood::Loved => Color32::from_rgb(255, 120, 180),
            MascotMood::Panic => Color32::from_rgb(255, 70, 70),
            MascotMood::Alert => Color32::from_rgb(255, 200, 80),
            MascotMood::Peaceful => p.accent_pink,
        };

        let card_bg = p.bg_card;
        painter.rect(
            rect,
            Rounding::same(12.0),
            card_bg,
            Stroke::new(1.5_f32, Color32::from_rgba_premultiplied(glow_color.r(), glow_color.g(), glow_color.b(), 100)),
        );

        let mascot_center = Pos2::new(rect.left() + 55.0, rect.center().y + 12.0);
        let t = time as f32;

        // 2. Animated Fluffy Tail (Swishing behind)
        let tail_sway = (t * 2.5).sin() * 18.0;
        let tail_base = Pos2::new(mascot_center.x - 22.0, mascot_center.y + 24.0);
        let tail_mid = Pos2::new(tail_base.x - 18.0 + tail_sway * 0.5, tail_base.y - 12.0);
        let tail_tip = Pos2::new(tail_base.x - 25.0 + tail_sway, tail_base.y - 28.0);

        painter.line_segment(
            [tail_base, tail_mid],
            Stroke::new(9.0_f32, Color32::from_rgb(255, 150, 195)),
        );
        painter.line_segment(
            [tail_mid, tail_tip],
            Stroke::new(7.0_f32, Color32::from_rgb(255, 180, 215)),
        );
        painter.circle_filled(tail_tip, 4.5, Color32::WHITE); // Cute white tail tip

        // 3. Aura / Halo (Pulse)
        let pulse = (t * 3.0).sin() * 3.0;
        let aura_radius = 42.0 + pulse;
        let aura_color = Color32::from_rgba_premultiplied(glow_color.r(), glow_color.g(), glow_color.b(), 25);
        painter.circle_filled(mascot_center, aura_radius, aura_color);

        // 4. Cat Ears
        let ear_twitch = match mood {
            MascotMood::Panic => (t * 12.0).sin() * 5.0,
            _ => (t * 1.8).sin() * 2.5,
        };

        // Left Ear
        let left_ear = vec![
            Pos2::new(mascot_center.x - 28.0, mascot_center.y - 14.0),
            Pos2::new(mascot_center.x - 24.0 + ear_twitch, mascot_center.y - 44.0),
            Pos2::new(mascot_center.x - 8.0, mascot_center.y - 26.0),
        ];
        painter.add(Shape::convex_polygon(
            left_ear,
            Color32::from_rgb(255, 130, 180),
            Stroke::new(1.0_f32, Color32::from_rgb(220, 80, 140)),
        ));
        // Left Ear Inner Pink
        let left_inner = vec![
            Pos2::new(mascot_center.x - 24.0, mascot_center.y - 18.0),
            Pos2::new(mascot_center.x - 22.0 + ear_twitch * 0.8, mascot_center.y - 38.0),
            Pos2::new(mascot_center.x - 12.0, mascot_center.y - 26.0),
        ];
        painter.add(Shape::convex_polygon(
            left_inner,
            Color32::from_rgb(255, 205, 225),
            Stroke::NONE,
        ));

        // Right Ear
        let right_ear = vec![
            Pos2::new(mascot_center.x + 8.0, mascot_center.y - 26.0),
            Pos2::new(mascot_center.x + 24.0 - ear_twitch, mascot_center.y - 44.0),
            Pos2::new(mascot_center.x + 28.0, mascot_center.y - 14.0),
        ];
        painter.add(Shape::convex_polygon(
            right_ear,
            Color32::from_rgb(255, 130, 180),
            Stroke::new(1.0_f32, Color32::from_rgb(220, 80, 140)),
        ));
        // Right Ear Inner Pink
        let right_inner = vec![
            Pos2::new(mascot_center.x + 12.0, mascot_center.y - 26.0),
            Pos2::new(mascot_center.x + 22.0 - ear_twitch * 0.8, mascot_center.y - 38.0),
            Pos2::new(mascot_center.x + 24.0, mascot_center.y - 18.0),
        ];
        painter.add(Shape::convex_polygon(
            right_inner,
            Color32::from_rgb(255, 205, 225),
            Stroke::NONE,
        ));

        // 5. Head / Face (Chibi round head)
        let head_radius = 28.0;
        let skin_color = Color32::from_rgb(255, 241, 235);
        painter.circle_filled(mascot_center, head_radius, skin_color);
        painter.circle_stroke(mascot_center, head_radius, Stroke::new(1.2_f32, Color32::from_rgb(255, 170, 195)));

        // Hair Bangs (Soft pastel pink hair)
        let hair_points = vec![
            Pos2::new(mascot_center.x - 28.0, mascot_center.y - 12.0),
            Pos2::new(mascot_center.x - 20.0, mascot_center.y - 26.0),
            Pos2::new(mascot_center.x - 8.0, mascot_center.y - 12.0),
            Pos2::new(mascot_center.x + 2.0, mascot_center.y - 24.0),
            Pos2::new(mascot_center.x + 12.0, mascot_center.y - 10.0),
            Pos2::new(mascot_center.x + 24.0, mascot_center.y - 22.0),
            Pos2::new(mascot_center.x + 28.0, mascot_center.y - 12.0),
            Pos2::new(mascot_center.x + 22.0, mascot_center.y - 30.0),
            Pos2::new(mascot_center.x, mascot_center.y - 32.0),
            Pos2::new(mascot_center.x - 22.0, mascot_center.y - 30.0),
        ];
        painter.add(Shape::convex_polygon(
            hair_points,
            Color32::from_rgb(255, 140, 190),
            Stroke::NONE,
        ));

        // Hair Clip / Star
        let star_pos = Pos2::new(mascot_center.x - 18.0, mascot_center.y - 20.0);
        painter.circle_filled(star_pos, 4.0, Color32::from_rgb(255, 230, 80));

        // 6. Blushing Cheeks
        let blush_alpha = if mood == MascotMood::Loved { 180 } else { 110 };
        let blush_color = Color32::from_rgba_premultiplied(255, 120, 150, blush_alpha);
        painter.circle_filled(Pos2::new(mascot_center.x - 17.0, mascot_center.y + 6.0), 5.5, blush_color);
        painter.circle_filled(Pos2::new(mascot_center.x + 17.0, mascot_center.y + 6.0), 5.5, blush_color);

        // 7. Expressive Anime Eyes
        let eye_y = mascot_center.y - 2.0;
        let left_eye_x = mascot_center.x - 11.0;
        let right_eye_x = mascot_center.x + 11.0;

        if self.is_blinking || mood == MascotMood::Loved {
            // Happy closed curved eyes: ^  ^
            let left_arc = [
                Pos2::new(left_eye_x - 6.0, eye_y + 2.0),
                Pos2::new(left_eye_x, eye_y - 4.0),
                Pos2::new(left_eye_x + 6.0, eye_y + 2.0),
            ];
            let right_arc = [
                Pos2::new(right_eye_x - 6.0, eye_y + 2.0),
                Pos2::new(right_eye_x, eye_y - 4.0),
                Pos2::new(right_eye_x + 6.0, eye_y + 2.0),
            ];
            painter.line_segment([left_arc[0], left_arc[1]], Stroke::new(2.2_f32, Color32::from_rgb(80, 50, 70)));
            painter.line_segment([left_arc[1], left_arc[2]], Stroke::new(2.2_f32, Color32::from_rgb(80, 50, 70)));
            painter.line_segment([right_arc[0], right_arc[1]], Stroke::new(2.2_f32, Color32::from_rgb(80, 50, 70)));
            painter.line_segment([right_arc[1], right_arc[2]], Stroke::new(2.2_f32, Color32::from_rgb(80, 50, 70)));
        } else if mood == MascotMood::Panic {
            // Frantic / Crying Anime Eyes: >  <
            painter.line_segment(
                [Pos2::new(left_eye_x - 6.0, eye_y - 5.0), Pos2::new(left_eye_x + 3.0, eye_y)],
                Stroke::new(2.5_f32, Color32::from_rgb(80, 50, 70)),
            );
            painter.line_segment(
                [Pos2::new(left_eye_x + 3.0, eye_y), Pos2::new(left_eye_x - 6.0, eye_y + 5.0)],
                Stroke::new(2.5_f32, Color32::from_rgb(80, 50, 70)),
            );

            painter.line_segment(
                [Pos2::new(right_eye_x + 6.0, eye_y - 5.0), Pos2::new(right_eye_x - 3.0, eye_y)],
                Stroke::new(2.5_f32, Color32::from_rgb(80, 50, 70)),
            );
            painter.line_segment(
                [Pos2::new(right_eye_x - 3.0, eye_y), Pos2::new(right_eye_x + 6.0, eye_y + 5.0)],
                Stroke::new(2.5_f32, Color32::from_rgb(80, 50, 70)),
            );

            // Anime tears streaming down!
            let tear_y = mascot_center.y + 12.0 + (t * 20.0) % 15.0;
            painter.circle_filled(Pos2::new(left_eye_x - 8.0, tear_y), 3.0, Color32::from_rgb(100, 200, 255));
            painter.circle_filled(Pos2::new(right_eye_x + 8.0, tear_y), 3.0, Color32::from_rgb(100, 200, 255));
        } else {
            // Big Beautiful Anime Eyes with Highlights
            let iris_color = match mood {
                MascotMood::Alert => Color32::from_rgb(255, 170, 50),
                _ => Color32::from_rgb(140, 70, 220),
            };

            // Eye outlines and irises
            painter.circle_filled(Pos2::new(left_eye_x, eye_y), 6.5, iris_color);
            painter.circle_filled(Pos2::new(right_eye_x, eye_y), 6.5, iris_color);

            // Dark pupil
            painter.circle_filled(Pos2::new(left_eye_x, eye_y + 1.0), 4.0, Color32::from_rgb(45, 20, 60));
            painter.circle_filled(Pos2::new(right_eye_x, eye_y + 1.0), 4.0, Color32::from_rgb(45, 20, 60));

            // Anime White Sparkle Highlight (Top Left)
            painter.circle_filled(Pos2::new(left_eye_x - 2.5, eye_y - 2.5), 2.5, Color32::WHITE);
            painter.circle_filled(Pos2::new(right_eye_x - 2.5, eye_y - 2.5), 2.5, Color32::WHITE);

            // Sub-highlight (Bottom Right)
            painter.circle_filled(Pos2::new(left_eye_x + 2.0, eye_y + 2.5), 1.2, Color32::WHITE);
            painter.circle_filled(Pos2::new(right_eye_x + 2.0, eye_y + 2.5), 1.2, Color32::WHITE);

            // Eyelashes
            painter.line_segment(
                [Pos2::new(left_eye_x - 7.0, eye_y - 6.0), Pos2::new(left_eye_x + 6.0, eye_y - 6.0)],
                Stroke::new(1.8_f32, Color32::from_rgb(70, 40, 60)),
            );
            painter.line_segment(
                [Pos2::new(right_eye_x - 6.0, eye_y - 6.0), Pos2::new(right_eye_x + 7.0, eye_y - 6.0)],
                Stroke::new(1.8_f32, Color32::from_rgb(70, 40, 60)),
            );
        }

        // 8. Anime Mouth
        let mouth_center = Pos2::new(mascot_center.x, mascot_center.y + 12.0);
        match mood {
            MascotMood::Panic => {
                // Open frantic mouth 'D'
                painter.circle_filled(mouth_center, 4.0, Color32::from_rgb(220, 60, 90));
            }
            MascotMood::Alert => {
                // Small worried 'o'
                painter.circle_filled(mouth_center, 2.5, Color32::from_rgb(180, 80, 100));
            }
            _ => {
                // Cat mouth ω shape
                let stroke = Stroke::new(1.6_f32, Color32::from_rgb(90, 50, 70));
                let left_w = [
                    Pos2::new(mouth_center.x - 4.5, mouth_center.y),
                    Pos2::new(mouth_center.x - 2.0, mouth_center.y + 3.0),
                    Pos2::new(mouth_center.x, mouth_center.y + 1.0),
                ];
                let right_w = [
                    Pos2::new(mouth_center.x, mouth_center.y + 1.0),
                    Pos2::new(mouth_center.x + 2.0, mouth_center.y + 3.0),
                    Pos2::new(mouth_center.x + 4.5, mouth_center.y),
                ];
                painter.line_segment([left_w[0], left_w[1]], stroke);
                painter.line_segment([left_w[1], left_w[2]], stroke);
                painter.line_segment([right_w[0], right_w[1]], stroke);
                painter.line_segment([right_w[1], right_w[2]], stroke);
            }
        }

        // 9. Anime Sweat Drop (when warning)
        if mood == MascotMood::Alert {
            let sweat_pos = Pos2::new(mascot_center.x + 25.0, mascot_center.y - 16.0);
            painter.circle_filled(sweat_pos, 4.0, Color32::from_rgb(100, 220, 255));
            let drop_tip = vec![
                Pos2::new(sweat_pos.x - 3.5, sweat_pos.y),
                Pos2::new(sweat_pos.x, sweat_pos.y - 7.0),
                Pos2::new(sweat_pos.x + 3.5, sweat_pos.y),
            ];
            painter.add(Shape::convex_polygon(drop_tip, Color32::from_rgb(100, 220, 255), Stroke::NONE));
        }

        // 10. Cat Paw Waving (Greeting Senpai!)
        let paw_wave = (t * 4.0).sin() * 4.0;
        let paw_pos = Pos2::new(mascot_center.x + 26.0, mascot_center.y + 20.0 + paw_wave);
        painter.circle_filled(paw_pos, 7.5, Color32::WHITE);
        painter.circle_stroke(paw_pos, 7.5, Stroke::new(1.0_f32, Color32::from_rgb(255, 170, 195)));
        // Paw pads (pink beans!)
        painter.circle_filled(Pos2::new(paw_pos.x, paw_pos.y + 1.0), 3.0, Color32::from_rgb(255, 140, 180));
        painter.circle_filled(Pos2::new(paw_pos.x - 3.5, paw_pos.y - 3.5), 1.5, Color32::from_rgb(255, 140, 180));
        painter.circle_filled(Pos2::new(paw_pos.x, paw_pos.y - 4.5), 1.5, Color32::from_rgb(255, 140, 180));
        painter.circle_filled(Pos2::new(paw_pos.x + 3.5, paw_pos.y - 3.5), 1.5, Color32::from_rgb(255, 140, 180));

        // 11. Speech Bubble on the right
        let bubble_rect = Rect::from_min_max(
            Pos2::new(rect.left() + 105.0, rect.top() + 16.0),
            Pos2::new(rect.right() - 12.0, rect.bottom() - 36.0),
        );

        painter.rect(
            bubble_rect,
            Rounding::same(10.0),
            Color32::from_rgba_premultiplied(p.bg_dark.r(), p.bg_dark.g(), p.bg_dark.b(), 220),
            Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(glow_color.r(), glow_color.g(), glow_color.b(), 180)),
        );

        // Speech bubble beak pointing to mascot
        let beak = vec![
            Pos2::new(bubble_rect.left(), bubble_rect.center().y - 6.0),
            Pos2::new(bubble_rect.left() - 8.0, bubble_rect.center().y),
            Pos2::new(bubble_rect.left(), bubble_rect.center().y + 6.0),
        ];
        painter.add(Shape::convex_polygon(
            beak,
            Color32::from_rgba_premultiplied(p.bg_dark.r(), p.bg_dark.g(), p.bg_dark.b(), 220),
            Stroke::NONE,
        ));

        // Speech Text inside Bubble
        let text_rect = bubble_rect.shrink(8.0);
        painter.text(
            text_rect.left_top(),
            egui::Align2::LEFT_TOP,
            &self.speech_text,
            FontId::proportional(12.0),
            p.text_primary,
        );

        // Pet counter badge at the bottom of the card
        let pet_badge_text = format!("🐾 Pet Neko-chan: {} pats  (Click!)", self.pet_count);
        painter.text(
            Pos2::new(rect.center().x, rect.bottom() - 18.0),
            egui::Align2::CENTER_CENTER,
            pet_badge_text,
            FontId::proportional(11.5),
            p.accent_pink,
        );

        // Draw Floating Hearts from petting
        for heart in &self.hearts {
            let hp = heart.pos;
            let s = heart.size;
            let a = (heart.alpha * 255.0) as u8;
            let heart_color = Color32::from_rgba_premultiplied(255, 105, 180, a);

            // Draw cute heart glyph or vector
            painter.text(
                hp,
                egui::Align2::CENTER_CENTER,
                "♥",
                FontId::proportional(s * 1.5),
                heart_color,
            );
        }

        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mascot_initial_mood() {
        let mut mascot = AnimeMascot::new();
        assert_eq!(mascot.update_mood(0, 0, 0.0), MascotMood::Peaceful);
        assert_eq!(mascot.update_mood(0, 3, 0.0), MascotMood::Alert);
        assert_eq!(mascot.update_mood(1, 0, 0.0), MascotMood::Panic);
    }

    #[test]
    fn test_mascot_petting() {
        let mut mascot = AnimeMascot::new();
        mascot.pet(Pos2::new(100.0, 100.0));
        assert_eq!(mascot.pet_count, 1);
        assert!(!mascot.hearts.is_empty());
        assert_eq!(mascot.update_mood(0, 0, 0.0), MascotMood::Loved);

        // Advance time to expire pet effect
        mascot.update_mood(0, 0, 4.0);
        assert_eq!(mascot.update_mood(0, 0, 0.0), MascotMood::Peaceful);
    }
}
