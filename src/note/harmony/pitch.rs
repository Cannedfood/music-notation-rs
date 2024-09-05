use std::fmt::Display;

use super::{Chroma, Interval};

#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// The pitch of a note, stored as midi note number and cents.
/// Includes chroma, octave, and cents (1 cent = 1 halfstep/100).
pub struct Pitch(pub f32);
pub type PitchRange = core::range::Range<Pitch>;
impl Pitch {
    pub fn from_midi(midi: i32) -> Self { Pitch(midi as f32) }
    pub fn to_midi(self) -> i32 { self.0 as i32 }

    pub fn from_chroma_octave(chroma: impl Into<Chroma>, octave: i32) -> Self {
        let chroma = chroma.into().to_midi_chroma() as i32;
        Pitch::from_midi(chroma + octave * 12)
    }
    pub fn chroma(&self) -> Chroma {
        Chroma::from_midi_chroma((self.to_midi() % 12) as u8).unwrap()
    }
    pub fn octave(&self) -> i8 { (self.to_midi() / 12) as i8 - 1 }
    pub fn cents(&self) -> f32 { self.0.fract() / 100.0 }
    pub fn with_octave(&self, octave: i32) -> Self {
        Pitch::from_chroma_octave(self.chroma(), octave)
    }
    pub fn with_cents(&self, cents: f32) -> Self { Pitch(self.0.floor() + cents / 100.0) }

    pub fn to_hertz(self) -> f32 { 440.0 * 2.0_f32.powf((self.0 - 69.0) / 12.0) }
    pub fn from_hertz(hertz: f32) -> Self { Pitch(69.0 + 12.0 * (hertz / 440.0).log2()) }
}
impl Display for Pitch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.chroma(), self.octave())
    }
}

// Pitch math ops
impl std::ops::Sub<Pitch> for Pitch {
    type Output = Interval;
    fn sub(self, rhs: Pitch) -> Self::Output { Interval(self.0 - rhs.0) }
}
impl std::ops::Add<Interval> for Pitch {
    type Output = Pitch;
    fn add(self, rhs: Interval) -> Self::Output { Pitch(self.0 + rhs.0) }
}
impl std::ops::Sub<Interval> for Pitch {
    type Output = Pitch;
    fn sub(self, rhs: Interval) -> Self::Output { Pitch(self.0 - rhs.0) }
}
impl std::ops::AddAssign<Interval> for Pitch {
    fn add_assign(&mut self, rhs: Interval) { self.0 += rhs.0 }
}
impl std::ops::SubAssign<Interval> for Pitch {
    fn sub_assign(&mut self, rhs: Interval) { self.0 -= rhs.0 }
}
