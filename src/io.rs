use crate::visualize::app_state::AppState;
use crate::visualize::wrappers::HeightmapTexture;
use crate::State;
use std::cell::RefCell;
use std::rc::Rc;
use std::{fs, io};
use image::ImageError;
use image::imageops::FilterType;
use crate::heightmap::io::heightmap_to_image;

const STATE_FILE_EXT: &'static str = "ers";
const ICON_FILE_EXT: &'static str = "png";
const OUTPUT_DIRECTORY: &'static str = "saves";

#[derive(Debug)]
pub enum StateIoError {
    RWError(io::Error),
    InvalidBinary(bincode::Error),
    IconError(ImageError),
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

impl From<ImageError> for StateIoError {
    fn from(err: ImageError) -> Self {
        StateIoError::IconError(err)
    }
}

pub fn export_binary(state: &State, filename: &str) -> Result<(), StateIoError> {
    fs::create_dir_all(OUTPUT_DIRECTORY)?;
    let icon = heightmap_to_image(&state.app_state.simulation_state().get_heightmap());
    let icon = image::imageops::resize(&icon, 64, 64, FilterType::Nearest);
    icon.save(format!("{}/{}.{}", OUTPUT_DIRECTORY, filename, ICON_FILE_EXT))?;
    let result = bincode::serialize(state)?;
    fs::write(format!("{}/{}.{}", OUTPUT_DIRECTORY, filename, STATE_FILE_EXT), result)?;
    Ok(())
}

pub fn import_binary(file_path: &str) -> Result<State, StateIoError> {
    let data = fs::read(format!("{}/{}.{}", OUTPUT_DIRECTORY, file_path, STATE_FILE_EXT))?;
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
