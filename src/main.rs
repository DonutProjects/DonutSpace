mod camera;
mod game;
mod ship;
mod star;

use macroquad::prelude::*;

const SHIP_SIZE: f32 = 64.0;
const SHIP_SPEED: f32 = 500.0;
const FRICTION: f32 = 0.95;
const MAX_SPEED: f32 = 1000.0;
const CHUNK_SIZE: i32 = 500;
const STARS_PER_CHUNK: u32 = 10;

#[macroquad::main("Space Game")]
async fn main() {
    let mut game = game::GameState::new().await;

    loop {
        let (mx, my) = mouse_position();
        if is_mouse_button_pressed(MouseButton::Left) {
            let world_x = (mx - screen_width() / 2.0) / game.camera.zoom + game.camera.x;
            let world_y = (my - screen_height() / 2.0) / game.camera.zoom + game.camera.y;
            game.target_x = Some(world_x);
            game.target_y = Some(world_y);
        }

        game.update(get_frame_time());

        game.draw();

        next_frame().await;
    }
}
