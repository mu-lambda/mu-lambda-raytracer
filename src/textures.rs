use crate::vec::{Color, Point3};

pub trait Texture: Sync {
    fn value(&self, u: f64, v: f64, p: Point3) -> Color;
}

pub struct SolidColor {
    color: Color,
}

impl SolidColor {
    pub fn new(r: f64, g: f64, b: f64) -> SolidColor {
        SolidColor::from_color(Color::new(r, g, b))
    }
    pub fn from_color(color: Color) -> SolidColor {
        SolidColor { color }
    }
}

impl Texture for SolidColor {
    fn value(&self, _: f64, _: f64, _: Point3) -> Color {
        self.color
    }
}
