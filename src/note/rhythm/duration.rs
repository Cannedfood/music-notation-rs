use crate::rendering::math2d::Lerp;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Duration(pub(crate) i64);

impl Duration {
    pub const ZERO: Duration = Duration(0);

    pub const BEAT: i64 = 2i64.pow(8) * 3i64.pow(2) * 5i64.pow(2) * 7i64.pow(2);

    pub const WHOLE: Duration = Duration(Duration::BEAT * 4);
    pub const HALF: Duration = Duration(Duration::BEAT * 2);
    pub const QUARTER: Duration = Duration(Duration::BEAT);
    pub const QUARTER_TRIPLET: Duration = Duration(Duration::BEAT / 3);
    pub const EIGHTH: Duration = Duration(Duration::BEAT / 2);
    pub const EIGHTH_TRIPLET: Duration = Duration(Duration::BEAT / 3);
    pub const SIXTEENTH: Duration = Duration(Duration::BEAT / 4);
    pub const SIXTEENTH_TRIPLET: Duration = Duration(Duration::BEAT / 6);
    pub const THIRTY_SECOND: Duration = Duration(Duration::BEAT / 8);

    pub fn beats(self) -> f64 { self.0 as f64 / Duration::BEAT as f64 }
    pub fn from_beats_f64(beats: f64) -> Self { Duration((beats * Duration::BEAT as f64) as i64) }
    pub fn from_beats_f32(beats: f32) -> Self { Duration::from_beats_f64(beats as f64) }
    pub fn div_and_ceil(self, other: Duration) -> i64 { (self.0 + other.0 - 1) / other.0 }
    pub fn round(self, grid: Duration) -> Duration { ((self + grid / 2) / grid) * grid }
    pub fn floor(self, grid: Duration) -> Duration { (self / grid) * grid }
    pub fn ceil(self, grid: Duration) -> Duration { self.div_and_ceil(grid) * grid }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Duration {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_f64(self.0 as f64 / Duration::BEAT as f64)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Duration {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let time = f64::deserialize(deserializer)?;
        Ok(Duration((time * Duration::BEAT as f64) as i64))
    }
}

impl std::ops::Neg for Duration {
    type Output = Duration;
    fn neg(self) -> Self::Output { Duration(-self.0) }
}

/// Implements `Op<Rhs>` and `OpAssign<Rhs>` for Duration, where the result is Duration.
macro_rules! impl_duration_op {
    ($Op:ident, $op:ident, $OpAssign:ident, $op_assign:ident, $Rhs:ty, $expr:expr) => {
        impl std::ops::$Op<$Rhs> for Duration {
            type Output = Duration;
            fn $op(self, rhs: $Rhs) -> Self::Output {
                let f = $expr;
                f(self.0, rhs)
            }
        }
        impl std::ops::$OpAssign<$Rhs> for Duration {
            fn $op_assign(&mut self, rhs: $Rhs) { *self = std::ops::$Op::$op(*self, rhs); }
        }
    };
}

impl_duration_op!(
    Add,
    add,
    AddAssign,
    add_assign,
    Duration,
    |a: i64, b: Duration| Duration(a + b.0)
);
impl_duration_op!(
    Sub,
    sub,
    SubAssign,
    sub_assign,
    Duration,
    |a: i64, b: Duration| Duration(a - b.0)
);
impl_duration_op!(
    Rem,
    rem,
    RemAssign,
    rem_assign,
    Duration,
    |a: i64, b: Duration| Duration(a % b.0)
);
impl_duration_op!(Mul, mul, MulAssign, mul_assign, i64, |a: i64, b: i64| {
    Duration(a * b)
});
impl_duration_op!(Div, div, DivAssign, div_assign, i64, |a: i64, b: i64| {
    Duration(a / b)
});
impl_duration_op!(Rem, rem, RemAssign, rem_assign, i64, |a: i64, b: i64| {
    Duration(a % b)
});
impl_duration_op!(Mul, mul, MulAssign, mul_assign, f32, |a: i64, b: f32| {
    Duration((a as f64 * b as f64) as i64)
});
impl_duration_op!(Div, div, DivAssign, div_assign, f32, |a: i64, b: f32| {
    Duration((a as f64 / b as f64) as i64)
});
impl_duration_op!(Mul, mul, MulAssign, mul_assign, f64, |a: i64, b: f64| {
    Duration((a as f64 * b) as i64)
});
impl_duration_op!(Div, div, DivAssign, div_assign, f64, |a: i64, b: f64| {
    Duration((a as f64 / b) as i64)
});

// Duration / Duration -> i64 (no assign variant, different output type)
impl std::ops::Div<Duration> for Duration {
    type Output = i64;
    fn div(self, rhs: Duration) -> Self::Output { self.0 / rhs.0 }
}

// i64 * Duration -> Duration (commutative, no assign variant)
impl std::ops::Mul<Duration> for i64 {
    type Output = Duration;
    fn mul(self, rhs: Duration) -> Self::Output { Duration(self * rhs.0) }
}

impl Lerp for Duration {
    fn inverse_lerp(self, range: std::ops::RangeInclusive<Self>) -> f32 {
        (self - *range.start()).beats() as f32 / (*range.end() - *range.start()).beats() as f32
    }
    fn lerp(range: std::ops::RangeInclusive<Self>, t: f32) -> Self {
        *range.start() + (*range.end() - *range.start()) * t
    }
}
