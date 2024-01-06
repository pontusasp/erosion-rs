use macroquad::texture::{Image, Texture2D};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use crate::erode::{DropZone, Parameters};
use crate::heightmap::{self, Heightmap, HeightmapType};
use crate::partitioning::Method;
use crate::visualize::wrappers::HeightmapTexture;
use crate::visualize::{
    layered_heightmaps_to_texture, rgba_color_channel, HeightmapLayer, LayerMixMethod,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppParameters {
    pub erosion_params: Parameters,
    pub heightmap_type: HeightmapType,
    pub auto_apply: bool,
    pub grid_size: usize,
    pub margin: bool,
}

impl Default for AppParameters {
    fn default() -> Self {
        AppParameters {
            erosion_params: Parameters::default(),
            heightmap_type: HeightmapType::default(),
            auto_apply: true,
            grid_size: crate::PRESET_GRID_SIZE,
            margin: true,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ErodedState {
    pub id: usize,
    pub base_id: usize,
    pub diffs: Rc<RefCell<Vec<usize>>>,
    pub selected_diff: Rc<RefCell<usize>>,
    pub heightmap_eroded: Rc<HeightmapTexture>,
    pub heightmap_difference: Rc<RefCell<Vec<Rc<HeightmapTexture>>>>,
    pub heightmap_difference_normalized: Rc<RefCell<Vec<Rc<HeightmapTexture>>>>,
    pub erosion_method: Rc<Method>,
    pub margin_removed: bool,
    pub simulation_time: Duration,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseState {
    pub id: usize,
    pub erosion_method: Method,
    pub params: Parameters,
    pub drop_zone: DropZone,
    pub heightmap_base: Rc<HeightmapTexture>,
    pub heightmap_active: Rc<HeightmapTexture>,
}

impl BaseState {
    pub fn run_simulation(
        &self,
        id: usize,
        parameters: &Parameters,
        grid_size: usize,
        margin: bool,
    ) -> ErodedState {
        let time = std::time::Instant::now();
        let mut heightmap: Heightmap = self.erosion_method.erode_with_margin(
            margin,
            &self.heightmap_base.heightmap,
            parameters,
            &self.drop_zone,
            grid_size,
        );
        let elapsed = time.elapsed();
        heightmap.metadata_add("simulation_time", format!("{}", elapsed.as_secs_f32()));
        let new_margin = if margin {
            Method::max_margin(self.heightmap_base.heightmap.width, grid_size)
        } else {
            (0, 0, 0, 0)
        };
        let mut heightmap_diff = heightmap
            .subtract(
                &self
                    .heightmap_base
                    .heightmap
                    .with_margin(new_margin)
                    .heightmap,
            )
            .unwrap();
        let heightmap_diff_normalized = heightmap_diff.clone().normalize();
        println!("Done!");

        heightmap.calculate_total_height();
        heightmap_diff.calculate_total_height();
        ErodedState {
            id,
            base_id: self.id,
            diffs: Rc::new(RefCell::new(vec![self.id])),
            selected_diff: Rc::new(RefCell::new(self.id)),
            heightmap_eroded: Rc::new(heightmap.into()),
            heightmap_difference: Rc::new(RefCell::new(vec![Rc::new(heightmap_diff.into())])),
            heightmap_difference_normalized: Rc::new(RefCell::new(vec![Rc::new(
                heightmap_diff_normalized.into(),
            )])),
            erosion_method: Rc::new(self.erosion_method),
            margin_removed: margin,
            simulation_time: elapsed,
        }
    }

    pub fn set_active(&mut self, heightmap_texture: Rc<HeightmapTexture>) {
        self.heightmap_active = heightmap_texture;
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SimulationState {
    Base(BaseState),
    Eroded((BaseState, ErodedState)),
}

impl SimulationState {
    pub fn get_new_base(
        new_id: usize,
        heightmap_type: &HeightmapType,
        parameters: &Parameters,
    ) -> Self {
        let mut heightmap = heightmap::create_heightmap_from_preset(heightmap_type);
        heightmap.calculate_total_height();
        let heightmap = Rc::new(heightmap);
        SimulationState::Base(BaseState {
            id: new_id,
            erosion_method: Method::Default,
            params: parameters.clone(),
            drop_zone: DropZone::default(&heightmap),
            heightmap_base: Rc::new((&heightmap).into()),
            heightmap_active: Rc::new((&heightmap).into()),
        })
    }

    pub fn get_new_eroded(
        &self,
        new_id: usize,
        parameters: &Parameters,
        grid_size: usize,
        margin: bool,
    ) -> Self {
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
                heightmap_active: Rc::clone(&eroded.heightmap_eroded),
            };
        }

        let eroded = base.run_simulation(new_id, parameters, grid_size, margin);
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

    pub fn eroded_mut(&mut self) -> Option<&mut ErodedState> {
        match self {
            SimulationState::Base(_) => None,
            SimulationState::Eroded((_, eroded)) => Some(eroded),
        }
    }

    pub fn eroded(&self) -> Option<&ErodedState> {
        match self {
            SimulationState::Base(_) => None,
            SimulationState::Eroded((_, eroded)) => Some(eroded),
        }
    }

    pub fn get_active_heightmap_texture(&self) -> Rc<HeightmapTexture> {
        Rc::clone(&self.base().heightmap_active)
    }

    pub fn get_active(&self) -> Rc<Heightmap> {
        Rc::clone(&self.get_active_heightmap_texture().heightmap)
    }

    pub fn get_active_texture(&self) -> Rc<Texture2D> {
        if let Some(texture) = &self.get_active_heightmap_texture().texture {
            Rc::clone(texture)
        } else {
            eprintln!("WARN: get_active_texture(&self) called without any active texture set!");
            Rc::clone(&self.get_active_heightmap_texture().get_or_generate())
        }
    }

    pub fn set_active(&mut self, heightmap_texture: Rc<HeightmapTexture>) {
        self.base_mut().set_active(heightmap_texture);
    }

    pub fn set_active_separate(&mut self, heightmap: Rc<Heightmap>, image: Rc<Image>) {
        self.set_active(Rc::new(HeightmapTexture::new(heightmap, Some(image))))
    }

    pub fn id(&self) -> usize {
        match self {
            SimulationState::Base(base) => base.id,
            SimulationState::Eroded((_, eroded)) => eroded.id,
        }
    }

    pub fn get_heightmap(&self) -> Rc<Heightmap> {
        match self {
            SimulationState::Base(base) => Rc::clone(&base.heightmap_base.heightmap),
            SimulationState::Eroded((_, eroded)) => Rc::clone(&eroded.heightmap_eroded.heightmap),
        }
    }

    pub fn get_active_grid_texture(&self, app_parameters: &AppParameters) -> Texture2D {
        let grid = if let Some(state) = self.eroded() {
            state.erosion_method.get_grid(
                state.heightmap_eroded.heightmap.width,
                !state.margin_removed && app_parameters.margin,
                app_parameters.grid_size,
            )
        } else {
            let state = self.base();
            state.erosion_method.get_grid(
                state.heightmap_base.heightmap.width,
                app_parameters.margin,
                app_parameters.grid_size,
            )
        };
        let heightmap = self.get_active();
        let grid_texture = layered_heightmaps_to_texture(
            grid.width,
            &vec![
                &HeightmapLayer {
                    heightmap: &heightmap,
                    channel: rgba_color_channel::RGB,
                    strength: 1.0,
                    layer_mix_method: LayerMixMethod::Additive,
                    inverted: false,
                    modifies_alpha: false,
                },
                &HeightmapLayer {
                    heightmap: &grid,
                    channel: rgba_color_channel::RA,
                    strength: 1.0,
                    layer_mix_method: LayerMixMethod::Additive,
                    inverted: false,
                    modifies_alpha: false,
                },
            ],
            false,
            1.0,
        );
        grid_texture
    }
}
