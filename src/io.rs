use crate::visualize::app_state::AppState;
use crate::visualize::wrappers::HeightmapTexture;
use crate::State;
use std::cell::RefCell;
use std::rc::Rc;
use std::{fs, io};

#[derive(Debug)]
pub enum StateIoError {
    RWError(io::Error),
    InvalidBinary(bincode::Error),
}

impl From<io::Error> for StateIoError {
    fn from(err: io::Error) -> Self {
        StateIoError::RWError(err)
    }
}

impl From<bincode::Error> for StateIoError {
    fn from(err: bincode::Error) -> Self {
        StateIoError::InvalidBinary(err)
    }
}

pub fn export_binary(app_state: &State, file_path: &str) -> Result<(), StateIoError> {
    let result = bincode::serialize(app_state)?;
    fs::write(file_path, result)?;
    Ok(())
}

pub fn import_binary(file_path: &str) -> Result<State, StateIoError> {
    let data = fs::read(file_path)?;
    let mut result: State = bincode::deserialize(&data)?;
    repair_states(&mut result.app_state);
    Ok(result)
}

fn repair_states(app_state: &mut AppState) {
    for ref mut state in &mut app_state.simulation_states {
        let active_hm = &state.base().heightmap_active.heightmap;
        let active = HeightmapTexture::from(active_hm);
        state.base_mut().heightmap_active = Rc::new(active);

        let base_hm = &state.base().heightmap_base.heightmap;
        let base = HeightmapTexture::from(base_hm);
        state.base_mut().heightmap_base = Rc::new(base);

        if let Some(eroded_state) = state.eroded_mut() {
            let eroded_hm = &eroded_state.heightmap_eroded.heightmap;
            let eroded = HeightmapTexture::from(eroded_hm);
            eroded_state.heightmap_eroded = Rc::new(eroded);

            let mut diffs = Vec::new();
            for diff in eroded_state.heightmap_difference.borrow().iter() {
                let diff_fixed = HeightmapTexture::from(&diff.heightmap);
                diffs.push(Rc::new(diff_fixed));
            }
            eroded_state.heightmap_difference = Rc::new(RefCell::new(diffs));

            let mut diffs = Vec::new();
            for diff in eroded_state.heightmap_difference_normalized.borrow().iter() {
                let diff_fixed = HeightmapTexture::from(&diff.heightmap);
                diffs.push(Rc::new(diff_fixed));
            }
            eroded_state.heightmap_difference_normalized = Rc::new(RefCell::new(diffs));
        }
    }
}
