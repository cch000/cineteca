use std::{
    path::Path,
    process::{Command, Stdio},
    thread,
};

use cursive::{
    Cursive,
    view::{Nameable, Resizable, Scrollable, ViewWrapper},
    views::{NamedView, Panel, ResizedView, ScrollView, SelectView},
};

use crate::{
    collector::{Collector, Movie},
    tui::{filter_view::Filter, info_view::InfoView, stats_view::StatsView, user_data::UserData},
};

pub const SELECT_ID: &str = "select";
pub const SCROLL_ID: &str = "scroll";

type ViewType = Panel<ResizedView<NamedView<ScrollView<NamedView<SelectView<String>>>>>>;

pub struct ListView {
    view: ViewType,
}

impl ViewWrapper for ListView {
    cursive::wrap_impl!(self.view: ViewType);
}

impl ListView {
    pub fn new(siv: &Cursive, path: &Path) -> Self {
        let view = SelectView::<String>::new()
            .on_select(|siv, _| InfoView::refresh(siv))
            .with_name(SELECT_ID)
            .scrollable()
            .scroll_x(true)
            .with_name(SCROLL_ID)
            .full_screen();

        Self::background_refresh(siv, path);

        Self {
            view: Panel::new(view),
        }
    }

    pub fn refresh(siv: &mut Cursive) {
        let items: Vec<(String, String)> = siv
            .with_user_data(|user_data: &mut UserData| {
                let archive = user_data.get_mut_archive();
                let mut items: Vec<Movie> = archive.movies.clone();

                items.sort_by(|a, b| match (a.date_watched, b.date_watched) {
                    (None, None) => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                    (None, Some(_)) => std::cmp::Ordering::Less,
                    (Some(_), None) => std::cmp::Ordering::Greater,
                    (Some(date_a), Some(date_b)) => date_b.cmp(&date_a),
                });

                let filter = user_data.get_mut_filter();
                items
                    .into_iter()
                    .filter(|movie| match filter {
                        Filter::NotWatched => movie.date_watched.is_none(),
                        Filter::Watched => movie.date_watched.is_some(),
                        Filter::Empty => true,
                    })
                    .map(|item| (item.name.clone(), item.name))
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

        InfoView::refresh(siv);
        StatsView::refresh(siv);
    }

    pub fn toggle_watched(siv: &mut Cursive) {
        if let Some(name) = Self::get_selected_name(siv) {
            siv.with_user_data(|user_data: &mut UserData| {
                let archive = user_data.get_mut_archive();
                archive.toggle_watched(&name);
                archive.save().ok();
            });
        }

        Self::refresh(siv);
    }

    pub fn play_movie(siv: &mut Cursive) {
        if let Some(name) = Self::get_selected_name(siv) {
            siv.with_user_data(|user_data: &mut UserData| {
                let archive = user_data.get_mut_archive();
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

            Self::refresh(siv);
        }
    }

    pub fn get_selected_name(siv: &mut Cursive) -> Option<String> {
        siv.call_on_name(SELECT_ID, |s: &mut SelectView<String>| {
            s.selected_id()
                .and_then(|id| s.get_item(id).map(|(_, name)| name.clone()))
        })
        .flatten()
    }

    fn background_refresh(siv: &Cursive, path: &Path) {
        let path = path.to_path_buf();
        let cb = siv.cb_sink().clone();
        thread::spawn(move || {
            let (movies, hash) = Collector::collect(&path);

            cb.send(Box::new(move |siv| {
                siv.with_user_data(|user_data: &mut UserData| {
                    let archive = user_data.get_mut_archive();
                    archive.update(&movies, hash);
                    archive.save().ok();
                });

                Self::refresh(siv);
            }))
            .ok();
        });
    }
}
