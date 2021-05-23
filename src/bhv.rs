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
    pub fn new(a: Point3, b: Point3) -> AABB {
        let min = [a.e[0].min(b.e[0]), a.e[1].min(b.e[1]), a.e[2].min(b.e[2])];
        let max = [a.e[0].max(b.e[0]), a.e[1].max(b.e[1]), a.e[2].max(b.e[2])];
        AABB { minimum: Point3 { e: min }, maximum: Point3 { e: max } }
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
            min[a] = self.minimum.e[a].min(other.minimum.e[a]);
            max[a] = self.maximum.e[a].max(other.maximum.e[a]);
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
    root: Node<'a>,
}

impl<'a> BHV<'a> {
    pub fn new<'b>(scene: &'b mut SceneBuilder<'a>) -> BHV<'a> {
        if scene.contents.is_empty() {
            panic!();
        }
        let root = Node::new(scene.contents.as_mut_slice());
        scene.contents.clear();
        BHV { root }
    }
}

enum Node<'a> {
    Leaf(Box<dyn Bounded + 'a>),
    Inner { bounds: AABB, left: Box<Node<'a>>, right: Box<Node<'a>> },
}

impl<'a> Node<'a> {
    fn bounding_box(&self) -> AABB {
        match self {
            Node::Leaf(v) => v.bounding_box(),
            Node::Inner { bounds, left: _, right: _ } => *bounds,
        }
    }

    fn new<'b>(shapes: &'b mut [Option<Box<dyn Bounded + 'a>>]) -> Node<'a> {
        match shapes {
            [v] => Node::Leaf(v.take().unwrap()),
            _ => {
                let axis = rand::thread_rng().gen_range(0..3);
                let get_dim =
                    |a: &Option<Box<dyn Bounded + 'a>>| a.as_ref().unwrap().bounding_box().min().e[axis];
                let comparator =
                    |a: &Option<Box<dyn Bounded>>, b: &Option<Box<dyn Bounded>>| match get_dim(a)
                        .partial_cmp(&get_dim(b))
                    {
                        Some(ordering) => ordering,
                        None => Ordering::Equal,
                    };

                shapes.sort_by(comparator);
                let (left_shapes, right_shapes) = shapes.split_at_mut(shapes.len() / 2);

                let left = Box::new(Node::new(left_shapes));
                let right = Box::new(Node::new(right_shapes));
                let bounds = left.bounding_box().surround(&right.bounding_box());
                Node::Inner { left, right, bounds }

            }
        }
    }

    fn hit<'b>(&'b self, r: &Ray, tmin: f64, tmax: f64) -> Option<Hit<'b>> {
        match self {
            Node::Leaf(v) => v.hit(r, tmin, tmax),
            Node::Inner { left, right, bounds } => {
                if !bounds.hit(r, tmin, tmax) {
                    return None;
                }
                let hit_left = left.hit(r, tmin, tmax);
                match right.hit(r, tmin, hit_left.as_ref().map_or(tmax, |h| h.t)) {
                    None => hit_left,
                    hit_right => hit_right,
                }
            }
        }
    }
}

impl<'b> Hittable for BHV<'b> {
    fn hit<'a>(&'a self, r: &Ray, tmin: f64, tmax: f64) -> Option<Hit<'a>> {
        self.root.hit(r, tmin, tmax)
    }
}

impl<'b> Bounded for BHV<'b> {
    fn bounding_box(&self) -> AABB {
        self.root.bounding_box()
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
