# Music Notation

A library for music theory and notation.
It also provides utilities for rendering and editing scores.
Eventually, this is meant to be the basis for a music composition program.

Expect this crate to be split up eventually.

## Features

Music Theory Primitives

- `harmony`
  - `Interval` - And interval like an octave
  - `Pitch` - A specific pitch like C#3
  - `Chroma` - the "letter" of the note (A, C# etc.)
  - TODO: `Chord`, `KeySignature`, `Scale`
- `rhythm`
  - `Time` - An absolute point in time
  - `Duration` - The duration e.g. "half note"
  - `TimeSignature` - A time signature like 4/4 or 7/8
- `articulation`
  - `Finger`
  - `Hand`
  - `Velocity`
  - TODO: `Bend`, `AfterTouch`, `Location`(?), `Dampening`

Score and editing

- Midi Import (via. `midly`), TODO: Export
- Utilities for rendering the score
  - `MidiRoll`
  - TODO: `Tabs`, `StandardNotation`
