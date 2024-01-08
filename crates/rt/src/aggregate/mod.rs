pub mod bvh;
pub mod shapelist;

use crate::shape::Shape;

pub trait Aggregate: Shape {}
