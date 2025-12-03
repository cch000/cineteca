mod app;
mod archive;
mod collector;

use std::{env, path::PathBuf};

use app::App;

fn main() {
    let input = env::args().nth(1).map_or_else(
        || ".".to_string(),
        |mut input| {
            if input.ends_with('/') {
                input.remove(input.len() - 1);
            }
            input
        },
    );

    App::new(PathBuf::from(input)).run();
}
