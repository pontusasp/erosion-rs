use bevy::prelude::*;
use bevy_egui::{egui, EguiContextPass, EguiContexts, EguiPlugin};

use crate::erode::Parameters;
use crate::heightmap::HeightmapType;
use crate::visualize::app_state::{AppParameters, AppState, SimulationState};
use crate::visualize::events::UiEvent;
use crate::visualize::ui::{IsolineProperties, UiState};
use serde::{Deserialize, Serialize};

pub mod erode;
pub mod heightmap;
#[cfg(feature = "export")]
mod io;
pub mod math;
pub mod partitioning;
pub mod visualize;

const WIDTH: u32 = 1107;
const HEIGHT: u32 = 800;
const PRESET_GRID_SIZE: usize = 6;
const PRESET_HEIGHTMAP_SIZE: usize = 512;
const GRID_SIZE_RANGE_MIN: usize = 2;
const GRID_SIZE_RANGE_MAX: usize = 32;
const GAUSSIAN_BLUR_SIGMA_RANGE_MIN: f32 = 0.0;
const GAUSSIAN_BLUR_SIGMA_RANGE_MAX: f32 = 20.0;
const GAUSSIAN_BLUR_BOUNDARY_THICKNESS_MIN: u16 = 0;
const GAUSSIAN_BLUR_BOUNDARY_THICKNESS_MAX: u16 = 10;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct State {
    pub state_name: Option<String>,
    pub app_state: AppState,
    pub ui_state: UiState,
}

impl State {
    pub fn default() -> Self {
        Self::new(&HeightmapType::default())
    }

    pub fn new(heightmap_type: &HeightmapType) -> Self {
        Self {
            state_name: None,
            app_state: AppState {
                simulation_states: vec![SimulationState::get_new_base(
                    0,
                    &heightmap_type,
                    &Parameters::default(),
                )],
                simulation_base_indices: vec![0],
                parameters: AppParameters {
                    heightmap_type: *heightmap_type,
                    ..Default::default()
                },
            },
            ui_state: UiState {
                show_ui_all: true,
                show_ui_keybinds: false,
                show_ui_control_panel: true,
                show_ui_metadata: false,
                show_ui_metrics: false,
                show_ui_presentation_mode: true,
                show_grid: false,
                simulation_clear: true,
                simulation_regenerate: false,
                application_quit: false,
                ui_events: Vec::<UiEvent>::new(),
                ui_events_previous: Vec::<UiEvent>::new(),
                frame_slots: None,
                blur_sigma: 5.0,
                canny_edge: (2.5, 50.0),
                isoline: IsolineProperties {
                    height: 0.2,
                    error: 0.01,
                    flood_lower: false,
                    should_flood: true,
                    flooded_areas_lower: None,
                    flooded_areas_higher: None,
                    blur_augmentation: (false, 1.0, 5, 5),
                    advanced_texture: true,
                    flooded_errors: None,
                },
                #[cfg(feature = "export")]
                saves: io::list_state_files()
                    .ok()
                    .or_else(|| Some(Vec::new()))
                    .expect("Failed to access saved states."),
                screenshots: 0,
            },
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
enum Command {
    Engine,
    GenerateExample,
    GenerateScript,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin {
            enable_multipass_for_primary_context: true,
        })
        .add_systems(EguiContextPass, ui_example_system)
        .run();
    visualize::run();
}

fn ui_example_system(mut contexts: EguiContexts) {
    egui::Window::new("Hello").show(contexts.ctx_mut(), |ui| {
        ui.label("world");
    });
}
