use cursive::{
    Cursive, With,
    event::{Event, EventResult},
    theme::{BorderStyle, Palette},
    view::{Nameable, Resizable, Scrollable},
    views::{Dialog, NamedView, OnEventView, ScrollView, SelectView, TextView},
};
use movies::MoviesLib;
use rayon::slice::ParallelSliceMut;

use std::{
    error::Error,
    process::{Command, Stdio},
    sync::{Arc, RwLock},
    thread,
};

use crate::movies;

const HELP_KEYBINDS: &[&str] = &[
    "w -> mark as watched",
    "p -> play a movie",
    "? -> show this dialog",
    "q -> quit",
    "ESC -> go back",
];

pub struct App {
    movies: Arc<RwLock<MoviesLib>>,
}

impl App {
    pub fn new(path: &str) -> Self {
        Self {
            movies: Arc::new(RwLock::new(MoviesLib::init(path))),
        }
    }

    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        let mut siv = cursive::default();

        siv.set_theme(cursive::theme::Theme {
            shadow: true,
            borders: BorderStyle::Simple,
            palette: Palette::retro().with(|palette| {
                use cursive::style::BaseColor::{Blue, White};

                {
                    use cursive::style::Color::TerminalDefault;
                    use cursive::style::PaletteColor::{Background, Primary, TitlePrimary, View};

                    palette[Background] = TerminalDefault;
                    palette[View] = TerminalDefault;
                    palette[Primary] = White.dark();
                    palette[TitlePrimary] = Blue.dark();
                }

                {
                    use cursive::style::Effect::Bold;
                    use cursive::style::PaletteStyle::Highlight;
                    use cursive::style::Style;
                    palette[Highlight] = Style::from(Blue.light()).combine(Bold);
                }
            }),
        });

        siv.add_global_callback('q', Cursive::quit);

        siv.add_global_callback('?', |siv| {
            show_keybinds(siv);
        });

        siv.add_fullscreen_layer(
            Dialog::new()
                .title("CINETECA")
                .content(self.movies_view())
                .full_screen(),
        );

        let movies_refresh = Arc::clone(&self.movies);
        let cb = siv.cb_sink().clone();

        thread::spawn(move || {
            if let Ok(mut movies_lock) = movies_refresh.write() {
                movies_lock.refresh();
            } else {
                return;
            }

            cb.send(Box::new(move |siv: &mut Cursive| {
                siv.call_on_all_named("movies_select", |view: &mut SelectView<String>| {
                    Self::update_movies_view(&movies_refresh, view);
                });
            }))
            .ok();
        });

        siv.run();

        if let Ok(movies) = self.movies.read() {
            movies.save_movies()?;
        }

        Ok(())
    }

    fn movies_view(&self) -> OnEventView<ScrollView<NamedView<SelectView>>> {
        let mut select = SelectView::new().with_name("movies_select");

        Self::update_movies_view(&self.movies, &mut select.get_mut());

        let movies = Arc::clone(&self.movies);
        let movies_play = Arc::clone(&self.movies);
        let scrollable_select = select.scrollable().scroll_x(true);

        OnEventView::new(scrollable_select)
            .on_pre_event_inner('h', |s, _| Some(s.scroll_to_left()))
            .on_pre_event_inner('l', |s, _| Some(s.scroll_to_right()))
            .on_pre_event_inner('j', |s, _| {
                let cb = s.get_inner_mut().get_mut().select_down(1);
                s.scroll_to_important_area();
                Some(EventResult::Consumed(Some(cb)))
            })
            .on_pre_event_inner('k', |s, _| {
                let cb = s.get_inner_mut().get_mut().select_up(1);
                s.scroll_to_important_area();
                Some(EventResult::Consumed(Some(cb)))
            })
            .on_pre_event_inner('w', move |s, _| {
                Self::update_watched(&movies, &mut s.get_inner_mut().get_mut()).ok();
                Some(EventResult::Consumed(None))
            })
            .on_pre_event_inner('p', move |s, _| {
                Self::play_movie(&movies_play, &mut s.get_inner_mut().get_mut());
                Some(EventResult::Consumed(None))
            })
    }

    fn update_watched(
        movies: &Arc<RwLock<MoviesLib>>,
        view: &mut SelectView<String>,
    ) -> Result<(), Box<dyn Error>> {
        let selected = view.selected_id();
        let name = selected.and_then(|id| view.get_item(id).map(|(_, name)| name)); // Get the actual movie name

        if let Some(name) = name {
            if let Ok(mut movies) = movies.write() {
                movies.toggle_watched(name);
                movies.save_movies()?;
            }
        }

        Self::update_movies_view(movies, view);
        Ok(())
    }

    fn update_movies_view(movies: &Arc<RwLock<MoviesLib>>, view: &mut SelectView<String>) {
        let selected = view.selected_id();

        if let Ok(lib) = movies.read() {
            let mut items: Vec<_> = lib.movies.iter().collect();

            items.par_sort_by(|a, b| {
                a.watched
                    .cmp(&b.watched)
                    .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
            });

            view.clear();

            for movie in items {
                let display_name = if movie.watched {
                    format!("[WATCHED] {}", movie.name)
                } else {
                    movie.name.clone()
                };
                view.add_item(display_name, movie.name.clone());
            }
        }

        if let Some(selected) = selected {
            view.set_selection(selected);
        }
    }

    fn play_movie(movies: &Arc<RwLock<MoviesLib>>, s: &mut SelectView) {
        let Some(name) = s
            .selected_id()
            .and_then(|id| s.get_item(id).map(|(_, name)| name))
        else {
            return;
        };

        {
            let Ok(mut movies) = movies.write() else {
                return;
            };

            movies.set_watched(name);
            movies.save_movies().ok();

            let path = movies.get_path(name);

            Command::new("mpv")
                .arg(path)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .ok();
        }

        Self::update_movies_view(movies, s);
    }
}

fn show_keybinds(siv: &mut Cursive) {
    let dialog = Dialog::new().title("Keybinds").content(TextView::new(
        HELP_KEYBINDS
            .iter()
            .map(|s| (*s).to_owned() + "\n")
            .collect::<String>(),
    ));

    siv.add_layer(OnEventView::new(dialog).on_pre_event(
        Event::Key(cursive::event::Key::Esc),
        |app| {
            app.pop_layer();
        },
    ));
}
