use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Chroma {
    /// Needs to be -1 to ensure that Cb4 == B3.
    CFlat = -1,
    C     = 0,
    CSharp = 1,
    D     = 2,
    DSharp = 3,
    E     = 4,
    F     = 5,
    FSharp = 6,
    G     = 7,
    GSharp = 8,
    A     = 9,
    ASharp = 10,
    B     = 11,
}

impl Display for Chroma {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Chroma::CFlat => "Cb",
            Chroma::C => "C",
            Chroma::CSharp => "C#",
            Chroma::D => "D",
            Chroma::DSharp => "D#",
            Chroma::E => "E",
            Chroma::F => "F",
            Chroma::FSharp => "F#",
            Chroma::G => "G",
            Chroma::GSharp => "G#",
            Chroma::A => "A",
            Chroma::ASharp => "A#",
            Chroma::B => "B",
        })
    }
}

#[allow(non_upper_case_globals)]
impl Chroma {
    pub const DFlat: Chroma = Chroma::CSharp;
    pub const EFlat: Chroma = Chroma::DSharp;
    pub const GFlat: Chroma = Chroma::FSharp;
    pub const AFlat: Chroma = Chroma::GSharp;
    pub const BFlat: Chroma = Chroma::ASharp;
}

impl Chroma {
    // Returns a number between -1 and 11, the -1 Cb and ensures that Cb4 == B3.
    pub fn to_midi_chroma(self) -> i8 {
        match self {
            Chroma::CFlat => -1,
            Chroma::C => 0,
            Chroma::CSharp => 1,
            Chroma::D => 2,
            Chroma::DSharp => 3,
            Chroma::E => 4,
            Chroma::F => 5,
            Chroma::FSharp => 6,
            Chroma::G => 7,
            Chroma::GSharp => 8,
            Chroma::A => 9,
            Chroma::ASharp => 10,
            Chroma::B => 11,
        }
    }
    /// Expects a number between 0 and 11. If the number is out of bounds, returns None.
    pub fn from_midi_chroma(midi: u8) -> Option<Self> {
        match midi {
            0 => Some(Chroma::C),
            1 => Some(Chroma::CSharp),
            2 => Some(Chroma::D),
            3 => Some(Chroma::DSharp),
            4 => Some(Chroma::E),
            5 => Some(Chroma::F),
            6 => Some(Chroma::FSharp),
            7 => Some(Chroma::G),
            8 => Some(Chroma::GSharp),
            9 => Some(Chroma::A),
            10 => Some(Chroma::ASharp),
            11 => Some(Chroma::B),
            _ => None,
        }
    }
}
