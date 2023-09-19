use crate::heightmap;
use macroquad::prelude::*;

pub mod beyer;
pub mod lague;
pub mod ui;

pub async fn run() {
    lague::visualize().await;
}

fn heightmap_to_texture(heightmap: &heightmap::Heightmap) -> Texture2D {
    let buffer = heightmap.to_u8_rgba();

    let image = Image {
        bytes: buffer,
        width: heightmap.width.try_into().unwrap(),
        height: heightmap.height.try_into().unwrap(),
    };

    Texture2D::from_image(&image)
}
