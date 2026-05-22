use std::collections::BTreeSet;

use music_notation::note::harmony::Pitch;
use music_notation::note::rhythm::Time;
use music_notation::rendering::math2d::Rect;
use music_notation::score::Score;

pub use crate::action_trigger::{ActionTrigger, trigger};

pub struct EditorState {
    pub viewport: Rect<Time, Pitch>,
    pub score: Score,
    pub last_drawn: Option<Rect<Time, Pitch>>,
    pub selections: Vec<Rect<Time, Pitch>>,
}

pub struct Action {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: BTreeSet<String>,
    pub trigger: ActionTrigger,
}

pub struct ActionMap {
    pub entries: Vec<Action>,
}

impl Default for ActionMap {
    fn default() -> Self {
        ActionMap {
            entries: vec![
                // ── Viewport ──────────────────────────────────────────────
                Action {
                    id: "viewport.zoom_x".into(),
                    name: "Zoom Horizontal".into(),
                    description: "Zoom the view horizontally".into(),
                    category: BTreeSet::from(["viewport".into()]),
                    trigger: trigger("scroll_y; touch_zoom_x"),
                },
                Action {
                    id: "viewport.zoom_y".into(),
                    name: "Zoom Vertical".into(),
                    description: "Zoom the view vertically".into(),
                    category: BTreeSet::from(["viewport".into()]),
                    trigger: trigger("scroll_y+ctrl; touch_zoom_y"),
                },
                Action {
                    id: "viewport.pan_x".into(),
                    name: "Pan Horizontal".into(),
                    description: "Pan the view horizontally".into(),
                    category: BTreeSet::from(["viewport".into()]),
                    trigger: trigger("mouse_x+middle_mouse_button; touch_pan_x"),
                },
                Action {
                    id: "viewport.pan_y".into(),
                    name: "Pan Vertical".into(),
                    description: "Pan the view vertically".into(),
                    category: BTreeSet::from(["viewport".into()]),
                    trigger: trigger("mouse_y+middle_mouse_button; touch_pan_y"),
                },
                // ── Selection: Mouse ──────────────────────────────────────
                Action {
                    id: "selection.box_select".into(),
                    name: "Box Select".into(),
                    description: "Select items by dragging a box".into(),
                    category: BTreeSet::from(["selection".into(), "mouse".into()]),
                    trigger: trigger("drag(mouse_left)"),
                },
                Action {
                    id: "selection.click_set".into(),
                    name: "Click Select".into(),
                    description: "Set selection by clicking".into(),
                    category: BTreeSet::from(["selection".into(), "mouse".into()]),
                    trigger: trigger("click(mouse_left)"),
                },
                Action {
                    id: "selection.click_expand".into(),
                    name: "Expand Selection".into(),
                    description: "Expand selection box by shift-clicking".into(),
                    category: BTreeSet::from(["selection".into(), "mouse".into()]),
                    trigger: trigger("click(mouse_left+shift)"),
                },
                Action {
                    id: "selection.click_add".into(),
                    name: "Add to Selection".into(),
                    description: "Add to selection by ctrl-clicking".into(),
                    category: BTreeSet::from(["selection".into(), "mouse".into()]),
                    trigger: trigger("click(mouse_left+ctrl)"),
                },
                // ── Selection: Keyboard ───────────────────────────────────
                Action {
                    id: "selection.move_left".into(),
                    name: "Move Selection Left".into(),
                    description: "Move the current selection left".into(),
                    category: BTreeSet::from(["selection".into(), "keyboard".into()]),
                    trigger: trigger("left"),
                },
                Action {
                    id: "selection.move_right".into(),
                    name: "Move Selection Right".into(),
                    description: "Move the current selection right".into(),
                    category: BTreeSet::from(["selection".into(), "keyboard".into()]),
                    trigger: trigger("right"),
                },
                Action {
                    id: "selection.move_up".into(),
                    name: "Move Selection Up".into(),
                    description: "Move the current selection up".into(),
                    category: BTreeSet::from(["selection".into(), "keyboard".into()]),
                    trigger: trigger("up"),
                },
                Action {
                    id: "selection.move_down".into(),
                    name: "Move Selection Down".into(),
                    description: "Move the current selection down".into(),
                    category: BTreeSet::from(["selection".into(), "keyboard".into()]),
                    trigger: trigger("down"),
                },
                Action {
                    id: "selection.expand_left".into(),
                    name: "Expand Selection Left".into(),
                    description: "Expand the selection to the left".into(),
                    category: BTreeSet::from(["selection".into(), "keyboard".into()]),
                    trigger: trigger("left+shift"),
                },
                Action {
                    id: "selection.expand_right".into(),
                    name: "Expand Selection Right".into(),
                    description: "Expand the selection to the right".into(),
                    category: BTreeSet::from(["selection".into(), "keyboard".into()]),
                    trigger: trigger("right+shift"),
                },
                Action {
                    id: "selection.expand_up".into(),
                    name: "Expand Selection Up".into(),
                    description: "Expand the selection upward".into(),
                    category: BTreeSet::from(["selection".into(), "keyboard".into()]),
                    trigger: trigger("up+shift"),
                },
                Action {
                    id: "selection.expand_down".into(),
                    name: "Expand Selection Down".into(),
                    description: "Expand the selection downward".into(),
                    category: BTreeSet::from(["selection".into(), "keyboard".into()]),
                    trigger: trigger("down+shift"),
                },
                Action {
                    id: "selection.clear".into(),
                    name: "Clear Selection".into(),
                    description: "Clear the current selection".into(),
                    category: BTreeSet::from(["selection".into(), "keyboard".into()]),
                    trigger: trigger("escape"),
                },
                // ── Editing: Mouse ────────────────────────────────────────
                Action {
                    id: "edit.add_note".into(),
                    name: "Add Note at Cursor".into(),
                    description: "Add a note at the cursor position".into(),
                    category: BTreeSet::from(["editing".into(), "mouse".into()]),
                    trigger: trigger("a"),
                },
                Action {
                    id: "edit.drag_note".into(),
                    name: "Drag Note".into(),
                    description: "Drag a note to move it".into(),
                    category: BTreeSet::from(["editing".into(), "mouse".into()]),
                    trigger: trigger("drag(mouse_left)"),
                },
                Action {
                    id: "edit.drag_expand".into(),
                    name: "Drag Expand".into(),
                    description: "Drag to expand a note's duration".into(),
                    category: BTreeSet::from(["editing".into(), "mouse".into()]),
                    trigger: trigger("drag(mouse_left+ctrl)"),
                },
                Action {
                    id: "edit.drag_add_notes".into(),
                    name: "Drag Add Notes".into(),
                    description: "Drag to add new notes".into(),
                    category: BTreeSet::from(["editing".into(), "mouse".into()]),
                    trigger: trigger("drag(mouse_left+alt)"),
                },
                Action {
                    id: "edit.click_delete".into(),
                    name: "Delete".into(),
                    description: "Double-click to delete a note".into(),
                    category: BTreeSet::from(["editing".into(), "mouse".into()]),
                    trigger: trigger("click(double(mouse_left))"),
                },
                Action {
                    id: "edit.click_create".into(),
                    name: "Create".into(),
                    description: "Double-click to create a note".into(),
                    category: BTreeSet::from(["editing".into(), "mouse".into()]),
                    trigger: trigger("click(double(mouse_left))"),
                },
                // ── Editing: Keyboard ─────────────────────────────────────
                Action {
                    id: "edit.nudge_left".into(),
                    name: "Nudge Left".into(),
                    description: "Nudge the selection left".into(),
                    category: BTreeSet::from(["editing".into(), "keyboard".into()]),
                    trigger: trigger("left+ctrl"),
                },
                Action {
                    id: "edit.nudge_right".into(),
                    name: "Nudge Right".into(),
                    description: "Nudge the selection right".into(),
                    category: BTreeSet::from(["editing".into(), "keyboard".into()]),
                    trigger: trigger("right+ctrl"),
                },
                Action {
                    id: "edit.nudge_up".into(),
                    name: "Nudge Up".into(),
                    description: "Nudge the selection up".into(),
                    category: BTreeSet::from(["editing".into(), "keyboard".into()]),
                    trigger: trigger("up+ctrl"),
                },
                Action {
                    id: "edit.nudge_down".into(),
                    name: "Nudge Down".into(),
                    description: "Nudge the selection down".into(),
                    category: BTreeSet::from(["editing".into(), "keyboard".into()]),
                    trigger: trigger("down+ctrl"),
                },
                Action {
                    id: "edit.add_note_left".into(),
                    name: "Add Note Left".into(),
                    description: "Add a note to the left".into(),
                    category: BTreeSet::from(["editing".into(), "keyboard".into()]),
                    trigger: trigger("left+alt"),
                },
                Action {
                    id: "edit.add_note_right".into(),
                    name: "Add Note Right".into(),
                    description: "Add a note to the right".into(),
                    category: BTreeSet::from(["editing".into(), "keyboard".into()]),
                    trigger: trigger("right+alt"),
                },
                Action {
                    id: "edit.add_note_up".into(),
                    name: "Add Note Up".into(),
                    description: "Add a note above".into(),
                    category: BTreeSet::from(["editing".into(), "keyboard".into()]),
                    trigger: trigger("up+alt"),
                },
                Action {
                    id: "edit.add_note_down".into(),
                    name: "Add Note Down".into(),
                    description: "Add a note below".into(),
                    category: BTreeSet::from(["editing".into(), "keyboard".into()]),
                    trigger: trigger("down+alt"),
                },
                Action {
                    id: "edit.lengthen_left".into(),
                    name: "Lengthen Left".into(),
                    description: "Lengthen the note from the left".into(),
                    category: BTreeSet::from(["editing".into(), "keyboard".into()]),
                    trigger: trigger("left+ctrl+shift"),
                },
                Action {
                    id: "edit.lengthen_right".into(),
                    name: "Lengthen Right".into(),
                    description: "Lengthen the note from the right".into(),
                    category: BTreeSet::from(["editing".into(), "keyboard".into()]),
                    trigger: trigger("right+ctrl+shift"),
                },
                Action {
                    id: "edit.delete".into(),
                    name: "Delete Selection".into(),
                    description: "Delete the selected notes".into(),
                    category: BTreeSet::from(["editing".into(), "keyboard".into()]),
                    trigger: trigger("delete; backspace; d"),
                },
            ],
        }
    }
}
