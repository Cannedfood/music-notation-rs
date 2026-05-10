//! Reader for TuxGuitar `.tg` files (ZIP archive containing version.txt and content.xml).

use std::io::{Read, Seek};

use roxmltree::{Document, Node};
use zip::ZipArchive;

use crate::model::*;

// =============================================================================
// Error type
// =============================================================================

/// Errors that can occur when reading a `.tg` file.
#[derive(Debug, Clone)]
pub enum ReadError {
    Zip(String),
    Xml(roxmltree::Error),
    InvalidFormat(String),
    Io(String),
}

impl From<zip::result::ZipError> for ReadError {
    fn from(e: zip::result::ZipError) -> Self { ReadError::Zip(e.to_string()) }
}

impl From<roxmltree::Error> for ReadError {
    fn from(e: roxmltree::Error) -> Self { ReadError::Xml(e) }
}

impl From<std::io::Error> for ReadError {
    fn from(e: std::io::Error) -> Self { ReadError::Io(e.to_string()) }
}

impl std::fmt::Display for ReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReadError::Zip(e) => write!(f, "ZIP error: {e}"),
            ReadError::Xml(e) => write!(f, "XML error: {e}"),
            ReadError::InvalidFormat(msg) => write!(f, "Invalid format: {msg}"),
            ReadError::Io(e) => write!(f, "IO error: {e}"),
        }
    }
}

impl std::error::Error for ReadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ReadError::Zip(_) => None,
            ReadError::Xml(e) => Some(e),
            ReadError::InvalidFormat(_) => None,
            ReadError::Io(_) => None,
        }
    }
}

// =============================================================================
// Public API
// =============================================================================

/// Reads a TuxGuitar `.tg` file and returns the parsed song.
///
/// The input must be a ZIP archive containing:
/// - `version.txt`: format version string (e.g. "TuxGuitar_file_format 2.0")
/// - `content.xml`: full song data as XML
pub fn read_tg<R: Read + Seek>(reader: R) -> Result<TgSong, ReadError> {
    let mut archive = ZipArchive::new(reader)?;

    // Read and validate version.txt
    {
        let mut version_file = archive.by_name("version.txt")?;
        let mut version_str = String::new();
        version_file.read_to_string(&mut version_str)?;
        let version_str = version_str.trim();
        if !version_str.starts_with("TuxGuitar_file_format") {
            return Err(ReadError::InvalidFormat(format!(
                "Unexpected version string: {version_str}"
            )));
        }
    }

    // Read content.xml
    let xml_string = {
        let mut content_file = archive.by_name("content.xml")?;
        let mut xml = String::new();
        content_file.read_to_string(&mut xml)?;
        xml
    };

    // Parse XML
    let doc = Document::parse(&xml_string)?;
    let root = doc.root_element();

    if root.tag_name().name() != "TuxGuitarFile" {
        return Err(ReadError::InvalidFormat(format!(
            "Expected root element 'TuxGuitarFile', found '{}'",
            root.tag_name().name()
        )));
    }

    let song_node = child_elem(root, "TGSong")
        .ok_or_else(|| ReadError::InvalidFormat("Missing TGSong element".to_string()))?;

    parse_song(song_node)
}

// =============================================================================
// XML Helper Functions
// =============================================================================

/// Returns the first child element with the given tag name.
fn child_elem<'a>(node: Node<'a, '_>, name: &str) -> Option<Node<'a, 'a>> {
    node.children()
        .find(|n| n.is_element() && n.tag_name().name() == name)
}

/// Returns the text content of the first child element with the given tag name.
fn child_text<'a>(node: Node<'a, '_>, name: &str) -> Option<&'a str> {
    child_elem(node, name).and_then(|n| n.text())
}

/// Returns an iterator over all child elements with the given tag name.
#[allow(dead_code)]
fn children_iter<'a>(node: Node<'a, 'a>, name: &'a str) -> impl Iterator<Item = Node<'a, 'a>> + 'a {
    node.children()
        .filter(move |n| n.is_element() && n.tag_name().name() == name)
}

/// Returns an attribute value.
fn attr<'a>(node: Node<'a, '_>, name: &str) -> Option<&'a str> { node.attribute(name) }

/// Parses an attribute as i32.
fn attr_i32(node: Node<'_, '_>, name: &str) -> Result<i32, ReadError> {
    let value = node.attribute(name).ok_or_else(|| {
        ReadError::InvalidFormat(format!(
            "Missing attribute '{}' on element '{}'",
            name,
            node.tag_name().name()
        ))
    })?;
    value.parse::<i32>().map_err(|_| {
        ReadError::InvalidFormat(format!(
            "Invalid integer '{}' for attribute '{}' on element '{}'",
            value,
            name,
            node.tag_name().name()
        ))
    })
}

/// Parses an attribute as i64.
#[allow(dead_code)]
fn attr_i64(node: Node<'_, '_>, name: &str) -> Result<i64, ReadError> {
    let value = node.attribute(name).ok_or_else(|| {
        ReadError::InvalidFormat(format!(
            "Missing attribute '{}' on element '{}'",
            name,
            node.tag_name().name()
        ))
    })?;
    value.parse::<i64>().map_err(|_| {
        ReadError::InvalidFormat(format!(
            "Invalid integer '{}' for attribute '{}' on element '{}'",
            value,
            name,
            node.tag_name().name()
        ))
    })
}

/// Returns the text content of a node, or empty string if none.
fn text_content<'a>(node: Node<'a, '_>) -> &'a str { node.text().unwrap_or("") }

/// Parses text content as i32.
fn text_i32(node: Node<'_, '_>) -> Result<i32, ReadError> {
    let text = text_content(node).trim();
    text.parse::<i32>().map_err(|_| {
        ReadError::InvalidFormat(format!(
            "Invalid integer '{}' in element '{}'",
            text,
            node.tag_name().name()
        ))
    })
}

/// Parses text content as i64.
fn text_i64(node: Node<'_, '_>) -> Result<i64, ReadError> {
    let text = text_content(node).trim();
    text.parse::<i64>().map_err(|_| {
        ReadError::InvalidFormat(format!(
            "Invalid integer '{}' in element '{}'",
            text,
            node.tag_name().name()
        ))
    })
}

/// Parses text content of a named child element as i32, returning a default if absent.
fn child_i32(node: Node<'_, '_>, name: &str, default: i32) -> Result<i32, ReadError> {
    match child_elem(node, name) {
        Some(n) => text_i32(n),
        None => Ok(default),
    }
}

/// Parses text content of a named child element as i16, returning a default if absent.
fn child_i16(node: Node<'_, '_>, name: &str, default: i16) -> Result<i16, ReadError> {
    match child_elem(node, name) {
        Some(n) => {
            let text = text_content(n).trim();
            text.parse::<i16>().map_err(|_| {
                ReadError::InvalidFormat(format!(
                    "Invalid integer '{}' in element '{}'",
                    text, name
                ))
            })
        }
        None => Ok(default),
    }
}

// =============================================================================
// Song Parsing
// =============================================================================

fn parse_song(node: Node<'_, '_>) -> Result<TgSong, ReadError> {
    let mut song = TgSong {
        name: child_text(node, "name").unwrap_or("").to_string(),
        artist: child_text(node, "artist").unwrap_or("").to_string(),
        album: child_text(node, "album").unwrap_or("").to_string(),
        author: child_text(node, "author").unwrap_or("").to_string(),
        date: child_text(node, "date").unwrap_or("").to_string(),
        copyright: child_text(node, "copyright").unwrap_or("").to_string(),
        writer: child_text(node, "writer").unwrap_or("").to_string(),
        transcriber: child_text(node, "transcriber").unwrap_or("").to_string(),
        comments: child_text(node, "comments").unwrap_or("").to_string(),
        ..Default::default()
    };

    // Parse channels
    for ch_node in node
        .children()
        .filter(|n| n.is_element() && n.tag_name().name() == "TGChannel")
    {
        song.channels.push(parse_channel(ch_node)?);
    }

    // Parse measure headers with sticky values
    let mut prev_time_sig = TgTimeSignature::default();
    let mut prev_tempo = TgTempo::default();
    let mut current_start: i64 = QUARTER_TIME;

    for (idx, header_node) in node
        .children()
        .filter(|n| n.is_element() && n.tag_name().name() == "TGMeasureHeader")
        .enumerate()
    {
        let header =
            parse_measure_header(header_node, idx, current_start, &prev_time_sig, &prev_tempo)?;

        // Compute next start: current start + measure length
        let measure_length = header.time_signature.numerator as i64 * QUARTER_TIME * 4
            / header.time_signature.denominator as i64;
        current_start = header.start + measure_length;

        // Update sticky values
        prev_time_sig = header.time_signature.clone();
        prev_tempo = header.tempo.clone();

        song.measure_headers.push(header);
    }

    // Parse tracks
    for (idx, track_node) in node
        .children()
        .filter(|n| n.is_element() && n.tag_name().name() == "TGTrack")
        .enumerate()
    {
        song.tracks.push(parse_track(track_node, idx)?);
    }

    Ok(song)
}

// =============================================================================
// Channel Parsing
// =============================================================================

fn parse_channel(node: Node<'_, '_>) -> Result<TgChannel, ReadError> {
    let mut ch = TgChannel {
        channel_id: child_i32(node, "id", 0)?,
        bank: child_i16(node, "bank", 0)?,
        program: child_i16(node, "program", 0)?,
        volume: child_i16(node, "volume", 127)?,
        balance: child_i16(node, "balance", 64)?,
        chorus: child_i16(node, "chorus", 0)?,
        reverb: child_i16(node, "reverb", 0)?,
        phaser: child_i16(node, "phaser", 0)?,
        tremolo: child_i16(node, "tremolo", 0)?,
        name: child_text(node, "name").unwrap_or("").to_string(),
        ..Default::default()
    };

    for param_node in node
        .children()
        .filter(|n| n.is_element() && n.tag_name().name() == "TGChannelParameter")
    {
        let key = attr(param_node, "key").unwrap_or("").to_string();
        let value = attr(param_node, "value").unwrap_or("").to_string();
        ch.parameters.push(TgChannelParameter { key, value });
    }

    Ok(ch)
}

// =============================================================================
// Measure Header Parsing
// =============================================================================

fn parse_measure_header(
    node: Node<'_, '_>,
    index: usize,
    start: i64,
    prev_time_sig: &TgTimeSignature,
    prev_tempo: &TgTempo,
) -> Result<TgMeasureHeader, ReadError> {
    let mut header = TgMeasureHeader {
        number: (index + 1) as i32,
        start,
        time_signature: prev_time_sig.clone(),
        tempo: prev_tempo.clone(),
        marker: None,
        repeat_open: false,
        repeat_close: 0,
        repeat_alternative: 0,
        triplet_feel: TgTripletFeel::None,
        line_break: false,
    };

    // Time signature (sticky: only update if present)
    if let Some(ts_node) = child_elem(node, "timeSignature") {
        header.time_signature.numerator = attr_i32(ts_node, "numerator")?;
        header.time_signature.denominator = attr_i32(ts_node, "denominator")?;
    }

    // Tempo (sticky: only update if present)
    if let Some(tempo_node) = child_elem(node, "tempo") {
        let value = text_i32(tempo_node)?;
        let base = match attr(tempo_node, "base") {
            Some(b) => b
                .parse::<i32>()
                .map_err(|_| ReadError::InvalidFormat(format!("Invalid tempo base: {b}")))?,
            None => 4,
        };
        let dotted = attr(tempo_node, "dotted")
            .map(|v| v == "true")
            .unwrap_or(false);
        header.tempo = TgTempo {
            value,
            base,
            dotted,
        };
    }

    // Repeat open
    header.repeat_open = child_elem(node, "repeatOpen").is_some();

    // Repeat close
    if let Some(rc_node) = child_elem(node, "repeatClose") {
        header.repeat_close = text_i32(rc_node)?;
    }

    // Repeat alternative
    if let Some(ra_node) = child_elem(node, "repeatAlternative") {
        let mut bitmap: i32 = 0;
        for alt_node in ra_node
            .children()
            .filter(|n| n.is_element() && n.tag_name().name() == "alternative")
        {
            let alt_num = text_i32(alt_node)?;
            if (1..=32).contains(&alt_num) {
                bitmap |= 1 << (alt_num - 1);
            }
        }
        header.repeat_alternative = bitmap;
    }

    // Marker
    if let Some(marker_node) = child_elem(node, "marker") {
        let title = text_content(marker_node).to_string();
        let r = attr_i32(marker_node, "R").unwrap_or(0);
        let g = attr_i32(marker_node, "G").unwrap_or(0);
        let b = attr_i32(marker_node, "B").unwrap_or(0);
        header.marker = Some(TgMarker {
            title,
            color: TgColor { r, g, b },
        });
    }

    // Triplet feel
    if let Some(tf_node) = child_elem(node, "tripletFeel") {
        let tf_text = text_content(tf_node).trim();
        header.triplet_feel = match tf_text {
            "eighth" => TgTripletFeel::Eighth,
            "sixteenth" => TgTripletFeel::Sixteenth,
            _ => TgTripletFeel::None,
        };
    }

    // Line break
    header.line_break = child_elem(node, "lineBreak").is_some();

    Ok(header)
}

// =============================================================================
// Track Parsing
// =============================================================================

fn parse_track(node: Node<'_, '_>, index: usize) -> Result<TgTrack, ReadError> {
    let mut track = TgTrack {
        number: (index + 1) as i32,
        ..TgTrack::default()
    };

    track.max_fret = match attr(node, "maxFret") {
        Some(v) => Some(
            v.parse::<i32>()
                .map_err(|_| ReadError::InvalidFormat(format!("Invalid maxFret value: {v}")))?,
        ),
        None => None,
    };

    track.name = child_text(node, "name").unwrap_or("").to_string();

    // Solo/mute
    if let Some(sm_text) = child_text(node, "soloMute") {
        match sm_text.trim() {
            "solo" => track.solo = true,
            "mute" => track.mute = true,
            _ => {}
        }
    }

    // Channel ID
    if let Some(ch_text) = child_text(node, "channelId") {
        track.channel_id = ch_text.trim().parse::<i32>().unwrap_or(0);
    }

    // Offset
    if let Some(offset_text) = child_text(node, "offset") {
        track.offset = offset_text.trim().parse::<i32>().unwrap_or(0);
    }

    // Color
    if let Some(color_node) = child_elem(node, "color") {
        track.color = TgColor {
            r: attr_i32(color_node, "R").unwrap_or(0),
            g: attr_i32(color_node, "G").unwrap_or(0),
            b: attr_i32(color_node, "B").unwrap_or(0),
        };
    }

    // Strings
    for string_node in node
        .children()
        .filter(|n| n.is_element() && n.tag_name().name() == "TGString")
    {
        let value = text_i32(string_node)?;
        let number = (track.strings.len() + 1) as i32;
        track.strings.push(TgString { number, value });
    }

    // Lyrics
    if let Some(lyric_node) = child_elem(node, "TGLyric") {
        let from = match attr(lyric_node, "from") {
            Some(v) => v.parse::<i32>().unwrap_or(1),
            None => 1,
        };
        let lyrics = text_content(lyric_node).to_string();
        track.lyrics = TgLyric { from, lyrics };
    }

    // Measures with sticky clef and key signature
    let mut prev_clef = TgClef::Treble;
    let mut prev_key_signature: i32 = 0;

    for measure_node in node
        .children()
        .filter(|n| n.is_element() && n.tag_name().name() == "TGMeasure")
    {
        let measure = parse_measure(measure_node, &prev_clef, prev_key_signature)?;
        prev_clef = measure.clef;
        prev_key_signature = measure.key_signature;
        track.measures.push(measure);
    }

    Ok(track)
}

// =============================================================================
// Measure Parsing
// =============================================================================

fn parse_measure(
    node: Node<'_, '_>,
    prev_clef: &TgClef,
    prev_key_signature: i32,
) -> Result<TgMeasure, ReadError> {
    let mut measure = TgMeasure {
        clef: *prev_clef,
        key_signature: prev_key_signature,
        beats: Vec::new(),
    };

    // Clef (sticky)
    if let Some(clef_text) = child_text(node, "clef") {
        measure.clef = match clef_text.trim() {
            "treble" => TgClef::Treble,
            "bass" => TgClef::Bass,
            "tenor" => TgClef::Tenor,
            "alto" => TgClef::Alto,
            _ => *prev_clef,
        };
    }

    // Key signature (sticky)
    if let Some(ks_text) = child_text(node, "keySignature") {
        measure.key_signature = ks_text.trim().parse::<i32>().unwrap_or(prev_key_signature);
    }

    // Beats
    for beat_node in node
        .children()
        .filter(|n| n.is_element() && n.tag_name().name() == "TGBeat")
    {
        measure.beats.push(parse_beat(beat_node)?);
    }

    Ok(measure)
}

// =============================================================================
// Beat Parsing
// =============================================================================

fn parse_beat(node: Node<'_, '_>) -> Result<TgBeat, ReadError> {
    let mut beat = TgBeat::default();

    // Precise start
    if let Some(ps_node) = child_elem(node, "preciseStart") {
        beat.precise_start = text_i64(ps_node)?;
    }

    // Stroke
    if let Some(stroke_node) = child_elem(node, "stroke") {
        let direction = match attr(stroke_node, "direction") {
            Some("up") => TgStrokeDirection::Up,
            Some("down") => TgStrokeDirection::Down,
            _ => TgStrokeDirection::Down,
        };
        let value = attr_i32(stroke_node, "value")?;
        beat.stroke = Some(TgStroke { direction, value });
    }

    // Pick stroke
    if let Some(ps_node) = child_elem(node, "pickStroke") {
        let dir_text = text_content(ps_node).trim();
        beat.pick_stroke = match dir_text {
            "up" => Some(TgPickStrokeDirection::Up),
            "down" => Some(TgPickStrokeDirection::Down),
            _ => None,
        };
    }

    // Chord
    if let Some(chord_node) = child_elem(node, "chord") {
        beat.chord = Some(parse_chord(chord_node)?);
    }

    // Text
    if let Some(text_val) = child_text(node, "text")
        && !text_val.is_empty()
    {
        beat.text = Some(text_val.to_string());
    }

    // Voices
    for voice_node in node
        .children()
        .filter(|n| n.is_element() && n.tag_name().name() == "voice")
    {
        beat.voices.push(parse_voice(voice_node)?);
    }

    Ok(beat)
}

// =============================================================================
// Chord Parsing
// =============================================================================

fn parse_chord(node: Node<'_, '_>) -> Result<TgChord, ReadError> {
    let name = child_text(node, "name").unwrap_or("").to_string();
    let first_fret = child_i32(node, "firstFret", 0)?;

    let mut strings = Vec::new();
    for string_node in node
        .children()
        .filter(|n| n.is_element() && n.tag_name().name() == "string")
    {
        let fret = text_i32(string_node)?;
        strings.push(fret);
    }

    Ok(TgChord {
        name,
        first_fret,
        strings,
    })
}

// =============================================================================
// Voice Parsing
// =============================================================================

fn parse_voice(node: Node<'_, '_>) -> Result<TgVoice, ReadError> {
    let mut voice = TgVoice::default();

    // Direction
    if let Some(dir) = attr(node, "direction") {
        voice.direction = match dir {
            "up" => TgVoiceDirection::Up,
            "down" => TgVoiceDirection::Down,
            _ => TgVoiceDirection::None,
        };
    }

    // Empty attribute
    let empty_attr = attr(node, "empty").map(|v| v == "true").unwrap_or(false);

    // Duration
    if let Some(dur_node) = child_elem(node, "duration") {
        voice.duration = parse_duration(dur_node)?;
    }

    // Notes with sticky velocity
    let mut prev_velocity: i32 = 95;
    for note_node in node
        .children()
        .filter(|n| n.is_element() && n.tag_name().name() == "note")
    {
        let note = parse_note(note_node, prev_velocity)?;
        prev_velocity = note.velocity;
        voice.notes.push(note);
    }

    // Determine empty status
    voice.empty = if voice.notes.is_empty() {
        empty_attr
    }
    else {
        false
    };

    Ok(voice)
}

// =============================================================================
// Duration Parsing
// =============================================================================

fn parse_duration(node: Node<'_, '_>) -> Result<TgDuration, ReadError> {
    let mut dur = TgDuration {
        value: attr_i32(node, "value")?,
        ..Default::default()
    };

    // Dotted attribute: "dotted" or "doubleDotted"
    if let Some(dotted_val) = attr(node, "dotted") {
        match dotted_val {
            "dotted" => dur.dotted = true,
            "doubleDotted" => dur.double_dotted = true,
            _ => {}
        }
    }

    // Division type
    if let Some(div_node) = child_elem(node, "divisionType") {
        let enters = attr_i32(div_node, "enters")?;
        let times = attr_i32(div_node, "times")?;
        dur.division_type = Some(TgDivisionType { enters, times });
    }

    Ok(dur)
}

// =============================================================================
// Note Parsing
// =============================================================================

fn parse_note(node: Node<'_, '_>, prev_velocity: i32) -> Result<TgNote, ReadError> {
    let velocity = match attr(node, "velocity") {
        Some(v) => v
            .parse::<i32>()
            .map_err(|_| ReadError::InvalidFormat(format!("Invalid velocity value: {v}")))?,
        None => prev_velocity,
    };

    let note = TgNote {
        value: attr_i32(node, "value")?,
        string: attr_i32(node, "string")?,
        velocity,
        tied: attr(node, "tiedNote").map(|v| v == "true").unwrap_or(false),
        effect: parse_note_effects(node)?,
        alt_enharmonic: child_elem(node, "alternativeEnharmonic").is_some(),
    };

    Ok(note)
}

// =============================================================================
// Note Effects Parsing
// =============================================================================

fn parse_note_effects(node: Node<'_, '_>) -> Result<TgNoteEffect, ReadError> {
    let mut effect = TgNoteEffect {
        vibrato: child_elem(node, "vibrato").is_some(),
        dead_note: child_elem(node, "deadNote").is_some(),
        slide: child_elem(node, "slide").is_some(),
        hammer: child_elem(node, "hammer").is_some(),
        ghost_note: child_elem(node, "ghostNote").is_some(),
        accentuated: child_elem(node, "accentuatedNote").is_some(),
        heavy_accentuated: child_elem(node, "heavyAccentuatedNote").is_some(),
        palm_mute: child_elem(node, "palmMute").is_some(),
        staccato: child_elem(node, "staccato").is_some(),
        tapping: child_elem(node, "tapping").is_some(),
        slapping: child_elem(node, "slapping").is_some(),
        popping: child_elem(node, "popping").is_some(),
        fade_in: child_elem(node, "fadeIn").is_some(),
        let_ring: child_elem(node, "letRing").is_some(),
        ..Default::default()
    };

    // Bend
    if let Some(bend_node) = child_elem(node, "bend") {
        let points = parse_bend_points(bend_node)?;
        effect.bend = Some(points);
    }

    // Tremolo bar
    if let Some(tb_node) = child_elem(node, "tremoloBar") {
        let points = parse_bend_points(tb_node)?;
        effect.tremolo_bar = Some(points);
    }

    // Harmonic
    if let Some(harm_node) = child_elem(node, "harmonic") {
        let type_str = attr(harm_node, "type").unwrap_or("N.H");
        let harmonic_type = match type_str {
            "N.H" => TgHarmonicType::Natural,
            "A.H" => TgHarmonicType::Artificial,
            "T.H" => TgHarmonicType::Tapped,
            "P.H" => TgHarmonicType::Pinch,
            "S.H" => TgHarmonicType::Semi,
            other => {
                return Err(ReadError::InvalidFormat(format!(
                    "Unknown harmonic type: {other}"
                )));
            }
        };
        let data = attr_i32(harm_node, "data").unwrap_or(0);
        effect.harmonic = Some(TgHarmonic {
            harmonic_type,
            data,
        });
    }

    // Grace note
    if let Some(grace_node) = child_elem(node, "grace") {
        let fret = attr_i32(grace_node, "fret")?;
        let raw_duration = attr_i32(grace_node, "duration")?;
        let dynamic = attr_i32(grace_node, "dynamic")?;
        let transition_str = attr(grace_node, "transition").unwrap_or("none");
        let on_beat = attr(grace_node, "onBeat")
            .map(|v| v == "true")
            .unwrap_or(false);
        let dead = attr(grace_node, "dead")
            .map(|v| v == "true")
            .unwrap_or(false);

        // Map TGDuration value to grace duration constant: 64->1, 32->2, 16->3
        let duration = match raw_duration {
            64 => 1,
            32 => 2,
            16 => 3,
            other => other, // fallback: use as-is
        };

        let transition = match transition_str {
            "none" => TgGraceTransition::None,
            "slide" => TgGraceTransition::Slide,
            "bend" => TgGraceTransition::Bend,
            "hammer" => TgGraceTransition::Hammer,
            _ => TgGraceTransition::None,
        };

        effect.grace = Some(TgGrace {
            fret,
            duration,
            dynamic,
            transition,
            on_beat,
            dead,
        });
    }

    // Trill
    if let Some(trill_node) = child_elem(node, "trill") {
        let fret = attr_i32(trill_node, "fret")?;
        let duration = attr_i32(trill_node, "duration")?;
        effect.trill = Some(TgTrill { fret, duration });
    }

    // Tremolo picking
    if let Some(tp_node) = child_elem(node, "tremoloPicking") {
        let duration = attr_i32(tp_node, "duration")?;
        effect.tremolo_picking = Some(TgTremoloPicking { duration });
    }

    Ok(effect)
}

// =============================================================================
// Bend Points Parsing
// =============================================================================

fn parse_bend_points(node: Node<'_, '_>) -> Result<Vec<TgBendPoint>, ReadError> {
    let mut points = Vec::new();
    for point_node in node
        .children()
        .filter(|n| n.is_element() && n.tag_name().name() == "point")
    {
        let position = attr_i32(point_node, "position")?;
        let value = attr_i32(point_node, "value")?;
        points.push(TgBendPoint { position, value });
    }
    Ok(points)
}
