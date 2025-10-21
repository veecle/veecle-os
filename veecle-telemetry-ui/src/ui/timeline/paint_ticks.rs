// Some of this is adapted from `re_time_panel`.
//
// Copyright (c) 2022 Rerun Technologies AB <opensource@rerun.io>
//
// Permission is hereby granted, free of charge, to any
// person obtaining a copy of this software and associated
// documentation files (the "Software"), to deal in the
// Software without restriction, including without
// limitation the rights to use, copy, modify, merge,
// publish, distribute, sublicense, and/or sell copies of
// the Software, and to permit persons to whom the Software
// is furnished to do so, subject to the following
// conditions:
//
// The above copyright notice and this permission notice
// shall be included in all copies or substantial portions
// of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
// ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
// TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
// PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
// SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
// CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
// OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
// IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

use std::ops::RangeInclusive;

use egui::{Align2, Color32, Rect, Rgba, Stroke, lerp, pos2, remap_clamp};

use crate::ui::timeline::TimeRangeUi;

pub fn paint_time_ranges_and_ticks(
    time_range_ui: &TimeRangeUi,
    ui: &egui::Ui,
    time_area_painter: &egui::Painter,
    line_y_range: RangeInclusive<f32>,
) {
    let clip_rect = ui.clip_rect();
    let clip_left = clip_rect.left() as f64;
    let clip_right = clip_rect.right() as f64;

    let mut x_range = time_range_ui.data.x.clone();

    let mut time_range = RangeInclusive::new(
        time_range_ui.data.time.start().as_ns(),
        time_range_ui.data.time.end().as_ns(),
    );

    // Clamp segment to the visible portion to save CPU when zoomed in:
    let left_t = egui::emath::inverse_lerp(x_range.clone(), clip_left).unwrap_or(0.5);
    if 0.0 < left_t && left_t < 1.0 {
        x_range = clip_left..=*x_range.end();
        time_range = RangeInclusive::new(lerp(time_range.clone(), left_t), *time_range.end());
    }
    let right_t = egui::emath::inverse_lerp(x_range.clone(), clip_right).unwrap_or(0.5);
    if 0.0 < right_t && right_t < 1.0 {
        x_range = *x_range.start()..=clip_right;
        time_range = RangeInclusive::new(*time_range.start(), lerp(time_range, right_t));
    }

    let x_range = (*x_range.start() as f32)..=(*x_range.end() as f32);
    let rect = Rect::from_x_y_ranges(x_range, line_y_range.clone());
    time_area_painter
        .with_clip_rect(rect)
        .extend(paint_time_range_ticks(ui, &rect, time_range));
}

fn paint_time_range_ticks(
    ui: &egui::Ui,
    rect: &Rect,
    time_range: RangeInclusive<f64>,
) -> Vec<egui::Shape> {
    let font_id = egui::TextStyle::Small.resolve(ui.style());

    fn next_power_of_10(i: i64) -> i64 {
        i * 10
    }

    paint_ticks(
        ui.ctx(),
        ui.visuals().dark_mode,
        &font_id,
        rect,
        &ui.clip_rect(),
        time_range,
        next_power_of_10,
        grid_text,
    )
}

#[allow(clippy::too_many_arguments)]
fn paint_ticks(
    egui_ctx: &egui::Context,
    dark_mode: bool,
    font_id: &egui::FontId,
    canvas: &Rect,
    clip_rect: &Rect,
    time_range: RangeInclusive<f64>,
    next_time_step: fn(i64) -> i64,
    format_tick: impl Fn(i64) -> String,
) -> Vec<egui::Shape> {
    let color_from_alpha = |alpha: f32| -> Color32 {
        if dark_mode {
            Rgba::from_white_alpha(alpha * alpha).into()
        } else {
            Rgba::from_black_alpha(alpha).into()
        }
    };

    let x_from_time = |time: i64| -> f32 {
        let t = (time as f64 - time_range.start()) / (time_range.end() - time_range.start());
        lerp(canvas.x_range(), t as f32)
    };

    let visible_rect = clip_rect.intersect(*canvas);
    let mut shapes = vec![];

    if !visible_rect.is_positive() {
        return shapes;
    }

    let width_time = (time_range.end() - time_range.start()) as f32;
    let points_per_time = canvas.width() / width_time;
    let minimum_small_line_spacing = 4.0;
    let expected_text_width = 60.0;

    let line_strength_from_spacing = |spacing_time: i64| -> f32 {
        let next_tick_magnitude = next_time_step(spacing_time) / spacing_time;
        remap_clamp(
            spacing_time as f32 * points_per_time,
            minimum_small_line_spacing..=(next_tick_magnitude as f32 * minimum_small_line_spacing),
            0.0..=1.0,
        )
    };

    let text_color_from_spacing = |spacing_time: i64| -> Color32 {
        let alpha = remap_clamp(
            spacing_time as f32 * points_per_time,
            expected_text_width..=(3.0 * expected_text_width),
            0.0..=0.5,
        );
        color_from_alpha(alpha)
    };

    let max_small_lines = canvas.width() / minimum_small_line_spacing;
    let mut small_spacing_time = 1;
    while width_time / (small_spacing_time as f32) > max_small_lines {
        small_spacing_time = next_time_step(small_spacing_time);
    }
    let medium_spacing_time = next_time_step(small_spacing_time);
    let big_spacing_time = next_time_step(medium_spacing_time);

    // We fade in lines as we zoom in:
    let big_line_strength = line_strength_from_spacing(big_spacing_time);
    let medium_line_strength = line_strength_from_spacing(medium_spacing_time);
    let small_line_strength = line_strength_from_spacing(small_spacing_time);

    let big_line_color = color_from_alpha(0.4 * big_line_strength);
    let medium_line_color = color_from_alpha(0.4 * medium_line_strength);
    let small_line_color = color_from_alpha(0.4 * small_line_strength);

    let big_text_color = text_color_from_spacing(big_spacing_time);
    let medium_text_color = text_color_from_spacing(medium_spacing_time);
    let small_text_color = text_color_from_spacing(small_spacing_time);

    let mut current_time =
        (time_range.start().floor() as i64) / small_spacing_time * small_spacing_time;

    let end_time = (time_range.end().ceil() as i64).saturating_add(1);
    while current_time < end_time {
        let line_x = x_from_time(current_time);

        if visible_rect.min.x <= line_x && line_x <= visible_rect.max.x {
            let medium_line = current_time % medium_spacing_time == 0;
            let big_line = current_time % big_spacing_time == 0;

            let (height_factor, line_color, text_color) = if big_line {
                (medium_line_strength, big_line_color, big_text_color)
            } else if medium_line {
                (small_line_strength, medium_line_color, medium_text_color)
            } else {
                (0.0, small_line_color, small_text_color)
            };

            // Make line higher if it is stronger:
            let line_top = lerp(canvas.y_range(), lerp(0.75..=0.5, height_factor));

            shapes.push(egui::Shape::line_segment(
                [pos2(line_x, line_top), pos2(line_x, canvas.max.y)],
                Stroke::new(1.0, line_color),
            ));

            if text_color != Color32::TRANSPARENT {
                let text = format_tick(current_time);
                let text_x = line_x + 4.0;

                egui_ctx.fonts_mut(|fonts| {
                    shapes.push(egui::Shape::text(
                        fonts,
                        pos2(text_x, lerp(canvas.y_range(), 0.5)),
                        Align2::LEFT_CENTER,
                        &text,
                        font_id.clone(),
                        text_color,
                    ));
                });
            }
        }

        current_time = current_time.saturating_add(small_spacing_time);
    }

    shapes
}

fn grid_text(grid_ns: i64) -> String {
    let grid_ms = grid_ns as f64 * 1e-6;
    if grid_ns % 1_000_000 == 0 {
        format!("{grid_ms:.0} ms")
    } else if grid_ns % 100_000 == 0 {
        format!("{grid_ms:.1} ms")
    } else if grid_ns % 10_000 == 0 {
        format!("{grid_ms:.2} ms")
    } else {
        format!("{grid_ms:.3} ms")
    }
}
