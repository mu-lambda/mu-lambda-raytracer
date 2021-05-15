use rand::Rng;
use std::io::Write;
use std::ops;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Vec3 {
    pub e: [f64; 3],
}

impl Vec3 {
    pub fn new(e0: f64, e1: f64, e2: f64) -> Vec3 {
        Vec3 { e: { [e0, e1, e2] } }
    }

    pub fn random(min: f64, max: f64) -> Vec3 {
        let mut rng = rand::thread_rng();
        Vec3::new(rng.gen_range(min..max), rng.gen_range(min..max), rng.gen_range(min..max))
    }

    pub fn random_in_unit_sphere() -> Vec3 {
        loop {
            let p = Vec3::random(-1.0, 1.0);
            if p.length_squared() < 1.0 {
                return p;
            }
        }
    }

    pub fn random_unit_vector() -> Vec3 {
        unit_vector(&Vec3::random_in_unit_sphere())
    }

    pub fn random_in_hemisphere(normal: &Vec3) -> Vec3 {
        let in_unit_sphere = Vec3::random_in_unit_sphere();
        if dot(in_unit_sphere, *normal) > 0.0 {
            return in_unit_sphere;
        } else {
            return -in_unit_sphere;
        }
    }

    pub fn near_zero(&self) -> bool {
        const s: f64 = 1e-8;
        return self.e[0].abs() < s && self.e[1].abs() < s && self.e[2].abs() < s;
    }

    pub const ZERO: Vec3 = Vec3 { e: { [0.0, 0.0, 0.0] } };

    pub const ONE: Vec3 = Vec3 { e: { [1.0, 1.0, 1.0] } };

    pub fn length(&self) -> f64 {
        self.length_squared().sqrt()
    }

    pub fn length_squared(&self) -> f64 {
        self.e[0] * self.e[0] + self.e[1] * self.e[1] + self.e[2] * self.e[2]
    }

    pub fn x(&self) -> f64 {
        self.e[0]
    }
    pub fn y(&self) -> f64 {
        self.e[1]
    }
    pub fn z(&self) -> f64 {
        self.e[2]
    }

    pub fn r(&self) -> f64 {
        self.e[0]
    }
    pub fn g(&self) -> f64 {
        self.e[1]
    }
    pub fn b(&self) -> f64 {
        self.e[2]
    }
}

impl ops::Neg for &Vec3 {
    type Output = Vec3;
    fn neg(self) -> Vec3 {
        Vec3 { e: { [-self.e[0], -self.e[1], -self.e[2]] } }
    }
}

impl ops::Neg for Vec3 {
    type Output = Vec3;
    fn neg(self) -> Vec3 {
        -&self
    }
}

impl ops::Add for Vec3 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        &self + &rhs
    }
}

impl ops::Add for &Vec3 {
    type Output = Vec3;
    fn add(self, rhs: Self) -> Vec3 {
        Vec3 { e: { [self.e[0] + rhs.e[0], self.e[1] + rhs.e[1], self.e[2] + rhs.e[2]] } }
    }
}

impl ops::Sub for Vec3 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        &self - &rhs
    }
}

impl ops::Sub for &Vec3 {
    type Output = Vec3;
    fn sub(self, rhs: Self) -> Vec3 {
        Vec3 { e: { [self.e[0] - rhs.e[0], self.e[1] - rhs.e[1], self.e[2] - rhs.e[2]] } }
    }
}

impl ops::Mul<f64> for &Vec3 {
    type Output = Vec3;
    fn mul(self, rhs: f64) -> Vec3 {
        Vec3 { e: { [self.e[0] * rhs, self.e[1] * rhs, self.e[2] * rhs] } }
    }
}

impl ops::Mul<&Vec3> for &Vec3 {
    type Output = Vec3;
    fn mul(self, rhs: &Vec3) -> Vec3 {
        Vec3 { e: { [self.e[0] * rhs.e[0], self.e[1] * rhs.e[1], self.e[2] * rhs.e[2]] } }
    }
}

impl ops::Mul<f64> for Vec3 {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self {
        &self * rhs
    }
}

impl ops::Mul<Vec3> for Vec3 {
    type Output = Self;
    fn mul(self, rhs: Vec3) -> Self {
        &self * &rhs
    }
}

impl ops::Div<f64> for &Vec3 {
    type Output = Vec3;
    fn div(self, rhs: f64) -> Vec3 {
        Vec3 { e: { [self.e[0] / rhs, self.e[1] / rhs, self.e[2] / rhs] } }
    }
}

impl ops::Div<f64> for Vec3 {
    type Output = Self;
    fn div(self, rhs: f64) -> Self {
        &self / rhs
    }
}

impl ops::Mul<&Vec3> for f64 {
    type Output = Vec3;
    fn mul(self, rhs: &Vec3) -> Vec3 {
        rhs * self
    }
}

impl ops::Mul<Vec3> for f64 {
    type Output = Vec3;
    fn mul(self, rhs: Vec3) -> Vec3 {
        &rhs * self
    }
}

pub type Point3 = Vec3;
pub type Color = Vec3;

fn clamp(x: f64, min: f64, max: f64) -> f64 {
    if x < min {
        return min;
    }
    if x > max {
        return max;
    }
    return x;
}

pub type RGB = (i32, i32, i32);

pub fn to_rgb(color: &Color, samples_per_pixel: i32) -> RGB {
    let scale = 1.0f64 / samples_per_pixel as f64;
    let r = (color.r() * scale).sqrt();
    let g = (color.g() * scale).sqrt();
    let b = (color.b() * scale).sqrt();
    let ir = (255.999f64 * clamp(r, 0.0, 0.99999999)) as i32;
    let ig = (255.999f64 * clamp(g, 0.0, 0.99999999)) as i32;
    let ib = (255.999f64 * clamp(b, 0.0, 0.99999999)) as i32;
    (ir, ig, ib)
}

pub fn write_color(
    color: &Color,
    samples_per_pixel: i32,
    w: &mut dyn Write,
) -> std::io::Result<()> {
    let (ir, ig, ib) = to_rgb(color, samples_per_pixel);
    writeln!(w, "{} {} {}", ir, ig, ib)
}

pub fn unit_vector(v: &Vec3) -> Vec3 {
    v / v.length()
}

pub fn dot(u: Vec3, v: Vec3) -> f64 {
    return u.e[0] * v.e[0] + u.e[1] * v.e[1] + u.e[2] * v.e[2];
}

pub fn cross(u: Vec3, v: Vec3) -> Vec3 {
    Vec3::new(
        u.e[1] * v.e[2] - u.e[2] * v.e[1],
        u.e[2] * v.e[0] - u.e[0] * v.e[2],
        u.e[0] * v.e[1] - u.e[1] * v.e[0],
    )
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Ray {
    pub orig: Point3,
    pub dir: Vec3,
}

impl Ray {
    pub fn new(orig: Point3, dir: Vec3) -> Ray {
        Ray { orig, dir }
    }
    pub fn at(&self, t: f64) -> Point3 {
        &self.orig + &(t * &self.dir)
    }
}
