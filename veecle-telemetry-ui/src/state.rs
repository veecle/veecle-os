use crate::command::{CommandSender, SystemCommand, UICommand};
use crate::filter::Filters;
use crate::selection::{SelectionChange, SelectionState};

/// Maintains the global application state.
#[derive(Debug)]
pub struct AppState {
    selection_state: SelectionState,
    panel_states: PanelStates,

    filter: Filters,

    command_sender: CommandSender,
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Default)]
pub enum PanelState {
    Hidden,

    #[default]
    Expanded,
}

impl PanelState {
    #[inline]
    pub fn is_expanded(&self) -> bool {
        self == &PanelState::Expanded
    }

    #[inline]
    pub fn toggle(self) -> Self {
        match self {
            PanelState::Hidden => Self::Expanded,
            PanelState::Expanded => Self::Hidden,
        }
    }
}

#[derive(Debug)]
pub struct PanelStates {
    pub filter_panel: PanelState,
    pub selection_panel: PanelState,
}

impl Default for PanelStates {
    fn default() -> Self {
        PanelStates {
            filter_panel: PanelState::Expanded,
            selection_panel: PanelState::Hidden,
        }
    }
}

impl AppState {
    pub fn new(command_sender: CommandSender) -> Self {
        Self {
            selection_state: Default::default(),
            panel_states: Default::default(),
            filter: Default::default(),
            command_sender,
        }
    }

    /// Called at the start of each frame to update selection and hover states.
    ///
    /// See [`SelectionState::on_frame_start`].
    pub fn on_frame_start(&mut self) {
        let change = self.selection_state.on_frame_start();

        if matches!(change, SelectionChange::SelectionChanged) {
            if self.selection_state.get_selected().is_some() {
                self.panel_states.selection_panel = PanelState::Expanded;
            } else {
                self.panel_states.selection_panel = PanelState::Hidden;
            }
        }
    }

    pub fn selection(&self) -> &SelectionState {
        &self.selection_state
    }

    pub fn panel(&self) -> &PanelStates {
        &self.panel_states
    }

    pub fn toggle_filter_panel(&mut self) {
        self.panel_states.filter_panel = self.panel_states.filter_panel.toggle();
    }

    pub fn toggle_selection_panel(&mut self) {
        self.panel_states.selection_panel = self.panel_states.selection_panel.toggle();
    }

    pub fn filter(&self) -> &Filters {
        &self.filter
    }

    pub fn filter_mut(&mut self) -> &mut Filters {
        &mut self.filter
    }
}

impl AppState {
    /// [`CommandSender::send_system`].
    pub fn send_system(&self, command: SystemCommand) {
        self.command_sender.send_system(command);
    }

    /// [`CommandSender::send_ui`].
    pub fn send_ui(&self, command: UICommand) {
        self.command_sender.send_ui(command);
    }
}
