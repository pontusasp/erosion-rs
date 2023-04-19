use ds_heightmap::Runner;

pub mod heightmap;
pub mod erode;

fn create_heightmap(size: usize, depth: f32, roughness: f32) -> heightmap::Heightmap {
    let mut runner = Runner::new();
    runner.set_height(size);
    runner.set_width(size);

    runner.set_depth(depth);
    runner.set_rough(roughness);
    
    let output = runner.ds();
    heightmap::Heightmap {
        data: output.data,
        width: size,
        height: size,
        depth
    }
}

fn heightmap_to_image(heightmap: &heightmap::Heightmap, filename: &str) -> image::ImageResult<()> {
    let buffer = heightmap.to_u8();

    // Save the buffer as "image.png"
    let image_result = image::save_buffer(filename, &buffer as &[u8], heightmap.width.try_into().unwrap(), heightmap.height.try_into().unwrap(), image::ColorType::L8);

    image_result
}


fn main() {

    let size: usize = 1024;
    let depth: f32 = 2000.0;
    let roughness: f32 = 1.0;

    let heightmap = create_heightmap(size, depth, roughness);
    let heightmap_eroded = heightmap.erode();
    let heightmap_diff = heightmap.subtract(&heightmap_eroded).unwrap();

    heightmap_to_image(&heightmap, "heightmap.png").unwrap();
    heightmap_to_image(&heightmap_eroded, "heightmap_eroded.png").unwrap();
    heightmap_to_image(&heightmap_diff, "heightmap_diff.png").unwrap();

}
