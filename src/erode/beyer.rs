use crate::heightmap;
use rand::prelude::*;
use nalgebra::{vector, Vector2};

const DROPLETS: usize = 1000;
const INERTIA: f32 = 0.8;

struct Drop {
    position: Vector2<f32>,
    sediment: f32,
    water: f32,
    speed: f32,
    direction: Vector2<f32>
}

fn drop_position(drop: &Drop) -> (usize, usize) {
    let x = (drop.position.x) as i32;
    let y = (drop.position.y) as i32;

    (x.try_into().unwrap(), y.try_into().unwrap())
}

pub fn erode(heightmap: &heightmap::Heightmap) -> heightmap::Heightmap {
    let mut data = heightmap.data.clone();
    let mut rng = rand::thread_rng();
    
    for _ in 0..DROPLETS {
        
        let x = rng.gen::<f32>() * heightmap.width as f32;
        let y = rng.gen::<f32>() * heightmap.height as f32;
        
        let direction: f32 = rng.gen::<f32>() * std::f32::consts::PI * 2.0;
        
        let mut drop = Drop {
            position: vector![x, y],
            sediment: 0.0,
            water: 0.0,
            speed: 0.0,
            direction: vector![direction.cos(), direction.sin()]
        };

        let (ix, iy) = drop_position(&drop);

        println!("removing all sediment at drop position for testing");
        data[ix][iy] = 0.0;
        
    }

    heightmap::Heightmap::new(data, heightmap.width, heightmap.height, heightmap.depth)
}
