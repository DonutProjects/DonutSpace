use macroquad::prelude::*;
use std::collections::HashMap;

use crate::camera::Camera;
use crate::ship::Ship;
use crate::star::StarChunk;
use crate::ui::UIPanel;

// Engine positions relative to texture center (pixels)
const TEXTURE_WIDTH: f32 = 282.0;
const ENGINE1_DX: f32 = -48.5; // from center
const ENGINE1_DY: f32 = 136.5;
const ENGINE2_DX: f32 = 47.5;
const ENGINE2_DY: f32 = 136.5;

pub struct GameState {
    pub ship: Ship,
    pub camera: Camera,
    pub chunks: HashMap<(i32, i32), StarChunk>,
    pub target_x: Option<f32>,
    pub target_y: Option<f32>,
    pub ship_texture: Texture2D,
    pub ui: UIPanel,
}

impl GameState {
    pub async fn new() -> Self {
        let ship_texture = load_texture("src/ship1.png").await.unwrap();

        Self {
            ship: Ship {
                x: 0.0,
                y: 0.0,
                vx: 0.0,
                vy: 0.0,
                rotation: 0.0,
                size: super::SHIP_SIZE,
            },
            camera: Camera { x: 0.0, y: 0.0, zoom: 1.0 },
            chunks: HashMap::new(),
            target_x: None,
            target_y: None,
            ship_texture,
            ui: UIPanel::new(screen_width(), screen_height()).await,
        }
    }

    pub fn update_chunks(&mut self) {
        let view_radius = 10;
        let center_cx = (self.camera.x / super::CHUNK_SIZE as f32).floor() as i32;
        let center_cy = (self.camera.y / super::CHUNK_SIZE as f32).floor() as i32;

        for dx in -view_radius..=view_radius {
            for dy in -view_radius..=view_radius {
                let cx = center_cx + dx;
                let cy = center_cy + dy;
                let key = (cx, cy);
                if !self.chunks.contains_key(&key) {
                    let seed = cx as i64 * 1000000 + cy as i64;
                    self.chunks.insert(key, StarChunk::new(seed));
                }
            }
        }

        let visible_keys: Vec<_> = self.chunks.keys().cloned().collect();
        for key in visible_keys {
            let dist_x = (key.0 - center_cx).abs();
            let dist_y = (key.1 - center_cy).abs();
            if dist_x > view_radius + 5 || dist_y > view_radius + 5 {
                self.chunks.remove(&key);
            }
        }
    }

    pub fn update(&mut self, dt: f32) {
        // Handle zoom with mouse wheel
        let (_, wheel_y) = mouse_wheel();
        if wheel_y > 0.0 {
            self.camera.zoom *= 1.1;
        } else if wheel_y < 0.0 {
            self.camera.zoom /= 1.1;
            if self.camera.zoom < 0.1 { self.camera.zoom = 0.1; }
        }

        // Handle zoom with keys (optional)
        if is_key_pressed(KeyCode::Equal) || is_key_pressed(KeyCode::KpAdd) {
            self.camera.zoom *= 1.1;
        }
        if is_key_pressed(KeyCode::Minus) || is_key_pressed(KeyCode::KpSubtract) {
            self.camera.zoom /= 1.1;
            if self.camera.zoom < 0.1 { self.camera.zoom = 0.1; }
        }

        if let (Some(tx), Some(ty)) = (self.target_x, self.target_y) {
            let dx = tx - self.ship.x;
            let dy = ty - self.ship.y;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist > 50.0 {
                self.ship.vx += (dx / dist) * super::SHIP_SPEED * dt;
                self.ship.vy += (dy / dist) * super::SHIP_SPEED * dt;
            } else {
                self.target_x = None;
                self.target_y = None;
            }
        }

        self.ship.vx *= super::FRICTION;
        self.ship.vy *= super::FRICTION;

        let speed = (self.ship.vx * self.ship.vx + self.ship.vy * self.ship.vy).sqrt();
        if speed > super::MAX_SPEED {
            self.ship.vx = (self.ship.vx / speed) * super::MAX_SPEED;
            self.ship.vy = (self.ship.vy / speed) * super::MAX_SPEED;
        }

        // Face direction of movement (towards target)
        if speed > 1.0 {
            self.ship.rotation = self.ship.vy.atan2(self.ship.vx);
        }

        self.ship.x += self.ship.vx * dt;
        self.ship.y += self.ship.vy * dt;

        self.camera.x += (self.ship.x - self.camera.x) * 3.0 * dt;
        self.camera.y += (self.ship.y - self.camera.y) * 3.0 * dt;

        self.update_chunks();
    }

    pub fn draw(&self) {
        clear_background(BLACK);

        let (screen_w, screen_h) = (screen_width(), screen_height());

        for ((cx, cy), chunk) in &self.chunks {
            let chunk_world_x = *cx as f32 * super::CHUNK_SIZE as f32;
            let chunk_world_y = *cy as f32 * super::CHUNK_SIZE as f32;

            for star in &chunk.stars {
                let world_x = star.x + chunk_world_x;
                let world_y = star.y + chunk_world_y;
                let screen_x = (world_x - self.camera.x) * self.camera.zoom + screen_w / 2.0;
                let screen_y = (world_y - self.camera.y) * self.camera.zoom + screen_h / 2.0;

                if screen_x > -50.0 && screen_x < screen_w + 50.0
                    && screen_y > -50.0 && screen_y < screen_h + 50.0
                {
                    let draw_size = star.size;
                    draw_circle(screen_x, screen_y, draw_size, Color::new(
                        star.brightness,
                        star.brightness,
                        star.brightness,
                        1.0
                    ));
                }
            }
        }

        let ship_screen_x = (self.ship.x - self.camera.x) * self.camera.zoom + screen_w / 2.0;
        let ship_screen_y = (self.ship.y - self.camera.y) * self.camera.zoom + screen_h / 2.0;

        let vx = (self.ship.vx * self.ship.vx + self.ship.vy * self.ship.vy).sqrt();

        draw_texture_ex(
            &self.ship_texture,
            ship_screen_x - self.ship.size * self.camera.zoom / 2.0,
            ship_screen_y - self.ship.size * self.camera.zoom / 2.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(self.ship.size * self.camera.zoom, self.ship.size * self.camera.zoom)),
                // Pivot in screen-space: rotate around ship center
                pivot: Some(Vec2::new(ship_screen_x, ship_screen_y)),
                rotation: self.ship.rotation + std::f32::consts::PI - std::f32::consts::FRAC_PI_2,
                ..Default::default()
            },
        );

        // Draw engine flames (size fixed on screen; unaffected by zoom)
        let speed = (self.ship.vx * self.ship.vx + self.ship.vy * self.ship.vy).sqrt();
        if speed > 1.0 {
            // Use the same rotation as the rendered ship texture so flames stay attached to the engines.
            let render_rot = self.ship.rotation + std::f32::consts::PI - std::f32::consts::FRAC_PI_2;
            let scale = self.ship.size / TEXTURE_WIDTH;
            let cos_r = render_rot.cos();
            let sin_r = render_rot.sin();

            let ex1 = self.ship.x + (ENGINE1_DX * scale) * cos_r - (ENGINE1_DY * scale) * sin_r;
            let ey1 = self.ship.y + (ENGINE1_DX * scale) * sin_r + (ENGINE1_DY * scale) * cos_r;
            let ex2 = self.ship.x + (ENGINE2_DX * scale) * cos_r - (ENGINE2_DY * scale) * sin_r;
            let ey2 = self.ship.y + (ENGINE2_DX * scale) * sin_r + (ENGINE2_DY * scale) * cos_r;

            // Flame size tied to ship's screen size and speed (varies more with speed)
            let ship_screen_size = self.ship.size * self.camera.zoom;
            let speed_ratio = (speed / 200.0).clamp(0.5, 2.0); // allows lengthening up to 2x at high speeds
            let flame_len = 0.4 * ship_screen_size * speed_ratio;
            let flame_width = 0.1 * ship_screen_size;

            // Direction in screen space (match rendered ship orientation)
            let dir_x = render_rot.cos();
            let dir_y = render_rot.sin();
            // Rotate flame direction 90° CW (so it points downward on screen)
            let back_x = -dir_y;
            let back_y = dir_x;
            let perp_x = -back_y;
            let perp_y = back_x;

            let draw_flame = |base_world_x: f32, base_world_y: f32| {
                let base_sx = (base_world_x - self.camera.x) * self.camera.zoom + screen_w / 2.0;
                let base_sy = (base_world_y - self.camera.y) * self.camera.zoom + screen_h / 2.0;

                let tip_sx = base_sx + back_x * flame_len;
                let tip_sy = base_sy + back_y * flame_len;

                let base1_sx = base_sx + perp_x * flame_width;
                let base1_sy = base_sy + perp_y * flame_width;
                let base2_sx = base_sx - perp_x * flame_width;
                let base2_sy = base_sy - perp_y * flame_width;

                // Outer glow
                draw_triangle(
                    vec2(base1_sx, base1_sy),
                    vec2(base2_sx, base2_sy),
                    vec2(tip_sx, tip_sy),
                    Color::new(1.0, 0.6, 0.0, 0.5),
                );
                // Inner core
                draw_triangle(
                    vec2(base1_sx, base1_sy),
                    vec2(base2_sx, base2_sy),
                    vec2(tip_sx, tip_sy),
                    Color::new(1.0, 0.9, 0.2, 0.7),
                );
            };

            draw_flame(ex1, ey1);
            draw_flame(ex2, ey2);
        }

        draw_text(
            &format!("Pos: {:.0}, {:.0}", self.ship.x, self.ship.y),
            10.0,
            20.0,
            20.0,
            WHITE,
        );
        draw_text(
            &format!("Vel: {:.0}", vx),
            10.0,
            45.0,
            20.0,
            WHITE,
        );
        draw_text(
            &format!("Chunks: {}", self.chunks.len()),
            10.0,
            70.0,
            20.0,
            WHITE,
        );
        draw_text(
            &format!("Zoom: {:.2}x", self.camera.zoom),
            10.0,
            95.0,
            20.0,
            WHITE,
        );

        if let (Some(tx), Some(ty)) = (self.target_x, self.target_y) {
            let dist = ((tx - self.ship.x).powi(2) + (ty - self.ship.y).powi(2)).sqrt();
            let dist_text = if dist < 1000.0 {
                format!("Distance: {:.0} m", dist)
            } else if dist < 1_000_000.0 {
                format!("Distance: {:.2} km", dist / 1000.0)
            } else {
                format!("Distance: {:.4} Mm", dist / 1_000_000.0)
            };
            draw_text(
                &dist_text,
                10.0,
                120.0,
                20.0,
                GREEN,
            );

            let target_screen_x = (tx - self.camera.x) * self.camera.zoom + screen_w / 2.0;
            let target_screen_y = (ty - self.camera.y) * self.camera.zoom + screen_h / 2.0;
            draw_circle_lines(target_screen_x, target_screen_y, 10.0 * self.camera.zoom, 2.0, GREEN);
        }

        draw_text("Click to move, +/- to zoom", 10.0, screen_h - 20.0, 20.0, GRAY);

        self.ui.draw();
    }
}
