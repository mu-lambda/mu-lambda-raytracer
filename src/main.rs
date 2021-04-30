use std::io::{self, Write};

fn main() {

    // Image

    let image_width = 256;
    let image_height = 256;


    // Render
    
    println!("P3\n{} {}\n255", image_width, image_height);
    for j in (0..image_height).rev() {
        eprint!("\rScanlines remaining: {}", j); io::stderr().flush().unwrap();
        for i in 0..image_width {
            let r = f64::from(i) / f64::from(image_width - 1);
            let g = f64::from(j) / f64::from(image_height - 1);
            let b = 0.25f64;

            let ir = (255.999f64 * r) as i32;
            let ig = (255.999f64 * g) as i32;
            let ib = (255.999f64 * b) as i32;

            println!("{} {} {}", ir, ig, ib);

        }
    }
}

