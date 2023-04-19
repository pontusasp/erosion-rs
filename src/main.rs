use ds_heightmap::Runner;

#[derive(Debug)]
struct Heightmap {
    data: Vec<Vec<f32>>,
    width: usize,
    height: usize,
    depth: f32
}

#[derive(Debug)]
enum HeightmapError {
    MismatchingSize
}

fn create_heightmap(size: usize, depth: f32, roughness: f32) -> Heightmap {
    let mut runner = Runner::new();
    runner.set_height(size);
    runner.set_width(size);

    runner.set_depth(depth);
    runner.set_rough(roughness);
    
    let output = runner.ds();
    Heightmap {
        data: output.data,
        width: size,
        height: size,
        depth
    }
}

impl Heightmap {
    fn new(data: Vec<Vec<f32>>, width: usize, height: usize, depth: f32) -> Heightmap {
        Heightmap {
            data,
            width,
            height,
            depth
        }
    }

    fn erode(&self) -> Heightmap {
        let data = self.data.clone();
        let width = self.width;
        let height = self.height;
        let depth = self.depth;
        Heightmap::new(data, width, height, depth)
    }

    fn to_u8(&self) -> Vec<u8> {
        let mut buffer: Vec<u8> = Vec::new();

        for i in 0..self.width {
            for j in 0..self.height {
                let mut value = self.data[i][j];
                let u8_max: f32 = 255.0;
                value = value / (self.depth / u8_max);
                value = value.round();
                let value = value as i32;
                
                buffer.push(value.try_into().unwrap());
            }
        }

        buffer
    }

    fn subtract(&self, heightmap: &Heightmap) -> Result<Heightmap, HeightmapError> {
        let mut data: Vec<Vec<f32>> = Vec::new();
        
        if !(self.width == heightmap.width && self.height == heightmap.height) {
            return Err(HeightmapError::MismatchingSize)
        }

        for i in 0..self.width {
            let mut row = Vec::new();
            for j in 0..self.height {
                let value = self.data[i][j] - heightmap.data[i][j] + heightmap.depth;
                row.push(value);
            }
            data.push(row);
        }

        let diff = Heightmap::new(data, self.width, self.height, self.depth + heightmap.depth);
        Ok(diff)
    }

}

fn heightmap_to_image(heightmap: &Heightmap, filename: &str) -> image::ImageResult<()> {
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
