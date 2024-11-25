use glam::Vec2;
#[cfg(feature = "serde")]
use serde::Serialize;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    fn new(pos: Vec2, w: f32, h: f32) -> Self {
        let Vec2 { x, y } = pos;
        Self { x, y, w, h }
    }
    pub fn with_size(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }
}
