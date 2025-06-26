use crate::erode::Parameters;
use crate::heightmap::HeightmapType;
use crate::visualize::app_state::{AppParameters, AppState, SimulationState};
use crate::visualize::events::UiEvent;
use crate::visualize::ui::{IsolineProperties, UiState};
use crate::io;


impl Default for ErosionApp {
    fn default() -> Self {
        Self::default_with_heightmap_type(&HeightmapType::default())
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct ErosionApp {
    pub state_name: Option<String>,
    // #[serde(skip)] // This how you opt-out of serialization of a field
    pub app_state: AppState,
    pub ui_state: UiState,
}

impl ErosionApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
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
                    &heightmap_type,
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
                saves: io::list_state_files()
                    .ok()
                    .or_else(|| Some(Vec::new()))
                    .expect("Failed to access saved states."),
                screenshots: 0,
            },
        }
    }
}


impl eframe::App for ErosionApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        if self.ui_state.simulation_regenerate {
            self
                .app_state
                .simulation_states
                .push(SimulationState::get_new_base(
                    self.app_state.simulation_states.len(),
                    &self.app_state.parameters.heightmap_type,
                    &self.app_state.parameters.erosion_params,
                ));
            self
                .app_state
                .simulation_base_indices
                .push(self.app_state.simulation_states.len() - 1);
            self.ui_state.simulation_regenerate = false;
        }


        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("Erosion-RS");

            ui.separator();

            ui.add(egui::github_link_file!(
                "https://github.com/pontusasp/erosion-rs/blob/main/",
                "Source code. :D"
            ));

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
            });
        });
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}
