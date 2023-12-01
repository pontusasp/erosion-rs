use bracket_noise::prelude::NoiseType;
use egui::{Color32, Vec2};
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};

use crate::heightmap::{HeightmapParameters, HeightmapType};
use crate::visualize::events::UiEvent;
use crate::visualize::keybinds::{
    KEYCODE_NEW_HEIGHTMAP, KEYCODE_NEXT_PARTITIONING_METHOD, KEYCODE_PREVIOUS_PARTITIONING_METHOD,
};
use crate::visualize::ui::UiState;
use crate::{
    erode::Parameters, heightmap::ProceduralHeightmapSettings, partitioning,
    GAUSSIAN_BLUR_BOUNDARY_THICKNESS_MAX, GAUSSIAN_BLUR_BOUNDARY_THICKNESS_MIN,
    GAUSSIAN_BLUR_SIGMA_RANGE_MAX, GAUSSIAN_BLUR_SIGMA_RANGE_MIN, GRID_SIZE_RANGE_MAX,
    GRID_SIZE_RANGE_MIN,
};

use super::{canvas::Canvas, AppState, SimulationState};

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
                egui::Slider::new(
                    &mut ui_state.blur_sigma,
                    GAUSSIAN_BLUR_SIGMA_RANGE_MIN..=GAUSSIAN_BLUR_SIGMA_RANGE_MAX,
                )
                .text("Gaussian Blur Sigma"),
            );
            if ui.button("Blur").clicked() {
                ui_state.ui_events.push(UiEvent::Blur);
            }
            let (mut canny_low, mut canny_high) = ui_state.canny_edge;
            let mut updated = false;
            updated = updated
                || ui
                    .add(
                        egui::Slider::new(&mut canny_low, 0.0001..=canny_high)
                            .text("Lower Threshold"),
                    )
                    .changed();
            updated = updated
                || ui
                    .add(
                        egui::Slider::new(&mut canny_high, canny_low..=120.0)
                            .text("Upper Threshold"),
                    )
                    .changed();
            ui_state.canny_edge = (canny_low, canny_high);
            if ui.button("Edge Detect").clicked() || updated {
                ui_state.ui_events.push(UiEvent::EdgeDetect);
            }
            if ui.button("Blur + Edge Detect").clicked() {
                ui_state.ui_events.push(UiEvent::BlurEdgeDetect);
            }

            ui.separator();

            let mut props = ui_state.isoline;
            let mut updated = false;
            updated = updated
                || ui
                    .add(egui::Slider::new(&mut props.height, 0.0..=1.0).text("Isoline value"))
                    .changed();
            updated = updated
                || ui
                    .add(egui::Slider::new(&mut props.error, 0.0..=0.1).text("Isoline error"))
                    .changed();
            if ui.button("Show isoline").clicked() {
                updated = true;
            }

            let should_flood_inside_ = props.flood_lower.clone();
            updated = updated
                || ui
                    .toggle_value(
                        &mut props.flood_lower,
                        if should_flood_inside_ {
                            "Flooding inside"
                        } else {
                            "Flooding outside"
                        },
                    )
                    .changed();
            let should_flood_ = props.should_flood.clone();
            updated = updated
                || ui
                    .toggle_value(
                        &mut props.should_flood,
                        if should_flood_ {
                            "Disable Flooding"
                        } else {
                            "Enable Flooding"
                        },
                    )
                    .changed();

            let blur_augmentation_ = props.blur_augmentation.0.clone();
            updated = updated
                || ui
                    .toggle_value(
                        &mut props.blur_augmentation.0,
                        if blur_augmentation_ {
                            "Blur augmentation active"
                        } else {
                            "Blur augmentation inactive"
                        },
                    )
                    .changed();
            if blur_augmentation_ {
                updated = updated
                    || ui
                        .add(
                            egui::Slider::new(&mut props.blur_augmentation.1, 0.0..=5.0)
                                .text("Blur amount"),
                        )
                        .changed();
                updated = updated
                    || ui
                        .add(
                            egui::Slider::new(&mut props.blur_augmentation.2, 0..=10)
                                .text("Noise Reduction Kernel"),
                        )
                        .changed();
                updated = updated
                    || ui
                        .add(
                            egui::Slider::new(&mut props.blur_augmentation.3, 0..=10)
                                .text("Noise Reduction Iterations"),
                        )
                        .changed();
            }
            updated = updated
                || ui
                    .toggle_value(&mut props.advanced_texture, "Advanced Visualization")
                    .changed();

            let lower = props.flooded_areas_lower.unwrap_or(0);
            let higher = props.flooded_areas_higher.unwrap_or(0);
            let percentage = if higher > 0 {
                lower as f32 / (lower + higher) as f32 * 100.0
            } else {
                0.0
            };
            ui.label(format!(
                "Flooded {} / {} areas ({}%)",
                lower,
                lower + higher,
                percentage
            ));

            if updated {
                ui_state.isoline = props;
                ui_state.ui_events.push(UiEvent::Isoline);
            }
        });
    ui.separator();
}

pub fn erosion_method_selection(ui: &mut egui::Ui, ui_state: &mut UiState, state: &mut AppState) {
    egui::CollapsingHeader::new("Erosion Method Selection")
        .default_open(true)
        .show(ui, |ui| {
            for &method in partitioning::Method::iterator() {
                if method.matches(&state.simulation_state().base().erosion_method) {
                    ui.label(format!("-> {}", method.to_string()));
                } else {
                    ui.horizontal(|ui| {
                        if ui.button(method.to_string()).clicked() {
                            ui_state.ui_events.push(UiEvent::SelectMethod(method));
                        }
                        if method.matches(&state.simulation_state().base().erosion_method.next()) {
                            ui.label(format!("{:?}", KEYCODE_NEXT_PARTITIONING_METHOD));
                        } else if method
                            .matches(&state.simulation_state().base().erosion_method.previous())
                        {
                            ui.label(format!("{:?}", KEYCODE_PREVIOUS_PARTITIONING_METHOD));
                        }
                    });
                }
            }

            egui::CollapsingHeader::new("Partitioning Parameters")
                .default_open(true)
                .show(ui, |ui| {
                    {
                        let g = state.parameters.grid_size;
                        state
                            .simulation_state_mut()
                            .base_mut()
                            .erosion_method
                            .set_grid_size_unchecked(g);
                    }
                    match state.simulation_state_mut().base_mut().erosion_method {
                        partitioning::Method::Default => (),
                        partitioning::Method::Subdivision(ref mut grid_size)
                        | partitioning::Method::SubdivisionOverlap(ref mut grid_size) => {
                            ui.add(
                                egui::Slider::new(
                                    grid_size,
                                    GRID_SIZE_RANGE_MIN..=GRID_SIZE_RANGE_MAX,
                                )
                                .text("Grid Size"),
                            );
                            state.parameters.grid_size = *grid_size;
                        }
                        partitioning::Method::SubdivisionBlurBoundary((
                            ref mut grid_size,
                            (ref mut sigma, ref mut thickness),
                        )) => {
                            ui.add(
                                egui::Slider::new(
                                    grid_size,
                                    GRID_SIZE_RANGE_MIN..=GRID_SIZE_RANGE_MAX,
                                )
                                .text("Grid Size"),
                            );
                            ui.add(
                                egui::Slider::new(
                                    sigma,
                                    GAUSSIAN_BLUR_SIGMA_RANGE_MIN..=GAUSSIAN_BLUR_SIGMA_RANGE_MAX,
                                )
                                .text("Gaussian Blur Sigma"),
                            );
                            ui.add(
                                egui::Slider::new(
                                    thickness,
                                    GAUSSIAN_BLUR_BOUNDARY_THICKNESS_MIN
                                        ..=GAUSSIAN_BLUR_BOUNDARY_THICKNESS_MAX,
                                )
                                .text("Gaussian Blur Boundary Thickness"),
                            );
                            state.parameters.grid_size = *grid_size;
                        }
                        partitioning::Method::GridOverlapBlend(ref mut grid_size) => {
                            ui.add(
                                egui::Slider::new(
                                    grid_size,
                                    GRID_SIZE_RANGE_MIN..=GRID_SIZE_RANGE_MAX,
                                )
                                .text("Grid Size"),
                            );
                            state.parameters.grid_size = *grid_size;
                        }
                    };
                    ui.toggle_value(&mut state.parameters.margin, "Use Margin");
                    ui.toggle_value(&mut ui_state.show_grid, "Show Grid");
                });
        });

    ui.separator();
}

pub fn erosion_parameter_selection(ui: &mut egui::Ui, state: &mut AppState) {
    egui::CollapsingHeader::new("Erosion Parameters")
        .default_open(true)
        .show(ui, |ui| {
            ui.add(
                egui::Slider::new(&mut state.parameters.erosion_params.erosion_radius, 0..=5)
                    .text("Erosion Radius"),
            )
            .changed();
            ui.add(
                egui::Slider::new(&mut state.parameters.erosion_params.inertia, 0.0..=5.5)
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
                egui::Slider::new(&mut state.parameters.erosion_params.erode_speed, 0.0..=5.5)
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
                egui::Slider::new(&mut state.parameters.erosion_params.gravity, 0.0..=5.5)
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
                    0..=10_000_000,
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
pub fn layer_selection(ui: &mut egui::Ui, state: &AppState) {
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

    ui.separator();
}

fn heightmap_parameters(
    params: &mut HeightmapParameters,
    ui: &mut egui::Ui,
    ui_state: &mut UiState,
    state: &mut AppState,
) {
    let mut size = params.size;
    let mut updated = ui
        .add(egui::Slider::new(&mut size, 2usize.pow(6)..=2usize.pow(12)).text("Resolution"))
        .changed();
    params.size = size;

    ui.add(egui::Checkbox::new(
        &mut state.parameters.auto_apply,
        "Auto Apply",
    ));

    if ui.button("Reset").clicked() {
        params.reset();
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
}

fn procedural_generation_settings(
    settings: &mut ProceduralHeightmapSettings,
    ui: &mut egui::Ui,
    ui_state: &mut UiState,
    state: &mut AppState,
) {
    let mut updated = false;

    updated = updated
        || ui
            .add(egui::Slider::new(&mut settings.seed, 0..=10000000000).text("Seed"))
            .changed();

    let noise_type = settings.noise_type;
    egui::ComboBox::from_label("Noise Type")
        .selected_text(format!("{:?}", settings.noise_type))
        .show_ui(ui, |ui| {
            ui.selectable_value(&mut settings.noise_type, NoiseType::Value.into(), "Value");
            ui.selectable_value(
                &mut settings.noise_type,
                NoiseType::ValueFractal.into(),
                "Value Fractal",
            );
            ui.selectable_value(&mut settings.noise_type, NoiseType::Perlin.into(), "Perlin");
            ui.selectable_value(
                &mut settings.noise_type,
                NoiseType::PerlinFractal.into(),
                "Perlin
    Fractal",
            );
            ui.selectable_value(
                &mut settings.noise_type,
                NoiseType::Simplex.into(),
                "Simplex",
            );
            ui.selectable_value(
                &mut settings.noise_type,
                NoiseType::SimplexFractal.into(),
                "Simplex Fractal",
            );
            ui.selectable_value(
                &mut settings.noise_type,
                NoiseType::Cellular.into(),
                "Cellular",
            );
            ui.selectable_value(
                &mut settings.noise_type,
                NoiseType::WhiteNoise.into(),
                "WhiteNoise",
            );
            ui.selectable_value(&mut settings.noise_type, NoiseType::Cubic.into(), "Cubic");
            ui.selectable_value(
                &mut settings.noise_type,
                NoiseType::CubicFractal.into(),
                "Cubic Fractal",
            );
        });
    updated = updated || noise_type != settings.noise_type;

    updated = updated
        || ui
            .add(egui::Slider::new(&mut settings.fractal_octaves, 0..=28).text("Fractal Octaves"))
            .drag_released();
    updated = updated
        || ui
            .add(egui::Slider::new(&mut settings.fractal_gain, 0.0..=2.0).text("Fractal Gain"))
            .changed();
    updated = updated
        || ui
            .add(
                egui::Slider::new(&mut settings.fractal_lacunarity, 0.0..=7.0)
                    .text("Fractal Lacunarity"),
            )
            .drag_released();
    updated = updated
        || ui
            .add(egui::Slider::new(&mut settings.frequency, 0.0..=5.0).text("Frequency"))
            .changed();
    ui.add(egui::Checkbox::new(
        &mut state.parameters.auto_apply,
        "Auto Apply",
    ));

    if ui.button("Reset").clicked() {
        settings.reset();
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
}
pub fn heightmap_generation_settings(
    ui: &mut egui::Ui,
    ui_state: &mut UiState,
    state: &mut AppState,
) {
    egui::CollapsingHeader::new("Heightmap Generation")
        .default_open(true)
        .show(ui, |ui| {
            if state.simulation_state().eroded().is_none()
                && state.simulation_state().id() == state.simulation_base_indices.len() - 1
            {
                let mut heightmap_type = state.parameters.heightmap_type;
                egui::ComboBox::from_label("Heightmap Type")
                    .selected_text(format!("{}", heightmap_type))
                    .show_ui(ui, |ui| {
                        for ref mut t in HeightmapType::iterator() {
                            ui.selectable_value(&mut heightmap_type, *t, format!("{}", t));
                        }
                    });

                let type_changed = heightmap_type != state.parameters.heightmap_type;

                match heightmap_type {
                    HeightmapType::Procedural(ref mut params, ref mut settings) => {
                        heightmap_parameters(params, ui, ui_state, state);
                        procedural_generation_settings(settings, ui, ui_state, state);
                    }
                    _ => (),
                }

                state.parameters.heightmap_type = heightmap_type;
                if type_changed {
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
