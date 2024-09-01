#![feature(new_range_api)]

use egui::{Align2, Color32, FontId};
use music_notation::note::harmony::Chroma;
use music_notation::note::rhythm::{Time, TimeSignature};
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

fn show_score(ui: &mut egui::Ui, score: &mut Score, viewport: &mut MidiRollViewport) {
    let gradient = Gradient::new([
        egui::Color32::BLUE,
        egui::Color32::GREEN,
        egui::Color32::GOLD,
        egui::Color32::RED,
    ]);

    let (rect, res) = ui.allocate_exact_size(ui.available_size(), egui::Sense::click_and_drag());
    let mut midi_roll = MidiRoll::new(
        Rect {
            x: rect.left(),
            y: rect.top(),
            width: rect.width(),
            height: rect.height(),
        },
        *viewport,
    );

    ui.painter()
        .rect(rect, 0.0, egui::Color32::BLACK, (1.0, egui::Color32::WHITE));

    let painter = ui.painter_at(rect);

    if res.hovered() {
        let (zoomed, cursor_pos) = ui.input(|i| {
            (
                i.zoom_delta(),
                i.pointer.hover_pos().unwrap_or(rect.center()),
            )
        });
        viewport.zoom_by_factor(
            Vec2 {
                x: zoomed,
                y: zoomed,
            },
            (
                midi_roll.x_to_time(cursor_pos.x),
                midi_roll.y_to_pitch(cursor_pos.y),
            ),
        );
        midi_roll.viewport = *viewport;
    }

    if res.dragged() {
        let delta_time =
            midi_roll.width_to_beats(res.drag_delta().x + ui.input(|i| i.smooth_scroll_delta.x));
        let delta_pitch = midi_roll
            .height_to_halfsteps(res.drag_delta().y + ui.input(|i| i.smooth_scroll_delta.y));
        viewport.time_range.start -= delta_time;
        viewport.time_range.end -= delta_time;
        viewport.pitch_range.start += delta_pitch;
        viewport.pitch_range.end += delta_pitch;
    }

    for (time, beat_nr) in TimeSignature::default().beats(Time::ZERO, midi_roll.viewport.time_range)
    {
        let x = midi_roll.time_to_x(time);
        let stroke: egui::Stroke = {
            if beat_nr == 0 {
                (1.0, Color32::WHITE).into()
            }
            else {
                (1.0, Color32::GRAY).into()
            }
        };

        painter.line_segment(
            [
                (x, midi_roll.rect.top()).into(),
                (x, midi_roll.rect.bottom()).into(),
            ],
            stroke,
        );
    }

    for track in score.tracks.iter() {
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

        for note in track.notes[start..end].iter() {
            let note_rect = midi_roll.note_box(note.time, note.duration, note.pitch);
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
            let velocity_color = gradient.sample(note.velocity.to_f32());

            painter.rect(note_rect, 0.0, pitch_color, (1.0, velocity_color));
            if hovered {
                let mut text_position = note_rect.left_center();
                let mut outside = false;
                if text_position.x < midi_roll.rect.left() {
                    text_position.x = midi_roll.rect.left();
                    outside = true;
                }

                painter.text(
                    text_position,
                    Align2::LEFT_CENTER,
                    format!(
                        "{}{}, {}",
                        if outside { "< " } else { "" },
                        note.pitch,
                        note.velocity
                    ),
                    FontId::monospace(note_rect.height()),
                    Color32::WHITE,
                );
            }
        }
    }
}

fn main() {
    let score = music_notation::score::Score::from_midi_data(include_bytes!(
        "../../Queen - Bohemian Rhapsody.mid"
    ))
    .unwrap();

    let mut view = MidiRollViewport::default();

    eframe::run_simple_native(
        "Fun",
        eframe::NativeOptions::default(),
        move |cx, _frame| {
            egui::CentralPanel::default().show(cx, |ui| {
                show_score(ui, &mut score.clone(), &mut view);
            });
        },
    )
    .unwrap();
}
