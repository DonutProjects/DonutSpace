use macroquad::prelude::*;

#[derive(Debug, Clone)]
pub struct Ship {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub rotation: f32,
    pub size: f32,
}