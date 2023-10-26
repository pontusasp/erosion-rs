use crate::heightmap;

use std::cell::RefCell;
use std::rc::Rc;

use egui::{Pos2, Rect};
use macroquad::prelude::*;

pub mod canvas;
pub mod ui;
pub mod widgets;

use crate::erode::DropZone;
use crate::erode::Parameters;
use crate::heightmap::{Heightmap, HeightmapSettings};
use crate::partitioning::Method;
use crate::visualize::ui::*;
use crate::{erode, partitioning};

const SUBDIVISIONS: u32 = 3;
const GRID_SIZE: usize = 6;

pub enum SimulationState {
    Base(BaseState),
    Eroded((BaseState, ErodedState)),
}

impl SimulationState {
    pub fn get_new_base(
        new_id: usize,
        settings: Option<&HeightmapSettings>,
        parameters: &Parameters,
    ) -> Self {
        let mut heightmap = erode::initialize_heightmap(settings).normalize();
        heightmap.calculate_total_height();
        let texture = Rc::new(heightmap_to_texture(&heightmap));
        SimulationState::Base(BaseState {
            id: new_id,
            erosion_method: Method::Default,
            params: parameters.clone(),
            drop_zone: DropZone::default(&heightmap),
            heightmap_base: Rc::new(heightmap),
            texture_heightmap_base: Rc::clone(&texture),
            texture_active: Rc::clone(&texture),
        })
    }

    pub fn get_new_eroded(&self, new_id: usize, parameters: &Parameters) -> Self {
        let (mut base, eroded) = match self {
            SimulationState::Base(base) => (base.clone(), None),
            SimulationState::Eroded((base, eroded)) => (base.clone(), Some(eroded)),
        };

        if let Some(eroded) = eroded {
            base = BaseState {
                id: eroded.id,
                erosion_method: base.erosion_method,
                params: parameters.clone(),
                drop_zone: base.drop_zone,
                heightmap_base: Rc::clone(&eroded.heightmap_eroded),
                texture_heightmap_base: Rc::clone(&eroded.texture_eroded),
                texture_active: Rc::clone(&eroded.texture_eroded),
            };
        }

        let eroded = base.run_simulation(new_id, parameters);
        SimulationState::Eroded((base, eroded))
    }

    pub fn base(&self) -> &BaseState {
        match self {
            SimulationState::Base(base) => base,
            SimulationState::Eroded((base, _)) => base,
        }
    }

    pub fn base_mut(&mut self) -> &mut BaseState {
        match self {
            SimulationState::Base(base) => base,
            SimulationState::Eroded((base, _)) => base,
        }
    }

    pub fn eroded(&self) -> Option<&ErodedState> {
        match self {
            SimulationState::Base(_) => None,
            SimulationState::Eroded((_, eroded)) => Some(eroded),
        }
    }

    pub fn set_active_texture(&mut self, texture: &Rc<Texture2D>) {
        match self {
            SimulationState::Base(ref mut base) => base.set_active_texture(texture),
            SimulationState::Eroded((ref mut base, _)) => base.set_active_texture(texture),
        }
    }

    pub fn id(&self) -> usize {
        match self {
            SimulationState::Base(base) => base.id,
            SimulationState::Eroded((_, eroded)) => eroded.id,
        }
    }

    pub fn get_heightmap(&self) -> Rc<Heightmap> {
        match self {
            SimulationState::Base(base) => Rc::clone(&base.heightmap_base),
            SimulationState::Eroded((_, eroded)) => Rc::clone(&eroded.heightmap_eroded),
        }
    }
}

#[derive(Clone)]
pub struct BaseState {
    pub id: usize,
    pub erosion_method: Method,
    pub params: Parameters,
    pub drop_zone: DropZone,
    pub heightmap_base: Rc<Heightmap>,
    pub texture_heightmap_base: Rc<Texture2D>,
    pub texture_active: Rc<Texture2D>,
}

impl BaseState {
    pub fn run_simulation(&self, id: usize, parameters: &Parameters) -> ErodedState {
        print!("Eroding using ");
        let mut heightmap: Heightmap = (*self.heightmap_base).clone();
        match self.erosion_method {
            partitioning::Method::Default => {
                println!(
                    "{} method (no partitioning)",
                    partitioning::Method::Default.to_string()
                );
                partitioning::default_erode(&mut heightmap, &parameters, &self.drop_zone);
            }
            partitioning::Method::Subdivision => {
                println!("{} method", partitioning::Method::Subdivision.to_string());
                partitioning::subdivision_erode(&mut heightmap, &parameters, SUBDIVISIONS);
            }
            partitioning::Method::SubdivisionOverlap => {
                println!(
                    "{} method",
                    partitioning::Method::SubdivisionOverlap.to_string()
                );
                partitioning::subdivision_overlap_erode(&mut heightmap, &parameters, SUBDIVISIONS);
            }
            partitioning::Method::GridOverlapBlend => {
                println!(
                    "{} method",
                    partitioning::Method::GridOverlapBlend.to_string()
                );
                partitioning::grid_overlap_blend_erode(
                    &mut heightmap,
                    &parameters,
                    GRID_SIZE,
                    GRID_SIZE,
                );
            }
        }
        let heightmap_eroded_texture = heightmap_to_texture(&heightmap);
        let mut heightmap_diff = heightmap.subtract(&self.heightmap_base).unwrap();
        let heightmap_diff_texture = heightmap_to_texture(&heightmap_diff);
        let heightmap_diff_normalized = heightmap_diff.clone().normalize();
        let heightmap_diff_normalized_texture = heightmap_to_texture(&heightmap_diff_normalized);
        println!("Done!");

        heightmap.calculate_total_height();
        heightmap_diff.calculate_total_height();
        ErodedState {
            id,
            base_id: self.id,
            diffs: Rc::new(RefCell::new(vec![self.id])),
            selected_diff: Rc::new(RefCell::new(self.id)),
            heightmap_eroded: Rc::new(heightmap),
            heightmap_difference: Rc::new(RefCell::new(vec![Rc::new(heightmap_diff)])),
            erosion_method: Rc::new(self.erosion_method),
            texture_eroded: Rc::new(heightmap_eroded_texture),
            texture_difference: Rc::new(RefCell::new(vec![Rc::new(heightmap_diff_texture)])),
            texture_difference_normalized: Rc::new(RefCell::new(vec![Rc::new(
                heightmap_diff_normalized_texture,
            )])),
        }
    }

    pub fn set_active_texture(&mut self, texture: &Rc<Texture2D>) {
        self.texture_active = Rc::clone(texture);
    }
}

pub struct ErodedState {
    pub id: usize,
    pub base_id: usize,
    pub diffs: Rc<RefCell<Vec<usize>>>,
    pub selected_diff: Rc<RefCell<usize>>,
    pub heightmap_eroded: Rc<Heightmap>,
    pub heightmap_difference: Rc<RefCell<Vec<Rc<Heightmap>>>>,
    pub erosion_method: Rc<Method>,
    pub texture_eroded: Rc<Texture2D>,
    pub texture_difference: Rc<RefCell<Vec<Rc<Texture2D>>>>,
    pub texture_difference_normalized: Rc<RefCell<Vec<Rc<Texture2D>>>>,
}

impl ErodedState {
    pub fn diff_index_of(&self, diff_id: &usize) -> Option<usize> {
        for (i, d) in self.diffs.borrow().iter().enumerate() {
            if *diff_id == *d {
                return Some(i);
            }
        }
        None
    }
}

pub struct AppParameters {
    pub erosion_params: Parameters,
    pub heightmap_settings: HeightmapSettings,
    pub auto_apply: bool,
}

impl Default for AppParameters {
    fn default() -> Self {
        AppParameters {
            erosion_params: Parameters::default(),
            heightmap_settings: HeightmapSettings::default(),
            auto_apply: true,
        }
    }
}

pub struct AppState {
    pub simulation_states: Vec<SimulationState>,
    pub simulation_base_indices: Vec<usize>,
    pub parameters: AppParameters,
}

impl AppState {
    pub fn simulation_state(&self) -> &SimulationState {
        &self.simulation_states[*self.simulation_base_indices.last().unwrap()]
    }

    pub fn simulation_state_mut(&mut self) -> &mut SimulationState {
        &mut self.simulation_states[*self.simulation_base_indices.last().unwrap()]
    }
}

pub async fn run() {
    prevent_quit();

    let mut ui_state = UiState {
        show_ui_all: true,
        show_ui_keybinds: false,
        show_ui_control_panel: true,
        show_ui_metadata: false,
        show_ui_metrics: true,
        simulation_clear: true,
        simulation_regenerate: false,
        application_quit: false,
        ui_events: Vec::<UiEvent>::new(),
        ui_events_previous: Vec::<UiEvent>::new(),
        frame_slots: None,
    };

    let mut state = AppState {
        simulation_states: vec![SimulationState::get_new_base(
            0,
            None,
            &Parameters::default(),
        )],
        simulation_base_indices: vec![0],
        parameters: AppParameters::default(),
    };

    let mut corrected_size = false;

    // Update heightmap data
    while ui_state.simulation_clear && !ui_state.application_quit {
        ui_state.simulation_clear = false;

        if ui_state.simulation_regenerate {
            state.simulation_states.push(SimulationState::get_new_base(
                state.simulation_states.len(),
                Some(&state.parameters.heightmap_settings),
                &state.parameters.erosion_params,
            ));
            state
                .simulation_base_indices
                .push(state.simulation_states.len() - 1);
            ui_state.simulation_regenerate = false;
        }

        // Update UI
        while !is_quit_requested() && !ui_state.simulation_clear && !ui_state.application_quit {
            clear_background(BLACK);

            let canvas_rect = ui_state
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
                &state.simulation_state().base().texture_active,
            );

            ui_state.frame_slots = ui_draw(&mut ui_state, &mut state);

            poll_ui_events(&mut ui_state, &mut state);
            poll_ui_keybinds(&mut ui_state);
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