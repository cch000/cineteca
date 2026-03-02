use cursive::{
    Cursive,
    view::{Nameable, ViewWrapper},
    views::{NamedView, Panel, TextView},
};

use crate::tui::{list_view::ListView, user_data::UserData};

pub const INFO_ID: &str = "info";

type ViewType = Panel<NamedView<TextView>>;

pub struct InfoView {
    view: ViewType,
}

impl ViewWrapper for InfoView {
    cursive:: wrap_impl!(self.view:  ViewType);
}

impl InfoView {
    pub fn new() -> Self {
        let view = TextView::new("").with_name(INFO_ID);
        let view = Panel::new(view);

        Self { view }
    }

    pub fn refresh(siv: &mut Cursive) {
        let name = &ListView::get_selected_name(siv);

        let movie_data = siv.user_data().and_then(|d: &mut UserData| {
            d.archive_mut()
                .movies
                .iter()
                .find(|m| m.name() == *name)
                .map(|m| {
                    format!(
                        "WATCHED: {}\nLENGTH: {}",
                        m.pretty_since_watched(),
                        m.pretty_length()
                    )
                })
        });

        if let Some(content) = movie_data {
            siv.call_on_name(INFO_ID, |v: &mut TextView| {
                v.set_content(content);
            });
        }
    }
}
