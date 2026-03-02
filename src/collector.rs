use ffmpeg_next::log::Level::Quiet;
use std::{
    error::Error,
    hash::{DefaultHasher, Hash, Hasher},
    path::Path,
};
use walkdir::WalkDir;

use crate::movie::Movie;

pub struct Collector;

impl Collector {
    pub fn collect(path: &Path) -> (Vec<Movie>, u64) {
        Self::ffmpeg_init().expect("Failed to initialize ffmpeg");

        let mut hash = DefaultHasher::new();

        let mut movies: Vec<_> = WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok().and_then(|e| Movie::try_from(e.path()).ok()))
            .collect();

        movies.sort_by(|a, b| a.name().cmp(b.name()));
        movies.hash(&mut hash);

        (movies, hash.finish())
    }

    fn ffmpeg_init() -> Result<(), Box<dyn Error>> {
        ffmpeg_next::init()?;
        ffmpeg_next::util::log::set_level(Quiet);
        Ok(())
    }
}
