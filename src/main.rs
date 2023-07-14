use macroquad::prelude::*;
use std::env;

pub mod erode;
pub mod heightmap;
pub mod math;
pub mod visualize;
pub mod partitioning;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 800;

fn window_conf() -> Conf {
    Conf {
        window_title: "Erosion RS".to_owned(),
        window_width: WIDTH.try_into().unwrap(),
        window_height: HEIGHT.try_into().unwrap(),
        window_resizable: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    env::set_var("RUST_BACKTRACE", "1");

    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        erode::run_simulation();
    } else {
        visualize::run().await;
    }
}

