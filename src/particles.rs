use egui::{Color32, Pos2, Rect, Shape, Stroke, Vec2};
use std::f32::consts::PI;

#[derive(Clone)]
pub struct SakuraPetal {
    pub pos: Pos2,
    pub velocity: Vec2,
    pub size: f32,
    pub angle: f32,
    pub spin: f32,
    pub sway_phase: f32,
    pub sway_amp: f32,
    pub alpha: f32,
}

#[derive(Clone)]
pub struct Sparkle {
    pub pos: Pos2,
    pub max_size: f32,
    pub phase: f32,
    pub speed: f32,
    pub color: Color32,
}

pub struct ParticleSystem {
    pub petals: Vec<SakuraPetal>,
    pub sparkles: Vec<Sparkle>,
    pub enabled: bool,
}

impl ParticleSystem {
    pub fn new() -> Self {
        let mut petals = Vec::new();
        // Generate initial sakura petals with random distribution
        for i in 0..45 {
            let fi = i as f32;
            petals.push(SakuraPetal {
                pos: Pos2::new(fi * 37.0 % 1600.0, (fi * 53.0) % 900.0),
                velocity: Vec2::new(20.0 + (fi * 7.0) % 30.0, 35.0 + (fi * 11.0) % 45.0),
                size: 7.0 + (fi * 3.0) % 7.0,
                angle: (fi * 0.4) % (2.0 * PI),
                spin: 0.5 + (fi * 0.2) % 1.5,
                sway_phase: fi * 1.3,
                sway_amp: 25.0 + (fi * 4.0) % 30.0,
                alpha: 0.35 + (fi * 0.05) % 0.45,
            });
        }

        let mut sparkles = Vec::new();
        let sparkle_colors = [
            Color32::from_rgba_premultiplied(255, 255, 255, 180),
            Color32::from_rgba_premultiplied(255, 182, 218, 190),
            Color32::from_rgba_premultiplied(139, 233, 253, 190),
            Color32::from_rgba_premultiplied(241, 250, 140, 190),
        ];
        for i in 0..25 {
            let fi = i as f32;
            sparkles.push(Sparkle {
                pos: Pos2::new(fi * 67.0 % 1600.0, fi * 83.0 % 900.0),
                max_size: 4.0 + (fi * 2.0) % 6.0,
                phase: fi * 0.7,
                speed: 1.5 + (fi * 0.3) % 2.5,
                color: sparkle_colors[i % sparkle_colors.len()],
            });
        }

        Self {
            petals,
            sparkles,
            enabled: true,
        }
    }

    pub fn update(&mut self, dt: f32, bounds: Rect, time: f64) {
        if !self.enabled {
            return;
        }

        for (i, petal) in self.petals.iter_mut().enumerate() {
            let sway = (petal.sway_phase + (time as f32) * 1.5).sin() * petal.sway_amp * dt;
            petal.pos.x += petal.velocity.x * dt + sway;
            petal.pos.y += petal.velocity.y * dt;
            petal.angle += petal.spin * dt;

            // Wrap around screen bounds
            if petal.pos.y > bounds.bottom() + 20.0 || petal.pos.x > bounds.right() + 30.0 {
                petal.pos.y = bounds.top() - 20.0;
                let fi = i as f32;
                petal.pos.x = bounds.left() + (fi * 83.0 + (time as f32 * 40.0)) % bounds.width();
            }
        }

        for (i, sparkle) in self.sparkles.iter_mut().enumerate() {
            sparkle.phase += sparkle.speed * dt;
            if sparkle.phase > 2.0 * PI {
                sparkle.phase -= 2.0 * PI;
                let fi = i as f32;
                sparkle.pos.x = bounds.left() + (fi * 113.0 + (time as f32 * 60.0)) % bounds.width();
                sparkle.pos.y = bounds.top() + (fi * 79.0 + (time as f32 * 30.0)) % bounds.height();
            }
        }
    }

    pub fn paint(&self, painter: &egui::Painter) {
        if !self.enabled {
            return;
        }

        // Draw Sakura Petals
        for petal in &self.petals {
            let p = petal.pos;
            let s = petal.size;
            let a = petal.angle;
            let alpha_u8 = (petal.alpha * 255.0) as u8;
            let fill_color = Color32::from_rgba_premultiplied(255, 180, 210, alpha_u8);
            let stroke_color = Color32::from_rgba_premultiplied(255, 120, 180, alpha_u8 / 2);

            // Draw petal as curved rotated polygon
            let cos = a.cos();
            let sin = a.sin();

            let local_points = [
                Vec2::new(0.0, -s * 1.2),
                Vec2::new(s * 0.7, -s * 0.4),
                Vec2::new(s * 0.5, s * 0.8),
                Vec2::new(0.0, s * 1.0),
                Vec2::new(-s * 0.5, s * 0.8),
                Vec2::new(-s * 0.7, -s * 0.4),
            ];

            let points: Vec<Pos2> = local_points
                .iter()
                .map(|pt| {
                    let rx = pt.x * cos - pt.y * sin;
                    let ry = pt.x * sin + pt.y * cos;
                    Pos2::new(p.x + rx, p.y + ry)
                })
                .collect();

            painter.add(Shape::convex_polygon(
                points,
                fill_color,
                Stroke::new(0.5_f32, stroke_color),
            ));
        }

        // Draw Sparkles (4-point star)
        for sparkle in &self.sparkles {
            let scale = ((sparkle.phase).sin().abs()).powf(1.5);
            if scale < 0.05 {
                continue;
            }
            let r = sparkle.max_size * scale;
            let c = sparkle.pos;
            let color = sparkle.color;

            let points = vec![
                Pos2::new(c.x, c.y - r * 1.6),
                Pos2::new(c.x + r * 0.35, c.y - r * 0.35),
                Pos2::new(c.x + r * 1.6, c.y),
                Pos2::new(c.x + r * 0.35, c.y + r * 0.35),
                Pos2::new(c.x, c.y + r * 1.6),
                Pos2::new(c.x - r * 0.35, c.y + r * 0.35),
                Pos2::new(c.x - r * 1.6, c.y),
                Pos2::new(c.x - r * 0.35, c.y - r * 0.35),
            ];

            painter.add(Shape::convex_polygon(
                points,
                color,
                Stroke::NONE,
            ));
        }
    }
}
