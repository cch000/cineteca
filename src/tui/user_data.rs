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

    pub const fn archive(&self) -> &Archive {
        &self.archive
    }

    pub const fn filter(&self) -> Filter {
        self.filter
    }

    pub const fn archive_mut(&mut self) -> &mut Archive {
        &mut self.archive
    }

    pub const fn filter_mut(&mut self) -> &mut Filter {
        &mut self.filter
    }
}
