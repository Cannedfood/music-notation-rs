//! Parsing and display for keyboard/mouse/touch action triggers.
//!
//! A trigger is a small DSL describing one or more input gestures that can
//! activate an action. The grammar is:
//!
//! ```text
//! trigger := combo ( ';' combo )*       // alternatives ("any of")
//! combo   := atom  ( '+' atom  )*       // simultaneous inputs / modifiers
//! atom    := IDENT ( '(' trigger ')' )? // bare token or wrapper like
//!                                       // drag(...), click(...), double(...)
//! IDENT   := [A-Za-z_][A-Za-z0-9_]*
//! ```
//!
//! Whitespace around any token is ignored. Examples:
//!
//! - `"left+ctrl+shift"`
//! - `"scroll_y; touch_zoom_x"`
//! - `"click(double(mouse_left))"`
//! - `"drag(mouse_left+alt)"`

use std::fmt;
use std::str::FromStr;

/// A parsed input trigger: a list of alternative gestures.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ActionTrigger {
    /// Alternatives separated by `;` in the source. Any one of these
    /// matching is enough to fire the action.
    pub alternatives: Vec<Combo>,
}

/// A combination of atoms that must occur together (joined by `+`).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Combo {
    pub atoms: Vec<Atom>,
}

/// A single input token, optionally wrapping another trigger
/// (e.g. `drag(...)`, `click(...)`, `double(...)`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Atom {
    /// A bare named input, e.g. `left`, `ctrl`, `mouse_left`, `scroll_y`.
    Token(String),
    /// A named wrapper applied to an inner trigger, e.g. `drag(mouse_left+alt)`.
    Call {
        name:  String,
        inner: Box<ActionTrigger>,
    },
}

/// Error produced when parsing an [`ActionTrigger`] fails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseTriggerError {
    pub message:  String,
    pub position: usize,
}

impl fmt::Display for ParseTriggerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "invalid action trigger at byte {}: {}",
            self.position, self.message
        )
    }
}

impl std::error::Error for ParseTriggerError {}

/// Convenience constructor used throughout the action map.
///
/// Panics if `s` is not a valid trigger expression. Intended for static,
/// developer-authored defaults; for user input use [`str::parse`] instead.
pub fn trigger(s: &str) -> ActionTrigger { ActionTrigger::parse(s) }

impl ActionTrigger {
    /// Parse a trigger expression, panicking on malformed input.
    ///
    /// Use [`ActionTrigger::try_parse`] (or [`str::parse`]) to handle errors.
    pub fn parse(s: &str) -> ActionTrigger {
        match Self::try_parse(s) {
            Ok(t) => t,
            Err(e) => panic!("invalid action trigger {s:?}: {e}"),
        }
    }

    /// Parse a trigger expression, returning a descriptive error on failure.
    pub fn try_parse(s: &str) -> Result<ActionTrigger, ParseTriggerError> {
        let mut p = Parser::new(s);
        let t = p.parse_trigger()?;
        p.skip_ws();
        if p.pos < p.src.len() {
            return Err(p.err(format!("unexpected character {:?}", p.peek_char().unwrap())));
        }
        Ok(t)
    }
}

impl FromStr for ActionTrigger {
    type Err = ParseTriggerError;
    fn from_str(s: &str) -> Result<Self, Self::Err> { Self::try_parse(s) }
}

// ── Display ──────────────────────────────────────────────────────────────

impl fmt::Display for ActionTrigger {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, combo) in self.alternatives.iter().enumerate() {
            if i > 0 {
                f.write_str("; ")?;
            }
            write!(f, "{combo}")?;
        }
        Ok(())
    }
}

impl fmt::Display for Combo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, atom) in self.atoms.iter().enumerate() {
            if i > 0 {
                f.write_str("+")?;
            }
            write!(f, "{atom}")?;
        }
        Ok(())
    }
}

impl fmt::Display for Atom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Atom::Token(t) => f.write_str(t),
            Atom::Call { name, inner } => write!(f, "{name}({inner})"),
        }
    }
}

// ── Parser ───────────────────────────────────────────────────────────────

struct Parser<'a> {
    src:   &'a str,
    bytes: &'a [u8],
    pos:   usize,
}

impl<'a> Parser<'a> {
    fn new(src: &'a str) -> Self {
        Self {
            src,
            bytes: src.as_bytes(),
            pos: 0,
        }
    }

    fn err(&self, message: impl Into<String>) -> ParseTriggerError {
        ParseTriggerError {
            message:  message.into(),
            position: self.pos,
        }
    }

    fn skip_ws(&mut self) {
        while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_whitespace() {
            self.pos += 1;
        }
    }

    fn peek_char(&self) -> Option<char> { self.src[self.pos..].chars().next() }

    fn eat(&mut self, c: u8) -> bool {
        self.skip_ws();
        if self.pos < self.bytes.len() && self.bytes[self.pos] == c {
            self.pos += 1;
            true
        }
        else {
            false
        }
    }

    fn parse_ident(&mut self) -> Result<String, ParseTriggerError> {
        self.skip_ws();
        let start = self.pos;
        while self.pos < self.bytes.len() {
            let b = self.bytes[self.pos];
            let ok = if self.pos == start {
                b.is_ascii_alphabetic() || b == b'_'
            }
            else {
                b.is_ascii_alphanumeric() || b == b'_'
            };
            if !ok {
                break;
            }
            self.pos += 1;
        }
        if self.pos == start {
            return Err(self.err("expected identifier"));
        }
        Ok(self.src[start..self.pos].to_string())
    }

    fn parse_trigger(&mut self) -> Result<ActionTrigger, ParseTriggerError> {
        let mut alternatives = vec![self.parse_combo()?];
        while self.eat(b';') {
            alternatives.push(self.parse_combo()?);
        }
        Ok(ActionTrigger { alternatives })
    }

    fn parse_combo(&mut self) -> Result<Combo, ParseTriggerError> {
        let mut atoms = vec![self.parse_atom()?];
        while self.eat(b'+') {
            atoms.push(self.parse_atom()?);
        }
        Ok(Combo { atoms })
    }

    fn parse_atom(&mut self) -> Result<Atom, ParseTriggerError> {
        let name = self.parse_ident()?;
        if self.eat(b'(') {
            let inner = self.parse_trigger()?;
            if !self.eat(b')') {
                return Err(self.err("expected ')'"));
            }
            Ok(Atom::Call {
                name,
                inner: Box::new(inner),
            })
        }
        else {
            Ok(Atom::Token(name))
        }
    }
}

// ── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn roundtrip(s: &str, canonical: &str) {
        let t: ActionTrigger = s.parse().expect("parse");
        assert_eq!(t.to_string(), canonical, "input {s:?}");
        // Reparsing canonical form must yield an equal value.
        let t2: ActionTrigger = canonical.parse().expect("reparse");
        assert_eq!(t, t2);
    }

    #[test]
    fn simple_token() { roundtrip("escape", "escape"); }

    #[test]
    fn combo() { roundtrip("left+ctrl+shift", "left+ctrl+shift"); }

    #[test]
    fn alternatives() { roundtrip("delete; backspace; d", "delete; backspace; d"); }

    #[test]
    fn nested_calls() { roundtrip("click(double(mouse_left))", "click(double(mouse_left))"); }

    #[test]
    fn whitespace_tolerated() {
        roundtrip("  scroll_y  ;   touch_zoom_x ", "scroll_y; touch_zoom_x");
        roundtrip("drag( mouse_left + alt )", "drag(mouse_left+alt)");
    }

    #[test]
    fn rejects_trailing_garbage() {
        assert!("left++".parse::<ActionTrigger>().is_err());
        assert!("(".parse::<ActionTrigger>().is_err());
        assert!("drag(mouse_left".parse::<ActionTrigger>().is_err());
        assert!("".parse::<ActionTrigger>().is_err());
    }
}
