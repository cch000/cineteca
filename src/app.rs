use archive::Archive;
use cursive::{
    Cursive, With,
    event::{Event, EventResult},
    theme::{BorderStyle, Palette},
    view::{Nameable, Resizable, Scrollable},
    views::{Dialog, NamedView, OnEventView, ScrollView, SelectView, TextView},
};

use std::{
    path::PathBuf,
    process::{Command, Stdio},
    thread,
};

use crate::{archive, collector::Collector};

const HELP_KEYBINDS: &[&str] = &[
    "w -> mark as watched",
    "p -> play a movie",
    "? -> show this dialog",
    "q -> quit",
    "ESC -> go back",
];

const SELECT_ID: &str = "select";

pub struct App {
    path: PathBuf,
}

impl App {
    pub const fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn run(&self) {
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

        siv.set_user_data(Archive::init(&self.path));

        siv.add_global_callback('q', cursive::Cursive::quit);
        siv.add_global_callback('?', Self::show_keybinds);

        siv.add_fullscreen_layer(
            Dialog::new()
                .title("CINETECA")
                .content(Self::movies_view())
                .full_screen(),
        );

        let cb = siv.cb_sink().clone();
        let path = self.path.clone();

        thread::spawn(move || {
            let (movies, hash) = Collector::collect(&path);

            cb.send(Box::new(move |siv| {
                siv.with_user_data(|archive: &mut Archive| {
                    archive.update(&movies, hash);
                    archive.save().ok();
                });

                Self::update_movies_view(siv);
            }))
            .ok();
        });

        Self::update_movies_view(&mut siv);

        siv.run();
    }

    fn movies_view() -> OnEventView<ScrollView<NamedView<SelectView>>> {
        let select = SelectView::<String>::new()
            .with_name(SELECT_ID)
            .scrollable()
            .scroll_x(true);

        OnEventView::new(select)
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
            .on_pre_event('w', Self::toggle_watched)
            .on_pre_event('p', Self::play_movie)
    }

    fn toggle_watched(siv: &mut Cursive) {
        if let Some(name) = Self::get_selected_name(siv) {
            siv.with_user_data(|archive: &mut Archive| {
                archive.toggle_watched(&name);
                archive.save().ok();
            });
        }

        Self::update_movies_view(siv);
    }

    fn update_movies_view(siv: &mut Cursive) {
        let items: Vec<(String, String)> = siv
            .with_user_data(|archive: &mut Archive| {
                let mut items: Vec<_> = archive.movies.iter().collect();

                items.sort_by(|a, b| {
                    a.watched
                        .cmp(&b.watched)
                        .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
                });

                items
                    .into_iter()
                    .map(|item| {
                        let label = if item.watched {
                            format!("[WATCHED] {}", item.name)
                        } else {
                            item.name.clone()
                        };
                        (label, item.name.clone())
                    })
                    .collect()
            })
            .unwrap_or_default();

        if let Some(mut view) = siv.find_name::<SelectView<String>>(SELECT_ID) {
            let selected_id = view.selected_id();

            view.clear();

            for (label, id) in items {
                view.add_item(label, id);
            }

            if let Some(id) = selected_id {
                view.set_selection(id);
            }
        }
    }

    fn play_movie(siv: &mut Cursive) {
        if let Some(name) = Self::get_selected_name(siv) {
            siv.with_user_data(|archive: &mut Archive| {
                let path = archive.get_path(&name);
                let path_string = path.to_string_lossy().into_owned();

                Command::new("xdg-open")
                    .arg(path_string)
                    .stderr(Stdio::null())
                    .stdin(Stdio::null())
                    .spawn()
                    .ok();

                archive.set_watched(&name);
                archive.save().ok();
            });

            Self::update_movies_view(siv);
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

    fn get_selected_name(siv: &mut Cursive) -> Option<String> {
        siv.call_on_name(SELECT_ID, |s: &mut SelectView<String>| {
            s.selected_id()
                .and_then(|id| s.get_item(id).map(|(_, name)| name.clone()))
        })
        .flatten()
    }
}
