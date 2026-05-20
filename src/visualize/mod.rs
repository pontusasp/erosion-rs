use crate::heightmap;

use egui::{Color32, ColorImage};

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

fn heightmap_to_image(heightmap: &heightmap::Heightmap) -> ColorImage {
    let buffer = heightmap.to_u8_rgba();
    
    let [width, height] = [heightmap.width.try_into().unwrap(), heightmap.height.try_into().unwrap()];

    heightmap_from_buffer(width, height, buffer)
}

fn heightmap_from_buffer(width: usize, height: usize, buffer: Vec<u8>) -> ColorImage {
    let pixels: Vec<Color32> = buffer
        .chunks_exact(4)
        .map(|c| Color32::from_rgba_premultiplied(c[0], c[1], c[2], c[3]))
        .collect();
    ColorImage::new([width, height], pixels)
}

fn mix_heightmap_to_image(
    heightmap: &Heightmap,
    overlay: &Heightmap,
    channel: u8,
    invert: bool,
    round: bool,
) -> ColorImage {
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

    heightmap_from_buffer(heightmap.width.try_into().unwrap(), heightmap.height.try_into().unwrap(), buffer)
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
) -> ColorImage {
    let image = layered_heightmaps_to_image(size, layers, normalize_on_overflow, max_height);
    image
}

pub fn layered_heightmaps_to_image(
    size: usize,
    layers: &Vec<&HeightmapLayer>,
    normalize_on_overflow: bool,
    max_height: f32,
) -> ColorImage {
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

    let image = heightmap_from_buffer(
        size as usize,
        size as usize,
        buffer
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
    );

    image
}
