use macroquad::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ButtonAction {
    Placeholder,
    ToggleMap,
}

#[derive(Debug, Clone)]
pub struct Button {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub texture: Option<Texture2D>,
    pub is_hovered: bool,
    pub action: ButtonAction,
}

impl Button {
    fn new(x: f32, y: f32, width: f32, height: f32, action: ButtonAction) -> Self {
        Self {
            x,
            y,
            width,
            height,
            texture: None,
            is_hovered: false,
            action,
        }
    }

    pub fn update(&mut self) {
        let (mx, my) = mouse_position();
        self.is_hovered =
            mx >= self.x && mx <= self.x + self.width && my >= self.y && my <= self.y + self.height;
    }

    pub fn contains(&self, point: Vec2) -> bool {
        point.x >= self.x
            && point.x <= self.x + self.width
            && point.y >= self.y
            && point.y <= self.y + self.height
    }

    pub fn draw(&self) {
        let bg = if self.is_hovered {
            Color::new(0.13, 0.16, 0.22, 0.95)
        } else {
            Color::new(0.08, 0.10, 0.14, 0.86)
        };

        draw_rectangle(self.x, self.y, self.width, self.height, bg);
        draw_rectangle_lines(
            self.x,
            self.y,
            self.width,
            self.height,
            1.0,
            Color::new(0.22, 0.26, 0.33, 0.9),
        );

        if let Some(tex) = &self.texture {
            let tint = if self.is_hovered {
                WHITE
            } else {
                Color::new(0.82, 0.86, 0.92, 1.0)
            };
            let icon_size = self.height.min(self.width) * 0.62;
            let icon_x = self.x + (self.width - icon_size) / 2.0;
            let icon_y = self.y + (self.height - icon_size) / 2.0;
            draw_texture_ex(
                tex,
                icon_x,
                icon_y,
                tint,
                DrawTextureParams {
                    dest_size: Some(Vec2::new(icon_size, icon_size)),
                    ..Default::default()
                },
            );
        }
    }
}

pub struct UIPanel {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub buttons: Vec<Button>,
    pub visible: bool,
}

impl UIPanel {
    pub async fn new(_screen_width: f32, screen_height: f32) -> Self {
        let map_tex = load_texture("src/icons/map.png").await.ok();

        let mut buttons = vec![
            Button::new(0.0, 0.0, 0.0, 0.0, ButtonAction::Placeholder),
            Button::new(0.0, 0.0, 0.0, 0.0, ButtonAction::Placeholder),
            Button::new(0.0, 0.0, 0.0, 0.0, ButtonAction::Placeholder),
            Button::new(0.0, 0.0, 0.0, 0.0, ButtonAction::Placeholder),
            Button::new(0.0, 0.0, 0.0, 0.0, ButtonAction::Placeholder),
            Button::new(0.0, 0.0, 0.0, 0.0, ButtonAction::ToggleMap),
            Button::new(0.0, 0.0, 0.0, 0.0, ButtonAction::Placeholder),
        ];

        if let Some(button) = buttons.get_mut(5) {
            button.texture = map_tex;
        }

        let mut panel = Self {
            x: 16.0,
            y: screen_height * 0.14,
            width: 52.0,
            height: 0.0,
            buttons,
            visible: true,
        };
        panel.layout(screen_height);
        panel
    }

    fn layout(&mut self, screen_height: f32) {
        self.width = 52.0;
        self.x = 16.0;
        self.y = (screen_height * 0.14).clamp(18.0, screen_height - 460.0);

        let button_height = 48.0;
        let gap = 8.0;
        let inner_padding = 8.0;
        let button_width = self.width - inner_padding * 2.0;

        let mut y = self.y + inner_padding;
        for button in &mut self.buttons {
            button.x = self.x + inner_padding;
            button.y = y;
            button.width = button_width;
            button.height = button_height;
            y += button_height + gap;
        }

        self.height = y - self.y - gap + inner_padding;
    }

    pub fn update(&mut self) -> Option<ButtonAction> {
        if !self.visible {
            return None;
        }

        self.layout(screen_height());

        for button in &mut self.buttons {
            button.update();
        }

        if is_mouse_button_pressed(MouseButton::Left) {
            let mouse = mouse_vec();
            for button in &self.buttons {
                if button.contains(mouse) {
                    return Some(button.action.clone());
                }
            }
        }

        None
    }

    pub fn is_mouse_over(&self) -> bool {
        if !self.visible {
            return false;
        }

        let mouse = mouse_vec();
        let panel_bounds = Rect::new(self.x - 8.0, self.y - 8.0, self.width + 16.0, self.height + 16.0);
        panel_bounds.contains(mouse)
    }

    pub fn draw(&self) {
        if !self.visible {
            return;
        }

        let outer = Rect::new(self.x - 6.0, self.y - 8.0, self.width + 12.0, self.height + 16.0);
        draw_rectangle(outer.x, outer.y, outer.w, outer.h, Color::new(0.02, 0.03, 0.05, 0.80));
        draw_rectangle_lines(outer.x, outer.y, outer.w, outer.h, 1.0, Color::new(0.20, 0.24, 0.30, 0.95));

        draw_rectangle(
            self.x,
            self.y,
            self.width,
            self.height,
            Color::new(0.06, 0.07, 0.10, 0.96),
        );

        draw_rectangle(
            self.x,
            self.y,
            self.width,
            18.0,
            Color::new(0.10, 0.12, 0.16, 0.9),
        );

        for (index, button) in self.buttons.iter().enumerate() {
            if index > 0 {
                let sep_y = button.y - 4.0;
                draw_line(
                    self.x + 12.0,
                    sep_y,
                    self.x + self.width - 12.0,
                    sep_y,
                    1.0,
                    Color::new(0.16, 0.18, 0.24, 0.8),
                );
            }

            button.draw();
        }
    }

    pub fn handle_game_action(_action: &ButtonAction, _game: &mut crate::game::GameState) {}
}

fn mouse_vec() -> Vec2 {
    let (x, y) = mouse_position();
    vec2(x, y)
}
