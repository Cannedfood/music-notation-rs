pub mod edit;

use core::str;
use std::ops::Range;

use crate::note::Note;
use crate::note::articulation::Velocity;
use crate::note::harmony::{KeySignature, Pitch};
use crate::note::rhythm::{Duration, Tempo, Time, TimeSignature};

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Score {
    pub parts:     Vec<Part>,
    pub tempo_map: Vec<(Time, Tempo)>,
}
impl Score {
    pub fn time_range(&self) -> Option<Range<Time>> {
        let ranges: Vec<_> = self.parts.iter().filter_map(|p| p.time_range()).collect();
        let start = ranges.iter().map(|r| r.start).min()?;
        let end = ranges.iter().map(|r| r.end).max()?;
        Some(start..end)
    }
    pub fn pitch_range(&self) -> Option<Range<Pitch>> {
        let ranges: Vec<_> = self.parts.iter().filter_map(|p| p.pitch_range()).collect();
        let start = ranges
            .iter()
            .map(|r| r.start)
            .min_by(|a, b| a.partial_cmp(b).unwrap())?;
        let end = ranges
            .iter()
            .map(|r| r.end)
            .max_by(|a, b| a.partial_cmp(b).unwrap())?;
        Some(start..end)
    }
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
    pub fn time_range(&self) -> Option<Range<Time>> {
        let start = self.notes.iter().map(|n| n.time).min()?;
        let end = self.notes.iter().map(|n| n.time + n.duration).max()?;
        Some(start..end)
    }
    pub fn pitch_range(&self) -> Option<Range<Pitch>> {
        let start = self
            .notes
            .iter()
            .map(|n| n.pitch)
            .min_by(|a, b| a.partial_cmp(b).unwrap())?;
        let end = self
            .notes
            .iter()
            .map(|n| n.pitch)
            .max_by(|a, b| a.partial_cmp(b).unwrap())?;
        Some(start..end)
    }

    pub fn bars(&self) -> impl Iterator<Item = (Time, TimeSignature)> + '_ {
        self.notes
            .iter()
            .map(|n| n.time + n.duration)
            .max()
            .unwrap_or(Time::ZERO);

        TimeSignature::default()
            .bars_in(
                Time::ZERO,
                self.notes
                    .iter()
                    .map(|n| n.time + n.duration)
                    .max()
                    .unwrap_or(Time::ZERO),
            )
            .map(|t| (t, TimeSignature::default()))
    }
}

pub struct Bar {
    pub start: Time,
    pub end: Time,
    pub time_signature: TimeSignature,
    pub notes: Range<usize>,
}

// ── TuxGuitar (.tg) import ──────────────────────────────────────────────────

#[cfg(feature = "tuxguitar")]
#[derive(Debug, Clone)]
pub enum FromTgError {
    ReadError(tuxguitar::reader::ReadError),
}

#[cfg(feature = "tuxguitar")]
impl From<tuxguitar::reader::ReadError> for FromTgError {
    fn from(e: tuxguitar::reader::ReadError) -> Self { FromTgError::ReadError(e) }
}

#[cfg(feature = "tuxguitar")]
impl Score {
    /// Parse a TuxGuitar `.tg` file from a reader and convert it into a [`Score`].
    pub fn from_tg_data<R: std::io::Read + std::io::Seek>(reader: R) -> Result<Self, FromTgError> {
        let tg_song = tuxguitar::reader::read_tg(reader)?;
        Ok(Self::from_tg_song(&tg_song))
    }

    /// Convert a parsed [`tuxguitar::model::TgSong`] into a [`Score`].
    pub fn from_tg_song(song: &tuxguitar::model::TgSong) -> Self {
        use tuxguitar::model as tg;

        // Scale factor: TG uses 960 ticks per quarter, we use Duration::BEAT.
        const SCALE: i64 = Duration::BEAT / tg::QUARTER_TIME;

        // Build the tempo map from measure headers.
        let mut tempo_map: Vec<(Time, Tempo)> = Vec::new();
        let mut prev_tempo_qv: i32 = 0;
        for hdr in &song.measure_headers {
            let qv = hdr.tempo.quarter_value();
            if qv != prev_tempo_qv {
                let time = Time((hdr.start - tg::QUARTER_TIME) * SCALE);
                tempo_map.push((time, Tempo(qv as f32)));
                prev_tempo_qv = qv;
            }
        }

        // Convert each track to a Part.
        let parts: Vec<Part> = song
            .tracks
            .iter()
            .map(|track| {
                let mut part = Part {
                    description: track.name.clone(),
                    ..Default::default()
                };

                // Build time_signature changes from measure headers.
                let mut prev_ts: Option<(i32, i32)> = None;
                for hdr in &song.measure_headers {
                    let ts_pair = (hdr.time_signature.numerator, hdr.time_signature.denominator);
                    if prev_ts != Some(ts_pair) {
                        let time = Time((hdr.start - tg::QUARTER_TIME) * SCALE);
                        part.time_signature.push((time, TimeSignature {
                            numerator:   ts_pair.0 as u8,
                            subdivision: ts_pair.1 as u8,
                        }));
                        prev_ts = Some(ts_pair);
                    }
                }

                // Build key_signature changes from measures.
                let mut prev_ks: Option<i32> = None;
                for (measure, hdr) in track.measures.iter().zip(song.measure_headers.iter()) {
                    if prev_ks != Some(measure.key_signature) {
                        let time = Time((hdr.start - tg::QUARTER_TIME) * SCALE);
                        part.key_signature.push((time, KeySignature {
                            flats_sharps: measure.key_signature as i8,
                            major: true,
                        }));
                        prev_ks = Some(measure.key_signature);
                    }
                }

                // Convert notes from beats/voices.
                for (measure_idx, measure) in track.measures.iter().enumerate() {
                    let _hdr = &song.measure_headers[measure_idx];
                    for beat in &measure.beats {
                        // Beat approximate start in TG ticks.
                        let beat_tg_ticks =
                            tg::QUARTER_TIME * 4 * beat.precise_start / tg::WHOLE_PRECISE_DURATION;
                        let beat_time = Time((beat_tg_ticks - tg::QUARTER_TIME) * SCALE);

                        for voice in &beat.voices {
                            if voice.empty {
                                continue;
                            }
                            let voice_dur = Duration(voice.duration.ticks() * SCALE);

                            for note in &voice.notes {
                                // Compute MIDI pitch: string tuning + fret.
                                let midi_pitch = if (note.string as usize) <= track.strings.len()
                                    && note.string >= 1
                                {
                                    let string_val = track.strings[note.string as usize - 1].value;
                                    string_val + note.value
                                }
                                else {
                                    // Fallback: just use the fret value.
                                    note.value
                                };

                                part.notes.push(Note {
                                    time: beat_time,
                                    duration: voice_dur,
                                    pitch: Pitch::from_midi(midi_pitch),
                                    velocity: Velocity::from_midi(note.velocity as u8),
                                    string: if note.string > 0 {
                                        Some(note.string as u8)
                                    }
                                    else {
                                        None
                                    },
                                    ..Default::default()
                                });
                            }
                        }
                    }
                }

                // Sort notes by time for consistent ordering.
                part.notes.sort_by_key(|a| a.time);

                part
            })
            .collect();

        Score { parts, tempo_map }
    }
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
                            pitch: Pitch::from_midi(key.as_int() as i32),
                            velocity: Velocity::from_midi(vel.as_int()),
                            duration: Duration::SIXTEENTH,
                            channel: Some(channel.as_int()),
                            ..Default::default()
                        }),
                        MidiMessage::NoteOff { key, vel: _ } => {
                            // Find the note by pitch and set the duration so it ends *now*
                            let pitch = Pitch::from_midi(key.as_int() as i32);
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

            result.parts.push(track_data);
        }

        Ok(result)
    }
}
