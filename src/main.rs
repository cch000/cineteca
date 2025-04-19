use std::{
    error::Error,
    ffi::OsStr,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

use ffmpeg_next::log::Level::Quiet;

use cursive::{
    With,
    event::EventResult,
    reexports::ahash::HashSet,
    view::{Resizable, Scrollable},
    views::{Dialog, OnEventView, SelectView},
};
use walkdir::WalkDir;

struct Movie {
    name: String,
    path: PathBuf,
    watched: bool,
}

impl Movie {
    fn toggle_watched(&mut self) {
        let movie = self;

        movie.watched = !movie.watched;
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut c = cursive::default();

    let movies = Arc::new(RwLock::new(get_movies("/home/cch/Videos/arr")?));

    c.add_global_callback('q', |s| s.quit());

    let movies_for_view = Arc::clone(&movies);
    let select = SelectView::new().with(|list| {
        if let Ok(movies) = movies_for_view.read() {
            movies.iter().enumerate().for_each(|(index, movie)| {
                list.add_item(movie.name.clone(), index);
            });
        }
    });

    let movies_for_callback = Arc::clone(&movies);

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
            if let Ok(mut movies) = movies_for_callback.write() {
                if let Some(movie) = movies.get_mut(index) {
                    movie.toggle_watched();
                    let (label, _) = s.get_item_mut(index).unwrap();

                    *label = format!(
                        "{}{}",
                        if movie.watched { "[WATCHED] " } else { "" },
                        movie.name
                    )
                    .into();
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
    Ok(())
}

const EXTENSIONS: [&str; 3] = ["mkv", "mp4", "rs"];

fn is_movie(path: &Path, extensions: &HashSet<&OsStr>) -> Result<bool, Box<dyn Error>> {
    if !path
        .extension()
        .is_some_and(|ext| extensions.contains(&ext))
    {
        return Ok(false);
    }

    let duration = ffmpeg_next::format::input(path)?.duration() as f64
        / f64::from(ffmpeg_next::ffi::AV_TIME_BASE);

    Ok(duration.ge(&3600.0))
}

fn get_movies(movies_path: &str) -> Result<Vec<Movie>, Box<dyn Error>> {
    ffmpeg_next::init()?;
    ffmpeg_next::util::log::set_level(Quiet);

    let extensions: HashSet<&OsStr> = EXTENSIONS.iter().map(OsStr::new).collect();

    let movies: Vec<Movie> = WalkDir::new(movies_path)
        .max_depth(4)
        .into_iter()
        .filter_map(|res| res.ok())
        .filter_map(|entry| {
            let path = entry.path();
            if is_movie(path, &extensions).is_ok_and(|result| result) {
                Some(Movie {
                    name: path.file_name()?.to_owned().into_string().ok()?,
                    path: path.to_path_buf(),
                    watched: false,
                })
            } else {
                None
            }
        })
        .collect();
    Ok(movies)
}
