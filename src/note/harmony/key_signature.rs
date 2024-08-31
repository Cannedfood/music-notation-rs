#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KeySignature {
    pub flats_sharps: i8,
    pub major: bool,
}

impl KeySignature {
    pub fn from_midi(flats_sharps: i8, major: bool) -> Self {
        KeySignature {
            flats_sharps,
            major,
        }
    }
}
