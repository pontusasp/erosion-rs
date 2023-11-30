use bracket_noise::prelude::*;
use rayon::iter::IntoParallelRefMutIterator;
use rayon::prelude::*;
use serde::{Deserialize, Serialize, Serializer};
use std::collections::{HashMap, VecDeque};
use std::f32::consts::PI;
use std::fmt::{Display, Formatter};

use crate::math::{UVector2, Vector2};

use crate::visualize::wrappers::{FractalTypeWrapper, NoiseTypeWrapper};
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

    pub fn new_empty(
        width: usize,
        height: usize,
        depth: HeightmapPrecision,
        original_depth: HeightmapPrecision,
    ) -> Self {
        let data = vec![vec![0.0; height]; width];
        Self::new(data, width, height, depth, original_depth, None)
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

    pub fn with_margin(&self, margin_x: usize, margin_y: usize) -> PartialHeightmap {
        PartialHeightmap::from(
            self,
            &UVector2 {
                x: margin_x,
                y: margin_y,
            },
            &UVector2 {
                x: self.width - margin_x * 2,
                y: self.height - margin_y * 2,
            },
        )
    }

    pub fn blur(&self, sigma: f32) -> Option<Heightmap> {
        let gray_image: Option<GrayImage> = self.get_gray_image();
        let blurred_gray_image = imageops::blur(&gray_image?, sigma);

        let blurred_heightmap =
            Heightmap::from_u8(blurred_gray_image.as_raw(), self.width, self.height);
        Some(blurred_heightmap)
    }

    pub fn boolean(mut self, threshold: HeightmapPrecision, round_up: bool, invert: bool) -> Self {
        let one = if invert { 0.0 } else { 1.0 };
        let zero = 1.0 - one;
        for x in 0..self.width {
            for y in 0..self.height {
                let d = self.data[x][y];
                self.data[x][y] = if d == threshold {
                    if round_up {
                        one
                    } else {
                        zero
                    }
                } else if d < threshold {
                    zero
                } else {
                    one
                }
            }
        }
        self
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
        Some(*self.data.get(x)?.get(y)?)
    }

    pub fn get_with_int(&self, x: i32, y: i32) -> Option<HeightmapPrecision> {
        let x_usize: usize = x.try_into().ok()?;
        let y_usize: usize = y.try_into().ok()?;
        self.get(x_usize, y_usize)
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

    pub fn isoline(&self, height: HeightmapPrecision, error: HeightmapPrecision) -> Self {
        let func = |x: usize, y: usize| -> HeightmapPrecision {
            let h = self.data[x][y];
            if height - error < h && h < height + error {
                1.0
            } else {
                0.0
            }
        };

        create_heightmap_from_closure(self.width, 1.0, &func)
    }

    pub fn get_flood_points(&self, isoline: &Self, inside: bool) -> Vec<UVector2> {
        let mut points = Vec::new();
        for x0 in 0..self.width {
            for y0 in 0..self.height {
                if isoline.data[x0][y0] == 0.0 {
                    continue;
                }
                let adj = &[
                    (x0 != 0, (x0 as i32 - 1, y0 as i32)),
                    (x0 != self.width - 1, (x0 as i32 + 1, y0 as i32)),
                    (y0 != 0, (x0 as i32, y0 as i32 - 1)),
                    (y0 != self.height - 1, (x0 as i32, y0 as i32 + 1)),
                ];
                for (has_edge, (x1, y1)) in *adj {
                    if has_edge
                        && isoline.data[x1 as usize][y1 as usize] == 0.0
                        && ((inside && self.data[x1 as usize][y1 as usize] < self.data[x0][y0])
                            || (!inside && self.data[x0][y0] < self.data[x1 as usize][y1 as usize]))
                    {
                        points.push(UVector2::new(x1 as usize, y1 as usize));
                    }
                }
            }
        }
        points
    }

    pub fn from_points(size: usize, points: &Vec<UVector2>, fill: HeightmapPrecision) -> Self {
        let mut heightmap = Self::new_empty(size, size, fill, fill);
        for &UVector2 { x, y } in points {
            heightmap.data[x][y] = fill;
        }
        heightmap
    }

    pub fn filter_noise_points(
        size: usize,
        points: &Vec<UVector2>,
        kernel_radius: usize,
        iterations: usize,
    ) -> Vec<UVector2> {
        if iterations == 0 || kernel_radius == 0 {
            return points.clone();
        }
        let fill = 1.0;
        let heightmap = Self::from_points(size, points, fill);
        let mut points_clean = Vec::new();

        for point in points {
            let &UVector2 { x, y } = point;
            let middle = heightmap.data[x][y];
            let x0 = x as i32;
            let y0 = y as i32;
            let mut neighbours = 1;
            let mut neighbours_with_points = 1;
            for x1 in (x0 - kernel_radius as i32)..=(x0 + kernel_radius as i32) {
                for y1 in (y0 - kernel_radius as i32)..=(y0 + kernel_radius as i32) {
                    if x0 == x1 && y0 == y1 {
                        continue;
                    }
                    if let Some(value) = heightmap.get_with_int(x1, y1) {
                        neighbours += 1;
                        if value == middle {
                            neighbours_with_points += 1;
                        }
                    }
                }
            }
            let ratio: f32 = neighbours_with_points as f32 / neighbours as f32;
            let kernel_size = kernel_radius * 2 + 1;
            let minimal_desired_ratio: f32 = 1f32 / kernel_size as f32;
            if minimal_desired_ratio <= ratio {
                points_clean.push(*point);
            }
        }
        if iterations > 1 {
            Self::filter_noise_points(size, &points_clean, kernel_radius, iterations - 1)
        } else {
            points_clean
        }
    }

    pub fn flood_empty(&self, with: HeightmapPrecision, from: &Vec<UVector2>) -> (Self, usize) {
        let mut flooded = 0;
        let mut heightmap = self.clone();
        let mut queue = VecDeque::new();
        for from in from.iter() {
            if let Some(h) = heightmap.get(from.x, from.y) {
                if h != 0.0 {
                    continue;
                }
            } else {
                continue;
            }
            flooded += 1;
            heightmap.data[from.x][from.y] = with;
            queue.push_back(*from);

            while !queue.is_empty() {
                let pixel = queue.pop_front().unwrap();
                let adj = &[
                    (pixel.x != 0, (pixel.x as i32 - 1, pixel.y as i32)),
                    (
                        pixel.x != self.width - 1,
                        (pixel.x as i32 + 1, pixel.y as i32),
                    ),
                    (pixel.y != 0, (pixel.x as i32, pixel.y as i32 - 1)),
                    (
                        pixel.y != self.height - 1,
                        (pixel.x as i32, pixel.y as i32 + 1),
                    ),
                ];
                for (has_edge, (x, y)) in *adj {
                    if has_edge {
                        let data = &mut heightmap.data;
                        let x = x as usize;
                        let y = y as usize;
                        if data[x][y] == 0.0 {
                            data[x][y] = with;
                            queue.push_back(UVector2::new(x, y));
                        }
                    }
                }
            }
        }

        (heightmap, flooded)
    }

    pub fn flood_less_than(
        &self,
        height: HeightmapPrecision,
        with: HeightmapPrecision,
        from: &Vec<UVector2>,
    ) -> (Self, usize) {
        if height > with {
            panic!("Must flood with greater value than given height. ('height' must be <= 'with', {} !<= {})", height, with);
        }
        let mut flooded = 0;
        let mut heightmap = self.clone();
        let mut queue = VecDeque::new();
        for from in from.iter() {
            if let Some(h) = heightmap.get(from.x, from.y) {
                if h >= height {
                    continue;
                }
            } else {
                continue;
            }
            flooded += 1;
            heightmap.data[from.x][from.y] = with;
            queue.push_back(*from);

            while !queue.is_empty() {
                let pixel = queue.pop_front().unwrap();
                let adj = &[
                    (pixel.x != 0, (pixel.x - 1, pixel.y)),
                    (pixel.x != self.width - 1, (pixel.x + 1, pixel.y)),
                    (pixel.y != 0, (pixel.x, pixel.y - 1)),
                    (pixel.y != self.height - 1, (pixel.x, pixel.y + 1)),
                ];
                for (has_edge, (x, y)) in *adj {
                    if has_edge {
                        let data = &mut heightmap.data;
                        if data[x][y] < with {
                            data[x][y] = with;
                            queue.push_back(UVector2::new(x, y));
                        }
                    }
                }
            }
        }

        (heightmap, flooded)
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

const DEFAULT_HEIGHTMAP_PARAMETERS: HeightmapParameters =
    HeightmapParameters {
        size: crate::PRESET_HEIGHTMAP_SIZE,
    };


#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub struct HeightmapParameters {
    pub size: usize,
}

impl HeightmapParameters {
    const fn static_default() -> Self {
        DEFAULT_HEIGHTMAP_PARAMETERS
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

impl Default for HeightmapParameters {
    fn default() -> Self {
        DEFAULT_HEIGHTMAP_PARAMETERS
    }
}

#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum HeightmapType {
    Procedural(HeightmapParameters, ProceduralHeightmapSettings),
    XGradient(HeightmapParameters),
    XGradientRepeating(HeightmapParameters, f32),
    XGradientRepeatingAlternating(HeightmapParameters, f32),
    XHyperbolaGradient(HeightmapParameters),
    CenteredHillGradient(HeightmapParameters, f32),
    XSinWave(HeightmapParameters, f32),
}

impl HeightmapType {
    pub fn params(&self) -> &HeightmapParameters {
        match self {
            HeightmapType::Procedural(params, _) => params,
            HeightmapType::XGradient(params) => params,
            HeightmapType::XGradientRepeating(params, _) => params,
            HeightmapType::XGradientRepeatingAlternating(params, _) => params,
            HeightmapType::XHyperbolaGradient(params) => params,
            HeightmapType::CenteredHillGradient(params, _) => params,
            HeightmapType::XSinWave(params, _) => params,
        }
    }

    pub fn params_mut(&mut self) -> &mut HeightmapParameters {
        match self {
            HeightmapType::Procedural(params, _) => params,
            HeightmapType::XGradient(params) => params,
            HeightmapType::XGradientRepeating(params, _) => params,
            HeightmapType::XGradientRepeatingAlternating(params, _) => params,
            HeightmapType::XHyperbolaGradient(params) => params,
            HeightmapType::CenteredHillGradient(params, _) => params,
            HeightmapType::XSinWave(params, _) => params,
        }
    }
}

impl Default for HeightmapType {
    fn default() -> Self {
        HeightmapType::Procedural(HeightmapParameters::default(), ProceduralHeightmapSettings::default())
    }
}

impl Display for HeightmapType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            HeightmapType::Procedural(_, _) => f.collect_str("Procedural"),
            HeightmapType::XGradient(_) => f.collect_str("Gradient"),
            HeightmapType::XGradientRepeating(_, _) => f.collect_str("Gradient Repeating"),
            HeightmapType::XGradientRepeatingAlternating(_, _) => {
                f.collect_str("Gradient Repeating Alternating")
            }
            HeightmapType::XHyperbolaGradient(_) => f.collect_str("Hyperbola Gradient"),
            HeightmapType::CenteredHillGradient(_, _) => f.collect_str("Centered Hill"),
            HeightmapType::XSinWave(_, _) => f.collect_str("Sin Wave"),
        }
    }
}

impl HeightmapType {
    pub fn first() -> Self {
        HeightmapType::default()
    }

    pub fn iterator() -> impl Iterator<Item = HeightmapType> {
        static TYPES: [HeightmapType; 7] = [
            HeightmapType::Procedural(HeightmapParameters::static_default(), ProceduralHeightmapSettings::static_default()),
            HeightmapType::XGradient(HeightmapParameters::static_default()),
            HeightmapType::XGradientRepeating(HeightmapParameters::static_default(), 8.0),
            HeightmapType::XGradientRepeatingAlternating(HeightmapParameters::static_default(), 8.0),
            HeightmapType::XHyperbolaGradient(HeightmapParameters::static_default()),
            HeightmapType::CenteredHillGradient(HeightmapParameters::static_default(), 0.75),
            HeightmapType::XSinWave(HeightmapParameters::static_default(), 8.0),
        ];
        TYPES.iter().copied()
    }
}

pub fn create_heightmap_from_preset(preset: &HeightmapType) -> Heightmap {
    match preset {
        HeightmapType::Procedural(params, settings) => create_perlin_heightmap(&params, &settings),
        HeightmapType::XGradient(params) => {
            create_heightmap_from_closure(params.size, 1.0, &|x: usize, _: usize| {
                x as HeightmapPrecision / params.size as HeightmapPrecision
            })
        }
        HeightmapType::XGradientRepeating(params, repetitions) => {
            create_heightmap_from_closure(params.size, 1.0, &|x: usize, _: usize| {
                (repetitions * x as HeightmapPrecision / params.size as HeightmapPrecision).fract()
            })
        }
        HeightmapType::XGradientRepeatingAlternating(params, repetitions) => {
            create_heightmap_from_closure(params.size, 1.0, &|x: usize, _: usize| {
                let v = repetitions * x as HeightmapPrecision / params.size as HeightmapPrecision;
                if v.trunc() % 2.0 == 0.0 {
                    v.fract()
                } else {
                    1.0 - v.fract()
                }
            })
        }
        HeightmapType::XHyperbolaGradient(params) => {
            create_heightmap_from_closure(params.size, 1.0, &|x: usize, _: usize| {
                let gradient = x as HeightmapPrecision / params.size as HeightmapPrecision;
                gradient.powi(2)
            })
        }
        HeightmapType::CenteredHillGradient(params, hill_radius) => {
            create_heightmap_from_closure(params.size, 1.0, &|x: usize, y: usize| {
                let radius = params.size as HeightmapPrecision / 2.0;
                let x = x as HeightmapPrecision;
                let y = y as HeightmapPrecision;
                let distance = ((x - radius).powf(2.0) + (y - radius).powf(2.0)).sqrt();

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
        HeightmapType::XSinWave(params, inverse_frequency) => {
            create_heightmap_from_closure(params.size, 1.0, &|x: usize, _| {
                let t = x as HeightmapPrecision / params.size as HeightmapPrecision;
                ((t * PI * inverse_frequency + PI).cos() + 1.0) / 2.0
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

#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub struct ProceduralHeightmapSettings {
    pub seed: u64,
    pub noise_type: NoiseTypeWrapper,
    pub fractal_type: FractalTypeWrapper,
    pub fractal_octaves: i32,
    pub fractal_gain: f32,
    pub fractal_lacunarity: f32,
    pub frequency: f32,
}

const DEFAULT_PROCEDURAL_HEIGHTMAP_SETTINGS: ProceduralHeightmapSettings =
    ProceduralHeightmapSettings {
        seed: 1337,
        noise_type: NoiseTypeWrapper::Perlin,
        fractal_type: FractalTypeWrapper::FBM,
        fractal_octaves: 5,
        fractal_gain: 0.6,
        fractal_lacunarity: 2.0,
        frequency: 0.5,
    };

impl ProceduralHeightmapSettings {
    const fn static_default() -> Self {
        DEFAULT_PROCEDURAL_HEIGHTMAP_SETTINGS
    }

    pub fn reset(&mut self) {
        *self = ProceduralHeightmapSettings::default()
    }
}

impl Default for ProceduralHeightmapSettings {
    fn default() -> Self {
        DEFAULT_PROCEDURAL_HEIGHTMAP_SETTINGS
    }
}

pub fn create_perlin_heightmap(params: &HeightmapParameters, settings: &ProceduralHeightmapSettings) -> Heightmap {
    let mut noise = FastNoise::seeded(settings.seed);
    noise.set_noise_type(settings.noise_type.into());
    noise.set_fractal_type(settings.fractal_type.into());
    noise.set_fractal_octaves(settings.fractal_octaves);
    noise.set_fractal_gain(settings.fractal_gain);
    noise.set_fractal_lacunarity(settings.fractal_lacunarity);
    noise.set_frequency(settings.frequency);

    let denominator = 100.0;

    let mut data: HeightmapData = Vec::new();

    let mut min = noise.get_noise(0.0, 0.0);
    let mut max = min.clone();

    for x in 0..params.size {
        data.push(vec![]);
        for y in 0..params.size {
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
        params.size,
        params.size,
        max - min,
        max - min,
        None,
    )
    .normalize()
}

#[cfg(feature = "export")]
pub mod io {
    use crate::heightmap::*;
    use std::fs::{self, File};
    use std::io::prelude::*;

    #[derive(Debug)]
    pub enum HeightmapIOError {
        FileExportError,
        FileImportError,
    }

    pub fn export(
        heightmap: &Heightmap,
        path: &str,
        filename: &str,
    ) -> Result<(), HeightmapIOError> {
        fn _export(heightmap: &Heightmap, path: &str, filename: &str) -> std::io::Result<()> {
            fs::create_dir_all(path)?;
            let data = serde_json::to_string(&heightmap).unwrap();
            let mut file = File::create(format!("{}.json", filename))?;
            file.write_all(data.as_bytes())?;
            Ok(())
        }

        match _export(heightmap, path, filename) {
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

    pub fn save_heightmap_as_image(
        heightmap: &Heightmap,
        filename: &str,
    ) -> image::ImageResult<()> {
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

    pub fn heightmap_to_image(
        heightmap: &Heightmap,
    ) -> image::ImageBuffer<image::Luma<u8>, Vec<u8>> {
        let buffer = heightmap.to_u8();
        image::ImageBuffer::from_raw(
            heightmap.width.try_into().unwrap(),
            heightmap.height.try_into().unwrap(),
            buffer,
        )
        .unwrap()
    }

    pub fn export_heightmaps(heightmaps: Vec<&Heightmap>, path: &str, filenames: Vec<&str>) {
        println!("Exporting heightmaps...");
        for (heightmap, filename) in heightmaps.iter().zip(filenames.iter()) {
            io::export(heightmap, path, filename).unwrap();
            if let Err(e) = save_heightmap_as_image(heightmap, filename) {
                println!(
                    "Failed to save {}! Make sure the output folder exists.",
                    filename
                );
                println!("Given Reason: {}", e);
            }
        }
    }
}
