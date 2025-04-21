mod movies;

use std::{
    error::Error,
    process::{Command, Stdio},
    sync::{Arc, RwLock},
};

use cursive::{
    event::EventResult,
    view::{Resizable, Scrollable},
    views::{Dialog, OnEventView, SelectView},
    With,
};
use movies::MoviesLib;

fn main() -> Result<(), Box<dyn Error>> {
    let mut c = cursive::default();
    let movies_path = "/home/cch/Videos/arr";

    let movies_lib = Arc::new(RwLock::new(MoviesLib::init(movies_path)?));

    c.add_global_callback('q', |s| s.quit());

    let movies_lib_view: Arc<RwLock<MoviesLib>> = Arc::clone(&movies_lib);
    let mut select = SelectView::new().with(|list| {
        if let Ok(movies_lib) = movies_lib_view.read() {
            movies_lib
                .movies
                .iter()
                .enumerate()
                .for_each(|(index, movie)| {
                    let label = format!(
                        "{}{}",
                        if movie.watched { "[WATCHED] " } else { "" },
                        movie.name
                    );
                    list.add_item(label, index);
                });
        }
    });

    // Sets the callback for when "Enter" is pressed.
    let movies_lib_submit: Arc<RwLock<MoviesLib>> = Arc::clone(&movies_lib);
    select.set_on_submit(move |_, index| {
        Command::new("mpv")
            .arg(
                movies_lib_submit
                    .read()
                    .unwrap()
                    .get_movie(*index)
                    .path
                    .clone(),
            )
            .arg("--really-quiet") // Suppress MPV output
            .stdout(Stdio::null()) // Redirect stdout to null
            .stderr(Stdio::null())
            .spawn()
            .unwrap()
            .wait()
            .unwrap();
    });

    let movies_lib_callback: Arc<RwLock<MoviesLib>> = Arc::clone(&movies_lib);
    let select = OnEventView::new(select)
        .on_pre_event_inner('k', |s, _| {
            let cb = s.select_up(1);
            Some(EventResult::Consumed(Some(cb)))
        })
        .on_pre_event_inner('j', |s, _| {
            let cb = s.select_down(1);
            Some(EventResult::Consumed(Some(cb)))
        })
        .on_pre_event_inner('w', move |s, _| {
            let index = *s.selection().unwrap();
            if let Ok(mut movies_lib) = movies_lib_callback.write() {
                let movie = movies_lib.get_mut_movie(index);
                movie.toggle_watched();
                let (label, _) = s.get_item_mut(index).unwrap();

                *label = format!(
                    "{}{}",
                    if movie.watched { "[WATCHED] " } else { "" },
                    movie.name
                )
                .into();

                // Save movies after updating watched status
                if let Err(e) = &movies_lib.save_movies() {
                    eprintln!("Failed to save movies: {}", e);
                }
            }

            Some(EventResult::Consumed(None))
        });

    c.add_fullscreen_layer(
        Dialog::new()
            .title("MOVIES")
            .content(select.scrollable().scroll_y(true).show_scrollbars(true))
            .full_screen(),
    );

    c.run();

    // Save movies one final time before exiting
    if let Ok(movies_lib) = movies_lib.read() {
        if let Err(e) = &movies_lib.save_movies() {
            eprintln!("Failed to save movies on exit: {}", e);
        }
    }

    Ok(())
}
