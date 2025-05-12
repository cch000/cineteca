use std::{
    collections::HashSet,
    error::Error,
    ffi::OsStr,
    fs,
    hash::{DefaultHasher, Hash, Hasher},
    ops::Not,
    path::{Path, PathBuf},
};

use cursive::reexports::ahash::HashMap;
use ffmpeg_next::log::Level::Quiet;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use rayon::iter::{ParallelBridge, ParallelIterator};

const SAVE_FILE: &str = ".movies.json";
const EXTENSIONS: [&str; 4]  = ["mkv", "mp4", "avi", "mov"];

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Hash)]
pub struct Movie {
    pub name: String,
    pub path: PathBuf,
    pub watched: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MoviesLib {
    pub movies: HashMap<String, (PathBuf, bool)>,
    hash: u64,
}



impl MoviesLib {
    pub fn init(movies_path: &str) -> Result<MoviesLib, Box<dyn std::error::Error>> {
        let current_hash = Self::hash_dir(movies_path)?;

        let movies = if let Ok(saved_movies) = Self::load_movies_save() {
            if current_hash == saved_movies.hash {
                saved_movies
            } else {
                Self::build_movies_lib(movies_path, Some(saved_movies), Some(current_hash))?
            }
        } else {
            Self::build_movies_lib(movies_path, None, None)?
        };
        Ok(movies)
    }

    pub fn save_movies(&self) -> Result<(), Box<dyn Error>> {
        let json = serde_json::to_string_pretty(&self)?;
        fs::write(SAVE_FILE, json)?;
        Ok(())
    }

    pub fn toggle_watched(&mut self, name: &str) -> Result<(), Box<dyn Error>> {
        self.movies
            .entry(name.to_owned())
            .and_modify(|(_, watched)| *watched = watched.not());
        Ok(())
    }

    pub fn set_watched(&mut self, name: &str) -> Result<(), Box<dyn Error>> {
        self.movies
            .entry(name.to_owned())
            .and_modify(|(_, watched)| *watched = true);

        Ok(())
    }

    fn load_movies_save() -> Result<Self, Box<dyn Error>> {
        if Path::new(SAVE_FILE).exists() {
            let json = fs::read_to_string(SAVE_FILE)?;
            let movies: MoviesLib = serde_json::from_str(&json)?;
            Ok(movies)
        } else {
            Err("No saved movies or empty save file".into())
        }
    }

    fn hash_dir(movies_path: &str) -> Result<u64, Box<dyn Error>> {
        let names: Vec<String> = WalkDir::new(movies_path)
            .max_depth(3)
            .into_iter()
            .par_bridge()
            .filter_map(|res| res.ok())
            .filter_map(|entry| Some(entry.file_name().to_string_lossy().into_owned()))
            .collect();
        let mut hash = DefaultHasher::new();
        names.hash(&mut hash);
        Ok(hash.finish())
    }

    fn is_movie(path: &Path, extensions: &HashSet<&OsStr>) -> Result<bool, Box<dyn Error>> {
        if !path
            .extension()
            .is_some_and(|ext| extensions.contains(&ext))
        {
            Ok(false)
        } else {
            let duration = ffmpeg_next::format::input(path)?.duration() as f64
                / f64::from(ffmpeg_next::ffi::AV_TIME_BASE);

            Ok(duration.ge(&3600.0))
        }
    }

    fn process_movie_entry(entry: walkdir::DirEntry) -> Option<(String, (PathBuf, bool))> {
        let path = entry.path();

        let extensions: HashSet<&OsStr> = EXTENSIONS.iter().map(OsStr::new).collect();

        if !Self::is_movie(path, &extensions).unwrap_or(false) {
            return None;
        }

        let name = path
            .file_name()
            .and_then(|fname| fname.to_owned().into_string().ok())?;

        // During rebuild we want to include all valid movies,
        // so we don't filter based on existing_movies
        Some((name, (path.to_path_buf(), false)))
    }

    fn build_movies_lib(
        movies_path: &str,
        existing_movies: Option<MoviesLib>,
        current_hash: Option<u64>,
    ) -> Result<Self, Box<dyn Error>> {
        Self::ffmpeg_init()?;

        let mut current_movies: HashMap<String, (PathBuf, bool)> = WalkDir::new(movies_path)
            .into_iter()
            .par_bridge()
            .filter_map(Result::ok)
            .filter_map(Self::process_movie_entry)
            .collect();

        if let Some(lib) = &existing_movies {
            for (name, (_, watched)) in lib.movies.iter() {
                if let Some((_, existing_watched)) = current_movies.get_mut(name) {
                    *existing_watched = *watched;
                }
            }
        }

        match existing_movies {
            Some(mut lib) => {
                lib.movies = current_movies;
                lib.hash = current_hash.unwrap();
                Ok(lib)
            }
            None => Ok(MoviesLib {
                movies: current_movies,
                hash: Self::hash_dir(movies_path)?,
            }),
        }
    }

    fn ffmpeg_init() -> Result<(), Box<dyn Error>> {
        ffmpeg_next::init()?;
        ffmpeg_next::util::log::set_level(Quiet);
        Ok(())
    }
}
