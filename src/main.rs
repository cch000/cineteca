mod archive;
mod collector;
mod movie;
mod tui;

use std::{env, path::PathBuf};

use crate::tui::app::App;

fn main() {
    match env::args().nth(1).as_deref().unwrap_or(".") {
        "-h" | "--help" => println!("Usage: cineteca [path] #defaults to current dir"),
        input => App::new(
            PathBuf::from(input.trim_end_matches('/'))
                .canonicalize()
                .unwrap_or_else(|_| panic!("The provided path does no exist")),
        )
        .run(),
    }
}
