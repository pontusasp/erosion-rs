
#[derive(Debug)]
pub struct Heightmap {
    pub data: Vec<Vec<f32>>,
    pub width: usize,
    pub height: usize,
    pub depth: f32
}

#[derive(Debug)]
pub enum HeightmapError {
    MismatchingSize
}

impl Heightmap {
    pub fn new(data: Vec<Vec<f32>>, width: usize, height: usize, depth: f32) -> Heightmap {
        Heightmap {
            data,
            width,
            height,
            depth
        }
    }

    pub fn to_u8(&self) -> Vec<u8> {
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

    pub fn subtract(&self, heightmap: &Heightmap) -> Result<Heightmap, HeightmapError> {
        let mut data: Vec<Vec<f32>> = Vec::new();
        
        let depth = if self.depth > heightmap.depth {
            self.depth
        } else {
            heightmap.depth
        };
        
        if !(self.width == heightmap.width && self.height == heightmap.height) {
            return Err(HeightmapError::MismatchingSize)
        }

        for i in 0..self.width {
            let mut row = Vec::new();
            for j in 0..self.height {
                let value = (self.data[i][j] - heightmap.data[i][j]).abs();
                row.push(value);
            }
            data.push(row);
        }

        let diff = Heightmap::new(data, self.width, self.height, depth);
        Ok(diff)
    }

}

