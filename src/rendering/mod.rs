pub mod math2d;

use crate::note::harmony::{Interval, Pitch, PitchRange};
use crate::note::rhythm::{Duration, Time, TimeRange};
use crate::rendering::math2d::{Rect, Vec2};

#[derive(Debug, Clone, Copy)]
pub struct MidiRollViewport {
    pub time_range:  TimeRange,
    pub pitch_range: PitchRange,
}
impl Default for MidiRollViewport {
    fn default() -> Self {
        MidiRollViewport {
            time_range:  (Time::ZERO..Time::ZERO + Duration::WHOLE).into(),
            pitch_range: (Pitch::from_midi(0)..Pitch::from_midi(127)).into(),
        }
    }
}
impl MidiRollViewport {
    pub fn list_pitches(&self) -> impl Iterator<Item = Pitch> { self.pitch_range.into_iter() }

    pub fn as_rect(&self) -> Rect<Time, Pitch> {
        Rect {
            left:   self.time_range.start,
            right:  self.time_range.end,
            top:    self.pitch_range.start,
            bottom: self.pitch_range.end,
        }
    }

    fn apply_rect(&mut self, rect: Rect<Time, Pitch>) {
        self.time_range.start = rect.left;
        self.time_range.end = rect.right;
        self.pitch_range.start = rect.top;
        self.pitch_range.end = rect.bottom;
    }

    /// Zooms in or out by factor. Pivot defines where to zoom.
    pub fn zoom(&mut self, factor: Vec2, pivot: (Time, Pitch)) {
        self.apply_rect(self.as_rect().zoom(factor, Vec2 {
            x: pivot.0,
            y: pivot.1,
        }));
    }

    /// Zooms in or out by a number of clicks.
    /// You likely want to scale the clicks by some factor.
    /// Pivot defines where to zoom to/out of.
    pub fn zoom_by_clicks(&mut self, clicks: Vec2, pivot: (Time, Pitch)) {
        self.apply_rect(self.as_rect().zoom_by_clicks(clicks, Vec2 {
            x: pivot.0,
            y: pivot.1,
        }));
    }
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

#[derive(Default, Debug, Clone, Copy)]
pub struct MidiRoll {
    pub rect:     Rect,
    pub viewport: MidiRollViewport,
}
impl MidiRoll {
    // Grid methods
    pub fn beat_width(&self) -> f32 {
        self.rect.width()
            / (self.viewport.time_range.end - self.viewport.time_range.start).beats() as f32
    }
    pub fn halfstep_height(&self) -> f32 {
        self.rect.height()
            / (self.viewport.pitch_range.end - self.viewport.pitch_range.start).halfsteps()
    }
    pub fn width_to_beats(&self, width: f32) -> Duration {
        Duration::from_beats_f32(width / self.beat_width())
    }
    pub fn height_to_halfsteps(&self, height: f32) -> Interval {
        Interval::HALFSTEP * height / self.halfstep_height()
    }
    pub fn time_to_x(&self, time: Time) -> f32 {
        self.rect.left + (time - self.viewport.time_range.start).beats() as f32 * self.beat_width()
    }
    pub fn pitch_to_y(&self, pitch: Pitch) -> f32 {
        self.rect.top + (self.viewport.pitch_range.end - pitch).halfsteps() * self.halfstep_height()
    }
    pub fn x_to_time(&self, x: f32) -> Time {
        self.viewport.time_range.start
            + Duration::from_beats_f32((x - self.rect.left) / self.beat_width())
    }
    pub fn y_to_pitch(&self, y: f32) -> Pitch {
        self.viewport.pitch_range.end - self.height_to_halfsteps(y)
    }

    // Grid drawing
    pub fn note_lines(&self) -> impl Iterator<Item = NoteLine> + '_ {
        self.viewport.list_pitches().map(|pitch| {
            let y = self.pitch_to_y(pitch);
            NoteLine {
                x_start: self.rect.left,
                x_end: self.rect.right,
                y,
            }
        })
    }

    pub fn note_box(&self, time: Time, duration: Duration, pitch: Pitch) -> Rect {
        let x = self.time_to_x(time);
        let y = self.pitch_to_y(pitch);
        Rect {
            left:   x,
            top:    y,
            right:  x + self.beat_width() * duration.beats() as f32,
            bottom: y + self.halfstep_height(),
        }
    }
}
