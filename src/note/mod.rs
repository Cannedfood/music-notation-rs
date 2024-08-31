use std::collections::BTreeMap;

use articulation::Fraction;

pub mod articulation;
pub mod harmony;
pub mod rhythm;

#[derive(Debug, Default, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Note {
    // Timing
    pub time:     rhythm::Time,
    pub duration: rhythm::Duration,

    // Pitch
    pub pitch:    harmony::Pitch,
    pub velocity: articulation::Velocity,

    // Playing details
    pub finger: Option<(articulation::Finger, articulation::Hand)>,
    pub string: Option<u8>,
    pub damping: Option<Fraction>,
    pub channel: Option<u8>,
    pub aftertouch: BTreeMap<rhythm::Time, articulation::Velocity>,
    pub bend: BTreeMap<rhythm::Time, f32>,
}
