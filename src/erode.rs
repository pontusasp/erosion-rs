use crate::heightmap;
use bracket_noise::prelude::*;

pub mod beyer;
pub mod lague;

pub fn erode(heightmap: &heightmap::Heightmap) -> heightmap::Heightmap {
    beyer::simulate(&heightmap)
}

pub fn run_simulation() {
    // Normalize to get the most accuracy out of the png later since heightmap might not utilize full range of 0.0 to 1.0
    let heightmap = initialize_heightmap().normalize();

    let heightmap_eroded = erode(&heightmap);
    let heightmap_diff = heightmap.subtract(&heightmap_eroded).unwrap();

    heightmap::export_heightmaps(
        vec![&heightmap, &heightmap_eroded, &heightmap_diff],
        vec![
            "output/heightmap",
            "output/heightmap_eroded",
            "output/heightmap_diff",
        ],
    );

    println!("Done!");
}

pub fn initialize_heightmap() -> heightmap::Heightmap {
    let size: usize = 512;
    let depth: f32 = 2000.0;
    let roughness: f32 = 1.0;

    let debug = false;

    if debug {
        heightmap::create_heightmap_from_preset(
            heightmap::HeightmapPresets::CenteredHillSmallGradient,
            size,
        )
    } else {
        heightmap::create_perlin_heightmap(&heightmap::HeightmapSettings {
            noise_type: NoiseType::PerlinFractal,
            fractal_type: FractalType::FBM,
            fractal_octaves: 5,
            fractal_gain: 0.6,
            fractal_lacunarity: 2.0,
            frequency: 2.0,
            width: 512,
            height: 512,
        }, &1337)
        // heightmap::create_heightmap(size, depth, roughness)
    }
}
