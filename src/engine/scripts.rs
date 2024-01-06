use crate::engine::{Engine, EngineError};
use crate::erode::Parameters;
use crate::heightmap::{HeightmapParameters, HeightmapType};
use crate::partitioning::Method;
use crate::visualize::events::{poll_ui_events, UiEvent};
use crate::visualize::ui::UiState;
use crate::State;
use egui::{Pos2, Rect};
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::mem;

pub type Function = Vec<Instruction>;
pub type FunctionName = String;
pub type Script = HashMap<FunctionName, Function>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SnapshotAction {
    Take,
    PrintAll,
    SaveAndClear(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum IsolineAction {
    Queue,
    SetValue(f32),
    SetError(f32),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Instruction {
    NewState(HeightmapType),
    PushState,
    PopSetState,
    Poll,
    Flush,
    Render(bool),
    Queue(UiEvent),
    WindowSize((f32, f32)),
    WindowAutoSize((f32, f32)),
    Handover,
    Print(String),
    Snapshot(SnapshotAction),
    Nop,
    Call(FunctionName),
    Isoline(IsolineAction),
    Size(usize),
    GridSize(usize),
    SetName(String),
    SetErosionParameters(Parameters),
    SetAdvancedView(bool),
}

pub fn default() -> Script {
    use Instruction::*;
    let mut script = HashMap::new();
    script.insert(
        "main".to_string(),
        vec![
            NewState(HeightmapType::XSinWave(
                HeightmapParameters::default(),
                5f32,
            )),
            Print("Engine Started".to_string()),
            Queue(UiEvent::SelectMethod(Method::Subdivision(3))),
            Queue(UiEvent::RunSimulation),
            Snapshot(SnapshotAction::Take),
            Nop,
            Render(false),
            Queue(UiEvent::SelectMethod(Method::GridOverlapBlend(8))),
            Queue(UiEvent::RunSimulation),
            Render(false),
            Flush,
            Print("Done eroding.".to_string()),
            Print("Handing over".to_string()),
            // Handover,
            Print("Engine done.".to_string()),
        ],
    );
    script
}

fn poll(state: &mut State) {
    poll_ui_events(
        #[cfg(feature = "export")]
        &mut state.state_name,
        &mut state.ui_state,
        &mut state.app_state,
    );
}

fn draw(state: &mut State, ui: bool) {
    clear_background(BLACK);
    let canvas_rect = if ui {
        state
            .ui_state
            .frame_slots
            .as_ref()
            .and_then(|slots| slots.canvas)
    } else {
        None
    }
    .unwrap_or(Rect {
        min: Pos2 { x: 0.0, y: 0.0 },
        max: Pos2 {
            x: screen_width(),
            y: screen_height(),
        },
    });
    crate::visualize::draw_frame(
        &canvas_rect,
        &state.app_state.simulation_state().get_active_texture(),
    );

    state.ui_state.frame_slots = if ui {
        crate::visualize::ui::ui_draw(state)
    } else {
        None
    };
}

pub fn call(mut engine: Engine, function_name: &FunctionName) -> Result<Engine, EngineError> {
    let mut function = if let Some(function) = engine.script.get(function_name) {
        function.clone()
    } else {
        return Err(EngineError::MissingFunction(function_name.to_string()));
    };
    engine.main.append(&mut function);
    Ok(engine)
}

pub async fn tick(mut engine: Engine) -> Result<Engine, EngineError> {
    let state = &mut engine.state;
    let stack = &mut engine.stack;
    let result = if let Some(instruction) = engine.main.pop() {
        match instruction {
            Instruction::NewState(map_type) => {
                let mut s = State::new(&map_type);
                mem::swap(&mut s, state);
                Ok(())
            }
            Instruction::PushState => {
                stack.push(state.clone());
                Ok(())
            }
            Instruction::PopSetState => {
                if let Some(mut s) = stack.pop() {
                    mem::swap(&mut s, state);
                    Ok(())
                } else {
                    Err(EngineError::HasNoState)
                }
            }
            Instruction::Poll => {
                poll(state);
                Ok(())
            }
            Instruction::Flush => {
                while !state.ui_state.ui_events.is_empty() {
                    poll(state);
                }
                Ok(())
            }
            Instruction::Render(ui) => {
                draw(state, ui);
                next_frame().await;
                Ok(())
            }
            Instruction::Queue(event) => {
                state.ui_state.ui_events.push(event);
                Ok(())
            }
            Instruction::WindowSize((w, h)) => {
                request_new_screen_size(w, h);
                Ok(())
            }
            Instruction::WindowAutoSize((w, h)) => {
                let canvas_rect = state
                    .ui_state
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

                let fit = canvas_rect.width().min(canvas_rect.height());
                request_new_screen_size(
                    w + canvas_rect.height() - fit,
                    h + canvas_rect.width() - fit,
                );
                Ok(())
            }
            Instruction::Handover => {
                while !state.ui_state.application_quit && !is_quit_requested() {
                    draw(state, true);
                    poll(state);
                    crate::visualize::keybinds::poll_ui_keybinds(&mut state.ui_state);
                    next_frame().await;
                }
                Ok(())
            }
            Instruction::Print(s) => {
                println!("{}", s);
                Ok(())
            }
            Instruction::Snapshot(action) => match action {
                SnapshotAction::Take => {
                    if let Some(()) = engine.snapshot() {
                        Ok(())
                    } else {
                        Err(EngineError::MissingSnapshotData)
                    }
                }
                SnapshotAction::PrintAll => {
                    println!("{:?}", engine.snapshots_to_string()?);
                    Ok(())
                }
                SnapshotAction::SaveAndClear(filename) => {
                    engine.export_snapshots(&filename)?;
                    engine.snapshots.clear();
                    Ok(())
                }
            },
            Instruction::Nop => Ok(()),
            Instruction::Call(ref function_name) => {
                engine = call(engine, function_name)?;
                Ok(())
            }
            Instruction::Isoline(action) => match action {
                IsolineAction::Queue => {
                    state.ui_state.ui_events.push(UiEvent::Isoline);
                    Ok(())
                }
                IsolineAction::SetValue(value) => {
                    state.ui_state.isoline.height = value;
                    Ok(())
                }
                IsolineAction::SetError(error) => {
                    state.ui_state.isoline.error = error;
                    Ok(())
                }
            },
            Instruction::Size(size) => {
                state.app_state.parameters.heightmap_type.params_mut().size = size;
                Ok(())
            }
            Instruction::GridSize(size) => {
                state.app_state.parameters.grid_size = size;
                Ok(())
            }
            Instruction::SetName(name) => {
                state.state_name = Some(name);
                Ok(())
            }
            Instruction::SetErosionParameters(params) => {
                state.app_state.parameters.erosion_params = params;
                Ok(())
            }
            Instruction::SetAdvancedView(mode) => {
                state.ui_state.isoline.advanced_texture = mode;
                Ok(())
            }
        }
    } else {
        return Err(EngineError::HasNoInstruction);
    };

    if let Some(err) = result.err() {
        Err(err)
    } else {
        Ok(engine)
    }
}
