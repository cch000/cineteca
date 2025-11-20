use std::{
    collections::HashSet,
    error::Error,
    ffi::OsStr,
    fs::{self, Metadata},
    hash::{DefaultHasher, Hash, Hasher},
    path::{Path, PathBuf},
    sync::{Arc, mpsc},
    thread::{self, JoinHandle},
};

use ffmpeg_next::log::Level::Quiet;

use crate::movies_archive::{Movie, Show};

const EXTENSIONS: [&str; 4] = ["mkv", "mp4", "avi", "mov"];
const MIN_DURATION: i64 = 3600;
const MIN_FILE_SIZE: u64 = 300_000_000;

pub struct ProcessedMedia {
    movies: Vec<Movie>,
    shows: Vec<Show>,
}

impl ProcessedMedia {
    pub const fn new() -> Self {
        Self {
            movies: vec![],
            shows: vec![],
        }
    }
}

pub struct MovieCollector;

impl MovieCollector {
    pub fn collect_movies(movies_path: &str) -> (Vec<Movie>, u64) {
        Self::ffmpeg_init().expect("Failed to initialize ffmpeg");

        let extensions = Arc::new(
            EXTENSIONS
                .iter()
                .map(|ext| OsStr::new(ext))
                .collect::<HashSet<_>>(),
        );

        let mut hash = DefaultHasher::new();

        let top_entries: Vec<PathBuf> = fs::read_dir(movies_path)
            .expect("Invalid movie directory")
            .filter_map(std::result::Result::ok)
            .map(|entry| entry.path())
            .collect();

        // One chunk per cpu thread
        let max_chunks = thread::available_parallelism().unwrap().get();
        let chunk_size = top_entries.len().div_ceil(max_chunks);

        let (tx, rx) = mpsc::channel();

        let handles: Vec<JoinHandle<()>> = top_entries
            .chunks(chunk_size)
            .map(|chunk| {
                let tx = tx.clone();
                let extensions = Arc::clone(&extensions);
                let chunk = chunk.to_vec();

                thread::spawn(move || {
                    let mut processed_media = ProcessedMedia::new();

                    for path in chunk {
                        Self::process_dir(&mut processed_media, &path, &extensions).ok();
                    }

                    tx.send(processed_media)
                        .expect("Failed to send media from thread");
                })
            })
            .collect();

        let spawned_threads = handles.len();

        for h in handles {
            if let Err(_) = h.join() {
                eprintln!("Thread panicked");
            }
        }

        let mut movies: Vec<Movie> = rx
            .iter()
            .take(spawned_threads)
            .flat_map(|p| p.movies)
            .collect();

        movies.sort_by(|a, b| a.name.cmp(&b.name));
        movies.hash(&mut hash);

        (movies, hash.finish())
    }

    fn is_movie(
        path: &Path,
        extensions: &Arc<HashSet<&OsStr>>,
        metadata: &Metadata,
    ) -> Result<bool, Box<dyn Error>> {
        if !path
            .extension()
            .is_some_and(|ext| extensions.contains(&ext))
        {
            return Ok(false);
        }

        if metadata.len() < MIN_FILE_SIZE {
            return Ok(false);
        }

        let duration = ffmpeg_next::format::input(path)?.duration()
            / i64::from(ffmpeg_next::ffi::AV_TIME_BASE);

        Ok(duration.ge(&MIN_DURATION))
    }

    fn process_file(
        path: &Path,
        extensions: &Arc<HashSet<&OsStr>>,
    ) -> Result<Option<Movie>, Box<dyn Error>> {
        let metadata = path.metadata()?;

        if !metadata.is_file() {
            return Ok(None);
        }

        if !Self::is_movie(path, extensions, &metadata)? {
            return Ok(None);
        }

        let name = path
            .file_name()
            .ok_or("Invalid filename")?
            .to_string_lossy()
            .to_string();

        Ok(Some(Movie {
            name,
            path: path.to_path_buf(),
            watched: false,
        }))
    }

    fn process_dir(
        processed_media: &mut ProcessedMedia,
        path: &Path,
        extensions: &Arc<HashSet<&OsStr>>,
    ) -> Result<(), Box<dyn Error>> {
        if !path.is_dir() {
            if let Ok(Some(movie)) = Self::process_file(path, extensions) {
                processed_media.movies.push(movie);
            }
            return Ok(());
        }

        let entries = fs::read_dir(path)?;

        for entry in entries.filter_map(Result::ok) {
            let entry_path = entry.path();

            match entry.file_type() {
                Ok(ft) if ft.is_file() => {
                    if let Ok(Some(movie)) = Self::process_file(&entry_path, extensions) {
                        processed_media.movies.push(movie);
                    }
                }
                Ok(ft) if ft.is_dir() => {
                    Self::process_dir(processed_media, &entry_path, extensions)?;
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn ffmpeg_init() -> Result<(), Box<dyn Error>> {
        ffmpeg_next::init()?;
        ffmpeg_next::util::log::set_level(Quiet);
        Ok(())
    }
}
