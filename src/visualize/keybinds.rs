use std::collections::HashSet;

use egui::{Context, Key};

use crate::visualize::events::{UiEvent, UiWindow};
use crate::visualize::ui::UiState;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum UiKey {
    Single(Key),
    Shift(Key),
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum UiKeybind {
    Pressed(UiKey, UiEvent),
    Down(UiKey, UiEvent),
}

pub const KEYCODE_TOGGLE_ALL_UI: Key = Key::F1;
pub const KEYCODE_TOGGLE_CONTROL_PANEL_UI: Key = Key::F2;
pub const KEYCODE_TOGGLE_KEYBINDS_UI: Key = Key::F3;
pub const KEYCODE_TOGGLE_METADATA_UI: Key = Key::F4;
pub const KEYCODE_TOGGLE_METRICS_UI: Key = Key::F5;
pub const KEYCODE_NEW_HEIGHTMAP: Key = Key::G;
pub const KEYCODE_NEXT_PARTITIONING_METHOD: Key = Key::J;
pub const KEYCODE_PREVIOUS_PARTITIONING_METHOD: Key = Key::K;
pub const KEYBINDS: &[UiKeybind] = &[
    UiKeybind::Pressed(
        UiKey::Single(KEYCODE_TOGGLE_ALL_UI),
        UiEvent::ToggleUi(UiWindow::All),
    ),
    UiKeybind::Pressed(
        UiKey::Single(Key::F2),
        UiEvent::ToggleUi(UiWindow::ControlPanel),
    ),
    UiKeybind::Pressed(
        UiKey::Single(KEYCODE_TOGGLE_KEYBINDS_UI),
        UiEvent::ToggleUi(UiWindow::Keybinds),
    ),
    UiKeybind::Pressed(UiKey::Single(KEYCODE_NEW_HEIGHTMAP), UiEvent::NewHeightmap),
    UiKeybind::Pressed(UiKey::Single(Key::R), UiEvent::Clear),
    #[cfg(feature = "export")]
    UiKeybind::Pressed(UiKey::Single(Key::S), UiEvent::ExportHeightmap),
    UiKeybind::Pressed(UiKey::Single(Key::Tab), UiEvent::RunSimulation),
    UiKeybind::Pressed(UiKey::Single(Key::Q), UiEvent::Quit),
    UiKeybind::Pressed(UiKey::Single(Key::Escape), UiEvent::Quit),
    UiKeybind::Down(UiKey::Single(Key::Space), UiEvent::ShowBaseLayer),
    UiKeybind::Down(UiKey::Single(Key::D), UiEvent::ShowDifference),
    UiKeybind::Down(
        UiKey::Shift(Key::D),
        UiEvent::ShowDifferenceNormalized,
    ),
    UiKeybind::Pressed(
        UiKey::Single(KEYCODE_NEXT_PARTITIONING_METHOD),
        UiEvent::NextPartitioningMethod,
    ),
    UiKeybind::Pressed(
        UiKey::Single(KEYCODE_PREVIOUS_PARTITIONING_METHOD),
        UiEvent::PreviousPartitioningMethod,
    ),
    UiKeybind::Pressed(UiKey::Single(Key::ArrowUp), UiEvent::PreviousState),
    UiKeybind::Pressed(UiKey::Single(Key::ArrowDown), UiEvent::NextState),
    UiKeybind::Pressed(UiKey::Single(Key::ArrowLeft), UiEvent::PreviousDiff),
    UiKeybind::Pressed(UiKey::Single(Key::ArrowRight), UiEvent::NextDiff),
    UiKeybind::Pressed(
        UiKey::Single(KEYCODE_TOGGLE_METADATA_UI),
        UiEvent::ToggleUi(UiWindow::Metadata),
    ),
    UiKeybind::Pressed(
        UiKey::Single(KEYCODE_TOGGLE_METRICS_UI),
        UiEvent::ToggleUi(UiWindow::Metrics),
    ),
    UiKeybind::Pressed(UiKey::Single(Key::V), UiEvent::ShowErodedLayer),
    UiKeybind::Pressed(UiKey::Single(Key::B), UiEvent::Blur),
    UiKeybind::Pressed(UiKey::Single(Key::C), UiEvent::EdgeDetect),
    UiKeybind::Pressed(UiKey::Single(Key::X), UiEvent::BlurEdgeDetect),
    UiKeybind::Pressed(UiKey::Single(Key::I), UiEvent::Isoline),
    #[cfg(feature = "export")]
    UiKeybind::Pressed(UiKey::Single(Key::W), UiEvent::ExportState),
];

pub fn poll_ui_keybinds(ctx: &Context, ui_state: &mut UiState) {
    ctx.input(|input| {
        let mut consumed_keys = HashSet::new();
        for &keybind in KEYBINDS.iter() {
            match keybind {
                UiKeybind::Pressed(keybind, event) => match keybind {
                    UiKey::Single(_) => (),
                    UiKey::Shift(key_code) => {
                        if input.modifiers.shift
                            && input.key_pressed(key_code)
                            && !consumed_keys.contains(&key_code)
                        {
                            consumed_keys.insert(key_code);
                            ui_state.ui_events.push(event);
                        }
                    }
                },
                UiKeybind::Down(keybind, event) => match keybind {
                    UiKey::Single(_) => (),
                    UiKey::Shift(key_code) => {
                        if input.modifiers.shift
                            && input.key_down(key_code)
                            && !consumed_keys.contains(&key_code)
                        {
                            consumed_keys.insert(key_code);
                            ui_state.ui_events.push(event);
                        }
                    }
                },
            }
        }

        for &keybind in KEYBINDS.iter() {
            match keybind {
                UiKeybind::Pressed(keybind, event) => match keybind {
                    UiKey::Single(key_code) => {
                        if input.key_pressed(key_code) && !consumed_keys.contains(&key_code) {
                            consumed_keys.insert(key_code);
                            ui_state.ui_events.push(event);
                        }
                    }
                    UiKey::Shift(_) => (),
                },
                UiKeybind::Down(keybind, event) => match keybind {
                    UiKey::Single(key_code) => {
                        if input.key_down(key_code) && !consumed_keys.contains(&key_code) {
                            consumed_keys.insert(key_code);
                            ui_state.ui_events.push(event);
                        }
                    }
                    UiKey::Shift(_) => (),
                },
            }
        }
    });
}
