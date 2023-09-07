use macroquad::prelude::*;

use crate::erode::lague;
use crate::heightmap;
use crate::visualize::heightmap_to_texture;
use crate::{erode, partitioning};

const SUBDIVISIONS: u32 = 3;
const ITERATIONS: usize = 1000000;
const EROSION_METHODS: [partitioning::Method; 3] = [
    partitioning::Method::Default,
    partitioning::Method::Subdivision,
    partitioning::Method::SubdivisionOverlap,
];

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
*/

#[derive(Debug, Copy, Clone)]
enum UiWindow {
    Keybinds,
    Toggles,
}

#[derive(Debug, Copy, Clone)]
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
}

fn cycle_erosion_method(erosion_method_index: &mut usize) {
    *erosion_method_index = (*erosion_method_index
        + if is_key_pressed(KeyCode::J) {
            1
        } else if is_key_pressed(KeyCode::K) {
            EROSION_METHODS.len() - 1
        } else {
            0
        })
        % EROSION_METHODS.len();
    print!("Selected method: ");
    match EROSION_METHODS[*erosion_method_index] {
        partitioning::Method::Default => println!("Default (no partitioning)"),
        partitioning::Method::Subdivision => println!("Subdivision"),
        partitioning::Method::SubdivisionOverlap => println!("SubdivisionOverlap"),
    };
}

fn generate_drop_zone(heightmap: &heightmap::Heightmap) -> lague::DropZone {
    lague::DropZone::default(&heightmap)
}

pub async fn visualize() {
    prevent_quit();
    let mut show_keybinds = false;
    let mut restart = true;
    let mut quit = false;
    let mut regenerate = false;
    let mut erosion_method_index: usize = 0;

    let mut heightmap = erode::initialize_heightmap();
    heightmap.normalize();
    let mut heightmap_original = heightmap.clone();
    let mut drop_zone = generate_drop_zone(&heightmap);
    let mut ui_events: Vec<UiEvent> = vec![];

    let keybinds = [
        (vec![KeyCode::G], UiEvent::NewHeightmap),
        (vec![KeyCode::R], UiEvent::Clear),
        (vec![KeyCode::S], UiEvent::Export),
        (vec![KeyCode::H], UiEvent::ToggleUi(UiWindow::Keybinds)),
        (vec![KeyCode::E], UiEvent::RunSimulation),
        (vec![KeyCode::Q], UiEvent::Quit),
        (vec![KeyCode::Escape], UiEvent::Quit),
        (vec![KeyCode::Space], UiEvent::ShowBaseLayer),
        (vec![KeyCode::D], UiEvent::ShowDifference),
        (
            vec![KeyCode::LeftShift, KeyCode::D],
            UiEvent::ShowDifferenceNormalized,
        ),
        (vec![KeyCode::J], UiEvent::NextPartitioningMethod),
        (vec![KeyCode::K], UiEvent::PreviousPartitioningMethod),
    ];

    cycle_erosion_method(&mut erosion_method_index);

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

            for keybind in keybinds.iter() {
                let mut key_is_pressed = true;
                for key in keybind.0.iter() {
                    if !is_key_pressed(*key) {
                        key_is_pressed = false;
                        break;
                    }
                }
                if key_is_pressed {
                    ui_events.push(keybind.1);
                }
            }

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
                        UiWindow::Keybinds => {
                            show_keybinds = !show_keybinds;
                        }
                        UiWindow::Toggles => {}
                    },
                    UiEvent::RunSimulation => {
                        if !eroded {
                            print!("Eroding using ");
                            match EROSION_METHODS[erosion_method_index] {
                                partitioning::Method::Default => {
                                    println!("Default method (no partitioning)");
                                    partitioning::default_erode(
                                        &mut heightmap,
                                        &params,
                                        &drop_zone,
                                    );
                                }
                                partitioning::Method::Subdivision => {
                                    println!("Subdivision method");
                                    partitioning::subdivision_erode(
                                        &mut heightmap,
                                        &params,
                                        SUBDIVISIONS,
                                    );
                                }
                                partitioning::Method::SubdivisionOverlap => {
                                    println!("SubdivisionOverlap method");
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
                        cycle_erosion_method(&mut erosion_method_index);
                    }
                    UiEvent::PreviousPartitioningMethod => {
                        cycle_erosion_method(&mut erosion_method_index);
                    }
                };
            }
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

            egui_macroquad::ui(|egui_ctx| {
                if show_keybinds {
                    egui::Window::new("Keybinds").show(egui_ctx, |ui| {
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
                    });
                }
                egui::Window::new("Toggles").show(egui_ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("[H]");
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
                    });
                    ui.label("0: Default Heightmap");
                });
            });

            egui_macroquad::draw();

            next_frame().await;
        }
    }
}
