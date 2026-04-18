use egui::Align2;
use music_notation::note::harmony::{Chroma, Interval};
use music_notation::note::rhythm::{Duration, Time};
use music_notation::rendering::math2d::{Lerp, Rect, Vec2, vec2};
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
            if response.dragged_by(egui::PointerButton::Middle) {
                viewport -= viewport.size() * Vec2::from_egui(response.drag_delta() / rect.size());
            }

            // Prepare paint: Calculate appropriate grid size
            let grid_min_size = viewport.width() / 50;

            let mut grid_size = Duration::WHOLE;
            while grid_size / 2 > grid_min_size {
                grid_size /= 2;
            }

            // Paint grid
            for i in 0.. {
                let time = viewport.left.ceil(grid_size) + grid_size * i as i64;
                if time > viewport.right {
                    break;
                }

                let x = time.remap(viewport.x_range(), Rect::from_egui(rect).x_range());
                ui.painter().line_segment(
                    [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
                    (
                        1.0,
                        egui::Color32::from_white_alpha(
                            if (Time::ZERO - time) % Duration::WHOLE == Duration::ZERO {
                                255
                            }
                            else if (Time::ZERO - time) % Duration::BEAT == Duration::ZERO {
                                128
                            }
                            else if (Time::ZERO - time) % Duration::EIGHTH == Duration::ZERO {
                                64
                            }
                            else if (Time::ZERO - time) % Duration::SIXTEENTH == Duration::ZERO {
                                32
                            }
                            else {
                                0
                            },
                        ),
                    ),
                );
            }

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
                        .to_egui()
                        .shrink(1.0),
                        0.0,
                        boomwhacker_color(note.pitch.chroma(), 255),
                    );
                }
            }

            // Paint highlighted note
            if let Some(pointer) = ui.pointer_latest_pos() {
                let pos = Vec2::from_egui(pointer.to_vec2()).remap(Rect::from_egui(rect), viewport);

                let mut start = pos.x;
                let mut end = pos.x;
                let mut pitch = pos.y.with_cents(0.0);

                if response.dragged_by(egui::PointerButton::Primary)
                    && let Some(drag_start) = ui.input(|i| i.pointer.press_origin())
                {
                    let drag_start = Vec2::from_egui(drag_start.to_vec2())
                        .remap(Rect::from_egui(rect), viewport);
                    pitch = drag_start.y.with_cents(0.0);
                    end = drag_start.x;
                }

                if start > end {
                    std::mem::swap(&mut start, &mut end);
                }

                let note_color = boomwhacker_color(pitch.chroma(), 128);

                let note_rect = Rect {
                    left:   start.floor(grid_size),
                    right:  end.ceil(grid_size),
                    top:    pitch - Interval::HALFSTEP * 0.5,
                    bottom: pitch + Interval::HALFSTEP * 0.5,
                }
                .remap(viewport, Rect::from_egui(rect))
                .to_egui();
                ui.painter().rect_stroke(
                    note_rect,
                    0.0,
                    (1.0, note_color),
                    egui::StrokeKind::Inside,
                );
                ui.painter().text(
                    note_rect.right_center(),
                    Align2::LEFT_CENTER,
                    duration_name(grid_size),
                    egui::FontId::default(),
                    note_color,
                );
            }
        },
    )
}

fn duration_name(duration: Duration) -> &'static str {
    match duration {
        Duration::SIXTEENTH => "1/16",
        Duration::EIGHTH => "1/8",
        Duration::QUARTER => "1/4",
        Duration::HALF => "1/2",
        Duration::WHOLE => "1",
        _ => "?",
    }
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

fn boomwhacker_color(chroma: Chroma, alpha: u8) -> egui::Color32 {
    let rgb = egui::Color32::from_rgba_unmultiplied;
    match chroma {
        Chroma::A => rgb(78, 91, 185, alpha),       // Blue violet
        Chroma::ASharp => rgb(102, 53, 120, alpha), // Violet
        Chroma::B | Chroma::CFlat => rgb(232, 73, 157, alpha), // Hot pink
        Chroma::C => rgb(255, 29, 30, alpha),       // Red
        Chroma::CSharp => rgb(182, 42, 46, alpha),  // Light pink
        Chroma::D => rgb(241, 104, 88, alpha),      // Red Orange
        Chroma::DSharp => rgb(220, 113, 40, alpha), // Orange
        Chroma::E => rgb(238, 217, 112, alpha),     // Yellow
        Chroma::F => rgb(111, 224, 98, alpha),      // Green Yellow
        Chroma::FSharp => rgb(54, 138, 79, alpha),  // Green
        Chroma::G => rgb(58, 131, 145, alpha),      // Dark Green
        Chroma::GSharp => rgb(42, 71, 209, alpha),  // Blue
    }
}
