use crate::note::harmony::{Interval, Pitch, PitchRange};
use crate::note::rhythm::{Duration, Time, TimeRange};

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
    pub fn list_pitches(&self) -> impl Iterator<Item = Pitch> + '_ {
        let mut pitch = self.pitch_range.start;
        std::iter::from_fn(move || {
            if pitch >= self.pitch_range.end {
                return None;
            }

            pitch += Interval::HALFSTEP;
            Some(pitch)
        })
    }

    /// Zooms in or out by factor. Pivot defines where to zoom.
    pub fn zoom_by_factor(&mut self, factor: Vec2, pivot: (Time, Pitch)) {
        let (time_pivot, pitch_pivot) = pivot;

        self.time_range.start = time_pivot - (time_pivot - self.time_range.start) * factor.x;
        self.time_range.end = time_pivot + (self.time_range.end - time_pivot) * factor.x;

        self.pitch_range.start = pitch_pivot - (pitch_pivot - self.pitch_range.start) * factor.y;
        self.pitch_range.end = pitch_pivot + (self.pitch_range.end - pitch_pivot) * factor.y;
    }

    /// Zooms in or out by a number of clicks.
    /// You likely want to scale the clicks by some factor.
    /// Pivot defines where to zoom to/out of.
    pub fn zoom_by_clicks(&mut self, clicks: Vec2, pivot: (Time, Pitch)) {
        self.zoom_by_factor(
            Vec2 {
                x: 2f32.powf(clicks.x),
                y: 2f32.powf(clicks.y),
            },
            pivot,
        );
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
pub struct MidiRoll {
    pub rect:     Rect,
    pub viewport: MidiRollViewport,
}
impl MidiRoll {
    pub fn new(rect: Rect, viewport: MidiRollViewport) -> Self { MidiRoll { rect, viewport } }

    // Grid methods
    pub fn beat_width(&self) -> f32 {
        self.rect.width / (self.viewport.time_range.end - self.viewport.time_range.start).beats()
    }
    pub fn halfstep_height(&self) -> f32 {
        self.rect.height
            / (self.viewport.pitch_range.end - self.viewport.pitch_range.start).halfsteps()
    }
    pub fn width_to_beats(&self, width: f32) -> Duration {
        Duration::from_beats_f32(width / self.beat_width())
    }
    pub fn height_to_halfsteps(&self, height: f32) -> Interval {
        Interval::HALFSTEP * height / self.halfstep_height()
    }
    pub fn time_to_x(&self, time: Time) -> f32 {
        self.rect.x + (time - self.viewport.time_range.start).beats() * self.beat_width()
    }
    pub fn pitch_to_y(&self, pitch: Pitch) -> f32 {
        self.rect.y + (self.viewport.pitch_range.end - pitch).halfsteps() * self.halfstep_height()
    }
    pub fn x_to_time(&self, x: f32) -> Time {
        self.viewport.time_range.start
            + Duration::from_beats_f32((x - self.rect.x) / self.beat_width())
    }
    pub fn y_to_pitch(&self, y: f32) -> Pitch {
        self.viewport.pitch_range.end - self.height_to_halfsteps(y)
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

    pub fn note_box(&self, time: Time, duration: Duration, pitch: Pitch) -> Rect {
        Rect {
            x: self.time_to_x(time),
            y: self.pitch_to_y(pitch),
            width: self.beat_width() * duration.beats(),
            height: self.halfstep_height(),
        }
    }
}
