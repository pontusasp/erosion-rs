use bracket_noise::prelude::NoiseType;
use egui::{Color32, Vec2};
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};

use crate::{erode::Parameters, heightmap::HeightmapSettings, partitioning};

use super::{
    canvas::Canvas,
    ui::{
        UiEvent, UiState, KEYCODE_NEW_HEIGHTMAP,
        KEYCODE_NEXT_PARTITIONING_METHOD, KEYCODE_PREVIOUS_PARTITIONING_METHOD,
    },
    AppState, SimulationState,
};

pub fn plot_height(ui: &mut egui::Ui, state: &mut AppState) {
    let width = 800.0;
    let height = 500.0;
    let mut canvas = Canvas::new(
        Vec2::new(width, height),
        egui::Stroke::new(1.0, Color32::WHITE),
    );
    canvas.draw(ui);

    let heightmap = state.simulation_state().get_active();
    let max_height = heightmap.depth;

    let heights_along_y: Vec<f32> = heightmap
        .data
        .par_iter()
        .map(|col| col.par_iter().cloned().reduce(|| 0.0, |a, b| a + b) / col.len() as f32)
        .collect();

    let heights_along_x: Vec<f32> = {
        let mut heights = Vec::new();

        for y in 0..heightmap.height {
            heights.push(0.0);
            for x in 0..heightmap.width {
                let height = heightmap.data[x][y];
                heights[y] += height;
            }
            heights[y] /= heightmap.width as f32;
        }

        heights
    };

    canvas.stroke.color = Color32::BLUE;
    draw_polyline(ui, &heights_along_y, &canvas, width, height, max_height);
    canvas.draw_line(ui, Vec2::new(10.0, 10.0), Vec2::new(30.0, 10.0));

    canvas.stroke.color = Color32::RED;
    draw_polyline(ui, &heights_along_x, &canvas, width, height, max_height);
    canvas.draw_line(ui, Vec2::new(10.0, 10.0), Vec2::new(10.0, 30.0));
}

fn draw_polyline(
    ui: &mut egui::Ui,
    points: &Vec<f32>,
    canvas: &Canvas,
    width: f32,
    height: f32,
    max_height: f32,
) {
    for i in 1..points.len() {
        let progress0 = (i - 1) as f32 / (points.len() - 1) as f32;
        let progress1 = i as f32 / (points.len() - 1) as f32;
        let start = Vec2::new(progress0 * width, points[i - 1] / max_height * height);
        let end = Vec2::new(progress1 * width, points[i] / max_height * height);
        canvas.draw_line(ui, start, end);
    }
}

pub fn post_processing(ui: &mut egui::Ui, ui_state: &mut UiState) {
    egui::CollapsingHeader::new("Post Processing")
        .default_open(true)
        .show(ui, |ui| {
            ui.add(
                egui::Slider::new(&mut ui_state.blur_sigma, 0.0..=20.0).text("Gaussian Blur Sigma"),
            );
            if ui.button("Blur").clicked() {
                ui_state.ui_events.push(UiEvent::Blur);
            }
            let (mut canny_low, mut canny_high) = ui_state.canny_edge;
            ui.add(egui::Slider::new(&mut canny_low, 0.0001..=canny_high).text("Lower Threshold"));
            ui.add(egui::Slider::new(&mut canny_high, canny_low..=120.0).text("Upper Threshold"));
            ui_state.canny_edge = (canny_low, canny_high);
            if ui.button("Edge Detect").clicked() {
                ui_state.ui_events.push(UiEvent::EdgeDetect);
            }
        });
    ui.separator();
}

pub fn erosion_method_selection(ui: &mut egui::Ui, ui_state: &mut UiState, state: &AppState) {
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
}

pub fn erosion_parameter_selection(ui: &mut egui::Ui, ui_state: &mut UiState, state: &mut AppState) {
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
}
pub fn layer_selection(ui: &mut egui::Ui, ui_state: &mut UiState, state: &AppState) {
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
}
pub fn heightmap_generation_settings(ui: &mut egui::Ui, ui_state: &mut UiState, state: &mut AppState) {
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
    ui.separator();
}