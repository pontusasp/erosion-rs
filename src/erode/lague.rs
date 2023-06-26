use crate::heightmap::*;

#[derive(Clone, Copy, Debug)]
pub struct Parameters {
    erosion_radius: usize, // [2, 8], 3
    inertia: f32, // [0, 1], 0.05
    sediment_capacity_factor: f32, // 4
    min_sediment_capacity: f32, // 0.01
    erode_speed: f32, // [0, 1], 0.3
    deposit_speed: f32, // [0, 1], 0.3
    evaporate_speed: f32, // [0, 1], 0.1
    gravity: f32, // 4
    max_droplet_lifetime: usize, // 30
    initial_water_volume: f32, // 1
    initial_speed: f32, // 1
    num_interations: usize, // 1
}

pub const DEFAULT_PARAMS: Parameters = Parameters {
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
    num_interations: 1,
};

struct State {
    params: Parameters,
    current_map_size: usize,
    current_erosion_radius: usize,
    erosion_brush_indices: Vec<Vec<i32>>,
    erosion_brush_weights: Vec<Vec<f32>>,
}

pub fn erode(heightmap: &Heightmap, params: &Parameters) {
    let mut state = State {
        params: *params,
        current_map_size: 0,
        current_erosion_radius: 0,
        erosion_brush_indices: vec![],
        erosion_brush_weights: vec![],
    };

    initialize(&mut state, heightmap.width);

    

    todo!();
}

fn initialize(state: &mut State, map_size: usize) {
    state.current_map_size = map_size;

    if state.erosion_brush_indices.is_empty() || state.current_erosion_radius != state.params.erosion_radius || state.current_map_size != map_size {
        initialize_brush_indices(state, map_size, state.params.erosion_radius);
        state.current_erosion_radius = state.params.erosion_radius;
        state.current_map_size = map_size;
    }

}

fn calculate_height_and_gradient(state: &State, heightmap: &Heightmap, map_size: usize, pos_x: f32, pos_y: f32) -> HeightAndGradient {
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
    let mut weight_sum = 0.0f32;
    let mut add_index = 0;

    for i in 0..erosion_brush_indices_size {
        let centre_x = i % map_size;
        let centre_y = i / map_size;

        if centre_y as i32 <= radius || centre_y as i32 >= map_size as i32 - radius || centre_x as i32 <= radius + 1 || centre_x as i32 >= map_size as i32 - radius {
            weight_sum = 0.0;
            add_index = 0;
            for y in -radius..radius {
                for x in -radius..radius {
                    let sqr_dst: f32 = (x as f32).powi(2) + (y as f32).powi(2);
                    if sqr_dst < (radius * radius) as f32 {
                        let coord_x = centre_x as i32 + x;
                        let coord_y = centre_y as i32 + y;

                        if coord_x >= 0 && coord_x < map_size as i32 && coord_y >= 0 && coord_y < map_size as i32 {
                            let weight = 1.0 - sqr_dst.sqrt() / radius as f32;
                            weight_sum += weight;
                            weights.push(weight);
                            x_offsets.push(x);
                            y_offsets.push(y);
                            add_index += 1;
                        }
                    }
                }
            }
        }
    
        let num_entries = add_index;
        state.erosion_brush_indices.push(vec![]);
        state.erosion_brush_weights.push(vec![]);

        for j in 0..num_entries {
            state.erosion_brush_indices[i].push((y_offsets[j] + centre_y as i32) * map_size as i32 + x_offsets[j] + centre_x as i32);
            state.erosion_brush_weights[i].push(weights[j] / weight_sum);
        }

    }

}

struct HeightAndGradient {
    height: f32,
    gradient_x: f32,
    gradient_y: f32,
}
