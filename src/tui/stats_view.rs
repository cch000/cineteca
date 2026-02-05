use std::time::SystemTime;

use cursive::{
    Cursive,
    view::{Nameable, ViewWrapper},
    views::{NamedView, Panel, TextView},
};

use crate::tui::user_data::UserData;

pub const STATS_ID: &str = "stats";

type ViewType = Panel<NamedView<TextView>>;

pub struct StatsView {
    view: ViewType,
}

impl ViewWrapper for StatsView {
    cursive:: wrap_impl!(self.view:  ViewType);
}

impl StatsView {
    pub fn new() -> Self {
        let view = TextView::new("").with_name(STATS_ID);
        let view = Panel::new(view);

        Self { view }
    }

    pub fn refresh(siv: &mut Cursive) {
        let Some((total_num, watched_num, recent_num)) =
            siv.with_user_data(|user_data: &mut UserData| {
                let time = SystemTime::now();
                let movies = &user_data.get_mut_archive().movies;

                movies
                    .iter()
                    .fold((movies.len(), 0, 0), |(_total, watched, recent), m| {
                        let is_watched = m.date_watched.is_some();
                        let is_recent = m.date_watched.is_some_and(|d| {
                            time.duration_since(d)
                                .map(|dur| dur.as_secs() / 86400 <= 14)
                                .unwrap_or(false)
                        });
                        (
                            _total, //passthrough
                            watched + usize::from(is_watched),
                            recent + usize::from(is_recent),
                        )
                    })
            })
        else {
            return;
        };

        let remaining = total_num - watched_num;

        let content = format!("MOVIES TOTAL: {total_num}\n")
            + "WATCHED:\n"
            + &format!("├ last 14d: {recent_num}\n")
            + &format!("└ total: {watched_num}\n")
            + &format!("REMAINING: {remaining}");

        siv.call_on_name(STATS_ID, |v: &mut TextView| {
            v.set_content(content);
        });
    }
}
