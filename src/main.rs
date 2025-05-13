mod app;
mod movies;

use std::{env, error::Error};

use app::App;

fn main() -> Result<(), Box<dyn Error>> {
    let path = env::args()
        .nth(1)
        .unwrap_or_else(|| String::from("."));

    App::new(&path)?.run()?;

    Ok(())
}

