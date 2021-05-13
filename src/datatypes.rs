use std::io::Write;
use std::ops;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Vec3 {
    pub e: [f64; 3],
}

impl Vec3 {
    pub fn new(e0: f64, e1: f64, e2: f64) -> Vec3 {
        Vec3 {
            e: { [e0, e1, e2] },
        }
    }

    pub const ZERO: Vec3 = Vec3 {
        e: { [0.0, 0.0, 0.0] },
    };

    pub const ONE: Vec3 = Vec3 {
        e: { [1.0, 1.0, 1.0] },
    };

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

impl ops::Add for Vec3 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Vec3 {
            e: {
                [
                    self.e[0] + rhs.e[0],
                    self.e[1] + rhs.e[1],
                    self.e[2] + rhs.e[2],
                ]
            },
        }
    }
}

impl ops::Add for &Vec3 {
    type Output = Vec3;
    fn add(self, rhs: Self) -> Vec3 {
        Vec3 {
            e: {
                [
                    self.e[0] + rhs.e[0],
                    self.e[1] + rhs.e[1],
                    self.e[2] + rhs.e[2],
                ]
            },
        }
    }
}

impl ops::Sub for Vec3 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Vec3 {
            e: {
                [
                    self.e[0] - rhs.e[0],
                    self.e[1] - rhs.e[1],
                    self.e[2] - rhs.e[2],
                ]
            },
        }
    }
}

impl ops::Sub for &Vec3 {
    type Output = Vec3;
    fn sub(self, rhs: Self) -> Vec3 {
        Vec3 {
            e: {
                [
                    self.e[0] - rhs.e[0],
                    self.e[1] - rhs.e[1],
                    self.e[2] - rhs.e[2],
                ]
            },
        }
    }
}

impl ops::Mul<f64> for &Vec3 {
    type Output = Vec3;
    fn mul(self, rhs: f64) -> Vec3 {
        Vec3 {
            e: { [self.e[0] * rhs, self.e[1] * rhs, self.e[2] * rhs] },
        }
    }
}

impl ops::Mul<f64> for Vec3 {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self {
        Vec3 {
            e: { [self.e[0] * rhs, self.e[1] * rhs, self.e[2] * rhs] },
        }
    }
}

impl ops::Div<f64> for &Vec3 {
    type Output = Vec3;
    fn div(self, rhs: f64) -> Vec3 {
        Vec3 {
            e: { [self.e[0] / rhs, self.e[1] / rhs, self.e[2] / rhs] },
        }
    }
}

impl ops::Div<f64> for Vec3 {
    type Output = Self;
    fn div(self, rhs: f64) -> Self {
        Vec3 {
            e: { [self.e[0] / rhs, self.e[1] / rhs, self.e[2] / rhs] },
        }
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

pub fn write_color(color: &Color, w: &mut dyn Write) -> std::io::Result<()> {
    let ir = (255.999f64 * color.r()) as i32;
    let ig = (255.999f64 * color.g()) as i32;
    let ib = (255.999f64 * color.b()) as i32;
    writeln!(w, "{} {} {}", ir, ig, ib)
}

pub fn unit_vector(v: &Vec3) -> Vec3 {
    v / v.length()
}

pub fn dot(u: Vec3, v: Vec3) -> f64 {
    return u.e[0] * v.e[0] + u.e[1] * v.e[1] + u.e[2] * v.e[2];
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Ray {
    pub orig: Point3,
    pub dir: Vec3,
}

impl Ray {
    pub fn at(&self, t: f64) -> Point3 {
        &self.orig - &(t * &self.dir)
    }
}
