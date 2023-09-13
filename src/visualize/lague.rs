use std::collections::HashSet;

use macroquad::prelude::*;

use crate::erode::lague;
use crate::heightmap;
use crate::visualize::heightmap_to_texture;
use crate::{erode, partitioning};

const SUBDIVISIONS: u32 = 3;
const ITERATIONS: usize = 1000000;

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
enum UiWindow {
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
enum UiEvent {
    NewHeightmap,
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
}

impl UiEvent {
    pub fn info(self) -> String {
        match self {
            UiEvent::NewHeightmap => "Generate new heightmap".to_string(),
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
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum UiKey {
    Single(KeyCode),
    Double((KeyCode, KeyCode)),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum UiKeybind {
    Pressed(UiKey, UiEvent),
    Down(UiKey, UiEvent),
}

const KEYCODE_TOGGLE_CONTROL_PANEL_UI: KeyCode = KeyCode::F2;
const KEYCODE_TOGGLE_KEYBINDS_UI: KeyCode = KeyCode::F3;
const KEYCODE_NEXT_PARTITIONING_METHOD: KeyCode = KeyCode::J;
const KEYCODE_PREVIOUS_PARTITIONING_METHOD: KeyCode = KeyCode::K;
const KEYBINDS: [UiKeybind; 14] = [
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
    UiKeybind::Pressed(UiKey::Single(KeyCode::E), UiEvent::RunSimulation),
    UiKeybind::Pressed(UiKey::Single(KeyCode::Q), UiEvent::Quit),
    UiKeybind::Pressed(UiKey::Single(KeyCode::Escape), UiEvent::Quit),
    UiKeybind::Down(UiKey::Single(KeyCode::Space), UiEvent::ShowBaseLayer),
    UiKeybind::Down(UiKey::Single(KeyCode::D), UiEvent::ShowDifference),
    UiKeybind::Down(
        UiKey::Double((KeyCode::LeftShift, KeyCode::D)),
        UiEvent::ShowDifferenceNormalized,
    ),
    UiKeybind::Pressed(UiKey::Single(KEYCODE_NEXT_PARTITIONING_METHOD), UiEvent::NextPartitioningMethod),
    UiKeybind::Pressed(
        UiKey::Single(KEYCODE_PREVIOUS_PARTITIONING_METHOD),
        UiEvent::PreviousPartitioningMethod,
    ),
];

fn poll_ui_keybinds(events: &mut Vec<UiEvent>) {
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
                        events.push(event);
                    }
                }
            }
            UiKeybind::Down(keybind, event) => match keybind {
                UiKey::Single(_) => (),
                UiKey::Double(key_codes) => {
                    if is_key_down(key_codes.0)
                        && is_key_down(key_codes.1)
                        && !consumed_keys.contains(&key_codes.1)
                    {
                        consumed_keys.insert(key_codes.1);
                        events.push(event);
                    }
                }
            }
        }
    }
    for &keybind in KEYBINDS.iter() {
        match keybind {
            UiKeybind::Pressed(keybind, event) => match keybind {
                UiKey::Single(key_code) => {
                    if is_key_pressed(key_code) && !consumed_keys.contains(&key_code) {
                        consumed_keys.insert(key_code);
                        events.push(event);
                    }
                }
                UiKey::Double(_) => (),
            }
            UiKeybind::Down(keybind, event) => match keybind {
                UiKey::Single(key_code) => {
                    if is_key_down(key_code) && !consumed_keys.contains(&key_code) {
                        consumed_keys.insert(key_code);
                        events.push(event);
                    }
                }
                UiKey::Double(_) => (),
            }
        }
    }
}

fn generate_drop_zone(heightmap: &heightmap::Heightmap) -> lague::DropZone {
    lague::DropZone::default(&heightmap)
}

pub async fn visualize() {
    prevent_quit();
    let mut erosion_method = partitioning::Method::Default;
    let mut show_ui = true;
    let mut show_keybinds = false;
    let mut show_control_panel = true;
    let mut restart = true;
    let mut quit = false;
    let mut regenerate = false;

    let mut heightmap = erode::initialize_heightmap();
    heightmap.normalize();
    let mut heightmap_original = heightmap.clone();
    let mut drop_zone = generate_drop_zone(&heightmap);
    let mut ui_events: Vec<UiEvent> = vec![];
    let mut ui_events_previous: Vec<UiEvent> = vec![];

    while restart && !quit {
        restart = false;
        let mut eroded = false;

        if regenerate {
            heightmap = erode::initialize_heightmap();
            heightmap.normalize();
            heightmap_original = heightmap.clone();
            drop_zone = generate_drop_zone(&heightmap);
            regenerate = false;
        }

        let heightmap_texture = heightmap_to_texture(&heightmap_original);
        let params = lague::Parameters {
            num_iterations: ITERATIONS,
            ..lague::Parameters::default()
        };
        let mut heightmap_eroded_texture = None;
        let mut heightmap_diff = heightmap.subtract(&heightmap_original).unwrap();
        let mut heightmap_diff_texture = None;
        heightmap_diff.normalize();
        let mut heightmap_diff_normalized = None;

        while !is_quit_requested() && !restart && !quit {
            poll_ui_keybinds(&mut ui_events);

            let mut active_texture = if let Some(eroded_texture) = heightmap_eroded_texture {
                eroded_texture
            } else {
                heightmap_texture
            };

            for event in ui_events.iter() {
                match event {
                    UiEvent::NewHeightmap => {
                        println!("Regenerating heightmap");
                        restart = true;
                        regenerate = true;
                    }
                    UiEvent::Clear => {
                        println!("Restarting");
                        restart = true;
                    }
                    UiEvent::Export => {
                        heightmap::export_heightmaps(
                            vec![&heightmap_original, &heightmap, &heightmap_diff],
                            vec![
                                "output/heightmap",
                                "output/heightmap_eroded",
                                "output/heightmap_diff",
                            ],
                        );
                    }
                    UiEvent::ToggleUi(ui_window) => match ui_window {
                        UiWindow::All => {
                            show_ui = !show_ui;
                        }
                        UiWindow::Keybinds => {
                            show_keybinds = !show_keybinds;
                        }
                        UiWindow::ControlPanel => {
                            show_control_panel = !show_control_panel;
                        }
                    },
                    UiEvent::RunSimulation => {
                        if !eroded {
                            print!("Eroding using ");
                            match erosion_method {
                                partitioning::Method::Default => {
                                    println!(
                                        "{} method (no partitioning)",
                                        partitioning::Method::Default.to_string()
                                    );
                                    partitioning::default_erode(
                                        &mut heightmap,
                                        &params,
                                        &drop_zone,
                                    );
                                }
                                partitioning::Method::Subdivision => {
                                    println!(
                                        "{} method",
                                        partitioning::Method::Subdivision.to_string()
                                    );
                                    partitioning::subdivision_erode(
                                        &mut heightmap,
                                        &params,
                                        SUBDIVISIONS,
                                    );
                                }
                                partitioning::Method::SubdivisionOverlap => {
                                    println!(
                                        "{} method",
                                        partitioning::Method::SubdivisionOverlap.to_string()
                                    );
                                    partitioning::subdivision_overlap_erode(
                                        &mut heightmap,
                                        &params,
                                        SUBDIVISIONS,
                                    );
                                }
                            }
                            heightmap_eroded_texture = Some(heightmap_to_texture(&heightmap));
                            heightmap_diff = heightmap.subtract(&heightmap_original).unwrap();
                            heightmap_diff_texture = Some(heightmap_to_texture(&heightmap_diff));
                            heightmap_diff.normalize();
                            heightmap_diff_normalized = Some(heightmap_to_texture(&heightmap_diff));
                            println!("Done!");
                        }
                        eroded = true;
                    }
                    UiEvent::Quit => {
                        println!("Quitting...");
                        quit = true;
                    }
                    UiEvent::ShowBaseLayer => {
                        active_texture = heightmap_texture;
                    }
                    UiEvent::ShowDifference => {
                        if let Some(texture) = heightmap_diff_texture {
                            active_texture = texture;
                        }
                    }
                    UiEvent::ShowDifferenceNormalized => {
                        if let Some(texture) = heightmap_diff_normalized {
                            active_texture = texture;
                        }
                    }
                    UiEvent::NextPartitioningMethod => {
                        erosion_method = erosion_method.next();
                        println!("Selected {} method.", erosion_method.to_string());
                    }
                    UiEvent::PreviousPartitioningMethod => {
                        erosion_method = erosion_method.previous();
                        println!("Selected {} method.", erosion_method.to_string());
                    }
                    UiEvent::SelectMethod(method) => {
                        erosion_method = *method;
                        println!("Selected {} method.", erosion_method.to_string());
                    }
                };
            }
            (ui_events_previous, ui_events) = (ui_events, ui_events_previous);
            ui_events.clear();

            draw_texture_ex(
                active_texture,
                0.0,
                0.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(crate::WIDTH as f32, crate::HEIGHT as f32)),
                    ..Default::default()
                },
            );

            if show_ui {
                egui_macroquad::ui(|egui_ctx| {
                    if show_keybinds {
                        egui::Window::new(format!("Keybinds [{:?}]", KEYCODE_TOGGLE_KEYBINDS_UI)).show(egui_ctx, |ui| {
                            for keybind in KEYBINDS {
                                match keybind {
                                    UiKeybind::Pressed(keys, event) => {
                                        ui.horizontal(|ui| {
                                            if ui.button(event.info()).clicked() {
                                                ui_events.push(event);
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
                                        if ui_events_previous.contains(&event) {
                                            ui.label(event.info());
                                        } else {
                                            if ui.button(event.info()).clicked() {
                                                ui_events.push(event);
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
                        });
                    }

                    if show_control_panel {
                        egui::Window::new(format!("Control Panel [{:?}]", KEYCODE_TOGGLE_CONTROL_PANEL_UI)).show(egui_ctx, |ui| {
                            // Erosion Method Selection
                            ui.heading("Erosion Method Selection");
                            for &method in partitioning::Method::iterator() {
                                if method == erosion_method {
                                    ui.label(method.to_string());
                                } else {
                                    ui.horizontal(|ui| {
                                        if ui.button(method.to_string()).clicked() {
                                            ui_events.push(UiEvent::SelectMethod(method));
                                        }
                                        if method == erosion_method.next() {
                                        ui.label(format!("{:?}", KEYCODE_NEXT_PARTITIONING_METHOD));
                                        } else if method == erosion_method.previous() {
                                            ui.label(format!("{:?}", KEYCODE_PREVIOUS_PARTITIONING_METHOD));
                                        }
                                    });
                                }
                            }

                            ui.heading("Toggles");
                            // Show/Hide Keybinds
                            ui.horizontal(|ui| {
                                if ui
                                    .button(if show_keybinds {
                                        "Hide Keybinds"
                                    } else {
                                        "Show Keybinds"
                                    })
                                    .clicked()
                                {
                                    show_keybinds = !show_keybinds;
                                };
                                ui.label(format!("{:?}", KEYCODE_TOGGLE_KEYBINDS_UI));
                            });

                            // Image Layers
                            ui.heading("Image Layers");
                            ui.label("0: Default Heightmap");
                        });
                    }
                });

                egui_macroquad::draw();
            }

            next_frame().await;
        }
    }
}
