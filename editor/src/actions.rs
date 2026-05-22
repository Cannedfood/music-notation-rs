use std::collections::BTreeSet;

use crate::EditorState;
pub use crate::action_trigger::{ActionTrigger, trigger};

pub struct Action {
    pub id: String,
    pub name: String,
    pub description: String,
    pub trigger: ActionTrigger,
    pub update: Option<Box<dyn Fn(&mut EditorState)>>,
}
impl Action {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        trigger: ActionTrigger,
    ) -> Self {
        Action {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            trigger,
            update: None,
        }
    }
}

pub struct ActionMap {
    pub entries: Vec<Action>,
}

impl Default for ActionMap {
    fn default() -> Self {
        ActionMap {
            #[rustfmt::skip]
            entries: vec![
                // ── Viewport ──────────────────────────────────────────────
                Action::new("viewport.zoom_x", "Zoom Horizontal", "Zoom the view horizontally", trigger("scroll_y; touch_zoom_x")),
                Action::new("viewport.zoom_y", "Zoom Vertical",   "Zoom the view vertically",   trigger("scroll_y+ctrl; touch_zoom_y")),
                Action::new("viewport.pan_x",  "Pan Horizontal",  "Pan the view horizontally",  trigger("mouse_x+middle_mouse_button; touch_pan_x")),
                Action::new("viewport.pan_y",  "Pan Vertical",    "Pan the view vertically",    trigger("mouse_y+middle_mouse_button; touch_pan_y")),
                // ── Selection: Mouse ──────────────────────────────────────
                Action::new("selection.box_select",   "Box Select",       "Select items by dragging a box",         trigger("drag(mouse_left)")),
                Action::new("selection.click_set",    "Click Select",     "Set selection by clicking",              trigger("click(mouse_left)")),
                Action::new("selection.click_expand", "Expand Selection", "Expand selection box by shift-clicking", trigger("click(mouse_left+shift)")),
                Action::new("selection.click_add",    "Add to Selection", "Add to selection by ctrl-clicking",      trigger("click(mouse_left+ctrl)")),
                // ── Selection: Keyboard ───────────────────────────────────
                Action::new("selection.move_left",    "Move Selection Left",    "Move the current selection left",   trigger("left")),
                Action::new("selection.move_right",   "Move Selection Right",   "Move the current selection right",  trigger("right")),
                Action::new("selection.move_up",      "Move Selection Up",      "Move the current selection up",     trigger("up")),
                Action::new("selection.move_down",    "Move Selection Down",    "Move the current selection down",   trigger("down")),
                Action::new("selection.expand_left",  "Expand Selection Left",  "Expand the selection to the left",  trigger("left+shift")),
                Action::new("selection.expand_right", "Expand Selection Right", "Expand the selection to the right", trigger("right+shift")),
                Action::new("selection.expand_up",    "Expand Selection Up",    "Expand the selection upward",       trigger("up+shift")),
                Action::new("selection.expand_down",  "Expand Selection Down",  "Expand the selection downward",     trigger("down+shift")),
                Action::new("selection.clear",        "Clear Selection",        "Clear the current selection",       trigger("escape")),
                // ── Editing: Mouse ────────────────────────────────────────
                Action::new("edit.add_note",       "Add Note at Cursor", "Add a note at the cursor position", trigger("a")),
                Action::new("edit.drag_note",      "Drag Note",          "Drag a note to move it",            trigger("drag(mouse_left)")),
                Action::new("edit.drag_expand",    "Drag Expand",        "Drag to expand a note's duration",  trigger("drag(mouse_left+ctrl)")),
                Action::new("edit.drag_add_notes", "Drag Add Notes",     "Drag to add new notes",             trigger("drag(mouse_left+alt)")),
                Action::new("edit.click_delete",   "Delete",             "Double-click to delete a note",     trigger("click(double(mouse_left))")),
                Action::new("edit.click_create",   "Create",             "Double-click to create a note",     trigger("click(double(mouse_left))")),
                // ── Editing: Keyboard ─────────────────────────────────────
                Action::new("edit.nudge_left",     "Nudge Left",       "Nudge the selection left",         trigger("left+ctrl")),
                Action::new("edit.nudge_right",    "Nudge Right",      "Nudge the selection right",        trigger("right+ctrl")),
                Action::new("edit.nudge_up",       "Nudge Up",         "Nudge the selection up",           trigger("up+ctrl")),
                Action::new("edit.nudge_down",     "Nudge Down",       "Nudge the selection down",         trigger("down+ctrl")),
                Action::new("edit.add_note_left",  "Add Note Left",    "Add a note to the left",           trigger("left+alt")),
                Action::new("edit.add_note_right", "Add Note Right",   "Add a note to the right",          trigger("right+alt")),
                Action::new("edit.add_note_up",    "Add Note Up",      "Add a note above",                 trigger("up+alt")),
                Action::new("edit.add_note_down",  "Add Note Down",    "Add a note below",                 trigger("down+alt")),
                Action::new("edit.lengthen_left",  "Lengthen Left",    "Lengthen the note from the left",  trigger("left+ctrl+shift")),
                Action::new("edit.lengthen_right", "Lengthen Right",   "Lengthen the note from the right", trigger("right+ctrl+shift")),
                Action::new("edit.delete",         "Delete Selection", "Delete the selected notes",        trigger("delete; backspace; d")),
            ],
        }
    }
}
