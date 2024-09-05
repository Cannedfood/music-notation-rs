use super::{Duration, Time, TimeRange};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TimeGrid {
    pub start: Time,
    pub step:  Duration,
}
impl TimeGrid {
    pub fn new(start: Time, step: Duration) -> Self { TimeGrid { start, step } }
    pub fn iter_in_range(self, range: TimeRange) -> impl Iterator<Item = (i64, Time)> {
        let TimeGrid { start, step } = self;
        let start_idx = (range.start - self.start) / step;
        let end_idx = (range.end - self.start) / step;
        (start_idx..end_idx).map(move |i| (i, start + step * i))
    }
    pub fn closest(&self, time: Time) -> Option<Time> {
        let i = (time - self.start) / self.step;
        Some(self.start + self.step * i)
    }
}

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

    pub fn beats(self, start: Time, range: TimeRange) -> impl Iterator<Item = (Time, u32)> {
        let note_length = self.subdivision_duration();

        let total = range.end - start;
        let num_beats = (total + Duration(note_length.0 - 1)) / note_length;
        let first_visible_beat = (range.start.max(start) - start) / note_length;

        (first_visible_beat..num_beats)
            .map(move |i| (start + note_length * i, (i % self.numerator as i64) as u32))
    }

    pub fn grid(self, start_at: Time) -> TimeGrid {
        TimeGrid::new(start_at, self.subdivision_duration())
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
