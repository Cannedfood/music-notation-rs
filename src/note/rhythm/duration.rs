#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Duration(pub(crate) i64);

impl Duration {
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

    pub fn beats(self) -> f32 { self.0 as f32 / Duration::BEAT as f32 }
    pub fn from_beats_f32(beats: f32) -> Self { Duration((beats * Duration::BEAT as f32) as i64) }
    pub fn div_and_ceil(self, other: Duration) -> i64 { (self.0 + other.0 - 1) / other.0 }
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
impl std::ops::Add<Duration> for Duration {
    type Output = Duration;
    fn add(self, rhs: Duration) -> Self::Output { Duration(self.0 + rhs.0) }
}
impl std::ops::Sub<Duration> for Duration {
    type Output = Duration;
    fn sub(self, rhs: Duration) -> Self::Output { Duration(self.0 - rhs.0) }
}
impl std::ops::Div<Duration> for Duration {
    type Output = i64;
    fn div(self, rhs: Duration) -> Self::Output { self.0 / rhs.0 }
}
impl std::ops::Mul<i64> for Duration {
    type Output = Duration;
    fn mul(self, rhs: i64) -> Self::Output { Duration(self.0 * rhs) }
}
impl std::ops::Div<i64> for Duration {
    type Output = Duration;
    fn div(self, rhs: i64) -> Self::Output { Duration(self.0 / rhs) }
}
impl std::ops::Mul<f32> for Duration {
    type Output = Duration;
    fn mul(self, rhs: f32) -> Self::Output { Duration((self.0 as f64 * rhs as f64) as i64) }
}
impl std::ops::Div<f32> for Duration {
    type Output = Duration;
    fn div(self, rhs: f32) -> Self::Output { Duration((self.0 as f64 / rhs as f64) as i64) }
}
impl std::ops::Mul<f64> for Duration {
    type Output = Duration;
    fn mul(self, rhs: f64) -> Self::Output { Duration((self.0 as f64 * rhs) as i64) }
}
impl std::ops::Div<f64> for Duration {
    type Output = Duration;
    fn div(self, rhs: f64) -> Self::Output { Duration((self.0 as f64 / rhs) as i64) }
}
