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

use crate::erode::Parameters;
use crate::heightmap::{Heightmap, HeightmapType};
use crate::visualize::app_state::{AppParameters, AppState, SimulationState};
use crate::visualize::events::{poll_ui_events, UiEvent};
use crate::visualize::keybinds::poll_ui_keybinds;
use crate::visualize::ui::*;
const SUBDIVISIONS: u32 = 3;
const GRID_SIZE: usize = 6;
const PRESET_HEIGHTMAP_SIZE: usize = 512;

pub async fn run() {
    prevent_quit();

    let mut state = State {
        app_state: AppState {
            simulation_states: vec![SimulationState::get_new_base(
                0,
                &HeightmapType::default(),
                &Parameters::default(),
            )],
            simulation_base_indices: vec![0],
            parameters: AppParameters::default(),
        },
        ui_state: UiState {
            show_ui_all: true,
            show_ui_keybinds: false,
            show_ui_control_panel: true,
            show_ui_metadata: false,
            show_ui_metrics: false,
            simulation_clear: true,
            simulation_regenerate: false,
            application_quit: false,
            ui_events: Vec::<UiEvent>::new(),
            ui_events_previous: Vec::<UiEvent>::new(),
            frame_slots: None,
            blur_sigma: 5.0,
            canny_edge: (2.5, 50.0),
            isoline: IsolineProperties {
                height: 0.2,
                error: 0.01,
                flood_lower: false,
                should_flood: false,
                flooded_areas_lower: None,
                flooded_areas_higher: None,
                blur_augmentation: (false, 1.0),
            },
        },
    };
    let mut corrected_size = false;

    // Update heightmap data
    while state.ui_state.simulation_clear && !state.ui_state.application_quit {
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

            state.ui_state.frame_slots = ui_draw(&mut state);

            let app_state = &mut state.app_state;
            let ui_state = &mut state.ui_state;
            poll_ui_events(ui_state, app_state);
            poll_ui_keybinds(&mut state.ui_state);
            next_frame().await;
        }
    }
}

fn draw_frame(rect: &Rect, texture: &Texture2D) {
    let side = rect.width().min(rect.height());
    let margin_left = (rect.width() - side) / 2.0;
    let margin_top = (rect.height() - side) / 2.0;
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
