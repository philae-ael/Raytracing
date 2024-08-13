use core::f32;

use bytemuck::Zeroable;

use crate::color::{Luma, Rgb};

/// Represent a serie of samples from a given discribution.
/// It is used to get an easy access to mean, variance and
#[derive(Default, Clone)]
pub struct VarianceSeries {
    count: usize,
    sum: f32,
    sqsum: f32,
}

impl VarianceSeries {
    pub fn new() -> Self {
        Self {
            count: 0,
            sum: 0.0,
            sqsum: 0.0,
        }
    }
    pub fn add_sample(&mut self, sample: f32) {
        self.count += 1;
        self.sum += sample;
        self.sqsum += sample * sample;
    }
    pub fn merge(lhs: Self, rhs: Self) -> Self {
        Self {
            count: lhs.count + rhs.count,
            sum: lhs.sum + rhs.sum,
            sqsum: lhs.sqsum + rhs.sqsum,
        }
    }

    pub fn sum(&self) -> f32 {
        self.sum
    }

    pub fn mean(&self) -> f32 {
        self.sum / self.count as f32
    }

    pub fn variance(&self) -> f32 {
        if self.count <= 2 {
            return f32::INFINITY;
        }

        // This estimator is unbiased thx to the n - 1
        (self.sqsum - self.sum * self.sum / self.count as f32) / (self.count as f32 - 1.0)
    }

    /// Returns the error $\varepsilon$ such that the real value is between $\left[\varepsilon-m, \varepsilon+m\right]$ with 95% confidence
    ///
    /// This assume the distibution follows a normal law. This is clearly false.
    pub fn error_with_95_confidence(&self) -> Option<f32> {
        let df = self.count - 1;
        if df < 15 {
            return None;
        }

        Some(STUDENT_5[df.min(30) - 15] * self.variance().sqrt())
    }

    pub fn value(&self) -> f32 {
        self.mean()
    }

    pub fn is_precise_enough(&self, abs_err: f32) -> Option<f32> {
        self.error_with_95_confidence().and_then(|err| {
            if err < abs_err {
                Some(self.mean())
            } else {
                None
            }
        })
    }
}

/// Some values of the student distribution
///
/// Start with degree of freedom = 15
///
/// Left and right values are the same as the student distribution is symetric,
///
/// Generated with
/// ```python
/// from scipy import stats
/// [stats.t(df=i).interval(0.95)[0] for i in range(15, 31)]
/// ```
const STUDENT_5: [f32; 16] = [
    2.131, 2.120, 2.110, 2.101, 2.093, 2.086, 2.080, 2.074, 2.069, 2.064, 2.060, 2.056, 2.052,
    2.048, 2.045, 2.042,
];

#[derive(Default, Clone)]
pub struct RgbSeries {
    r: VarianceSeries,
    g: VarianceSeries,
    b: VarianceSeries,
}

impl RgbSeries {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_sample(&mut self, rgb: Rgb) {
        self.r.add_sample(rgb.0[0]);
        self.g.add_sample(rgb.0[1]);
        self.b.add_sample(rgb.0[2]);
    }

    pub fn is_precise_enough(&self, abs_err: f32) -> Option<Rgb> {
        let r = self.r.is_precise_enough(abs_err)?;
        let g = self.g.is_precise_enough(abs_err)?;
        let b = self.b.is_precise_enough(abs_err)?;

        Some(Rgb::from_array([r, g, b]))
    }
    pub fn mean(&self) -> Rgb {
        let r = self.r.mean();
        let g = self.g.mean();
        let b = self.b.mean();

        Rgb::from_array([r, g, b])
    }
    pub fn variance(&self) -> Luma {
        let r = self.r.variance();
        let g = self.g.variance();
        let b = self.b.variance();

        // Hum... can we do better than that?
        Luma((r * r + g * g + b * b).sqrt())
    }
    pub fn merge(lhs: Self, rhs: Self) -> Self {
        Self {
            r: VarianceSeries::merge(lhs.r, rhs.r),
            g: VarianceSeries::merge(lhs.g, rhs.g),
            b: VarianceSeries::merge(lhs.b, rhs.b),
        }
    }
}

#[derive(Clone)]
pub struct FilteredRgb {
    rgb: Rgb,
    sum_of_weigth: f32,
}
impl Default for FilteredRgb {
    fn default() -> Self {
        Self::new()
    }
}

impl FilteredRgb {
    pub fn new() -> Self {
        Self {
            rgb: Rgb::zeroed(),
            sum_of_weigth: 0.0,
        }
    }

    pub fn add_sample(&mut self, color: Rgb, weight: f32) {
        self.sum_of_weigth += weight;
        self.rgb = self.rgb + weight * color;
    }

    pub fn value(&self) -> Rgb {
        if self.sum_of_weigth == 0.0 {
            return self.rgb;
        }
        self.rgb / self.sum_of_weigth
    }

    pub fn merge(self, rhs: Self) -> Self {
        Self {
            rgb: self.rgb + rhs.rgb,
            sum_of_weigth: self.sum_of_weigth + rhs.sum_of_weigth,
        }
    }
}
