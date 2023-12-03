pub mod scripts;

use crate::engine::scripts::{tick, Function, Instruction, Script};
use crate::erode::Parameters;
use crate::heightmap::HeightmapType;
use crate::State;
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug)]
pub enum EngineError {
    HasNoState,
    HasNoInstruction,
    MissingSnapshotData,
    JsonError(serde_json::Error),
    MissingMainFunction,
    MissingFunction(String),
    RWError(std::io::Error),
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
    pub main: Function,
    pub script: Script,
    pub stack: Stack,
    pub snapshots: Vec<Snapshot>,
}

impl Engine {
    pub fn ready(&self) -> bool {
        !self.main.is_empty()
    }

    pub fn snapshot(&mut self) -> Option<()> {
        let tuning = Tuning {
            parameters: self.state.app_state.parameters.erosion_params,
            map_type: self.state.app_state.parameters.heightmap_type,
            flatness: self
                .state
                .app_state
                .simulation_state()
                .get_heightmap()
                .get_average_height()?,
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

    pub fn snapshots_to_string(&self) -> Result<String, EngineError> {
        Ok(serde_json::to_string(&self.snapshots)?)
    }

    pub fn export_snapshots(&self, filename: &str) -> Result<(), EngineError> {
        fs::write(filename, self.snapshots_to_string()?)?;
        Ok(())
    }
}

pub async fn launch(mut script: Script) -> Result<Engine, EngineError> {
    prevent_quit();
    for (_, fun) in script.iter_mut() {
        fun.reverse()
    }
    let stack: Stack = Vec::new();
    let snapshots: Vec<Snapshot> = Vec::new();
    let mut main = script
        .remove("main")
        .ok_or(EngineError::MissingMainFunction)?;
    let state = if let Some(instruction) = main.pop() {
        match instruction {
            Instruction::NewState(map_type) => State::new(&map_type),
            _ => return Err(EngineError::HasNoState),
        }
    } else {
        return Err(EngineError::HasNoState);
    };

    let mut engine = Engine {
        state,
        main,
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

impl From<serde_json::Error> for EngineError {
    fn from(err: serde_json::Error) -> Self {
        EngineError::JsonError(err)
    }
}

impl From<std::io::Error> for EngineError {
    fn from(err: std::io::Error) -> Self {
        EngineError::RWError(err)
    }
}
