use crate::math::Vector2;

pub type HeightmapPrecision = f32;
pub type HeightmapData = Vec<Vec<HeightmapPrecision>>;

#[derive(Debug, Clone)]
pub struct Heightmap {
    pub data: HeightmapData,
    pub width: usize,
    pub height: usize,
    pub depth: HeightmapPrecision,
    pub original_depth: HeightmapPrecision
}

#[derive(Debug)]
pub enum HeightmapError {
    MismatchingSize,
    OutOfBounds
}

impl Heightmap {
    pub fn new(data: HeightmapData, width: usize, height: usize, depth: HeightmapPrecision, original_depth: HeightmapPrecision) -> Heightmap {
        Heightmap {
            data,
            width,
            height,
            depth,
            original_depth
        }
    }

    pub fn get_range(&self) -> (HeightmapPrecision, HeightmapPrecision) {
        let mut min = self.data[0][0];
        let mut max = self.data[0][0];
        for i in 0..self.width {
            for j in 0..self.height {
                let value = self.data[i][j];
                if value < min {
                    min = value;
                }
                if value > max {
                    max = value;
                }
            }
        }
        (min, max)
    }

    pub fn normalize(&mut self) {
        let (min, max) = self.get_range();
        let range = max - min;
        for i in 0..self.width {
            for j in 0..self.height {
                let value = self.data[i][j];
                self.data[i][j] = (value - min) / range;
            }
        }
    }

    pub fn to_u8(&self) -> Vec<u8> {
        let mut buffer: Vec<u8> = Vec::new();
        let mut errors: Vec<i32> = Vec::new();

        for i in 0..self.width {
            for j in 0..self.height {
                let mut value = self.data[i][j];
                let u8_max: HeightmapPrecision = 255.0;
                value = value / (self.depth / u8_max);
                value = value.round();
                let value = value as i32;
                
                if let Some(value) = value.try_into().ok() {
                    buffer.push(value);
                } else {
                    errors.push(value);
                    buffer.push(if value < 0 {
                            0
                        } else {
                            255
                        });
                }
            }
        }
        if errors.len() > 0 {
            eprintln!("heightmap.rs: Could not convert {} / {} ({:.5}%) values to u8 ({:?})", errors.len(), buffer.len(), errors.len() as f32 / buffer.len() as f32, errors);
        }

        buffer
    }

    pub fn subtract(&self, heightmap: &Heightmap) -> Result<Heightmap, HeightmapError> {
        let mut data: HeightmapData = Vec::new();
        
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

        let diff = Heightmap::new(data, self.width, self.height, depth, heightmap.original_depth);
        Ok(diff)
    }

    pub fn set(&mut self, x: usize, y: usize, z: HeightmapPrecision) -> Result<(), HeightmapError> {
        if x >= self.width || y >= self.height {
            Err(HeightmapError::OutOfBounds)
        } else {
            self.data[x][y] = z;
            Ok(())
        }
    }

    pub fn get(&self, x: usize, y: usize) -> Option<HeightmapPrecision> {
        if x >= self.width || y >= self.height {
            None
        } else {
            Some(self.data[x][y])
        }
    }

    pub fn get_clamped(&self, x: i32, y: i32) -> HeightmapPrecision {
        let mut x = x;
        let mut y = y;

        if x >= self.width as i32 {
            x = (self.width - 1) as i32;
        }

        if y >= self.height as i32 {
            y = (self.height - 1) as i32;
        }

        if x < 0 {
            x = 0;
        }

        if y < 0 {
            y = 0;
        }

        self.data[x as usize][y as usize]
    }

    pub fn gradient(&self, x: usize, y: usize) -> Option<Vector2> {
        // if x == 0 || y == 0 {
        //     return None
        // }

        let dx = self.get_clamped(x as i32, y as i32) as HeightmapPrecision - self.get_clamped(x as i32 - 1, y as i32) as HeightmapPrecision;
        let dy = self.get_clamped(x as i32, y as i32) as HeightmapPrecision - self.get_clamped(x as i32, y as i32 - 1) as HeightmapPrecision;
        
        Some(Vector2::new(dx, dy))
    }

    pub fn interpolated_gradient(&self, position: &Vector2) -> Option<Vector2> {
        let (fx, fy) = position.to_tuple();

        let (x, y) = match position.to_usize() {
            Ok(t) => t,
            Err(_) => (0, 0) // TODO fix this!!
            // Err(_) => return None TODO fix this!!
        };

        let frac_x = fx - fx.floor();
        let frac_y = fy - fy.floor();

        let tl = self.gradient(x + 0, y + 0)?;
        let tr = self.gradient(x + 1, y + 0)?;
        let bl = self.gradient(x + 0, y + 1)?;
        let br = self.gradient(x + 1, y + 1)?;
        
        let interpolate_l = tl.interpolate(&bl, frac_y);
        let interpolate_r = tr.interpolate(&br, frac_y);
        Some(interpolate_l.interpolate(&interpolate_r, frac_x))
    }

    pub fn interpolated_height(&self, position: &Vector2) -> Option<HeightmapPrecision> {
        let (fx, fy) = position.to_tuple();

        let (x, y) = match position.to_usize() {
            Ok(t) => t,
            Err(_) => (0, 0) // TODO fix this!!
            // Err(_) => return None TODO fix this!!
        };

        let frac_x = fx - fx.floor();
        let frac_y = fy - fy.floor();

        let tl = self.get_clamped(x as i32 + 0, y as i32 + 0);
        let tr = self.get_clamped(x as i32 + 1, y as i32 + 0);
        let bl = self.get_clamped(x as i32 + 0, y as i32 + 1);
        let br = self.get_clamped(x as i32 + 1, y as i32 + 1);
        
        let interpolate_l = (1.0 - frac_y) * tl + frac_y * bl;
        let interpolate_r = (1.0 - frac_y) * tr + frac_y * br;
        Some((1.0 - frac_x) * interpolate_l + frac_x * interpolate_r)
    }

}

