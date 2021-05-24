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

pub struct Checker<TOdd: Texture, TEven: Texture> {
    odd: TOdd,
    even: TEven,
}

impl<TOdd: Texture, TEven: Texture> Checker<TOdd, TEven> {
    pub fn new(odd: TOdd, even: TEven) -> Checker<TOdd, TEven> {
        Checker { odd, even }
    }
}

impl<TOdd: Texture, TEven: Texture> Texture for Checker<TOdd, TEven> {
    fn value(&self, u: f64, v: f64, p: Point3) -> Color {
        let sines = (10.0 * p.x()).sin() * (10.0 * p.y()).sin() * (10.0 * p.z()).sin();
        if sines < 0.0 {
            self.odd.value(u, v, p)
        } else {
            self.even.value(u, v, p)
        }
    }
}
