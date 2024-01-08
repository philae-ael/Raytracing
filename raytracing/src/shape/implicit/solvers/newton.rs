use glam::Vec3;

use crate::{math::float::FloatAsExt, ray::Ray, shape::implicit::ImplicitSolver};

use super::ImplicitSolution;

/// Implement a solver by using a Newton algorithm
/// Derivative are computed in a dumb way
/// algorithm will give the wrong result in a lot of cases (multiple solutions, ...) or no solution, even if one solution exists
pub struct NewtonSolver {
    /// Both the max error accepted and the epsion used of derivative calculations (ugly, i know)
    pub eps: f32,
    pub max_iter: usize,
}

impl ImplicitSolver for NewtonSolver {
    fn solve<F: Fn(Vec3) -> f32>(&self, f: F, ray: Ray) -> Option<ImplicitSolution> {
        let mut t = ray.bounds.0;
        let mut steps = 0;
        let f_along_ray = |t| f(ray.at(t));
        let f_along_ray_unchecked = |t| f(ray.at_unchecked(t));

        let (ray_start, ray_end) = ray.bounds;
        loop {
            let ft = f_along_ray(t);
            let Some(ft) = ft.as_non_zero(self.eps) else {break};

            if steps >= self.max_iter {
                //log::info!("No Hit {ft}");
                return None;
            }

            steps += 1;

            // if t == ray.bounds.1, t + self.eps is outside the ray normal range of operation,
            // thus the use of f_along_ray_unchecked
            let dft = (f_along_ray_unchecked(t + self.eps) - f_along_ray(t)) / self.eps;

            // pass this statement, dft guarantedt to be non zero
            let Some(dft) = dft.as_non_zero(self.eps) else {return  None};

            let new_t = t - ft / dft;
            t = new_t.min(ray_end).max(ray_start);
        }

        let x = ray.at(t);
        let normal = Vec3::new(
            (f(x + self.eps * Vec3::X) - f(x)) / self.eps,
            (f(x + self.eps * Vec3::Y) - f(x)) / self.eps,
            (f(x + self.eps * Vec3::Z) - f(x)) / self.eps,
        );
        Some(ImplicitSolution {
            t,
            hit_point: x,
            normal: normal.normalize_or_zero(),
        })
    }
}
