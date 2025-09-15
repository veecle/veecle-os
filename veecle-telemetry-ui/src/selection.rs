//! Module for managing hover / selection state.
//!
//! See [`SelectionState`].

use std::cell::Cell;

use veecle_telemetry::SpanContext;

use crate::store::{LogId, SpanRef};

/// Represents an item that can be interacted with in the application.
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum Item {
    /// A span item identified by its [`SpanContext`].
    Span(SpanContext),
    /// A log item identified by its [`LogId`].
    Log(LogId),
}

impl From<LogId> for Item {
    fn from(value: LogId) -> Self {
        Item::Log(value)
    }
}

impl From<SpanContext> for Item {
    fn from(value: SpanContext) -> Self {
        Item::Span(value)
    }
}

/// Maintains the global application context.
///
/// So far, this is just double buffered hover tracking (write for the current frame, read from the last frame).
#[derive(Debug, Default)]
pub struct SelectionState {
    last_frame: InnerSelectionState,
    this_frame: Cell<InnerSelectionState>,
}

#[derive(Debug, Default, Copy, Clone)]
struct InnerSelectionState {
    hovered: Option<Item>,
    selected: Option<Item>,
}

pub enum SelectionChange {
    NoChange,
    SelectionChanged,
}

impl SelectionState {
    /// Called at the start of each frame to update hover states.
    /// Moves the current frame's hover state to last frame and clears current frame.
    pub fn on_frame_start(&mut self) -> SelectionChange {
        let this_frame = self.this_frame.take();

        let selection_changed = this_frame.selected != self.last_frame.selected;

        // keep selected, reset hovered
        self.this_frame.get_mut().selected = this_frame.selected;

        self.last_frame = this_frame;

        if selection_changed {
            SelectionChange::SelectionChanged
        } else {
            SelectionChange::NoChange
        }
    }

    fn write(&self, action: impl FnOnce(&mut InnerSelectionState)) {
        let mut state = self.this_frame.get();
        action(&mut state);
        self.this_frame.set(state);
    }

    /// Sets the currently hovered item.
    pub fn set_hovered(&self, item: Item) {
        self.write(|state| {
            state.hovered = Some(item);
        });
    }

    /// Checks if a specific item was hovered in the last frame.
    pub fn is_hovered(&self, item: Item) -> bool {
        self.last_frame.hovered == Some(item)
    }

    /// Checks if a span or any of its associated logs were hovered in the last frame.
    pub fn is_span_hovered(&self, span: SpanRef) -> bool {
        self.is_hovered(span.context.into())
            || span
                .logs
                .iter()
                .any(|&log_id| self.is_hovered(log_id.into()))
    }

    /// Sets the currently selected item.
    pub fn set_selected(&self, item: Item) {
        self.write(|state| {
            state.selected = Some(item);
        });
    }

    /// Clears the currently selected item.
    pub fn clear_selected(&self) {
        self.write(|state| {
            state.selected = None;
        });
    }

    /// Checks if a specific item is selected.
    pub fn is_selected(&self, item: Item) -> bool {
        self.last_frame.selected == Some(item)
    }

    pub fn get_selected(&self) -> Option<Item> {
        self.last_frame.selected
    }
}
