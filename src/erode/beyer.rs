use crate::heightmap::*;
use rand::prelude::*;

const DROPLETS: usize = 1000;
const P_INERTIA: f32 = 0.8;
const P_CAPACITY: f32 = 8.0;
const P_DEPOSITION: f32 = 0.05;
const P_EROSION: f32 = 0.9;
const P_EVAPORATION: f32 = 0.05;
const P_RADIUS: usize = 3;
const P_MIN_SLOPE: f32 = 0.05;
const P_GRAVITY: f32 = 10.0;

pub struct Vector2 {
    x: HeightmapPrecision,
    y: HeightmapPrecision
}

impl Vector2 {
    fn new(x: HeightmapPrecision, y: HeightmapPrecision) -> Vector2 {
        Vector2 {
            x,
            y
        }
    }

    fn set_x(&mut self, x: HeightmapPrecision) {
        self.x = x;
    }
    
    fn set_y(&mut self, y: HeightmapPrecision) {
        self.y = y;
    }
    
}

pub enum Drop {
    Alive {
        position: Vector2,
        direction: Vector2,
        speed: f32,
        water: f32,
        sediment: f32
    },
    Dead
}

impl Drop {
    fn new(position: Vector2, sediment: f32, water: f32, speed: f32, direction: Vector2) -> Self {
       Drop::Alive {
           position,
           sediment,
           water,
           speed,
           direction
       } 
    }
    
    fn usize_position(&self) -> Option<(usize, usize)> {
        match self {
            Drop::Alive { position, .. } => {
                let x = (position.x).round() as i32;
                let y = (position.y).round() as i32;

                Some((x.try_into().unwrap(), y.try_into().unwrap()))
            },
            Drop::Dead => None
        }
    }

    fn gradient(&mut self, heightmap: &Heightmap) -> Option<Vector2> {
        let (ix, iy) = self.usize_position()?;
        
        if let Drop::Alive { position, .. } = self {
            let fx = position.x;
            let fy = position.y;
            
            let p_x0_y0 = heightmap.data[ix + 0][iy + 0];
            let p_x1_y0 = heightmap.data[ix + 1][iy + 0];
            let p_x0_y1 = heightmap.data[ix + 0][iy + 1];
            let p_x1_y1 = heightmap.data[ix + 1][iy + 1];

            let v = fx - fx.floor();
            let u = fy - fy.floor();

            let x0 = (p_x1_y0 - p_x0_y0) * (1.0 - v) + (p_x1_y1 - p_x0_y1) * v;
            let x1 = (p_x0_y1 - p_x0_y0) * (1.0 - u) + (p_x1_y1 - p_x1_y0) * u;
            
            Some(Vector2::new(x0, x1))
        } else {
            None
        }
        
    }

    fn set_direction(&mut self, gradient: &Vector2) {
        if let Drop::Alive { direction, .. } = self {
            let x_dir = direction.x;
            let y_dir = direction.y;
            
            direction.set_x(x_dir * P_INERTIA - gradient.x * (1.0 - P_INERTIA));
            direction.set_y(y_dir * P_INERTIA - gradient.y * (1.0 - P_INERTIA));
        }
    }
}

fn create_drop(heightmap: &Heightmap, rng: &mut ThreadRng) -> Drop {
        let x = rng.gen::<HeightmapPrecision>() * heightmap.width as HeightmapPrecision;
        let y = rng.gen::<HeightmapPrecision>() * heightmap.height as HeightmapPrecision;
        
        let direction: f32 = rng.gen::<f32>() * std::f32::consts::PI * 2.0;
        
        Drop::new(
            Vector2::new(x, y),
            0.0,
            0.0,
            0.0,
            Vector2::new(direction.cos(), direction.sin())
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
        
        while let Drop::Alive{..} = drop {
            tick(&mut heightmap, &mut drop)
        };
    }

    heightmap
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_fn_drop_usize_position() {
        let drop = Drop::new(Vector2::new(1.1, 2.8), 0.0, 0.0, 0.0, Vector2::new(0.0, 0.0));
        let usize_position = Some((1usize, 3usize));
        assert_eq!(drop.usize_position(), usize_position);

        let drop = Drop::Dead;
        assert_eq!(drop.usize_position(), None);
    }
    
}
