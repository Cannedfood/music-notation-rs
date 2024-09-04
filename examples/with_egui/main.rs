#![feature(new_range_api)]

use egui::{Align2, Color32, FontId};
use music_notation::note::harmony::{Chroma, Interval, Pitch};
use music_notation::note::rhythm::{Time, TimeSignature};
use music_notation::note::Note;
use music_notation::score::edit::{Cursor, EditState};
use music_notation::score::rendering::{MidiRoll, MidiRollViewport, Rect, Vec2};
use music_notation::score::Score;

#[derive(Debug, Clone, Copy)]
pub struct Gradient<const N: usize>([egui::Color32; N]);
impl<const N: usize> Gradient<N> {
    pub const fn new(colors: [egui::Color32; N]) -> Self { Gradient(colors) }

    pub fn sample(&self, t: f32) -> egui::Color32 {
        let idx_f = t.clamp(0.0, 1.0) * (N - 1) as f32;
        let lo = self.0[idx_f.floor() as usize];
        let hi = self.0[idx_f.ceil() as usize];
        let frac = idx_f.fract();

        lo.lerp_to_gamma(hi, frac)
    }
}

fn boomwhacker_color(chroma: Chroma) -> Color32 {
    fn rgb(r: u32, g: u32, b: u32) -> Color32 { Color32::from_rgb(r as u8, g as u8, b as u8) }
    match chroma {
        Chroma::A => rgb(78, 91, 185),                  // Blue violet
        Chroma::ASharp => rgb(102, 53, 120),            // Violet
        Chroma::B | Chroma::CFlat => rgb(232, 73, 157), // Hot pink
        Chroma::C => rgb(255, 29, 30),                  // Red
        Chroma::CSharp => rgb(182, 42, 46),             // Light pink
        Chroma::D => rgb(241, 104, 88),                 // Red Orange
        Chroma::DSharp => rgb(220, 113, 40),            // Orange
        Chroma::E => rgb(238, 217, 112),                // Yellow
        Chroma::F => rgb(111, 224, 98),                 // Green Yellow
        Chroma::FSharp => rgb(54, 138, 79),             // Green
        Chroma::G => rgb(58, 131, 145),                 // Dark Green
        Chroma::GSharp => rgb(42, 71, 209),             // Blue
    }
}

#[derive(Default, Debug, Clone)]
pub struct ScoreEditor {
    pub score: Score,
    pub edit:  EditState,
    pub view:  MidiRoll,
}
impl ScoreEditor {
    fn new(score: Score) -> Self {
        ScoreEditor {
            score,
            ..Default::default()
        }
    }

    fn show(&mut self, ui: &mut egui::Ui) {
        // Allocate rect
        let (rect, res) =
            ui.allocate_exact_size(ui.available_size(), egui::Sense::click_and_drag());
        self.view.rect = Rect {
            x: rect.left(),
            y: rect.top(),
            width: rect.width(),
            height: rect.height(),
        };

        // Handle events

        if res.hovered() {
            let (zoomed, cursor_pos) = ui.input(|i| {
                (
                    i.zoom_delta(),
                    i.pointer.hover_pos().unwrap_or(rect.center()),
                )
            });
            self.view.viewport.zoom_by_factor(
                Vec2 { x: zoomed, y: 1.0 },
                (
                    self.view.x_to_time(cursor_pos.x),
                    self.view.y_to_pitch(cursor_pos.y),
                ),
            );
        }

        if res.dragged() {
            let delta_time = self
                .view
                .width_to_beats(res.drag_delta().x + ui.input(|i| i.smooth_scroll_delta.x));
            let delta_pitch = self
                .view
                .height_to_halfsteps(res.drag_delta().y + ui.input(|i| i.smooth_scroll_delta.y));
            self.view.viewport.time_range.start -= delta_time;
            self.view.viewport.time_range.end -= delta_time;
            self.view.viewport.pitch_range.start += delta_pitch;
            self.view.viewport.pitch_range.end += delta_pitch;
        }

        // Paint background / border
        ui.painter()
            .rect(rect, 0.0, egui::Color32::BLACK, (1.0, egui::Color32::WHITE));

        // Paint content
        let painter = ui.painter_at(rect);
        self.paint_beat_lines(&painter);
        self.paint_note_lines(&painter);

        for track in self.score.tracks.iter() {
            for note in Self::visible_note_range_in(&self.view.viewport, track) {
                let rect = self.paint_note(note, ui, &painter);
                if res.clicked()
                    && res
                        .interact_pointer_pos()
                        .map(|p| rect.contains(p))
                        .unwrap_or(false)
                {
                    if !ui.input(|i| i.modifiers.ctrl) {
                        self.edit.cursors.clear();
                    }
                    self.edit.cursors.push(Cursor {
                        track: 0,
                        time_range: (note.time..(note.time + note.duration)).into(),
                        pitch_range: (note.pitch..note.pitch).into(),
                    });
                }
            }
        }

        for cursor in self.edit.cursors.iter() {
            let cursor_rect = egui::Rect::from_min_max(
                (
                    self.view.time_to_x(cursor.time_range.start),
                    self.view.pitch_to_y(cursor.pitch_range.end),
                )
                    .into(),
                (
                    self.view.time_to_x(cursor.time_range.end),
                    self.view
                        .pitch_to_y(cursor.pitch_range.start - Interval::HALFSTEP),
                )
                    .into(),
            );
            painter.rect_stroke(cursor_rect, 0.0, egui::Stroke::new(1.0, Color32::WHITE));
        }
    }

    fn visible_note_range_in<'a>(
        viewport: &MidiRollViewport,
        track: &'a music_notation::score::Part,
    ) -> impl Iterator<Item = &'a Note> + 'a {
        let start = match track
            .notes
            .binary_search_by_key(&viewport.time_range.start, |note| note.time + note.duration)
        {
            Ok(i) => i,
            Err(i) => i,
        };
        let end = match track
            .notes
            .binary_search_by_key(&viewport.time_range.end, |note| note.time)
        {
            Ok(i) => i,
            Err(i) => i,
        };

        track.notes[start..end].iter()
    }

    fn paint_beat_lines(&self, painter: &egui::Painter) {
        for (time, beat_nr) in
            TimeSignature::default().beats(Time::ZERO, self.view.viewport.time_range)
        {
            let x = self.view.time_to_x(time);
            let brightness = self.view.beat_width().clamp(0.0, 255.0) as u8;
            let stroke: egui::Stroke = {
                if beat_nr == 0 {
                    (1.0, Color32::from_gray(brightness)).into()
                }
                else {
                    (1.0, Color32::from_gray(brightness / 3)).into()
                }
            };

            painter.line_segment(
                [
                    (x, self.view.rect.top()).into(),
                    (x, self.view.rect.bottom()).into(),
                ],
                stroke,
            );
        }
    }

    fn paint_note_lines(&self, painter: &egui::Painter) {
        for pitch in (0..255).step_by(12).map(Pitch::from_midi) {
            let brightness = (self.view.halfstep_height() * 12.0).clamp(0.0, 255.0) as u8;
            let stroke: egui::Stroke = if pitch.chroma() == Chroma::C && pitch.octave() == 4 {
                (3.0, Color32::from_gray(brightness)).into()
            }
            else {
                (1.0, Color32::from_gray(brightness / 3)).into()
            };

            let y = self.view.pitch_to_y(pitch.with_cents(-50.0));
            painter.line_segment(
                [
                    (self.view.rect.left(), y).into(),
                    (self.view.rect.right(), y).into(),
                ],
                stroke,
            );
        }
    }

    fn paint_note(
        &self,
        note: &music_notation::note::Note,
        ui: &mut egui::Ui,
        painter: &egui::Painter,
    ) -> egui::Rect {
        let note_rect = self.view.note_box(note.time, note.duration, note.pitch);
        let note_rect = egui::Rect::from_min_size(
            (note_rect.x, note_rect.y).into(),
            (note_rect.width, note_rect.height).into(),
        );

        let hovered = ui.input(|i| {
            i.pointer
                .hover_pos()
                .map(|p| note_rect.contains(p))
                .unwrap_or(false)
        });

        let pitch_color = boomwhacker_color(note.pitch.chroma());
        // let velocity_color = gradient.sample(note.velocity.to_f32());

        painter.rect_filled(note_rect, 0.0, pitch_color);
        if hovered {
            let content_rect = note_rect.shrink(3.0);

            let mut text_position = content_rect.left_center();
            let mut outside = false;
            if text_position.x < self.view.rect.left() {
                text_position.x = self.view.rect.left();
                outside = true;
            }

            let luminance = {
                let [r, g, b, _] = egui::Rgba::from(pitch_color).to_array();
                (0.299 * r * r + 0.587 * g * g + 0.114 * b * b).sqrt()
            };

            painter.text(
                text_position,
                Align2::LEFT_CENTER,
                format!(
                    "{}{}, {}",
                    if outside { "< " } else { "" },
                    note.pitch,
                    note.velocity
                ),
                FontId::monospace(content_rect.height().clamp(8.0, 32.0)),
                if luminance < 0.5 {
                    Color32::WHITE
                }
                else {
                    Color32::BLACK
                },
            );
        }

        note_rect
    }
}

fn main() {
    let mut score_editor = ScoreEditor::new(
        music_notation::score::Score::from_midi_data(include_bytes!(
            "../../Queen - Bohemian Rhapsody.mid"
        ))
        .unwrap(),
    );

    eframe::run_simple_native(
        "Fun",
        eframe::NativeOptions::default(),
        move |cx, _frame| {
            egui::CentralPanel::default().show(cx, |ui| {
                score_editor.show(ui);
            });
        },
    )
    .unwrap();
}
