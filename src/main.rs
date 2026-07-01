mod archive;
mod collector;
mod movie;
mod tui;

use std::{env, error::Error, path::PathBuf};

use crate::tui::app::App;

fn main() -> Result<(), Box<dyn Error>> {
    match env::args().nth(1).as_deref().unwrap_or(".") {
        "-h" | "--help" => {
            println!("Usage: cineteca [path] #defaults to current dir");
            Ok(())
        }
        input => {
            App::run(&PathBuf::from(input).canonicalize()?);
            Ok(())
        }
    }
}
