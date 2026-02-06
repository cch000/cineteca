use cursive::{
    Cursive, With,
    event::Event,
    theme::{BorderStyle, Palette},
    view::Resizable,
    views::{
        Dialog, DummyView, LinearLayout, NamedView, OnEventView, Panel, ScrollView, SelectView,
        TextView,
    },
};

use std::path::PathBuf;

use crate::{
    archive::Archive,
    tui::{
        filter_view::FilterView,
        info_view::InfoView,
        list_view::{ListView, SCROLL_ID, SELECT_ID},
        stats_view::StatsView,
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

const CONTENT: &str = r"
  ___(_)_ __   ___| |_ ___  ___ __ _ 
 / __| | '_ \ / _ \ __/ _ \/ __/ _` |
| (__| | | | |  __/ ||  __/ (_| (_| |
 \___|_|_| |_|\___|\__\___|\___\__,_|";

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
        let info_view = InfoView::new();
        let stats_view = StatsView::new();

        siv.add_fullscreen_layer(
            LinearLayout::vertical()
                .child(
                    LinearLayout::horizontal()
                        .child(TextView::new(CONTENT).fixed_width(40))
                        .child(
                            LinearLayout::vertical()
                                .child(DummyView.fixed_height(2))
                                .child(filter_view.fixed_height(3).full_width()),
                        ),
                )
                .child(
                    LinearLayout::horizontal().child(list_view).child(
                        LinearLayout::vertical()
                            .child(info_view)
                            .child(Panel::new(DummyView).full_height())
                            .child(stats_view)
                            .fixed_width(19),
                    ),
                ),
        );

        //Populate views

        ListView::refresh(&mut siv);
        FilterView::refresh(&mut siv);
        InfoView::refresh(&mut siv);
        StatsView::refresh(&mut siv);

        siv.run();
    }

    fn setup_keybinds(siv: &mut cursive::CursiveRunnable) {
        siv.add_global_callback('q', cursive::Cursive::quit);
        siv.add_global_callback('?', Self::show_keybinds);

        // Helper to move selection and manually trigger the InfoView refresh
        let move_and_refresh = |s: &mut Cursive, direction: i32| {
            let mut current_name = None;

            s.call_on_name(SELECT_ID, |v: &mut SelectView| {
                let steps = direction.unsigned_abs() as usize;
                if direction > 0 {
                    v.select_down(steps);
                } else {
                    v.select_up(steps);
                }

                if let Some(name) = v.selection() {
                    current_name = Some((*name).clone());
                }
            });

            if let Some(_name) = current_name {
                InfoView::refresh(s);
            }

            s.call_on_name(SCROLL_ID, |v: &mut ScrollView<NamedView<SelectView>>| {
                v.scroll_to_left();
                v.scroll_to_important_area();
            });
        };

        siv.add_global_callback('j', move |s| move_and_refresh(s, 1));
        siv.add_global_callback('k', move |s| move_and_refresh(s, -1));

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
