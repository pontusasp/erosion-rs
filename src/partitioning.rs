use std::sync::{Arc, Mutex};
use std::thread;
use crate::erode::lague;
use crate::heightmap;
use crate::math::UVector2;

pub enum Method {
    Subdivision,
}

pub fn subdivision_erode(heightmap: &mut heightmap::Heightmap, params: &lague::Parameters, subdivisions: u32) {
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
    let mut handles = Vec::new();

    let mut params = params.clone();
    params.num_iterations /= partitions.len();
    for i in 0..partitions.len() {
        let partition = Arc::clone(&partitions[i]);
        let handle = thread::spawn(move || {
            lague::erode(&mut partition.lock().unwrap().heightmap, &params);
        });
        handles.push(handle);
    }
    for handle in handles {
        handle.join().unwrap();
    }
    for partition in partitions {
        partition.lock().unwrap().apply_to(heightmap);
    }

    // heightmap = heightmap::PartialHeightmap::combine(&tl, &bl, &tr, &br);

}