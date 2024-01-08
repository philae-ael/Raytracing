pub trait FloatAsExt {
    /// Returns `Some(f)` is f is near zero (near is given by eps) else returns None
    /// The result value is guaranted to be far enough from 0
    ///
    /// Returns None for NaN and Some(f) for +/- infty
    fn as_non_zero(self, eps: Self) -> Option<f32>;

    /// Returns `Some(f)` is f is finite else returns None
    /// The result value is guaranted to be finite
    ///
    /// Returns None for NaN and +/- infty
    fn as_finite(self) -> Option<f32>;
}

impl FloatAsExt for f32 {
    fn as_non_zero(self, eps: Self) -> Option<f32> {
        (self.abs() > eps).then_some(self)
    }

    fn as_finite(self) -> Option<f32> {
        self.is_finite().then_some(self)
    }
}

#[cfg(test)]
mod tests {
    use super::FloatAsExt;

    #[test]
    fn as_non_zero_test() {
        assert_eq!(0.0.as_non_zero(0.1), None);
        assert_eq!(1.0.as_non_zero(0.1), Some(1.0));
        assert_eq!((-0.01).as_non_zero(0.1), None);
        assert_eq!((-1.0).as_non_zero(0.1), Some(-1.0));
        assert_eq!(f32::NAN.as_non_zero(0.1), None);
        assert_eq!(f32::INFINITY.as_non_zero(0.1), Some(f32::INFINITY));
    }
    #[test]
    fn as_finite_test() {
        assert_eq!(0.0.as_finite(), Some(0.0));
        assert_eq!(1.0.as_finite(), Some(1.0));
        assert_eq!((-0.01).as_finite(), Some(-0.01));
        assert_eq!((-1.0).as_finite(), Some(-1.0));
        assert_eq!(f32::NAN.as_finite(), None);
        assert_eq!(f32::INFINITY.as_finite(), None);
    }
}
