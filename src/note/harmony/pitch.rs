use std::fmt::Display;

use super::{Chroma, Interval};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Octave(pub i8);

#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Pitch(pub f32);

impl Pitch {
    pub fn from_midi(midi: u8) -> Self { Pitch(midi as f32) }
    pub fn to_midi(self) -> u8 { self.0 as u8 }
    pub fn to_hertz(self) -> f32 { 440.0 * 2.0_f32.powf((self.0 - 69.0) / 12.0) }
    pub fn from_chroma_octave(chroma: impl Into<Chroma>, octave: impl Into<Octave>) -> Self {
        let octave = octave.into();
        let chroma = chroma.into();

        let chroma = chroma.to_midi_chroma() as i32;
        let octave = octave.0 as i32;
        Pitch::from_midi((chroma + octave * 12) as u8)
    }
    pub fn chroma(&self) -> Chroma { Chroma::from_midi_chroma(self.to_midi() % 12).unwrap() }
    pub fn octave(&self) -> Octave { Octave((self.to_midi() / 12) as i8) }
    pub fn frequency(&self) -> f64 { 440.0 * 2.0_f64.powf((self.0 as f64 - 69.0) / 12.0) }
}
impl Display for Pitch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.chroma(), self.octave().0)
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
