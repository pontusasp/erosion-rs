use crate::heightmap;

pub mod lague;

pub fn initialize_heightmap(settings: Option<&heightmap::HeightmapSettings>) -> heightmap::Heightmap {
    let size: usize = 512;

    let debug = false;

    if debug {
        heightmap::create_heightmap_from_preset(
            heightmap::HeightmapPresets::CenteredHillSmallGradient,
            size,
        )
    } else {
        heightmap::create_perlin_heightmap(&settings.unwrap_or(&heightmap::HeightmapSettings::default()))
        // let depth: f32 = 2000.0;
        // let roughness: f32 = 1.0;
        // heightmap::create_heightmap(size, depth, roughness)
    }
}
