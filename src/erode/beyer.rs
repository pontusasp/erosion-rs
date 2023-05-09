use crate::heightmap::*;
use rand::prelude::*;

const DROPLETS: usize = 1_000;
const P_INERTIA: f32 = 0.8;
const P_CAPACITY: f32 = 8.0;
const P_DEPOSITION: f32 = 0.05;
const P_EROSION: f32 = 0.9;
const P_EVAPORATION: f32 = 0.05;
// const P_RADIUS: usize = 3;
const P_MIN_SLOPE: f32 = 0.05;
const P_GRAVITY: f32 = 0.2;
const P_MAX_PATH: usize = 10000;

const P_MIN_WATER: f32 = 0.00005;
const P_MIN_SPEED: f32 = 0.000001;

#[derive(Debug, PartialEq, Clone, Copy)]
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

#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
pub enum DropError {
    DropIsDead, 
    InvalidValue(&'static str),
    InvalidPosition(&'static str, Vector2)
}

impl Drop {

    fn new() -> Drop {
        Drop::Alive {
            position: Vector2::new(0.0, 0.0),
            direction: Vector2::new(0.0, 0.0),
            speed: 0.0,
            water: 0.0,
            sediment: 0.0
        }
    }

    fn set_position(&mut self, position: Vector2) -> Result<(), DropError> {
        if let Drop::Alive { position: p, .. } = self {
            *p = position;
            Ok(())
        } else {
            Err(DropError::DropIsDead)
        }
    }

    fn get_position(&self) -> Result<Vector2, DropError> {
        match self {
            Drop::Alive { position, .. } => Ok(*position),
            Drop::Dead => Err(DropError::DropIsDead)
        }
    }

    fn set_direction(&mut self, direction: Vector2) -> Result<(), DropError> {
        match self {
            Drop::Alive { direction: d, .. } => {
                *d = direction;
                Ok(())
            },
            Drop::Dead => Err(DropError::DropIsDead)
        }
    }

    fn set_speed(&mut self, speed: f32) -> Result<(), DropError> {
        if speed < 0.0 {
            Err(DropError::InvalidValue("Speed cannot be negative"))
        } else {
            match self {
                Drop::Alive { speed: s, .. } => {
                    *s = speed;
                    Ok(())
                },
                Drop::Dead => Err(DropError::DropIsDead)
            }
        }
    }

    fn get_speed(&self) -> Result<f32, DropError> {
        match self {
            Drop::Alive { speed, .. } => Ok(*speed),
            Drop::Dead => Err(DropError::DropIsDead)
        }
    }
     
    fn set_water(&mut self, water: f32) -> Result<(), DropError> {
        if water < 0.0 {
            Err(DropError::InvalidValue("Water cannot be negative"))
        } else {
            match self {
                Drop::Alive { water: w, .. } => {
                    *w = water;
                    Ok(())
                },
                Drop::Dead => Err(DropError::DropIsDead)
            }
        }
    }

    fn get_water(&self) -> Result<f32, DropError> {
        match self {
            Drop::Alive { water, .. } => Ok(*water),
            Drop::Dead => Err(DropError::DropIsDead)
        }
    }

    fn set_sediment(&mut self, sediment: f32) -> Result<(), DropError> {
        if sediment < 0.0 {
            Err(DropError::InvalidValue("Sediment cannot be negative"))
        } else {
            match self {
                Drop::Alive { sediment: s, .. } => {
                    *s = sediment;
                    Ok(())
                },
                Drop::Dead => Err(DropError::DropIsDead)
            }
        }
    }
    
    fn get_sediment(&self) -> Result<f32, DropError> {
        match self {
            Drop::Alive { sediment, .. } => Ok(*sediment),
            Drop::Dead => Err(DropError::DropIsDead)
        }
    }
    
    fn set_dead(&mut self) -> Result<(), DropError> {
        match self {
            Drop::Alive { .. } => {
                *self = Drop::Dead;
                Ok(())
            },
            Drop::Dead => Err(DropError::DropIsDead)
        }
    }

    fn should_die(&self) -> Result<bool, DropError> {
        match self {
            Drop::Alive { .. } => Ok(self.get_water()? < P_MIN_WATER || self.get_speed()? < P_MIN_SPEED),
            Drop::Dead => Err(DropError::DropIsDead)
        }
    }

    fn usize_position(&self) -> Result<(usize, usize), DropError> {
        match self {
            Drop::Alive { position, .. } => {
                let x = (position.x).round() as i32;
                let y = (position.y).round() as i32;

                if let (Some(x), Some(y)) = (x.try_into().ok(), y.try_into().ok())  {
                    Ok((x, y))
                } else {
                    Err(DropError::InvalidPosition("usize_position", *position))
                }
            },
            Drop::Dead => Err(DropError::DropIsDead)
        }
    }

    fn gradient(&mut self, heightmap: &Heightmap) -> Result<Vector2, DropError> {
        let (mut ix, mut iy) = self.usize_position()?;
        if ix >= heightmap.width - 1 {
            ix -= 1;
        }
        if iy >= heightmap.height - 1 {
            iy -= 1;
        }
        
        match self {
            Drop::Alive { position, .. } => {
                let fx = position.x;
                let fy = position.y;
                
                let p_x0_y0 = heightmap.data.get(ix + 0).and_then(|v| v.get(iy + 0)).ok_or(DropError::InvalidPosition("gradient x0 y0", *position))?;
                let p_x1_y0 = heightmap.data.get(ix + 1).and_then(|v| v.get(iy + 0)).ok_or(DropError::InvalidPosition("gradient x1 y0", *position))?;
                let p_x0_y1 = heightmap.data.get(ix + 0).and_then(|v| v.get(iy + 1)).ok_or(DropError::InvalidPosition("gradient x0 y1", *position))?;
                let p_x1_y1 = heightmap.data.get(ix + 1).and_then(|v| v.get(iy + 1)).ok_or(DropError::InvalidPosition("gradient x1 y1", *position))?;

                let v = fx - fx.floor();
                let u = fy - fy.floor();

                let x0 = (p_x1_y0 - p_x0_y0) * (1.0 - v) + (p_x1_y1 - p_x0_y1) * v;
                let x1 = (p_x0_y1 - p_x0_y0) * (1.0 - u) + (p_x1_y1 - p_x1_y0) * u;
                
                Ok(Vector2::new(x0, x1))
            },
            Drop::Dead => Err(DropError::DropIsDead)
        }
        
    }

    fn update_direction(&mut self, gradient: &Vector2, random_angle: f32) -> Result<(), DropError> {
        match self {
            Drop::Alive { direction, .. } => {
                let x_dir = direction.x;
                let y_dir = direction.y;
                
                direction.set_x(x_dir * P_INERTIA - gradient.x * (1.0 - P_INERTIA));
                direction.set_y(y_dir * P_INERTIA - gradient.y * (1.0 - P_INERTIA));
                
                // Check if direction is zero vector
                if direction.x == 0.0 && direction.y == 0.0 {
                    direction.set_x(random_angle.cos());
                    direction.set_y(random_angle.sin());  
                }
                Ok(())
            },
            Drop::Dead => Err(DropError::DropIsDead)
        }
    }
    
    fn update_position(&mut self) -> Result<(), DropError> {
        match self {
            Drop::Alive { position, direction, .. } => {
                position.set_x(position.x + direction.x);
                position.set_y(position.y + direction.y);
                Ok(())
            },
            Drop::Dead => Err(DropError::DropIsDead)
        }
    }
    
    fn update_water(&mut self) -> Result<(), DropError> {
        match self {
            Drop::Alive { water, .. } => {
                *water *= 1.0 - P_EVAPORATION;
                Ok(())
            },
            Drop::Dead => Err(DropError::DropIsDead)
        }
    }

    fn calculate_capacity(&self, height_delta: &f32) -> Result<f32, DropError> {
        if let Drop::Alive { speed, water, .. } = self {
            let capacity = speed * *water * P_CAPACITY * P_MIN_SLOPE.max(-*height_delta);
            if capacity < 0.0 {
                Err(DropError::InvalidValue("Capacity cannot be negative"))
            } else {
                Ok(capacity)
            }
        } else {
            Err(DropError::DropIsDead)
        }
    }

    fn update_speed(&mut self, height_delta: &f32) -> Result<(), DropError> {
        match self {
            Drop::Alive { speed, .. } => {
                let new_speed = ((*speed).powi(2) + *height_delta * P_GRAVITY).max(0.0).sqrt();
                if new_speed < 0.0 {
                    Err(DropError::InvalidValue("Speed cannot be negative"))
                } else {
                    *speed = new_speed;
                    Ok(())
                }
            },
            Drop::Dead => Err(DropError::DropIsDead)
        }
    }
    
}

fn create_drop(heightmap: &Heightmap, rng: &mut ThreadRng) -> Result<Drop, DropError> {
    let x = rng.gen::<HeightmapPrecision>() * heightmap.width as HeightmapPrecision;
    let y = rng.gen::<HeightmapPrecision>() * heightmap.height as HeightmapPrecision;
    
    let direction: f32 = rng.gen::<f32>() * std::f32::consts::PI * 2.0;
    
    let mut drop = Drop::new();
    drop.set_position(Vector2::new(x, y))?;
    drop.set_direction(Vector2::new(direction.cos(), direction.sin()))?;
    drop.set_speed(0.0)?;
    drop.set_water(1.0)?;
    drop.set_sediment(0.0)?;
    Ok(drop)
}

fn kill_drop(drop: &mut Drop, heightmap: &mut Heightmap, starting_ix: usize, starting_iy: usize) -> Result<(), DropError> {
    let sediment = drop.get_sediment()?;
    let height = heightmap.get(starting_ix, starting_iy).unwrap();
    heightmap.set(starting_ix, starting_iy, height + sediment).unwrap();
    drop.set_dead()?;
    Ok(())
}

fn tick(heightmap: &mut Heightmap, drop: &mut Drop, rng: &mut ThreadRng) -> Result<(), DropError> {
    let (ix, iy) = drop.usize_position()?;
    if ix >= heightmap.width || iy >= heightmap.height {
        drop.set_dead()?;
        return Ok(());
    }

    let gradient = drop.gradient(heightmap)?;
    let random_angle: f32 = rng.gen::<f32>() * std::f32::consts::PI * 2.0;
    drop.update_direction(&gradient, random_angle)?;

    let height_old = heightmap.get(ix, iy).ok_or(DropError::InvalidPosition("tick: height_old", drop.get_position()?))?; // TODO: Add interpolated height
    drop.update_position()?;

    let (ix_new, iy_new) = if let Ok((ix, iy)) = drop.usize_position() {
        (ix, iy)
    } else {
        kill_drop(drop, heightmap, ix, iy)?;
        return Ok(());
    };

    if ix_new >= heightmap.width || iy_new >= heightmap.height {
        kill_drop(drop, heightmap, ix, iy)?;
        return Ok(());
    }
        
    let height_new = heightmap.get(ix_new, iy_new).ok_or(DropError::InvalidPosition("tick: height_new", drop.get_position()?))?; // TODO: Add interpolated height


    let height_delta = height_new - height_old;
    if height_delta > P_MIN_SLOPE {
        let drop_sediment = drop.get_sediment()?;
        let sediment = height_delta.min(drop_sediment);
        heightmap.set(ix, iy, height_old + sediment).unwrap();
        drop.set_sediment(drop_sediment - sediment)?;
    } else {
        let c = drop.calculate_capacity(&height_delta)?;
        let sediment = drop.get_sediment()?;

        if c < sediment {
            let deposit = (sediment - c) * P_DEPOSITION;
            heightmap.set(ix, iy, height_old + deposit).unwrap();
            drop.set_sediment(sediment - deposit)?;
        } else {
            // We need to make sure height_delta is <= 0.0 or we will 
            // get negative erosion if P_MIN_SLOPE is set above 0.0.
            let erosion = (-height_delta.min(0.0)).min((c - sediment) * P_EROSION);
            heightmap.set(ix, iy, height_old - erosion).unwrap();
            drop.set_sediment(sediment + erosion)?;
        }
    }
    drop.update_speed(&height_delta)?;
    drop.update_water()?;
   
    // println!("Running test...");
    // let height_test = heightmap.get(ix, iy).unwrap() * 0.99;
    // heightmap.set(ix, iy, height_test).unwrap();

    if drop.should_die().unwrap() {
        kill_drop(drop, heightmap, ix, iy)?;
    }

    Ok(())
}

pub fn erode(heightmap: &Heightmap) -> Heightmap {
    let mut heightmap = heightmap.clone();
    let mut rng = rand::thread_rng();
    
    let mut bar = progress::Bar::new();
    bar.set_job_title("Eroding...");

    let mut killed = 0;
    
    for i in 0..DROPLETS {
        let mut drop = match create_drop(&heightmap, &mut rng) {
            Ok(drop) => drop,
            Err(e) => {
                println!("Error while creating drop: {:?}", e);
                break;
            }
        };
        let mut steps = 0;
        
        while let Drop::Alive{..} = drop {
            let result = tick(&mut heightmap, &mut drop, &mut rng);
            if let Err(e) = result {
                println!("Error during tick: {:?}", e);
                break;
            }

            steps += 1;
            if steps > P_MAX_PATH {
                drop.set_dead().unwrap();
                killed += 1;
                break;
            }
        };
        
        if i % 100 == 0 {
            bar.reach_percent((((i+1) as f32 / DROPLETS as f32) * 100.0).round() as i32);
        } else if i == DROPLETS - 1 {
            bar.reach_percent(100);
        }
    }

    println!("\nKilled: {} / {}", killed, DROPLETS);

    heightmap
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_drop() -> Drop {
        Drop::Alive {
            position: Vector2::new(0.8, 2.5),
            speed: 1.0,
            water: 1.0,
            sediment: 0.0,
            direction: Vector2::new(1.0, 0.0)
        }
    }
    
    #[test]
    fn test_drop_usize_position() {
        let drop = create_drop();
        let usize_position = (1usize, 3usize);
        assert_eq!(drop.usize_position().unwrap(), usize_position);

        let drop = Drop::Dead;
        assert_eq!(drop.usize_position(), Err(DropError::DropIsDead));
    }

    #[test]
    fn test_drop_evaporation() {
        let water = 1.0;
        let mut drop = create_drop();
        drop.set_water(water).unwrap();

        drop.update_water().unwrap();
        assert_eq!(drop.get_water().unwrap(), water * (1.0 - P_EVAPORATION));

        drop.update_water().unwrap();
        assert_eq!(drop.get_water().unwrap(), water * (1.0 - P_EVAPORATION).powi(2));

        drop.update_water().unwrap();
        assert_eq!(drop.get_water().unwrap(), water * (1.0 - P_EVAPORATION).powi(3));
    }

    fn test_drop_set_get_dead() {
        let mut drop = create_drop();
        assert_ne!(drop, Drop::Dead);
        drop.set_dead().unwrap();
        assert_eq!(drop, Drop::Dead);
    }

    fn test_drop_set_get_sediment(sediment: f32) {
        let mut drop = create_drop();
        drop.set_sediment(sediment).unwrap();
        assert_eq!(drop.get_sediment().unwrap(), sediment);
    }

    fn test_drop_set_get_water(water: f32) {
        let mut drop = create_drop();
        drop.set_water(water).unwrap();
        assert_eq!(drop.get_water().unwrap(), water);
    }

    fn test_drop_set_get_speed(speed: f32) {
        let mut drop = create_drop();
        drop.set_speed(speed).unwrap();
        assert_eq!(drop.get_speed().unwrap(), speed);
    }

    fn test_drop_set_get_direction(direction_: Vector2) {
        let mut drop = create_drop();
        drop.set_direction(direction_).unwrap();
        if let Drop::Alive{direction, ..} = drop {
            assert_eq!(direction, direction_);
        } else {
            panic!("Drop is dead");
        }
    }

    fn test_drop_set_get_position(position: Vector2) {
        let mut drop = create_drop();
        drop.set_position(position).unwrap();
        assert_eq!(drop.get_position().unwrap(), position);
    }

    #[test]
    fn test_drop_setters_getters() {
        let sediment = 1.0;
        let water = 1.0;
        let speed = 1.0;
        let direction = Vector2::new(1.0, 0.0);
        let position = Vector2::new(0.8, 2.5);
        
        test_drop_set_get_sediment(sediment);
        test_drop_set_get_water(water);
        test_drop_set_get_speed(speed);
        test_drop_set_get_direction(direction);
        test_drop_set_get_position(position);
        test_drop_set_get_dead();
    }


   
}
