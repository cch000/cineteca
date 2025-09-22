use std::{
    collections::HashSet,
    error::Error,
    ffi::OsStr,
    hash::{DefaultHasher, Hash, Hasher},
    path::Path,
    sync::mpsc,
    thread::{self, JoinHandle},
};

use ffmpeg_next::log::Level::Quiet;
use rayon::{
    iter::{ParallelBridge, ParallelIterator},
    slice::ParallelSliceMut,
};
use walkdir::{DirEntry, WalkDir};

use crate::movies_archive::Movie;

const EXTENSIONS: [&str; 4] = ["mkv", "mp4", "avi", "mov"];
const MIN_DURATION: i64 = 3600;

pub struct MovieCollector;

impl MovieCollector {
    pub fn collect_movies(movies_path: &str) -> (Vec<Movie>, u64) {
        Self::ffmpeg_init().expect("Failed to initialize ffmpeg");
        let extensions: HashSet<&OsStr> = EXTENSIONS.iter().map(OsStr::new).collect();

        let mut hash = DefaultHasher::new();

        let entries: Vec<_> = WalkDir::new(movies_path)
            .into_iter()
            .par_bridge()
            .filter_map(Result::ok)
            .collect();

        // One chunk per cpu thread
        let num_chunks = thread::available_parallelism().unwrap().get();
        let chunk_size = entries.len().div_ceil(num_chunks);

        let (tx, rx) = mpsc::channel();

        let handles: Vec<JoinHandle<()>> = entries
            .chunks(chunk_size)
            .map(<[walkdir::DirEntry]>::to_vec)
            .map(|chunk| {
                let tx = tx.clone();
                let extensions = extensions.clone();

                thread::spawn(move || {
                    let movies: Vec<Movie> = chunk
                        .into_iter()
                        .filter_map(|entry| Self::process_movie_entry(&entry, &extensions))
                        .collect();
                    tx.send(movies).expect("Failed to send my share of movies");
                })
            })
            .collect();

        for h in handles {
            h.join().unwrap();
        }

        let mut movies: Vec<Movie> = rx.iter().take(num_chunks).flatten().collect();

        movies.par_sort_by(|a, b| a.name.cmp(&b.name));

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

            Ok(duration.ge(&MIN_DURATION))
        } else {
            Ok(false)
        }
    }

    fn process_movie_entry(entry: &DirEntry, extensions: &HashSet<&OsStr>) -> Option<Movie> {
        let path = entry.path();

        if !Self::is_movie(path, extensions).unwrap_or(false) {
            return None;
        }

        let name = path
            .file_name()
            .and_then(|fname| fname.to_owned().into_string().ok())?;

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
