use std::mem;
use crate::State;

pub enum Instruction {
    SetState(State),
    PushState,
    PopSetState,
}

pub type Script = Vec<Instruction>;
pub type Stack = Vec<State>;

pub enum EngineError {
    NoInitialState,
    HasNoState,
}

fn tick(state: &mut State, instruction: Instruction, stack: &mut Stack) -> Result<(), EngineError> {
    match instruction {
        Instruction::SetState(mut s) => {
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
    }
}

pub fn launch(mut script: Script) -> Result<State, EngineError> {
    script.reverse();
    let mut stack: Stack = Vec::new();
    let mut state = if let Some(Instruction::SetState(s)) = script.pop() {
        s
    } else {
        return Err(EngineError::NoInitialState)
    };

    while !script.is_empty() {
        let instruction = script.pop().unwrap();
        tick(&mut state, instruction, &mut stack)?;
    }

    Ok(state)
}