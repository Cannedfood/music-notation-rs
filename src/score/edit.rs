use core::range::Range;

use crate::note::harmony::Pitch;
use crate::note::rhythm::Time;
use crate::note::Note;

#[derive(Debug, Clone, Copy)]
pub struct Cursor {
    pub track: usize,
    pub time_range: Range<Time>,
    pub pitch_range: Range<Pitch>,
}

#[derive(Debug, Clone)]
pub enum Commands {
    Insert(Note),
    Delete,
    MoveTime(i32),
    MovePitch(i32),
}

#[derive(Default, Debug, Clone)]
pub struct EditState {
    pub cursors: Vec<Cursor>,
}
