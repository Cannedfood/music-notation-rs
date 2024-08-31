use std::fmt::Display;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Velocity(pub u8);
impl Velocity {
    pub fn from_midi(vel: u8) -> Self { Self(vel) }
    pub fn as_midi(&self) -> u8 { self.0 }

    pub fn from_f32(f: f32) -> Self { Velocity((f * 127.0).clamp(0.0, 127.0) as u8) }
    pub fn from_f64(f: f64) -> Self { Velocity((f * 127.0).clamp(0.0, 127.0) as u8) }
    pub fn to_f32(&self) -> f32 { self.0 as f32 / 127.0 }
    pub fn to_f64(&self) -> f64 { self.0 as f64 / 127.0 }
}
impl Display for Velocity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{}", self.0) }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Finger {
    Thumb,
    Index,
    Middle,
    Ring,
    Pinky,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Hand {
    Left,
    Right,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Fraction(pub u8);
impl Fraction {
    pub fn from_f32(f: f32) -> Self { Fraction((f * 255.0).clamp(0.0, 255.0) as u8) }
    pub fn from_f64(f: f64) -> Self { Fraction((f * 255.0).clamp(0.0, 255.0) as u8) }
    pub fn to_f32(&self) -> f32 { self.0 as f32 / 255.0 }
    pub fn to_f64(&self) -> f64 { self.0 as f64 / 255.0 }
}
impl From<Fraction> for f32 {
    fn from(val: Fraction) -> Self { val.to_f32() }
}
impl From<Fraction> for f64 {
    fn from(val: Fraction) -> Self { val.to_f64() }
}
impl From<f32> for Fraction {
    fn from(f: f32) -> Self { Fraction::from_f32(f) }
}
impl From<f64> for Fraction {
    fn from(f: f64) -> Self { Fraction::from_f64(f) }
}
