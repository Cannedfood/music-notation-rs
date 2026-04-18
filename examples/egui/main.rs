use music_notation::note::harmony::{Chroma, Interval};
use music_notation::note::rhythm::{Duration, Time};
use music_notation::rendering::math2d::{Rect, Vec2, vec2};
use music_notation::score::Score;

fn main() -> Result<(), eframe::Error> {
    let score =
        Score::from_midi_data(include_bytes!("../../Queen - Bohemian Rhapsody.mid")).unwrap();
    let mut viewport = Rect::from_ranges(
        score.time_range().unwrap_or_default(),
        score.pitch_range().unwrap_or_default(),
    );

    eframe::run_ui_native(
        "Editor",
        eframe::NativeOptions::default(),
        move |ui, _frame| {
            ui.label("Hello, world!");

            let (rect, response) = ui.allocate_at_least(ui.available_size(), egui::Sense::all());

            // Handle interactions (before painting for latency!!)
            // Zoom
            let (zoomed, cursor_pos) = ui.input(|i| {
                (
                    i.zoom_delta(),
                    i.pointer.hover_pos().unwrap_or(rect.center()),
                )
            });
            viewport = viewport.zoom(
                vec2(zoomed, 1.0),
                Vec2::from_egui(cursor_pos.to_vec2()).remap(Rect::from_egui(rect), viewport),
            );
            // Drag/Move around
            viewport -= viewport.size() * Vec2::from_egui(response.drag_delta() / rect.size());

            // Paint notes
            ui.set_clip_rect(rect);
            for part in &score.parts {
                for note in &part.notes {
                    ui.painter().rect_filled(
                        Rect {
                            left:   note.time,
                            right:  note.time + note.duration,
                            top:    note.pitch - Interval::HALFSTEP * 0.5,
                            bottom: note.pitch + Interval::HALFSTEP * 0.5,
                        }
                        .remap(viewport, Rect::from_egui(rect))
                        .to_egui(),
                        0.0,
                        boomwhacker_color(note.pitch.chroma()),
                    );
                }
            }

            // Paint highlighted note
            if let Some(hover_pos) = response.hover_pos() {
                let pos =
                    Vec2::from_egui(hover_pos.to_vec2()).remap(Rect::from_egui(rect), viewport);

                let time_grid_size = Duration::EIGHTH;
                let time = Time::ZERO + ((pos.x - Time::ZERO) / time_grid_size) * time_grid_size;
                let pitch = pos.y.with_cents(0.0);

                let note_rect = Rect {
                    left:   time,
                    right:  time + time_grid_size,
                    top:    pitch - Interval::HALFSTEP * 0.5,
                    bottom: pitch + Interval::HALFSTEP * 0.5,
                }
                .remap(viewport, Rect::from_egui(rect))
                .to_egui();
                ui.painter().rect_stroke(
                    note_rect,
                    0.0,
                    (1.0, egui::Color32::from_white_alpha(128)),
                    egui::StrokeKind::Inside,
                );
            }
        },
    )
}

trait EguiConvert {
    type T;

    fn to_egui(&self) -> Self::T;
    fn from_egui(value: Self::T) -> Self;
}
impl EguiConvert for Rect {
    type T = egui::Rect;

    fn to_egui(&self) -> Self::T { egui::Rect::from_x_y_ranges(self.x_range(), self.y_range()) }

    fn from_egui(value: Self::T) -> Self {
        Rect {
            left:   value.left(),
            right:  value.right(),
            top:    value.top(),
            bottom: value.bottom(),
        }
    }
}
impl EguiConvert for Vec2 {
    type T = egui::Vec2;

    fn to_egui(&self) -> Self::T { egui::Vec2::new(self.x, self.y) }
    fn from_egui(value: Self::T) -> Self {
        Vec2 {
            x: value.x,
            y: value.y,
        }
    }
}

fn boomwhacker_color(chroma: Chroma) -> egui::Color32 {
    let rgb = egui::Color32::from_rgb;
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
