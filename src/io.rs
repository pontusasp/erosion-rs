use crate::heightmap::io::heightmap_to_image;
use crate::visualize::app_state::AppState;
use crate::visualize::ui::UiState;
use crate::visualize::wrappers::HeightmapTexture;
use crate::State;
use image::imageops::FilterType;
use image::ImageError;
use std::cell::RefCell;
use std::rc::Rc;
use std::{fs, io};

const STATE_FILE_EXT: &'static str = "ers";
const ICON_FILE_EXT: &'static str = "png";
const OUTPUT_DIRECTORY: &'static str = "saves";
pub const DEFAULT_NAME: &'static str = "Unnamed";

#[derive(Debug)]
pub enum StateIoError {
    RWError(io::Error),
    InvalidBinary(bincode::Error),
    InvalidJson(serde_json::Error),
    IconError(ImageError),
}

impl From<io::Error> for StateIoError {
    fn from(err: io::Error) -> Self {
        StateIoError::RWError(err)
    }
}

impl From<serde_json::Error> for StateIoError {
    fn from(err: serde_json::Error) -> Self {
        StateIoError::InvalidJson(err)
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

pub fn export_icon(state: &State, filename: &str) -> Result<(), StateIoError> {
    fs::create_dir_all(OUTPUT_DIRECTORY)?;
    let icon = heightmap_to_image(&state.app_state.simulation_state().get_heightmap());
    let icon = image::imageops::resize(&icon, 64, 64, FilterType::Nearest);
    icon.save(format!(
        "{}/{}.{}",
        OUTPUT_DIRECTORY, filename, ICON_FILE_EXT
    ))?;
    Ok(())
}

pub fn export_json(state: &State, filename: &str) -> Result<(), StateIoError> {
    fs::create_dir_all(OUTPUT_DIRECTORY)?;
    let result = serde_json::to_string(state)?;
    fs::write(
        format!("{}/{}.{}.json", OUTPUT_DIRECTORY, filename, STATE_FILE_EXT),
        result,
    )?;
    Ok(())
}

pub fn export_binary(state: &State, filename: &str) -> Result<(), StateIoError> {
    fs::create_dir_all(OUTPUT_DIRECTORY)?;
    let result = bincode::serialize(state)?;
    fs::write(
        format!("{}/{}.{}", OUTPUT_DIRECTORY, filename, STATE_FILE_EXT),
        result,
    )?;
    Ok(())
}

pub fn import(file_name: &str) -> Result<State, StateIoError> {
    let binary_result = import_binary(file_name);
    let result = if let Err(_) = binary_result {
        import_json(file_name)
    } else {
        binary_result
    };
    result
}

pub fn import_json(file_name: &str) -> Result<State, StateIoError> {
    let data = fs::read_to_string(format!(
        "{}/{}.{}.json",
        OUTPUT_DIRECTORY, file_name, STATE_FILE_EXT
    ))?;
    let mut result: State = serde_json::from_str(&data)?;
    repair_app_state(&mut result.app_state);
    repair_ui_state(&mut result.ui_state);
    Ok(result)
}

pub fn import_binary(file_name: &str) -> Result<State, StateIoError> {
    let data = fs::read(format!(
        "{}/{}.{}",
        OUTPUT_DIRECTORY, file_name, STATE_FILE_EXT
    ))?;
    let mut result: State = bincode::deserialize(&data)?;
    repair_app_state(&mut result.app_state);
    repair_ui_state(&mut result.ui_state);
    Ok(result)
}

fn repair_ui_state(ui_state: &mut UiState) {
    ui_state.saves = list_state_files().expect("Failed to access saved states.");
}

fn repair_app_state(app_state: &mut AppState) {
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

pub type StateFile = (String, Option<String>);

pub fn list_state_files() -> Result<Vec<StateFile>, StateIoError> {
    list_state_files_custom_path(OUTPUT_DIRECTORY)
}

pub fn list_state_files_custom_path(path: &str) -> Result<Vec<StateFile>, StateIoError> {
    let mut files = Vec::new();
    let paths = fs::read_dir(path)?;

    let json_extension = format!(".{}.json", STATE_FILE_EXT);
    let extension = format!(".{}", STATE_FILE_EXT);
    for path_result in paths {
        if let Ok(path) = path_result {
            let is_file = path
                .file_type()
                .and_then(|file| Ok(file.is_file()))
                .unwrap_or(false);
            let file_name = path
                .file_name()
                .into_string()
                .expect("Can't read filename! Are there any special characters in it?");
            let is_state_file =
                file_name.ends_with(&extension) || file_name.ends_with(&json_extension);
            if is_file && is_state_file {
                files.push(
                    file_name
                        .strip_suffix(&json_extension)
                        .or_else(|| file_name.strip_suffix(&extension))
                        .expect("Failed to process file name.")
                        .to_string(),
                )
            }
        }
    }

    let icon_extension = format!(".{}", ICON_FILE_EXT);
    let list = files
        .iter()
        .map(|state_name| {
            let mut state_icon_name = state_name.clone();
            state_icon_name.push_str(&icon_extension);

            let icon = if fs::metadata(&state_icon_name).is_ok() {
                Some(state_icon_name)
            } else {
                None
            };

            (state_name.to_string(), icon)
        })
        .collect();

    Ok(list)
}
