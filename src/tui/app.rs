use cursive::{
    Cursive, With,
    event::Event,
    theme::{BorderStyle, Palette},
    views::{Dialog, LinearLayout, NamedView, OnEventView, ScrollView, SelectView, TextView},
};

use std::path::PathBuf;

use crate::{
    archive::Archive,
    tui::{
        filter_view::FilterView,
        list_view::{ListView, SCROLL_ID, SELECT_ID},
        user_data::UserData,
    },
};

const HELP_KEYBINDS: &[&str] = &[
    "w -> mark as watched",
    "p -> play a movie",
    "? -> show this dialog",
    "q -> quit",
    "s -> toggle watched filter",
    "ESC -> go back",
];

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

        Self::setup_keybinds(&mut siv);

        siv.set_user_data(UserData::new(Archive::init(&self.path)));

        let list_view = ListView::new(&siv, &self.path);
        let filter_view = FilterView::new();

        siv.add_fullscreen_layer(
            LinearLayout::horizontal()
                .child(list_view)
                .child(filter_view),
        );

        //Populate views
        ListView::refresh(&mut siv);
        FilterView::refresh(&mut siv);

        siv.run();
    }

    fn setup_keybinds(siv: &mut cursive::CursiveRunnable) {
        siv.add_global_callback('q', cursive::Cursive::quit);
        siv.add_global_callback('?', Self::show_keybinds);
        siv.add_global_callback('h', |siv| {
            siv.call_on_name(SCROLL_ID, |v: &mut ScrollView<NamedView<SelectView>>| {
                v.scroll_to_left();
            });
        });
        siv.add_global_callback('l', |siv| {
            siv.call_on_name(SCROLL_ID, |v: &mut ScrollView<NamedView<SelectView>>| {
                v.scroll_to_right();
            });
        });
        siv.add_global_callback('j', |siv| {
            siv.call_on_name(SELECT_ID, |v: &mut SelectView| v.select_down(1));
            siv.call_on_name(SCROLL_ID, |v: &mut ScrollView<NamedView<SelectView>>| {
                v.scroll_to_important_area()
            });
        });
        siv.add_global_callback('k', |siv| {
            siv.call_on_name(SELECT_ID, |v: &mut SelectView| v.select_up(1));
            siv.call_on_name(SCROLL_ID, |v: &mut ScrollView<NamedView<SelectView>>| {
                v.scroll_to_important_area()
            });
        });
        siv.add_global_callback('w', ListView::toggle_watched);
        siv.add_global_callback('p', ListView::play_movie);
        siv.add_global_callback('s', FilterView::change_filter);
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
}
