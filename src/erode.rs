use crate::heightmap;
pub mod beyer;

pub fn erode(heightmap: &heightmap::Heightmap) -> heightmap::Heightmap {
    beyer::simulate(&heightmap)
}
