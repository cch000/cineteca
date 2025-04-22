mod movies;

use cursive::{
    event::EventResult,
    view::{Resizable, Scrollable},
    views::{Dialog, OnEventView, SelectView},
};
use movies::MoviesLib;
use std::{
    error::Error,
    process::{Command, Stdio},
    sync::{Arc, RwLock},
};

struct App {
    movies: Arc<RwLock<MoviesLib>>,
}

impl App {
    fn new(path: &str) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            movies: Arc::new(RwLock::new(MoviesLib::init(path)?)),
        })
    }

    fn run(&self) -> Result<(), Box<dyn Error>> {
        let mut app = cursive::default();

        app.add_global_callback('q', |s| s.quit());

        app.add_fullscreen_layer(
            Dialog::new()
                .title("Movies Library")
                .content(self.movies_view()?.scrollable())
                .full_screen(),
        );

        app.run();

        if let Ok(movies) = self.movies.read() {
            movies.save_movies()?;
        }

        Ok(())
    }

    fn movies_view(&self) -> Result<OnEventView<SelectView<usize>>, Box<dyn Error>> {
        let mut select = SelectView::new();
        let movies = Arc::clone(&self.movies);
        let movies_clone = Arc::clone(&self.movies);

        Self::update_movies_view(&movies, &mut select, None)?;

        let view = OnEventView::new(select)
            .on_pre_event_inner('k', |s, _| {
                let cb = s.select_up(1);
                Some(EventResult::Consumed(Some(cb)))
            })
            .on_pre_event_inner('j', |s, _| {
                let cb = s.select_down(1);
                Some(EventResult::Consumed(Some(cb)))
            })
            .on_pre_event_inner('w', move |s, _| {
                if let Some(idx) = s.selection() {
                    Self::update_movies_view(&movies, s, Some(*idx)).ok();
                }
                Some(EventResult::Consumed(None))
            })
            .on_pre_event_inner('p', move |s, _| {
                if let Some(idx) = s.selection() {
                    Self::update_movies_view(&movies_clone, s, Some(*idx)).ok();
                    if let Ok(lib) = &movies_clone.read() {
                        let movie = lib.get_movie(*idx)?;
                        Command::new("mpv")
                            .arg(&movie.path)
                            .arg("--really-quiet")
                            .stdout(Stdio::null())
                            .stderr(Stdio::null())
                            .spawn()
                            .ok();
                    }
                }
                Some(EventResult::Consumed(None))
            });
        Ok(view)
    }

    fn update_movies_view(
        movies: &Arc<RwLock<MoviesLib>>,
        view: &mut SelectView<usize>,
        toggle_idx: Option<usize>,
    ) -> Result<(), Box<dyn Error>> {
        let selected = view.selected_id();

        // Toggle watched status if requested
        if let Some(idx) = toggle_idx {
            if let Ok(mut lib) = movies.write() {
                if let Some(movie) = lib.get_mut_movie(idx) {
                    movie.toggle_watched();
                }
                lib.save_movies().ok();
            }
        }

        if let Ok(lib) = movies.read() {
            let mut items: Vec<_> = lib.movies.iter().enumerate().collect();

            items.sort_by(|(_, a), (_, b)| {
                a.watched
                    .cmp(&b.watched)
                    .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
            });

            view.clear();
            for (idx, movie) in items {
                let name = if movie.watched {
                    format!("[WATCHED] {}", movie.name)
                } else {
                    movie.name.clone()
                };
                view.add_item(name, idx);
            }

            if let Some(id) = selected {
                view.set_selection(id);
            }
        }
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let app = App::new("/home/cch/Videos/arr")?;
    app.run()?;
    Ok(())
}
