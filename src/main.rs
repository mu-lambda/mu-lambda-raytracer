use std::io::{self, Write};
mod datatypes;
use datatypes::{write_color, Color, Vec3};

fn main() {
    // Image

    let image_width = 256;
    let image_height = 256;

    let v = datatypes::Vec3::new(1.0, 1.0, 1.0);

    // Render

    println!("P3\n{} {}\n255", image_width, image_height);
    for j in (0..image_height).rev() {
        eprint!("\rScanlines remaining: {}", j);
        io::stderr().flush().unwrap();
        for i in 0..image_width {
            let c = datatypes::Color::new(
                f64::from(i) / f64::from(image_width - 1),
                f64::from(j) / f64::from(image_height - 1),
                0.25f64,
            );

            datatypes::write_color(&c, &mut std::io::stdout());
        }
    }
}
