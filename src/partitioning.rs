use std::sync::{Arc, Mutex};
use std::thread;
use crate::erode::lague;
use crate::heightmap;
use crate::math::UVector2;

pub enum Method {
    Subdivision,
    SubdivisionOverlap,
}

fn subdivide(heightmap: &heightmap::Heightmap, subdivisions: u32) -> Vec<Arc<Mutex<heightmap::PartialHeightmap>>> {
    let slice_amount = 2_usize.pow(subdivisions);
    let slices = UVector2 { x: slice_amount, y: slice_amount };
    let size = UVector2 { x: heightmap.width / slices.x, y: heightmap.height / slices.y };
    let mut partitions = Vec::new();
    for x in 0..slices.x {
        for y in 0..slices.y {
            let anchor = UVector2 { x: x * size.x, y: y * size.y };
            let partition = Arc::new(Mutex::new(heightmap::PartialHeightmap::from(&heightmap, &anchor, &size)));
            partitions.push(partition);
        }
    }
    partitions
}

fn subdivide_partition(partial: &heightmap::PartialHeightmap, subdivisions: u32) -> Vec<Arc<Mutex<heightmap::PartialHeightmap>>> {
    let slice_amount = 2_usize.pow(subdivisions) - 1;
    let slices = UVector2 { x: slice_amount, y: slice_amount };
    let size = UVector2 { x: partial.heightmap.width / slices.x, y: partial.heightmap.height / slices.y };
    let mut partitions = Vec::new();
    for x in 0..slices.x {
        for y in 0..slices.y {
            let anchor = UVector2 { x: x * size.x, y: y * size.y };
            let partition = Arc::new(Mutex::new(partial.nest(&anchor, &size)));
            partitions.push(partition);
        }
    }
    partitions
}

fn erode_multiple(heightmaps: &Vec<Arc<Mutex<heightmap::PartialHeightmap>>>, params: lague::Parameters, heightmap: &mut heightmap::Heightmap) {
    let mut handles = Vec::new();
    for i in 0..heightmaps.len() {
        let partition = Arc::clone(&heightmaps[i]);
        let handle = thread::spawn(move || {
            lague::erode(&mut partition.lock().unwrap().heightmap, &params);
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

pub fn subdivision_erode(heightmap: &mut heightmap::Heightmap, params: &lague::Parameters, subdivisions: u32) {
    let partitions = subdivide(heightmap, subdivisions);

    let mut params = params.clone();
    params.num_iterations /= partitions.len();
}

pub fn subdivision_overlap_erode(heightmap: &mut heightmap::Heightmap, params: &lague::Parameters, subdivisions: u32) {
    assert!(subdivisions > 0);
    let partitions = subdivide(heightmap, subdivisions);
    let (cell_width, cell_height) = {
        let partition = partitions[0].lock().unwrap();
        (partition.heightmap.width, partition.heightmap.height)
    };

    let mut params = params.clone();
    params.num_iterations /= (partitions.len() + partitions.len() - 1) / 2;

    erode_multiple(&partitions, params, heightmap);

    let partial = heightmap::PartialHeightmap::from(heightmap, &UVector2 { x: cell_width / 2, y: cell_height / 2 }, &UVector2 { x: heightmap.width - cell_width, y: heightmap.height - cell_height });
    let nested_partitions = subdivide_partition(&partial, subdivisions);
    erode_multiple(&nested_partitions, params, heightmap);
}