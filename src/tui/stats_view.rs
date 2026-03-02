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
        let Some(user_data) = siv.user_data::<UserData>() else {
            return;
        };

        let time = SystemTime::now();
        let movies = &user_data.archive().movies;
        let total_num = movies.len();

        let (watched_num, recent_num) = movies.iter().fold((0, 0), |(watched, recent), m| {
            let is_watched = m.since_watched().is_some();
            let is_recent = m.since_watched().is_some_and(|d| {
                time.duration_since(d)
                    .map(|dur| dur.as_secs() / 86400 <= 14)
                    .unwrap_or(false)
            });
            (
                watched + usize::from(is_watched),
                recent + usize::from(is_recent),
            )
        });

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
