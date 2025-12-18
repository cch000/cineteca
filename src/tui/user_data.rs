use crate::{archive::Archive, tui::filter_view::Filter};

pub struct UserData {
    archive: Archive,
    filter: Filter,
}

impl UserData {
    pub const fn new(archive: Archive) -> Self {
        Self {
            archive,
            filter: Filter::Empty,
        }
    }

    pub const fn get_filter(&self) -> &Filter {
        &self.filter
    }

    pub const fn get_mut_filter(&mut self) -> &mut Filter {
        &mut self.filter
    }

    pub const fn get_mut_archive(&mut self) -> &mut Archive {
        &mut self.archive
    }
}
