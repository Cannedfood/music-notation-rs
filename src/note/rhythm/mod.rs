mod duration;
mod grid;
mod time;

pub use duration::*;
pub use grid::*;
pub use time::*;

#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Tempo(pub f32);
impl Tempo {
    pub fn from_micros_per_beat(micros_per_beat: u32) -> Self {
        Tempo(60_000_000.0 / micros_per_beat as f32)
    }
    pub fn to_duration(self) -> Duration { Duration((60_000.0 / self.0) as i64) }
}
