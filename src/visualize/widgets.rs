use bracket_noise::prelude::NoiseType;
use egui::{Color32, Rect, Vec2};

use crate::{erode::Parameters, heightmap::HeightmapSettings, partitioning};

use super::{
    canvas::Canvas,
    ui::{
        UiEvent, UiKey, UiKeybind, UiState, UiWindow, KEYBINDS, KEYCODE_NEW_HEIGHTMAP,
        KEYCODE_NEXT_PARTITIONING_METHOD, KEYCODE_PREVIOUS_PARTITIONING_METHOD,
        KEYCODE_TOGGLE_ALL_UI, KEYCODE_TOGGLE_CONTROL_PANEL_UI, KEYCODE_TOGGLE_KEYBINDS_UI,
        KEYCODE_TOGGLE_METADATA_UI, KEYCODE_TOGGLE_METRICS_UI,
    },
    AppState, SimulationState,
};

pub fn ui_top_panel(egui_ctx: &egui::Context, ui_state: &mut UiState) {
    egui::TopBottomPanel::top("top_panel").show(egui_ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            ui.heading("Erosion RS");
            if ui
                .button(format!(
                    "[{:?}] {} UI",
                    KEYCODE_TOGGLE_ALL_UI,
                    if ui_state.show_ui_all { "Hide" } else { "Show" }
                ))
                .clicked()
            {
                ui_state.ui_events.push(UiEvent::ToggleUi(UiWindow::All));
            }
            if ui
                .button(format!(
                    "[{:?}] {} Control Panel",
                    KEYCODE_TOGGLE_CONTROL_PANEL_UI,
                    if ui_state.show_ui_control_panel {
                        "Hide"
                    } else {
                        "Show"
                    }
                ))
                .clicked()
            {
                ui_state
                    .ui_events
                    .push(UiEvent::ToggleUi(UiWindow::ControlPanel));
            }
            if ui
                .button(format!(
                    "[{:?}] {} Keybinds",
                    KEYCODE_TOGGLE_KEYBINDS_UI,
                    if ui_state.show_ui_keybinds {
                        "Hide"
                    } else {
                        "Show"
                    }
                ))
                .clicked()
            {
                ui_state
                    .ui_events
                    .push(UiEvent::ToggleUi(UiWindow::Keybinds));
            };
            if ui
                .button(format!(
                    "[{:?}] {} Metadata",
                    KEYCODE_TOGGLE_METADATA_UI,
                    if ui_state.show_ui_metadata {
                        "Hide"
                    } else {
                        "Show"
                    }
                ))
                .clicked()
            {
                ui_state
                    .ui_events
                    .push(UiEvent::ToggleUi(UiWindow::Metadata));
            };
            if ui
                .button(format!(
                    "[{:?}] {} Metrics",
                    KEYCODE_TOGGLE_METRICS_UI,
                    if ui_state.show_ui_metrics {
                        "Hide"
                    } else {
                        "Show"
                    }
                ))
                .clicked()
            {
                ui_state
                    .ui_events
                    .push(UiEvent::ToggleUi(UiWindow::Metrics));
            };
        });
    });
}

pub fn ui_side_panel(egui_ctx: &egui::Context, ui_state: &mut UiState, state: &mut AppState) {
    egui::SidePanel::left("left_panel").show_animated(
        egui_ctx,
        ui_state.show_ui_control_panel,
        |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                // Erosion Method Selection
                egui::CollapsingHeader::new("Erosion Method Selection")
                    .default_open(true)
                    .show(ui, |ui| {
                        for &method in partitioning::Method::iterator() {
                            if method == state.simulation_state().base().erosion_method {
                                ui.label(format!("-> {}", method.to_string()));
                            } else {
                                ui.horizontal(|ui| {
                                    if ui.button(method.to_string()).clicked() {
                                        ui_state.ui_events.push(UiEvent::SelectMethod(method));
                                    }
                                    if method
                                        == state.simulation_state().base().erosion_method.next()
                                    {
                                        ui.label(format!("{:?}", KEYCODE_NEXT_PARTITIONING_METHOD));
                                    } else if method
                                        == state.simulation_state().base().erosion_method.previous()
                                    {
                                        ui.label(format!(
                                            "{:?}",
                                            KEYCODE_PREVIOUS_PARTITIONING_METHOD
                                        ));
                                    }
                                });
                            }
                        }

                        egui::CollapsingHeader::new("Partitioning Parameters")
                            .default_open(true)
                            .show(ui, |ui| {
                                ui.label("coming soon...");
                            });
                    });

                ui.separator();

                egui::CollapsingHeader::new("Erosion Parameters")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.add(
                            egui::Slider::new(
                                &mut state.parameters.erosion_params.erosion_radius,
                                0..=5,
                            )
                            .text("Erosion Radius"),
                        )
                        .changed();
                        ui.add(
                            egui::Slider::new(
                                &mut state.parameters.erosion_params.inertia,
                                0.0..=5.5,
                            )
                            .text("Inertia"),
                        )
                        .changed();
                        ui.add(
                            egui::Slider::new(
                                &mut state.parameters.erosion_params.sediment_capacity_factor,
                                0.0..=5.5,
                            )
                            .text("Sediment Capacity Factor"),
                        )
                        .changed();
                        ui.add(
                            egui::Slider::new(
                                &mut state.parameters.erosion_params.min_sediment_capacity,
                                0.0..=5.5,
                            )
                            .text("Min Sediment Capacity"),
                        )
                        .changed();
                        ui.add(
                            egui::Slider::new(
                                &mut state.parameters.erosion_params.erode_speed,
                                0.0..=5.5,
                            )
                            .text("Erode Speed"),
                        )
                        .changed();
                        ui.add(
                            egui::Slider::new(
                                &mut state.parameters.erosion_params.deposit_speed,
                                0.0..=5.5,
                            )
                            .text("Deposit Speed"),
                        )
                        .changed();
                        ui.add(
                            egui::Slider::new(
                                &mut state.parameters.erosion_params.evaporate_speed,
                                0.0..=5.5,
                            )
                            .text("Evaporate Speed"),
                        )
                        .changed();
                        ui.add(
                            egui::Slider::new(
                                &mut state.parameters.erosion_params.gravity,
                                0.0..=5.5,
                            )
                            .text("Gravity"),
                        )
                        .changed();
                        ui.add(
                            egui::Slider::new(
                                &mut state.parameters.erosion_params.max_droplet_lifetime,
                                0..=5,
                            )
                            .text("Max Droplet Lifetime"),
                        )
                        .changed();
                        ui.add(
                            egui::Slider::new(
                                &mut state.parameters.erosion_params.initial_water_volume,
                                0.0..=5.5,
                            )
                            .text("Initial Water Volume"),
                        )
                        .changed();
                        ui.add(
                            egui::Slider::new(
                                &mut state.parameters.erosion_params.initial_speed,
                                0.0..=5.5,
                            )
                            .text("Initial Speed"),
                        )
                        .changed();
                        ui.add(
                            egui::Slider::new(
                                &mut state.parameters.erosion_params.num_iterations,
                                0..=2000000,
                            )
                            .text("Num Iterations"),
                        )
                        .changed();

                        if ui.button("Reset").clicked() {
                            state.parameters.erosion_params = Parameters::default();
                        }
                    });

                ui.separator();

                egui::CollapsingHeader::new("Layers")
                    .default_open(true)
                    .show(ui, |ui| {
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
                                if *state.simulation_base_indices.last().unwrap() == simulation.id()
                                {
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

                ui.separator();

                egui::CollapsingHeader::new("Heightmap Generation")
                    .default_open(true)
                    .show(ui, |ui| {
                        if state.simulation_state().eroded().is_none()
                            && state.simulation_state().id()
                                == state.simulation_base_indices.len() - 1
                        {
                            let mut updated = false;

                            updated = updated
                                || ui
                                    .add(
                                        egui::Slider::new(
                                            &mut state.parameters.heightmap_settings.seed,
                                            0..=10000000000,
                                        )
                                        .text("Seed"),
                                    )
                                    .changed();

                            let noise_type = state.parameters.heightmap_settings.noise_type;
                            egui::ComboBox::from_label("Noise Type")
                                .selected_text(format!(
                                    "{:?}",
                                    state.parameters.heightmap_settings.noise_type
                                ))
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut state.parameters.heightmap_settings.noise_type,
                                        NoiseType::Value,
                                        "Value",
                                    );
                                    ui.selectable_value(
                                        &mut state.parameters.heightmap_settings.noise_type,
                                        NoiseType::ValueFractal,
                                        "Value Fractal",
                                    );
                                    ui.selectable_value(
                                        &mut state.parameters.heightmap_settings.noise_type,
                                        NoiseType::Perlin,
                                        "Perlin",
                                    );
                                    ui.selectable_value(
                                        &mut state.parameters.heightmap_settings.noise_type,
                                        NoiseType::PerlinFractal,
                                        "Perlin
    Fractal",
                                    );
                                    ui.selectable_value(
                                        &mut state.parameters.heightmap_settings.noise_type,
                                        NoiseType::Simplex,
                                        "Simplex",
                                    );
                                    ui.selectable_value(
                                        &mut state.parameters.heightmap_settings.noise_type,
                                        NoiseType::SimplexFractal,
                                        "Simplex Fractal",
                                    );
                                    ui.selectable_value(
                                        &mut state.parameters.heightmap_settings.noise_type,
                                        NoiseType::Cellular,
                                        "Cellular",
                                    );
                                    ui.selectable_value(
                                        &mut state.parameters.heightmap_settings.noise_type,
                                        NoiseType::WhiteNoise,
                                        "WhiteNoise",
                                    );
                                    ui.selectable_value(
                                        &mut state.parameters.heightmap_settings.noise_type,
                                        NoiseType::Cubic,
                                        "Cubic",
                                    );
                                    ui.selectable_value(
                                        &mut state.parameters.heightmap_settings.noise_type,
                                        NoiseType::CubicFractal,
                                        "Cubic Fractal",
                                    );
                                });
                            updated = updated
                                || noise_type != state.parameters.heightmap_settings.noise_type;

                            updated = updated
                                || ui
                                    .add(
                                        egui::Slider::new(
                                            &mut state
                                                .parameters
                                                .heightmap_settings
                                                .fractal_octaves,
                                            0..=28,
                                        )
                                        .text("Fractal Octaves"),
                                    )
                                    .drag_released();
                            updated = updated
                                || ui
                                    .add(
                                        egui::Slider::new(
                                            &mut state.parameters.heightmap_settings.fractal_gain,
                                            0.0..=2.0,
                                        )
                                        .text("Fractal Gain"),
                                    )
                                    .changed();
                            updated = updated
                                || ui
                                    .add(
                                        egui::Slider::new(
                                            &mut state
                                                .parameters
                                                .heightmap_settings
                                                .fractal_lacunarity,
                                            0.0..=7.0,
                                        )
                                        .text("Fractal Lacunarity"),
                                    )
                                    .drag_released();
                            updated = updated
                                || ui
                                    .add(
                                        egui::Slider::new(
                                            &mut state.parameters.heightmap_settings.frequency,
                                            0.0..=5.0,
                                        )
                                        .text("Frequency"),
                                    )
                                    .changed();
                            let mut size = state.parameters.heightmap_settings.width;
                            updated = updated
                                || ui
                                    .add(egui::Slider::new(&mut size, 64..=1024).text("Resolution"))
                                    .changed();
                            state.parameters.heightmap_settings.width = size;
                            state.parameters.heightmap_settings.height = size;

                            ui.add(egui::Checkbox::new(
                                &mut state.parameters.auto_apply,
                                "Auto Apply",
                            ));

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
                        } else {
                            ui.label("Parameters only available for new base layers.");
                            if ui
                                .button(format!(
                                    "[{:?}] Create new base layer",
                                    KEYCODE_NEW_HEIGHTMAP
                                ))
                                .clicked()
                            {
                                ui_state.ui_events.push(UiEvent::NewHeightmap);
                            }
                        }
                    });
            });
        },
    );
}

pub fn ui_keybinds_window(egui_ctx: &egui::Context, ui_state: &mut UiState) {
    if ui_state.show_ui_keybinds {
        egui::Window::new(format!("Keybinds [{:?}]", KEYCODE_TOGGLE_KEYBINDS_UI)).show(
            egui_ctx,
            |ui| {
                for keybind in KEYBINDS {
                    match keybind {
                        UiKeybind::Pressed(keys, event) => {
                            ui.horizontal(|ui| {
                                if ui.button(event.info()).clicked() {
                                    ui_state.ui_events.push(*event);
                                }
                                match keys {
                                    UiKey::Single(key_code) => {
                                        ui.label(format!("[{:?}]", key_code))
                                    }
                                    UiKey::Double(key_codes) => {
                                        ui.label(format!("[{:?}-{:?}]", key_codes.0, key_codes.1))
                                    }
                                };
                            });
                        }
                        UiKeybind::Down(keys, event) => {
                            if ui_state.ui_events_previous.contains(&event) {
                                ui.label(event.info());
                            } else {
                                if ui.button(event.info()).clicked() {
                                    ui_state.ui_events.push(*event);
                                }
                            }
                            match keys {
                                UiKey::Single(key_code) => ui.label(format!("({:?})", key_code)),
                                UiKey::Double(key_codes) => {
                                    ui.label(format!("({:?}-{:?})", key_codes.0, key_codes.1))
                                }
                            };
                        }
                    }
                }
            },
        );
    }
}

pub fn ui_metadata_window(egui_ctx: &egui::Context, ui_state: &mut UiState, state: &mut AppState) {
    if ui_state.show_ui_metadata {
        egui::Window::new(format!("Metadata")).show(egui_ctx, |ui| {
            ui.heading("Base Heightmap");
            ui.label(format!(
                "Width x Height: {} x {}",
                state.simulation_state().base().heightmap_base.width,
                state.simulation_state().base().heightmap_base.height
            ));
            ui.label(format!(
                "Depth: {}",
                state.simulation_state().base().heightmap_base.depth
            ));
            ui.label(format!(
                "Original Depth: {}",
                state
                    .simulation_state()
                    .base()
                    .heightmap_base
                    .original_depth
            ));
            if let Some(height) = state
                .simulation_state()
                .get_heightmap()
                .get_average_height()
            {
                ui.label(format!("Average Height: {}", height));
            }
            if let Some(height) = state.simulation_state().base().heightmap_base.total_height {
                ui.label(format!("Total Depth: {}", height));
            }
            if let Some(metadata) = state
                .simulation_state()
                .base()
                .heightmap_base
                .metadata
                .clone()
            {
                for (k, v) in metadata.iter() {
                    ui.label(format!("{}: {}", k, v));
                }
            }
            if let Some(eroded) = state.simulation_state().eroded() {
                ui.heading("Eroded Heightmap");
                ui.label(format!(
                    "Width x Height: {} x {}",
                    eroded.heightmap_eroded.width,
                    state.simulation_state().base().heightmap_base.height
                ));
                ui.label(format!("Depth: {}", eroded.heightmap_eroded.depth));
                ui.label(format!(
                    "Original Depth: {}",
                    eroded.heightmap_eroded.original_depth
                ));
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
}

pub fn ui_metrics_window(
    egui_ctx: &egui::Context,
    ui_state: &mut UiState,
    state: &mut AppState,
) -> Option<Rect> {
    let mut rect = None;
    if ui_state.show_ui_metrics {
        rect = Some(
            egui::Window::new(format!("Metrics [{:?}]", KEYCODE_TOGGLE_METRICS_UI))
                .show(egui_ctx, |ui| {
                    plot_height(ui, state);
                    plot_height(ui, state);
                })
                .unwrap()
                .response
                .rect,
        );
    }
    rect
}

fn plot_height(ui: &mut egui::Ui, state: &mut AppState) {
    let width = 200.0;
    let height = 200.0;
    let mut canvas = Canvas::new(
        Vec2::new(width, height),
        egui::Stroke::new(1.0, Color32::WHITE),
    );
    canvas.draw(ui);
    canvas.draw_circle(ui, Vec2::new(50.0, 50.0), 10.0, Color32::RED);
}
