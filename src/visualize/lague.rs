use macroquad::prelude::*;
use std::thread;
use std::sync::{Arc, Mutex};

use crate::erode;
use crate::erode::lague;
use crate::heightmap;
use crate::math::UVector2;
use crate::visualize::heightmap_to_texture;

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
                    println!("Eroding...");
                    let subdivisions = 2;
                    let slice_amount = 2_usize.pow(subdivisions);
                    let slices = UVector2 { x: slice_amount, y: slice_amount };
                    let size = UVector2 { x: heightmap.width / slices.x, y: heightmap.height / slices.y };
                    let mut partitions = Vec::new();
                    for x in 0..slices.x {
                        for y in 0..slices.y {
                            let anchor = UVector2 { x: x * size.x, y: y * size.y };
                            let partition = Arc::new(Mutex::new(heightmap::PartialHeightmap::from(&heightmap, &anchor, &size)));
                            partitions.push(partition);
                        }
                    }
                    let mut handles = Vec::new();

                    let mut params = params.clone();
                    params.num_iterations /= partitions.len();
                    for i in 0..partitions.len() {
                        let partition = Arc::clone(&partitions[i]);
                        let handle = thread::spawn(move || {
                            lague::erode(&mut partition.lock().unwrap().heightmap, &params);
                        });
                        handles.push(handle);
                    }
                    for handle in handles {
                        handle.join().unwrap();
                    }
                    for partition in partitions {
                        partition.lock().unwrap().apply_to(&mut heightmap);
                    }

                    // heightmap = heightmap::PartialHeightmap::combine(&tl, &bl, &tr, &br);

                    println!("Done!");
                    heightmap_eroded_texture = Some(heightmap_to_texture(&heightmap));
                    heightmap_diff = heightmap.subtract(&heightmap_original).unwrap();
                    heightmap_diff_texture = Some(heightmap_to_texture(&heightmap_diff));
                    heightmap_diff.normalize();
                    heightmap_diff_normalized = Some(heightmap_to_texture(&heightmap_diff));
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