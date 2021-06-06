use crate::vec::{Color, Point3, Vec3};
use rand::Rng;

pub trait Texture: Sync {
    fn value(&self, u: f64, v: f64, p: Point3) -> Color;
}

#[derive(Copy, Clone)]
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

#[derive(Copy, Clone)]
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
        let sines = (5.0 * p.x()).sin() * (5.0 * p.y()).sin() * (5.0 * p.z()).sin();
        if sines < 0.0 {
            self.odd.value(u, v, p)
        } else {
            self.even.value(u, v, p)
        }
    }
}

const POINT_COUNT: usize = 1024;

struct Perlin {
    ranvec: [Vec3; POINT_COUNT],
    perm_x: [usize; POINT_COUNT],
    perm_y: [usize; POINT_COUNT],
    perm_z: [usize; POINT_COUNT],
    turbulence_depth: i32,
}

impl Perlin {
    pub fn new(rng: &mut dyn rand::RngCore) -> Perlin {
        let mut ranvec: [Vec3; POINT_COUNT] = [Vec3::ZERO; POINT_COUNT];
        for i in 0..POINT_COUNT {
            ranvec[i] = Vec3::random(-1.0, 1.0, rng).unit();
        }
        Perlin {
            ranvec,
            perm_x: Perlin::permute(rng),
            perm_y: Perlin::permute(rng),
            perm_z: Perlin::permute(rng),
            turbulence_depth: 7,
        }
    }

    fn turbulence(&self, p: &Point3) -> f64 {
        let mut accum = 0.0f64;
        let mut temp_p = *p;
        let mut weight = 1.0;

        for _ in 0..self.turbulence_depth {
            accum += weight * self.noise(&temp_p);
            weight *= 0.5;
            temp_p = 2.0 * temp_p;
        }

        accum.abs()
    }

    fn noise(&self, p: &Point3) -> f64 {
        let u = p.x() - p.x().floor();
        let v = p.y() - p.y().floor();
        let w = p.z() - p.z().floor();

        let i = p.x().floor() as isize;
        let j = p.y().floor() as isize;
        let k = p.z().floor() as isize;

        let mut c = [[[Vec3::ZERO; 2]; 2]; 2];

        for di in 0..2 {
            for dj in 0..2 {
                for dk in 0..2 {
                    let ii = (i + di as isize).rem_euclid(POINT_COUNT as isize) as usize;
                    let jj = (j + dj as isize).rem_euclid(POINT_COUNT as isize) as usize;
                    let kk = (k + dk as isize).rem_euclid(POINT_COUNT as isize) as usize;
                    c[di][dj][dk] = self.ranvec[self.perm_x[ii] ^ self.perm_y[jj] ^ self.perm_z[kk]];
                }
            }
        }

        Perlin::trilinear_interpolation(&c, u, v, w)
    }

    fn trilinear_interpolation(c: &[[[Vec3; 2]; 2]; 2], u: f64, v: f64, w: f64) -> f64 {
        let uu = u * u * (3.0 - 2.0 * u);
        let vv = v * v * (3.0 - 2.0 * v);
        let ww = w * w * (3.0 - 2.0 * w);

        let mut accum = 0.0f64;

        for i in 0..2 {
            for j in 0..2 {
                for k in 0..2 {
                    let weight = Vec3::new(u - i as f64, v - j as f64, w - k as f64);
                    accum += ((i as f64) * uu + ((1 - i) as f64) * (1.0 - uu))
                        * ((j as f64) * vv + ((1 - j) as f64) * (1.0 - vv))
                        * ((k as f64) * ww + ((1 - k) as f64) * (1.0 - ww))
                        * weight.dot(c[i][j][k]);
                }
            }
        }
        accum
    }

    fn permute(rng: &mut dyn rand::RngCore) -> [usize; POINT_COUNT] {
        let mut result: [usize; POINT_COUNT] = [0; POINT_COUNT];
        for i in 0..POINT_COUNT {
            result[i] = i
        }
        for i in (1..POINT_COUNT).rev() {
            let j = rng.gen_range(0..i);
            let tmp = result[i];
            result[i] = result[j];
            result[j] = tmp;
        }
        result
    }
}

#[derive(Clone)]
pub struct NoiseTexture {
    noise: std::sync::Arc<Perlin>,
    scale: f64,
}

impl NoiseTexture {
    pub fn new(scale: f64, rng: &mut dyn rand::RngCore) -> NoiseTexture {
        NoiseTexture { noise: std::sync::Arc::new(Perlin::new(rng)), scale }
    }
}

impl Texture for NoiseTexture {
    fn value(&self, _u: f64, _v: f64, p: Point3) -> Color {
        Color::ONE * 0.5 * (1.0 + (self.scale * p.z() + 10.0 * self.noise.turbulence(&(self.scale * p))).sin())
    }
}
