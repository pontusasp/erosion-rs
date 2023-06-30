use macroquad::prelude::*;

use crate::erode;
use crate::erode::lague;
use crate::visualize::heightmap_to_texture;

pub async fn visualize() {
    prevent_quit();
    let mut restart = true;

    while restart {
        restart = false;

        let mut heightmap = erode::initialize_heightmap();
        heightmap.normalize();
        let heightmap_original = heightmap.clone();
        let mut params = lague::DEFAULT_PARAMS;
        params.num_iterations = 1000;
        lague::erode(&mut heightmap, &params);
        let heightmap_eroded_texture = heightmap_to_texture(&heightmap);
        let heightmap_texture = heightmap_to_texture(&heightmap_original);
        let mut heightmap_diff = heightmap.subtract(&heightmap_original).unwrap();
        let heightmap_diff_texture = heightmap_to_texture(&heightmap_diff);
        heightmap_diff.normalize();
        let heightmap_diff_normalized = heightmap_to_texture(&heightmap_diff);

        while !is_quit_requested() && !restart {

            draw_texture_ex(
                if is_key_down(KeyCode::Space) {
                    heightmap_texture
                } else if is_key_down(KeyCode::D) {
                    if is_key_down(KeyCode::LeftShift) {
                        heightmap_diff_normalized
                    } else {
                        heightmap_diff_texture
                    }
                } else {
                    heightmap_eroded_texture
                },
                // heightmap_texture,
                0.0,
                0.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(crate::WIDTH as f32, crate::HEIGHT as f32)),
                    ..Default::default()
                },
            );

            if is_key_pressed(KeyCode::R) {
                restart = true;
            }

            next_frame().await;
        }
    }
}