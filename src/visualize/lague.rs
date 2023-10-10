use std::cell::RefCell;
use std::rc::Rc;

use macroquad::prelude::*;
use bracket_noise::prelude::*;

use crate::erode::lague;
use crate::erode::lague::DropZone;
use crate::erode::lague::Parameters;
use crate::heightmap;
use crate::heightmap::Heightmap;
use crate::heightmap::HeightmapSettings;
use crate::partitioning::Method;
use crate::visualize::heightmap_to_texture;
use crate::visualize::ui::*;
use crate::{erode, partitioning};

const SUBDIVISIONS: u32 = 3;
const ITERATIONS: usize = 1000000;

fn generate_drop_zone(heightmap: &heightmap::Heightmap) -> lague::DropZone {
    lague::DropZone::default(&heightmap)
}

pub enum SimulationState {
    Base(BaseState),
    Eroded((BaseState, ErodedState)),
}

impl SimulationState {
    pub fn get_new_base(new_id: usize, settings: Option<&HeightmapSettings>) -> Self {
        let mut heightmap = erode::initialize_heightmap(settings).normalize();
        heightmap.calculate_total_height();
        let texture = Rc::new(heightmap_to_texture(&heightmap));
        SimulationState::Base(BaseState {
            id: new_id,
            erosion_method: Method::Default,
            params: Parameters {
                num_iterations: ITERATIONS,
                ..Default::default()
            },
            drop_zone: DropZone::default(&heightmap),
            heightmap_base: Rc::new(heightmap),
            texture_heightmap_base: Rc::clone(&texture),
            texture_active: Rc::clone(&texture),
        })
    }

    pub fn get_new_eroded(&self, new_id: usize) -> Self {
        let (mut base, eroded) = match self {
            SimulationState::Base(base) => (base.clone(), None),
            SimulationState::Eroded((base, eroded)) => (base.clone(), Some(eroded)),
        };

        if let Some(eroded) = eroded {
            base = BaseState {
                id: eroded.id,
                erosion_method: base.erosion_method,
                params: base.params,
                drop_zone: base.drop_zone,
                heightmap_base: Rc::clone(&eroded.heightmap_eroded),
                texture_heightmap_base: Rc::clone(&eroded.texture_eroded),
                texture_active: Rc::clone(&eroded.texture_eroded),
            };
        }

        let eroded = base.run_simulation(new_id);
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
    pub fn run_simulation(&self, id: usize) -> ErodedState {
        print!("Eroding using ");
        let mut heightmap: Heightmap = (*self.heightmap_base).clone();
        match self.erosion_method {
            partitioning::Method::Default => {
                println!(
                    "{} method (no partitioning)",
                    partitioning::Method::Default.to_string()
                );
                partitioning::default_erode(&mut heightmap, &self.params, &self.drop_zone);
            }
            partitioning::Method::Subdivision => {
                println!("{} method", partitioning::Method::Subdivision.to_string());
                partitioning::subdivision_erode(&mut heightmap, &self.params, SUBDIVISIONS);
            }
            partitioning::Method::SubdivisionOverlap => {
                println!(
                    "{} method",
                    partitioning::Method::SubdivisionOverlap.to_string()
                );
                partitioning::subdivision_overlap_erode(&mut heightmap, &self.params, SUBDIVISIONS);
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
    pub lague_params: Parameters,
    pub heightmap_settings: HeightmapSettings,
    pub auto_apply: bool,
}

impl Default for AppParameters {
    fn default() -> Self {
        AppParameters {
            lague_params: Parameters::default(),
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

pub async fn visualize() {
    prevent_quit();

    let mut ui_state = UiState {
        show_ui_all: true,
        show_ui_keybinds: false,
        show_ui_control_panel: true,
        simulation_clear: true,
        simulation_regenerate: false,
        application_quit: false,
        ui_events: Vec::<UiEvent>::new(),
        ui_events_previous: Vec::<UiEvent>::new(),
    };

    let mut state = AppState {
        simulation_states: vec![SimulationState::get_new_base(0, None)],
        simulation_base_indices: vec![0],
        parameters: AppParameters::default(),
    };

    // Update heightmap data
    while ui_state.simulation_clear && !ui_state.application_quit {
        ui_state.simulation_clear = false;

        if ui_state.simulation_regenerate {
            state
                .simulation_states
                .push(SimulationState::get_new_base(state.simulation_states.len(), Some(&state.parameters.heightmap_settings)));
            state
                .simulation_base_indices
                .push(state.simulation_states.len() - 1);
            ui_state.simulation_regenerate = false;
        }

        // Update UI
        while !is_quit_requested() && !ui_state.simulation_clear && !ui_state.application_quit {
            draw_texture_ex(
                *state.simulation_state().base().texture_active,
                0.0,
                0.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(crate::WIDTH as f32, crate::HEIGHT as f32)),
                    ..Default::default()
                },
            );

            ui_draw(&mut ui_state, &mut state);
            poll_ui_keybinds(&mut ui_state);
            poll_ui_events(&mut ui_state, &mut state);
            next_frame().await;
        }
    }
}
