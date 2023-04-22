use crate::heightmap;

pub fn erode(heightmap: &heightmap::Heightmap) -> heightmap::Heightmap {
    let mut data = heightmap.data.clone();
    
    for i in 0..heightmap.width {
        for j in 0..heightmap.height {
            if i - j < 20 {
                data[i][j] = 0.0f32;
            }
        }
    }

    heightmap::Heightmap::new(data, heightmap.width, heightmap.height, heightmap.depth)
}
