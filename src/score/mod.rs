pub mod edit;
pub mod rendering;

use core::str;
use std::ops::Range;

use crate::note::articulation::Velocity;
use crate::note::harmony::{KeySignature, Pitch};
use crate::note::rhythm::{Duration, Tempo, Time, TimeSignature};
use crate::note::Note;

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Score {
    pub tracks:    Vec<Part>,
    pub tempo_map: Vec<(Time, Tempo)>,
}

#[derive(Debug, Clone, Default, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Part {
    pub description: String,
    pub notes: Vec<Note>,
    pub time_signature: Vec<(Time, TimeSignature)>,
    pub key_signature: Vec<(Time, KeySignature)>,
}
impl Part {
    pub fn bars(&self) -> impl Iterator<Item = (Time, TimeSignature)> + '_ {
        self.notes
            .iter()
            .map(|n| n.time + n.duration)
            .max()
            .unwrap_or(Time::ZERO);

        return TimeSignature::default()
            .bars_in(
                Time::ZERO,
                self.notes
                    .iter()
                    .map(|n| n.time + n.duration)
                    .max()
                    .unwrap_or(Time::ZERO),
            )
            .map(|t| (t, TimeSignature::default()));
    }
}

pub struct Bar {
    pub start: Time,
    pub end: Time,
    pub time_signature: TimeSignature,
    pub notes: Range<usize>,
}

#[cfg(feature = "midly")]
#[derive(Debug, Clone)]
pub enum FromMidiError {
    ParseError(midly::Error),
}
#[cfg(feature = "midly")]
impl From<midly::Error> for FromMidiError {
    fn from(e: midly::Error) -> Self { FromMidiError::ParseError(e) }
}

#[cfg(feature = "midly")]
impl Score {
    pub fn from_midi_data(data: &[u8]) -> Result<Self, FromMidiError> {
        use midly::{MetaMessage, MidiMessage, Timing, TrackEvent, TrackEventKind};

        let (header, tracks) = midly::parse(data)?;

        let mut result = Score::default();

        for track in tracks {
            let mut track_data = Part::default();
            let mut time = Time::ZERO;
            let (mut time_numerator, mut time_denomintr) = match header.timing {
                // Default 120 BPM until we get a tempo event
                Timing::Timecode(frames_per_second, subframes_per_frame) => (
                    120,
                    60 * frames_per_second.as_int() as u64 * subframes_per_frame as u64,
                ),
                Timing::Metrical(ticks_per_beat) => (1u64, ticks_per_beat.as_int() as u64),
            };

            for event in track? {
                let TrackEvent { delta, kind } = event?;
                time += (Duration::QUARTER * delta.as_int() as i64 * time_numerator as i64)
                    / time_denomintr as i64;

                match kind {
                    TrackEventKind::Midi { channel, message } => match message {
                        MidiMessage::NoteOn { key, vel } => track_data.notes.push(Note {
                            time,
                            pitch: Pitch::from_midi(key.as_int()),
                            velocity: Velocity::from_midi(vel.as_int()),
                            duration: Duration::SIXTEENTH,
                            channel: Some(channel.as_int()),
                            ..Default::default()
                        }),
                        MidiMessage::NoteOff { key, vel: _ } => {
                            // Find the note by pitch and set the duration so it ends *now*
                            let pitch = Pitch::from_midi(key.as_int());
                            let note = track_data.notes.iter_mut().rev().find(|n| n.pitch == pitch);
                            if let Some(note) = note {
                                note.duration = time - note.time;
                            }
                        }
                        MidiMessage::Aftertouch { key: _, vel: _ } => (),
                        MidiMessage::Controller {
                            controller: _,
                            value: _,
                        } => (),
                        MidiMessage::ProgramChange { program: _ } => (),
                        MidiMessage::ChannelAftertouch { vel: _ } => (),
                        MidiMessage::PitchBend { bend: _ } => (),
                    },
                    TrackEventKind::SysEx(_) => (),
                    TrackEventKind::Escape(_) => (),
                    TrackEventKind::Meta(meta) => match meta {
                        MetaMessage::TrackNumber(_) => (),
                        MetaMessage::Text(_) => (),
                        MetaMessage::Copyright(_) => (),
                        MetaMessage::TrackName(name) => {
                            if let Ok(name) = str::from_utf8(name) {
                                track_data.description += name;
                            }
                        }
                        MetaMessage::InstrumentName(name) => {
                            if let Ok(name) = str::from_utf8(name) {
                                track_data.description += name;
                            }
                        }
                        MetaMessage::Lyric(_) => (),
                        MetaMessage::Marker(_) => (),
                        MetaMessage::CuePoint(_) => (),
                        MetaMessage::ProgramName(_) => (),
                        MetaMessage::DeviceName(name) => {
                            if let Ok(name) = str::from_utf8(name) {
                                track_data.description += name;
                            }
                        }
                        MetaMessage::MidiChannel(_) => (),
                        MetaMessage::MidiPort(_) => (),
                        MetaMessage::EndOfTrack => (),
                        MetaMessage::Tempo(micros_per_beat) => {
                            if let Timing::Timecode(frames_per_second, subframes_per_frame) =
                                header.timing
                            {
                                (time_numerator, time_denomintr) = (
                                    micros_per_beat.as_int() as u64,
                                    1000000
                                        * frames_per_second.as_int() as u64
                                        * subframes_per_frame as u64,
                                );
                            }

                            result.tempo_map.push((
                                time,
                                Tempo::from_micros_per_beat(micros_per_beat.as_int()),
                            ));
                        }
                        MetaMessage::SmpteOffset(_) => (),
                        MetaMessage::TimeSignature(
                            numerator,
                            denominator,
                            _clocks_per_click,
                            _32nd_per_notes_per_quarter,
                        ) => {
                            track_data
                                .time_signature
                                .push((time, (numerator, denominator).into()));
                        }
                        MetaMessage::KeySignature(sharps_flats, minor) => {
                            track_data
                                .key_signature
                                .push((time, KeySignature::from_midi(sharps_flats, minor)));
                        }
                        MetaMessage::SequencerSpecific(_) => (),
                        MetaMessage::Unknown(..) => (),
                    },
                }
            }

            result.tracks.push(track_data);
        }

        Ok(result)
    }
}
