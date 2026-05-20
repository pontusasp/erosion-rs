use crate::erode::Parameters;
use crate::heightmap::HeightmapType;
use crate::visualize::app_state::{AppParameters, AppState, SimulationState};
use crate::visualize::events::{poll_ui_events, UiEvent};
use crate::visualize::keybinds::poll_ui_keybinds;
use crate::visualize::ui::{ui_draw, IsolineProperties, UiState};


impl Default for ErosionApp {
    fn default() -> Self {
        Self::default_with_heightmap_type(&HeightmapType::default())
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct ErosionApp {
    pub state_name: Option<String>,
    pub app_state: AppState,
    pub ui_state: UiState,

    /// Cached texture handle for displaying the active heightmap.
    #[serde(skip)]
    texture_handle: Option<egui::TextureHandle>,
    /// Cached texture handle for the grid overlay.
    #[serde(skip)]
    grid_texture_handle: Option<egui::TextureHandle>,
    /// Pointer value of the last ColorImage we uploaded, used for invalidation.
    #[serde(skip)]
    last_image_ptr: usize,
    /// Whether the grid texture needs to be regenerated.
    #[serde(skip)]
    grid_dirty: bool,
}

impl ErosionApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Load previous app state (if any).
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }

    pub fn default_with_heightmap_type(heightmap_type: &HeightmapType) -> Self {
        Self {
            state_name: None,
            app_state: AppState {
                simulation_states: vec![SimulationState::get_new_base(
                    0,
                    heightmap_type,
                    &Parameters::default(),
                )],
                simulation_base_indices: vec![0],
                parameters: AppParameters {
                    heightmap_type: *heightmap_type,
                    ..Default::default()
                },
            },
            ui_state: UiState {
                show_ui_all: true,
                show_ui_keybinds: false,
                show_ui_control_panel: true,
                show_ui_metadata: false,
                show_ui_metrics: false,
                show_ui_presentation_mode: true,
                show_grid: false,
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
                    should_flood: true,
                    flooded_areas_lower: None,
                    flooded_areas_higher: None,
                    blur_augmentation: (false, 1.0, 5, 5),
                    advanced_texture: true,
                    flooded_errors: None,
                },
                #[cfg(feature = "export")]
                saves: crate::io::list_state_files()
                    .ok()
                    .or_else(|| Some(Vec::new()))
                    .expect("Failed to access saved states."),
                screenshots: 0,
            },
            texture_handle: None,
            grid_texture_handle: None,
            last_image_ptr: 0,
            grid_dirty: true,
        }
    }

    /// Create an ErosionApp instance for serialization/export purposes only.
    pub fn for_export(
        state_name: Option<String>,
        app_state: AppState,
        ui_state: UiState,
    ) -> Self {
        Self {
            state_name,
            app_state,
            ui_state,
            texture_handle: None,
            grid_texture_handle: None,
            last_image_ptr: 0,
            grid_dirty: true,
        }
    }
}


impl eframe::App for ErosionApp {
    /// Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle regeneration
        if self.ui_state.simulation_regenerate {
            self.app_state
                .simulation_states
                .push(SimulationState::get_new_base(
                    self.app_state.simulation_states.len(),
                    &self.app_state.parameters.heightmap_type,
                    &self.app_state.parameters.erosion_params,
                ));
            self.app_state
                .simulation_base_indices
                .push(self.app_state.simulation_states.len() - 1);
            self.ui_state.simulation_regenerate = false;
            self.grid_dirty = true;
        }

        // Handle clear (reset state)
        if self.ui_state.simulation_clear {
            let new_state = Self::default_with_heightmap_type(&self.app_state.parameters.heightmap_type);
            self.app_state = new_state.app_state;
            self.ui_state.simulation_clear = false;
            self.texture_handle = None;
            self.grid_texture_handle = None;
            self.last_image_ptr = 0;
            self.grid_dirty = true;
        }

        // Draw UI panels and windows
        ui_draw(
            ctx,
            &mut self.ui_state,
            &mut self.app_state,
            &mut self.state_name,
        );

        // Central panel with heightmap rendering
        egui::CentralPanel::default().show(ctx, |ui| {
            let active_image = self.app_state.simulation_state().get_active_image();
            let image_ptr = std::rc::Rc::as_ptr(&active_image) as usize;

            // Update texture if image changed
            if image_ptr != self.last_image_ptr || self.texture_handle.is_none() {
                self.texture_handle = Some(ctx.load_texture(
                    "heightmap",
                    (*active_image).clone(),
                    egui::TextureOptions::NEAREST,
                ));
                self.last_image_ptr = image_ptr;
                self.grid_dirty = true;
            }

            // Update grid texture if needed
            if self.ui_state.show_grid && self.grid_dirty {
                let grid_image = self
                    .app_state
                    .simulation_state()
                    .get_active_grid_texture(&self.app_state.parameters);
                self.grid_texture_handle = Some(ctx.load_texture(
                    "grid_overlay",
                    grid_image,
                    egui::TextureOptions::NEAREST,
                ));
                self.grid_dirty = false;
            }

            // Render heightmap
            if let Some(ref texture) = self.texture_handle {
                let available = ui.available_size();
                let side = available.x.min(available.y);
                let size = egui::vec2(side, side);
                ui.centered_and_justified(|ui| {
                    ui.image(egui::load::SizedTexture::new(texture.id(), size));
                });

                // Render grid overlay on top
                if self.ui_state.show_grid {
                    if let Some(ref grid_tex) = self.grid_texture_handle {
                        let rect = ui.min_rect();
                        let margin_x = (rect.width() - side) / 2.0;
                        let margin_y = (rect.height() - side) / 2.0;
                        let img_rect = egui::Rect::from_min_size(
                            egui::pos2(rect.min.x + margin_x, rect.min.y + margin_y),
                            size,
                        );
                        ui.painter().image(
                            grid_tex.id(),
                            img_rect,
                            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                            egui::Color32::WHITE,
                        );
                    }
                }
            }
        });

        // Poll keybinds
        poll_ui_keybinds(ctx, &mut self.ui_state);

        // Process events
        poll_ui_events(
            #[cfg(feature = "export")]
            &mut self.state_name,
            &mut self.ui_state,
            &mut self.app_state,
        );

        // Handle quit
        if self.ui_state.application_quit {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            self.ui_state.application_quit = false;
        }
    }
}
