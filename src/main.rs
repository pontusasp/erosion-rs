use ds_heightmap::Runner;
use std::env;

pub mod heightmap;
pub mod erode;

fn create_heightmap(size: usize, original_depth: f32, roughness: f32) -> heightmap::Heightmap {
    let mut runner = Runner::new();
    runner.set_height(size);
    runner.set_width(size);

    runner.set_depth(original_depth);
    runner.set_rough(roughness);
    
    let depth = 1.0;
    
    let output = runner.ds();
    heightmap::Heightmap {
        data: output.data.into_iter().map(|row| row.into_iter().map(|value| value as heightmap::HeightmapPrecision / original_depth).collect()).collect(),
        width: size,
        height: size,
        depth,
        original_depth
    }
}

fn heightmap_to_image(heightmap: &heightmap::Heightmap, filename: &str) -> image::ImageResult<()> {
    let buffer = heightmap.to_u8();

    // Save the buffer as "image.png"
    let image_result = image::save_buffer(filename, &buffer as &[u8], heightmap.width.try_into().unwrap(), heightmap.height.try_into().unwrap(), image::ColorType::L8);

    image_result
}


fn main() {
    
    env::set_var("RUST_BACKTRACE", "1");

    let size: usize = 1024;
    let depth: f32 = 2000.0;
    let roughness: f32 = 1.0;

    let heightmap = create_heightmap(size, depth, roughness);
    let heightmap_eroded = erode::erode(&heightmap);
    let heightmap_diff = heightmap.subtract(&heightmap_eroded).unwrap();

    heightmap_to_image(&heightmap, "heightmap.png").unwrap();
    heightmap_to_image(&heightmap_eroded, "heightmap_eroded.png").unwrap();
    heightmap_to_image(&heightmap_diff, "heightmap_diff.png").unwrap();

}
