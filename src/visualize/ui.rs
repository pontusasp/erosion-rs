use std::{collections::HashSet, mem, rc::Rc};

use egui::{Color32, Rect};
use macroquad::prelude::*;

#[cfg(feature = "export")]
use crate::heightmap::io::export_heightmaps;

use crate::{partitioning, visualize::heightmap_to_texture};

use super::{
    widgets::{
        ui_keybinds_window, ui_metadata_window, ui_metrics_window, ui_side_panel, ui_top_panel,
    },
    AppState,
};

/*
Keybinds:
- [G] generate new heightmap
- [R] restart
- [S] export
- [E] erode
- [H] Show/Hide Keybinds
- [Q|Esc] quit
- [Space] show heightmap texture
- [D] show diff
- [Shift-D] show diff normalized
- [J] select next partitioning method
- [K] select previous partitioning method
ui.label("[G] Generate New Heightmap");
ui.label("[R] Restart");
ui.label("[S] Export");
ui.label("[H] Show/Hide Keybinds");
ui.label("[E] Erode");
ui.label("[Q][Escape] Quit");
ui.label("[Space] Show Heightmap Texture");
ui.label("[D] Show Diff");
ui.label("[Shift-D] Show Diff Normalized");
ui.label("[J] Select Next Partitioning Method");
ui.label("[K] Select Previous Partitioning Method");
 */

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum UiWindow {
    All,
    Keybinds,
    ControlPanel,
    Metadata,
    Metrics,
}

impl UiWindow {
    pub fn to_string(self) -> String {
        match self {
            UiWindow::All => "All UI".to_string(),
            UiWindow::Keybinds => "Keybinds UI".to_string(),
            UiWindow::ControlPanel => "Control Panel UI".to_string(),
            UiWindow::Metadata => "Metadata UI".to_string(),
            UiWindow::Metrics => "Metrics UI".to_string(),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum UiEvent {
    NewHeightmap,
    ReplaceHeightmap,
    Clear,
    #[cfg(feature = "export")]
    Export,
    RunSimulation,
    ToggleUi(UiWindow),
    Quit,
    ShowBaseLayer,
    ShowDifference,
    ShowDifferenceNormalized,
    NextPartitioningMethod,
    PreviousPartitioningMethod,
    SelectMethod(partitioning::Method),
    NextState,
    PreviousState,
    SelectState(usize),
    NextDiff,
    PreviousDiff,
}

impl UiEvent {
    pub fn info(self) -> String {
        match self {
            UiEvent::NewHeightmap => "Generate new heightmap".to_string(),
            UiEvent::ReplaceHeightmap => "Replace heightmap".to_string(),
            UiEvent::Clear => "Clear simulations".to_string(),
            #[cfg(feature = "export")]
            UiEvent::Export => "Export layers".to_string(),
            UiEvent::RunSimulation => "Run simulation".to_string(),
            UiEvent::ToggleUi(window) => format!("Toggles {}", window.to_string()).to_string(),
            UiEvent::Quit => "Quit".to_string(),
            UiEvent::ShowBaseLayer => "Show base layer".to_string(),
            UiEvent::ShowDifference => "Show difference".to_string(),
            UiEvent::ShowDifferenceNormalized => "Show difference normalized".to_string(),
            UiEvent::NextPartitioningMethod => "Select next partitioning method".to_string(),
            UiEvent::PreviousPartitioningMethod => {
                "Select previous partitioning method".to_string()
            }
            UiEvent::SelectMethod(method) => {
                format!("Select method {}", method.to_string()).to_string()
            }
            UiEvent::NextState => "Select next state".to_string(),
            UiEvent::PreviousState => "Select previous state".to_string(),
            UiEvent::SelectState(id) => format!("Select state #{}", id).to_string(),
            UiEvent::NextDiff => "Select next state for diff".to_string(),
            UiEvent::PreviousDiff => "Select previous state for diff".to_string(),
        }
    }
}

pub struct UiState {
    pub show_ui_all: bool,
    pub show_ui_keybinds: bool,
    pub show_ui_control_panel: bool,
    pub show_ui_metadata: bool,
    pub show_ui_metrics: bool,
    pub simulation_clear: bool,
    pub simulation_regenerate: bool,
    pub application_quit: bool,
    pub ui_events: Vec<UiEvent>,
    pub ui_events_previous: Vec<UiEvent>,
    pub frame_slots: Option<FrameSlots>,
}

impl UiState {
    pub fn clear_events(&mut self) {
        mem::swap(&mut self.ui_events_previous, &mut self.ui_events);
        self.ui_events.clear();
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum UiKey {
    Single(KeyCode),
    Double((KeyCode, KeyCode)),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum UiKeybind {
    Pressed(UiKey, UiEvent),
    Down(UiKey, UiEvent),
}

pub const KEYCODE_TOGGLE_ALL_UI: KeyCode = KeyCode::F1;
pub const KEYCODE_TOGGLE_CONTROL_PANEL_UI: KeyCode = KeyCode::F2;
pub const KEYCODE_TOGGLE_KEYBINDS_UI: KeyCode = KeyCode::F3;
pub const KEYCODE_TOGGLE_METADATA_UI: KeyCode = KeyCode::F4;
pub const KEYCODE_TOGGLE_METRICS_UI: KeyCode = KeyCode::F5;
pub const KEYCODE_NEW_HEIGHTMAP: KeyCode = KeyCode::G;
pub const KEYCODE_NEXT_PARTITIONING_METHOD: KeyCode = KeyCode::J;
pub const KEYCODE_PREVIOUS_PARTITIONING_METHOD: KeyCode = KeyCode::K;
pub const KEYBINDS: &[UiKeybind] = &[
    UiKeybind::Pressed(
        UiKey::Single(KEYCODE_TOGGLE_ALL_UI),
        UiEvent::ToggleUi(UiWindow::All),
    ),
    UiKeybind::Pressed(
        UiKey::Single(KeyCode::F2),
        UiEvent::ToggleUi(UiWindow::ControlPanel),
    ),
    UiKeybind::Pressed(
        UiKey::Single(KEYCODE_TOGGLE_KEYBINDS_UI),
        UiEvent::ToggleUi(UiWindow::Keybinds),
    ),
    UiKeybind::Pressed(UiKey::Single(KEYCODE_NEW_HEIGHTMAP), UiEvent::NewHeightmap),
    UiKeybind::Pressed(UiKey::Single(KeyCode::R), UiEvent::Clear),
    #[cfg(feature = "export")]
    UiKeybind::Pressed(UiKey::Single(KeyCode::S), UiEvent::Export),
    UiKeybind::Pressed(UiKey::Single(KeyCode::Enter), UiEvent::RunSimulation),
    UiKeybind::Pressed(UiKey::Single(KeyCode::Q), UiEvent::Quit),
    UiKeybind::Pressed(UiKey::Single(KeyCode::Escape), UiEvent::Quit),
    UiKeybind::Down(UiKey::Single(KeyCode::Space), UiEvent::ShowBaseLayer),
    UiKeybind::Down(UiKey::Single(KeyCode::D), UiEvent::ShowDifference),
    UiKeybind::Down(
        UiKey::Double((KeyCode::LeftShift, KeyCode::D)),
        UiEvent::ShowDifferenceNormalized,
    ),
    UiKeybind::Pressed(
        UiKey::Single(KEYCODE_NEXT_PARTITIONING_METHOD),
        UiEvent::NextPartitioningMethod,
    ),
    UiKeybind::Pressed(
        UiKey::Single(KEYCODE_PREVIOUS_PARTITIONING_METHOD),
        UiEvent::PreviousPartitioningMethod,
    ),
    UiKeybind::Pressed(UiKey::Single(KeyCode::Up), UiEvent::PreviousState),
    UiKeybind::Pressed(UiKey::Single(KeyCode::Down), UiEvent::NextState),
    UiKeybind::Pressed(UiKey::Single(KeyCode::Left), UiEvent::PreviousDiff),
    UiKeybind::Pressed(UiKey::Single(KeyCode::Right), UiEvent::NextDiff),
    UiKeybind::Pressed(
        UiKey::Single(KEYCODE_TOGGLE_METADATA_UI),
        UiEvent::ToggleUi(UiWindow::Metadata),
    ),
    UiKeybind::Pressed(
        UiKey::Single(KEYCODE_TOGGLE_METRICS_UI),
        UiEvent::ToggleUi(UiWindow::Metrics),
    ),
];

pub fn poll_ui_keybinds(ui_state: &mut UiState) {
    let mut consumed_keys = HashSet::new();
    for &keybind in KEYBINDS.iter() {
        match keybind {
            UiKeybind::Pressed(keybind, event) => match keybind {
                UiKey::Single(_) => (),
                UiKey::Double(key_codes) => {
                    if is_key_pressed(key_codes.0)
                        && is_key_pressed(key_codes.1)
                        && !consumed_keys.contains(&key_codes.1)
                    {
                        consumed_keys.insert(key_codes.1);
                        ui_state.ui_events.push(event);
                    }
                }
            },
            UiKeybind::Down(keybind, event) => match keybind {
                UiKey::Single(_) => (),
                UiKey::Double(key_codes) => {
                    if is_key_down(key_codes.0)
                        && is_key_down(key_codes.1)
                        && !consumed_keys.contains(&key_codes.1)
                    {
                        consumed_keys.insert(key_codes.1);
                        ui_state.ui_events.push(event);
                    }
                }
            },
        }
    }
    for &keybind in KEYBINDS.iter() {
        match keybind {
            UiKeybind::Pressed(keybind, event) => match keybind {
                UiKey::Single(key_code) => {
                    if is_key_pressed(key_code) && !consumed_keys.contains(&key_code) {
                        consumed_keys.insert(key_code);
                        ui_state.ui_events.push(event);
                    }
                }
                UiKey::Double(_) => (),
            },
            UiKeybind::Down(keybind, event) => match keybind {
                UiKey::Single(key_code) => {
                    if is_key_down(key_code) && !consumed_keys.contains(&key_code) {
                        consumed_keys.insert(key_code);
                        ui_state.ui_events.push(event);
                    }
                }
                UiKey::Double(_) => (),
            },
        }
    }
}

fn get_or_calculate_selected_diff_index(state: &AppState) -> Option<usize> {
    if let Some(eroded) = state.simulation_state().eroded() {
        if let Some(i) = eroded.diff_index_of(&eroded.selected_diff.borrow()) {
            Some(i)
        } else {
            let heightmap = &eroded.heightmap_eroded;
            let mut heightmap_diff = heightmap
                .subtract(
                    if let Some(eroded) =
                        state.simulation_states[*eroded.selected_diff.borrow()].eroded()
                    {
                        &eroded.heightmap_eroded
                    } else {
                        &state.simulation_states[*eroded.selected_diff.borrow()]
                            .base()
                            .heightmap_base
                    },
                )
                .unwrap();
            heightmap_diff.calculate_total_height();
            let heightmap_diff_texture = heightmap_to_texture(&heightmap_diff);
            let heightmap_diff_normalized = heightmap_diff.clone().normalize();
            let heightmap_diff_normalized_texture =
                heightmap_to_texture(&heightmap_diff_normalized);

            eroded
                .heightmap_difference
                .borrow_mut()
                .push(Rc::new(heightmap_diff));
            eroded
                .texture_difference
                .borrow_mut()
                .push(Rc::new(heightmap_diff_texture));
            eroded
                .texture_difference_normalized
                .borrow_mut()
                .push(Rc::new(heightmap_diff_normalized_texture));
            eroded
                .diffs
                .borrow_mut()
                .push(eroded.selected_diff.borrow().clone());
            Some(eroded.diffs.borrow().len() - 1)
        }
    } else {
        None
    }
}

pub fn poll_ui_events(ui_state: &mut UiState, state: &mut AppState) {
    {
        let texture = if let Some(eroded) = state.simulation_state().eroded() {
            Some(Rc::clone(&eroded.texture_eroded))
        } else {
            None
        };

        if let Some(texture) = texture {
            state.simulation_state_mut().set_active_texture(&texture);
        }
    };

    for event in ui_state.ui_events.iter() {
        match event {
            UiEvent::NewHeightmap => {
                println!("Regenerating heightmap");
                ui_state.simulation_clear = true;
                ui_state.simulation_regenerate = true;
            }
            UiEvent::ReplaceHeightmap => {
                println!("Regenerating heightmap");
                state.simulation_states.pop();
                state.simulation_base_indices.pop();
                ui_state.simulation_clear = true;
                ui_state.simulation_regenerate = true;
            }
            UiEvent::Clear => {
                println!("Restarting");
                ui_state.simulation_clear = true;
            }
            #[cfg(feature = "export")]
            UiEvent::Export => match state.simulation_state() {
                SimulationState::Base(base) => {
                    export_heightmaps(vec![&base.heightmap_base], vec!["output/heightmap"]);
                }
                SimulationState::Eroded((base, eroded)) => {
                    let diff_index: usize =
                        if let Some(i) = eroded.diff_index_of(&eroded.selected_diff.borrow()) {
                            i
                        } else {
                            0
                        };
                    export_heightmaps(
                        vec![
                            &base.heightmap_base,
                            &eroded.heightmap_eroded,
                            &eroded.heightmap_difference.borrow()[diff_index],
                        ],
                        vec![
                            "output/heightmap",
                            "output/heightmap_eroded",
                            "output/heightmap_diff",
                        ],
                    );
                }
            },
            UiEvent::ToggleUi(ui_window) => match ui_window {
                UiWindow::All => {
                    ui_state.show_ui_all = !ui_state.show_ui_all;
                }
                UiWindow::Keybinds => {
                    ui_state.show_ui_keybinds = !ui_state.show_ui_keybinds;
                }
                UiWindow::ControlPanel => {
                    ui_state.show_ui_control_panel = !ui_state.show_ui_control_panel;
                }
                UiWindow::Metadata => {
                    ui_state.show_ui_metadata = !ui_state.show_ui_metadata;
                }
                UiWindow::Metrics => {
                    ui_state.show_ui_metrics = !ui_state.show_ui_metrics;
                }
            },
            UiEvent::RunSimulation => {
                let simulation_state = state.simulation_state().get_new_eroded(
                    state.simulation_states.len(),
                    &state.parameters.erosion_params,
                );
                state.simulation_states.push(simulation_state);
                state
                    .simulation_base_indices
                    .push(state.simulation_states.len() - 1);
            }
            UiEvent::Quit => {
                println!("Quitting...");
                ui_state.application_quit = true;
            }
            UiEvent::ShowBaseLayer => {
                let texture = Rc::clone(&state.simulation_state().base().texture_heightmap_base);
                state.simulation_state_mut().set_active_texture(&texture);
            }
            UiEvent::ShowDifference => {
                let texture = if let Some(eroded) = state.simulation_state().eroded() {
                    let diff_index: usize = get_or_calculate_selected_diff_index(state).unwrap();
                    Some(Rc::clone(&eroded.texture_difference.borrow()[diff_index]))
                } else {
                    None
                };

                if let Some(texture) = texture {
                    state.simulation_state_mut().set_active_texture(&texture);
                }
            }
            UiEvent::ShowDifferenceNormalized => {
                let texture = if let Some(eroded) = state.simulation_state().eroded() {
                    let diff_index: usize = get_or_calculate_selected_diff_index(state).unwrap();
                    Some(Rc::clone(
                        &eroded.texture_difference_normalized.borrow()[diff_index],
                    ))
                } else {
                    None
                };

                if let Some(texture) = texture {
                    state.simulation_state_mut().set_active_texture(&texture);
                }
            }
            UiEvent::NextPartitioningMethod => {
                state.simulation_state_mut().base_mut().erosion_method =
                    state.simulation_state().base().erosion_method.next();

                println!(
                    "Selected {} method.",
                    state.simulation_state().base().erosion_method.to_string()
                );
            }
            UiEvent::PreviousPartitioningMethod => {
                state.simulation_state_mut().base_mut().erosion_method =
                    state.simulation_state().base().erosion_method.previous();
                println!(
                    "Selected {} method.",
                    state.simulation_state().base().erosion_method.to_string()
                );
            }
            UiEvent::SelectMethod(method) => {
                state.simulation_state_mut().base_mut().erosion_method = *method;
                println!(
                    "Selected {} method.",
                    state.simulation_state().base().erosion_method.to_string()
                );
            }
            UiEvent::NextState => {
                let index = state.simulation_base_indices[state.simulation_base_indices.len() - 1];
                let len = state.simulation_base_indices.len();
                state.simulation_base_indices[len - 1] = (index + 1) % len;
            }
            UiEvent::PreviousState => {
                let index = state.simulation_base_indices[state.simulation_base_indices.len() - 1];
                let len = state.simulation_base_indices.len();
                state.simulation_base_indices[len - 1] = (index + len - 1) % len;
            }
            UiEvent::SelectState(id) => {
                let len = state.simulation_base_indices.len();
                state.simulation_base_indices[len - 1] = id % len;
            }
            UiEvent::NextDiff => {
                if let Some(eroded) = state.simulation_state().eroded() {
                    let mut selected_diff = *eroded.selected_diff.borrow();
                    let len = state.simulation_base_indices.len();
                    selected_diff = (selected_diff + 1) % len;
                    eroded.selected_diff.replace(selected_diff);
                }
            }
            UiEvent::PreviousDiff => {
                if let Some(eroded) = state.simulation_state().eroded() {
                    let mut selected_diff = *eroded.selected_diff.borrow();
                    let len = state.simulation_base_indices.len();
                    selected_diff = (selected_diff + len - 1) % len;
                    eroded.selected_diff.replace(selected_diff);
                }
            }
        };
    }
    ui_state.clear_events();
}

pub struct FrameSlots {
    pub canvas: Option<Rect>,
}

pub fn ui_draw(ui_state: &mut UiState, state: &mut AppState) -> Option<FrameSlots> {
    if ui_state.show_ui_all {
        let mut central_rect = None;
        egui_macroquad::ui(|egui_ctx| {
            // Top Panel
            ui_top_panel(egui_ctx, ui_state);

            // Side Panel
            ui_side_panel(egui_ctx, ui_state, state);

            // Central Panel
            central_rect = Some(
                egui::CentralPanel::default()
                    .frame(egui::containers::Frame {
                        fill: Color32::TRANSPARENT,
                        ..Default::default()
                    })
                    .show(egui_ctx, |_| {})
                    .response
                    .rect,
            );

            ui_keybinds_window(egui_ctx, ui_state);
            ui_metadata_window(egui_ctx, ui_state, state);
            ui_metrics_window(egui_ctx, ui_state, state);
        });

        egui_macroquad::draw();
        Some(FrameSlots {
            canvas: central_rect,
        })
    } else {
        None
    }
}
