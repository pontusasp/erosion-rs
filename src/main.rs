use ds_heightmap::Runner;
use macroquad::prelude::*;
use std::env;
use macroquad::ui::root_ui;

pub mod erode;
pub mod heightmap;
pub mod math;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 800;

fn window_conf() -> Conf {
    Conf {
        window_title: "Erosion RS".to_owned(),
        window_width: WIDTH.try_into().unwrap(),
        window_height: HEIGHT.try_into().unwrap(),
        window_resizable: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        debug().await;
    } else {
        run_simulation();
    }
}

#[derive(Debug, Clone)]
struct State {
    heightmap: heightmap::Heightmap,
    drop: erode::beyer::Drop,
    iteration: usize,
    drop_count: usize,
}

async fn debug() {
    env::set_var("RUST_BACKTRACE", "1");
    prevent_quit();

    let mut rng = ::rand::thread_rng();

    while !is_quit_requested() {
        let mut heightmap_ = initialize_heightmap();
        heightmap_.normalize(); // Normalize to get the most accuracy out of the png later since heightmap might not utilize full range of 0.0 to 1.0

        let mut drop = erode::beyer::create_drop(
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

            // draw_text(
            //     &format!("Iteration: {}", iteration),
            //     10.0,
            //     20.0,
            //     20.0,
            //     WHITE,
            // );
            // draw_text(
            //     &format!("Drop count: {}", drop_count),
            //     10.0,
            //     40.0,
            //     20.0,
            //     WHITE,
            // );

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

fn run_simulation() {
    env::set_var("RUST_BACKTRACE", "1");

    let mut heightmap = initialize_heightmap();
    heightmap.normalize(); // Normalize to get the most accuracy out of the png later since heightmap might not utilize full range of 0.0 to 1.0

    let heightmap_eroded = erode::erode(&heightmap);
    let heightmap_diff = heightmap.subtract(&heightmap_eroded).unwrap();

    export_heightmaps(
        vec![&heightmap, &heightmap_eroded, &heightmap_diff],
        vec![
            "output/heightmap",
            "output/heightmap_eroded",
            "output/heightmap_diff",
        ],
    );

    println!("Done!");
}

fn export_heightmaps(heightmaps: Vec<&heightmap::Heightmap>, filenames: Vec<&str>) {
    println!("Exporting heightmaps...");
    for (heightmap, filename) in heightmaps.iter().zip(filenames.iter()) {
        heightmap_to_image(heightmap, filename).unwrap();
        heightmap::io::export(heightmap, filename).unwrap();
    }
}

fn create_heightmap(size: usize, original_depth: f32, roughness: f32) -> heightmap::Heightmap {
    let mut runner = Runner::new();
    runner.set_height(size);
    runner.set_width(size);

    runner.set_depth(original_depth);
    runner.set_rough(roughness);

    let depth = 1.0;

    let output = runner.ds();
    heightmap::Heightmap {
        data: output
            .data
            .into_iter()
            .map(|row| {
                row.into_iter()
                    .map(|value| value as heightmap::HeightmapPrecision / original_depth)
                    .collect()
            })
            .collect(),
        width: size,
        height: size,
        depth,
        original_depth,
        metadata: None,
    }
}

fn create_heightmap_from_closure(
    size: usize,
    original_depth: f32,
    closure: &dyn Fn(usize, usize) -> heightmap::HeightmapPrecision,
) -> heightmap::Heightmap {
    let mut data: Vec<Vec<heightmap::HeightmapPrecision>> = Vec::new();
    for i in 0..size {
        let mut row = Vec::new();
        for j in 0..size {
            row.push(closure(i, j));
        }
        data.push(row);
    }

    heightmap::Heightmap {
        data,
        width: size,
        height: size,
        depth: 1.0,
        original_depth,
        metadata: None,
    }
}

fn initialize_heightmap() -> heightmap::Heightmap {
    let size: usize = 256;
    let depth: f32 = 2000.0;
    let roughness: f32 = 1.0;

    let debug = false;

    // Y gradient
    // let debug_heightmap = create_heightmap_from_closure(size, depth, &|_: usize, y: usize| y as heightmap::HeightmapPrecision / size as heightmap::HeightmapPrecision);

    // Inverted Y gradient
    // let debug_heightmap = create_heightmap_from_closure(size, depth, &|_: usize, y: usize| 1.0 - y as heightmap::HeightmapPrecision / size as heightmap::HeightmapPrecision);

    // Y hyperbola gradient
    // let debug_heightmap = create_heightmap_from_closure(size, depth, &|_: usize, y: usize| {
    // let gradient = y as heightmap::HeightmapPrecision / size as heightmap::HeightmapPrecision;
    // gradient.powi(2)
    // });

    // Centered hill gradient
    // let debug_heightmap = create_heightmap_from_closure(size, depth, &|x: usize, y: usize| {
    //     let gradient = (x as heightmap::HeightmapPrecision
    //         - size as heightmap::HeightmapPrecision / 2.0)
    //         .powi(2)
    //         + (y as heightmap::HeightmapPrecision - size as heightmap::HeightmapPrecision / 2.0)
    //         .powi(2);
    //     1.0 - gradient / (size as heightmap::HeightmapPrecision / 2.0).powi(2)
    // });

    // Centered small hill gradient
    let debug_heightmap = create_heightmap_from_closure(size, depth, &|x: usize, y: usize| {
        let radius = size as heightmap::HeightmapPrecision / 2.0;
        let x = x as heightmap::HeightmapPrecision;
        let y = y as heightmap::HeightmapPrecision;
        let distance = ((x - radius).powf(2.0) + (y - radius).powf(2.0)).sqrt();

        let hill_radius = 0.75;

        if distance < radius * hill_radius {
            let to = radius * hill_radius;
            let from = 0.0;
            let gradient = (distance - from) / (to - from);
            ((std::f32::consts::PI * gradient).cos() + 1.0) / 2.0
        } else {
            0.0
        }
    });

    if debug {
        debug_heightmap
    } else {
        create_heightmap(size, depth, roughness)
    }
}

fn heightmap_to_image(heightmap: &heightmap::Heightmap, filename: &str) -> image::ImageResult<()> {
    let buffer = heightmap.to_u8();

    // Save the buffer as filename on disk
    image::save_buffer(
        format!("{}.png", filename),
        &buffer as &[u8],
        heightmap.width.try_into().unwrap(),
        heightmap.height.try_into().unwrap(),
        image::ColorType::L8,
    )
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
