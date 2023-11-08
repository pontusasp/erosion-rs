use std::{collections::HashSet, mem, rc::Rc};

use egui::{Color32, Rect};
use macroquad::prelude::*;

#[cfg(feature = "export")]
use super::SimulationState;
#[cfg(feature = "export")]
use crate::heightmap::io::export_heightmaps;

use crate::heightmap::HeightmapPrecision;
use crate::partitioning;
use crate::visualize::events::UiEvent;
use crate::visualize::wrappers::HeightmapTexture;

use super::{
    mix_heightmap_to_texture,
    panels::{
        ui_keybinds_window, ui_metadata_window, ui_metrics_window, ui_side_panel, ui_top_panel,
    },
    AppState,
};

#[derive(Clone, Copy, PartialEq)]
pub struct IsolineProperties {
    pub height: HeightmapPrecision,
    pub error: HeightmapPrecision,
    pub flood_lower: bool,
    pub should_flood: bool,
    pub flooded_areas_lower: Option<usize>,
    pub flooded_areas_higher: Option<usize>,
    pub blur_augmentation: (bool, f32),
}

pub struct UiState {
    pub show_ui_all: bool,
    pub show_ui_keybinds: bool,
    pub show_ui_control_panel: bool,
    pub show_ui_metadata: bool,
    pub show_ui_metrics: bool,
    pub simulation_clear: bool,
    pub simulation_regenerate: bool,
    pub application_quit: bool,
    pub ui_events: Vec<UiEvent>,
    pub ui_events_previous: Vec<UiEvent>,
    pub frame_slots: Option<FrameSlots>,
    pub blur_sigma: f32,
    pub canny_edge: (f32, f32),
    pub isoline: IsolineProperties,
}

impl UiState {
    pub fn clear_events(&mut self) {
        mem::swap(&mut self.ui_events_previous, &mut self.ui_events);
        self.ui_events.clear();
    }
}

pub struct FrameSlots {
    pub canvas: Option<Rect>,
}

pub fn ui_draw(ui_state: &mut UiState, state: &mut AppState) -> Option<FrameSlots> {
    if ui_state.show_ui_all {
        let mut central_rect = None;
        egui_macroquad::ui(|egui_ctx| {
            // Top Panel
            ui_top_panel(egui_ctx, ui_state);

            // Side Panel
            ui_side_panel(egui_ctx, ui_state, state);

            // Central Panel
            central_rect = Some(
                egui::CentralPanel::default()
                    .frame(egui::containers::Frame {
                        fill: Color32::TRANSPARENT,
                        ..Default::default()
                    })
                    .show(egui_ctx, |_| {})
                    .response
                    .rect,
            );

            ui_keybinds_window(egui_ctx, ui_state);
            ui_metadata_window(egui_ctx, ui_state, state);
            ui_metrics_window(egui_ctx, ui_state, state);
        });

        egui_macroquad::draw();
        Some(FrameSlots {
            canvas: central_rect,
        })
    } else {
        None
    }
}
