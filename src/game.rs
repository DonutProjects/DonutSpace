use macroquad::prelude::*;
use std::collections::HashMap;

use crate::camera::Camera;
use crate::ship::Ship;
use crate::star::{Planet, StarChunk, StarSystem, Station};
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
    pub station_texture: Option<Texture2D>,
    pub ui: UIPanel,
}

impl GameState {
    pub async fn new() -> Self {
        let ship_texture = load_texture("src/ship1.png").await.unwrap();
        let station_texture = load_optional_texture(&[
            "src/station.png",
            "src/station1.png",
            "src/icons/station.png",
            "src/icons/station1.png",
        ])
        .await;
        let mut chunks = HashMap::new();
        let hub_chunk = StarChunk::new(0, 0, 0);
        let spawn_position = hub_chunk
            .systems
            .iter()
            .find(|system| system.is_trade_hub)
            .and_then(|system| system.first_station_world_position(0, 0))
            .unwrap_or((super::CHUNK_SIZE as f32 * 0.5, super::CHUNK_SIZE as f32 * 0.5));
        chunks.insert((0, 0), hub_chunk);

        Self {
            ship: Ship {
                x: spawn_position.0 + 4_500.0,
                y: spawn_position.1 + 2_000.0,
                vx: 0.0,
                vy: 0.0,
                rotation: 0.0,
                size: super::SHIP_SIZE,
            },
            camera: Camera {
                x: spawn_position.0,
                y: spawn_position.1,
                zoom: 0.0025,
            },
            chunks,
            target_x: None,
            target_y: None,
            ship_texture,
            station_texture,
            ui: UIPanel::new(screen_width(), screen_height()).await,
        }
    }

    pub fn update_chunks(&mut self) {
        let view_radius = 2;
        let center_cx = (self.camera.x / super::CHUNK_SIZE as f32).floor() as i32;
        let center_cy = (self.camera.y / super::CHUNK_SIZE as f32).floor() as i32;

        for dx in -view_radius..=view_radius {
            for dy in -view_radius..=view_radius {
                let cx = center_cx + dx;
                let cy = center_cy + dy;
                let key = (cx, cy);
                if !self.chunks.contains_key(&key) {
                    let seed = cx as i64 * 1000000 + cy as i64;
                    self.chunks.insert(key, StarChunk::new(seed, cx, cy));
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
            if self.camera.zoom < 0.0002 { self.camera.zoom = 0.0002; }
        }

        // Handle zoom with keys (optional)
        if is_key_pressed(KeyCode::Equal) || is_key_pressed(KeyCode::KpAdd) {
            self.camera.zoom *= 1.1;
        }
        if is_key_pressed(KeyCode::Minus) || is_key_pressed(KeyCode::KpSubtract) {
            self.camera.zoom /= 1.1;
            if self.camera.zoom < 0.0002 { self.camera.zoom = 0.0002; }
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

            for system in &chunk.systems {
                let system_world_x = chunk_world_x + system.x;
                let system_world_y = chunk_world_y + system.y;
                self.draw_star_system(system, system_world_x, system_world_y, screen_w, screen_h);
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
            &format!("Systems: {}", self.chunks.values().map(|chunk| chunk.systems.len()).sum::<usize>()),
            10.0,
            95.0,
            20.0,
            WHITE,
        );
        draw_text(
            &format!(
                "Stations: {}",
                self.chunks
                    .values()
                    .flat_map(|chunk| &chunk.systems)
                    .map(|system| {
                        system
                            .planets
                            .iter()
                            .map(|planet| planet.stations.len())
                            .sum::<usize>()
                    })
                    .sum::<usize>()
            ),
            10.0,
            120.0,
            20.0,
            WHITE,
        );
        draw_text(
            &format!("Zoom: {:.2}x", self.camera.zoom),
            10.0,
            145.0,
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
                170.0,
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

    fn draw_star_system(
        &self,
        system: &StarSystem,
        system_world_x: f32,
        system_world_y: f32,
        screen_w: f32,
        screen_h: f32,
    ) {
        let center = world_to_screen(system_world_x, system_world_y, &self.camera, screen_w, screen_h);
        if center.x < -300.0 || center.x > screen_w + 300.0 || center.y < -300.0 || center.y > screen_h + 300.0 {
            return;
        }

        let star_color = Color::new(system.star_color.0, system.star_color.1, system.star_color.2, 1.0);
        let glow_radius = (system.star_radius * self.camera.zoom * 2.4).max(16.0);
        draw_circle(
            center.x,
            center.y,
            glow_radius,
            Color::new(system.star_color.0, system.star_color.1, system.star_color.2, 0.12),
        );
        draw_circle(
            center.x,
            center.y,
            (system.star_radius * self.camera.zoom).max(4.0),
            star_color,
        );

        for planet in &system.planets {
            self.draw_planet(system, planet, system_world_x, system_world_y, screen_w, screen_h);
        }

        if system.is_trade_hub && self.camera.zoom > 0.35 {
            let label = "Trade Hub";
            let dims = measure_text(label, None, 18, 1.0);
            draw_text(
                label,
                center.x - dims.width / 2.0,
                center.y - glow_radius - 12.0,
                18.0,
                Color::new(0.98, 0.88, 0.56, 1.0),
            );
        }
    }

    fn draw_planet(
        &self,
        system: &StarSystem,
        planet: &Planet,
        system_world_x: f32,
        system_world_y: f32,
        screen_w: f32,
        screen_h: f32,
    ) {
        let orbit_offset = vec2(
            planet.orbit_radius * planet.angle.cos(),
            planet.orbit_radius * planet.angle.sin(),
        );
        let planet_world_x = system_world_x + orbit_offset.x;
        let planet_world_y = system_world_y + orbit_offset.y;
        let planet_screen = world_to_screen(planet_world_x, planet_world_y, &self.camera, screen_w, screen_h);

        if self.camera.zoom > 0.22 {
            let system_screen = world_to_screen(system_world_x, system_world_y, &self.camera, screen_w, screen_h);
            draw_circle_lines(
                system_screen.x,
                system_screen.y,
                planet.orbit_radius * self.camera.zoom,
                1.0,
                Color::new(0.14, 0.18, 0.24, 0.55),
            );
        }

        let planet_color = Color::new(planet.color.0, planet.color.1, planet.color.2, 1.0);
        draw_circle(
            planet_screen.x,
            planet_screen.y,
            (planet.size * self.camera.zoom).max(3.0),
            planet_color,
        );

        if system.is_trade_hub && self.camera.zoom > 0.45 && !planet.stations.is_empty() {
            draw_circle_lines(
                planet_screen.x,
                planet_screen.y,
                (planet.size * self.camera.zoom).max(3.0) + 4.0,
                1.0,
                Color::new(0.95, 0.80, 0.48, 0.6),
            );
        }

        for station in &planet.stations {
            self.draw_station(planet_world_x, planet_world_y, planet, station, screen_w, screen_h);
        }
    }

    fn draw_station(
        &self,
        planet_world_x: f32,
        planet_world_y: f32,
        planet: &Planet,
        station: &Station,
        screen_w: f32,
        screen_h: f32,
    ) {
        let station_world_x = planet_world_x + station.orbit_radius * station.angle.cos();
        let station_world_y = planet_world_y + station.orbit_radius * station.angle.sin();
        let station_screen = world_to_screen(station_world_x, station_world_y, &self.camera, screen_w, screen_h);
        let planet_screen = world_to_screen(planet_world_x, planet_world_y, &self.camera, screen_w, screen_h);

        if self.camera.zoom > 0.45 {
            draw_line(
                planet_screen.x,
                planet_screen.y,
                station_screen.x,
                station_screen.y,
                1.0,
                Color::new(0.35, 0.45, 0.56, 0.45),
            );
        }

        let station_size = (station.size * self.camera.zoom).max(8.0);
        if let Some(texture) = &self.station_texture {
            draw_texture_ex(
                texture,
                station_screen.x - station_size / 2.0,
                station_screen.y - station_size / 2.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(Vec2::new(station_size, station_size)),
                    rotation: station.angle,
                    ..Default::default()
                },
            );
        } else {
            // Fallback so generation still works if the station sprite file is absent in the repo.
            draw_circle(
                station_screen.x,
                station_screen.y,
                station_size * 0.38,
                Color::new(0.72, 0.78, 0.85, 1.0),
            );
            draw_rectangle(
                station_screen.x - station_size * 0.45,
                station_screen.y - station_size * 0.12,
                station_size * 0.9,
                station_size * 0.24,
                Color::new(0.50, 0.58, 0.68, 1.0),
            );
            draw_rectangle(
                station_screen.x - station_size * 0.12,
                station_screen.y - station_size * 0.45,
                station_size * 0.24,
                station_size * 0.9,
                Color::new(0.50, 0.58, 0.68, 1.0),
            );
        }

        if self.camera.zoom > 0.9 {
            draw_circle_lines(
                planet_screen.x,
                planet_screen.y,
                (planet.size + station.orbit_radius) * self.camera.zoom,
                1.0,
                Color::new(0.18, 0.24, 0.30, 0.25),
            );
        }
    }
}

fn world_to_screen(world_x: f32, world_y: f32, camera: &Camera, screen_w: f32, screen_h: f32) -> Vec2 {
    vec2(
        (world_x - camera.x) * camera.zoom + screen_w / 2.0,
        (world_y - camera.y) * camera.zoom + screen_h / 2.0,
    )
}

async fn load_optional_texture(paths: &[&str]) -> Option<Texture2D> {
    for path in paths {
        if let Ok(texture) = load_texture(path).await {
            return Some(texture);
        }
    }
    None
}
