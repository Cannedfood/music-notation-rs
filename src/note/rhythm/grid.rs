use super::{Duration, Time, TimeRange};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TimeGrid {
    pub range: TimeRange,
    pub step:  Duration,
}
impl TimeGrid {
    pub fn new(range: TimeRange, step: Duration) -> Self { TimeGrid { range, step } }
    pub fn iter(&self) -> impl Iterator<Item = Time> {
        let start = self.range.start;
        let step = self.step;
        let n = (self.range.end - self.range.start).div_and_ceil(self.step);
        (0..n).map(move |i| start + step * i)
    }
    pub fn closest(&self, time: Time) -> Option<Time> {
        if time < self.range.start || time >= self.range.end {
            return None;
        }

        let i = (time - self.range.start) / self.step;
        Some(self.range.start + self.step * i)
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
