use crate::heightmap;
mod beyer;

pub fn erode(heightmap: &heightmap::Heightmap) -> heightmap::Heightmap {
    beyer::erode(&heightmap)
}
