use crate::heightmap::Heightmap;
use serde::{Deserialize, Serialize};
#[cfg(feature = "export")]
use std::mem;
use std::rc::Rc;

use super::SimulationState;
#[cfg(feature = "export")]
use crate::heightmap::io::export_heightmaps;

use crate::partitioning;
use crate::visualize::ui::UiState;
use crate::visualize::wrappers::HeightmapTexture;
#[cfg(feature = "export")]
use crate::State;

use super::{
    layered_heightmaps_to_texture, mix_heightmap_to_texture, rgba_color_channel, AppState,
    HeightmapLayer, LayerMixMethod,
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum UiEvent {
    NewHeightmap,
    ReplaceHeightmap,
    Clear,
    #[cfg(feature = "export")]
    ExportHeightmap,
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
    ShowErodedLayer,
    Blur,
    EdgeDetect,
    BlurEdgeDetect,
    Isoline,
    #[cfg(feature = "export")]
    ExportState,
    #[cfg(feature = "export")]
    ReadState(usize),
    #[cfg(feature = "export")]
    ExportStateAs,
}

impl UiEvent {
    pub fn info(self) -> String {
        match self {
            UiEvent::NewHeightmap => "Generate new heightmap".to_string(),
            UiEvent::ReplaceHeightmap => "Replace heightmap".to_string(),
            UiEvent::Clear => "Clear simulations".to_string(),
            #[cfg(feature = "export")]
            UiEvent::ExportHeightmap => "Export layers".to_string(),
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
            UiEvent::ShowErodedLayer => "Show eroded layer".to_string(),
            UiEvent::Blur => "Blur currently selected state".to_string(),
            UiEvent::EdgeDetect => "Apply canny edge detection to selected state".to_string(),
            UiEvent::BlurEdgeDetect => {
                "Apply blur then canny edge detection to selected state".to_string()
            }
            UiEvent::Isoline => "Show isoline".to_string(),
            #[cfg(feature = "export")]
            UiEvent::ExportState => "Export State".to_string(),
            #[cfg(feature = "export")]
            UiEvent::ReadState(_) => "Read State from Disk".to_string(),
            #[cfg(feature = "export")]
            UiEvent::ExportStateAs => "Export State As".to_string(),
        }
    }
}

fn get_or_calculate_selected_diff_index(state: &AppState) -> Option<usize> {
    if let Some(eroded) = state.simulation_state().eroded() {
        if let Some(i) = eroded.diff_index_of(&eroded.selected_diff.borrow()) {
            Some(i)
        } else {
            let heightmap = &eroded.heightmap_eroded.heightmap;
            let mut heightmap_diff = heightmap
                .subtract(
                    if let Some(eroded) =
                        state.simulation_states[*eroded.selected_diff.borrow()].eroded()
                    {
                        &eroded.heightmap_eroded.heightmap
                    } else {
                        &state.simulation_states[*eroded.selected_diff.borrow()]
                            .base()
                            .heightmap_base
                            .heightmap
                    },
                )
                .unwrap();
            heightmap_diff.calculate_total_height();
            let heightmap_diff_normalized = heightmap_diff.clone().normalize();

            eroded
                .heightmap_difference
                .borrow_mut()
                .push(Rc::new(heightmap_diff.into()));
            eroded
                .heightmap_difference_normalized
                .borrow_mut()
                .push(Rc::new(heightmap_diff_normalized.into()));
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

fn push_base(app_state: &mut AppState) {
    println!("Regenerating heightmap");
    app_state
        .simulation_states
        .push(SimulationState::get_new_base(
            app_state.simulation_states.len(),
            &app_state.parameters.heightmap_type,
            &app_state.parameters.erosion_params,
        ));
    app_state
        .simulation_base_indices
        .push(app_state.simulation_states.len() - 1);
}

fn try_set_eroded_layer_active(state: &mut AppState) {
    let texture = if let Some(eroded) = state.simulation_state().eroded() {
        Some(Rc::clone(&eroded.heightmap_eroded))
    } else {
        None
    };

    if let Some(heightmap) = texture {
        state.simulation_state_mut().set_active(heightmap);
    }
}

fn poll_ui_events_pre_check(ui_state: &mut UiState) {
    for event in ui_state.ui_events.clone() {
        match event {
            #[cfg(feature = "export")]
            UiEvent::ExportStateAs => {
                // If we are exporting, ignore all other events
                ui_state.ui_events.retain(|&e| e == event);
                break;
            }
            _ => {}
        }
    }
}

pub fn poll_ui_events(
    #[cfg(feature = "export")] state_name: &mut Option<String>,
    ui_state: &mut UiState,
    app_state: &mut AppState,
) {
    poll_ui_events_pre_check(ui_state);

    let mut next_frame_events = Vec::new();
    for event in ui_state.ui_events.clone().iter() {
        match event {
            UiEvent::NewHeightmap => {
                push_base(app_state);
            }
            UiEvent::ReplaceHeightmap => {
                app_state.simulation_states.pop();
                app_state.simulation_base_indices.pop();
                push_base(app_state);
            }
            UiEvent::Clear => {
                println!("Restarting");
                ui_state.simulation_clear = true;
            }
            #[cfg(feature = "export")]
            UiEvent::ExportHeightmap => match app_state.simulation_state() {
                SimulationState::Base(base) => {
                    export_heightmaps(
                        vec![&base.heightmap_base.heightmap],
                        "output",
                        vec!["heightmap"],
                    );
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
                            &base.heightmap_base.heightmap,
                            &eroded.heightmap_eroded.heightmap,
                            &eroded.heightmap_difference.borrow()[diff_index].heightmap,
                            &eroded.heightmap_difference_normalized.borrow()[diff_index].heightmap,
                        ],
                        "output",
                        vec![
                            "heightmap",
                            "heightmap_eroded",
                            "heightmap_diff",
                            "heightmap_diff_normalized",
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
                let simulation_state = app_state.simulation_state().get_new_eroded(
                    app_state.simulation_states.len(),
                    &app_state.parameters.erosion_params,
                );
                app_state.simulation_states.push(simulation_state);
                app_state
                    .simulation_base_indices
                    .push(app_state.simulation_states.len() - 1);
                try_set_eroded_layer_active(app_state);
            }
            UiEvent::Quit => {
                println!("Quitting...");
                ui_state.application_quit = true;
            }
            UiEvent::ShowBaseLayer => {
                let heightmap = Rc::clone(&app_state.simulation_state().base().heightmap_base);
                app_state.simulation_state_mut().set_active(heightmap);
            }
            UiEvent::ShowDifference => {
                let texture = if let Some(eroded) = app_state.simulation_state().eroded() {
                    let diff_index: usize =
                        get_or_calculate_selected_diff_index(app_state).unwrap();
                    let diff_heightmap =
                        Rc::clone(&eroded.heightmap_difference.borrow()[diff_index]);
                    Some(diff_heightmap)
                } else {
                    None
                };

                if let Some(heightmap) = texture {
                    app_state.simulation_state_mut().set_active(heightmap);
                }
            }
            UiEvent::ShowDifferenceNormalized => {
                let texture = if let Some(eroded) = app_state.simulation_state().eroded() {
                    let diff_index: usize =
                        get_or_calculate_selected_diff_index(app_state).unwrap();
                    let diff_heightmap =
                        Rc::clone(&eroded.heightmap_difference_normalized.borrow()[diff_index]);
                    Some(diff_heightmap)
                } else {
                    None
                };

                if let Some(heightmap) = texture {
                    app_state.simulation_state_mut().set_active(heightmap);
                }
            }
            UiEvent::NextPartitioningMethod => {
                app_state.simulation_state_mut().base_mut().erosion_method =
                    app_state.simulation_state().base().erosion_method.next();

                println!(
                    "Selected {} method.",
                    app_state
                        .simulation_state()
                        .base()
                        .erosion_method
                        .to_string()
                );
            }
            UiEvent::PreviousPartitioningMethod => {
                app_state.simulation_state_mut().base_mut().erosion_method = app_state
                    .simulation_state()
                    .base()
                    .erosion_method
                    .previous();
                println!(
                    "Selected {} method.",
                    app_state
                        .simulation_state()
                        .base()
                        .erosion_method
                        .to_string()
                );
            }
            UiEvent::SelectMethod(method) => {
                app_state.simulation_state_mut().base_mut().erosion_method = *method;
                println!(
                    "Selected {} method.",
                    app_state
                        .simulation_state()
                        .base()
                        .erosion_method
                        .to_string()
                );
            }
            UiEvent::NextState => {
                let index =
                    app_state.simulation_base_indices[app_state.simulation_base_indices.len() - 1];
                let len = app_state.simulation_base_indices.len();
                app_state.simulation_base_indices[len - 1] = (index + 1) % len;
            }
            UiEvent::PreviousState => {
                let index =
                    app_state.simulation_base_indices[app_state.simulation_base_indices.len() - 1];
                let len = app_state.simulation_base_indices.len();
                app_state.simulation_base_indices[len - 1] = (index + len - 1) % len;
            }
            UiEvent::SelectState(id) => {
                let len = app_state.simulation_base_indices.len();
                app_state.simulation_base_indices[len - 1] = id % len;
            }
            UiEvent::NextDiff => {
                if let Some(eroded) = app_state.simulation_state().eroded() {
                    let mut selected_diff = *eroded.selected_diff.borrow();
                    let len = app_state.simulation_base_indices.len();
                    selected_diff = (selected_diff + 1) % len;
                    eroded.selected_diff.replace(selected_diff);
                }
            }
            UiEvent::PreviousDiff => {
                if let Some(eroded) = app_state.simulation_state().eroded() {
                    let mut selected_diff = *eroded.selected_diff.borrow();
                    let len = app_state.simulation_base_indices.len();
                    selected_diff = (selected_diff + len - 1) % len;
                    eroded.selected_diff.replace(selected_diff);
                }
            }
            UiEvent::ShowErodedLayer => {
                try_set_eroded_layer_active(app_state);
            }

            UiEvent::Blur => {
                if let Some(heightmap) = app_state
                    .simulation_state()
                    .get_heightmap()
                    .blur(ui_state.blur_sigma)
                {
                    let heightmap_texture = Rc::new(heightmap.into());
                    app_state
                        .simulation_state_mut()
                        .set_active(heightmap_texture);
                } else {
                    eprintln!("Failed to blur selected state!");
                }
            }
            UiEvent::EdgeDetect => {
                let (low, high) = ui_state.canny_edge;
                let og = app_state.simulation_state().get_heightmap();
                if let Some(heightmap) = og.canny_edge(low, high) {
                    let texture =
                        Rc::new(mix_heightmap_to_texture(&og, &heightmap, 0, true, false));
                    let heightmap_texture =
                        Rc::new(HeightmapTexture::new(Rc::new(heightmap), Some(texture)));
                    app_state
                        .simulation_state_mut()
                        .set_active(heightmap_texture);
                } else {
                    eprintln!("Failed to edge detect selected state!");
                }
            }
            UiEvent::BlurEdgeDetect => {
                let (low, high) = ui_state.canny_edge;
                let og = app_state.simulation_state().get_heightmap();
                if let Some(heightmap) = og
                    .blur(ui_state.blur_sigma)
                    .and_then(|blurred| blurred.canny_edge(low, high))
                {
                    let texture =
                        Rc::new(mix_heightmap_to_texture(&og, &heightmap, 0, true, false));
                    let heightmap_texture =
                        Rc::new(HeightmapTexture::new(Rc::new(heightmap), Some(texture)));
                    app_state
                        .simulation_state_mut()
                        .set_active(heightmap_texture);
                } else {
                    eprintln!("Failed to blur or edge detect selected state!");
                }
            }
            UiEvent::Isoline => {
                let props = ui_state.isoline;
                let heightmap = app_state.simulation_state().get_heightmap();
                let outside = (*heightmap).clone().boolean(
                    props.height + props.error * if props.flood_lower { 1.0 } else { -1.0 },
                    true,
                    props.flood_lower,
                );
                let isoline = {
                    let h = heightmap.isoline(props.height, props.error);
                    if props.blur_augmentation.0 {
                        h.blur(props.blur_augmentation.1)
                            .and_then(|b| Some(b.boolean(0.0, false, false)))
                            .unwrap_or(h)
                    } else {
                        h
                    }
                };
                let flood = {
                    let flood = heightmap.get_flood_points(&isoline, props.flood_lower);
                    if props.blur_augmentation.0 {
                        Heightmap::filter_noise_points(
                            heightmap.width,
                            &flood,
                            props.blur_augmentation.2,
                            props.blur_augmentation.3,
                        )
                    } else {
                        flood
                    }
                };
                let flooded = if props.should_flood {
                    let flood_amount = 1f32.min(props.height + (1.0 - props.height) / 3.0);
                    let (flooded, areas) = isoline.flood_empty(flood_amount, &flood);
                    let flood_inverse = heightmap.get_flood_points(&flooded, !props.flood_lower);
                    if props.flood_lower {
                        ui_state.isoline.flooded_areas_lower = Some(areas);
                        ui_state.isoline.flooded_areas_higher =
                            Some(flooded.flood_empty(flood_amount, &flood_inverse).1);
                    } else {
                        ui_state.isoline.flooded_areas_lower =
                            Some(flooded.flood_empty(flood_amount, &flood_inverse).1);
                        ui_state.isoline.flooded_areas_higher = Some(areas);
                    }
                    Some(flooded)
                } else {
                    None
                };
                let flood_line = Heightmap::from_points(heightmap.width, &flood, 1.0);
                let flood_line_blurred = flood_line.blur(1.0).unwrap().boolean(0.0, false, false);

                let hm = Rc::new(flooded.unwrap_or(isoline));

                let tex = if props.advanced_texture {
                    Rc::new(layered_heightmaps_to_texture(
                        hm.width,
                        &vec![
                            &HeightmapLayer {
                                heightmap: &heightmap,
                                channel: rgba_color_channel::RGB,
                                strength: 1.0,
                                layer_mix_method: LayerMixMethod::Additive,
                                inverted: false,
                                modifies_alpha: false,
                            },
                            &HeightmapLayer {
                                heightmap: &hm,
                                channel: rgba_color_channel::RGB,
                                strength: 0.5,
                                layer_mix_method: LayerMixMethod::Multiply,
                                inverted: false,
                                modifies_alpha: false,
                            },
                            &HeightmapLayer {
                                heightmap: &outside,
                                channel: rgba_color_channel::R,
                                strength: 0.3,
                                layer_mix_method: LayerMixMethod::Multiply,
                                inverted: false,
                                modifies_alpha: false,
                            },
                            &HeightmapLayer {
                                heightmap: &flood_line_blurred,
                                channel: rgba_color_channel::B,
                                strength: 0.3,
                                layer_mix_method: LayerMixMethod::AdditiveClamp,
                                inverted: false,
                                modifies_alpha: false,
                            },
                            &HeightmapLayer {
                                heightmap: &flood_line,
                                channel: rgba_color_channel::B,
                                strength: 1.0,
                                layer_mix_method: LayerMixMethod::AdditiveClamp,
                                inverted: false,
                                modifies_alpha: false,
                            },
                        ],
                        true,
                        1.0,
                    ))
                } else {
                    Rc::new(mix_heightmap_to_texture(&hm, &outside, 0, false, false))
                };

                app_state
                    .simulation_state_mut()
                    .set_active(Rc::new(HeightmapTexture::new(hm, Some(tex))));
            }
            #[cfg(feature = "export")]
            UiEvent::ExportState => {
                let filename = if let Some(filename) = &state_name {
                    filename.as_str()
                } else {
                    crate::io::DEFAULT_NAME
                };
                crate::io::export_json(
                    &State {
                        state_name: state_name.clone(),
                        app_state: app_state.clone(),
                        ui_state: ui_state.clone(),
                    },
                    filename,
                )
                    .expect("Failed to export state!");
                crate::io::export_binary(
                    &State {
                        state_name: state_name.clone(),
                        app_state: app_state.clone(),
                        ui_state: ui_state.clone(),
                    },
                    filename,
                )
                    .expect("Failed to export state!");
                crate::io::export_icon(
                    &State {
                        state_name: state_name.clone(),
                        app_state: app_state.clone(),
                        ui_state: ui_state.clone(),
                    },
                    filename,
                )
                .expect("Failed to export icon!");
            }
            #[cfg(feature = "export")]
            UiEvent::ReadState(index) => {
                let state_file = ui_state
                    .saves
                    .get(*index)
                    .expect("Something went wrong when loading the file.");
                let mut result = crate::io::import(&state_file.0);
                if let Ok(State {
                    state_name: ref mut state_name_,
                    app_state: ref mut app_state_,
                    ui_state: ref mut ui_state_,
                }) = result
                {
                    mem::swap(state_name, state_name_);
                    mem::swap(app_state, app_state_);
                    mem::swap(ui_state, ui_state_);
                } else {
                    eprintln!("Failed to read state! {:?}", result.err().unwrap());
                }
            }
            #[cfg(feature = "export")]
            UiEvent::ExportStateAs => {
                next_frame_events.push(UiEvent::ExportStateAs);
            }
        };
    }
    ui_state.clear_events();
    ui_state.ui_events.append(&mut next_frame_events);
}
