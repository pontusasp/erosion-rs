//#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

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
pub mod erosion;

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

fn main_old() {
    visualize::run();
}

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([300.0, 220.0])
            .with_icon(
                // NOTE: Adding an icon is optional
                eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon64x64.png")[..])
                    .expect("Failed to load icon"),
            ),
        ..Default::default()
    };
    eframe::run_native(
        "Erosion-RS",
        native_options,
        Box::new(|cc| Ok(Box::new(erosion::ErosionApp::new(cc)))),
    )
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;

    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("No window")
            .document()
            .expect("No document");

        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect("Failed to find the_canvas_id")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("the_canvas_id was not a HtmlCanvasElement");

        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| Ok(Box::new(erosion::ErosionApp::new(cc)))),
            )
            .await;

        // Remove the loading text and spinner:
        if let Some(loading_text) = document.get_element_by_id("loading_text") {
            match start_result {
                Ok(_) => {
                    loading_text.remove();
                }
                Err(e) => {
                    loading_text.set_inner_html(
                        "<p> The app has crashed. See the developer console for details. </p>",
                    );
                    panic!("Failed to start eframe: {e:?}");
                }
            }
        }
    });
}
