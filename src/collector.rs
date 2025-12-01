use std::{
    collections::HashSet,
    error::Error,
    ffi::OsStr,
    hash::{DefaultHasher, Hash, Hasher},
    path::Path,
    sync::mpsc,
    thread,
};

use ffmpeg_next::log::Level::Quiet;
use walkdir::{DirEntry, WalkDir};

use crate::archive::Movie;

const EXTENSIONS: [&str; 4] = ["mkv", "mp4", "avi", "mov"];
const MIN_DURATION: i64 = 3600;

pub struct Collector;

impl Collector {
    pub fn collect(path: &str) -> (Vec<Movie>, u64) {
        Self::ffmpeg_init().expect("Failed to initialize ffmpeg");

        let extensions: HashSet<&OsStr> = EXTENSIONS.iter().map(OsStr::new).collect();
        let mut hash = DefaultHasher::new();

        let max_threads = thread::available_parallelism().unwrap().get();
        let (tx, rx) = mpsc::channel();

        for chunk in WalkDir::new(path)
            .into_iter()
            .filter_map(Result::ok)
            .collect::<Vec<_>>()
            .chunks(WalkDir::new(path).into_iter().count() / max_threads + 1)
        {
            let tx = tx.clone();
            let chunk = chunk.to_vec();
            let extensions = extensions.clone();

            thread::spawn(move || {
                for entry in chunk {
                    if let Some(movie) = Collector::process_entry(&entry, &extensions) {
                        tx.send(movie).ok();
                    }
                }
            });
        }

        drop(tx);

        let mut movies: Vec<Movie> = rx.iter().collect();

        movies.sort_by(|a, b| a.name.cmp(&b.name));
        movies.hash(&mut hash);

        (movies, hash.finish())
    }

    fn is_movie(path: &Path, extensions: &HashSet<&OsStr>) -> Result<bool, Box<dyn Error>> {
        if path
            .extension()
            .is_some_and(|ext| extensions.contains(&ext))
        {
            let duration = ffmpeg_next::format::input(path)?.duration()
                / i64::from(ffmpeg_next::ffi::AV_TIME_BASE);
            Ok(duration >= MIN_DURATION)
        } else {
            Ok(false)
        }
    }

    fn process_entry(entry: &DirEntry, extensions: &HashSet<&OsStr>) -> Option<Movie> {
        let path = entry.path();

        if !Self::is_movie(path, extensions).unwrap_or(false) {
            return None;
        }

        let name = path.file_name()?.to_str()?.to_string();

        Some(Movie {
            name,
            path: path.to_path_buf(),
            watched: false,
        })
    }

    fn ffmpeg_init() -> Result<(), Box<dyn Error>> {
        ffmpeg_next::init()?;
        ffmpeg_next::util::log::set_level(Quiet);
        Ok(())
    }
}
