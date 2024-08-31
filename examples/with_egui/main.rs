use egui::{Align2, Color32, FontId};
use music_data::note::harmony::Pitch;
use music_data::score::rendering::{MidiRoll, Rect, Viewport};
use music_data::score::Score;

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

fn show_score(ui: &mut egui::Ui, score: &mut Score, view: &mut Viewport) {
    let gradient = Gradient::new([
        egui::Color32::BLUE,
        egui::Color32::GREEN,
        egui::Color32::GOLD,
        egui::Color32::RED,
    ]);

    let (rect, res) = ui.allocate_exact_size(ui.available_size(), egui::Sense::click_and_drag());
    let midi_roll = MidiRoll::new(
        Rect {
            x: rect.left(),
            y: rect.top(),
            width: rect.width(),
            height: rect.height(),
        },
        *view,
        score,
    );

    ui.painter()
        .rect(rect, 0.0, egui::Color32::BLACK, (1.0, egui::Color32::WHITE));

    let painter = ui.painter_at(rect);

    if res.hovered() {
        let factor = 2.0f32.powf(ui.input(|i| -i.smooth_scroll_delta.y / 500.0));
        if factor != 1.0 {
            let halfstep_height = rect.height() / (view.pitch_end - view.pitch_start).halfsteps();

            let pivot = Pitch(
                ui.input(|i| i.pointer.hover_pos().unwrap_or_default().y - rect.top())
                    / halfstep_height,
            );
            let above_pivot = view.pitch_end - pivot;
            let below_pivot = view.pitch_start - pivot;

            view.pitch_start = pivot + below_pivot * factor;
            view.pitch_end = pivot + above_pivot * factor;
        }
    }

    if res.dragged() {
        let delta_time = -midi_roll.width_to_beats(res.drag_delta().x);
        view.time_start += delta_time;
        view.time_end += delta_time;
    }

    for track in score.tracks.iter() {
        let start = match track
            .notes
            .binary_search_by_key(&view.time_start, |note| note.time + note.duration)
        {
            Ok(i) => i,
            Err(i) => i,
        };
        let end = match track
            .notes
            .binary_search_by_key(&view.time_end, |note| note.time)
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
    let score = music_data::score::Score::from_midi_data(include_bytes!(
        "../../Queen - Bohemian Rhapsody.mid"
    ))
    .unwrap();

    let mut view = Viewport::default();

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
