use bevy::prelude::*;
use std::collections::HashSet;
use crate::visualize::events::{UiEvent, UiWindow};
use crate::visualize::ui::UiState;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum UiKey {
    Single(KeyCode),
    Double((KeyCode, KeyCode)),
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum UiKeybind {
    Pressed(UiKey, UiEvent),
    Down(UiKey, UiEvent),
}

pub const KEYCODE_TOGGLE_ALL_UI: KeyCode = KeyCode::F1;
pub const KEYCODE_TOGGLE_CONTROL_PANEL_UI: KeyCode = KeyCode::F2;
pub const KEYCODE_TOGGLE_KEYBINDS_UI: KeyCode = KeyCode::F3;
pub const KEYCODE_TOGGLE_METADATA_UI: KeyCode = KeyCode::F4;
pub const KEYCODE_TOGGLE_METRICS_UI: KeyCode = KeyCode::F5;
pub const KEYCODE_NEW_HEIGHTMAP: KeyCode = KeyCode::G;
pub const KEYCODE_NEXT_PARTITIONING_METHOD: KeyCode = KeyCode::J;
pub const KEYCODE_PREVIOUS_PARTITIONING_METHOD: KeyCode = KeyCode::K;
pub const KEYBINDS: &[UiKeybind] = &[
    UiKeybind::Pressed(
        UiKey::Single(KEYCODE_TOGGLE_ALL_UI),
        UiEvent::ToggleUi(UiWindow::All),
    ),
    UiKeybind::Pressed(
        UiKey::Single(KeyCode::F2),
        UiEvent::ToggleUi(UiWindow::ControlPanel),
    ),
    UiKeybind::Pressed(
        UiKey::Single(KEYCODE_TOGGLE_KEYBINDS_UI),
        UiEvent::ToggleUi(UiWindow::Keybinds),
    ),
    UiKeybind::Pressed(UiKey::Single(KEYCODE_NEW_HEIGHTMAP), UiEvent::NewHeightmap),
    UiKeybind::Pressed(UiKey::Single(KeyCode::KeyR), UiEvent::Clear),
    #[cfg(feature = "export")]
    UiKeybind::Pressed(UiKey::Single(KeyCode::KeyS), UiEvent::ExportHeightmap),
    UiKeybind::Pressed(UiKey::Single(KeyCode::Tab), UiEvent::RunSimulation),
    UiKeybind::Pressed(UiKey::Single(KeyCode::KeyQ), UiEvent::Quit),
    UiKeybind::Pressed(UiKey::Single(KeyCode::Escape), UiEvent::Quit),
    UiKeybind::Down(UiKey::Single(KeyCode::Space), UiEvent::ShowBaseLayer),
    UiKeybind::Down(UiKey::Single(KeyCode::KeyD), UiEvent::ShowDifference),
    UiKeybind::Down(
        UiKey::Double((KeyCode::ShiftLeft, KeyCode::KeyD)),
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
    UiKeybind::Pressed(UiKey::Single(KeyCode::ArrowUp), UiEvent::PreviousState),
    UiKeybind::Pressed(UiKey::Single(KeyCode::ArrowDown), UiEvent::NextState),
    UiKeybind::Pressed(UiKey::Single(KeyCode::ArrowLeft), UiEvent::PreviousDiff),
    UiKeybind::Pressed(UiKey::Single(KeyCode::ArrowRight), UiEvent::NextDiff),
    UiKeybind::Pressed(
        UiKey::Single(KEYCODE_TOGGLE_METADATA_UI),
        UiEvent::ToggleUi(UiWindow::Metadata),
    ),
    UiKeybind::Pressed(
        UiKey::Single(KEYCODE_TOGGLE_METRICS_UI),
        UiEvent::ToggleUi(UiWindow::Metrics),
    ),
    UiKeybind::Pressed(UiKey::Single(KeyCode::KeyV), UiEvent::ShowErodedLayer),
    UiKeybind::Pressed(UiKey::Single(KeyCode::KeyB), UiEvent::Blur),
    UiKeybind::Pressed(UiKey::Single(KeyCode::KeyC), UiEvent::EdgeDetect),
    UiKeybind::Pressed(UiKey::Single(KeyCode::KeyX), UiEvent::BlurEdgeDetect),
    UiKeybind::Pressed(UiKey::Single(KeyCode::KeyI), UiEvent::Isoline),
    #[cfg(feature = "export")]
    UiKeybind::Pressed(UiKey::Single(KeyCode::KeyW), UiEvent::ExportState),
];

pub fn poll_ui_keybinds(ui_state: &mut UiState, keys: Res<ButtonInput<KeyCode>>) {
    let mut consumed_keys = HashSet::new();
    for &keybind in KEYBINDS.iter() {
        match keybind {
            UiKeybind::Pressed(keybind, event) => match keybind {
                UiKey::Single(_) => (),
                UiKey::Double(key_codes) => {
                    if keys.just_pressed(key_codes.0)
                        && keys.just_pressed(key_codes.1)
                        && !consumed_keys.contains(&key_codes.1)
                    {
                        consumed_keys.insert(key_codes.1);
                        ui_state.ui_events.push(event);
                    }
                }
            },
            UiKeybind::Down(keybind, event) => match keybind {
                UiKey::Single(_) => (),
                UiKey::Double(key_codes) => {
                    if keys.pressed(key_codes.0)
                        && keys.pressed(key_codes.1)
                        && !consumed_keys.contains(&key_codes.1)
                    {
                        consumed_keys.insert(key_codes.1);
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
                    if keys.just_pressed(key_code) && !consumed_keys.contains(&key_code) {
                        consumed_keys.insert(key_code);
                        ui_state.ui_events.push(event);
                    }
                }
                UiKey::Double(_) => (),
            },
            UiKeybind::Down(keybind, event) => match keybind {
                UiKey::Single(key_code) => {
                    if keys.pressed(key_code) && !consumed_keys.contains(&key_code) {
                        consumed_keys.insert(key_code);
                        ui_state.ui_events.push(event);
                    }
                }
                UiKey::Double(_) => (),
            },
        }
    }
}
