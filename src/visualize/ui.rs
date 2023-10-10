use std::{collections::HashSet, mem, rc::Rc};

use macroquad::prelude::*;
use bracket_noise::prelude::*;

use crate::{heightmap::{self, HeightmapSettings}, partitioning, visualize::heightmap_to_texture};

use super::lague::{AppState, SimulationState};

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
}

impl UiWindow {
    pub fn to_string(self) -> String {
        match self {
            UiWindow::All => "All UI".to_string(),
            UiWindow::Keybinds => "Keybinds UI".to_string(),
            UiWindow::ControlPanel => "Control Panel UI".to_string(),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum UiEvent {
    NewHeightmap,
    ReplaceHeightmap,
    Clear,
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
    pub simulation_clear: bool,
    pub simulation_regenerate: bool,
    pub application_quit: bool,
    pub ui_events: Vec<UiEvent>,
    pub ui_events_previous: Vec<UiEvent>,
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

pub const KEYCODE_TOGGLE_CONTROL_PANEL_UI: KeyCode = KeyCode::F2;
pub const KEYCODE_TOGGLE_KEYBINDS_UI: KeyCode = KeyCode::F3;
pub const KEYCODE_NEXT_PARTITIONING_METHOD: KeyCode = KeyCode::J;
pub const KEYCODE_PREVIOUS_PARTITIONING_METHOD: KeyCode = KeyCode::K;
pub const KEYBINDS: [UiKeybind; 18] = [
    UiKeybind::Pressed(UiKey::Single(KeyCode::F1), UiEvent::ToggleUi(UiWindow::All)),
    UiKeybind::Pressed(
        UiKey::Single(KeyCode::F2),
        UiEvent::ToggleUi(UiWindow::ControlPanel),
    ),
    UiKeybind::Pressed(
        UiKey::Single(KEYCODE_TOGGLE_KEYBINDS_UI),
        UiEvent::ToggleUi(UiWindow::Keybinds),
    ),
    UiKeybind::Pressed(UiKey::Single(KeyCode::G), UiEvent::NewHeightmap),
    UiKeybind::Pressed(UiKey::Single(KeyCode::R), UiEvent::Clear),
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
            UiEvent::Export => match state.simulation_state() {
                SimulationState::Base(base) => {
                    heightmap::export_heightmaps(
                        vec![&base.heightmap_base],
                        vec!["output/heightmap"],
                    );
                }
                SimulationState::Eroded((base, eroded)) => {
                    let diff_index: usize =
                        if let Some(i) = eroded.diff_index_of(&eroded.selected_diff.borrow()) {
                            i
                        } else {
                            0
                        };
                    heightmap::export_heightmaps(
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
            },
            UiEvent::RunSimulation => {
                let simulation_state = state
                    .simulation_state()
                    .get_new_eroded(state.simulation_states.len());
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

pub fn ui_draw(ui_state: &mut UiState, state: &mut AppState) {
    if ui_state.show_ui_all {
        egui_macroquad::ui(|egui_ctx| {
            if ui_state.show_ui_keybinds {
                egui::Window::new(format!("Keybinds [{:?}]", KEYCODE_TOGGLE_KEYBINDS_UI)).show(
                    egui_ctx,
                    |ui| {
                        for keybind in KEYBINDS {
                            match keybind {
                                UiKeybind::Pressed(keys, event) => {
                                    ui.horizontal(|ui| {
                                        if ui.button(event.info()).clicked() {
                                            ui_state.ui_events.push(event);
                                        }
                                        match keys {
                                            UiKey::Single(key_code) => {
                                                ui.label(format!("[{:?}]", key_code))
                                            }
                                            UiKey::Double(key_codes) => ui.label(format!(
                                                "[{:?}-{:?}]",
                                                key_codes.0, key_codes.1
                                            )),
                                        };
                                    });
                                }
                                UiKeybind::Down(keys, event) => {
                                    if ui_state.ui_events_previous.contains(&event) {
                                        ui.label(event.info());
                                    } else {
                                        if ui.button(event.info()).clicked() {
                                            ui_state.ui_events.push(event);
                                        }
                                    }
                                    match keys {
                                        UiKey::Single(key_code) => {
                                            ui.label(format!("({:?})", key_code))
                                        }
                                        UiKey::Double(key_codes) => ui.label(format!(
                                            "({:?}-{:?})",
                                            key_codes.0, key_codes.1
                                        )),
                                    };
                                }
                            }
                        }
                    },
                );
            }

            if state.simulation_state().eroded().is_none() {
                egui::Window::new(format!("Parameters"))
                    .show(egui_ctx, |ui| {
                        ui.heading("Heightmap Generation");
                        let mut updated = false;

                        updated = updated || ui.add(egui::Slider::new(&mut state.parameters.heightmap_settings.seed, 0..=10000000000).text("Seed")).changed();

                        let noise_type = state.parameters.heightmap_settings.noise_type;
                        egui::ComboBox::from_label("Noise Type")
                            .selected_text(format!("{:?}", state.parameters.heightmap_settings.noise_type))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut state.parameters.heightmap_settings.noise_type, NoiseType::Value, "Value");
                                ui.selectable_value(&mut state.parameters.heightmap_settings.noise_type, NoiseType::ValueFractal, "Value Fractal");
                                ui.selectable_value(&mut state.parameters.heightmap_settings.noise_type, NoiseType::Perlin, "Perlin");
                                ui.selectable_value(&mut state.parameters.heightmap_settings.noise_type, NoiseType::PerlinFractal, "Perlin
Fractal");
                                ui.selectable_value(&mut state.parameters.heightmap_settings.noise_type, NoiseType::Simplex, "Simplex");
                                ui.selectable_value(&mut state.parameters.heightmap_settings.noise_type, NoiseType::SimplexFractal, "Simplex Fractal");
                                ui.selectable_value(&mut state.parameters.heightmap_settings.noise_type, NoiseType::Cellular, "Cellular");
                                ui.selectable_value(&mut state.parameters.heightmap_settings.noise_type, NoiseType::WhiteNoise, "WhiteNoise");
                                ui.selectable_value(&mut state.parameters.heightmap_settings.noise_type, NoiseType::Cubic, "Cubic");
                                ui.selectable_value(&mut state.parameters.heightmap_settings.noise_type, NoiseType::CubicFractal, "Cubic Fractal");
                            });
                        updated = updated || noise_type != state.parameters.heightmap_settings.noise_type;

                        updated = updated || ui.add(egui::Slider::new(&mut state.parameters.heightmap_settings.fractal_octaves, 0..=28).text("Fractal Octaves")).drag_released();
                        updated = updated || ui.add(egui::Slider::new(&mut state.parameters.heightmap_settings.fractal_gain, 0.0..=2.0).text("Fractal Gain")).changed();
                        updated = updated || ui.add(egui::Slider::new(&mut state.parameters.heightmap_settings.fractal_lacunarity, 0.0..=7.0).text("Fractal Lacunarity")).drag_released();
                        updated = updated || ui.add(egui::Slider::new(&mut state.parameters.heightmap_settings.frequency, 0.0..=5.0).text("Frequency")).changed();
                        let mut size = state.parameters.heightmap_settings.width;
                        updated = updated || ui.add(egui::Slider::new(&mut size, 64..=1024).text("Size")).changed();
                        state.parameters.heightmap_settings.width = size;
                        state.parameters.heightmap_settings.height = size;

                        ui.add(egui::Checkbox::new(&mut state.parameters.auto_apply, "Auto Apply"));

                        if ui.button("Reset").clicked() {
                            state.parameters.heightmap_settings = HeightmapSettings::default();
                            updated = true;
                        }

                        let mut apply = false;
                        if !state.parameters.auto_apply {
                            apply = ui.button("Apply").clicked();
                        }

                        let update = (state.parameters.auto_apply && updated) || apply;
                        if update {
                            ui_state.ui_events.push(UiEvent::ReplaceHeightmap);
                        }
                    });
            }

            {
                egui::Window::new(format!("Metadata"))
                    .show(egui_ctx, |ui| {
                        ui.heading("Base Heightmap");
                        ui.label(format!("Width x Height: {} x {}", state.simulation_state().base().heightmap_base.width, state.simulation_state().base().heightmap_base.height));
                        ui.label(format!("Depth: {}", state.simulation_state().base().heightmap_base.depth));
                        ui.label(format!("Original Depth: {}", state.simulation_state().base().heightmap_base.original_depth));
                        if let Some(height) = state.simulation_state().get_heightmap().get_average_height() {
                            ui.label(format!("Average Height: {}", height));
                        }
                        if let Some(height) = state.simulation_state().base().heightmap_base.total_height {
                            ui.label(format!("Total Depth: {}", height));
                        }
                        if let Some(metadata) = state.simulation_state().base().heightmap_base.metadata.clone() {
                            for (k, v) in metadata.iter() {
                                ui.label(format!("{}: {}", k, v));
                            }
                        }
                        if let Some(eroded) = state.simulation_state().eroded() {
                            ui.heading("Eroded Heightmap");
                            ui.label(format!("Width x Height: {} x {}", eroded.heightmap_eroded.width, state.simulation_state().base().heightmap_base.height));
                            ui.label(format!("Depth: {}", eroded.heightmap_eroded.depth));
                            ui.label(format!("Original Depth: {}", eroded.heightmap_eroded.original_depth));
                            if let Some(height) = eroded.heightmap_eroded.get_average_height() {
                                ui.label(format!("Average Height: {}", height));
                            }
                            if let Some(height) = eroded.heightmap_eroded.total_height {
                                ui.label(format!("Total Depth: {}", height));
                            }
                            if let Some(metadata) = eroded.heightmap_eroded.metadata.clone() {
                                for (k, v) in metadata.iter() {
                                    ui.label(format!("{}: {}", k, v));
                                }
                            }
                        }
                    });
            }

            if ui_state.show_ui_control_panel {
                egui::Window::new(format!(
                    "Control Panel [{:?}]",
                    KEYCODE_TOGGLE_CONTROL_PANEL_UI
                ))
                .show(egui_ctx, |ui| {
                    // Erosion Method Selection
                    ui.heading("Erosion Method Selection");
                    for &method in partitioning::Method::iterator() {
                        if method == state.simulation_state().base().erosion_method {
                            ui.label(format!("-> {}", method.to_string()));
                        } else {
                            ui.horizontal(|ui| {
                                if ui.button(method.to_string()).clicked() {
                                    ui_state.ui_events.push(UiEvent::SelectMethod(method));
                                }
                                if method == state.simulation_state().base().erosion_method.next() {
                                    ui.label(format!("{:?}", KEYCODE_NEXT_PARTITIONING_METHOD));
                                } else if method
                                    == state.simulation_state().base().erosion_method.previous()
                                {
                                    ui.label(format!("{:?}", KEYCODE_PREVIOUS_PARTITIONING_METHOD));
                                }
                            });
                        }
                    }

                    ui.heading("Toggles");
                    // Show/Hide Keybinds
                    ui.horizontal(|ui| {
                        if ui
                            .button(if ui_state.show_ui_keybinds {
                                "Hide Keybinds"
                            } else {
                                "Show Keybinds"
                            })
                            .clicked()
                        {
                            ui_state.show_ui_keybinds = !ui_state.show_ui_keybinds;
                        };
                        ui.label(format!("{:?}", KEYCODE_TOGGLE_KEYBINDS_UI));
                    });

                    let selected_diff: Option<usize> =
                        if let Some(eroded) = state.simulation_state().eroded() {
                            Some((*eroded.selected_diff.borrow()).clone())
                        } else {
                            None
                        };
                    // Image Layers
                    ui.heading("Image Layers");
                    for simulation in state.simulation_states.iter() {
                        ui.horizontal(|ui| {
                            if *state.simulation_base_indices.last().unwrap() == simulation.id() {
                                ui.label("-> ");
                            }
                            match simulation {
                                SimulationState::Base(base) => {
                                    ui.label(format!("{}: [Base Layer]", base.id));
                                }
                                SimulationState::Eroded((_, eroded)) => {
                                    ui.label(format!(
                                        "{}: {} eroded from #{}",
                                        eroded.id,
                                        eroded.erosion_method.to_string(),
                                        eroded.base_id
                                    ));
                                }
                            }
                            if let Some(selected_diff) = selected_diff {
                                if simulation.id() == selected_diff {
                                    ui.label(" <-- diff");
                                }
                            }
                        });
                    }
                });
            }
        });

        egui_macroquad::draw();
    }
}
