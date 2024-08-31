use crate::note::harmony::{Chroma, Interval, Octave, Pitch};
use crate::note::Note;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct GuitarTuning {
    pub strings: Vec<Pitch>,
}
impl Default for GuitarTuning {
    fn default() -> GuitarTuning { GuitarTuning::standard() }
}
impl GuitarTuning {
    pub fn from_intervals(lowest: Pitch, intervals: &[Interval]) -> GuitarTuning {
        let mut strings = vec![lowest];
        for interval in intervals {
            strings.push(*strings.last().unwrap() + *interval);
        }
        GuitarTuning { strings }
    }
    pub fn from_pitches(strings: Vec<Pitch>) -> GuitarTuning { GuitarTuning { strings } }

    pub fn standard() -> GuitarTuning {
        GuitarTuning::from_intervals(Pitch::from_chroma_octave(Chroma::E, Octave(4)), &[
            Interval::FOURTH,
            Interval::FOURTH,
            Interval::FOURTH,
            Interval::MAJOR_THIRD,
            Interval::FOURTH,
            Interval::FOURTH,
        ])
    }
}

pub fn guess_strings(notes: &[Note]) -> GuitarTuning {
    let mut strings = vec![];
    for note in notes {
        if let Some(string) = note.string {
            if string as usize >= strings.len() {
                strings.resize(string as usize + 1, Pitch::default());
            }
            strings[string as usize] = note.pitch;
        }
    }
    GuitarTuning { strings }
}
