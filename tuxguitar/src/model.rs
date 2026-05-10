//! TuxGuitar model types representing the data in a `.tg` file.

/// Ticks per quarter note in TuxGuitar's approximate time system.
pub const QUARTER_TIME: i64 = 960;

/// Precise time of the first beat (used internally by TuxGuitar).
pub const PRECISE_STARTING_POINT: i64 = 2_882_880;

/// Number of precise-time fractions in a whole note.
/// Computed as lcm(64, all division-type enters) * 4.
pub const WHOLE_PRECISE_DURATION: i64 = 11_531_520;

// =============================================================================
// Song
// =============================================================================

/// Top-level song structure containing metadata, channels, measure headers, and tracks.
#[derive(Debug, Clone, Default)]
pub struct TgSong {
    pub name: String,
    pub artist: String,
    pub album: String,
    pub author: String,
    pub date: String,
    pub copyright: String,
    pub writer: String,
    pub transcriber: String,
    pub comments: String,
    pub channels: Vec<TgChannel>,
    pub measure_headers: Vec<TgMeasureHeader>,
    pub tracks: Vec<TgTrack>,
}

// =============================================================================
// Channel
// =============================================================================

/// A MIDI channel with instrument and effect settings.
#[derive(Debug, Clone)]
pub struct TgChannel {
    pub channel_id: i32,
    pub bank: i16,
    pub program: i16,
    pub volume: i16,
    pub balance: i16,
    pub chorus: i16,
    pub reverb: i16,
    pub phaser: i16,
    pub tremolo: i16,
    pub name: String,
    pub parameters: Vec<TgChannelParameter>,
}

impl Default for TgChannel {
    fn default() -> Self {
        Self {
            channel_id: 0,
            bank: 0,
            program: 0,
            volume: 127,
            balance: 64,
            chorus: 0,
            reverb: 0,
            phaser: 0,
            tremolo: 0,
            name: String::new(),
            parameters: Vec::new(),
        }
    }
}

/// A key-value parameter attached to a channel.
#[derive(Debug, Clone, Default)]
pub struct TgChannelParameter {
    pub key:   String,
    pub value: String,
}

// =============================================================================
// Measure Header
// =============================================================================

/// Global measure information shared across all tracks (time signature, tempo, repeats, etc.).
#[derive(Debug, Clone)]
pub struct TgMeasureHeader {
    pub number: i32,
    pub start: i64,
    pub time_signature: TgTimeSignature,
    pub tempo: TgTempo,
    pub marker: Option<TgMarker>,
    pub repeat_open: bool,
    pub repeat_close: i32,
    pub repeat_alternative: i32,
    pub triplet_feel: TgTripletFeel,
    pub line_break: bool,
}

impl Default for TgMeasureHeader {
    fn default() -> Self {
        Self {
            number: 1,
            start: QUARTER_TIME,
            time_signature: TgTimeSignature::default(),
            tempo: TgTempo::default(),
            marker: None,
            repeat_open: false,
            repeat_close: 0,
            repeat_alternative: 0,
            triplet_feel: TgTripletFeel::None,
            line_break: false,
        }
    }
}

// =============================================================================
// Time Signature
// =============================================================================

/// A time signature such as 4/4, 3/4, 6/8, etc.
/// The denominator is a duration value (1, 2, 4, 8, 16, 32, 64).
#[derive(Debug, Clone)]
pub struct TgTimeSignature {
    pub numerator:   i32,
    pub denominator: i32,
}

impl Default for TgTimeSignature {
    fn default() -> Self {
        Self {
            numerator:   4,
            denominator: 4,
        }
    }
}

// =============================================================================
// Tempo
// =============================================================================

/// A tempo marking with a base duration and optional dot.
#[derive(Debug, Clone)]
pub struct TgTempo {
    /// Raw BPM value for the given base duration.
    pub value:  i32,
    /// Duration value that the BPM refers to (4 = quarter, 2 = half, etc.).
    pub base:   i32,
    /// Whether the base duration is dotted.
    pub dotted: bool,
}

impl Default for TgTempo {
    fn default() -> Self {
        Self {
            value:  120,
            base:   4,
            dotted: false,
        }
    }
}

impl TgTempo {
    /// Computes the equivalent quarter-note BPM.
    pub fn quarter_value(&self) -> i32 {
        let qv = self.value * 4 / self.base;
        if self.dotted { qv * 3 / 2 } else { qv }
    }
}

// =============================================================================
// Marker
// =============================================================================

/// A rehearsal marker attached to a measure.
#[derive(Debug, Clone)]
pub struct TgMarker {
    pub title: String,
    pub color: TgColor,
}

/// An RGB color.
#[derive(Debug, Clone, Copy, Default)]
pub struct TgColor {
    pub r: i32,
    pub g: i32,
    pub b: i32,
}

// =============================================================================
// Triplet Feel
// =============================================================================

/// The triplet-feel (swing) mode for a measure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TgTripletFeel {
    #[default]
    None,
    Eighth,
    Sixteenth,
}

// =============================================================================
// Track
// =============================================================================

/// A track representing one instrument/part in the song.
#[derive(Debug, Clone)]
pub struct TgTrack {
    pub number: i32,
    pub name: String,
    pub channel_id: i32,
    pub offset: i32,
    pub solo: bool,
    pub mute: bool,
    pub color: TgColor,
    pub strings: Vec<TgString>,
    pub lyrics: TgLyric,
    pub measures: Vec<TgMeasure>,
    pub max_fret: Option<i32>,
}

impl Default for TgTrack {
    fn default() -> Self {
        Self {
            number: 1,
            name: String::new(),
            channel_id: 0,
            offset: 0,
            solo: false,
            mute: false,
            color: TgColor::default(),
            strings: Vec::new(),
            lyrics: TgLyric::default(),
            measures: Vec::new(),
            max_fret: None,
        }
    }
}

/// A string on a fretted instrument, with its tuning.
#[derive(Debug, Clone, Copy, Default)]
pub struct TgString {
    /// String number (1 = highest pitch string).
    pub number: i32,
    /// MIDI pitch of the open string.
    pub value:  i32,
}

/// Lyrics attached to a track, starting at a given measure number.
#[derive(Debug, Clone)]
pub struct TgLyric {
    /// The measure number where lyrics start.
    pub from:   i32,
    /// The lyrics text.
    pub lyrics: String,
}

impl Default for TgLyric {
    fn default() -> Self {
        Self {
            from:   1,
            lyrics: String::new(),
        }
    }
}

// =============================================================================
// Measure
// =============================================================================

/// A single measure within a track containing beats.
#[derive(Debug, Clone, Default)]
pub struct TgMeasure {
    pub clef: TgClef,
    pub key_signature: i32,
    pub beats: Vec<TgBeat>,
}

/// The clef for a measure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TgClef {
    #[default]
    Treble,
    Bass,
    Tenor,
    Alto,
}

// =============================================================================
// Beat
// =============================================================================

/// A beat (a vertical slice of time) containing voices and optional decorations.
#[derive(Debug, Clone)]
pub struct TgBeat {
    pub precise_start: i64,
    pub voices: Vec<TgVoice>,
    pub stroke: Option<TgStroke>,
    pub pick_stroke: Option<TgPickStrokeDirection>,
    pub chord: Option<TgChord>,
    pub text: Option<String>,
}

impl Default for TgBeat {
    fn default() -> Self {
        Self {
            precise_start: PRECISE_STARTING_POINT,
            voices: Vec::new(),
            stroke: None,
            pick_stroke: None,
            chord: None,
            text: None,
        }
    }
}

// =============================================================================
// Voice
// =============================================================================

/// A voice within a beat (TuxGuitar supports multiple voices per beat).
#[derive(Debug, Clone)]
pub struct TgVoice {
    pub duration:  TgDuration,
    pub notes:     Vec<TgNote>,
    pub direction: TgVoiceDirection,
    pub empty:     bool,
}

impl Default for TgVoice {
    fn default() -> Self {
        Self {
            duration:  TgDuration::default(),
            notes:     Vec::new(),
            direction: TgVoiceDirection::None,
            empty:     true,
        }
    }
}

/// The stem direction for a voice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TgVoiceDirection {
    #[default]
    None,
    Up,
    Down,
}

// =============================================================================
// Duration
// =============================================================================

/// A rhythmic duration (e.g. quarter, eighth, dotted half, triplet eighth).
#[derive(Debug, Clone)]
pub struct TgDuration {
    /// Duration value: 1=whole, 2=half, 4=quarter, 8=eighth, 16=sixteenth, 32=thirty-second, 64=sixty-fourth.
    pub value: i32,
    pub dotted: bool,
    pub double_dotted: bool,
    pub division_type: Option<TgDivisionType>,
}

impl Default for TgDuration {
    fn default() -> Self {
        Self {
            value: 4,
            dotted: false,
            double_dotted: false,
            division_type: None,
        }
    }
}

impl TgDuration {
    /// Returns the duration in approximate ticks (where QUARTER_TIME = 960 ticks per quarter note).
    pub fn ticks(&self) -> i64 {
        let mut time = QUARTER_TIME * 4 / self.value as i64;
        if self.dotted {
            time += time / 2;
        }
        else if self.double_dotted {
            time += time * 3 / 4;
        }
        if let Some(ref div) = self.division_type {
            time = time * div.times as i64 / div.enters as i64;
        }
        time
    }
}

/// A tuplet division type (e.g. triplet: enters=3, times=2).
#[derive(Debug, Clone, Copy)]
pub struct TgDivisionType {
    /// Number of notes that fit into the space.
    pub enters: i32,
    /// Number of notes the space would normally hold.
    pub times:  i32,
}

// =============================================================================
// Note
// =============================================================================

/// A single note within a voice.
#[derive(Debug, Clone)]
pub struct TgNote {
    /// Fret number.
    pub value: i32,
    /// String number (1-based).
    pub string: i32,
    /// MIDI velocity (0–127).
    pub velocity: i32,
    /// Whether this note is tied to a previous note.
    pub tied: bool,
    /// Note effects (bends, slides, harmonics, etc.).
    pub effect: TgNoteEffect,
    /// Whether to use an alternate enharmonic spelling.
    pub alt_enharmonic: bool,
}

impl Default for TgNote {
    fn default() -> Self {
        Self {
            value: 0,
            string: 1,
            velocity: 95,
            tied: false,
            effect: TgNoteEffect::default(),
            alt_enharmonic: false,
        }
    }
}

// =============================================================================
// Note Effect
// =============================================================================

/// Effects that can be applied to a note.
#[derive(Debug, Clone, Default)]
pub struct TgNoteEffect {
    pub vibrato: bool,
    pub dead_note: bool,
    pub slide: bool,
    pub hammer: bool,
    pub ghost_note: bool,
    pub accentuated: bool,
    pub heavy_accentuated: bool,
    pub palm_mute: bool,
    pub staccato: bool,
    pub tapping: bool,
    pub slapping: bool,
    pub popping: bool,
    pub fade_in: bool,
    pub let_ring: bool,
    pub bend: Option<Vec<TgBendPoint>>,
    pub tremolo_bar: Option<Vec<TgBendPoint>>,
    pub harmonic: Option<TgHarmonic>,
    pub grace: Option<TgGrace>,
    pub trill: Option<TgTrill>,
    pub tremolo_picking: Option<TgTremoloPicking>,
}

/// A point in a bend or tremolo-bar curve.
#[derive(Debug, Clone, Copy)]
pub struct TgBendPoint {
    /// Position along the note duration (0–12 typically).
    pub position: i32,
    /// Bend amount in quarter-tone units.
    pub value:    i32,
}

// =============================================================================
// Harmonic
// =============================================================================

/// A harmonic effect on a note.
#[derive(Debug, Clone, Copy)]
pub struct TgHarmonic {
    pub harmonic_type: TgHarmonicType,
    pub data: i32,
}

/// The type of harmonic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TgHarmonicType {
    Natural,
    Artificial,
    Tapped,
    Pinch,
    Semi,
}

// =============================================================================
// Grace Note
// =============================================================================

/// A grace note preceding the main note.
#[derive(Debug, Clone, Copy)]
pub struct TgGrace {
    pub fret: i32,
    pub duration: i32,
    pub dynamic: i32,
    pub transition: TgGraceTransition,
    pub on_beat: bool,
    pub dead: bool,
}

/// The transition type from a grace note to the main note.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TgGraceTransition {
    #[default]
    None,
    Slide,
    Bend,
    Hammer,
}

// =============================================================================
// Trill & Tremolo Picking
// =============================================================================

/// A trill effect (rapid alternation between two frets).
#[derive(Debug, Clone, Copy)]
pub struct TgTrill {
    pub fret:     i32,
    /// Duration value (4=quarter, 8=eighth, 16=sixteenth).
    pub duration: i32,
}

/// A tremolo-picking effect (rapid repetition of a note).
#[derive(Debug, Clone, Copy)]
pub struct TgTremoloPicking {
    /// Duration value (8=eighth, 16=sixteenth, 32=thirty-second).
    pub duration: i32,
}

// =============================================================================
// Stroke
// =============================================================================

/// A strum stroke across the strings of a beat.
#[derive(Debug, Clone, Copy)]
pub struct TgStroke {
    pub direction: TgStrokeDirection,
    /// Duration value controlling stroke speed.
    pub value:     i32,
}

/// Direction of a strum stroke.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TgStrokeDirection {
    Up,
    Down,
}

/// Direction of a pick stroke.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TgPickStrokeDirection {
    Up,
    Down,
}

// =============================================================================
// Chord
// =============================================================================

/// A chord diagram attached to a beat.
#[derive(Debug, Clone)]
pub struct TgChord {
    pub name: String,
    pub first_fret: i32,
    /// Fret values per string (-1 means the string is not played).
    pub strings: Vec<i32>,
}
