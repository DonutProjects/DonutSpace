mod camera;
mod game;
mod ship;
mod star;
mod ui;
use macroquad::prelude::*;

const SHIP_SIZE: f32 = 64.0;
const SHIP_SPEED: f32 = 1000.0;
const FRICTION: f32 = 0.95;
const MAX_SPEED: f32 = 1000.0;
const CHUNK_SIZE: i32 = 2_000_000;
const STARS_PER_CHUNK: u32 = 1;

#[derive(Clone, Copy)]
enum AppState {
    Menu,
    Playing,
}

#[macroquad::main("DonutSpace")]
async fn main() {
    let mut app_state = AppState::Menu;
    let mut game = game::GameState::new().await;

    loop {
        match app_state {
            AppState::Menu => {
                if draw_menu() {
                    app_state = AppState::Playing;
                }
            }
            AppState::Playing => {
                let clicked_action = game.ui.update();
                let (mx, my) = mouse_position();
                if is_mouse_button_pressed(MouseButton::Left) && !game.ui.is_mouse_over() {
                    let world_x = (mx - screen_width() / 2.0) / game.camera.zoom + game.camera.x;
                    let world_y = (my - screen_height() / 2.0) / game.camera.zoom + game.camera.y;
                    game.target_x = Some(world_x);
                    game.target_y = Some(world_y);
                }

                game.update(get_frame_time());

                if let Some(action) = clicked_action {
                    ui::UIPanel::handle_game_action(&action, &mut game);
                }

                game.draw();
            }
        }

        next_frame().await;
    }
}

fn draw_menu() -> bool {
    clear_background(BLACK);

    let screen_w = screen_width();
    let screen_h = screen_height();

    // Title
    let title = "DONUT SPACE";
    let title_size = 60.0;
    let title_dims = measure_text(title, None, title_size as u16, 1.0);
    draw_text(
        title,
        screen_w / 2.0 - title_dims.width / 2.0,
        screen_h / 2.0 - 100.0,
        title_size,
        WHITE,
    );

    // Instructions
    let play_text = "Press SPACE to Start";
    let play_dims = measure_text(play_text, None, 30, 1.0);
    draw_text(
        play_text,
        screen_w / 2.0 - play_dims.width / 2.0,
        screen_h / 2.0,
        30.0,
        GREEN,
    );

    let exit_text = "Press ESC to Exit";
    let exit_dims = measure_text(exit_text, None, 30, 1.0);
    draw_text(
        exit_text,
        screen_w / 2.0 - exit_dims.width / 2.0,
        screen_h / 2.0 + 50.0,
        30.0,
        RED,
    );

    // Handle input
    if is_key_pressed(KeyCode::Space) {
        return true; // Start game
    }
    if is_key_pressed(KeyCode::Escape) {
        std::process::exit(0);
    }

    false
}
