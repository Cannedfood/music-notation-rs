use egui::{Align2, Color32, FontId};
use music_notation::note::harmony::Chroma;
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
        let (zoomed, cursor_pos) = ui.input(|i| (i.zoom_delta(), i.pointer.hover_pos().unwrap()));
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
        viewport.time_start -= delta_time;
        viewport.time_end -= delta_time;
        viewport.pitch_start += delta_pitch;
        viewport.pitch_end += delta_pitch;
    }

    for track in score.tracks.iter() {
        let start = match track
            .notes
            .binary_search_by_key(&viewport.time_start, |note| note.time + note.duration)
        {
            Ok(i) => i,
            Err(i) => i,
        };
        let end = match track
            .notes
            .binary_search_by_key(&viewport.time_end, |note| note.time)
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

            painter.rect_filled(note_rect, 0.0, gradient.sample(note.velocity.to_f32()));
            if hovered {
                painter.text(
                    note_rect.left_top(),
                    Align2::LEFT_TOP,
                    format!("{}, {}", note.pitch, note.velocity),
                    FontId::proportional(note_rect.height()),
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
