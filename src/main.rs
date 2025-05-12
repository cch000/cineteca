mod app;
mod movies;

use std::{env, error::Error};

use app::App;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    let path = match args.len() > 1 {
        true => &args[1],
        false => ".",
    };

    (App::new(path)?).run()?;

    Ok(())
}
