use cursive::{
    Cursive, With,
    event::{Event, EventResult},
    theme::{BorderStyle, Palette},
    view::{Resizable, Scrollable},
    views::{Dialog, OnEventView, SelectView, TextView},
};
use movies::MoviesLib;

use std::{
    error::Error,
    process::{Command, Stdio},
    sync::{Arc, RwLock},
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
    pub fn new(path: &str) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            movies: Arc::new(RwLock::new(MoviesLib::init(path)?)),
        })
    }

    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        let mut app = cursive::default();

        app.set_theme(cursive::theme::Theme {
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

        app.add_global_callback('q', Cursive::quit);

        app.add_global_callback('?', |app| {
            show_keybinds(app);
        });

        app.add_fullscreen_layer(
            Dialog::new()
                .title("CINETECA")
                .content(self.movies_view().scrollable())
                .full_screen(),
        );

        app.run();

        if let Ok(movies) = self.movies.read() {
            movies.save_movies()?;
        }

        Ok(())
    }

    fn movies_view(&self) -> OnEventView<SelectView> {
        let mut select = SelectView::new();
        let movies = Arc::clone(&self.movies);
        let movies_play = Arc::clone(&self.movies);

        Self::update_movies_view(&movies, &mut select);

        OnEventView::new(select)
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
                Self::play_movie(&movies_play, s);
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
            if let Ok(mut lib) = movies.write() {
                lib.toggle_watched(name);
                lib.save_movies()?;
            }
        }

        Self::update_movies_view(movies, view);
        Ok(())
    }

    fn update_movies_view(movies: &Arc<RwLock<MoviesLib>>, view: &mut SelectView<String>) {
        let selected = view.selected_id();

        if let Ok(lib) = movies.read() {
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
                    format!("[WATCHED] {name}")
                } else {
                    name.clone()
                };
                view.add_item(display_name, name.clone());
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
            let Ok(mut lib) = movies.write() else { return };

            let Some((path, _)) = lib.movies.get(name) else {
                return;
            };

            Command::new("mpv")
                .arg(path)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .ok();

            lib.set_watched(name);
            lib.save_movies().ok();
        }

        Self::update_movies_view(movies, s);
    }
}

fn show_keybinds(app: &mut Cursive) {
    let dialog = Dialog::new().title("Keybinds").content(TextView::new(
        HELP_KEYBINDS
            .iter()
            .map(|s| (*s).to_owned() + "\n")
            .collect::<String>(),
    ));

    app.add_layer(OnEventView::new(dialog).on_pre_event(
        Event::Key(cursive::event::Key::Esc),
        |app| {
            app.pop_layer();
        },
    ));
}
