//! Contains the objects that are meant to be renderered:
//! - Spheres
//! - Triangles
//! - Meshes
//! - ...
//!
//! There are two sub-kind of shapes: implicit ones and the others.
//!
//! Implicit shapes are shapes for which the surface is find by solving
//! F(x, y, z) = 0.
//! It may be easier to quickly add a shape in an implicit manner (compare [Sphere] and [implicit::Sphere] source code) but it may give inexact or plain wrong results.
//! Furthermore, intersection implementation is probably slower.
//!
//! See [implicit] for details.
//!
//!
//! Explicit shapes are shapes for which finding where is surface is doesn't require an optimization process.
//!
//! All explicit shapes are reimported bellow

pub mod plane;
pub mod sphere;
pub mod triangle;

pub use plane::Plane;
pub use sphere::Sphere;
pub use triangle::{Triangle, TriangleBuilder};

use crate::{math::bounds::Bounds, ray::Ray};

/// An abstracted shape to be rendered by raytracing.
///
/// To render a shape we only need to know whether a ray intersect it and if so,
///  some information about the shape at the intersection point
pub trait Shape: Sync + Send {
    /// Check whether `ray` intersect the shape defined by `self` if so, gives all the information needed
    fn intersection_full(&self, ray: Ray) -> FullIntersectionResult;

    /// Check whether `ray` intersect the shape defined by `self` if so, return the minimal amount of information
    /// It is often used to cast shadow rays
    fn intersect_bare(&self, ray: Ray) -> MinIntersectionResult;

    /// Returns the bounding box of the shape, if any.
    fn bounding_box(&self) -> Bounds;
}

pub mod local_info {
    use crate::{
        material::{texture::Uv, MaterialId},
        math::point::Point,
    };
    use glam::Vec3;

    /// Contains all the local information that could be needed
    ///
    /// Note that all the information is computed. If not all information is needed, prefer other kinds of local_info.
    #[derive(Debug)]
    pub struct Full {
        pub pos: Point,
        pub normal: Vec3,
        pub material: MaterialId,
        pub uv: Uv,
    }

    /// Contains only the pure geometrical information needed to locate the point.
    #[derive(Debug)]
    pub struct Minimum {
        pub pos: Point,
    }
}

/// Holds local informations and the time of a colision between a ray and a shape.
#[derive(Debug)]
pub struct RayIntersection<LocalInfo> {
    pub t: f32,
    pub local_info: LocalInfo,
}

/// A `Result`-like type that takes care of intersections data.
#[derive(Debug)]
pub enum IntersectionResult<LocalInfo> {
    Intersection(RayIntersection<LocalInfo>),
    NoIntersection,
}

impl<T> IntersectionResult<T> {
    pub fn or_then<F: Fn() -> Self>(self, f: F) -> Self {
        match self {
            Self::Intersection(_) => self,
            _ => f(),
        }
    }

    pub fn is_intersection(&self) -> bool {
        matches!(self, Self::Intersection(_))
    }

    pub fn unwrap(self) -> RayIntersection<T> {
        match self {
            Self::Intersection(t) => t,
            _ => panic!("Unwraped an no_intersection"),
        }
    }

    pub fn min(self, other: Self) -> Self {
        let Self::Intersection(RayIntersection { t: t1, .. }) = self else {
            return other;
        };
        let Self::Intersection(RayIntersection { t: t2, .. }) = other else {
            return self;
        };

        if t1 < t2 {
            self
        } else {
            other
        }
    }
}

pub type MinIntersectionResult = IntersectionResult<local_info::Minimum>;
pub type FullIntersectionResult = IntersectionResult<local_info::Full>;
