use macroquad::prelude::*;

use crate::erode::lague;
use crate::heightmap;
use crate::visualize::heightmap_to_texture;
use crate::{erode, partitioning};

const SUBDIVISIONS: u32 = 3;
const ITERATIONS: usize = 1000000;
const EROSION_METHODS: [partitioning::Method; 2] = [
    partitioning::Method::Subdivision,
    partitioning::Method::SubdivisionOverlap,
];

/*
Keybinds:
- [G] generate new heightmap
- [R] restart
- [S] export
- [E] erode
- [Space] show heightmap texture
- [D] show diff
- [Shift-D] show diff normalized
- [J] select next partitioning method
- [K] select previous partitioning method
*/

pub async fn visualize() {
    prevent_quit();
    let mut restart = true;
    let mut regenerate = false;
    let mut erosion_method_index: usize = 0;

    let mut heightmap = erode::initialize_heightmap();
    heightmap.normalize();
    let mut heightmap_original = heightmap.clone();

    while restart {
        restart = false;
        let mut eroded = false;

        if regenerate {
            heightmap = erode::initialize_heightmap();
            heightmap.normalize();
            heightmap_original = heightmap.clone();
            regenerate = false;
        }

        let heightmap_texture = heightmap_to_texture(&heightmap_original);
        let params = lague::Parameters {
            num_iterations: ITERATIONS,
            ..lague::Parameters::default()
        };
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
                } {
                    texture
                } else {
                    heightmap_texture
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

            if is_key_pressed(KeyCode::J) || is_key_pressed(KeyCode::K) {
                erosion_method_index = (erosion_method_index
                    + if is_key_pressed(KeyCode::J) { 1 } else { EROSION_METHODS.len()-1 })
                    % EROSION_METHODS.len();
                print!("Select method: ");
                match EROSION_METHODS[erosion_method_index] {
                    partitioning::Method::Subdivision => println!("Subdivision"),
                    partitioning::Method::SubdivisionOverlap => println!("SubdivisionOverlap"),
                };
            }

            if is_key_pressed(KeyCode::E) {
                if !eroded {
                    print!("Eroding using ");
                    match EROSION_METHODS[erosion_method_index] {
                        partitioning::Method::Subdivision => {
                            println!("subdivision method");
                            partitioning::subdivision_erode(&mut heightmap, &params, SUBDIVISIONS);
                        }
                        partitioning::Method::SubdivisionOverlap => {
                            println!("SubdivisionOverlap method");
                            partitioning::subdivision_overlap_erode(
                                &mut heightmap,
                                &params,
                                SUBDIVISIONS,
                            );
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
                println!("Restarting");
                restart = true;
            }

            if is_key_pressed(KeyCode::G) {
                println!("Regenerating heightmap");
                restart = true;
                regenerate = true;
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
