use std::error::Error;

use cursive::{
    With,
    view::Resizable,
    views::{Dialog, ListView, TextView},
};
use walkdir::WalkDir;
fn main() -> Result<(), Box<dyn Error>> {
    let mut c = cursive::default();
    c.add_global_callback('q', |s| s.quit());

    let movies = get_movies()?;
    c.add_fullscreen_layer(
        Dialog::new()
            .title("MOVIES")
            .content(ListView::new().with(|list| {
                for elem in movies.iter() {
                    list.add_child("> ", TextView::new(elem));
                }
            }))
            .full_screen(),
    );

    c.run();
    Ok(())
}

fn get_movies() -> Result<Vec<String>, Box<dyn Error>> {
    let names = WalkDir::new("./")
        .max_depth(4)
        .into_iter()
        .filter_map(|res| res.ok())
        .filter_map(|entry| {
            let path = entry.path();
            if path
                .extension().is_some_and(|ext| ext == "mkv" || ext == "rs")
            {
                path.file_name()?.to_owned().into_string().ok()
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    Ok(names)
}
