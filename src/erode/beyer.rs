use crate::heightmap::*;
use crate::math::*;
use rand::prelude::*;
use rand::thread_rng;

pub const DROPLETS: usize = 1_000;
pub const P_INERTIA: f32 = 0.9;
pub const P_CAPACITY: f32 = 8.0;
pub const P_DEPOSITION: f32 = 0.05;
pub const P_EROSION: f32 = 0.9;
pub const P_EVAPORATION: f32 = 0.05;
pub const P_RADIUS: usize = 3;
pub const P_MIN_SLOPE: f32 = 0.00000001;
pub const P_GRAVITY: f32 = 9.2;
pub const P_MAX_PATH: usize = 10000;

pub const P_MIN_WATER: f32 = 0.001;
pub const P_MIN_SPEED: f32 = 0.001;

#[derive(Debug, Clone, PartialEq)]
pub enum Drop {
    Alive {
        position: Vector2,
        direction: Vector2,
        speed: f32,
        water: f32,
        sediment: f32,
    },
    Dead,
}

#[derive(Debug, PartialEq)]
pub enum DropError {
    DropIsDead,
    InvalidValue(String),
    InvalidPosition(String, Vector2),
}

impl Drop {
    pub fn new() -> Drop {
        Drop::Alive {
            position: Vector2::new(0.0, 0.0),
            direction: Vector2::new(0.0, 0.0),
            speed: 0.0,
            water: 0.0,
            sediment: 0.0,
        }
    }

    pub fn set_position(&mut self, position: Vector2) -> Result<(), DropError> {
        if let Drop::Alive { position: p, .. } = self {
            *p = position;
            Ok(())
        } else {
            Err(DropError::DropIsDead)
        }
    }

    pub fn get_position(&self) -> Result<Vector2, DropError> {
        match self {
            Drop::Alive { position, .. } => Ok(*position),
            Drop::Dead => Err(DropError::DropIsDead),
        }
    }

    pub fn set_direction(&mut self, direction: Vector2) -> Result<(), DropError> {
        match self {
            Drop::Alive { direction: d, .. } => {
                *d = direction;
                Ok(())
            }
            Drop::Dead => Err(DropError::DropIsDead),
        }
    }

    pub fn set_speed(&mut self, speed: f32) -> Result<(), DropError> {
        if speed < 0.0 {
            Err(DropError::InvalidValue(
                "Speed cannot be negative".to_string(),
            ))
        } else {
            match self {
                Drop::Alive { speed: s, .. } => {
                    *s = speed;
                    Ok(())
                }
                Drop::Dead => Err(DropError::DropIsDead),
            }
        }
    }

    pub fn get_speed(&self) -> Result<f32, DropError> {
        match self {
            Drop::Alive { speed, .. } => Ok(*speed),
            Drop::Dead => Err(DropError::DropIsDead),
        }
    }

    pub fn set_water(&mut self, water: f32) -> Result<(), DropError> {
        if water < 0.0 {
            Err(DropError::InvalidValue(
                "Water cannot be negative".to_string(),
            ))
        } else {
            match self {
                Drop::Alive { water: w, .. } => {
                    *w = water;
                    Ok(())
                }
                Drop::Dead => Err(DropError::DropIsDead),
            }
        }
    }

    pub fn get_water(&self) -> Result<f32, DropError> {
        match self {
            Drop::Alive { water, .. } => Ok(*water),
            Drop::Dead => Err(DropError::DropIsDead),
        }
    }

    pub fn set_sediment(&mut self, sediment: f32) -> Result<(), DropError> {
        if sediment < 0.0 {
            Err(DropError::InvalidValue(
                "Sediment cannot be negative".to_string(),
            ))
        } else {
            match self {
                Drop::Alive { sediment: s, .. } => {
                    *s = sediment;
                    Ok(())
                }
                Drop::Dead => Err(DropError::DropIsDead),
            }
        }
    }

    pub fn get_sediment(&self) -> Result<f32, DropError> {
        match self {
            Drop::Alive { sediment, .. } => Ok(*sediment),
            Drop::Dead => Err(DropError::DropIsDead),
        }
    }

    pub fn set_dead(&mut self) -> Result<(), DropError> {
        match self {
            Drop::Alive { .. } => {
                *self = Drop::Dead;
                Ok(())
            }
            Drop::Dead => Err(DropError::DropIsDead),
        }
    }

    pub fn get_angle(&self) -> Result<f32, DropError> {
        match self {
            Drop::Alive { direction, .. } => Ok(direction.y.atan2(direction.x)),
            Drop::Dead => Err(DropError::DropIsDead),
        }
    }

    pub fn get_capacity(&self, height_delta: HeightmapPrecision) -> Result<f32, DropError> {
        match self {
            Drop::Alive { speed, water, .. } => {
                let capacity = P_MIN_SLOPE.max(-height_delta) * speed * water * P_CAPACITY;
                if capacity < 0.0 {
                    Err(DropError::InvalidValue(
                        "Capacity cannot be negative".to_string(),
                    ))
                } else {
                    Ok(capacity)
                }
            }
            Drop::Dead => Err(DropError::DropIsDead),
        }
    }

    pub fn should_die(&self) -> Result<bool, DropError> {
        match self {
            Drop::Alive { .. } => {
                Ok(self.get_water()? < P_MIN_WATER || self.get_speed()? < P_MIN_SPEED)
            }
            Drop::Dead => Err(DropError::DropIsDead),
        }
    }

    pub fn usize_position(&self) -> Result<(usize, usize), DropError> {
        match self {
            Drop::Alive { position, .. } => {
                if let Ok(usize_pos) = position.to_usize() {
                    Ok(usize_pos)
                } else {
                    Err(DropError::InvalidPosition(
                        "usize_position".to_string(),
                        *position,
                    ))
                }
            }
            Drop::Dead => Err(DropError::DropIsDead),
        }
    }

    pub fn gradient(&mut self, heightmap: &Heightmap) -> Result<Vector2, DropError> {
        // let (mut ix, mut iy) = self.usize_position()?;
        // if ix >= heightmap.width - 1 {
        //     ix -= 1;
        // }
        // if iy >= heightmap.height - 1 {
        //     iy -= 1;
        // }

        // match self {
        //     Drop::Alive { position, .. } => {
        //         let grad = heightmap.gradient(position);

        //         match grad {
        //             Ok(g) => Ok(g),
        //             Err(e) => Err(DropError::InvalidPosition(e, *position))
        //         }
        //     },
        //     Drop::Dead => Err(DropError::DropIsDead)
        // }

        todo!();
    }

    pub fn update_direction(
        &mut self,
        gradient: &Vector2,
        random_angle: f32,
    ) -> Result<(), DropError> {
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
                } else {
                    direction.normalize();
                }
                Ok(())
            }
            Drop::Dead => Err(DropError::DropIsDead),
        }
    }

    pub fn update_position(&mut self) -> Result<(), DropError> {
        match self {
            Drop::Alive {
                position,
                direction,
                ..
            } => {
                position.set_x(position.x + direction.x);
                position.set_y(position.y + direction.y);
                Ok(())
            }
            Drop::Dead => Err(DropError::DropIsDead),
        }
    }

    pub fn update_water(&mut self) -> Result<(), DropError> {
        match self {
            Drop::Alive { water, .. } => {
                *water *= 1.0 - P_EVAPORATION;
                Ok(())
            }
            Drop::Dead => Err(DropError::DropIsDead),
        }
    }

    pub fn update_speed(&mut self, height_delta: &f32) -> Result<(), DropError> {
        match self {
            Drop::Alive { speed, .. } => {
                let new_speed = ((*speed).powi(2) + *height_delta * P_GRAVITY).abs().sqrt();
                if new_speed < 0.0 || new_speed.is_nan() {
                    Err(DropError::InvalidValue(
                        "Speed cannot be negative".to_string(),
                    ))
                } else {
                    *speed = new_speed;
                    Ok(())
                }
            }
            Drop::Dead => Err(DropError::DropIsDead),
        }
    }
}

pub fn random_position(heightmap: &Heightmap, rng: &mut ThreadRng) -> Vector2 {
    let x = rng.gen::<HeightmapPrecision>() * heightmap.width as HeightmapPrecision;
    let y = rng.gen::<HeightmapPrecision>() * heightmap.height as HeightmapPrecision;
    Vector2::new(x, y)
}

pub fn create_drop(
    position: Vector2,
    random_angle: f32,
    total_angle: &mut f32,
) -> Result<Drop, DropError> {
    *total_angle += random_angle;

    let mut drop = Drop::new();
    drop.set_position(position)?;
    drop.set_direction(Vector2::new(random_angle.cos(), random_angle.sin()))?;
    drop.set_speed(0.0)?;
    drop.set_water(1.0)?;
    drop.set_sediment(0.0)?;
    Ok(drop)
}

pub fn kill_drop(
    drop: &mut Drop,
    heightmap: &mut Heightmap,
    starting_ix: usize,
    starting_iy: usize,
) -> Result<(), DropError> {
    let sediment = drop.get_sediment()?;
    let height = match heightmap.get(starting_ix, starting_iy) {
        Some(h) => h,
        None => panic!(
            "kill_drop: heightmap.get returned None at ({}, {})",
            starting_ix, starting_iy
        ), // None => return Err(DropError::InvalidPosition("kill_drop: height".to_string(), drop.get_position()?))
    };
    heightmap
        .set(starting_ix, starting_iy, height + sediment)
        .unwrap();
    drop.set_dead()?;
    Ok(())
}

pub fn get_random_angle(rng: &mut ThreadRng) -> f32 {
    rng.gen::<f32>() * std::f32::consts::PI * 2.0
}

pub fn deposit(
    drop: &mut Drop,
    heightmap: &mut Heightmap,
    position_start: Vector2,
    height_delta: HeightmapPrecision,
) -> Result<(), DropError> {
    pub fn _place(
        heightmap: &mut Heightmap,
        pos: (usize, usize),
        deposition: f32,
        height: HeightmapPrecision,
        fraction: Vector2,
    ) -> Result<(), HeightmapError> {
        heightmap.set(
            pos.0 + 0,
            pos.1 + 0,
            deposition * (1.0 - fraction.x) * (1.0 - fraction.y) + height,
        )?;
        heightmap.set(
            pos.0 + 1,
            pos.1 + 0,
            deposition * (1.0 - fraction.x) * (1.0 - fraction.y) + height,
        )?;
        heightmap.set(
            pos.0 + 0,
            pos.1 + 1,
            deposition * (1.0 - fraction.x) * (1.0 - fraction.y) + height,
        )?;
        heightmap.set(
            pos.0 + 1,
            pos.1 + 1,
            deposition * (1.0 - fraction.x) * (1.0 - fraction.y) + height,
        )?;
        Ok(())
    }

    let pos_i = position_start.to_usize().unwrap();
    let fraction = position_start - Vector2::from_usize_tuple(pos_i);
    let height = match heightmap.get(pos_i.0, pos_i.1) {
        Some(h) => h,
        None => panic!(
            "deposit: heightmap.get returned None at ({}, {})",
            pos_i.0, pos_i.1
        ), // None => return Err(DropError::InvalidPosition("deposit: height".to_string(), position_start))
    };
    let sediment = drop.get_sediment()?;
    let capacity = drop.get_capacity(height_delta)?;

    let deposition = if height_delta > P_MIN_SLOPE {
        height_delta.min(sediment)
    } else {
        (sediment - capacity) * P_DEPOSITION
    };
    drop.set_sediment(sediment - deposition)?;

    match _place(heightmap, pos_i, deposition, height, fraction) {
        // Err(HeightmapError::OutOfBounds) => panic!("deposit: heightmap.set returned OutOfBounds"),
        Err(HeightmapError::OutOfBounds) => Err(DropError::InvalidPosition(
            "deposit: heightmap.set".to_string(),
            position_start,
        )),
        Err(HeightmapError::MismatchingSize) => {
            unreachable!("deposit: heightmap.set returned MismatchingSize")
        }
        Ok(()) => Ok(()),
    }
}

pub fn erode(
    drop: &mut Drop,
    heightmap: &mut Heightmap,
    position_start: Vector2,
    height_delta: HeightmapPrecision,
) -> Result<(), DropError> {
    let pos_i = position_start.to_usize().unwrap();
    let fraction = position_start - Vector2::from_usize_tuple(pos_i);
    let height = match heightmap.get(pos_i.0, pos_i.1) {
        Some(h) => h,
        None => panic!(
            "erode: heightmap.get returned None at ({}, {})",
            pos_i.0, pos_i.1
        ), // None => return Err(DropError::InvalidPosition("erode: height".to_string(), position_start))
    };
    let sediment = drop.get_sediment()?;
    let capacity = drop.get_capacity(height_delta)?;

    let erosion = (-height_delta.min(0.0)).min((capacity - sediment) * P_EROSION);
    drop.set_sediment(sediment + erosion)?;
    //    heightmap.set(ix, iy, height_old - erosion).unwrap();

    let x0 = if pos_i.0 > P_RADIUS {
        pos_i.0 - P_RADIUS
    } else {
        0
    };
    let x1 = if pos_i.0 + P_RADIUS + 1 < heightmap.width {
        pos_i.0 + P_RADIUS + 1
    } else {
        heightmap.width
    };

    let y0 = if pos_i.1 > P_RADIUS {
        pos_i.1 - P_RADIUS
    } else {
        0
    };
    let y1 = if pos_i.1 + P_RADIUS + 1 < heightmap.height {
        pos_i.1 + P_RADIUS + 1
    } else {
        heightmap.height
    };

    //    let erosion = if height_delta > P_MIN_SLOPE {
    //        height_delta.min(sediment)
    //    } else {
    //        (sediment - capacity) * P_DEPOSITION
    //    };
    //    drop.set_sediment(sediment - deposition)?;

    let mut kernel = [[0.0; P_RADIUS * 2 + 1]; P_RADIUS * 2 + 1];
    let mut sum = 0.0;
    for ix in x0..x1 {
        for iy in y0..y1 {
            let h = match heightmap.get(ix, iy) {
                Some(h) => h,
                None => panic!("erode: heightmap.get returned None at ({}, {})", ix, iy), // None => return Err(DropError::InvalidPosition("erode: height".to_string(), position_start))
            };
            let radius = (((ix as i32 - pos_i.0 as i32).pow(2)
                + (iy as i32 - pos_i.1 as i32).pow(2)) as f32)
                .sqrt();
            if radius.is_nan() {
                panic!("erode: radius is NaN at ({}, {})", ix, iy);
            }
            let weight = P_RADIUS as f32 - radius;
            kernel[ix - x0][iy - y0] = weight;
            sum += weight;
        }
    }

    if sum == 0.0 {
        return Ok(());
    }

    for ix in x0..x1 {
        for iy in y0..y1 {
            let height = match heightmap.get(ix, iy) {
                Some(h) => h,
                None => panic!("erode: heightmap.get returned None at ({}, {})", ix, iy), // None => return Err(DropError::InvalidPosition("erode: height".to_string(), position_start))
            };
            heightmap
                .set(
                    ix,
                    iy,
                    height - erosion * kernel[ix - x0][iy - y0] / sum as HeightmapPrecision,
                )
                .unwrap();
        }
    }

    Ok(())
}

pub fn tick(
    heightmap: &mut Heightmap,
    drop: &mut Drop,
    random_angle: f32,
) -> Result<(), DropError> {
    let position_old: Vector2 = drop.get_position()?;
    let (ix_old, iy_old) = position_old.to_usize().unwrap();

    let gradient = match heightmap.interpolated_gradient(&position_old) {
        Some(gradient) => gradient,
        None => {
            return Err(DropError::InvalidPosition(
                "tick: gradient".to_string(),
                drop.get_position()?,
            ));
        }
    };

    let height_old = match heightmap.interpolated_height(&position_old) {
        Some(height) => height,
        None => {
            return Err(DropError::InvalidPosition(
                "tick: height_old".to_string(),
                position_old,
            ));
        }
    };

    drop.update_direction(&gradient, random_angle)?;

    drop.update_position()?;

    let position_new = drop.get_position()?;
    let (ix, iy) = if let Ok(integers) = position_new.to_usize() {
        integers
    } else {
        kill_drop(drop, heightmap, ix_old, iy_old)?;
        return Ok(());
    };

    if ix >= heightmap.width || iy >= heightmap.height {
        kill_drop(drop, heightmap, ix_old, iy_old)?;
        return Ok(());
    }

    let height_new = match heightmap.interpolated_height(&position_new) {
        Some(height) => height,
        None => {
            return Err(DropError::InvalidPosition(
                "tick: height_new".to_string(),
                position_new,
            ));
        }
    };

    let height_delta = height_new - height_old;

    let capacity = drop.get_capacity(height_delta)?;
    let sediment = drop.get_sediment()?;

    if height_delta > P_MIN_SLOPE && sediment > capacity {
        deposit(drop, heightmap, position_old, height_delta)?;
    } else {
        erode(drop, heightmap, position_old, height_delta)?;
    }

    drop.update_speed(&height_delta)?;
    drop.update_water()?;

    if drop.should_die().unwrap() {
        kill_drop(drop, heightmap, ix, iy)?;
    }

    Ok(())
}

pub fn simulate(heightmap: &Heightmap) -> Heightmap {
    let mut heightmap = heightmap.clone();
    let mut rng = rand::thread_rng();

    let mut bar = progress::Bar::new();
    bar.set_job_title("Eroding...");

    let mut killed = 0;
    let mut total_distance = 0.0;
    let mut total_starting_angle = 0.0;
    let mut total_ending_angle = 0.0;
    let mut total_movement = Vector2::new(0.0, 0.0);

    for i in 0..DROPLETS {
        let mut drop = match create_drop(
            random_position(&heightmap, &mut rng),
            get_random_angle(&mut rng),
            &mut total_starting_angle,
        ) {
            Ok(drop) => drop,
            Err(e) => {
                eprintln!("Error while creating drop: {:?}", e);
                break;
            }
        };
        let mut steps = 0;
        let initial_position = drop.get_position().unwrap();
        let mut last_position = initial_position.clone();
        let mut last_angle = drop.get_angle().unwrap();

        while let Drop::Alive { .. } = drop {
            last_position = drop.get_position().unwrap();
            last_angle = drop.get_angle().unwrap();
            let result = tick(&mut heightmap, &mut drop, get_random_angle(&mut rng));
            if let Err(e) = result {
                eprintln!("Error during tick: {:?}", e);
                break;
            }

            steps += 1;
            if steps > P_MAX_PATH {
                drop.set_dead().unwrap();
                killed += 1;
                break;
            }
        }
        total_ending_angle += last_angle;
        total_distance += (last_position - initial_position).magnitude();
        total_movement = total_movement + last_position - initial_position;

        if i % 10 == 0 {
            bar.reach_percent((((i + 1) as f32 / DROPLETS as f32) * 100.0).round() as i32);
        } else if i == DROPLETS - 1 {
            bar.reach_percent(100);
        }
    }

    heightmap.metadata_add("DROPLETS", DROPLETS.to_string());
    heightmap.metadata_add("P_INERTIA", P_INERTIA.to_string());
    heightmap.metadata_add("P_CAPACITY", P_CAPACITY.to_string());
    heightmap.metadata_add("P_DEPOSITION", P_DEPOSITION.to_string());
    heightmap.metadata_add("P_EROSION", P_EROSION.to_string());
    heightmap.metadata_add("P_EVAPORATION", P_EVAPORATION.to_string());
    heightmap.metadata_add("P_RADIUS", P_RADIUS.to_string());
    heightmap.metadata_add("P_MIN_SLOPE", P_MIN_SLOPE.to_string());
    heightmap.metadata_add("P_GRAVITY", P_GRAVITY.to_string());
    heightmap.metadata_add("P_MAX_PATH", P_MAX_PATH.to_string());
    heightmap.metadata_add("P_MIN_WATER", P_MIN_WATER.to_string());
    heightmap.metadata_add("P_MIN_SPEED", P_MIN_SPEED.to_string());

    heightmap.metadata_add("killed", killed.to_string());
    heightmap.metadata_add(
        "average_distance",
        (total_distance / DROPLETS as f32).to_string(),
    );
    heightmap.metadata_add(
        "average_starting_angle",
        (total_starting_angle / DROPLETS as f32 / std::f32::consts::PI * 180.0).to_string(),
    );
    heightmap.metadata_add(
        "average_ending_angle",
        (total_ending_angle / DROPLETS as f32 / std::f32::consts::PI * 180.0).to_string(),
    );
    heightmap.metadata_add(
        "average_movement",
        format!("{:?}", total_movement * (1.0 / DROPLETS as f32)),
    );

    println!("\nKilled: {} / {}", killed, DROPLETS);
    println!("Average distance: {}", total_distance / DROPLETS as f32);
    println!(
        "Average starting angle: {}",
        total_starting_angle / DROPLETS as f32 / std::f32::consts::PI * 180.0
    );
    println!(
        "Average ending angle: {}",
        total_ending_angle / DROPLETS as f32 / std::f32::consts::PI * 180.0
    );
    println!(
        "Average movement: {:?}",
        total_movement * (1.0 / DROPLETS as f32)
    );

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
            direction: Vector2::new(1.0, 0.0),
        }
    }

    #[test]
    fn test_drop_usize_position() {
        let drop = create_drop();
        let usize_position = (0usize, 2usize);
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
        assert_eq!(
            drop.get_water().unwrap(),
            water * (1.0 - P_EVAPORATION).powi(2)
        );

        drop.update_water().unwrap();
        assert_eq!(
            drop.get_water().unwrap(),
            water * (1.0 - P_EVAPORATION).powi(3)
        );
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
        if let Drop::Alive { direction, .. } = drop {
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

    #[test]
    fn test_vector2_ops() {
        assert_eq!(
            Vector2::new(1.0, 2.0) - Vector2::new(3.0, -4.0),
            Vector2::new(-2.0, 6.0)
        );
    }

    #[test]
    fn test_erosion() {
        let width = 10usize;
        let height = 10usize;
        let x = width as f32 * 0.4;
        let y = height as f32 * 0.4;

        let mut drop = Drop::new();
        drop.set_position(Vector2::new(x, y)).unwrap();
        drop.set_direction(Vector2::new(1.0, 0.0)).unwrap();
        drop.set_speed(1.0).unwrap();
        drop.set_water(1.0).unwrap();
        drop.set_sediment(0.0).unwrap();

        let mut data = Vec::new();
        let radius = ((width.pow(2) + height.pow(2)) as f32).sqrt();
        for x in 0..width {
            let mut row = Vec::new();
            for y in 0..height {
                let distance = ((x as f32 - width as f32 / 2.0).powi(2)
                    + (y as f32 - height as f32 / 2.0).powi(2))
                .sqrt();
                row.push(distance / radius);
            }
            data.push(row);
        }

        let mut heightmap = Heightmap::new(data.clone(), width, height, 1.0, 1.0);
        tick(&mut heightmap, &mut drop, 0.0).unwrap();

        assert_ne!(heightmap.data, data);
    }
}
