pub mod bvh;
pub mod embree;
pub mod shapelist;

use crate::shape::Shape;

pub trait Aggregate: Shape {}
