use super::Duration;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Time(pub(crate) i64);

pub type TimeRange = core::range::Range<Time>;

impl Time {
    pub const ZERO: Time = Time(0);
}

#[cfg(feature = "serde")]
impl serde::Serialize for Time {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_f64(self.0 as f64 / Duration::BEAT as f64)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Time {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let time = f64::deserialize(deserializer)?;
        Ok(Time((time * Duration::BEAT as f64) as i64))
    }
}

// Time math ops
impl std::ops::Add<Duration> for Time {
    type Output = Time;
    fn add(self, rhs: Duration) -> Self::Output { Time(self.0 + rhs.0) }
}

impl std::ops::Sub<Duration> for Time {
    type Output = Time;
    fn sub(self, rhs: Duration) -> Self::Output { Time(self.0 - rhs.0) }
}

impl std::ops::Sub<Time> for Time {
    type Output = Duration;
    fn sub(self, rhs: Time) -> Self::Output { Duration(self.0 - rhs.0) }
}

// Assign ops
impl std::ops::AddAssign<Duration> for Time {
    fn add_assign(&mut self, rhs: Duration) { self.0 += rhs.0; }
}

impl std::ops::SubAssign<Duration> for Time {
    fn sub_assign(&mut self, rhs: Duration) { self.0 -= rhs.0; }
}
