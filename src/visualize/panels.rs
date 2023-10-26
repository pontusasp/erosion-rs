use egui::Rect;

use super::{
    ui::{
        UiEvent, UiKey, UiKeybind, UiState, UiWindow, KEYBINDS, KEYCODE_TOGGLE_ALL_UI,
        KEYCODE_TOGGLE_CONTROL_PANEL_UI, KEYCODE_TOGGLE_KEYBINDS_UI, KEYCODE_TOGGLE_METADATA_UI,
        KEYCODE_TOGGLE_METRICS_UI,
    },
    widgets::*,
    AppState,
};

pub fn ui_top_panel(egui_ctx: &egui::Context, ui_state: &mut UiState) {
    egui::TopBottomPanel::top("top_panel").show(egui_ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            ui.heading("Erosion RS");
            if ui
                .button(format!(
                    "[{:?}] {} UI",
                    KEYCODE_TOGGLE_ALL_UI,
                    if ui_state.show_ui_all { "Hide" } else { "Show" }
                ))
                .clicked()
            {
                ui_state.ui_events.push(UiEvent::ToggleUi(UiWindow::All));
            }
            if ui
                .button(format!(
                    "[{:?}] {} Control Panel",
                    KEYCODE_TOGGLE_CONTROL_PANEL_UI,
                    if ui_state.show_ui_control_panel {
                        "Hide"
                    } else {
                        "Show"
                    }
                ))
                .clicked()
            {
                ui_state
                    .ui_events
                    .push(UiEvent::ToggleUi(UiWindow::ControlPanel));
            }
            if ui
                .button(format!(
                    "[{:?}] {} Keybinds",
                    KEYCODE_TOGGLE_KEYBINDS_UI,
                    if ui_state.show_ui_keybinds {
                        "Hide"
                    } else {
                        "Show"
                    }
                ))
                .clicked()
            {
                ui_state
                    .ui_events
                    .push(UiEvent::ToggleUi(UiWindow::Keybinds));
            };
            if ui
                .button(format!(
                    "[{:?}] {} Metadata",
                    KEYCODE_TOGGLE_METADATA_UI,
                    if ui_state.show_ui_metadata {
                        "Hide"
                    } else {
                        "Show"
                    }
                ))
                .clicked()
            {
                ui_state
                    .ui_events
                    .push(UiEvent::ToggleUi(UiWindow::Metadata));
            };
            if ui
                .button(format!(
                    "[{:?}] {} Metrics",
                    KEYCODE_TOGGLE_METRICS_UI,
                    if ui_state.show_ui_metrics {
                        "Hide"
                    } else {
                        "Show"
                    }
                ))
                .clicked()
            {
                ui_state
                    .ui_events
                    .push(UiEvent::ToggleUi(UiWindow::Metrics));
            };
        });
    });
}

pub fn ui_side_panel(egui_ctx: &egui::Context, ui_state: &mut UiState, state: &mut AppState) {
    egui::SidePanel::left("left_panel").show_animated(
        egui_ctx,
        ui_state.show_ui_control_panel,
        |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                // Erosion Method Selection
                erosion_method_selection(ui, ui_state, state);
                erosion_parameter_selection(ui, state);
                layer_selection(ui, state);
                heightmap_generation_settings(ui, ui_state, state);
                post_processing(ui, ui_state);
            });
        },
    );
}

pub fn ui_keybinds_window(egui_ctx: &egui::Context, ui_state: &mut UiState) {
    if ui_state.show_ui_keybinds {
        egui::Window::new(format!("Keybinds [{:?}]", KEYCODE_TOGGLE_KEYBINDS_UI)).show(
            egui_ctx,
            |ui| {
                for keybind in KEYBINDS {
                    match keybind {
                        UiKeybind::Pressed(keys, event) => {
                            ui.horizontal(|ui| {
                                if ui.button(event.info()).clicked() {
                                    ui_state.ui_events.push(*event);
                                }
                                match keys {
                                    UiKey::Single(key_code) => {
                                        ui.label(format!("[{:?}]", key_code))
                                    }
                                    UiKey::Double(key_codes) => {
                                        ui.label(format!("[{:?}-{:?}]", key_codes.0, key_codes.1))
                                    }
                                };
                            });
                        }
                        UiKeybind::Down(keys, event) => {
                            if ui_state.ui_events_previous.contains(&event) {
                                ui.label(event.info());
                            } else {
                                if ui.button(event.info()).clicked() {
                                    ui_state.ui_events.push(*event);
                                }
                            }
                            match keys {
                                UiKey::Single(key_code) => ui.label(format!("({:?})", key_code)),
                                UiKey::Double(key_codes) => {
                                    ui.label(format!("({:?}-{:?})", key_codes.0, key_codes.1))
                                }
                            };
                        }
                    }
                }
            },
        );
    }
}

pub fn ui_metadata_window(egui_ctx: &egui::Context, ui_state: &mut UiState, state: &mut AppState) {
    if ui_state.show_ui_metadata {
        egui::Window::new(format!("Metadata")).show(egui_ctx, |ui| {
            ui.heading("Base Heightmap");
            ui.label(format!(
                "Width x Height: {} x {}",
                state.simulation_state().base().heightmap_base.width,
                state.simulation_state().base().heightmap_base.height
            ));
            ui.label(format!(
                "Depth: {}",
                state.simulation_state().base().heightmap_base.depth
            ));
            ui.label(format!(
                "Original Depth: {}",
                state
                    .simulation_state()
                    .base()
                    .heightmap_base
                    .original_depth
            ));
            if let Some(height) = state
                .simulation_state()
                .get_heightmap()
                .get_average_height()
            {
                ui.label(format!("Average Height: {}", height));
            }
            if let Some(height) = state.simulation_state().base().heightmap_base.total_height {
                ui.label(format!("Total Depth: {}", height));
            }
            if let Some(metadata) = state
                .simulation_state()
                .base()
                .heightmap_base
                .metadata
                .clone()
            {
                for (k, v) in metadata.iter() {
                    ui.label(format!("{}: {}", k, v));
                }
            }
            if let Some(eroded) = state.simulation_state().eroded() {
                ui.heading("Eroded Heightmap");
                ui.label(format!(
                    "Width x Height: {} x {}",
                    eroded.heightmap_eroded.width,
                    state.simulation_state().base().heightmap_base.height
                ));
                ui.label(format!("Depth: {}", eroded.heightmap_eroded.depth));
                ui.label(format!(
                    "Original Depth: {}",
                    eroded.heightmap_eroded.original_depth
                ));
                if let Some(height) = eroded.heightmap_eroded.get_average_height() {
                    ui.label(format!("Average Height: {}", height));
                }
                if let Some(height) = eroded.heightmap_eroded.total_height {
                    ui.label(format!("Total Depth: {}", height));
                }
                if let Some(metadata) = eroded.heightmap_eroded.metadata.clone() {
                    for (k, v) in metadata.iter() {
                        ui.label(format!("{}: {}", k, v));
                    }
                }
            }
        });
    }
}

pub fn ui_metrics_window(
    egui_ctx: &egui::Context,
    ui_state: &mut UiState,
    state: &mut AppState,
) -> Option<Rect> {
    let mut rect = None;
    if ui_state.show_ui_metrics {
        rect = Some(
            egui::Window::new(format!("Metrics [{:?}]", KEYCODE_TOGGLE_METRICS_UI))
                .show(egui_ctx, |ui| {
                    ui.heading("Average Height");
                    plot_height(ui, state);
                })
                .unwrap()
                .response
                .rect,
        );
    }
    rect
}
