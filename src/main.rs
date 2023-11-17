use crate::visualize::app_state::AppState;
use crate::visualize::ui::UiState;
use image::io::Reader as ImageReader;
use macroquad::miniquad::conf::Icon;
use macroquad::prelude::*;
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
const PRESET_SUBDIVISIONS: u32 = 3;
const PRESET_GRID_SIZE: usize = 6;
const PRESET_HEIGHTMAP_SIZE: usize = 512;
const SUBDIVISIONS_RANGE_MIN: u32 = 0;
const SUBDIVISIONS_RANGE_MAX: u32 = 7;
const GRID_SIZE_RANGE_MIN: usize = 1;
const GRID_SIZE_RANGE_MAX: usize = 128;
const GAUSSIAN_BLUR_SIGMA_RANGE_MIN: f32 = 0.0;
const GAUSSIAN_BLUR_SIGMA_RANGE_MAX: f32 = 20.0;
const GAUSSIAN_BLUR_BOUNDARY_THICKNESS_MIN: u16 = 0;
const GAUSSIAN_BLUR_BOUNDARY_THICKNESS_MAX: u16 = 10;


fn window_conf() -> Conf {
    let icon_small_img = ImageReader::open("assets/icon16x16.png")
        .and_then(|file| Ok(file.decode()))
        .ok()
        .unwrap()
        .unwrap();
    let icon_medium_img = ImageReader::open("assets/icon32x32.png")
        .and_then(|file| Ok(file.decode()))
        .ok()
        .unwrap()
        .unwrap();
    let icon_large_img = ImageReader::open("assets/icon64x64.png")
        .and_then(|file| Ok(file.decode()))
        .ok()
        .unwrap()
        .unwrap();

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

    Conf {
        window_title: "Erosion RS".to_owned(),
        window_width: WIDTH.try_into().unwrap(),
        window_height: HEIGHT.try_into().unwrap(),
        window_resizable: true,
        icon: Some(Icon {
            small: icon_small,
            medium: icon_medium,
            big: icon_large,
        }),
        ..Default::default()
    }
}

#[derive(Serialize, Deserialize)]
pub struct State {
    pub state_name: Option<String>,
    pub app_state: AppState,
    pub ui_state: UiState,
}

#[macroquad::main(window_conf)]
async fn main() {
    visualize::run().await;
}
