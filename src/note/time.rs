#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Tempo(pub f32);
impl Tempo {
    pub fn from_micros_per_beat(micros_per_beat: u32) -> Self {
        Tempo(60_000_000.0 / micros_per_beat as f32)
    }
    pub fn to_duration(self) -> Duration { Duration((60_000.0 / self.0) as i64) }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Duration(i64);
impl Duration {
    const BEAT: i64 = 2i64.pow(8) * 3i64.pow(2) * 5i64.pow(2) * 7i64.pow(2);

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

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Time(i64);
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

pub type TimeRange = core::range::Range<Time>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TimeSignature {
    pub numerator:   u8,
    pub subdivision: u8,
}
impl TimeSignature {
    pub fn subdivision_duration(&self) -> Duration {
        Duration(Duration::BEAT / self.subdivision as i64)
    }
    pub fn bar_length(&self) -> Duration { self.subdivision_duration() * self.numerator as i64 }
    pub fn bars_in(&self, start: Time, end: Time) -> impl Iterator<Item = Time> {
        let num_bars = ((end - start) + Duration(self.bar_length().0 - 1)) / self.bar_length();
        let bar_length = self.bar_length();
        (0..num_bars).map(move |i| start + bar_length * i)
    }
}
impl Default for TimeSignature {
    fn default() -> Self {
        TimeSignature {
            numerator:   4,
            subdivision: 4,
        }
    }
}
impl From<(u8, u8)> for TimeSignature {
    fn from((numerator, denominator): (u8, u8)) -> Self {
        TimeSignature {
            numerator,
            subdivision: denominator,
        }
    }
}

// Duration math ops
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
