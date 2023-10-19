use crate::erode::lague;
use crate::heightmap;
use crate::math::UVector2;
use std::slice::Iter;
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Method {
    Default,
    Subdivision,
    SubdivisionOverlap,
    GridOverlapBlend,
}

impl Method {
    pub fn to_string(self) -> String {
        match self {
            Method::Default => String::from("Default"),
            Method::Subdivision => String::from("Subdivision"),
            Method::SubdivisionOverlap => String::from("SubdivisionOverlap"),
            Method::GridOverlapBlend => String::from("GridOverlapBlend"),
        }
    }

    pub fn next(self) -> Self {
        match self {
            Method::Default => Method::Subdivision,
            Method::Subdivision => Method::SubdivisionOverlap,
            Method::SubdivisionOverlap => Method::GridOverlapBlend,
            Method::GridOverlapBlend => Method::Default,
        }
    }

    pub fn previous(self) -> Self {
        match self {
            Method::Subdivision => Method::Default,
            Method::SubdivisionOverlap => Method::Subdivision,
            Method::GridOverlapBlend => Method::SubdivisionOverlap,
            Method::Default => Method::GridOverlapBlend,
        }
    }

    pub fn iterator() -> Iter<'static, Method> {
        static EROSION_METHODS: [Method; 4] = [
            Method::Default,
            Method::Subdivision,
            Method::SubdivisionOverlap,
            Method::GridOverlapBlend,
        ];
        EROSION_METHODS.iter()
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
    params: lague::Parameters,
    heightmap: &mut heightmap::Heightmap,
) {
    let mut handles = Vec::new();
    for i in 0..heightmaps.len() {
        let partition = Arc::clone(&heightmaps[i]);
        let handle = thread::spawn(move || {
            let heightmap = &mut partition.lock().unwrap().heightmap;
            let drop_zone = lague::DropZone::default(heightmap);
            lague::erode(heightmap, &params, &drop_zone);
        });
        handles.push(handle);
    }
    for handle in handles {
        handle.join().unwrap();
    }
    for partition in heightmaps {
        partition.lock().unwrap().apply_to(heightmap);
    }
}

pub fn default_erode(
    heightmap: &mut heightmap::Heightmap,
    params: &lague::Parameters,
    drop_zone: &lague::DropZone,
) {
    lague::erode(heightmap, &params, drop_zone);
}

pub fn subdivision_erode(
    heightmap: &mut heightmap::Heightmap,
    params: &lague::Parameters,
    subdivisions: u32,
) {
    let partitions = subdivide(heightmap, subdivisions);

    let mut params = params.clone();
    params.num_iterations /= partitions.len();

    erode_multiple(&partitions, params, heightmap);
}

pub fn subdivision_overlap_erode(
    heightmap: &mut heightmap::Heightmap,
    params: &lague::Parameters,
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

fn get_grid(heightmap: &heightmap::Heightmap, rect_min: &UVector2, rect_max: &UVector2, grid_cells: &UVector2) -> Vec<Vec<Arc<Mutex<heightmap::PartialHeightmap>>>> {
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

fn erode_grid(grid: &Vec<Vec<Arc<Mutex<heightmap::PartialHeightmap>>>>, params: &lague::Parameters) {
    let mut params = params.clone();
    let grid_width = grid.len();
    let grid_height = grid[0].len();
    params.num_iterations /= grid_width * grid_height;

    let mut handles = Vec::new();
    for x in 0..grid_width {
        for y in 0..grid_height {
            let partition = Arc::clone(&grid[x][y]);
            let handle = thread::spawn(move || {
                let heightmap = &mut partition.lock().unwrap().heightmap;
                let drop_zone = lague::DropZone::default(heightmap);
                lague::erode(heightmap, &params, &drop_zone);
            });
            handles.push(handle);
        }
    }
    for handle in handles {
        handle.join().unwrap();
    }
}

fn blend_cells(center: Arc<Mutex<heightmap::PartialHeightmap>>, tl: Arc<Mutex<heightmap::PartialHeightmap>>, tr: Arc<Mutex<heightmap::PartialHeightmap>>, bl: Arc<Mutex<heightmap::PartialHeightmap>>, br: Arc<Mutex<heightmap::PartialHeightmap>>) {
    let mut center = center.lock().unwrap();
    let tl = tl.lock().unwrap();
    let tr = tr .lock().unwrap();
    let bl = bl .lock().unwrap();
    let br = br .lock().unwrap();

    tl.blend_apply_to(&mut center);
    tr.blend_apply_to(&mut center);
    bl.blend_apply_to(&mut center);
    br.blend_apply_to(&mut center);
}

pub fn grid_overlap_blend_erode(
    heightmap: &mut heightmap::Heightmap,
    params: &lague::Parameters,
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
            let mut handles = Vec::new();
            for x in (i..offset_grid.len()).step_by(2) {
                for y in (j..offset_grid[x].len()).step_by(2) {
                    let center = Arc::clone(&offset_grid[x][y]);
                    let tl = Arc::clone(&grid[x][y]);
                    let tr = Arc::clone(&grid[x + 1][y]);
                    let bl = Arc::clone(&grid[x][y + 1]);
                    let br = Arc::clone(&grid[x + 1][y + 1]);
                    let handle = thread::spawn(move || {
                        blend_cells(center, tl, tr, bl, br);
                    });
                    handles.push(handle);
                }
            }
            for handle in handles {
                handle.join().unwrap();
            }
        }
    }

    for col in offset_grid {
        for partition in col {
            partition.lock().unwrap().apply_to(heightmap);
        }
    }
}
