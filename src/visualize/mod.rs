use crate::heightmap;

use std::cell::RefCell;
use std::rc::Rc;

use egui::{Pos2, Rect};
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};

pub mod wrappers;
pub mod canvas;
pub mod panels;
pub mod ui;
pub mod widgets;

use crate::erode::DropZone;
use crate::erode::Parameters;
use crate::heightmap::{Heightmap, HeightmapType};
use crate::partitioning;
use crate::partitioning::Method;
use crate::visualize::ui::*;
use crate::visualize::wrappers::HeightmapTexture;

const SUBDIVISIONS: u32 = 3;
const GRID_SIZE: usize = 6;
const PRESET_HEIGHTMAP_SIZE: usize = 512;

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
    };

    let mut state = AppState {
        simulation_states: vec![SimulationState::get_new_base(
            0,
            &HeightmapType::default(),
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
                &state.parameters.heightmap_type,
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
                &state.simulation_state().get_active_texture(),
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
