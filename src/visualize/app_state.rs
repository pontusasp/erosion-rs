use macroquad::texture::Texture2D;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;

use crate::erode::{DropZone, Parameters};
use crate::heightmap::{Heightmap, HeightmapType};
use crate::partitioning::Method;
use crate::visualize::wrappers::HeightmapTexture;
use crate::visualize::{GRID_SIZE, PRESET_HEIGHTMAP_SIZE, SUBDIVISIONS};
use crate::{heightmap, partitioning};

#[derive(Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
pub struct AppParameters {
    pub erosion_params: Parameters,
    pub heightmap_type: HeightmapType,
    pub auto_apply: bool,
}

impl Default for AppParameters {
    fn default() -> Self {
        AppParameters {
            erosion_params: Parameters::default(),
            heightmap_type: HeightmapType::default(),
            auto_apply: true,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ErodedState {
    pub id: usize,
    pub base_id: usize,
    pub diffs: Rc<RefCell<Vec<usize>>>,
    pub selected_diff: Rc<RefCell<usize>>,
    pub heightmap_eroded: Rc<HeightmapTexture>,
    pub heightmap_difference: Rc<RefCell<Vec<Rc<HeightmapTexture>>>>,
    pub heightmap_difference_normalized: Rc<RefCell<Vec<Rc<HeightmapTexture>>>>,
    pub erosion_method: Rc<Method>,
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

#[derive(Clone, Serialize, Deserialize)]
pub struct BaseState {
    pub id: usize,
    pub erosion_method: Method,
    pub params: Parameters,
    pub drop_zone: DropZone,
    pub heightmap_base: Rc<HeightmapTexture>,
    pub heightmap_active: Rc<HeightmapTexture>,
}

impl BaseState {
    pub fn run_simulation(&self, id: usize, parameters: &Parameters) -> ErodedState {
        print!("Eroding using ");
        let mut heightmap: Heightmap = (*self.heightmap_base.heightmap).clone();
        match self.erosion_method {
            Method::Default => {
                println!("{} method (no partitioning)", Method::Default.to_string());
                partitioning::default_erode(&mut heightmap, &parameters, &self.drop_zone);
            }
            Method::Subdivision => {
                println!("{} method", Method::Subdivision.to_string());
                partitioning::subdivision_erode(&mut heightmap, &parameters, SUBDIVISIONS);
            }
            Method::SubdivisionBlurBoundary((sigma, thickness)) => {
                println!(
                    "{} method",
                    Method::SubdivisionBlurBoundary((
                        partitioning::GAUSSIAN_DEFAULT_SIGMA,
                        partitioning::GAUSSIAN_DEFAULT_BOUNDARY_THICKNESS
                    ))
                    .to_string()
                );
                partitioning::subdivision_blur_boundary_erode(
                    &mut heightmap,
                    &parameters,
                    SUBDIVISIONS,
                    sigma,
                    thickness,
                );
            }
            Method::SubdivisionOverlap => {
                println!("{} method", Method::SubdivisionOverlap.to_string());
                partitioning::subdivision_overlap_erode(&mut heightmap, &parameters, SUBDIVISIONS);
            }
            Method::GridOverlapBlend => {
                println!("{} method", Method::GridOverlapBlend.to_string());
                partitioning::grid_overlap_blend_erode(
                    &mut heightmap,
                    &parameters,
                    GRID_SIZE,
                    GRID_SIZE,
                );
            }
        }
        let mut heightmap_diff = heightmap.subtract(&self.heightmap_base.heightmap).unwrap();
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
        }
    }

    pub fn set_active(&mut self, heightmap_texture: Rc<HeightmapTexture>) {
        self.heightmap_active = heightmap_texture;
    }
}

#[derive(Serialize, Deserialize)]
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
        let mut heightmap =
            heightmap::create_heightmap_from_preset(heightmap_type, PRESET_HEIGHTMAP_SIZE);
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
                heightmap_active: Rc::clone(&eroded.heightmap_eroded),
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

    pub fn set_active_separate(&mut self, heightmap: Rc<Heightmap>, texture: Rc<Texture2D>) {
        self.set_active(Rc::new(HeightmapTexture::new(heightmap, Some(texture))))
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
}

pub mod io {
    use super::AppState;
    use std::{fs, io};

    pub fn export(app_state: &AppState, file_path: &str) {
        let result = serde_json::to_string(app_state);
        if let Ok(json) = result {
            fs::write(file_path, json)
                .expect(&format!("Could not save app state to \"{}\"!", file_path));
        } else if let Err(e) = result {
            eprintln!("Failed to serialize app state! {}", e);
        }
    }

    pub enum AppStateImportError {
        ReadError(io::Error),
        InvalidFileFormat(serde_json::Error),
    }

    impl From<io::Error> for AppStateImportError {
        fn from(err: io::Error) -> Self {
            AppStateImportError::ReadError(err)
        }
    }

    impl From<serde_json::Error> for AppStateImportError {
        fn from(err: serde_json::Error) -> Self {
            AppStateImportError::InvalidFileFormat(err)
        }
    }

    pub fn import(file_path: &str) -> Result<AppState, AppStateImportError> {
        let data = fs::read_to_string(file_path)?;
        let result: AppState = serde_json::from_str(&data)?;
        Ok(result)
    }
}
