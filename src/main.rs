mod app;
mod movies;

use std::{env, error::Error};

use app::App;

fn main() -> Result<(), Box<dyn Error>> {
    let path = env::args().nth(1).map_or_else(
        || ".".to_string(),
        |mut path| {
            if path.ends_with('/') {
                path.remove(path.len() - 1);
            }

            path
        },
    );

    App::new(&path).run()?;

    Ok(())
}
