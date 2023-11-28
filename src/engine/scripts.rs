use std::mem;
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use egui::{Pos2, Rect};
use crate::engine::{Engine, EngineError};
use crate::heightmap::HeightmapType;
use crate::partitioning::Method;
use crate::State;
use crate::visualize::events::{poll_ui_events, UiEvent};

pub type Script = Vec<Instruction>;

#[derive(Debug, Serialize, Deserialize)]
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
    Snapshot,
    Nop,
}

pub fn default() -> Script {
    use Instruction::*;
    vec![
        NewState(HeightmapType::XSinWave(5f32)),
        Print("Engine Started".to_string()),
        Queue(UiEvent::SelectMethod(Method::Subdivision(3))),
        Queue(UiEvent::RunSimulation),
        Snapshot,
        Nop,
        Render(false),
        Queue(UiEvent::SelectMethod(Method::GridOverlapBlend(8))),
        Queue(UiEvent::RunSimulation),
        Render(false),
        Flush,
        Print("Done eroding.".to_string()),
        Print("Handing over".to_string()),
        Handover,
        Print("Engine done.".to_string()),
    ]
}



fn poll(state: &mut State) {
    poll_ui_events(
        #[cfg(feature = "export")] &mut state.state_name,
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
    }.unwrap_or(Rect {
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

pub async fn tick(mut engine: Engine) -> Result<Engine, EngineError> {
    let state = &mut engine.state;
    let stack = &mut engine.stack;
    let result = if let Some(instruction) = engine.script.pop() {
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
            Instruction::Snapshot => {
                if let Some(()) = engine.snapshot() {
                    Ok(())
                } else {
                    Err(EngineError::MissingSnapshotData)
                }
            }
            Instruction::Nop => {
                Ok(())
            }
        }
    } else {
        return Err(EngineError::HasNoInstruction)
    };

    if let Some(err) = result.err() {
        Err(err)
    } else {
        Ok(engine)
    }
}

