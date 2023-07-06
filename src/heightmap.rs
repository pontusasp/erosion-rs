use ds_heightmap::Runner;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::math::{UVector2, Vector2};
pub mod io;

pub type HeightmapPrecision = f32;
pub type HeightmapData = Vec<Vec<HeightmapPrecision>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Heightmap {
    pub data: HeightmapData,
    pub width: usize,
    pub height: usize,
    pub depth: HeightmapPrecision,
    pub original_depth: HeightmapPrecision,
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartialHeightmap {
    pub anchor: UVector2,
    pub heightmap: Heightmap,
}

#[derive(Debug)]
pub enum HeightmapError {
    MismatchingSize,
    OutOfBounds,
}

impl Heightmap {
    pub fn new(
        data: HeightmapData,
        width: usize,
        height: usize,
        depth: HeightmapPrecision,
        original_depth: HeightmapPrecision,
    ) -> Heightmap {
        Heightmap {
            data,
            width,
            height,
            depth,
            original_depth,
            metadata: None,
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

    pub fn set_range(&mut self, min: HeightmapPrecision, max: HeightmapPrecision) {
        let (old_min, old_max) = self.get_range();
        let old_range = old_max - old_min;
        let new_range = max - min;
        for i in 0..self.width {
            for j in 0..self.height {
                let value = self.data[i][j];
                self.data[i][j] = ((value - old_min) / old_range) * new_range + min;
            }
        }
    }

    pub fn to_u8_rgba(&self) -> Vec<u8> {
        let mut buffer: Vec<u8> = Vec::new();
        let mut errors: Vec<i32> = Vec::new();

        for j in 0..self.height {
            for i in 0..self.width {
                let mut value = self.data[i][j];
                let u8_max: HeightmapPrecision = 255.0;
                value = value / (self.depth / u8_max);
                value = value.round();
                let value = value as i32;

                if let Some(value) = value.try_into().ok() {
                    buffer.push(value);
                    buffer.push(value);
                    buffer.push(value);
                } else {
                    errors.push(value);
                    buffer.push(if value < 0 { 0 } else { 255 });
                    buffer.push(if value < 0 { 0 } else { 255 });
                    buffer.push(if value < 0 { 0 } else { 255 });
                }
                buffer.push(255);
            }
        }
        if errors.len() > 0 {
            eprintln!(
                "heightmap.rs: Could not convert {} / {} ({:.5}%) values to u8 ({:?})",
                errors.len(),
                buffer.len(),
                errors.len() as f32 / buffer.len() as f32,
                errors
            );
        }

        buffer
    }

    pub fn to_u8(&self) -> Vec<u8> {
        let mut buffer: Vec<u8> = Vec::new();
        let mut errors: Vec<i32> = Vec::new();

        for j in 0..self.height {
            for i in 0..self.width {
                let mut value = self.data[i][j];
                let u8_max: HeightmapPrecision = 255.0;
                value = value / (self.depth / u8_max);
                value = value.round();
                let value = value as i32;

                if let Some(value) = value.try_into().ok() {
                    buffer.push(value);
                } else {
                    errors.push(value);
                    buffer.push(if value < 0 { 0 } else { 255 });
                }
            }
        }
        if errors.len() > 0 {
            eprintln!(
                "heightmap.rs: Could not convert {} / {} ({:.5}%) values to u8 ({:?})",
                errors.len(),
                buffer.len(),
                errors.len() as f32 / buffer.len() as f32,
                errors
            );
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
            return Err(HeightmapError::MismatchingSize);
        }

        for i in 0..self.width {
            let mut row = Vec::new();
            for j in 0..self.height {
                let value = (self.data[i][j] - heightmap.data[i][j]).abs();
                row.push(value);
            }
            data.push(row);
        }

        let diff = Heightmap::new(
            data,
            self.width,
            self.height,
            depth,
            heightmap.original_depth,
        );
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

        let dx = self.get_clamped(x as i32, y as i32) as HeightmapPrecision
            - self.get_clamped(x as i32 - 1, y as i32) as HeightmapPrecision;
        let dy = self.get_clamped(x as i32, y as i32) as HeightmapPrecision
            - self.get_clamped(x as i32, y as i32 - 1) as HeightmapPrecision;

        Some(Vector2::new(dx, dy))
    }

    pub fn interpolated_gradient(&self, position: &Vector2) -> Option<Vector2> {
        let (fx, fy) = position.to_tuple();

        let (x, y) = match position.to_usize() {
            Ok(t) => t,
            Err(_) => (0, 0), // TODO fix this!!
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
            Err(_) => (0, 0), // TODO fix this!!
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

    pub fn metadata_add(&mut self, key: &str, value: String) {
        if let Some(hashmap) = &mut self.metadata {
            hashmap.insert(key.to_string(), value);
        } else {
            let mut hashmap = HashMap::new();
            hashmap.insert(key.to_string(), value);
            self.metadata = Some(hashmap);
        }
    }
}

impl PartialHeightmap {
    pub fn from(heightmap: &Heightmap, anchor: &UVector2, size: &UVector2) -> Self {
        let mut data: Vec<Vec<HeightmapPrecision>> = vec![vec![0.0; size.y]; size.x];
        for x in 0..size.x {
            for y in 0..size.y {
                data[x][y] = heightmap.data[x + anchor.x][y + anchor.y];
            }
        }
        PartialHeightmap {
            anchor: anchor.clone(),
            heightmap: Heightmap {
                data,
                width: size.x,
                height: size.y,
                depth: heightmap.depth,
                original_depth: heightmap.original_depth,
                metadata: heightmap.metadata.clone(),
            }
        }
    }

    pub fn apply_to(&self, heightmap: &mut Heightmap) {
        for x in 0..self.heightmap.width {
            for y in 0..self.heightmap.height {
                heightmap.data[x + self.anchor.x][y + self.anchor.y] = self.heightmap.data[x][y];
            }
        }
    }
}

pub enum HeightmapPresets {
    // Flat,
    // Random,
    YGradient,
    InvertedYGradient,
    YHyperbolaGradient,
    CenteredHillGradient,
    CenteredHillSmallGradient,
}

pub fn create_heightmap_from_preset(preset: HeightmapPresets, size: usize) -> Heightmap {
    match preset {
        HeightmapPresets::YGradient => create_heightmap_from_closure(size, 1.0, &|_: usize, y: usize| y as HeightmapPrecision / size as HeightmapPrecision),
        HeightmapPresets::InvertedYGradient => create_heightmap_from_closure(size, 1.0, &|_: usize, y: usize| 1.0 - y as HeightmapPrecision / size as HeightmapPrecision),
        HeightmapPresets::YHyperbolaGradient => create_heightmap_from_closure(size, 1.0, &|_: usize, y: usize| {
            let gradient = y as HeightmapPrecision / size as HeightmapPrecision;
            gradient.powi(2)
        }),
        HeightmapPresets::CenteredHillGradient => create_heightmap_from_closure(size, 1.0, &|x: usize, y: usize| {
            let gradient = (x as HeightmapPrecision - size as HeightmapPrecision / 2.0)
                .powi(2) + (y as HeightmapPrecision - size as HeightmapPrecision / 2.0) .powi(2);
            1.0 - gradient / (size as HeightmapPrecision / 2.0).powi(2)
        }),
        HeightmapPresets::CenteredHillSmallGradient => create_heightmap_from_closure(size, 1.0, &|x: usize, y: usize| {
            let radius = size as HeightmapPrecision / 2.0;
            let x = x as HeightmapPrecision;
            let y = y as HeightmapPrecision;
            let distance = ((x - radius).powf(2.0) + (y - radius).powf(2.0)).sqrt();

            let hill_radius = 0.75;

            if distance < radius * hill_radius {
                let to = radius * hill_radius;
                let from = 0.0;
                let gradient = (distance - from) / (to - from);
                ((std::f32::consts::PI * gradient).cos() + 1.0) / 2.0
            } else {
                0.0
            }
        }),
    }
}

pub fn heightmap_to_image(heightmap: &Heightmap, filename: &str) -> image::ImageResult<()> {
    let buffer = heightmap.to_u8();

    // Save the buffer as filename on disk
    image::save_buffer(
        format!("{}.png", filename),
        &buffer as &[u8],
        heightmap.width.try_into().unwrap(),
        heightmap.height.try_into().unwrap(),
        image::ColorType::L8,
    )
}


pub fn create_heightmap(size: usize, original_depth: f32, roughness: f32) -> Heightmap {
    let mut runner = Runner::new();
    runner.set_height(size);
    runner.set_width(size);

    runner.set_depth(original_depth);
    runner.set_rough(roughness);

    let depth = 1.0;

    let output = runner.ds();
    Heightmap {
        data: output
            .data
            .into_iter()
            .map(|row| {
                row.into_iter()
                    .map(|value| value as HeightmapPrecision / original_depth)
                    .collect()
            })
            .collect(),
        width: size,
        height: size,
        depth,
        original_depth,
        metadata: None,
    }
}

pub fn create_heightmap_from_closure(
    size: usize,
    original_depth: f32,
    closure: &dyn Fn(usize, usize) -> HeightmapPrecision,
) -> Heightmap {
    let mut data: Vec<Vec<HeightmapPrecision>> = Vec::new();
    for i in 0..size {
        let mut row = Vec::new();
        for j in 0..size {
            row.push(closure(i, j));
        }
        data.push(row);
    }

    Heightmap {
        data,
        width: size,
        height: size,
        depth: 1.0,
        original_depth,
        metadata: None,
    }
}

pub fn export_heightmaps(heightmaps: Vec<&Heightmap>, filenames: Vec<&str>) {
    println!("Exporting heightmaps...");
    for (heightmap, filename) in heightmaps.iter().zip(filenames.iter()) {
        heightmap_to_image(heightmap, filename).unwrap();
        io::export(heightmap, filename).unwrap();
    }
}
