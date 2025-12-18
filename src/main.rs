mod archive;
mod collector;
mod tui;

use std::{env, error::Error, path::PathBuf};

use crate::tui::app::App;

fn main() -> Result<(), Box<dyn Error>> {
    let input = env::args().nth(1).map_or_else(
        || ".".to_string(),
        |mut input| {
            if input.ends_with('/') {
                input.remove(input.len() - 1);
            }
            input
        },
    );

    if input == "-h" || input == "--help" {
        println!("Usage:");
        println!("cineteca [path/to/scan] (defaults to current dir)");
        return Ok(());
    }

    let path = PathBuf::from(input);

    if !path.exists() {
        return Err("Provided path does not exist".into());
    }

    App::new(path).run();

    Ok(())
}
