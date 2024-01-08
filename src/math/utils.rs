use std::ops::{Add, Mul};

pub fn lerp<T>(t: f64, x: T, y: T) -> T
where
    T: Add<T, Output=T> + std::cmp::PartialEq,
    f64: Mul<T, Output=T>, 
{
    t * x + (1.0 - t) * y
}

pub fn clamp(x: f64) -> f64 {
    if x > 1.0 {
        1.0
    } else if x < 0.0 {
        0.0
    } else {
        x
    }
}
