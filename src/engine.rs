pub mod scripts;

use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use crate::engine::scripts::{Instruction, Script, tick};
use crate::erode::Parameters;
use crate::heightmap::HeightmapType;
use crate::State;

#[derive(Debug)]
pub enum EngineError {
    HasNoState,
    HasNoInstruction,
    MissingSnapshotData,
}

pub type Stack = Vec<State>;

#[derive(Debug, Serialize, Deserialize)]
pub struct Tuning {
    pub parameters: Parameters,
    pub map_type: HeightmapType,
    pub flatness: f32,
    pub grid_size: usize,
    pub isoline_value: f32,
    pub isoline_error: f32,
}

type TimeStart = f32;
type TimeEnd = f32;
type Flooded = usize;
type Unflooded = usize;

#[derive(Debug, Serialize, Deserialize)]
pub enum Measurement {
    Time(Option<TimeStart>, Option<TimeEnd>),
    LowAreas(Flooded, Unflooded),
    HighAreas(Flooded, Unflooded),
}

pub type Snapshot = (Tuning, Vec<Measurement>);

pub struct Engine {
    pub state: State,
    pub script: Script,
    pub stack: Stack,
    pub snapshots: Vec<Snapshot>,
}

impl Engine {
    pub fn ready(&self) -> bool {
        !self.script.is_empty()
    }

    pub fn snapshot(&mut self) -> Option<()> {
        let tuning = Tuning {
            parameters: self.state.app_state.parameters.erosion_params,
            map_type: self.state.app_state.parameters.heightmap_type,
            flatness: self.state.app_state.simulation_state().get_heightmap().get_average_height()?,
            grid_size: self.state.app_state.parameters.grid_size,
            isoline_value: self.state.ui_state.isoline.height,
            isoline_error: self.state.ui_state.isoline.error,
        };
        let flood_a = self.state.ui_state.isoline.flooded_areas_lower;
        let flood_b = self.state.ui_state.isoline.flooded_areas_higher;
        let flooded_areas = if self.state.ui_state.isoline.flood_lower {
            Measurement::LowAreas(flood_a?, flood_b?)
        } else {
            Measurement::HighAreas(flood_b?, flood_a?)
        };
        let measurements = vec![flooded_areas];
        let snapshot: Snapshot = (tuning, measurements);
        self.snapshots.push(snapshot);
        Some(())
    }
}

pub async fn launch(mut script: Script) -> Result<Engine, EngineError> {
    prevent_quit();
    script.reverse();
    let stack: Stack = Vec::new();
    let snapshots: Vec<Snapshot> = Vec::new();
    let state = if let Some(instruction) = script.pop() {
        match instruction {
            Instruction::NewState(map_type) => State::new(&map_type),
            _ => return Err(EngineError::HasNoState)
        }
    } else {
        return Err(EngineError::HasNoState)
    };

    let mut engine = Engine {
        state,
        script,
        stack,
        snapshots,
    };

    engine = turn(engine).await?;
    Ok(engine)
}

pub async fn turn(mut engine: Engine) -> Result<Engine, EngineError> {
    while engine.ready() {
        engine = tick(engine).await?;
    }
    Ok(engine)
}
