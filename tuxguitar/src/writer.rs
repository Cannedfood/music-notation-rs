//! Writer for TuxGuitar `.tg` files (ZIP archive containing version.txt and content.xml).

use std::io::{Cursor, Seek, Write};

use quick_xml::Writer;
use quick_xml::events::{BytesDecl, BytesText, Event};
use zip::write::{SimpleFileOptions, ZipWriter};

use crate::model::*;

// =============================================================================
// Error type
// =============================================================================

/// Errors that can occur when writing a `.tg` file.
#[derive(Debug)]
pub enum WriteError {
    Zip(zip::result::ZipError),
    Xml(quick_xml::Error),
    Io(std::io::Error),
}

impl From<zip::result::ZipError> for WriteError {
    fn from(e: zip::result::ZipError) -> Self { WriteError::Zip(e) }
}

impl From<quick_xml::Error> for WriteError {
    fn from(e: quick_xml::Error) -> Self { WriteError::Xml(e) }
}

impl From<std::io::Error> for WriteError {
    fn from(e: std::io::Error) -> Self { WriteError::Io(e) }
}

impl std::fmt::Display for WriteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WriteError::Zip(e) => write!(f, "ZIP error: {e}"),
            WriteError::Xml(e) => write!(f, "XML error: {e}"),
            WriteError::Io(e) => write!(f, "IO error: {e}"),
        }
    }
}

impl std::error::Error for WriteError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            WriteError::Zip(e) => Some(e),
            WriteError::Xml(e) => Some(e),
            WriteError::Io(e) => Some(e),
        }
    }
}

// =============================================================================
// Public API
// =============================================================================

/// Writes a TuxGuitar song to a `.tg` file (ZIP archive).
///
/// The output contains:
/// - `version.txt` with the format version string
/// - `content.xml` with the full song data as XML
pub fn write_tg<W: Write + Seek>(song: &TgSong, writer: W) -> Result<(), WriteError> {
    let xml_bytes = build_xml(song)?;

    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    let mut zip = ZipWriter::new(writer);

    zip.start_file("version.txt", options)?;
    zip.write_all(b"TuxGuitar_file_format 2.0")?;

    zip.start_file("content.xml", options)?;
    zip.write_all(&xml_bytes)?;

    zip.finish()?;
    Ok(())
}

// =============================================================================
// XML generation
// =============================================================================

type XmlWriter = Writer<Cursor<Vec<u8>>>;

/// Builds the content.xml bytes for the given song.
fn build_xml(song: &TgSong) -> Result<Vec<u8>, std::io::Error> {
    let mut writer = Writer::new(Cursor::new(Vec::new()));

    // XML declaration
    writer.write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))?;

    // Root element
    writer
        .create_element("TuxGuitarFile")
        .write_inner_content(|w| {
            write_version(w)?;
            write_song(w, song)?;
            Ok(())
        })?;

    Ok(writer.into_inner().into_inner())
}

fn write_version(w: &mut XmlWriter) -> std::io::Result<()> {
    w.create_element("TGVersion")
        .with_attribute(("major", "2"))
        .with_attribute(("minor", "1"))
        .with_attribute(("revision", "0"))
        .write_empty()?;
    Ok(())
}

fn write_song(w: &mut XmlWriter, song: &TgSong) -> std::io::Result<()> {
    w.create_element("TGSong").write_inner_content(|w| {
        // Metadata
        write_text_element(w, "name", &song.name)?;
        write_text_element(w, "artist", &song.artist)?;
        write_text_element(w, "album", &song.album)?;
        write_text_element(w, "author", &song.author)?;
        write_text_element(w, "date", &song.date)?;
        write_text_element(w, "copyright", &song.copyright)?;
        write_text_element(w, "writer", &song.writer)?;
        write_text_element(w, "transcriber", &song.transcriber)?;
        write_text_element(w, "comments", &song.comments)?;

        // Channels
        for channel in &song.channels {
            write_channel(w, channel)?;
        }

        // Measure headers
        for header in &song.measure_headers {
            write_measure_header(w, header)?;
        }

        // Tracks
        for track in &song.tracks {
            write_track(w, track)?;
        }

        Ok(())
    })?;
    Ok(())
}

// =============================================================================
// Channel
// =============================================================================

fn write_channel(w: &mut XmlWriter, ch: &TgChannel) -> std::io::Result<()> {
    w.create_element("TGChannel").write_inner_content(|w| {
        write_text_element(w, "id", &ch.channel_id.to_string())?;
        write_text_element(w, "bank", &ch.bank.to_string())?;
        write_text_element(w, "program", &ch.program.to_string())?;
        write_text_element(w, "volume", &ch.volume.to_string())?;
        write_text_element(w, "balance", &ch.balance.to_string())?;
        write_text_element(w, "chorus", &ch.chorus.to_string())?;
        write_text_element(w, "reverb", &ch.reverb.to_string())?;
        write_text_element(w, "phaser", &ch.phaser.to_string())?;
        write_text_element(w, "tremolo", &ch.tremolo.to_string())?;
        write_text_element(w, "name", &ch.name)?;

        for param in &ch.parameters {
            w.create_element("TGChannelParameter")
                .with_attribute(("key", param.key.as_str()))
                .with_attribute(("value", param.value.as_str()))
                .write_empty()?;
        }

        Ok(())
    })?;
    Ok(())
}

// =============================================================================
// Measure Header
// =============================================================================

fn write_measure_header(w: &mut XmlWriter, header: &TgMeasureHeader) -> std::io::Result<()> {
    w.create_element("TGMeasureHeader")
        .write_inner_content(|w| {
            // Time signature
            let num_str = header.time_signature.numerator.to_string();
            let den_str = header.time_signature.denominator.to_string();
            w.create_element("timeSignature")
                .with_attribute(("numerator", num_str.as_str()))
                .with_attribute(("denominator", den_str.as_str()))
                .write_empty()?;

            // Tempo
            write_tempo(w, &header.tempo)?;

            // Repeat open
            if header.repeat_open {
                w.create_element("repeatOpen").write_empty()?;
            }

            // Repeat close
            if header.repeat_close != 0 {
                let val = header.repeat_close.to_string();
                w.create_element("repeatClose")
                    .write_text_content(BytesText::new(&val))?;
            }

            // Repeat alternative
            if header.repeat_alternative != 0 {
                w.create_element("repeatAlternative")
                    .write_inner_content(|w| {
                        // Bitmap: bit N means alternative N+1
                        for bit in 0..8 {
                            if header.repeat_alternative & (1 << bit) != 0 {
                                let alt = (bit + 1).to_string();
                                w.create_element("alternative")
                                    .write_text_content(BytesText::new(&alt))?;
                            }
                        }
                        Ok(())
                    })?;
            }

            // Marker
            if let Some(ref marker) = header.marker {
                let r = marker.color.r.to_string();
                let g = marker.color.g.to_string();
                let b = marker.color.b.to_string();
                w.create_element("marker")
                    .with_attribute(("R", r.as_str()))
                    .with_attribute(("G", g.as_str()))
                    .with_attribute(("B", b.as_str()))
                    .write_text_content(BytesText::new(&marker.title))?;
            }

            // Triplet feel
            if header.triplet_feel != TgTripletFeel::None {
                let value = match header.triplet_feel {
                    TgTripletFeel::None => "none",
                    TgTripletFeel::Eighth => "eighth",
                    TgTripletFeel::Sixteenth => "sixteenth",
                };
                w.create_element("tripletFeel")
                    .write_text_content(BytesText::new(value))?;
            }

            // Line break
            if header.line_break {
                w.create_element("lineBreak").write_empty()?;
            }

            Ok(())
        })?;
    Ok(())
}

fn write_tempo(w: &mut XmlWriter, tempo: &TgTempo) -> std::io::Result<()> {
    let base_str = tempo.base.to_string();
    let value_str = tempo.value.to_string();

    let mut elem = w.create_element("tempo");
    if tempo.base != 4 {
        elem = elem.with_attribute(("base", base_str.as_str()));
    }
    if tempo.dotted {
        elem = elem.with_attribute(("dotted", "true"));
    }

    elem.write_text_content(BytesText::new(&value_str))?;
    Ok(())
}

// =============================================================================
// Track
// =============================================================================

fn write_track(w: &mut XmlWriter, track: &TgTrack) -> std::io::Result<()> {
    let max_fret_str = track.max_fret.map(|v| v.to_string());

    let mut elem = w.create_element("TGTrack");
    if let Some(ref mf) = max_fret_str {
        elem = elem.with_attribute(("maxFret", mf.as_str()));
    }

    elem.write_inner_content(|w| {
        write_text_element(w, "name", &track.name)?;

        // Solo/mute
        if track.solo {
            write_text_element(w, "soloMute", "solo")?;
        }
        else if track.mute {
            write_text_element(w, "soloMute", "mute")?;
        }

        let channel_id_str = track.channel_id.to_string();
        write_text_element(w, "channelId", &channel_id_str)?;

        if track.offset != 0 {
            let offset_str = track.offset.to_string();
            write_text_element(w, "offset", &offset_str)?;
        }

        // Color
        let r = track.color.r.to_string();
        let g = track.color.g.to_string();
        let b = track.color.b.to_string();
        w.create_element("color")
            .with_attribute(("R", r.as_str()))
            .with_attribute(("G", g.as_str()))
            .with_attribute(("B", b.as_str()))
            .write_empty()?;

        // Strings
        for s in &track.strings {
            let val = s.value.to_string();
            w.create_element("TGString")
                .write_text_content(BytesText::new(&val))?;
        }

        // Lyrics
        let from_str = track.lyrics.from.to_string();
        w.create_element("TGLyric")
            .with_attribute(("from", from_str.as_str()))
            .write_text_content(BytesText::new(&track.lyrics.lyrics))?;

        // Measures
        for (i, measure) in track.measures.iter().enumerate() {
            write_measure(w, measure, i)?;
        }

        Ok(())
    })?;
    Ok(())
}

// =============================================================================
// Measure
// =============================================================================

fn write_measure(w: &mut XmlWriter, measure: &TgMeasure, index: usize) -> std::io::Result<()> {
    w.create_element("TGMeasure").write_inner_content(|w| {
        // Always write clef and key signature for every measure.
        // The reader uses these to detect changes from the previous measure.
        // Writing them unconditionally is safe and simpler.
        let _ = index;
        let clef_str = match measure.clef {
            TgClef::Treble => "treble",
            TgClef::Bass => "bass",
            TgClef::Tenor => "tenor",
            TgClef::Alto => "alto",
        };
        write_text_element(w, "clef", clef_str)?;
        write_text_element(w, "keySignature", &measure.key_signature.to_string())?;

        // Beats
        for beat in &measure.beats {
            write_beat(w, beat)?;
        }

        Ok(())
    })?;
    Ok(())
}

// =============================================================================
// Beat
// =============================================================================

fn write_beat(w: &mut XmlWriter, beat: &TgBeat) -> std::io::Result<()> {
    w.create_element("TGBeat").write_inner_content(|w| {
        write_text_element(w, "preciseStart", &beat.precise_start.to_string())?;

        // Stroke
        if let Some(ref stroke) = beat.stroke {
            let dir = match stroke.direction {
                TgStrokeDirection::Up => "up",
                TgStrokeDirection::Down => "down",
            };
            let val = stroke.value.to_string();
            w.create_element("stroke")
                .with_attribute(("direction", dir))
                .with_attribute(("value", val.as_str()))
                .write_empty()?;
        }

        // Pick stroke
        if let Some(ref ps) = beat.pick_stroke {
            let dir = match ps {
                TgPickStrokeDirection::Up => "up",
                TgPickStrokeDirection::Down => "down",
            };
            w.create_element("pickStroke")
                .write_text_content(BytesText::new(dir))?;
        }

        // Chord
        if let Some(ref chord) = beat.chord {
            write_chord(w, chord)?;
        }

        // Text
        if let Some(ref text) = beat.text
            && !text.is_empty()
        {
            write_text_element(w, "text", text)?;
        }

        // Voices
        for voice in &beat.voices {
            write_voice(w, voice)?;
        }

        Ok(())
    })?;
    Ok(())
}

// =============================================================================
// Chord
// =============================================================================

fn write_chord(w: &mut XmlWriter, chord: &TgChord) -> std::io::Result<()> {
    w.create_element("chord").write_inner_content(|w| {
        write_text_element(w, "name", &chord.name)?;
        write_text_element(w, "firstFret", &chord.first_fret.to_string())?;
        for &fret in &chord.strings {
            write_text_element(w, "string", &fret.to_string())?;
        }
        Ok(())
    })?;
    Ok(())
}

// =============================================================================
// Voice
// =============================================================================

fn write_voice(w: &mut XmlWriter, voice: &TgVoice) -> std::io::Result<()> {
    let mut elem = w.create_element("voice");

    match voice.direction {
        TgVoiceDirection::Up => {
            elem = elem.with_attribute(("direction", "up"));
        }
        TgVoiceDirection::Down => {
            elem = elem.with_attribute(("direction", "down"));
        }
        TgVoiceDirection::None => {}
    }

    if voice.notes.is_empty() || voice.empty {
        elem = elem.with_attribute(("empty", "true"));
    }

    elem.write_inner_content(|w| {
        // Duration
        write_duration(w, &voice.duration)?;

        // Notes
        let mut prev_velocity: Option<i32> = None;
        for note in &voice.notes {
            write_note(w, note, prev_velocity)?;
            prev_velocity = Some(note.velocity);
        }

        Ok(())
    })?;
    Ok(())
}

// =============================================================================
// Duration
// =============================================================================

fn write_duration(w: &mut XmlWriter, dur: &TgDuration) -> std::io::Result<()> {
    let value_str = dur.value.to_string();
    let mut elem = w.create_element("duration");
    elem = elem.with_attribute(("value", value_str.as_str()));

    if dur.dotted {
        elem = elem.with_attribute(("dotted", "dotted"));
    }
    else if dur.double_dotted {
        elem = elem.with_attribute(("dotted", "doubleDotted"));
    }

    if let Some(ref div) = dur.division_type {
        let enters_str = div.enters.to_string();
        let times_str = div.times.to_string();
        elem.write_inner_content(|w| {
            w.create_element("divisionType")
                .with_attribute(("enters", enters_str.as_str()))
                .with_attribute(("times", times_str.as_str()))
                .write_empty()?;
            Ok(())
        })?;
    }
    else {
        elem.write_empty()?;
    }

    Ok(())
}

// =============================================================================
// Note
// =============================================================================

fn write_note(w: &mut XmlWriter, note: &TgNote, prev_velocity: Option<i32>) -> std::io::Result<()> {
    let value_str = note.value.to_string();
    let string_str = note.string.to_string();
    let velocity_str = note.velocity.to_string();

    let mut elem = w.create_element("note");
    elem = elem.with_attribute(("value", value_str.as_str()));
    elem = elem.with_attribute(("string", string_str.as_str()));

    // Only write velocity if different from previous note in same voice
    let should_write_velocity = match prev_velocity {
        Some(prev) => note.velocity != prev,
        None => true,
    };
    if should_write_velocity {
        elem = elem.with_attribute(("velocity", velocity_str.as_str()));
    }

    if note.tied {
        elem = elem.with_attribute(("tiedNote", "true"));
    }

    let has_effects = has_any_effect(&note.effect) || note.alt_enharmonic;

    if has_effects {
        elem.write_inner_content(|w| {
            write_note_effects(w, &note.effect)?;
            if note.alt_enharmonic {
                w.create_element("alternativeEnharmonic").write_empty()?;
            }
            Ok(())
        })?;
    }
    else {
        elem.write_empty()?;
    }

    Ok(())
}

fn has_any_effect(effect: &TgNoteEffect) -> bool {
    effect.vibrato
        || effect.dead_note
        || effect.slide
        || effect.hammer
        || effect.ghost_note
        || effect.accentuated
        || effect.heavy_accentuated
        || effect.palm_mute
        || effect.staccato
        || effect.tapping
        || effect.slapping
        || effect.popping
        || effect.fade_in
        || effect.let_ring
        || effect.bend.is_some()
        || effect.tremolo_bar.is_some()
        || effect.harmonic.is_some()
        || effect.grace.is_some()
        || effect.trill.is_some()
        || effect.tremolo_picking.is_some()
}

fn write_note_effects(w: &mut XmlWriter, effect: &TgNoteEffect) -> std::io::Result<()> {
    // Boolean flags as empty elements
    if effect.vibrato {
        w.create_element("vibrato").write_empty()?;
    }
    if effect.dead_note {
        w.create_element("deadNote").write_empty()?;
    }
    if effect.slide {
        w.create_element("slide").write_empty()?;
    }
    if effect.hammer {
        w.create_element("hammer").write_empty()?;
    }
    if effect.ghost_note {
        w.create_element("ghostNote").write_empty()?;
    }
    if effect.accentuated {
        w.create_element("accentuatedNote").write_empty()?;
    }
    if effect.heavy_accentuated {
        w.create_element("heavyAccentuatedNote").write_empty()?;
    }
    if effect.palm_mute {
        w.create_element("palmMute").write_empty()?;
    }
    if effect.staccato {
        w.create_element("staccato").write_empty()?;
    }
    if effect.tapping {
        w.create_element("tapping").write_empty()?;
    }
    if effect.slapping {
        w.create_element("slapping").write_empty()?;
    }
    if effect.popping {
        w.create_element("popping").write_empty()?;
    }
    if effect.fade_in {
        w.create_element("fadeIn").write_empty()?;
    }
    if effect.let_ring {
        w.create_element("letRing").write_empty()?;
    }

    // Bend
    if let Some(ref points) = effect.bend {
        w.create_element("bend").write_inner_content(|w| {
            for point in points {
                let pos = point.position.to_string();
                let val = point.value.to_string();
                w.create_element("point")
                    .with_attribute(("position", pos.as_str()))
                    .with_attribute(("value", val.as_str()))
                    .write_empty()?;
            }
            Ok(())
        })?;
    }

    // Tremolo bar
    if let Some(ref points) = effect.tremolo_bar {
        w.create_element("tremoloBar").write_inner_content(|w| {
            for point in points {
                let pos = point.position.to_string();
                let val = point.value.to_string();
                w.create_element("point")
                    .with_attribute(("position", pos.as_str()))
                    .with_attribute(("value", val.as_str()))
                    .write_empty()?;
            }
            Ok(())
        })?;
    }

    // Harmonic
    if let Some(ref harmonic) = effect.harmonic {
        let type_str = match harmonic.harmonic_type {
            TgHarmonicType::Natural => "N.H",
            TgHarmonicType::Artificial => "A.H",
            TgHarmonicType::Tapped => "T.H",
            TgHarmonicType::Pinch => "P.H",
            TgHarmonicType::Semi => "S.H",
        };
        let data_str = harmonic.data.to_string();
        w.create_element("harmonic")
            .with_attribute(("type", type_str))
            .with_attribute(("data", data_str.as_str()))
            .write_empty()?;
    }

    // Grace note
    if let Some(ref grace) = effect.grace {
        let transition_str = match grace.transition {
            TgGraceTransition::None => "none",
            TgGraceTransition::Slide => "slide",
            TgGraceTransition::Bend => "bend",
            TgGraceTransition::Hammer => "hammer",
        };
        // Map grace duration constant to TGDuration value: 1->64, 2->32, 3->16
        let duration_value = match grace.duration {
            1 => 64,
            2 => 32,
            3 => 16,
            other => other, // fallback: use as-is
        };
        let fret_str = grace.fret.to_string();
        let dur_str = duration_value.to_string();
        let dyn_str = grace.dynamic.to_string();
        w.create_element("grace")
            .with_attribute(("fret", fret_str.as_str()))
            .with_attribute(("duration", dur_str.as_str()))
            .with_attribute(("dynamic", dyn_str.as_str()))
            .with_attribute(("transition", transition_str))
            .with_attribute(("onBeat", if grace.on_beat { "true" } else { "false" }))
            .with_attribute(("dead", if grace.dead { "true" } else { "false" }))
            .write_empty()?;
    }

    // Trill
    if let Some(ref trill) = effect.trill {
        let fret_str = trill.fret.to_string();
        let dur_str = trill.duration.to_string();
        w.create_element("trill")
            .with_attribute(("fret", fret_str.as_str()))
            .with_attribute(("duration", dur_str.as_str()))
            .write_empty()?;
    }

    // Tremolo picking
    if let Some(ref tp) = effect.tremolo_picking {
        let dur_str = tp.duration.to_string();
        w.create_element("tremoloPicking")
            .with_attribute(("duration", dur_str.as_str()))
            .write_empty()?;
    }

    Ok(())
}

// =============================================================================
// Helpers
// =============================================================================

/// Writes a simple element with text content: `<tag>text</tag>`.
fn write_text_element(w: &mut XmlWriter, tag: &str, text: &str) -> std::io::Result<()> {
    w.create_element(tag)
        .write_text_content(BytesText::new(text))?;
    Ok(())
}
