use egui::Align2;
use music_notation::note::Note;
use music_notation::note::harmony::{Chroma, Interval, Pitch};
use music_notation::note::rhythm::{Duration, Time};
use music_notation::rendering::math2d::{Lerp, Rect, Vec2, vec2};
use music_notation::score::Score;

fn main() -> Result<(), eframe::Error> {
    let mut score =
        Score::from_midi_data(include_bytes!("../../Queen - Bohemian Rhapsody.mid")).unwrap();
    let mut viewport = Rect::from_ranges(
        score.time_range().unwrap_or_default(),
        score.pitch_range().unwrap_or_default(),
    );

    let mut last_drawn: Option<Rect<Time, Pitch>> = None;
    let mut selections: Vec<Rect<Time, Pitch>> = Vec::new();

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

            // Place note
            if let Some(last_drawn) = last_drawn
                && ui.input(|i| i.pointer.primary_released())
            {
                let note = Note {
                    time: last_drawn.left,
                    duration: last_drawn.width(),
                    pitch: last_drawn.top + (last_drawn.bottom - last_drawn.top) / 2.0,
                    ..Default::default()
                };

                let mut place_note = true;
                for part in score.parts.iter_mut() {
                    part.notes.retain(|n| {
                        if n.pitch != note.pitch {
                            return true;
                        }

                        let exact_overlap = n.time == note.time && n.duration == note.duration;
                        if exact_overlap {
                            place_note = false;
                        }

                        n.time >= (note.time + note.duration) || n.time + n.duration <= note.time
                    });
                }

                if place_note {
                    selections.clear();
                    if ui.input(|i| i.modifiers.alt) {
                        let start = note.time;
                        let end = note.time + note.duration;

                        for i in 0.. {
                            let time = start + grid_size * i as i64;
                            if time >= end {
                                break;
                            }

                            let note = Note {
                                time,
                                duration: grid_size,
                                ..note.clone()
                            };

                            selections.push(Rect {
                                left:   note.time,
                                top:    note.pitch - Interval::HALFSTEP * 0.5,
                                right:  note.time + note.duration,
                                bottom: note.pitch + Interval::HALFSTEP * 0.5,
                            });
                            score.parts[0].notes.push(note);
                        }
                    }
                    else {
                        selections.push(Rect {
                            left:   note.time,
                            top:    note.pitch - Interval::HALFSTEP * 0.5,
                            right:  note.time + note.duration,
                            bottom: note.pitch + Interval::HALFSTEP * 0.5,
                        });
                        score.parts[0].notes.push(note);
                    }
                }
            }

            // Handle selection
            if let Some(last_drawn) = last_drawn
                && ui.input(|i| i.pointer.secondary_released())
            {
                selections = score
                    .parts
                    .iter()
                    .flat_map(|p| p.notes.iter())
                    .filter(|n| {
                        let n_left = n.time;
                        let n_right = n.time + n.duration;
                        let n_top = n.pitch - Interval::HALFSTEP * 0.5;
                        let n_bottom = n.pitch + Interval::HALFSTEP * 0.5;

                        n_left < last_drawn.right
                            && n_right > last_drawn.left
                            && n_top < last_drawn.bottom
                            && n_bottom > last_drawn.top
                    })
                    .map(|n| {
                        let start = n.time;
                        let end = n.time + n.duration;

                        Rect {
                            left:   start,
                            right:  end,
                            top:    n.pitch - Interval::HALFSTEP * 0.5,
                            bottom: n.pitch + Interval::HALFSTEP * 0.5,
                        }
                    })
                    .collect();
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
                        .shrink(2.0),
                        0.0,
                        boomwhacker_color(note.pitch.chroma(), 255),
                    );
                }
            }

            // Paint selections
            for selection in selections.iter() {
                let selection = selection.remap(viewport, Rect::from_egui(rect)).to_egui();
                ui.painter().rect_stroke(
                    selection.shrink(1.0),
                    0.0,
                    (1.0, egui::Color32::WHITE),
                    egui::StrokeKind::Outside,
                );
            }

            // Paint highlighted note
            if let Some(pointer) = ui.pointer_latest_pos() {
                let pos = Vec2::from_egui(pointer.to_vec2()).remap(Rect::from_egui(rect), viewport);

                let mut start = pos.x;
                let mut end = pos.x;
                let mut pitch_start = pos.y.with_cents(0.0);
                let mut pitch_end = pos.y.with_cents(0.0);
                let mut stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);
                let mut fill = egui::Color32::TRANSPARENT;

                if ui.input(|i| i.pointer.primary_down())
                    && let Some(drag_start) = ui.input(|i| i.pointer.press_origin())
                {
                    let drag_start = Vec2::from_egui(drag_start.to_vec2())
                        .remap(Rect::from_egui(rect), viewport);
                    end = drag_start.x;
                    fill = boomwhacker_color(pitch_start.chroma(), 128);
                    stroke = egui::Stroke::NONE;
                }

                if ui.input(|i| i.pointer.secondary_down())
                    && let Some(drag_start) = ui.input(|i| i.pointer.press_origin())
                {
                    let drag_start = Vec2::from_egui(drag_start.to_vec2())
                        .remap(Rect::from_egui(rect), viewport);
                    end = drag_start.x;
                    pitch_end = drag_start.y;
                }

                if start > end {
                    std::mem::swap(&mut start, &mut end);
                }
                if pitch_start > pitch_end {
                    std::mem::swap(&mut pitch_start, &mut pitch_end);
                }

                let note_rect = Rect {
                    left:   start.floor(grid_size),
                    right:  end.ceil(grid_size),
                    top:    pitch_start - Interval::HALFSTEP * 0.5,
                    bottom: pitch_end + Interval::HALFSTEP * 0.5,
                };
                last_drawn = Some(note_rect);

                let note_rect = note_rect.remap(viewport, Rect::from_egui(rect)).to_egui();
                ui.painter()
                    .rect(note_rect, 0.0, fill, stroke, egui::StrokeKind::Outside);
                ui.painter().text(
                    note_rect.right_center(),
                    Align2::LEFT_CENTER,
                    duration_name(grid_size),
                    egui::FontId::default(),
                    egui::Color32::WHITE,
                );
            }
        },
    )
}

fn duration_name(duration: Duration) -> String {
    if duration == Duration::WHOLE {
        return "1".to_string();
    }
    let n = Duration::WHOLE / duration;
    format!("1/{}", n)
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
