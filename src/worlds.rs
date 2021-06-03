use crate::bhv;
use crate::hittable::{Hittable, HittableList};
use crate::materials::{Dielectric, DiffuseLight, Lambertian, Metal};
use crate::raytrace::{Background, BlackBackground, GradientBackground};
use crate::shapes::{Block, Sphere, XYRect, XZRect, YZRect};
use crate::textures;
use crate::textures::{NoiseTexture, SolidColor};
use crate::transforms;
use crate::vec::{Color, Point3, Vec3};
use rand::Rng;

pub trait World {
    fn name(&self) -> &'static str;
    fn camera(&self) -> WorldCamera;
    fn background(&self) -> Box<dyn Background>;
    fn build(&self, rng: &mut dyn rand::RngCore) -> Box<dyn Hittable>;
}

pub struct WorldCamera {
    pub lookfrom: Point3,
    pub lookat: Point3,
    pub field_of_view: f64,
}

struct Simple {}

impl World for Simple {
    fn name(&self) -> &'static str {
        "simple"
    }

    fn background(&self) -> Box<dyn Background> {
        Box::new(GradientBackground::default())
    }

    fn camera(&self) -> WorldCamera {
        WorldCamera {
            lookfrom: Point3::new(-2.0, 2.0, 1.0),
            lookat: Point3::new(0.0, 0.0, -1.0),
            field_of_view: 20.0,
        }
    }

    fn build(&self, rng: &mut dyn rand::RngCore) -> Box<dyn Hittable> {
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

        let bhv = bhv::BHV::new(&mut world, rng);
        Box::new(bhv)
    }
}

fn rnd01(rng: &mut dyn rand::RngCore) -> f64 {
    rng.gen_range(0.0..1.0)
}

struct Random {}

impl World for Random {
    fn name(&self) -> &'static str {
        "random"
    }
    fn background(&self) -> Box<dyn Background> {
        Box::new(GradientBackground::default())
    }
    fn camera(&self) -> WorldCamera {
        WorldCamera {
            lookfrom: Point3::new(13.0, 2.0, 3.0),
            lookat: Point3::new(0.0, 0.0, 0.0),
            field_of_view: 20.0,
        }
    }

    fn build(&self, rng: &mut dyn rand::RngCore) -> Box<dyn Hittable> {
        let mut world = bhv::SceneBuilder::new();

        let ground_material = Lambertian::new(SolidColor::new(0.5, 0.5, 0.5));
        world.add(Sphere::new(Point3::new(0.0, -1000.0, 0.0), 1000.0, ground_material));

        for a in -11..11 {
            for b in -11..11 {
                let choose_mat = rnd01(rng);
                let center =
                    Point3::new(a as f64 + 0.9 * rnd01(rng), 0.2, b as f64 + 0.9 * rnd01(rng));

                if (center - Point3::new(4.0, 0.2, 0.0)).length() > 0.9 {
                    if choose_mat < 0.8 {
                        let albedo = Color::random_unit(rng) * Color::random_unit(rng);
                        let solid = SolidColor::from_color(albedo);
                        world.add(Sphere::new(center, 0.2, Lambertian::new(solid)));
                    } else if choose_mat < 0.95 {
                        let albedo = Color::random(0.5, 1.0, rng);
                        let fuzz = rng.gen_range(0.0..0.5);
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

        Box::new(bhv::BHV::new(&mut world, rng))
    }
}

struct RandomChk {}

impl World for RandomChk {
    fn name(&self) -> &'static str {
        "random_chk"
    }
    fn background(&self) -> Box<dyn Background> {
        Box::new(GradientBackground::default())
    }

    fn camera(&self) -> WorldCamera {
        WorldCamera {
            lookfrom: Point3::new(13.0, 2.0, 3.0),
            lookat: Point3::new(0.0, 0.0, 0.0),
            field_of_view: 20.0,
        }
    }

    fn build(&self, rng: &mut dyn rand::RngCore) -> Box<dyn Hittable> {
        let mut world = bhv::SceneBuilder::new();

        let checker =
            textures::Checker::new(SolidColor::new(0.2, 0.3, 0.1), SolidColor::new(0.9, 0.9, 0.9));
        let ground_material = Lambertian::new(checker);
        world.add(Sphere::new(Point3::new(0.0, -1000.0, 0.0), 1000.0, ground_material));

        for a in -11..11 {
            for b in -11..11 {
                let choose_mat = rnd01(rng);
                let center =
                    Point3::new(a as f64 + 0.9 * rnd01(rng), 0.2, b as f64 + 0.9 * rnd01(rng));

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
                        let fuzz = rng.gen_range(0.0..0.5);
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

        Box::new(bhv::BHV::new(&mut world, rng))
    }
}

struct TwoSpheres {}

impl World for TwoSpheres {
    fn name(&self) -> &'static str {
        "two_spheres"
    }
    fn background(&self) -> Box<dyn Background> {
        Box::new(GradientBackground::default())
    }

    fn camera(&self) -> WorldCamera {
        WorldCamera {
            lookfrom: Point3::new(13.0, 2.0, 3.0),
            lookat: Point3::new(0.0, 0.0, 0.0),
            field_of_view: 20.0,
        }
    }

    fn build(&self, rng: &mut dyn rand::RngCore) -> Box<dyn Hittable> {
        let mut shapes = HittableList::new();
        let pertext = NoiseTexture::new(4.0, rng);
        shapes.add(Sphere::new(
            Point3::new(0.0, -1000.0, 0.0),
            1000.0,
            Lambertian::new(pertext.clone()),
        ));
        shapes.add(Sphere::new(Point3::new(0.0, 2.0, 0.0), 2.0, Lambertian::new(pertext)));

        Box::new(shapes)
    }
}

struct SimpleLight {}

impl World for SimpleLight {
    fn name(&self) -> &'static str {
        "simple_light"
    }
    fn background(&self) -> Box<dyn Background> {
        Box::new(BlackBackground::new())
    }

    fn camera(&self) -> WorldCamera {
        WorldCamera {
            lookfrom: Point3::new(20.0, 3.0, 6.0),
            lookat: Point3::new(0.0, 2.0, 0.0),
            field_of_view: 20.0,
        }
    }

    fn build(&self, rng: &mut dyn rand::RngCore) -> Box<dyn Hittable> {
        let mut shapes = HittableList::new();
        let pertext = NoiseTexture::new(4.0, rng);
        shapes.add(Sphere::new(
            Point3::new(0.0, -1000.0, 0.0),
            1000.0,
            Lambertian::new(pertext.clone()),
        ));
        shapes.add(Sphere::new(Point3::new(0.0, 2.0, 0.0), 2.0, Lambertian::new(pertext)));

        let difflight = DiffuseLight::new(SolidColor::new(4.0, 4.0, 4.0));
        shapes.add(XYRect::new(3.0, 5.0, 1.0, 3.0, -2.0, difflight.clone()));
        shapes.add(Sphere::new(Point3::new(0.0, 6.0, 0.0), 1.0, difflight));

        Box::new(shapes)
    }
}

struct CornellBox {}

impl World for CornellBox {
    fn name(&self) -> &'static str {
        "cornell_box"
    }
    fn background(&self) -> Box<dyn Background> {
        Box::new(BlackBackground::new())
    }

    fn camera(&self) -> WorldCamera {
        WorldCamera {
            lookfrom: Point3::new(278.0, 278.0, -800.0),
            lookat: Point3::new(278.0, 278.0, 0.0),
            field_of_view: 40.0,
        }
    }

    fn build(&self, _: &mut dyn rand::RngCore) -> Box<dyn Hittable> {
        let mut shapes = HittableList::new();
        let red = Lambertian::new(SolidColor::new(0.65, 0.05, 0.05));
        let white = Lambertian::new(SolidColor::new(0.73, 0.73, 0.73));
        let green = Lambertian::new(SolidColor::new(0.12, 0.45, 0.15));
        let light = DiffuseLight::new(SolidColor::new(15.0, 15.0, 15.0));

        shapes.add(YZRect::new(0.0, 555.0, 0.0, 555.0, 555.0, green));
        shapes.add(YZRect::new(0.0, 555.0, 0.0, 555.0, 0.0, red));

        shapes.add(XZRect::new(213.0, 343.0, 227.0, 332.0, 554.0, light));

        shapes.add(XZRect::new(0.0, 555.0, 0.0, 555.0, 0.0, white));
        shapes.add(XZRect::new(0.0, 555.0, 0.0, 555.0, 555.0, white));
        shapes.add(XYRect::new(0.0, 555.0, 0.0, 555.0, 555.0, white));

        let large_block = Block::new(Point3::ZERO, Point3::new(165.0, 330.0, 165.0), white);
        shapes.add(transforms::Translate::new(large_block, Vec3::new(265.0, 0.0, 295.0)));

        let small_block = Block::new(Point3::ZERO, Point3::new(165.0, 165.0, 165.0), white);
        shapes.add(transforms::Translate::new(small_block, Vec3::new(130.0, 0.0, 65.0)));

        Box::new(shapes)
    }
}

pub fn worlds() -> Vec<Box<dyn World>> {
    vec![
        Box::new(Simple {}),
        Box::new(Random {}),
        Box::new(RandomChk {}),
        Box::new(TwoSpheres {}),
        Box::new(SimpleLight {}),
        Box::new(CornellBox {}),
    ]
}
