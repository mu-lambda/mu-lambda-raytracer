use crate::bhv;
use crate::hittable::Hittable;
use crate::materials::{Dielectric, Lambertian, Metal};
use crate::shapes::Sphere;
use crate::textures::{SolidColor, Texture};
use crate::vec::{Color, Point3};
use rand::Rng;

pub fn simple_world<'a>() -> Box<dyn Hittable + 'a> {
    let mat_ground = Lambertian::new(SolidColor::new(0.8, 0.8, 0.0));
    let mat_center = Lambertian::new(SolidColor::new(0.1, 0.3, 0.5));
    let mat_left = Dielectric::new(1.5);
    let mat_right = Metal::new(Color::new(0.8, 0.6, 0.2), 0.0);

    let mut world = bhv::SceneBuilder::new();

    world
        .add(Sphere::new(Point3::new(0.0, -100.5, -1.0), 100.0, mat_ground))
        .add(Sphere::new(Point3::new(0.0, 0.0, -1.0), 0.5, mat_center))
        .add(Sphere::new(Point3::new(-1.0, 0.0, -1.0), 0.5, mat_left.clone()))
        .add(Sphere::new(Point3::new(-1.0, 0.0, -1.0), -0.4, mat_left))
        .add(Sphere::new(Point3::new(1.0, 0.0, -1.0), 0.5, mat_right));

    let bhv = bhv::BHV::new(&mut world);
    Box::new(bhv)
}

fn rnd01(rng: &mut dyn rand::RngCore) -> f64 {
    rng.gen_range(0.0..1.0)
}

pub fn random_world<'a>(rng: &mut dyn rand::RngCore) -> Box<dyn Hittable + 'a> {
    let mut world = bhv::SceneBuilder::new();

    let ground_material = Lambertian::new(SolidColor::new(0.5, 0.5, 0.5));
    world.add(Sphere::new(Point3::new(0.0, -1000.0, 0.0), 1000.0, ground_material));

    for a in -11..11 {
        for b in -11..11 {
            let choose_mat = rnd01(rng);
            let center = Point3::new(a as f64 + 0.9 * rnd01(rng), 0.2, b as f64 + 0.9 * rnd01(rng));

            if (center - Point3::new(4.0, 0.2, 0.0)).length() > 0.9 {
                if choose_mat < 0.8 {
                    let albedo = Color::random_unit(rng) * Color::random_unit(rng);
                    world.add(Sphere::new(
                        center,
                        0.2,
                        Lambertian::new(SolidColor::from_color(albedo)),
                    ));
                } else if choose_mat < 0.95 {
                    let albedo = Color::random(0.5, 1.0, rng);
                    let fuzz = rand::thread_rng().gen_range(0.0..0.5);
                    world.add(Sphere::new(center, 0.2, Metal::new(albedo, fuzz)));
                } else {
                    world.add(Sphere::new(center, 0.2, Dielectric::new(1.5)));
                }
            }
        }
    }

    world
        .add(Sphere::new(Point3::new(0.0, 1.0, 0.0), 1.0, Dielectric::new(1.5)))
        .add(Sphere::new(
            Point3::new(-4.0, 1.0, 0.0),
            1.0,
            Lambertian::new(SolidColor::new(0.4, 0.2, 0.1)),
        ))
        .add(Sphere::new(
            Point3::new(4.0, 1.0, 0.0),
            1.0,
            Metal::new(Color::new(0.7, 0.6, 0.5), 0.0),
        ));

    Box::new(bhv::BHV::new(&mut world))
}

pub enum World {
    Simple,
    Random,
}
