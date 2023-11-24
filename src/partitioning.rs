use crate::erode;
use crate::erode::{DropZone, Parameters};
use crate::heightmap;
use crate::heightmap::{Heightmap, HeightmapPrecision};
use crate::math::UVector2;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;
use std::slice::Iter;
use std::sync::{Arc, Mutex};

pub const GAUSSIAN_DEFAULT_SIGMA: f32 = 2.0;
pub const GAUSSIAN_DEFAULT_BOUNDARY_THICKNESS: u16 = 2;

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum Method {
    Default,
    Subdivision(u32),
    SubdivisionBlurBoundary((u32, (f32, u16))),
    SubdivisionOverlap(u32),
    GridOverlapBlend(usize),
}

impl Method {
    pub fn to_string(self) -> String {
        match self {
            Method::Default => String::from("Default"),
            Method::Subdivision(_) => String::from("Subdivision"),
            Method::SubdivisionBlurBoundary(_) => String::from("SubdivisionBlurBoundary"),
            Method::SubdivisionOverlap(_) => String::from("SubdivisionOverlap"),
            Method::GridOverlapBlend(_) => String::from("GridOverlapBlend"),
        }
    }

    pub fn next(self) -> Self {
        match self {
            Method::Default => Method::Subdivision(crate::PRESET_SUBDIVISIONS),
            Method::Subdivision(subdivisions) => Method::SubdivisionBlurBoundary((
                subdivisions,
                (GAUSSIAN_DEFAULT_SIGMA, GAUSSIAN_DEFAULT_BOUNDARY_THICKNESS),
            )),
            Method::SubdivisionBlurBoundary((subdivisions, _)) => {
                Method::SubdivisionOverlap(subdivisions)
            }
            Method::SubdivisionOverlap(_) => Method::GridOverlapBlend(crate::PRESET_GRID_SIZE),
            Method::GridOverlapBlend(_) => Method::Default,
        }
    }

    pub fn previous(self) -> Self {
        match self {
            Method::Subdivision(_) => Method::Default,
            Method::SubdivisionBlurBoundary((subdivisions, _)) => Method::Subdivision(subdivisions),
            Method::SubdivisionOverlap(subdivisions) => Method::SubdivisionBlurBoundary((
                subdivisions,
                (GAUSSIAN_DEFAULT_SIGMA, GAUSSIAN_DEFAULT_BOUNDARY_THICKNESS),
            )),
            Method::GridOverlapBlend(_) => Method::SubdivisionOverlap(crate::PRESET_SUBDIVISIONS),
            Method::Default => Method::GridOverlapBlend(crate::PRESET_GRID_SIZE),
        }
    }

    pub fn matches(&self, other: &Self) -> bool {
        match self {
            Method::Default => matches!(other, Method::Default),
            Method::Subdivision(_) => matches!(other, Method::Subdivision(_)),
            Method::SubdivisionBlurBoundary(_) => {
                matches!(other, Method::SubdivisionBlurBoundary(_))
            }
            Method::SubdivisionOverlap(_) => matches!(other, Method::SubdivisionOverlap(_)),
            Method::GridOverlapBlend(_) => matches!(other, Method::GridOverlapBlend(_)),
        }
    }

    pub fn iterator() -> Iter<'static, Method> {
        static EROSION_METHODS: &[Method] = &[
            Method::Default,
            Method::Subdivision(crate::PRESET_SUBDIVISIONS),
            Method::SubdivisionBlurBoundary((
                crate::PRESET_SUBDIVISIONS,
                (GAUSSIAN_DEFAULT_SIGMA, GAUSSIAN_DEFAULT_BOUNDARY_THICKNESS),
            )),
            Method::SubdivisionOverlap(crate::PRESET_SUBDIVISIONS),
            Method::GridOverlapBlend(crate::PRESET_GRID_SIZE),
        ];
        EROSION_METHODS.iter()
    }

    pub fn set_subdivisions_unchecked(&mut self, value: u32) {
        match self {
            Method::Default => (),
            Method::Subdivision(ref mut subdivisions)
            | Method::SubdivisionOverlap(ref mut subdivisions) => {
                *subdivisions = value;
            }
            Method::SubdivisionBlurBoundary((ref mut subdivisions, _)) => {
                *subdivisions = value;
            }
            Method::GridOverlapBlend(_) => (),
        };
    }

    pub fn set_grid_size_unchecked(&mut self, value: usize) {
        match self {
            Method::Default => (),
            Method::Subdivision(_) | Method::SubdivisionOverlap(_) => (),
            Method::SubdivisionBlurBoundary(_) => (),
            Method::GridOverlapBlend(ref mut grid_size) => {
                *grid_size = value;
            }
        };
    }

    pub fn erode_with_margin(
        &self,
        use_margin: bool,
        heightmap: &Heightmap,
        parameters: &Parameters,
        drop_zone: &DropZone,
        subdivisions: u32,
        grid_size: usize,
    ) -> Heightmap {
        print!("Eroding using ");
        let heightmap_size = heightmap.width;
        let (local_margin, margin) = if use_margin {
            let max_margin = Self::max_margin(heightmap_size, subdivisions, grid_size);
            let local_margin = self.margin_size(heightmap_size, subdivisions, grid_size);
            let (mr, mt, ml, mb) = max_margin;
            let (lr, lt, ll, lb) = local_margin;
            let margin = (mr - lr, mt - lt, ml - ll, mb - lb);
            (local_margin, margin)
        } else {
            ((0,0,0,0),(0,0,0,0))
        };
        let mut partition = heightmap.with_margin(margin);
        match self {
            Method::Default => {
                println!("{} method (no partitioning)", Method::Default.to_string());
                default_erode(&mut partition.heightmap, &parameters, &drop_zone);
            }
            Method::Subdivision(subdivisions) => {
                println!("{} method", Method::Subdivision(*subdivisions).to_string());
                subdivision_erode(&mut partition.heightmap, &parameters, *subdivisions);
            }
            Method::SubdivisionBlurBoundary((subdivisions, (sigma, thickness))) => {
                println!(
                    "{} method",
                    Method::SubdivisionBlurBoundary((
                        *subdivisions,
                        (GAUSSIAN_DEFAULT_SIGMA, GAUSSIAN_DEFAULT_BOUNDARY_THICKNESS)
                    ))
                    .to_string()
                );
                subdivision_blur_boundary_erode(
                    &mut partition.heightmap,
                    &parameters,
                    *subdivisions,
                    *sigma,
                    *thickness,
                );
            }
            Method::SubdivisionOverlap(subdivisions) => {
                println!(
                    "{} method",
                    Method::SubdivisionOverlap(*subdivisions).to_string()
                );
                subdivision_overlap_erode(&mut partition.heightmap, &parameters, *subdivisions);
            }
            Method::GridOverlapBlend(grid_size) => {
                println!(
                    "{} method",
                    Method::GridOverlapBlend(*grid_size).to_string()
                );
                grid_overlap_blend_erode(
                    &mut partition.heightmap,
                    &parameters,
                    *grid_size,
                    *grid_size,
                );
            }
        }
        partition
            .heightmap
            .with_margin(local_margin)
            .heightmap
    }

    pub fn margin_size(&self, heightmap_size: usize, subdivisions: u32, grid_size: usize) -> (usize, usize, usize, usize) {
        let subdivision_cell_size = heightmap_size / 2_usize.pow(subdivisions);
        let margins = match self {
            Method::Default => (0, 0, 0, 0),
            Method::Subdivision(_) => (0, 0, 0, 0),
            Method::SubdivisionBlurBoundary(_) => (0, 0, 0, 0),
            Method::SubdivisionOverlap(_) => {
                // let m = subdivision_cell_size / 2;
                // (m, m, m, m)

                let grid_size = 2_usize.pow(subdivisions);
                let grid_cell_size = heightmap_size / grid_size;
                let offset0 = (grid_cell_size) / 2;
                let offset1 = heightmap_size - (grid_cell_size) / 2;

                let slices = grid_size - 1;
                let slice_size = (offset1 - offset0) / slices;

                let total_inner_size = slices * slice_size;

                let margin0 = offset0;
                let margin1 = heightmap_size - margin0 - total_inner_size;
                (margin1, margin0, margin0, margin1)
            },
            Method::GridOverlapBlend(_) => {
                let grid_cell_size = heightmap_size / grid_size;
                let offset0 = (grid_cell_size) / 2;
                let offset1 = heightmap_size - (grid_cell_size) / 2;

                let slices = grid_size - 1;
                let slice_size = (offset1 - offset0) / slices;

                let total_inner_size = slices * slice_size;

                let margin0 = offset0;
                let margin1 = heightmap_size - margin0 - total_inner_size;
                (margin1, margin0, margin0, margin1)
            },
        };
        let (right, top, left, bottom) = margins;
        (right, top, left, bottom)
    }

    pub fn max_margin(heightmap_size: usize, subdivisions: u32, grid_size: usize) -> (usize, usize, usize, usize) {
        let mut largest_margin_r = 0;
        let mut largest_margin_t = 0;
        let mut largest_margin_l = 0;
        let mut largest_margin_b = 0;
        for &m in Self::iterator() {
            let (r, t, l, b) = m.margin_size(heightmap_size, subdivisions, grid_size);
            largest_margin_r = largest_margin_r.max(r);
            largest_margin_t = largest_margin_t.max(t);
            largest_margin_l = largest_margin_l.max(l);
            largest_margin_b = largest_margin_b.max(b);
        }
        (largest_margin_r, largest_margin_t, largest_margin_l, largest_margin_b)
    }
}

fn subdivide(
    heightmap: &heightmap::Heightmap,
    subdivisions: u32,
) -> Vec<Arc<Mutex<heightmap::PartialHeightmap>>> {
    let slice_amount = 2_usize.pow(subdivisions);
    let slices = UVector2 {
        x: slice_amount,
        y: slice_amount,
    };
    let size = UVector2 {
        x: heightmap.width / slices.x,
        y: heightmap.height / slices.y,
    };
    let mut partitions = Vec::new();
    for x in 0..slices.x {
        for y in 0..slices.y {
            let anchor = UVector2 {
                x: x * size.x,
                y: y * size.y,
            };
            let partition = Arc::new(Mutex::new(heightmap::PartialHeightmap::from(
                &heightmap, &anchor, &size,
            )));
            partitions.push(partition);
        }
    }
    partitions
}

fn subdivide_partition(
    partial: &heightmap::PartialHeightmap,
    subdivisions: u32,
) -> Vec<Arc<Mutex<heightmap::PartialHeightmap>>> {
    let slice_amount = 2_usize.pow(subdivisions) - 1;
    let slices = UVector2 {
        x: slice_amount,
        y: slice_amount,
    };
    let size = UVector2 {
        x: partial.heightmap.width / slices.x,
        y: partial.heightmap.height / slices.y,
    };
    let mut partitions = Vec::new();
    for x in 0..slices.x {
        for y in 0..slices.y {
            let anchor = UVector2 {
                x: x * size.x,
                y: y * size.y,
            };
            let partition = Arc::new(Mutex::new(partial.nest(&anchor, &size)));
            partitions.push(partition);
        }
    }
    partitions
}

fn erode_multiple(
    heightmaps: &Vec<Arc<Mutex<heightmap::PartialHeightmap>>>,
    params: erode::Parameters,
    heightmap: &mut heightmap::Heightmap,
) {
    heightmaps.par_iter().for_each(|partition| {
        let heightmap = &mut partition.lock().unwrap().heightmap;
        let drop_zone = erode::DropZone::default(heightmap);
        erode::erode(heightmap, &params, &drop_zone);
    });

    for partition in heightmaps {
        partition.lock().unwrap().apply_to(heightmap);
    }
}

pub fn default_erode(
    heightmap: &mut heightmap::Heightmap,
    params: &erode::Parameters,
    drop_zone: &erode::DropZone,
) {
    erode::erode(heightmap, &params, drop_zone);
}

pub fn subdivision_erode(
    heightmap: &mut heightmap::Heightmap,
    params: &erode::Parameters,
    subdivisions: u32,
) {
    let partitions = subdivide(heightmap, subdivisions);

    let mut params = params.clone();
    params.num_iterations /= partitions.len();

    erode_multiple(&partitions, params, heightmap);
}

pub fn subdivision_blur_boundary_erode(
    heightmap: &mut heightmap::Heightmap,
    params: &erode::Parameters,
    subdivisions: u32,
    sigma: f32,
    thickness: u16,
) {
    subdivision_erode(heightmap, params, subdivisions);
    let blurred = heightmap.blur(sigma).unwrap();
    let size = heightmap.width;
    let mask = heightmap::create_heightmap_from_closure(
        heightmap.width,
        1.0,
        &|x, y| -> HeightmapPrecision {
            let chunk = size as i32 / 2i32.pow(subdivisions);
            let dx = (chunk - x as i32 % chunk).abs().min(x as i32 % chunk);
            let dy = (chunk - y as i32 % chunk).abs().min(y as i32 % chunk);
            let d = dx.min(dy);
            if d >= thickness as i32 {
                0.0
            } else {
                (d as f32 / thickness as f32 * PI / 2.0).cos()
            }
        },
    );
    heightmap
        .overlay(&blurred, &mask)
        .expect("Subdivision Blur Boundary Erode failed.");
}

pub fn subdivision_overlap_erode(
    heightmap: &mut heightmap::Heightmap,
    params: &erode::Parameters,
    subdivisions: u32,
) {
    assert!(subdivisions > 0);
    let partitions = subdivide(heightmap, subdivisions);
    let (cell_width, cell_height) = {
        let partition = partitions[0].lock().unwrap();
        (partition.heightmap.width, partition.heightmap.height)
    };

    let mut params = params.clone();
    params.num_iterations /= (partitions.len() + partitions.len() - 1) / 2;

    erode_multiple(&partitions, params, heightmap);

    let partial = heightmap::PartialHeightmap::from(
        heightmap,
        &UVector2 {
            x: cell_width / 2,
            y: cell_height / 2,
        },
        &UVector2 {
            x: heightmap.width - cell_width,
            y: heightmap.height - cell_height,
        },
    );
    let nested_partitions = subdivide_partition(&partial, subdivisions);
    erode_multiple(&nested_partitions, params, heightmap);
}

fn get_grid(
    heightmap: &heightmap::Heightmap,
    rect_min: &UVector2,
    rect_max: &UVector2,
    grid_cells: &UVector2,
) -> Vec<Vec<Arc<Mutex<heightmap::PartialHeightmap>>>> {
    let mut grid = Vec::new();
    let slice_width = (rect_max.x - rect_min.x) / grid_cells.x;
    let slice_height = (rect_max.y - rect_min.y) / grid_cells.y;
    for x in 0..grid_cells.x {
        let mut row = Vec::new();
        for y in 0..grid_cells.y {
            let anchor = UVector2 {
                x: x * slice_width + rect_min.x,
                y: y * slice_height + rect_min.y,
            };
            let size = UVector2 {
                x: slice_width,
                y: slice_height,
            };
            let partition = Arc::new(Mutex::new(heightmap::PartialHeightmap::from(
                &heightmap, &anchor, &size,
            )));
            row.push(partition);
        }
        grid.push(row);
    }
    grid
}

fn erode_grid(
    grid: &Vec<Vec<Arc<Mutex<heightmap::PartialHeightmap>>>>,
    params: &erode::Parameters,
) {
    let mut params = params.clone();
    let grid_width = grid.len();
    let grid_height = grid[0].len();
    params.num_iterations /= grid_width * grid_height;

    (0..grid_width).for_each(|x| {
        (0..grid_height).into_par_iter().for_each(|y| {
            let partition = Arc::clone(&grid[x][y]);
            let heightmap = &mut partition.lock().unwrap().heightmap;
            let drop_zone = erode::DropZone::default(heightmap);
            erode::erode(heightmap, &params, &drop_zone);
        });
    });
}

fn blend_cells(
    center: Arc<Mutex<heightmap::PartialHeightmap>>,
    tl: Arc<Mutex<heightmap::PartialHeightmap>>,
    tr: Arc<Mutex<heightmap::PartialHeightmap>>,
    bl: Arc<Mutex<heightmap::PartialHeightmap>>,
    br: Arc<Mutex<heightmap::PartialHeightmap>>,
) {
    let mut center = center.lock().unwrap();
    let tl = tl.lock().unwrap();
    let tr = tr.lock().unwrap();
    let bl = bl.lock().unwrap();
    let br = br.lock().unwrap();

    tl.blend_apply_to(&mut center);
    tr.blend_apply_to(&mut center);
    bl.blend_apply_to(&mut center);
    br.blend_apply_to(&mut center);
}

pub fn grid_overlap_blend_erode(
    heightmap: &mut heightmap::Heightmap,
    params: &erode::Parameters,
    grid_x_slices: usize,
    grid_y_slices: usize,
) {
    let slice_width = heightmap.width / grid_x_slices;
    let slice_height = heightmap.height / grid_y_slices;
    let offset_grid = get_grid(
        heightmap,
        &UVector2 {
            x: slice_width / 2,
            y: slice_height / 2,
        },
        &UVector2 {
            x: heightmap.width - slice_width / 2,
            y: heightmap.height - slice_height / 2,
        },
        &UVector2 {
            x: grid_x_slices - 1,
            y: grid_y_slices - 1,
        },
    );

    let grid = get_grid(
        heightmap,
        &UVector2 { x: 0, y: 0 },
        &UVector2 {
            x: heightmap.width,
            y: heightmap.height,
        },
        &UVector2 {
            x: grid_x_slices,
            y: grid_y_slices,
        },
    );

    erode_grid(&grid, params);
    erode_grid(&offset_grid, params);

    for i in 0..=1 {
        for j in 0..=1 {
            (i..offset_grid.len()).step_by(2).for_each(|x| {
                (j..offset_grid[x].len())
                    .into_par_iter()
                    .step_by(2)
                    .for_each(|y| {
                        let center = Arc::clone(&offset_grid[x][y]);
                        let tl = Arc::clone(&grid[x][y]);
                        let tr = Arc::clone(&grid[x + 1][y]);
                        let bl = Arc::clone(&grid[x][y + 1]);
                        let br = Arc::clone(&grid[x + 1][y + 1]);
                        blend_cells(center, tl, tr, bl, br);
                    });
            });
        }
    }

    for col in offset_grid {
        for partition in col {
            partition.lock().unwrap().apply_to(heightmap);
        }
    }
}
