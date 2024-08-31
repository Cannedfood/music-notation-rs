use crate::note::harmony::{Chroma, Interval, Octave, Pitch};
use crate::note::Note;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct GuitarTuning {
    pub strings: Vec<Pitch>,
    pub frets:   usize,
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
        GuitarTuning { strings, frets: 24 }
    }
    pub fn from_pitches(strings: Vec<Pitch>) -> GuitarTuning { GuitarTuning { strings, frets: 24 } }

    pub fn standard() -> GuitarTuning {
        GuitarTuning::from_intervals(Pitch::from_chroma_octave(Chroma::E, Octave(4)), &[
            Interval::FOURTH,
            Interval::FOURTH,
            Interval::FOURTH,
            Interval::MAJOR_THIRD,
            Interval::FOURTH,
        ])
    }
}

pub fn guess_fingerings(tuning: &GuitarTuning, notes: &mut [Note]) {
    // TODO: Use an actual heuristic
    for note in notes {
        if note.string.is_some() {
            continue;
        }

        note.string = tuning
            .strings
            .iter()
            .enumerate()
            .map(|(string, pitch)| {
                (
                    string,
                    (*pitch..=(*pitch + Interval::from_halfsteps(tuning.frets as f32))),
                )
            })
            .filter(|(_, pitch_range)| pitch_range.contains(&note.pitch))
            .map(|(string, _)| string as u8)
            .next();
    }
}
