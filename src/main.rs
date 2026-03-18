use macroquad::prelude::*;
use ::rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::collections::HashMap;

const SHIP_SIZE: f32 = 64.0;
const SHIP_SPEED: f32 = 500.0;
const FRICTION: f32 = 0.95;
const MAX_SPEED: f32 = 1000.0;
const ROTATION_SPEED: f32 = 3.0; // radians per second
const CHUNK_SIZE: i32 = 500;
const STARS_PER_CHUNK: u32 = 30;

struct Ship {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    rotation: f32,
    size: f32,
}

struct Camera {
    x: f32,
    y: f32,
    zoom: f32,
}

struct Star {
    x: f32,
    y: f32,
    size: f32,
    brightness: f32,
}

struct StarChunk {
    stars: Vec<Star>,
}

impl StarChunk {
    fn new(seed: i64) -> Self {
        let mut rng: ChaCha8Rng = SeedableRng::seed_from_u64(seed as u64);
        let mut stars = Vec::new();
        for _ in 0..STARS_PER_CHUNK {
            stars.push(Star {
                x: rng.gen_range(0.0..CHUNK_SIZE as f32),
                y: rng.gen_range(0.0..CHUNK_SIZE as f32),
                size: rng.gen_range(0.5..2.5),
                brightness: rng.gen_range(0.5..1.0),
            });
        }
        Self { stars }
    }
}

struct GameState {
    ship: Ship,
    camera: Camera,
    chunks: HashMap<(i32, i32), StarChunk>,
    target_x: Option<f32>,
    target_y: Option<f32>,
    ship_texture: Texture2D,
}

impl GameState {
    async fn new() -> Self {
        let ship_texture = load_texture("src/ship1.png").await.unwrap();

        Self {
            ship: Ship {
                x: 0.0,
                y: 0.0,
                vx: 0.0,
                vy: 0.0,
                rotation: 0.0,
                size: SHIP_SIZE,
            },
            camera: Camera { x: 0.0, y: 0.0, zoom: 1.0 },
            chunks: HashMap::new(),
            target_x: None,
            target_y: None,
            ship_texture,
        }
    }

    fn update_chunks(&mut self) {
        let view_radius = 5;
        let center_cx = (self.camera.x / CHUNK_SIZE as f32).floor() as i32;
        let center_cy = (self.camera.y / CHUNK_SIZE as f32).floor() as i32;

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
            if dist_x > view_radius + 2 || dist_y > view_radius + 2 {
                self.chunks.remove(&key);
            }
        }
    }

    fn update(&mut self, dt: f32) {
        // Handle zoom
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
                self.ship.vx += (dx / dist) * SHIP_SPEED * dt;
                self.ship.vy += (dy / dist) * SHIP_SPEED * dt;

                // Smooth rotation towards target
                let desired_rotation = dy.atan2(dx);
                let mut angle_diff = (desired_rotation - self.ship.rotation).rem_euclid(2.0 * std::f32::consts::PI);
                if angle_diff > std::f32::consts::PI {
                    angle_diff -= 2.0 * std::f32::consts::PI;
                }
                let rotation_dir = angle_diff.signum();
                let rotation_amount = (angle_diff.abs()).min(ROTATION_SPEED * dt);
                self.ship.rotation += rotation_dir * rotation_amount;
                self.ship.rotation = self.ship.rotation.rem_euclid(2.0 * std::f32::consts::PI);
            } else {
                self.target_x = None;
                self.target_y = None;
            }
        }

        self.ship.vx *= FRICTION;
        self.ship.vy *= FRICTION;

        let speed = (self.ship.vx * self.ship.vx + self.ship.vy * self.ship.vy).sqrt();
        if speed > MAX_SPEED {
            self.ship.vx = (self.ship.vx / speed) * MAX_SPEED;
            self.ship.vy = (self.ship.vy / speed) * MAX_SPEED;
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

    fn draw(&self) {
        clear_background(BLACK);

        let (screen_w, screen_h) = (screen_width(), screen_height());
        let offset_x = screen_w / 2.0 - self.camera.x * self.camera.zoom;
        let offset_y = screen_h / 2.0 - self.camera.y * self.camera.zoom;

        for ((cx, cy), chunk) in &self.chunks {
            let chunk_world_x = *cx as f32 * CHUNK_SIZE as f32;
            let chunk_world_y = *cy as f32 * CHUNK_SIZE as f32;

            for star in &chunk.stars {
                let world_x = star.x + chunk_world_x;
                let world_y = star.y + chunk_world_y;
                let screen_x = (world_x - self.camera.x) * self.camera.zoom + screen_w / 2.0;
                let screen_y = (world_y - self.camera.y) * self.camera.zoom + screen_h / 2.0;

                if screen_x > -10.0 && screen_x < screen_w + 10.0
                    && screen_y > -10.0 && screen_y < screen_h + 10.0
                {
                    draw_circle(screen_x, screen_y, star.size * self.camera.zoom, Color::new(
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

        if self.target_x.is_some() {
            let tx = self.target_x.unwrap() + offset_x;
            let ty = self.target_y.unwrap() + offset_y;
            draw_circle_lines(tx, ty, 10.0, 2.0, GREEN);
        }

        draw_text("Click to move, +/- to zoom", 10.0, screen_h - 20.0, 20.0, GRAY);
    }
}

#[macroquad::main("Space Game")]
async fn main() {
    let mut game = GameState::new().await;

    loop {
        let (mx, my) = mouse_position();
        if is_mouse_button_pressed(MouseButton::Left) {
            game.target_x = Some(mx - screen_width() / 2.0 + game.camera.x);
            game.target_y = Some(my - screen_height() / 2.0 + game.camera.y);
        }

        game.update(get_frame_time());

        game.draw();

        next_frame().await;
    }
}
