use core::range::Range;

use crate::note::harmony::Pitch;
use crate::note::rhythm::Time;

#[derive(Debug, Clone, Copy)]
pub struct Cursor {
    pub track: usize,
    pub time_range: Range<Time>,
    pub pitch_range: Range<Pitch>,
}

#[derive(Default, Debug, Clone)]
pub struct EditState {
    pub cursors: Vec<Cursor>,
}
