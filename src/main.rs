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

    fn movies_view(&self) -> Result<OnEventView<SelectView<String>>, Box<dyn Error>> {
        let mut select = SelectView::new();
        let movies = Arc::clone(&self.movies);
        let movies_clone = Arc::clone(&self.movies);

        Self::update_movies_view(&movies, &mut select)?;

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
                Self::update_watched(&movies, s).ok();
                Some(EventResult::Consumed(None))
            })
            .on_pre_event_inner('p', move |s, _| {
                Self::play_movie(&movies_clone, s);
                Some(EventResult::Consumed(None))
            });
        Ok(view)
    }

    fn update_watched(
        movies: &Arc<RwLock<MoviesLib>>,
        view: &mut SelectView<String>,
    ) -> Result<(), Box<dyn Error>> {
        let selected = view.selected_id();
        let name = selected.and_then(|id| view.get_item(id).map(|(_, name)| name)); // Get the actual movie name

        if let Some(name) = name {
            if let Ok(mut lib) = movies.write() {
                lib.toggle_watched(name)?;
                lib.save_movies()?;
            }
        }

        Self::update_movies_view(movies, view)?;
        Ok(())
    }

    fn update_movies_view(
        movies: &Arc<RwLock<MoviesLib>>,
        view: &mut SelectView<String>,
    ) -> Result<(), Box<dyn Error>> {
        if let Ok(lib) = movies.read() {
            let selected = view.selected_id();
            let mut items: Vec<_> = lib.movies.iter().collect();

            items.sort_by(|(a_name, a_data), (b_name, b_data)| {
                a_data
                    .1
                    .cmp(&b_data.1)
                    .then_with(|| a_name.to_lowercase().cmp(&b_name.to_lowercase()))
            });

            view.clear();
            for (name, (_, watched)) in items {
                let display_name = if *watched {
                    format!("[WATCHED] {}", name)
                } else {
                    name.clone()
                };
                view.add_item(display_name, name.clone());
            }

            if let Some(selected) = selected {
                view.set_selection(selected);
            }
        }
        Ok(())
    }

    fn play_movie(movies_clone: &Arc<RwLock<MoviesLib>>, s: &mut SelectView) {
        let name = s
            .selected_id()
            .and_then(|id| s.get_item(id).map(|(_, name)| name));

        if let Some(name) = name {
            if let Ok(mut lib) = movies_clone.write() {
                if let Some((path, _)) = lib.movies.get(name) {
                    Command::new("mpv")
                        .arg(path)
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .spawn()
                        .ok();

                    lib.set_watched(name).ok();
                    lib.save_movies().ok();
                }
            }
            Self::update_movies_view(movies_clone, s).ok();
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let app = App::new("/home/cch/Videos/arr")?;
    app.run()?;
    Ok(())
}
