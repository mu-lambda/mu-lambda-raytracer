use crate::hittable::{Hit, Hittable};
use crate::vec::{Point3, Ray, Vec3};
use rand::Rng;
use std::cmp::Ordering;
use std::fmt;

// Axis-Aligned Bounding Box
#[derive(Copy, Clone, Debug)]
pub struct AABB {
    pub minimum: Point3,
    pub maximum: Point3,
}

impl AABB {
    pub fn new(minimum: Point3, maximum: Point3) -> AABB {
        AABB { minimum, maximum }
    }

    pub fn min(&self) -> Point3 {
        self.minimum
    }
    pub fn max(&self) -> Point3 {
        self.maximum
    }

    fn hit(&self, r: &Ray, tmin: f64, tmax: f64) -> bool {
        let mut tmin = tmin;
        let mut tmax = tmax;
        for a in 0..3 {
            let inv_d = 1.0 / r.dir.e[a];
            let mut t0 = (self.minimum.e[a] - r.orig.e[a]) * inv_d;
            let mut t1 = (self.maximum.e[a] - r.orig.e[a]) * inv_d;
            if t1 < t0 {
                std::mem::swap(&mut t0, &mut t1);
            }
            tmin = if t0 > tmin { t0 } else { tmin };
            tmax = if t1 < tmax { t1 } else { tmax };
            if tmax <= tmin {
                return false;
            }
        }
        true
    }

    pub fn surround(&self, other: &AABB) -> AABB {
        let mut min: [f64; 3] = [0.0, 0.0, 0.0];
        let mut max: [f64; 3] = [0.0, 0.0, 0.0];
        for a in 0..3 {
            min[a] = self.minimum.e[a]
                .min(other.minimum.e[a])
                .min(self.maximum.e[a])
                .min(other.maximum.e[a]);
            max[a] = self.minimum.e[a]
                .max(other.minimum.e[a])
                .max(self.maximum.e[a])
                .max(other.maximum.e[a]);
        }
        AABB::new(Point3 { e: min }, Point3 { e: max })
    }
}

impl fmt::Display for AABB {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}:{}", self.minimum, self.maximum)
    }
}

pub trait Bounded: Hittable {
    fn bounding_box(&self) -> AABB;
}

pub struct SceneBuilder<'a> {
    contents: Vec<Option<Box<dyn Bounded + 'a>>>,
}

impl<'a> SceneBuilder<'a> {
    pub fn new() -> SceneBuilder<'a> {
        SceneBuilder { contents: Vec::new() }
    }
    pub fn add<T: Bounded + 'a>(&mut self, v: T) -> &mut Self {
        self.contents.push(Some(Box::new(v)));
        self
    }

    pub fn push<T: Bounded + 'a>(&mut self, v: Box<T>) -> &mut Self {
        self.contents.push(Some(v));
        self
    }
}

// Bounded Volume Hierarchy
pub struct BHV<'a> {
    left: Option<Box<dyn Bounded + 'a>>,
    right: Option<Box<dyn Bounded + 'a>>,
    bounds: AABB,
}

fn surround<'a>(a: &Option<Box<dyn Bounded + 'a>>, b: &Option<Box<dyn Bounded + 'a>>) -> AABB {
    match (a.as_ref(), b.as_ref()) {
        (Some(a), None) | (None, Some(a)) => a.bounding_box(),
        (Some(a), Some(b)) => a.bounding_box().surround(&b.bounding_box()),
        (None, None) => panic!(),
    }
}

impl<'a> BHV<'a> {
    pub fn new<'b>(scene: &'b mut SceneBuilder<'a>) -> BHV<'a> {
        let result = BHV::new_inner(scene.contents.as_mut_slice());
        scene.contents.clear();
        result
    }
    pub fn new_inner<'b>(objects: &'b mut [Option<Box<dyn Bounded + 'a>>]) -> BHV<'a> {
        let axis = rand::thread_rng().gen_range(0..3);
        let get_dim =
            |a: &Option<Box<dyn Bounded + 'a>>| a.as_ref().unwrap().bounding_box().min().e[axis];
        let comparator =
            |a: &Option<Box<dyn Bounded>>, b: &Option<Box<dyn Bounded>>| match get_dim(a)
                .partial_cmp(&get_dim(b))
            {
                Some(ordering) => ordering,
                None => panic!("a = {} b = {}", get_dim(a), get_dim(b)),
            };

        let left;
        let right;
        match objects.len() {
            1 => {
                left = Some(objects[0].take().unwrap());
                right = None;
            }
            // Optimize representation: two objects become leaf nodes.
            2 => match comparator(&objects[0], &objects[1]) {
                Ordering::Less => {
                    left = Some(objects[0].take().unwrap());
                    right = Some(objects[1].take().unwrap());
                }
                Ordering::Greater => {
                    left = Some(objects[1].take().unwrap());
                    right = Some(objects[0].take().unwrap());
                }
                Ordering::Equal => {
                    left = Some(objects[1].take().unwrap());
                    right = Some(objects[0].take().unwrap());
                }
            },
            _ => {
                objects.sort_by(comparator);
                let (left_objects, right_objects) = objects.split_at_mut(objects.len() / 2);

                left = Some(Box::new(BHV::new_inner(left_objects)));
                right = Some(Box::new(BHV::new_inner(right_objects)));
            }
        }
        let bounds = surround(&left, &right);
        BHV { left, right, bounds }
    }
}

impl<'b> Hittable for BHV<'b> {
    fn hit<'a>(&'a self, r: &Ray, tmin: f64, tmax: f64) -> Option<Hit<'a>> {
        if !self.bounds.hit(r, tmin, tmax) {
            return None;
        }

        let hit_left = self.left.as_ref().map_or(None, |v| v.hit(r, tmin, tmax));
        let tmax_for_right = match hit_left.as_ref() {
            Some(h) => h.t,
            None => tmax,
        };
        let hit_right = self.right.as_ref().map_or(None, |v| v.hit(r, tmin, tmax_for_right));
        match hit_right {
            Some(_) => hit_right,
            None => hit_left,
        }
    }
}

impl<'b> Bounded for BHV<'b> {
    fn bounding_box(&self) -> AABB {
        self.bounds
    }
}

#[cfg(test)]
mod aabb_tests {
    use super::*;

    #[test]
    fn test_minmax() {
        let aabb = AABB::new(Point3::new(1.0, 1.0, 1.0), Point3::new(2.0, 2.0, 2.0));
        let aabb_rev = AABB::new(aabb.max(), aabb.min());
        let r = Ray::new(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 1.0, 1.0));
        assert_eq!(true, aabb.hit(&r, 0.0, f64::INFINITY));
        assert_eq!(true, aabb_rev.hit(&r, 0.0, f64::INFINITY));
    }

    #[test]
    fn test_edge_parallel() {
        let aabb = AABB::new(Point3::new(1.0, 1.0, 1.0), Point3::new(2.0, 2.0, 2.0));
        let aabb_rev = AABB::new(aabb.max(), aabb.min());
        // Ray in Y-direction from projection of a minimum to XZ plane, inside the cube.
        let r = Ray::new(Point3::new(1.0001, 0.0, 1.0001), Vec3::new(0.0, 1.0, 0.0));
        assert_eq!(true, aabb.hit(&r, 0.0, f64::INFINITY));
        assert_eq!(true, aabb_rev.hit(&r, 0.0, f64::INFINITY));
    }

    #[test]
    fn test_edge_parallel_outside() {
        let aabb = AABB::new(Point3::new(1.0, 1.0, 1.0), Point3::new(2.0, 2.0, 2.0));
        let aabb_rev = AABB::new(aabb.max(), aabb.min());
        // Ray in Y-direction from projection of a minimum to XZ plane, outside the cube.
        let r = Ray::new(Point3::new(0.99999, 0.0, 0.9999), Vec3::new(0.0, 1.0, 0.0));
        assert_eq!(false, aabb.hit(&r, 0.0, f64::INFINITY));
        assert_eq!(false, aabb_rev.hit(&r, 0.0, f64::INFINITY));
    }

    #[test]
    fn test_face_parallel() {
        let aabb = AABB::new(Point3::new(1.0, 1.0, 1.0), Point3::new(2.0, 2.0, 2.0));
        let aabb_rev = AABB::new(aabb.max(), aabb.min());
        // Ray in Y-direction from projection of a center of an edge to XZ plane, inside the cube.
        let r = Ray::new(Point3::new(1.5, 0.0, 1.0001), Vec3::new(0.0, 3.0, 0.0));
        assert_eq!(true, aabb.hit(&r, 0.0, f64::INFINITY));
        assert_eq!(true, aabb_rev.hit(&r, 0.0, f64::INFINITY));
    }

    #[test]
    fn test_face_parallel_outside() {
        let aabb = AABB::new(Point3::new(1.0, 1.0, 1.0), Point3::new(2.0, 2.0, 2.0));
        let aabb_rev = AABB::new(aabb.max(), aabb.min());
        // Ray in Y-direction from projection of a center of an edge to XZ plane, outside the cube.
        let r = Ray::new(Point3::new(1.5, 0.0, 0.9999), Vec3::new(0.0, 3.0, 0.0));
        assert_eq!(false, aabb.hit(&r, 0.0, f64::INFINITY));
        assert_eq!(false, aabb_rev.hit(&r, 0.0, f64::INFINITY));
    }
}
