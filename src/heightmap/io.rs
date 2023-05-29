use crate::heightmap::*;
use std::fs::File;
use std::io::prelude::*;

#[derive(Debug)]
pub enum HeightmapIOError {
    FileExportError,
    FileImportError,
}

pub fn export(heightmap: &Heightmap, filename: &str) -> Result<(), HeightmapIOError> {
    fn _export(heightmap: &Heightmap, filename: &str) -> std::io::Result<()> {
        let data = serde_json::to_string(&heightmap).unwrap();
        let mut file = File::create(filename)?;
        file.write_all(data.as_bytes())?;
        Ok(())
    }

    match _export(heightmap, filename) {
        Ok(_) => Ok(()),
        Err(_) => Err(HeightmapIOError::FileExportError),
    }
}

pub fn import(filename: &str) -> Result<Heightmap, HeightmapIOError> {
    fn _import(filename: &str) -> std::io::Result<Heightmap> {
        let mut data = String::new();
        {
            let mut file = File::open(filename)?;
            file.read_to_string(&mut data)?;
        }

        let heightmap: Heightmap = serde_json::from_str(&data).unwrap();

        Ok(heightmap)
    }
    match _import(filename) {
        Ok(heightmap) => Ok(heightmap),
        Err(_) => Err(HeightmapIOError::FileImportError),
    }
}
