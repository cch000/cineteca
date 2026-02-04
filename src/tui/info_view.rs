use std::time::SystemTime;

use cursive::{
    Cursive,
    view::{Nameable, ViewWrapper},
    views::{NamedView, Panel, TextView},
};

use crate::{
    collector::Movie,
    tui::{list_view::ListView, user_data::UserData},
};

pub const INFO_ID: &str = "info";

type ViewType = Panel<NamedView<TextView>>;

pub struct InfoView {
    view: ViewType,
}

impl ViewWrapper for InfoView {
    cursive:: wrap_impl!(self. view:  ViewType);
}

impl InfoView {
    pub fn new() -> Self {
        let view = TextView::new("").with_name(INFO_ID);
        let view = Panel::new(view);

        Self { view }
    }

    pub fn refresh(siv: &mut Cursive) {
        let name = &ListView::get_selected_name(siv).unwrap();

        let movie_data = siv
            .with_user_data(|user_data: &mut UserData| {
                let archive = user_data.get_mut_archive();
                archive
                    .movies
                    .iter()
                    .find(|m| m.name == *name)
                    .map(Self::format_movie_info)
            })
            .flatten();

        if let Some(content) = movie_data {
            siv.call_on_name(INFO_ID, |v: &mut TextView| {
                v.set_content(content);
            });
        }
    }

    fn format_movie_info(movie: &Movie) -> String {
        let watched_status = movie.date_watched.map_or_else(
            || "Not yet".to_string(),
            |time| {
                let days = SystemTime::now()
                    .duration_since(time)
                    .unwrap_or_default()
                    .as_secs()
                    / 86400;

                match days {
                    0 => "Today".to_string(),
                    1 => "A day ago".to_string(),
                    _ => format!("{days} days ago"),
                }
            },
        );

        let length = movie.duration / 60;

        format!("WATCHED: {watched_status}\n\nLENGTH: {length} minutes")
    }
}
