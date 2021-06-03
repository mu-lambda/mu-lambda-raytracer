use crate::textures::Texture;
use crate::vec::{Color, Point3};
use image::RgbImage;

#[derive(Clone)]
pub struct Image {
    image: std::sync::Arc<RgbImage>,
}

impl Image {
    pub fn new(image: RgbImage) -> Image {
        Image { image: std::sync::Arc::new(image) }
    }
}

impl Texture for Image {
    fn value(&self, u: f64, v: f64, _: Point3) -> Color {
        let u = u.clamp(0.0, 1.0);
        let v = (1.0 - v).clamp(0.0, 1.0);

        let (width, height) = self.image.dimensions();
        let i = (u * (width as f64)) as u32;
        let j = (v * (height as f64)) as u32;
        let i = i.clamp(0, width - 1);
        let j = j.clamp(0, height - 1);
        let pixel = self.image.get_pixel(i, j);
        Color::new((pixel[0] as f64) / 255.0, (pixel[1] as f64) / 255.0, (pixel[2] as f64) / 255.0)
    }
}
