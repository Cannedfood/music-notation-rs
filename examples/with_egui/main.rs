#![feature(new_range_api)]

mod player;

use core::f32;
use std::collections::HashSet;

use egui::{Align2, Color32, FontId};
use music_notation::note::articulation::Velocity;
use music_notation::note::harmony::{Chroma, Interval, Pitch};
use music_notation::note::rhythm::{Time, TimeSignature};
use music_notation::note::Note;
use music_notation::score::edit::{Cursor, EditState};
use music_notation::score::rendering::{MidiRoll, MidiRollViewport, Rect, Vec2};
use music_notation::score::Score;
use player::{start_player, Player};

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
    pub edit: EditState,
    pub view: MidiRoll,
    pub play_line: Time,
    pub playing: bool, // TODO: Move to player
    pub selected_parts: HashSet<usize>,
}
impl ScoreEditor {
    fn new(score: Score) -> Self {
        ScoreEditor {
            score,
            ..Default::default()
        }
    }

    fn show_part_manager(&mut self, ui: &mut egui::Ui) {
        ui.heading("Left Panel");
        for (i, part) in self.score.parts.iter().enumerate() {
            let mut selected = self.selected_parts.contains(&i);
            ui.checkbox(
                &mut selected,
                format!("Part {} {}", i, &part.description).trim(),
            );
            if selected {
                self.selected_parts.insert(i);
            }
            else {
                self.selected_parts.remove(&i);
            }
        }
    }

    fn show_midi_roll(&mut self, ui: &mut egui::Ui, player: &Player) {
        // Allocate rect
        let (rect, res) =
            ui.allocate_exact_size(ui.available_size(), egui::Sense::click_and_drag());
        self.view.rect = Rect {
            x: rect.left(),
            y: rect.top(),
            width: rect.width(),
            height: rect.height(),
        };

        for event in player.events.try_iter() {
            match event {
                player::PlayerEvents::Time(time) => self.play_line = time,
            }
        }

        if ui.input(|i| i.key_pressed(egui::Key::Space)) {
            if self.playing {
                player.commands.send(player::PlayerCommands::Pause).unwrap();
                self.playing = false;
            }
            else {
                player
                    .commands
                    .send(player::PlayerCommands::SetBuffer({
                        let mut notes: Vec<_> = self
                            .score
                            .parts
                            .iter()
                            .enumerate()
                            .filter_map(|(i, part)| {
                                self.selected_parts.contains(&i).then_some(part)
                            })
                            .flat_map(|t| t.notes.iter())
                            .cloned()
                            .collect();
                        notes.sort_unstable_by_key(|n| n.time);
                        notes
                    }))
                    .unwrap();
                player
                    .commands
                    .send(player::PlayerCommands::SetTime(self.play_line))
                    .unwrap();
                player.commands.send(player::PlayerCommands::Start).unwrap();

                self.playing = true;
            }
        }
        if self.playing {
            ui.ctx().request_repaint();
        }

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

        let pointer_pos_raw = ui.input(|i| i.pointer.hover_pos());
        let pointer_pos = ui.input(|i| {
            i.pointer
                .hover_pos()
                .map(|p| (self.view.x_to_time(p.x), self.view.y_to_pitch(p.y)))
        });

        if res.clicked_by(egui::PointerButton::Secondary) {
            if let Some(pointer_pos) = pointer_pos {
                self.play_line = pointer_pos.0;
                if self.playing {
                    player
                        .commands
                        .send(player::PlayerCommands::SetTime(self.play_line))
                        .unwrap();
                    player
                        .commands
                        .send(player::PlayerCommands::SetBuffer({
                            let mut notes: Vec<_> = self
                                .score
                                .parts
                                .iter()
                                .enumerate()
                                .filter_map(|(i, part)| {
                                    self.selected_parts.contains(&i).then_some(part)
                                })
                                .flat_map(|t| t.notes.iter())
                                .cloned()
                                .collect();
                            notes.sort_unstable_by_key(|n| n.time);
                            notes
                        }))
                        .unwrap();
                }
            }
        }

        // Paint background / border
        ui.painter()
            .rect(rect, 0.0, egui::Color32::BLACK, (1.0, egui::Color32::WHITE));

        // Paint content
        let painter = ui.painter_at(rect);
        self.paint_beat_lines(&painter);
        self.paint_note_lines(&painter);
        painter.line_segment(
            [
                (self.view.time_to_x(self.play_line), self.view.rect.top()).into(),
                (self.view.time_to_x(self.play_line), self.view.rect.bottom()).into(),
            ],
            egui::Stroke::new(1.0, Color32::from_rgb(255, 0, 0)),
        );

        let mut any_note_hovered = false;

        for (i, part) in self.score.parts.iter().enumerate() {
            let part_selected = self.selected_parts.contains(&i);
            if !part_selected {
                continue;
            }

            for note in Self::visible_note_range_in(&self.view.viewport, part) {
                let rect = self.paint_note(note, ui, &painter, part_selected);
                let note_hovered = pointer_pos_raw.map(|p| rect.contains(p)).unwrap_or(false);
                any_note_hovered |= note_hovered;

                if res.clicked() && note_hovered {
                    if !ui.input(|i| i.modifiers.ctrl) {
                        self.edit.cursors.clear();
                    }
                    self.edit.cursors.push(Cursor {
                        part: 0,
                        time_range: (note.time..(note.time + note.duration)).into(),
                        pitch_range: (note.pitch..note.pitch).into(),
                    });
                }
            }
        }

        if !any_note_hovered {
            if let Some(pointer_pos) = pointer_pos {
                let time_sig = TimeSignature::default();
                let position = time_sig.grid(Time::ZERO).closest(pointer_pos.0).unwrap();
                let rect = self.view.note_box(
                    position,
                    time_sig.subdivision_duration(),
                    pointer_pos.1.with_cents(0.0),
                );
                let rect = egui::Rect::from_min_size(
                    (rect.x, rect.y).into(),
                    (rect.width, rect.height).into(),
                );

                painter.rect_stroke(rect, 0.0, egui::Stroke::new(1.0, Color32::WHITE));

                if res.clicked() {
                    let new_note = Note {
                        time: position,
                        duration: time_sig.subdivision_duration(),
                        pitch: pointer_pos.1.with_cents(0.0),
                        velocity: Velocity::from_f32(1.0),
                        ..Default::default()
                    };

                    let notes = &mut self.score.parts[0].notes;
                    let insert_at = notes
                        .binary_search_by_key(&position, |note| note.time)
                        .unwrap_or_else(|i| i);
                    notes.insert(insert_at, new_note);
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
        part: &'a music_notation::score::Part,
    ) -> impl Iterator<Item = &'a Note> + 'a {
        let start = match part
            .notes
            .binary_search_by_key(&viewport.time_range.start, |note| note.time + note.duration)
        {
            Ok(i) => i,
            Err(i) => i,
        };
        let end = match part
            .notes
            .binary_search_by_key(&viewport.time_range.end, |note| note.time)
        {
            Ok(i) => i,
            Err(i) => i,
        };

        part.notes[start..end].iter()
    }

    fn paint_beat_lines(&self, painter: &egui::Painter) {
        let time_sig = TimeSignature::default();
        for (i, time) in time_sig
            .grid(Time::ZERO)
            .iter_in_range(self.view.viewport.time_range)
        {
            let beat_nr = i % time_sig.numerator as i64;

            let x = self.view.time_to_x(time);
            let brightness = self.view.beat_width().clamp(0.0, 255.0) as u8;
            let stroke: egui::Stroke = {
                if time == Time::ZERO {
                    (5.0, Color32::from_gray(brightness)).into()
                }
                else if beat_nr == 0 {
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
        part_selected: bool,
    ) -> egui::Rect {
        let note_rect = self.view.note_box(note.time, note.duration, note.pitch);
        let note_rect = egui::Rect::from_min_size(
            (note_rect.x, note_rect.y).into(),
            (note_rect.width.max(5.0), note_rect.height).into(),
        );

        let hovered = ui.input(|i| {
            i.pointer
                .hover_pos()
                .map(|p| note_rect.contains(p))
                .unwrap_or(false)
        });

        let pitch_color = boomwhacker_color(note.pitch.chroma());
        // let velocity_color = gradient.sample(note.velocity.to_f32());

        if part_selected {
            painter.rect_filled(note_rect, 0.0, pitch_color);
        }
        else {
            painter.rect_filled(
                note_rect,
                0.0,
                egui::Color32::from_rgba_unmultiplied(
                    pitch_color.r(),
                    pitch_color.g(),
                    pitch_color.b(),
                    0x22,
                ),
            );
        }
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

    // ron::ser::to_writer_pretty(
    //     BufWriter::new(File::create("./out.ron").unwrap()),
    //     &score_editor.score,
    //     Default::default(),
    // )
    // .unwrap();

    let player = start_player();

    eframe::run_simple_native(
        "Fun",
        eframe::NativeOptions::default(),
        move |cx, _frame| {
            egui::SidePanel::left("left_panel").show(cx, |ui| {
                score_editor.show_part_manager(ui);
            });

            egui::CentralPanel::default().show(cx, |ui| {
                score_editor.show_midi_roll(ui, &player);
            });
        },
    )
    .unwrap();
}
