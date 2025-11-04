// Some of this is adapted from `puffin_egui`.
//
// Copyright (c) 2019 Embark Studios
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

use std::collections::HashMap;

use egui::emath::GuiRounding;
use egui::{
    Align2, Color32, FontId, LayerId, Painter, Pos2, Rect, Response, Rgba, Shape, Stroke, pos2,
    remap_clamp,
};
use veecle_telemetry::SpanContext;

use crate::selection::SelectionState;
use crate::store::{LogRef, SpanRef, Store, Timestamp};
use crate::ui::timeline::TimeRangeUi;

pub const RECT_HEIGHT: f32 = 20.0;
pub const RECT_MARGIN: f32 = 1.0;
pub const RECT_SPACING: f32 = 0.0;
pub const RECT_ROUNDING: f32 = 2.0;
pub const RECT_MIN_WIDTH: f32 = 1.0;

#[derive(Debug, Default)]
pub struct SpanUiMetadata {
    positions: HashMap<SpanContext, Pos2>,
    links: Vec<(SpanContext, Pos2)>,
}

pub fn links_ui(
    time_range_ui: &TimeRangeUi,
    painter: &Painter,
    store: &Store,
    metadata: SpanUiMetadata,
) {
    let SpanUiMetadata {
        positions: coordinates,
        links,
    } = metadata;

    let pairs = links
        .into_iter()
        .filter_map(|(linked_id, target_pos)| {
            Some((
                find_source_position(time_range_ui, store, &coordinates, linked_id)?,
                target_pos,
            ))
        })
        .collect::<Vec<_>>();

    for (source_pos, target_pos) in pairs {
        let span = target_pos.x - source_pos.x;
        assert!(span >= 0.0, "target should not be before the source");

        let mid_offset = ARROW_LENGTH.min((span - ARROW_LENGTH).max(0.0));
        let mid_x = source_pos.x + mid_offset;

        // o---
        //    |
        //    --->

        paint_arrow(
            painter,
            &[
                source_pos,
                pos2(mid_x, source_pos.y),
                pos2(mid_x, target_pos.y),
                target_pos,
            ],
        );
    }
}

const ARROW_LENGTH: f32 = 15.0;

fn paint_arrow(painter: &Painter, positions: &[Pos2]) {
    assert!(
        positions.len() >= 2,
        "can't draw an arrow without at least 2 points"
    );

    let color = HOVER_COLOR;
    let stroke = Stroke::new(1.0, color);

    let max_arrow_len = ARROW_LENGTH;

    let iter = positions.windows(2);
    let length = iter.len();

    for (i, points) in iter.enumerate() {
        let from = points[0];
        let to = points[1];

        if i == 0 {
            painter.circle_filled(from, 2.0, color);
        }

        if i < length - 1 {
            painter.line_segment([from, to], stroke);
            continue;
        }

        let vec = to - from;
        let distance = vec.length();
        let arrow_length = distance.min(max_arrow_len);

        if distance <= max_arrow_len {
            painter.arrow(from, vec, stroke);
        } else {
            let arrow_length_fraction = arrow_length / distance;
            let line_length_fraction = 1.0 - arrow_length_fraction;

            painter.line_segment([from, from.lerp(to, line_length_fraction)], stroke);
            painter.arrow(
                from.lerp(to, line_length_fraction),
                vec * arrow_length_fraction,
                stroke,
            );
        }
    }
}

fn find_source_position(
    time_range_ui: &TimeRangeUi,
    store: &Store,
    coordinates: &HashMap<SpanContext, Pos2>,
    span_context: SpanContext,
) -> Option<Pos2> {
    if let Some(pos) = coordinates.get(&span_context) {
        return Some(*pos);
    }

    let span = store
        .get_span(span_context)
        .expect("span has to exist in the store");

    let x = time_range_ui.x_from_time_f32(span.end);

    span.parent
        .and_then(|id| find_source_position(time_range_ui, store, coordinates, id))
        .map(|parent_pos| pos2(x, parent_pos.y))
}

#[allow(clippy::too_many_arguments)]
pub fn traces_ui(
    ui: &mut egui::Ui,
    selection_state: &SelectionState,
    time_range_ui: &TimeRangeUi,
    traces: &[SpanRef],
    max_depth: usize,
    rect: Rect,
    response: &Response,
    painter: &Painter,
    metadata: &mut SpanUiMetadata,
) {
    let info = Info {
        ctx: ui.ctx(),
        canvas: rect,
        response,
        painter,
        text_height: 15.0,
        layer_id: ui.layer_id(),
        font_id: egui::TextStyle::Body.resolve(ui.style()),
    };

    let min_y = info.canvas.top() + RECT_SPACING;

    for span in traces {
        paint_scope(
            selection_state,
            time_range_ui,
            metadata,
            &info,
            *span,
            0,
            max_depth,
            min_y,
        );
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum PaintResult {
    Culled,
    Normal,
}

struct Info<'a> {
    ctx: &'a egui::Context,
    /// Bounding box of canvas in points:
    canvas: Rect,
    /// Interaction with the profiler canvas
    response: &'a Response,
    painter: &'a Painter,
    text_height: f32,
    /// LayerId to use as parent for tooltips
    layer_id: LayerId,

    font_id: FontId,
}

#[allow(clippy::too_many_arguments)]
fn paint_scope(
    selection_state: &SelectionState,
    time_range_ui: &TimeRangeUi,
    metadata: &mut SpanUiMetadata,
    info: &Info,
    span: SpanRef,
    depth: usize,
    max_depth: usize,
    min_y: f32,
) -> PaintResult {
    if depth >= max_depth {
        return PaintResult::Culled;
    }

    let top_y = min_y + (depth as f32) * (RECT_HEIGHT + RECT_SPACING);

    // display log markers of spans we are not displaying
    let show_child_logs = depth + 1 == max_depth;

    let result = paint_record(
        selection_state,
        time_range_ui,
        metadata,
        info,
        "",
        "",
        span,
        top_y,
        show_child_logs,
    );

    if result != PaintResult::Culled {
        for child_scope in span.children() {
            paint_scope(
                selection_state,
                time_range_ui,
                metadata,
                info,
                child_scope,
                depth + 1,
                max_depth,
                min_y,
            );
        }

        if selection_state.is_span_hovered(span) {
            egui::Tooltip::always_open(
                info.ctx.clone(),
                info.layer_id,
                egui::Id::new("veecle_os_span_tooltip"),
                egui::PopupAnchor::Pointer,
            )
            .show(|ui| {
                paint_scope_details(ui, span, time_range_ui.data.end());
            });
        }
    }

    result
}

fn paint_scope_details(ui: &mut egui::Ui, span: SpanRef, max: Timestamp) {
    egui::Grid::new("scope_details_tooltip")
        .num_columns(2)
        .show(ui, |ui| {
            ui.monospace("id");
            ui.monospace(format!("{}", span.context));
            ui.end_row();

            ui.monospace("operation name");
            ui.monospace(span.metadata.name.as_str());
            ui.end_row();

            ui.monospace("duration");
            if span.end == Timestamp::MAX {
                ui.monospace(format!("{:.3}+ ms", (max - span.start).as_ms()));
            } else {
                ui.monospace(format!("{:.3} ms", span.duration_ms()));
            }
            ui.end_row();

            if let Some(file) = &span.metadata.file {
                ui.monospace("file");
                ui.monospace(file.as_str());
                ui.end_row();
            }

            ui.monospace("target");
            ui.monospace(span.metadata.target.as_str());
            ui.end_row();

            ui.monospace("children");
            ui.monospace(span.children.len().to_string());
            ui.end_row();
        });
}

const SELECTED_COLOR: Rgba = Rgba::from_rgb(0.9, 0.9, 0.9);
const HOVER_COLOR: Rgba = Rgba::from_rgb(0.7, 0.7, 0.7);
const INACTIVE_COLOR: Rgba = Rgba::from_rgb(0.4, 0.4, 0.4);

#[allow(clippy::too_many_arguments)]
fn paint_record(
    selection_state: &SelectionState,
    time_range_ui: &TimeRangeUi,
    metadata: &mut SpanUiMetadata,
    info: &Info,
    prefix: &str,
    suffix: &str,
    span: SpanRef,
    top_y: f32,
    show_child_logs: bool,
) -> PaintResult {
    let start_x = time_range_ui.x_from_time_f32(span.start);
    let stop_x = time_range_ui.x_from_time_f32(span.end.min(time_range_ui.data.end()));

    if info.canvas.max.x < start_x || stop_x < info.canvas.min.x {
        return PaintResult::Culled;
    }

    let bottom_y = top_y + RECT_HEIGHT;

    let middle_y = top_y + RECT_HEIGHT / 2.0;

    metadata
        .positions
        .insert(span.context, pos2(stop_x, middle_y));

    for link in &span.links {
        metadata.links.push((*link, pos2(start_x, middle_y)));
    }

    let rect = Rect::from_min_max(pos2(start_x, top_y), pos2(stop_x, bottom_y));

    let is_hovered = match info.response.hover_pos() {
        Some(hover_pos) => rect.contains(hover_pos),
        None => false,
    };
    let is_clicked = is_hovered && info.response.clicked();

    if is_hovered {
        selection_state.set_hovered(span.context.into());
    }

    let is_selected = selection_state.is_selected(span.context.into());

    if is_clicked {
        if is_selected {
            selection_state.clear_selected();
        } else {
            selection_state.set_selected(span.context.into());
        }
    }

    let is_hovered = is_hovered || selection_state.is_span_hovered(span);

    let rect_color = if is_selected {
        SELECTED_COLOR
    } else if is_hovered {
        HOVER_COLOR
    } else {
        INACTIVE_COLOR
    };

    let min_width = RECT_MIN_WIDTH;

    let top_y_margin = top_y + RECT_MARGIN;
    let bottom_y_margin = bottom_y - RECT_MARGIN;
    let draw_rect = Rect::from_min_max(pos2(start_x, top_y_margin), pos2(stop_x, bottom_y_margin));

    if rect.width() <= min_width {
        // faster to draw it as a thin line
        info.painter.line_segment(
            [draw_rect.center_top(), draw_rect.center_bottom()],
            egui::Stroke::new(min_width, rect_color),
        );
    } else {
        info.painter
            .rect_filled(draw_rect, RECT_ROUNDING, rect_color);

        for activity in &span.activity {
            let start_x = time_range_ui.x_from_time_f32(activity.start);
            let stop_x = time_range_ui.x_from_time_f32(activity.end.min(time_range_ui.data.end()));

            let rect =
                Rect::from_min_max(pos2(start_x, top_y_margin), pos2(stop_x, bottom_y_margin));

            let rect_color =
                color_from_duration(span.duration()).multiply(if is_hovered { 0.8 } else { 1.0 });

            info.painter.rect_filled(rect, RECT_ROUNDING, rect_color);
        }
    }

    for_each_log(span, show_child_logs, &mut |log| {
        paint_log_triangle(info, time_range_ui, log, rect_color, bottom_y_margin);
    });

    let wide_enough_for_text = stop_x - start_x > 32.0;
    if wide_enough_for_text {
        let painter = info.painter.with_clip_rect(rect.intersect(info.canvas));

        let scope_name = &span.metadata.name;

        let text = if span.end == Timestamp::MAX {
            let duration_ms = (time_range_ui.data.end() - span.start).as_ms();
            format!(
                "{}{} {:6.3}+ ms {}",
                prefix,
                scope_name.as_str(),
                duration_ms,
                suffix
            )
        } else {
            let duration_ms = span.duration_ms();
            format!(
                "{}{} {:6.3} ms {}",
                prefix,
                scope_name.as_str(),
                duration_ms,
                suffix
            )
        };
        let pos = pos2(
            start_x + 4.0,
            top_y + 0.5 * (RECT_HEIGHT - info.text_height),
        );
        let pos = pos.round_to_pixels(painter.pixels_per_point());
        const TEXT_COLOR: Color32 = Color32::BLACK;
        painter.text(
            pos,
            Align2::LEFT_TOP,
            text,
            info.font_id.clone(),
            TEXT_COLOR,
        );
    }

    PaintResult::Normal
}

fn for_each_log(span: SpanRef, recursive: bool, f: &mut impl FnMut(LogRef)) {
    for log in span.logs() {
        f(log)
    }

    if recursive {
        for child in span.children() {
            for_each_log(child, recursive, f);
        }
    }
}

fn paint_log_triangle(
    info: &Info,
    time_range_ui: &TimeRangeUi,
    log: LogRef,
    color: impl Into<Color32>,
    y: f32,
) {
    let x = time_range_ui.x_from_time_f32(log.timestamp);

    let w = 5.0;
    let h = 10.0;

    let y_min = y;
    let y_max = y + h;

    let triangle = vec![
        pos2(x, y_min),           // top
        pos2(x - 0.5 * w, y_max), // bottom left
        pos2(x + 0.5 * w, y_max), // bottom right
    ];
    info.painter
        .add(Shape::convex_polygon(triangle, color, Stroke::NONE));
}

fn color_from_duration(ns: Timestamp) -> Rgba {
    let ms = ns.as_ms() as f32;
    // Brighter = more time.
    // So we start with dark colors (blue) and later bright colors (green).
    let b = remap_clamp(ms, 0.0..=5.0, 1.0..=0.3);
    let r = remap_clamp(ms, 0.0..=10.0, 0.5..=0.8);
    let g = remap_clamp(ms, 10.0..=33.0, 0.1..=0.8);
    let a = 0.9;
    Rgba::from_rgb(r, g, b) * a
}
