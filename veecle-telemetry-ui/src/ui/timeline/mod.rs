//! Components which display span data.
//!
//! See [`TraceTimelinePanel`].

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

use std::collections::HashSet;
use std::ops::RangeInclusive;

use eframe::epaint::Color32;
use egui::containers::scroll_area::ScrollSource;
use egui::emath::Rot2;
use egui::{CornerRadius, CursorIcon, NumExt, PointerButton, Rangef, Rect, Shape, pos2, remap};
use indexmap::IndexMap;

use crate::selection::SelectionState;
use crate::state::AppState;
use crate::store::{SpanRef, Store, Timestamp, TimestampF};
use crate::ui::timeline::traces::{RECT_HEIGHT, SpanUiMetadata, links_ui, traces_ui};

mod paint_ticks;
mod traces;

/// Panel which displays spans grouped by actors.
///
/// Spans get rendered similar to a flame chart.
#[derive(Debug)]
pub struct TraceTimelinePanel {
    /// Width of the entity name columns previous frame.
    prev_col_width: f32,

    /// The right side of the entity name column; updated during its painting.
    next_col_right: f32,

    /// The time axis view, regenerated each frame.
    time_range_ui: TimeRangeUi,

    time_view: Option<TimeView>,

    collapsed_actors: HashSet<String>,
}

impl Default for TraceTimelinePanel {
    fn default() -> Self {
        Self {
            prev_col_width: 400.0,
            next_col_right: 0.0,
            time_range_ui: Default::default(),
            time_view: None,
            collapsed_actors: Default::default(),
        }
    }
}

#[derive(Copy, Clone, Debug)]
struct TimeView {
    /// Start of the time range.
    pub start: TimestampF,

    /// Duration of the time range.
    ///
    /// `end = min + duration`
    pub duration: TimestampF,
}

/// Time range and corresponding screen coordinates for all data in the store.
#[derive(Clone, Debug)]
struct DataTimeRange {
    /// The range on the x-axis in the UI, in screen coordinates.
    ///
    /// Matches [`Self::time`] (linear transform).
    ///
    /// Uses `f64` because the ends of this range can be way outside the screen
    /// when we are very zoomed in.
    pub x: RangeInclusive<f64>,

    /// Matches [`Self::x`] (linear transform).
    pub time: RangeInclusive<TimestampF>,
}

impl DataTimeRange {
    /// Returns the start timestamp of data in the store.
    fn start(&self) -> Timestamp {
        Timestamp::from(*self.time.start())
    }

    /// Returns the end timestamp of data in the store.
    fn end(&self) -> Timestamp {
        Timestamp::from(*self.time.end())
    }
}

#[derive(Debug)]
struct TimeRangeUi {
    /// The total UI x-range we're viewing.
    x_range: RangeInclusive<f64>,

    /// The range of time we're viewing.
    time_view: TimeView,

    /// The total range of time including screen space coordinates.
    data: DataTimeRange,

    /// x distance per time unit.
    points_per_time: f64,
}

impl Default for TimeRangeUi {
    /// Safe, meaningless default
    fn default() -> Self {
        Self {
            x_range: 0.0..=1.0,
            time_view: TimeView {
                start: TimestampF::from_ns(0.0),
                duration: TimestampF::from_ns(1.0),
            },
            data: DataTimeRange {
                x: 0.0..=1.0,
                time: TimestampF::from(Timestamp::MIN)..=TimestampF::from(Timestamp::MAX),
            },
            points_per_time: 1.0,
        }
    }
}

// 100ns
const MIN_ZOOM_DURATION_NS: f64 = 100.0;
// 1h
const MAX_ZOOM_DURATION_NS: f64 = 60.0 * 60.0 * 1000.0 * 1000.0 * 1000.0;

impl TimeRangeUi {
    fn new(x_range: Rangef, time_view: TimeView, min: Timestamp, max: Timestamp) -> Self {
        //      <------- time_view ------>
        //      <-------- x_range ------->
        //      |                        |
        // [              data                ]

        let x_range = (x_range.min as f64)..=(x_range.max as f64);
        let width_in_ui = *x_range.end() - *x_range.start();
        let points_per_time = width_in_ui / time_view.duration.as_ns();
        let points_per_time = if points_per_time > 0.0 && points_per_time.is_finite() {
            points_per_time
        } else {
            1.0
        };

        let data_time = {
            let range_width = TimestampF::from(max - min).as_ns() * points_per_time;

            let data_x_range = 0.0..=range_width;

            DataTimeRange {
                x: data_x_range,
                time: TimestampF::from(min)..=TimestampF::from(max),
            }
        };

        let mut slf = Self {
            x_range,
            time_view,
            data: data_time,
            points_per_time,
        };

        let time_start_x = slf.x_from_time_f64(*slf.data.time.start());
        slf.data.x = (*slf.data.x.start() + time_start_x)..=(*slf.data.x.end() + time_start_x);

        slf
    }

    fn x_from_time_f32(&self, t: Timestamp) -> f32 {
        self.x_from_time_f64(TimestampF::from(t)) as f32
    }

    fn x_from_time_f64(&self, t: TimestampF) -> f64 {
        self.x_range.start() + ((t - self.time_view.start).as_ns() * self.points_per_time)
    }

    fn time_from_x(&self, x: f64) -> TimestampF {
        let start_x = *self.x_range.start();
        let start_time = self.time_view.start;

        start_time + TimestampF::from_ns((x - start_x) / self.points_per_time)
    }

    /// Pan the view, returning the new view.
    fn pan(&self, delta_x: f32) -> TimeView {
        TimeView {
            start: self.time_from_x(*self.x_range.start() + delta_x as f64),
            duration: self.time_view.duration,
        }
    }

    /// Zoom the view around the given x, returning the new view.
    fn zoom_at(&self, x: f32, zoom_factor: f32) -> TimeView {
        let min_zoom_factor = self.time_view.duration.as_ns() / MIN_ZOOM_DURATION_NS;
        let max_zoom_factor = self.time_view.duration.as_ns() / MAX_ZOOM_DURATION_NS;

        let zoom_factor = (zoom_factor as f64)
            .min(min_zoom_factor)
            .max(max_zoom_factor);

        let mut min_x = *self.x_range.start();
        let max_x = *self.x_range.end();
        let t = remap(x as f64, min_x..=max_x, 0.0..=1.0);

        let width = max_x - min_x;

        let new_width = width / zoom_factor;
        let width_delta = new_width - width;

        min_x -= t * width_delta;

        TimeView {
            start: self.time_from_x(min_x),
            duration: TimestampF::from_ns(self.time_view.duration.as_ns() / zoom_factor),
        }
    }
}

impl TraceTimelinePanel {
    /// Show the timeline panel.
    pub fn show(&mut self, ui: &mut egui::Ui, store: &Store, app_state: &AppState) {
        let window_height = ui.ctx().content_rect().height();

        let min_height = 150.0;
        let min_top_space = 150.0;
        let panel = egui::TopBottomPanel::bottom("timeline_panel")
            .resizable(true)
            .min_height(min_height)
            .max_height((window_height - min_top_space).max(min_height).round())
            .default_height((0.33 * window_height).clamp(min_height, 1024.0).round());

        panel.show_inside(ui, |ui| {
            egui::Frame::default().show(ui, |ui| {
                self.frame_ui(ui, store, app_state);
            });
        });
    }

    fn frame_ui(&mut self, ui: &mut egui::Ui, store: &Store, app_state: &AppState) {
        //               |timeline            |
        // ------------------------------------
        // tree          |streams             |
        //               |  . .   .   . . .   |
        //               |            . . . . |
        //               ▲
        //               └ tree_max_y (= time_x_left)

        self.next_col_right = ui.min_rect().left(); // `next_col_right` will expand during the call

        let time_x_left =
            (ui.min_rect().left() + self.prev_col_width + ui.spacing().item_spacing.x)
                .at_most(ui.max_rect().right() - 100.0)
                .at_least(80.); // cover the empty recording case

        // Where the time will be shown.
        let time_bg_x_range = Rangef::new(time_x_left, ui.max_rect().right());
        let time_fg_x_range = {
            // Painting to the right of the scroll bar (if any) looks bad:
            let right = ui.max_rect().right() - ui.spacing_mut().scroll.bar_outer_margin;
            debug_assert!(time_x_left < right);
            Rangef::new(time_x_left, right)
        };

        let side_margin = 16.0; // don't zoom edge to edge by default, leave a small gap.

        self.time_range_ui = initialize_time_range_ui(
            store,
            Rangef::new(
                time_fg_x_range.min + side_margin,
                time_fg_x_range.max - side_margin,
            ),
            self.time_view,
        );

        if store.continuous {
            // Showing a continuously updating view of "now", continuously render to be smooth
            ui.ctx().request_repaint();
        }

        let full_y_range = Rangef::new(ui.min_rect().bottom(), ui.max_rect().bottom());

        let timeline_rect = {
            let top = ui.min_rect().bottom();

            let size = egui::vec2(self.prev_col_width, 28.0);
            ui.allocate_ui_with_layout(size, egui::Layout::top_down(egui::Align::LEFT), |ui| {
                ui.set_min_size(size);
                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                ui.add_space(5.0); // hack to vertically center the text
                ui.strong("Actors");
            });

            let bottom = ui.min_rect().bottom();
            Rect::from_x_y_ranges(time_fg_x_range, top..=bottom)
        };

        let streams_rect = Rect::from_x_y_ranges(
            time_fg_x_range,
            timeline_rect.bottom()..=ui.max_rect().bottom(),
        );

        // includes the timeline and streams areas.
        let time_bg_area_rect = Rect::from_x_y_ranges(time_bg_x_range, full_y_range);
        let time_bg_area_highlighted_rect = time_bg_area_rect
            .with_min_x(
                self.time_range_ui
                    .x_from_time_f32(self.time_range_ui.data.start()),
            )
            .with_max_x(
                self.time_range_ui
                    .x_from_time_f32(self.time_range_ui.data.end()),
            );
        let time_fg_area_rect = Rect::from_x_y_ranges(time_fg_x_range, full_y_range);
        let time_bg_area_painter = ui.painter().with_clip_rect(time_bg_area_rect);
        let time_area_painter = ui.painter().with_clip_rect(time_fg_area_rect);

        let mid_color = ui
            .visuals()
            .extreme_bg_color
            .lerp_to_gamma(ui.visuals().code_bg_color, 0.2);
        time_bg_area_painter.rect_filled(
            time_bg_area_rect,
            CornerRadius::default(),
            ui.visuals().extreme_bg_color,
        );
        time_bg_area_painter.rect_filled(
            time_bg_area_highlighted_rect,
            CornerRadius::default(),
            mid_color,
        );

        ui.painter().hline(
            0.0..=ui.max_rect().right(),
            timeline_rect.bottom(),
            ui.visuals().widgets.noninteractive.bg_stroke,
        );

        paint_ticks::paint_time_ranges_and_ticks(
            &self.time_range_ui,
            ui,
            &time_area_painter,
            timeline_rect.top()..=timeline_rect.bottom(),
        );

        let time_area_response = interact_with_streams_rect(
            &self.time_range_ui,
            &mut self.time_view, // time_ctrl,
            ui,
            &time_bg_area_rect,
            &streams_rect,
        );

        // Don't draw on top of the time ticks
        let lower_time_area_painter = ui.painter().with_clip_rect(Rect::from_x_y_ranges(
            time_fg_x_range,
            ui.min_rect().bottom()..=ui.max_rect().bottom(),
        ));

        // All the entity rows and their data density graphs
        self.actors_ui(
            store,
            &time_area_response,
            &lower_time_area_painter,
            app_state,
            ui,
        );

        {
            // Paint a shadow between the stream names on the left
            // and the data on the right:
            let shadow_width = 30.0;

            // In the design the shadow starts under the time markers.
            // let shadow_y_start =
            //    timeline_rect.bottom() + ui.visuals().widgets.noninteractive.bg_stroke.width;
            // This looks great but only if there are still time markers.
            // When they move to the right (or have a cut) one expects the shadow to go all the way up.
            // But that's quite complicated so let's have the shadow all the way
            let shadow_y_start = full_y_range.min;

            let shadow_y_end = full_y_range.max;
            let rect = egui::Rect::from_x_y_ranges(
                time_x_left..=(time_x_left + shadow_width),
                shadow_y_start..=shadow_y_end,
            );
            draw_shadow_line(ui, rect, egui::Direction::LeftToRight);
        }

        // remember where to show the time for next frame:
        self.prev_col_width = self.next_col_right - ui.min_rect().left();
    }

    fn actors_ui(
        &mut self,
        store: &Store,
        time_area_response: &egui::Response,
        time_area_painter: &egui::Painter,
        app_state: &AppState,
        ui: &mut egui::Ui,
    ) {
        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            // We turn off `ScrollSource::DRAG` so that the `ScrollArea` don't steal input from
            // the earlier `interact_with_time_area`.
            // We implement drag-to-scroll manually instead!
            .scroll_source(ScrollSource::SCROLL_BAR | ScrollSource::MOUSE_WHEEL)
            .show(ui, |ui| {
                // we don't want any spacing between items
                ui.style_mut().spacing.item_spacing.y = 0.0;

                let mut span_metadata = SpanUiMetadata::default();

                let data = convert(app_state, store);
                for (actor_name, data) in &data.actors {
                    self.show_collapsing_actor(
                        time_area_response,
                        time_area_painter,
                        actor_name,
                        data,
                        app_state.selection(),
                        &mut span_metadata,
                        ui,
                    );
                }

                links_ui(&self.time_range_ui, time_area_painter, store, span_metadata);
            });
    }

    #[allow(clippy::too_many_arguments)]
    fn show_collapsing_actor(
        &mut self,
        time_area_response: &egui::Response,
        time_area_painter: &egui::Painter,
        actor_name: &String,
        actor: &Vec<ActorSpanRow>,
        selection_state: &SelectionState,
        span_metadata: &mut SpanUiMetadata,
        ui: &mut egui::Ui,
    ) {
        let is_expanded = !self.collapsed_actors.contains(actor_name);

        let id = ui.make_persistent_id(actor_name);
        let openness = ui.ctx().animate_bool_responsive(id, is_expanded);

        let button_padding = ui.spacing().button_padding;

        let available = ui.available_rect_before_wrap();

        // leave space for arrow
        let align = 16.0;

        let text_pos = available.min + egui::vec2(align, 0.0);
        let wrap_width = available.right() - text_pos.x;

        let galley = egui::WidgetText::from(actor_name).into_galley(
            ui,
            Some(egui::TextWrapMode::Extend),
            wrap_width,
            egui::TextStyle::Monospace,
        );
        let text_max_x = text_pos.x + galley.size().x;

        let mut desired_width = text_max_x + button_padding.x - available.left();
        if ui.visuals().collapsing_header_frame {
            desired_width = desired_width.max(available.width()); // fill full width
        }

        let desired_height = RECT_HEIGHT;

        let desired_size = egui::vec2(desired_width, desired_height);
        let (_, rect) = ui.allocate_space(desired_size);

        let mut header_response = ui.interact(rect, id, egui::Sense::click());
        let text_pos = pos2(
            text_pos.x,
            header_response.rect.center().y - galley.size().y / 2.0,
        );

        if header_response.clicked() {
            if is_expanded {
                self.collapsed_actors.insert(actor_name.clone());
            } else {
                self.collapsed_actors.remove(actor_name);
            }

            header_response.mark_changed();
        }

        if ui.is_rect_visible(rect) {
            let visuals = ui.style().interact(&header_response);

            {
                let space_around_icon = 1.0;
                let icon_width = ui.spacing().icon_width_inner;

                let icon_rect = egui::Rect::from_center_size(
                    header_response.rect.left_center()
                        + egui::vec2(space_around_icon + icon_width / 2.0, 0.0),
                    egui::Vec2::splat(icon_width),
                );

                let icon_response = header_response.clone().with_new_rect(icon_rect);
                paint_collapsing_triangle(
                    ui,
                    openness,
                    icon_rect.center(),
                    ui.style().interact(&icon_response),
                );
            }

            ui.painter().galley(text_pos, galley, visuals.text_color());
        }

        let mut top = rect.top();
        let mut allocated_height = rect.height();

        for (index, actor_row) in actor.iter().enumerate() {
            let desired_body_height =
                RECT_HEIGHT * ((actor_row.max_depth as f32 * openness).max(1.0) + 0.5);

            let bottom = if desired_body_height > allocated_height {
                let desired_body_size = egui::vec2(
                    desired_width,
                    f32::max(0.0, desired_body_height - allocated_height),
                );
                allocated_height = 0.0;
                let (_, body_rect) = ui.allocate_space(desired_body_size);
                body_rect.bottom()
            } else {
                allocated_height -= desired_body_height;

                top + desired_body_height
            };

            // Show the spans in the timeline area.
            let traces_rect =
                Rect::from_x_y_ranges(time_area_response.rect.x_range(), Rangef::new(top, bottom));

            let max_depth = (traces_rect.y_range().span() / RECT_HEIGHT)
                .floor()
                .max(1.0) as usize;

            // Draw separator line first so log triangles can draw on top.
            let line_stroke = ui.visuals().widgets.noninteractive.bg_stroke;
            // Centered in the gap between spans (`0.5 * RECT_HEIGHT`).
            let line_y = bottom - RECT_HEIGHT / 4.0;

            let not_last = index < actor.len() - 1;
            if not_last {
                let path = &[
                    pos2(traces_rect.x_range().min, line_y),
                    pos2(traces_rect.x_range().max, line_y),
                ];

                ui.painter()
                    .add(Shape::dashed_line(path, line_stroke, 4.0, 8.0));
            } else {
                let path = vec![
                    pos2(available.min.x, line_y),
                    pos2(traces_rect.x_range().max, line_y),
                ];

                ui.painter().add(Shape::line(path, line_stroke));
            }

            // Draw spans including log triangles.
            traces_ui(
                ui,
                selection_state,
                &self.time_range_ui,
                &actor_row.spans,
                max_depth,
                traces_rect,
                time_area_response,
                time_area_painter,
                span_metadata,
            );

            top = bottom;
        }

        let response_rect = header_response.rect;

        self.next_col_right = self.next_col_right.max(response_rect.right());
    }
}

fn initialize_time_range_ui(
    store: &Store,
    time_x_range: Rangef,
    time_view: Option<TimeView>,
) -> TimeRangeUi {
    // Handle the case where no data has been loaded yet.
    if store.start == Timestamp::MAX || store.end == Timestamp::MIN {
        let default_start = Timestamp::from_ns(0);
        let default_end = Timestamp::from_ns(1_000_000_000); // 1 second default range

        let time_view = time_view.unwrap_or_else(|| TimeView {
            start: TimestampF::from(default_start),
            duration: TimestampF::from(default_end - default_start),
        });

        return TimeRangeUi::new(time_x_range, time_view, default_start, default_end);
    }

    let max = if store.continuous {
        store.end + Timestamp::from_ns(store.last_update.elapsed().as_nanos() as i64)
    } else {
        store.end
    };

    let time_view = time_view.unwrap_or_else(|| TimeView {
        start: TimestampF::from(store.start),
        duration: TimestampF::from_ns((max.as_ns() as f64) - (store.start.as_ns() as f64)),
    });

    TimeRangeUi::new(time_x_range, time_view, store.start, max)
}

/// Returns a scroll delta
#[must_use]
fn interact_with_streams_rect(
    time_range_ui: &TimeRangeUi,
    active_time_view: &mut Option<TimeView>,
    ui: &egui::Ui,
    full_rect: &Rect,
    streams_rect: &Rect,
) -> egui::Response {
    let pointer_pos = ui.input(|i| i.pointer.hover_pos());

    let mut delta_x = 0.0;
    let mut zoom_factor = 1.0;

    // Check for zoom/pan inputs (via e.g. horizontal scrolling) on the entire
    // time area rectangle, including the timeline rect.
    let full_rect_hovered = pointer_pos.is_some_and(|pointer_pos| full_rect.contains(pointer_pos));
    if full_rect_hovered {
        ui.input(|input| {
            delta_x += input.smooth_scroll_delta.x;
            zoom_factor *= input.zoom_delta_2d().x;
        });
    }

    // We only check for drags in the streams rect,
    // because drags in the timeline rect should move the time
    // (or create loop sections).
    let response = ui.interact(
        *streams_rect,
        ui.id().with("time_area_interact"),
        egui::Sense::click_and_drag(),
    );
    if response.dragged_by(PointerButton::Primary) {
        delta_x += response.drag_delta().x;
        ui.ctx().set_cursor_icon(CursorIcon::AllScroll);
    }
    if response.dragged_by(PointerButton::Secondary) {
        zoom_factor *= (response.drag_delta().y * 0.01).exp();
    }

    if delta_x != 0.0 {
        let new_view_range = time_range_ui.pan(-delta_x);
        *active_time_view = Some(new_view_range);
    }

    if zoom_factor != 1.0
        && let Some(pointer_pos) = pointer_pos
    {
        let new_view_range = time_range_ui.zoom_at(pointer_pos.x, zoom_factor);
        *active_time_view = Some(new_view_range);
    }

    if response.double_clicked() {
        *active_time_view = None;
    }

    response
}

fn paint_collapsing_triangle(
    ui: &egui::Ui,
    openness: f32,
    center: egui::Pos2,
    visuals: &egui::style::WidgetVisuals,
) {
    // This value is hard coded because, from a UI perspective, the size of the triangle is
    // given and fixed, and shouldn't vary based on the area it's in.
    static TRIANGLE_SIZE: f32 = 8.0;

    // Normalized in [0, 1]^2 space.
    // Note on how these coords have been computed: https://github.com/rerun-io/rerun/pull/2920
    // Discussion on the future of icons:  https://github.com/rerun-io/rerun/issues/2960
    let mut points = vec![
        pos2(0.80387, 0.470537),
        pos2(0.816074, 0.5),
        pos2(0.80387, 0.529463),
        pos2(0.316248, 1.017085),
        pos2(0.286141, 1.029362),
        pos2(0.257726, 1.017592),
        pos2(0.245118, 0.987622),
        pos2(0.245118, 0.012378),
        pos2(0.257726, -0.017592),
        pos2(0.286141, -0.029362),
        pos2(0.316248, -0.017085),
        pos2(0.80387, 0.470537),
    ];

    use std::f32::consts::TAU;
    let rotation = Rot2::from_angle(egui::remap(openness, 0.0..=1.0, 0.0..=TAU / 4.0));
    for p in &mut points {
        *p = center + rotation * (*p - pos2(0.5, 0.5)) * TRIANGLE_SIZE;
    }

    ui.painter().add(Shape::convex_polygon(
        points,
        visuals.fg_stroke.color,
        egui::Stroke::NONE,
    ));
}

/// Draws a shadow into the given rect with the shadow direction given from dark to light
fn draw_shadow_line(ui: &egui::Ui, rect: Rect, direction: egui::Direction) {
    let color_dark = egui::Color32::from_black_alpha(77);
    let color_bright = Color32::TRANSPARENT;

    let (left_top, right_top, left_bottom, right_bottom) = match direction {
        egui::Direction::RightToLeft => (color_bright, color_dark, color_bright, color_dark),
        egui::Direction::LeftToRight => (color_dark, color_bright, color_dark, color_bright),
        egui::Direction::BottomUp => (color_bright, color_bright, color_dark, color_dark),
        egui::Direction::TopDown => (color_dark, color_dark, color_bright, color_bright),
    };

    use egui::epaint::Vertex;
    let shadow = egui::Mesh {
        indices: vec![0, 1, 2, 2, 1, 3],
        vertices: vec![
            Vertex {
                pos: rect.left_top(),
                uv: egui::epaint::WHITE_UV,
                color: left_top,
            },
            Vertex {
                pos: rect.right_top(),
                uv: egui::epaint::WHITE_UV,
                color: right_top,
            },
            Vertex {
                pos: rect.left_bottom(),
                uv: egui::epaint::WHITE_UV,
                color: left_bottom,
            },
            Vertex {
                pos: rect.right_bottom(),
                uv: egui::epaint::WHITE_UV,
                color: right_bottom,
            },
        ],
        texture_id: Default::default(),
    };

    ui.painter().add(shadow);
}

#[derive(Debug, Clone)]
struct ActorSpanRow<'a> {
    pub max_depth: usize,
    pub spans: Vec<SpanRef<'a>>,
}

#[derive(Debug, Clone)]
struct SelectedData<'a> {
    pub range: (Timestamp, Timestamp),

    pub actors: IndexMap<String, Vec<ActorSpanRow<'a>>>,
}

fn span_depth(span: SpanRef) -> usize {
    span.children()
        .map(|span| span_depth(span))
        .max()
        .unwrap_or(0)
        + 1
}

fn convert<'a>(app_state: &'a AppState, store: &'a Store) -> SelectedData<'a> {
    let mut selected = SelectedData {
        range: (Timestamp::MAX, Timestamp::MIN),
        actors: Default::default(),
    };

    for span in app_state.filter().filter_root_spans(store) {
        // update range
        selected.range.0 = selected.range.0.min(span.start);
        selected.range.1 = selected.range.1.max(span.end);

        let Some(actor) = selected.actors.get_mut(&span.actor) else {
            selected.actors.insert(
                span.actor.clone(),
                vec![ActorSpanRow {
                    max_depth: span_depth(span),
                    spans: vec![span],
                }],
            );

            continue;
        };

        let mut depth = 0;
        loop {
            let Some(actor_row) = actor.get_mut(depth) else {
                actor.push(ActorSpanRow {
                    max_depth: span_depth(span),
                    spans: vec![span],
                });

                break;
            };

            let last_span = actor_row
                .spans
                .last()
                .expect("ActorData always gets initialized with one span.");

            if span.start >= last_span.end {
                actor_row.max_depth = actor_row.max_depth.max(span_depth(span));
                actor_row.spans.push(span);

                break;
            }

            depth += 1;
        }
    }

    if selected.range.0 > selected.range.1 {
        selected.range.1 = selected.range.0;
    }

    selected
}
