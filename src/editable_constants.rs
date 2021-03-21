use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Deserialize, Serialize)]
struct Constants{
    fade_to_white_speed: f32            // per second ( 1 fully white, 10 moves to white in tenth of sec starting from black)
}

fn read_constants( ) {
    let mut path_buf = env::current_dir().unwrap();
    path_buf = path_buf.join("constants.json");

}

