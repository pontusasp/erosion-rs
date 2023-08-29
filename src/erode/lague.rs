use crate::heightmap::*;
use rand::prelude::*;
use crate::math::Vector2;

#[derive(Clone, Copy, Debug)]
pub struct Parameters {
    pub erosion_radius: usize, // [2, 8], 3
    pub inertia: f32, // [0, 1], 0.05
    pub sediment_capacity_factor: f32, // 4
    pub min_sediment_capacity: f32, // 0.01
    pub erode_speed: f32, // [0, 1], 0.3
    pub deposit_speed: f32, // [0, 1], 0.3
    pub evaporate_speed: f32, // [0, 1], 0.1
    pub gravity: f32, // 4
    pub max_droplet_lifetime: usize, // 30
    pub initial_water_volume: f32, // 1
    pub initial_speed: f32, // 1
    pub num_iterations: usize, // 1
}

impl Default for Parameters {
    fn default() -> Self {
        Parameters {
            erosion_radius: 3,
            inertia: 0.05,
            sediment_capacity_factor: 4.0,
            min_sediment_capacity: 0.01,
            erode_speed: 0.3,
            deposit_speed: 0.3,
            evaporate_speed: 0.1,
            gravity: 4.0,
            max_droplet_lifetime: 30,
            initial_water_volume: 1.0,
            initial_speed: 1.0,
            num_iterations: 1,
        }
    }
}

pub struct DropZone {
    min: Vector2,
    max: Vector2,
    validate: Option<fn(Vector2) -> bool>
}

impl DropZone {
    pub fn default(heightmap: &Heightmap) -> Self {
        DropZone {
            min: Vector2 { x: 0.0, y: 0.0 },
            max: Vector2 { x: heightmap.width as f32 - 1.0, y: heightmap.height as f32 - 1.0 },
            validate: None,
        }
    }
}

pub struct State {
    params: Parameters,
    current_map_size: usize,
    current_erosion_radius: usize,
    erosion_brush_indices: Vec<Vec<i32>>,
    erosion_brush_weights: Vec<Vec<f32>>,
    rng: rand::rngs::ThreadRng,
}

impl State {
    fn random_in_range(&mut self, min: f32, max: f32) -> f32 {
        self.rng.gen::<f32>() * (max - min) + min
    }
}

fn index_to_position(index: usize, width: usize) -> (usize, usize) {
    (index % width, index / width)
}

pub fn erode(heightmap: &mut Heightmap, params: &Parameters, drop_zone: DropZone) {
    let mut state = State {
        params: *params,
        current_map_size: 0,
        current_erosion_radius: 0,
        erosion_brush_indices: vec![],
        erosion_brush_weights: vec![],
        rng: rand::thread_rng(),
    };

    initialize(&mut state, heightmap.width);
    add_metadata(&mut state, heightmap);

    for _iteration in 0..params.num_iterations {
        let mut pos_x = state.random_in_range(0.0, heightmap.width as f32 - 1.0);
        let mut pos_y = state.random_in_range(0.0, heightmap.height as f32 - 1.0);
        if let Some(validate) = drop_zone.validate {
            while !validate(Vector2 { x: pos_x, y: pos_y }) {
                pos_x = state.random_in_range(0.0, heightmap.width as f32 - 1.0);
                pos_y = state.random_in_range(0.0, heightmap.height as f32 - 1.0);
            }
        }
        let mut dir_x = 0.0;
        let mut dir_y = 0.0;
        let mut speed = state.params.initial_speed;
        let mut water = state.params.initial_water_volume;
        let mut sediment = 0.0;

        for _lifetime in 0..params.max_droplet_lifetime {
            let node_x = pos_x.floor() as usize;
            let node_y = pos_y.floor() as usize;
            let droplet_index = node_y * heightmap.width + node_x;

            let cell_offset_x = pos_x - node_x as f32;
            let cell_offset_y = pos_y - node_y as f32;

            let height_and_gradient = calculate_height_and_gradient(heightmap, pos_x, pos_y);

            dir_x = dir_x * state.params.inertia - height_and_gradient.gradient_x * (1.0 - state.params.inertia);
            dir_y = dir_y * state.params.inertia - height_and_gradient.gradient_y * (1.0 - state.params.inertia);

            let len = (dir_x * dir_x + dir_y * dir_y).sqrt();
            if len != 0.0 {
                dir_x /= len;
                dir_y /= len;
            }
            pos_x += dir_x;
            pos_y += dir_y;

            if (dir_x == 0.0 && dir_y == 0.0) || pos_x < 0.0 || pos_x >= heightmap.width as f32 - 1.0 || pos_y < 0.0 || pos_y >= heightmap.height as f32 - 1.0 {
                break;
            }

            let new_height = calculate_height_and_gradient(heightmap, pos_x, pos_y).height;
            let delta_height = new_height - height_and_gradient.height;

            let sediment_capacity = (-delta_height * speed * water * state.params.sediment_capacity_factor).max(state.params.min_sediment_capacity);

            if sediment > sediment_capacity || delta_height > 0.0 {
                let amount_to_deposit = if delta_height > 0.0 {
                    delta_height.min(sediment)
                } else {
                    (sediment - sediment_capacity) * state.params.deposit_speed
                };
                sediment -= amount_to_deposit;

                heightmap.data[node_x][node_y] += amount_to_deposit * (1.0 - cell_offset_x) * (1.0 - cell_offset_y);
                heightmap.data[node_x + 1][node_y] += amount_to_deposit * cell_offset_x * (1.0 - cell_offset_y);
                heightmap.data[node_x][node_y + 1] += amount_to_deposit * (1.0 - cell_offset_x) * cell_offset_y;
                heightmap.data[node_x + 1][node_y + 1] += amount_to_deposit * cell_offset_x * cell_offset_y;
            } else {
                let amount_to_erode = ((sediment_capacity - sediment) * state.params.erode_speed).min(-delta_height);

                for brush_point_index in 0..state.erosion_brush_indices[droplet_index].len() {
                    let node_index = state.erosion_brush_indices[droplet_index][brush_point_index];
                    let (node_x, node_y) = index_to_position(node_index as usize, heightmap.width);
                    let weighted_erode_amount = amount_to_erode * state.erosion_brush_weights[droplet_index][brush_point_index];
                    let delta_sediment = heightmap.data[node_x][node_y].min(weighted_erode_amount);
                    heightmap.data[node_x][node_y] -= delta_sediment;
                    sediment += delta_sediment;
                }
            }


            speed = (speed * speed + delta_height * state.params.gravity).sqrt();
            water *= 1.0 - state.params.evaporate_speed;
        }
    }
}

fn initialize(state: &mut State, map_size: usize) {
    state.current_map_size = map_size;

    if state.erosion_brush_indices.is_empty() || state.current_erosion_radius != state.params.erosion_radius || state.current_map_size != map_size {
        initialize_brush_indices(state, map_size, state.params.erosion_radius);
        state.current_erosion_radius = state.params.erosion_radius;
        state.current_map_size = map_size;
    }

}

fn calculate_height_and_gradient(heightmap: &Heightmap, pos_x: f32, pos_y: f32) -> HeightAndGradient {
    let coord_x = pos_x as usize;
    let coord_y = pos_y as usize;

    let x = pos_x - coord_x as f32;
    let y = pos_y - coord_y as f32;

    let height_nw = heightmap.data[coord_x + 0][coord_y + 0];
    let height_ne = heightmap.data[coord_x + 1][coord_y + 0];
    let height_sw = heightmap.data[coord_x + 0][coord_y + 1];
    let height_se = heightmap.data[coord_x + 1][coord_y + 1];

    let gradient_x = (height_ne - height_nw) * (1.0 - y) + (height_se - height_sw) * y;
    let gradient_y = (height_sw - height_nw) * (1.0 - x) + (height_se - height_ne) * x;

    let height = height_nw * (1.0 - x) * (1.0 - y) + height_ne * x * (1.0 - y) + height_sw * (1.0 - x) * y + height_se * x * y;

    HeightAndGradient {
        height,
        gradient_x,
        gradient_y
    }
}

fn initialize_brush_indices(state: &mut State, map_size: usize, radius: usize) {
    let radius: i32 = radius.try_into().unwrap();

    let erosion_brush_indices_size = map_size * map_size;
    let mut x_offsets: Vec<i32> = vec![];
    let mut y_offsets: Vec<i32> = vec![];
    let mut weights: Vec<f32> = vec![];
    state.erosion_brush_indices.resize(erosion_brush_indices_size, vec![]);
    state.erosion_brush_weights.resize(erosion_brush_indices_size, vec![]);
    x_offsets.resize((radius as usize).pow(2) * 4, 0);
    y_offsets.resize((radius as usize).pow(2) * 4, 0);
    weights.resize((radius as usize).pow(2) * 4, 0.0);
    let mut weight_sum = 0.0f32;
    let mut add_index = 0;

    for i in 0..erosion_brush_indices_size {
        let centre_x = i % map_size;
        let centre_y = i / map_size;

        if centre_y as i32 <= radius || centre_y as i32 >= map_size as i32 - radius || centre_x as i32 <= radius + 1 || centre_x as i32 >= map_size as i32 - radius {
            weight_sum = 0.0;
            add_index = 0;
            for y in -radius..=radius {
                for x in -radius..=radius {
                    let sqr_dst: f32 = (x as f32).powi(2) + (y as f32).powi(2);
                    if sqr_dst < (radius * radius) as f32 {
                        let coord_x = centre_x as i32 + x;
                        let coord_y = centre_y as i32 + y;

                        if coord_x >= 0 && coord_x < map_size as i32 && coord_y >= 0 && coord_y < map_size as i32 {
                            let weight = 1.0 - sqr_dst.sqrt() / radius as f32;
                            weight_sum += weight;
                            weights[add_index] = weight;
                            x_offsets[add_index] = x;
                            y_offsets[add_index] = y;
                            add_index += 1;
                        }
                    }
                }
            }
        }
    
        let num_entries = add_index;
        state.erosion_brush_indices[i] = vec![];
        state.erosion_brush_weights[i] = vec![];
        state.erosion_brush_indices[i].resize(num_entries, 0);
        state.erosion_brush_weights[i].resize(num_entries, 0.0);

        for j in 0..num_entries {
            state.erosion_brush_indices[i][j] = (y_offsets[j] + centre_y as i32) * map_size as i32 + x_offsets[j] + centre_x as i32;
            state.erosion_brush_weights[i][j] = weights[j] / weight_sum;
        }

    }

    assert_eq!(state.erosion_brush_indices.len(), erosion_brush_indices_size);
    assert_eq!(state.erosion_brush_weights.len(), erosion_brush_indices_size);
    assert_eq!(x_offsets.len(), radius as usize * radius as usize * 4);
    assert_eq!(y_offsets.len(), radius as usize * radius as usize * 4);
    assert_eq!(weights.len(), radius as usize * radius as usize * 4);

}

struct HeightAndGradient {
    height: f32,
    gradient_x: f32,
    gradient_y: f32,
}

pub fn add_metadata(state: &State, heightmap: &mut Heightmap) {
    heightmap.metadata_add("EROSION_TYPE", "LAGUE".to_string());
    heightmap.metadata_add("EROSION_RADIUS", state.params.erosion_radius.to_string());
    heightmap.metadata_add("INERTIA", state.params.inertia.to_string());
    heightmap.metadata_add("SEDIMENT_CAPACITY_FACTOR", state.params.sediment_capacity_factor.to_string());
    heightmap.metadata_add("MIN_SEDIMENT_CAPACITY", state.params.min_sediment_capacity.to_string());
    heightmap.metadata_add("ERODE_SPEED", state.params.erode_speed.to_string());
    heightmap.metadata_add("DEPOSIT_SPEED", state.params.deposit_speed.to_string());
    heightmap.metadata_add("EVAPORATE_SPEED", state.params.evaporate_speed.to_string());
    heightmap.metadata_add("GRAVITY", state.params.gravity.to_string());
    heightmap.metadata_add("MAX_DROPLET_LIFETIME", state.params.max_droplet_lifetime.to_string());
    heightmap.metadata_add("INITIAL_WATER_VOLUME", state.params.initial_water_volume.to_string());
    heightmap.metadata_add("INITIAL_SPEED", state.params.initial_speed.to_string());
    heightmap.metadata_add("NUM_ITERATIONS", state.params.num_iterations.to_string());
}
