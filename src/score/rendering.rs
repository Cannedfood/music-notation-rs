use super::Score;
use crate::note::harmony::{Interval, Pitch};
use crate::note::time::{Duration, Time, TimeSignature};

#[derive(Debug, Clone, Copy)]
pub struct Viewport {
    pub time_start:  Time,
    pub time_end:    Time,
    pub pitch_start: Pitch,
    pub pitch_end:   Pitch,
}
impl Default for Viewport {
    fn default() -> Self {
        Viewport {
            time_start:  Time::ZERO,
            time_end:    Time::ZERO + Duration::from_beats_f32(4.0),
            pitch_start: Pitch::from_midi(0),
            pitch_end:   Pitch::from_midi(127),
        }
    }
}
impl Viewport {
    pub fn list_pitches(&self) -> impl Iterator<Item = Pitch> + '_ {
        let mut pitch = self.pitch_start;
        std::iter::from_fn(move || {
            if pitch >= self.pitch_end {
                return None;
            }

            pitch = pitch + Interval::HALFSTEP;
            Some(pitch)
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}
impl Rect {
    pub fn left(&self) -> f32 { self.x }
    pub fn right(&self) -> f32 { self.x + self.width }
    pub fn top(&self) -> f32 { self.y }
    pub fn bottom(&self) -> f32 { self.y + self.height }
}

#[derive(Debug, Clone, Copy)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

pub struct NoteLine {
    pub x_start: f32,
    pub x_end: f32,
    pub y: f32,
}
pub struct TimeLine {
    pub x: f32,
    pub y_start: f32,
    pub y_end: f32,
    pub is_bar_line: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct MidiRoll<'a> {
    pub rect:     Rect,
    pub viewport: Viewport,
    pub score:    &'a Score,
}
impl<'a> MidiRoll<'a> {
    pub fn new(rect: Rect, viewport: Viewport, score: &'a Score) -> Self {
        MidiRoll {
            rect,
            viewport,
            score,
        }
    }

    // Grid methods
    pub fn beat_width(&self) -> f32 {
        self.rect.width / (self.viewport.time_end - self.viewport.time_start).beats()
    }
    pub fn halfstep_height(&self) -> f32 {
        self.rect.height / (self.viewport.pitch_end - self.viewport.pitch_start).halfsteps()
    }
    pub fn width_to_beats(&self, width: f32) -> Duration {
        Duration::from_beats_f32(width / self.beat_width())
    }
    pub fn height_to_halfsteps(&self, height: f32) -> Duration {
        Duration::from_beats_f32(height / self.halfstep_height())
    }
    pub fn time_to_x(&self, time: Time) -> f32 {
        self.rect.x + (time - self.viewport.time_start).beats() * self.beat_width()
    }
    pub fn pitch_to_y(&self, pitch: Pitch) -> f32 {
        self.rect.y + (self.viewport.pitch_end - pitch).halfsteps() * self.halfstep_height()
    }

    // Grid drawing
    pub fn note_lines(&self) -> impl Iterator<Item = NoteLine> + '_ {
        self.viewport.list_pitches().map(|pitch| {
            let y = self.pitch_to_y(pitch);
            NoteLine {
                x_start: self.rect.x,
                x_end: self.rect.x + self.rect.width,
                y,
            }
        })
    }
    pub fn time_lines(&self) -> Vec<TimeLine> { Vec::new() }

    pub fn time_lines_for(
        &self,
        start: Time,
        end: Time,
        sig: TimeSignature,
    ) -> impl Iterator<Item = TimeLine> + '_ {
        let step = sig.subdivision_duration();
        let mut time = start;
        let mut counter = 0;
        std::iter::from_fn(move || {
            if time >= end {
                return None;
            }

            let is_bar_line = counter % sig.numerator == 0;
            let line = TimeLine {
                x: self.time_to_x(time),
                y_start: self.rect.y,
                y_end: self.rect.y + self.rect.height,
                is_bar_line,
            };
            counter += 1;
            time += step;

            Some(line)
        })
    }

    pub fn note_box(&self, time: Time, duration: Duration, pitch: Pitch) -> Rect {
        Rect {
            x: self.time_to_x(time),
            y: self.pitch_to_y(pitch),
            width: self.beat_width() * duration.beats(),
            height: self.halfstep_height(),
        }
    }
}
