use bracket_noise::prelude::*;
use rayon::iter::IntoParallelRefMutIterator;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::math::{UVector2, Vector2};

use image::*;

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
    pub total_height: Option<HeightmapPrecision>,
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
        metadata: Option<HashMap<String, String>>,
    ) -> Heightmap {
        Heightmap {
            data,
            width,
            height,
            depth,
            original_depth,
            metadata,
            total_height: None,
        }
    }

    pub fn from_u8(data: &Vec<u8>, width: usize, height: usize) -> Self {
        let mut data_f32 = vec![vec![0.0; height]; width];
        let data: Vec<&[u8]> = data.chunks(height).collect();

        data_f32.par_iter_mut().enumerate().for_each(|(i, col)| {
            for j in 0..height {
                let value: HeightmapPrecision = data[j][i] as f32 / 255.0;
                col[j] = value;
            }
        });

        Heightmap::new(data_f32, width, height, 1.0, 1.0, None)
    }

    fn get_gray_image(&self) -> Option<GrayImage> {
        let width = self.width.try_into().ok();
        let height = self.height.try_into().ok();
        ImageBuffer::from_vec(width?, height?, self.to_u8())
    }

    pub fn blur(&self, sigma: f32) -> Option<Heightmap> {
        let gray_image: Option<GrayImage> = self.get_gray_image();
        let blurred_gray_image = imageops::blur(&gray_image?, sigma);

        let blurred_heightmap =
            Heightmap::from_u8(blurred_gray_image.as_raw(), self.width, self.height);
        Some(blurred_heightmap)
    }

    pub fn canny_edge(&self, low: f32, high: f32) -> Option<Heightmap> {
        let gray_image: Option<GrayImage> = self.get_gray_image();
        let canny_edge_image = imageproc::edges::canny(&gray_image?, low, high);

        Some(Heightmap::from_u8(
            canny_edge_image.as_raw(),
            self.width,
            self.height,
        ))
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

    pub fn normalize(mut self) -> Self {
        let (min, max) = self.get_range();
        let range = max - min;
        for i in 0..self.width {
            for j in 0..self.height {
                let value = self.data[i][j];
                self.data[i][j] = (value - min) / range;
            }
        }
        self.depth = 1.0;
        self
    }

    pub fn calculate_total_height(&mut self) -> HeightmapPrecision {
        if let Some(height) = self.total_height {
            height
        } else {
            let height =
                self.data
                    .iter()
                    .fold(0.0, |accumulator: f32, col: &Vec<HeightmapPrecision>| {
                        accumulator
                            + col
                                .iter()
                                .fold(0.0, |accumulator: f32, value: &HeightmapPrecision| {
                                    accumulator + value
                                })
                    });
            self.total_height = Some(height);
            height
        }
    }

    pub fn calculate_average_height(&mut self) -> HeightmapPrecision {
        self.calculate_total_height() / (self.width * self.height) as f32
    }

    pub fn get_average_height(&self) -> Option<HeightmapPrecision> {
        Some(self.total_height? / (self.width * self.height) as f32)
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
        if errors.len() > 0 && errors.len() < 256 {
            eprintln!(
                "heightmap.rs: Could not convert {} / {} ({:.5}%) values to u8 ({:?})",
                errors.len(),
                buffer.len(),
                errors.len() as f32 / buffer.len() as f32,
                errors
            );
        } else if errors.len() > 0 {
            eprintln!(
                "heightmap.rs: Could not convert {} / {} ({:.5}%) values to u8.)",
                errors.len(),
                buffer.len(),
                errors.len() as f32 / buffer.len() as f32
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
        if errors.len() > 0 && errors.len() < 256 {
            eprintln!(
                "heightmap.rs: Could not convert {} / {} ({:.5}%) values to u8 ({:?})",
                errors.len(),
                buffer.len(),
                errors.len() as f32 / buffer.len() as f32,
                errors
            );
        } else if errors.len() > 0 {
            eprintln!(
                "heightmap.rs: Could not convert {} / {} ({:.5}%) values to u8.)",
                errors.len(),
                buffer.len(),
                errors.len() as f32 / buffer.len() as f32
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
            None,
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

    pub fn overlay(&mut self, overlay: &Self, mask: &Self) -> Result<(), HeightmapError> {
        if self.width != overlay.width
            || self.height != overlay.height
            || self.width != mask.width
            || self.height != mask.height
        {
            return Err(HeightmapError::MismatchingSize);
        }
        for x in 0..self.width {
            for y in 0..self.height {
                let v0 = self.data[x][y];
                let v1 = overlay.data[x][y];
                let m = mask.data[x][y];
                self.data[x][y] = v1 * m + v0 * (1.0 - m);
            }
        }
        Ok(())
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
            heightmap: Heightmap::new(
                data,
                size.x,
                size.y,
                heightmap.depth,
                heightmap.original_depth,
                heightmap.metadata.clone(),
            ),
        }
    }

    pub fn nest(&self, anchor: &UVector2, size: &UVector2) -> Self {
        let mut data: Vec<Vec<HeightmapPrecision>> = vec![vec![0.0; size.y]; size.x];
        for x in 0..size.x {
            for y in 0..size.y {
                data[x][y] = self.heightmap.data[x + anchor.x][y + anchor.y];
            }
        }
        PartialHeightmap {
            anchor: self.anchor + *anchor,
            heightmap: Heightmap::new(
                data,
                size.x,
                size.y,
                self.heightmap.depth,
                self.heightmap.original_depth,
                self.heightmap.metadata.clone(),
            ),
        }
    }

    pub fn apply_to(&self, heightmap: &mut Heightmap) {
        for x in 0..self.heightmap.width {
            for y in 0..self.heightmap.height {
                heightmap.data[x + self.anchor.x][y + self.anchor.y] = self.heightmap.data[x][y];
            }
        }
    }

    pub fn blend_apply_to(&self, other: &mut PartialHeightmap) {
        let rect_min = UVector2::new(
            self.anchor.x.max(other.anchor.x),
            self.anchor.y.max(other.anchor.y),
        );
        let rect_max = UVector2::new(
            (self.anchor.x + self.heightmap.width).min(other.anchor.x + other.heightmap.width),
            (self.anchor.y + self.heightmap.height).min(other.anchor.y + other.heightmap.height),
        );

        for x in 0..(rect_max.x - rect_min.x) {
            for y in 0..(rect_max.y - rect_min.y) {
                let sx = x + rect_min.x - self.anchor.x;
                let sy = y + rect_min.y - self.anchor.y;
                let ox = x + rect_min.x - other.anchor.x;
                let oy = y + rect_min.y - other.anchor.y;

                let h1 = self.heightmap.data[sx][sy];
                let h2 = other.heightmap.data[ox][oy];
                let min = -1.0;
                let max = 1.0;
                let lerp_x = min
                    + (max - min)
                        * (ox as HeightmapPrecision / other.heightmap.width as HeightmapPrecision);
                let factor_x = lerp_x.abs();
                let lerp_y = min
                    + (max - min)
                        * (oy as HeightmapPrecision / other.heightmap.height as HeightmapPrecision);
                let factor_y = lerp_y.abs();
                let factor = (1.0 - factor_x * factor_y).powf(6.5);
                let height = h2 * factor + h1 * (1.0 - factor);

                other.heightmap.data[ox][oy] = height;
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
        HeightmapPresets::YGradient => {
            create_heightmap_from_closure(size, 1.0, &|_: usize, y: usize| {
                y as HeightmapPrecision / size as HeightmapPrecision
            })
        }
        HeightmapPresets::InvertedYGradient => {
            create_heightmap_from_closure(size, 1.0, &|_: usize, y: usize| {
                1.0 - y as HeightmapPrecision / size as HeightmapPrecision
            })
        }
        HeightmapPresets::YHyperbolaGradient => {
            create_heightmap_from_closure(size, 1.0, &|_: usize, y: usize| {
                let gradient = y as HeightmapPrecision / size as HeightmapPrecision;
                gradient.powi(2)
            })
        }
        HeightmapPresets::CenteredHillGradient => {
            create_heightmap_from_closure(size, 1.0, &|x: usize, y: usize| {
                let gradient = (x as HeightmapPrecision - size as HeightmapPrecision / 2.0).powi(2)
                    + (y as HeightmapPrecision - size as HeightmapPrecision / 2.0).powi(2);
                1.0 - gradient / (size as HeightmapPrecision / 2.0).powi(2)
            })
        }
        HeightmapPresets::CenteredHillSmallGradient => {
            create_heightmap_from_closure(size, 1.0, &|x: usize, y: usize| {
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
            })
        }
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

    Heightmap::new(data, size, size, 1.0, original_depth, None)
}

pub struct HeightmapSettings {
    pub seed: u64,
    pub noise_type: NoiseType,
    pub fractal_type: FractalType,
    pub fractal_octaves: i32,
    pub fractal_gain: f32,
    pub fractal_lacunarity: f32,
    pub frequency: f32,
    pub width: usize,
    pub height: usize,
}

impl Default for HeightmapSettings {
    fn default() -> Self {
        HeightmapSettings {
            seed: 1337,
            noise_type: NoiseType::PerlinFractal,
            fractal_type: FractalType::FBM,
            fractal_octaves: 5,
            fractal_gain: 0.6,
            fractal_lacunarity: 2.0,
            frequency: 2.0,
            width: 512,
            height: 512,
        }
    }
}

pub fn create_perlin_heightmap(settings: &HeightmapSettings) -> Heightmap {
    let mut noise = FastNoise::seeded(settings.seed);
    noise.set_noise_type(settings.noise_type);
    noise.set_fractal_type(settings.fractal_type);
    noise.set_fractal_octaves(settings.fractal_octaves);
    noise.set_fractal_gain(settings.fractal_gain);
    noise.set_fractal_lacunarity(settings.fractal_lacunarity);
    noise.set_frequency(settings.frequency);

    let denominator = 100.0;

    let mut data: HeightmapData = Vec::new();

    let mut min = noise.get_noise(0.0, 0.0);
    let mut max = min.clone();

    for x in 0..settings.width {
        data.push(vec![]);
        for y in 0..settings.height {
            let n = noise.get_noise(x as f32 / denominator, y as f32 / denominator);
            if n < min {
                min = n;
            }
            if n > max {
                max = n;
            }
            data.last_mut().unwrap().push(n);
        }
    }

    Heightmap::new(
        data,
        settings.width,
        settings.height,
        max - min,
        max - min,
        None,
    )
    .normalize()
}

#[cfg(feature = "export")]
pub mod io {
    use crate::heightmap::*;
    use std::fs::File;
    use std::io::prelude::*;

    #[derive(Debug)]
    pub enum HeightmapIOError {
        FileExportError,
        FileImportError,
    }

    pub fn export(heightmap: &Heightmap, filename: &str) -> Result<(), HeightmapIOError> {
        fn _export(heightmap: &Heightmap, filename: &str) -> std::io::Result<()> {
            let data = serde_json::to_string(&heightmap).unwrap();
            let mut file = File::create(format!("{}.json", filename))?;
            file.write_all(data.as_bytes())?;
            Ok(())
        }

        match _export(heightmap, filename) {
            Ok(_) => Ok(()),
            Err(_) => Err(HeightmapIOError::FileExportError),
        }
    }

    pub fn import(filename: &str) -> Result<Heightmap, HeightmapIOError> {
        fn _import(filename: &str) -> std::io::Result<Heightmap> {
            let mut data = String::new();
            {
                let mut file = File::open(filename)?;
                file.read_to_string(&mut data)?;
            }

            let heightmap: Heightmap = serde_json::from_str(&data).unwrap();

            Ok(heightmap)
        }
        match _import(filename) {
            Ok(heightmap) => Ok(heightmap),
            Err(_) => Err(HeightmapIOError::FileImportError),
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

    pub fn export_heightmaps(heightmaps: Vec<&Heightmap>, filenames: Vec<&str>) {
        println!("Exporting heightmaps...");
        for (heightmap, filename) in heightmaps.iter().zip(filenames.iter()) {
            if let Err(e) = heightmap_to_image(heightmap, filename) {
                println!(
                    "Failed to save {}! Make sure the output folder exists.",
                    filename
                );
                println!("Given Reason: {}", e);
            }
            io::export(heightmap, filename).unwrap();
        }
    }
}
