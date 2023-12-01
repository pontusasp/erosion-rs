use crate::{heightmap, State};

use egui::{Pos2, Rect};
use macroquad::prelude::*;

pub mod app_state;
pub mod canvas;
pub mod events;
pub mod keybinds;
pub mod panels;
pub mod ui;
pub mod widgets;
pub mod wrappers;

use crate::heightmap::Heightmap;
use crate::visualize::app_state::{AppState, SimulationState};
use crate::visualize::events::poll_ui_events;
use crate::visualize::keybinds::poll_ui_keybinds;
use crate::visualize::ui::*;

pub fn generate_default_state() -> State {
    State::default()
}

pub async fn run() {
    prevent_quit();

    let mut state = {
        let state = generate_default_state();
        let autoload_default: Option<State> = {
            #[cfg(feature = "export")]
            {
                let default = state
                    .ui_state
                    .saves
                    .iter()
                    .find(|&save| save.0 == "default");
                if let Some(state_file) = default {
                    crate::io::import(&state_file.0).ok()
                } else {
                    None
                }
            }
            #[cfg(not(feature = "export"))]
            {
                None
            }
        };

        if let Some(default) = autoload_default {
            default
        } else {
            state
        }
    };

    let mut launching = true;

    let mut corrected_size = false;

    // Update heightmap data
    while launching || state.ui_state.simulation_clear && !state.ui_state.application_quit {
        launching = false;
        if state.ui_state.simulation_clear {
            state = generate_default_state();
        }
        state.ui_state.simulation_clear = false;

        if state.ui_state.simulation_regenerate {
            state
                .app_state
                .simulation_states
                .push(SimulationState::get_new_base(
                    state.app_state.simulation_states.len(),
                    &state.app_state.parameters.heightmap_type,
                    &state.app_state.parameters.erosion_params,
                ));
            state
                .app_state
                .simulation_base_indices
                .push(state.app_state.simulation_states.len() - 1);
            state.ui_state.simulation_regenerate = false;
        }

        // Update UI
        while !is_quit_requested()
            && !state.ui_state.simulation_clear
            && !state.ui_state.application_quit
        {
            clear_background(BLACK);

            let canvas_rect = state
                .ui_state
                .frame_slots
                .as_ref()
                .and_then(|slots| slots.canvas)
                .unwrap_or(Rect {
                    min: Pos2 { x: 0.0, y: 0.0 },
                    max: Pos2 {
                        x: screen_width(),
                        y: screen_height(),
                    },
                });

            if !corrected_size {
                let fit = canvas_rect.width().min(canvas_rect.height());
                request_new_screen_size(
                    crate::WIDTH as f32 + canvas_rect.height() - fit,
                    crate::HEIGHT as f32 + canvas_rect.width() - fit,
                );
                corrected_size = true;
            }
            draw_frame(
                &canvas_rect,
                &state.app_state.simulation_state().get_active_texture(),
            );
            if state.ui_state.show_grid {
                draw_frame(
                    &canvas_rect,
                    &state
                        .app_state
                        .simulation_state()
                        .get_active_grid_texture(&state.app_state.parameters),
                );
            }

            state.ui_state.frame_slots = ui_draw(&mut state);

            #[cfg(feature = "export")]
            let state_name = &mut state.state_name;
            let app_state = &mut state.app_state;
            let ui_state = &mut state.ui_state;
            poll_ui_events(
                #[cfg(feature = "export")]
                state_name,
                ui_state,
                app_state,
            );
            poll_ui_keybinds(&mut state.ui_state);
            next_frame().await;
        }
    }
}

pub fn draw_frame(rect: &Rect, texture: &Texture2D) {
    let side = rect.width().min(rect.height());
    let margin_left = (rect.width() - side) / 2.0;
    let margin_top = (rect.height() - side) / 2.0;
    texture.set_filter(FilterMode::Nearest);
    draw_texture_ex(
        *texture,
        rect.min.x + margin_left,
        rect.min.y + margin_top,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(side, side)),
            ..Default::default()
        },
    );
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

fn mix_heightmap_to_texture(
    heightmap: &Heightmap,
    overlay: &Heightmap,
    channel: u8,
    invert: bool,
    round: bool,
) -> Texture2D {
    let overlay = overlay.to_u8();
    let mut buffer = heightmap.to_u8_rgba();

    for i in (0..buffer.len()).step_by(4) {
        let mut overlay = overlay[i / 4] as f32 / 255.0;
        if invert {
            overlay = 1.0 - overlay;
        }
        let k = i + channel as usize;
        let keep = buffer[k];
        let r = i;
        let g = i + 1;
        let b = i + 2;
        buffer[r] = (buffer[r] as f32 * overlay) as u8;
        buffer[g] = (buffer[g] as f32 * overlay) as u8;
        buffer[b] = (buffer[b] as f32 * overlay) as u8;
        if round && overlay < 0.5 {
            buffer[k] = ((1.0 - overlay.round()) * 255.0) as u8;
        } else {
            buffer[k] = keep;
        }
    }

    let image = Image {
        bytes: buffer,
        width: heightmap.width.try_into().unwrap(),
        height: heightmap.height.try_into().unwrap(),
    };

    Texture2D::from_image(&image)
}

pub enum LayerMixMethod {
    Additive,
    AdditiveClamp,
    Multiply,
    Difference,
}

pub mod rgba_color_channel {
    pub type Channel = u8;
    pub const R: Channel = 0b0001;
    pub const G: Channel = 0b0010;
    pub const B: Channel = 0b0100;
    pub const A: Channel = 0b1000;
    pub const RGBA: Channel = R | G | B | A;
    pub const RGA: Channel = R | G | A;
    pub const RBA: Channel = R | B | A;
    pub const GBA: Channel = G | B | A;
    pub const RA: Channel = R | A;
    pub const GA: Channel = B | A;
    pub const BA: Channel = G | A;
    pub const RGB: Channel = R | G | B;
    pub const RG: Channel = R | G;
    pub const RB: Channel = R | B;
    pub const GB: Channel = G | B;
}
pub struct HeightmapLayer<'a> {
    pub heightmap: &'a Heightmap,
    pub channel: rgba_color_channel::Channel,
    pub strength: f32,
    pub layer_mix_method: LayerMixMethod,
    pub inverted: bool,
    pub modifies_alpha: bool,
}

pub fn layered_heightmaps_to_texture(
    size: usize,
    layers: &Vec<&HeightmapLayer>,
    normalize_on_overflow: bool,
    max_height: f32,
) -> Texture2D {
    let mut buffer: Vec<f32> = vec![0.0; 4 * size * size];
    let mut highest = 0f32;

    // Set alpha to full by default
    for i in (3..buffer.len()).step_by(4) {
        buffer[i] = max_height;
    }

    for &layer in layers.iter() {
        highest = 0f32;
        for i in 0..(size * size) {
            let x = i % size;
            let y = i / size;
            let height = if layer.inverted {
                max_height - layer.heightmap.data[x][y]
            } else {
                layer.heightmap.data[x][y]
            };
            let channels = [
                (
                    layer.channel & rgba_color_channel::R == rgba_color_channel::R,
                    i * 4 + 0,
                    false,
                ),
                (
                    layer.channel & rgba_color_channel::G == rgba_color_channel::G,
                    i * 4 + 1,
                    false,
                ),
                (
                    layer.channel & rgba_color_channel::B == rgba_color_channel::B,
                    i * 4 + 2,
                    false,
                ),
                (
                    layer.channel & rgba_color_channel::A == rgba_color_channel::A,
                    i * 4 + 3,
                    !layer.modifies_alpha,
                ),
            ];
            for channel in channels {
                let c = &mut buffer[channel.1];
                let c_copy = *c;
                if channel.0 {
                    match layer.layer_mix_method {
                        LayerMixMethod::Additive => {
                            *c += height;
                        }
                        LayerMixMethod::AdditiveClamp => {
                            *c = max_height.min(*c + height);
                        }
                        LayerMixMethod::Multiply => {
                            *c *= height / max_height;
                        }
                        LayerMixMethod::Difference => {
                            *c = (*c - height).abs();
                        }
                    }
                } else {
                    match layer.layer_mix_method {
                        LayerMixMethod::Multiply => {
                            if !channel.2 {
                                *c = 0.0;
                            }
                        }
                        _ => (),
                    }
                }
                *c = c_copy * (1f32 - layer.strength) + *c * layer.strength;
                if normalize_on_overflow {
                    highest = highest.max(*c);
                } else {
                    *c = max_height.min(*c);
                }
            }
        }
    }

    let image = Image {
        bytes: buffer
            .iter()
            .map(|&float| {
                let value = if normalize_on_overflow && highest > max_height {
                    float / (highest / max_height)
                } else {
                    float
                };
                (value / max_height * 255.0).trunc() as u8
            })
            .collect(),
        width: size as u16,
        height: size as u16,
    };

    Texture2D::from_image(&image)
}
