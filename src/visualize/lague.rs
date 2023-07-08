use macroquad::prelude::*;
use std::thread;
use std::sync::{Arc, Mutex};

use crate::{erode, partitioning};
use crate::erode::lague;
use crate::heightmap;
use crate::math::UVector2;
use crate::visualize::heightmap_to_texture;

const EROSION_METHOD: partitioning::Method = partitioning::Method::Subdivision;

pub async fn visualize() {
    prevent_quit();
    let mut restart = true;

    while restart {
        restart = false;
        let mut eroded = false;

        let mut heightmap = erode::initialize_heightmap();
        heightmap.normalize();
        let heightmap_original = heightmap.clone();
        let heightmap_texture = heightmap_to_texture(&heightmap_original);
        let mut params = lague::DEFAULT_PARAMS;
        params.num_iterations = 1000000;
        let mut heightmap_eroded_texture = None;
        let mut heightmap_diff = heightmap.subtract(&heightmap_original).unwrap();
        let mut heightmap_diff_texture = None;
        heightmap_diff.normalize();
        let mut heightmap_diff_normalized = None;

        while !is_quit_requested() && !restart {
            draw_texture_ex(
                if is_key_down(KeyCode::Space) {
                    heightmap_texture
                } else if let Some(texture) = if is_key_down(KeyCode::D) {
                    if is_key_down(KeyCode::LeftShift) {
                        heightmap_diff_normalized
                    } else {
                        heightmap_diff_texture
                    }
                } else {
                    heightmap_eroded_texture
                } { texture } else { heightmap_texture },
                // heightmap_texture,
                0.0,
                0.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(crate::WIDTH as f32, crate::HEIGHT as f32)),
                    ..Default::default()
                },
            );

            if is_key_pressed(KeyCode::E) {
                if !eroded {
                    print!("Eroding using ");
                    match EROSION_METHOD {
                        partitioning::Method::Subdivision => {
                            println!("subdivision method");
                            partitioning::subdivision_erode(&mut heightmap, &params);
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

            if is_key_pressed(KeyCode::R) {
                restart = true;
            }

            if is_key_pressed(KeyCode::S) {
                heightmap::export_heightmaps(
                    vec![&heightmap_original, &heightmap, &heightmap_diff],
                    vec![
                        "output/heightmap",
                        "output/heightmap_eroded",
                        "output/heightmap_diff",
                    ],
                );
            }

            next_frame().await;
        }
    }
}