use crate::erode::Parameters;
use crate::heightmap::HeightmapType;
use crate::visualize::app_state::{AppParameters, AppState, SimulationState};
use crate::visualize::events::UiEvent;
use crate::visualize::ui::{IsolineProperties, UiState};
use image::io::Reader as ImageReader;
use macroquad::miniquad::conf::Icon;
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use std::{env, fs};
use crate::generate_tests::generate_all_permutations;

pub mod generate_tests;
pub mod engine;
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
const GRID_SIZE_RANGE_MAX: usize = 128;
const GAUSSIAN_BLUR_SIGMA_RANGE_MIN: f32 = 0.0;
const GAUSSIAN_BLUR_SIGMA_RANGE_MAX: f32 = 20.0;
const GAUSSIAN_BLUR_BOUNDARY_THICKNESS_MIN: u16 = 0;
const GAUSSIAN_BLUR_BOUNDARY_THICKNESS_MAX: u16 = 10;

fn window_conf() -> Conf {
    fn icons() -> Option<Icon> {
        let icon_small_img = ImageReader::open("assets/icon16x16.png")
            .and_then(|file| Ok(file.decode()))
            .ok()?.ok()?;
        let icon_medium_img = ImageReader::open("assets/icon32x32.png")
            .and_then(|file| Ok(file.decode()))
            .ok()?.ok()?;
        let icon_large_img = ImageReader::open("assets/icon64x64.png")
            .and_then(|file| Ok(file.decode()))
            .ok()?.ok()?;

        let icon_small_bytes = icon_small_img.as_bytes();
        let icon_medium_bytes = icon_medium_img.as_bytes();
        let icon_large_bytes = icon_large_img.as_bytes();

        let small_len = icon_small_bytes.len();
        let medium_len = icon_small_bytes.len();
        let large_len = icon_small_bytes.len();

        let icon_small: [u8; 16 * 16 * 4] = icon_small_bytes.try_into().expect(
            format!(
                "16x16 icon given incorrect size: {} instead of {}",
                small_len,
                16 * 16 * 4
            )
                .as_str(),
        );
        let icon_medium: [u8; 32 * 32 * 4] = icon_medium_bytes.try_into().expect(
            format!(
                "32x32 icon given incorrect size: {} instead of {}",
                medium_len,
                32 * 32 * 4
            )
                .as_str(),
        );
        let icon_large: [u8; 64 * 64 * 4] = icon_large_bytes.try_into().expect(
            format!(
                "64x64 icon given incorrect size: {} instead of {}",
                large_len,
                64 * 64 * 4
            )
                .as_str(),
        );

        Some(Icon {
            small: icon_small,
            medium: icon_medium,
            big: icon_large,
        })
    }

    Conf {
        window_title: "Erosion RS".to_owned(),
        window_width: WIDTH.try_into().unwrap(),
        window_height: HEIGHT.try_into().unwrap(),
        window_resizable: true,
        icon: icons(),
        ..Default::default()
    }
}

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
                },
                #[cfg(feature = "export")]
                saves: io::list_state_files().ok().or_else(|| Some(Vec::new())).expect("Failed to access saved states."),
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

#[macroquad::main(window_conf)]
async fn main() {
    let args: Vec<String> = env::args().collect();

    let command_bindings: &[(String, Command)] = &[
        ("--engine".to_string(), Command::Engine),
        ("-e".to_string(), Command::Engine),
        ("--generate-example".to_string(), Command::GenerateExample),
        ("--generate-script".to_string(), Command::GenerateScript),
    ];

    let mut commands: Vec<Command> = args
        .iter()
        .filter_map(|str| {
            for (binding, command) in command_bindings {
                if str == binding {
                    return Some(*command);
                }
            }
            None
        })
        .collect();

    commands.sort();
    commands.dedup_by(|a, b| a == b);

    dbg!(&commands);

    for (_i, command) in commands.iter().enumerate() {
        match command {
            Command::Engine => {
                // let script = if let Some(script_raw) = fs::read_to_string("script.erss").ok() {
                //     serde_json::from_str(&script_raw).expect("Failed to parse script.")
                // } else {
                //     engine::scripts::default()
                // };
                let script = generate_all_permutations();

                let engine_result = engine::launch(script).await;
                if let Ok(_state) = engine_result {
                } else if let Err(err) = engine_result {
                    println!("Engine died. Reason: {:?}", err);
                };
            }
            Command::GenerateExample => {
                let result = serde_json::to_string(&engine::scripts::default());
                if let Ok(example) = result {
                    let result = fs::write("script.example.erss", example);
                    if let Ok(()) = result {
                    } else {
                        panic!("Example can't be converted to json!");
                    }
                }
            }
            Command::GenerateScript => {
                let result = serde_json::to_string(&generate_tests::generate_test());
                if let Ok(example) = result {
                    let result = fs::write("script.erss", example);
                    if let Ok(()) = result {
                    } else {
                        panic!("Failed to serialize script!");
                    }
                }
            }
        }
    }

    if commands.is_empty() {
        visualize::run().await;
    }
}
