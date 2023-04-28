use crate::heightmap::*;
use rand::prelude::*;
use nalgebra::{vector, Vector2};

const DROPLETS: usize = 1000;
const P_INERTIA: f32 = 0.8;
const P_CAPACITY: f32 = 8.0;
const P_DEPOSITION: f32 = 0.05;
const P_EROSION: f32 = 0.9;
const P_EVAPORATION: f32 = 0.05;
const P_RADIUS: usize = 3;
const P_MIN_SLOPE: f32 = 0.05;
const P_GRAVITY: f32 = 10.0;

pub enum Drop {
    Alive {
        position: Vector2<f32>,
        sediment: f32,
        water: f32,
        speed: f32,
        direction: Vector2<f32>
    },
    Dead
}

impl Drop {
    fn new(position: Vector2<f32>, sediment: f32, water: f32, speed: f32, direction: Vector2<f32>) -> Self {
       Drop::Alive {
           position,
           sediment,
           water,
           speed,
           direction
       } 
    }
    
    fn usize_position(&self) -> Option<(usize, usize)> {
        if let Drop::Alive{ position, .. } = self {
            let x = (position.x) as i32;
            let y = (position.y) as i32;

            Some((x.try_into().unwrap(), y.try_into().unwrap()))
        } else {
            None
        }
    }
}

fn create_drop(heightmap: &Heightmap, rng: &mut ThreadRng) -> Drop {
        let x = rng.gen::<HeightmapPrecision>() * heightmap.width as HeightmapPrecision;
        let y = rng.gen::<HeightmapPrecision>() * heightmap.height as HeightmapPrecision;
        
        let direction: f32 = rng.gen::<f32>() * std::f32::consts::PI * 2.0;
        
        Drop::new(
            vector![x, y],
            0.0,
            0.0,
            0.0,
            vector![direction.cos(), direction.sin()]
        )
}

fn tick(heightmap: &mut Heightmap, drop: &mut Drop) {
    if let Some((ix, iy)) = drop.usize_position() {
        println!("removing all sediment at drop position for testing");
        heightmap.set(ix, iy, 0.0);
    }
}

pub fn erode(heightmap: &Heightmap) -> Heightmap {
    let mut heightmap = heightmap.clone();
    let mut rng = rand::thread_rng();
    
    for _ in 0..DROPLETS {
        let mut drop = create_drop(&heightmap, &mut rng);
        let mut alive = true;
        
        while alive {
            tick(&mut heightmap, &mut drop);
            alive = if let Drop::Dead = drop { false } else { true };
        }
    }

    heightmap
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_fn_drop_usize_position() {
        let drop = Drop::new(vector![1.1, 2.8], 0.0, 0.0, 0.0, vector![0.0, 0.0]);
        let usize_position = Some((1usize, 2usize));
        assert_eq!(drop.usize_position(), usize_position);
    }
    
}
