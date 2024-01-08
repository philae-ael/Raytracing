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

pub mod implicit;
pub mod shape;
pub mod sphere;

pub use shape::{
    local_info, FullIntersectionResult, IntersectionResult, MinIntersectionResult, Shape,
};

pub use sphere::Sphere;
