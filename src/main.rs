mod app;
mod movies;

use std::{env, error::Error};

use app::App;

fn main() -> Result<(), Box<dyn Error>> {
    let path = match env::args().nth(1) {
        Some(mut path) => {
            if path.ends_with('/') {
                path.remove(path.len() - 1);
            }

            path
        }
        None => ".".to_string(),
    };

    App::new(&path).run()?;

    Ok(())
}
