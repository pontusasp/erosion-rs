use macroquad::prelude::*;

use crate::heightmap;
use crate::erode;
use crate::visualize::heightmap_to_texture;

#[derive(Debug, Clone)]
struct State {
    heightmap: heightmap::Heightmap,
    drop: erode::beyer::Drop,
    iteration: usize,
    drop_count: usize,
}

pub async fn debug() {
    prevent_quit();

    let mut rng = ::rand::thread_rng();

    while !is_quit_requested() {
        let mut heightmap_ = erode::initialize_heightmap();
        heightmap_.normalize(); // Normalize to get the most accuracy out of the png later since heightmap might not utilize full range of 0.0 to 1.0

        let drop = erode::beyer::create_drop(
            erode::beyer::random_position(&heightmap_, &mut rng),
            erode::beyer::get_random_angle(&mut rng),
            &mut 0.0,
        )
        .unwrap();

        let state_ = State {
            heightmap: heightmap_.clone(),
            drop,
            iteration: 0,
            drop_count: 0,
        };

        let mut states = vec![state_.clone()];
        let mut state_index = 0;

        let mut last_state = state_index + 1;
        let mut last_heightmap_texture = heightmap_to_texture(&heightmap_);

        next_frame().await;

        while !is_quit_requested() && !is_key_pressed(KeyCode::R) {
            clear_background(BLACK);

            let mut steps = 1;
            if is_key_down(KeyCode::Key2) {
                steps *= 2;
            }
            if is_key_down(KeyCode::Key3) {
                steps *= 3;
            }
            if is_key_down(KeyCode::Key4) {
                steps *= 4;
            }
            if is_key_down(KeyCode::Key5) {
                steps *= 5;
            }
            if is_key_down(KeyCode::Key6) {
                steps *= 6;
            }
            if is_key_down(KeyCode::Key7) {
                steps *= 7;
            }
            if is_key_down(KeyCode::Key8) {
                steps *= 8;
            }
            if is_key_down(KeyCode::Key9) {
                steps *= 9;
            }
            if is_key_down(KeyCode::Key0) {
                steps *= 1000;
            }

            if is_key_pressed(KeyCode::P) {
                states.truncate(state_index + 1);
            }

            if is_key_down(KeyCode::J) || is_key_pressed(KeyCode::Right) {
                state_index += 1;
                if state_index >= states.len() {
                    let State {
                        mut drop,
                        mut heightmap,
                        mut iteration,
                        mut drop_count,
                    } = states.last().unwrap().clone();
                    for _ in 0..steps {
                        if drop != erode::beyer::Drop::Dead {
                            erode::beyer::tick(&mut heightmap, &mut drop, 2.0).unwrap();
                        } else {
                            drop = erode::beyer::create_drop(
                                erode::beyer::random_position(&heightmap_, &mut rng),
                                erode::beyer::get_random_angle(&mut rng),
                                &mut 0.0,
                            )
                            .unwrap();
                            drop_count += 1;
                        }
                        iteration += 1;
                    }
                    states.push(State {
                        drop,
                        heightmap,
                        iteration,
                        drop_count,
                    });
                }
            } else if is_key_down(KeyCode::K) || is_key_pressed(KeyCode::Left) {
                if state_index > 0 {
                    state_index -= 1;
                }
            };

            let State {
                drop,
                heightmap,
                iteration,
                drop_count,
            } = states.get(state_index).unwrap();

            if !is_key_down(KeyCode::Space) {
                if last_state != state_index {
                    last_heightmap_texture = heightmap_to_texture(&heightmap);
                    last_heightmap_texture.set_filter(FilterMode::Nearest);
                }
                // Draw heightmap
                draw_texture_ex(
                    last_heightmap_texture,
                    0.0,
                    0.0,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(screen_width(), screen_height())),
                        ..Default::default()
                    },
                );
            } else {
                let mut diff = heightmap.subtract(&heightmap_).unwrap();
                diff.normalize();

                let diff_texture = heightmap_to_texture(&diff);
                diff_texture.set_filter(FilterMode::Nearest);
                // Draw heightmap
                draw_texture_ex(
                    diff_texture,
                    0.0,
                    0.0,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(screen_width(), screen_height())),
                        ..Default::default()
                    },
                );
            }

            match drop {
                erode::beyer::Drop::Alive { position, .. } => {
                    let theta = drop.get_angle().unwrap();
                    let x = position.x / heightmap_.width as f32 * screen_width();
                    let y = position.y / heightmap_.height as f32 * screen_height();
                    let r =
                        erode::beyer::P_RADIUS as f32 * screen_width() / heightmap_.width as f32;
                    draw_circle_lines(x, y, r, 1.5, RED);
                    draw_line(
                        x + r * theta.cos(),
                        y + r * theta.sin(),
                        x + r * theta.cos() * 3.0,
                        y + r * theta.sin() * 3.0,
                        1.5,
                        RED,
                    );
                }
                erode::beyer::Drop::Dead => {}
            }

            if screen_width() != screen_height() {
                request_new_screen_size(
                    screen_width().min(screen_height()),
                    screen_width().min(screen_height()),
                );
            }

            egui_macroquad::ui(|egui_ctx| {
                egui::Window::new("Erosion")
                    // .default_size(egui::vec2(200.0, 100.0))
                    .show(egui_ctx, |ui| {
                        ui.label(&format!("Iteration: {}", iteration));
                        ui.label(&format!("Drop count: {}", drop_count));
                        ui.label(&format!("State: {}", state_index));
                        ui.label(&format!("Speed: {}", steps));
                        ui.label("");
                        ui.label("Controls:");
                        ui.label("Space: Show difference");
                        ui.label("J: Next state");
                        ui.label("K: Previous state");
                        ui.label("P: Reset");
                        ui.label("R: Restart");
                        ui.label("1-9: Speed");
                    });
            });

            egui_macroquad::draw();

            last_state = state_index;
            next_frame().await
        }
        if is_key_pressed(KeyCode::R) {
            println!("Restarting...");
        } else {
            println!("Bye!");
        }
    }
}
