use std::fmt::Display;

use cursive::{
    Cursive,
    view::{Nameable, Resizable, ViewWrapper},
    views::{NamedView, Panel, ResizedView, TextView},
    wrap_impl,
};

use crate::tui::{list_view::ListView, user_data::UserData};

const FILTER_ID: &str = "filter";

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Filter {
    Watched,
    NotWatched,
    Empty,
}

impl Display for Filter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::Watched => write!(f, "Watched"),
            Self::NotWatched => write!(f, "Not watched"),
            Self::Empty => write!(f, "None"),
        }
    }
}

impl Filter {
    fn cycle(&mut self) {
        *self = match self {
            Filter::NotWatched => Filter::Watched,
            Filter::Watched => Filter::Empty,
            Filter::Empty => Filter::NotWatched,
        };
    }
}

type ViewType = ResizedView<Panel<NamedView<TextView>>>;

pub struct FilterView {
    view: ViewType,
}

impl ViewWrapper for FilterView {
    wrap_impl!(self. view: ViewType);
}

impl FilterView {
    pub fn new() -> Self {
        let view = TextView::new("").with_name(FILTER_ID);
        let view = Panel::new(view).fixed_size((21, 3));

        Self { view }
    }

    pub fn refresh(siv: &mut Cursive) {
        let filter = siv
            .user_data::<UserData>()
            .unwrap()
            .get_filter()
            .to_string();

        if let Some(mut view) = siv.find_name::<TextView>(FILTER_ID) {
            view.set_content("Filter: ".to_owned() + &filter);
        }
    }

    pub fn change_filter(siv: &mut Cursive) {
        siv.with_user_data(|user_data: &mut UserData| {
            user_data.get_mut_filter().cycle();
        });

        Self::refresh(siv);
        ListView::refresh(siv);
    }
}
