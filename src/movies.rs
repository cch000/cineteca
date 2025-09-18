use std::{
    collections::{HashMap, HashSet},
    error::Error,
    ffi::OsStr,
    fs,
    hash::{DefaultHasher, Hash, Hasher},
    ops::Not,
    path::{Path, PathBuf},
};

use ffmpeg_next::log::Level::Quiet;
use serde::{Deserialize, Serialize};
use walkdir::{DirEntry, WalkDir};

use rayon::iter::{ParallelBridge, ParallelIterator};

const SAVE_FILE: &str = ".movies.json";
const EXTENSIONS: [&str; 4] = ["mkv", "mp4", "avi", "mov"];

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

    #[serde(skip)]
    save_path: String,
    #[serde(skip)]
    movies_path: String,
}

impl MoviesLib {
    pub fn init(movies_path: &str) -> Self {
        let save_path = format!("{movies_path}/{SAVE_FILE}");

        if let Some(mut saved_movies) = Self::load_saved_movies(&save_path) {
            saved_movies.movies_path = movies_path.to_string();
            saved_movies.save_path = save_path;
            saved_movies
        } else {
            Self::build_movies_lib(movies_path, None, None, &save_path)
        }
    }

    pub fn refresh(&mut self) {
        let hash = Self::hash_dir(&self.movies_path);
        if hash != self.hash {
            *self = Self::build_movies_lib(
                &self.movies_path,
                Some(&self.movies),
                Some(hash),
                &self.save_path,
            );
        }
    }

    pub fn save_movies(&self) -> Result<(), Box<dyn Error>> {
        let json = serde_json::to_string_pretty(&self)?;
        fs::write(&self.save_path, json)?;
        Ok(())
    }

    pub fn toggle_watched(&mut self, name: &str) {
        self.movies
            .entry(name.to_owned())
            .and_modify(|(_, watched)| *watched = watched.not());
    }

    pub fn set_watched(&mut self, name: &str) {
        self.movies
            .entry(name.to_owned())
            .and_modify(|(_, watched)| *watched = true);
    }

    fn load_saved_movies(save_path: &String) -> Option<Self> {
        if Path::new(save_path).exists() {
            let json = fs::read_to_string(save_path).unwrap();

            let movies = serde_json::from_str(&json).expect("Error parsing json");
            Some(movies)
        } else {
            None
        }
    }

    fn hash_dir(movies_path: &str) -> u64 {
        let names: Vec<String> = WalkDir::new(movies_path)
            .into_iter()
            .par_bridge()
            .filter_map(Result::ok)
            .filter_map(|entry| Some(entry.file_name().to_string_lossy().into_owned()))
            .collect();

        let mut hash = DefaultHasher::new();
        names.hash(&mut hash);
        hash.finish()
    }

    fn is_movie(path: &Path, extensions: &HashSet<&OsStr>) -> Result<bool, Box<dyn Error>> {
        if path
            .extension()
            .is_some_and(|ext| extensions.contains(&ext))
        {
            let duration = ffmpeg_next::format::input(path)?.duration()
                / i64::from(ffmpeg_next::ffi::AV_TIME_BASE);

            Ok(duration.ge(&3600))
        } else {
            Ok(false)
        }
    }

    fn process_movie_entry(entry: &DirEntry) -> Option<(String, (PathBuf, bool))> {
        let path = entry.path();

        let extensions: HashSet<&OsStr> = EXTENSIONS.iter().map(OsStr::new).collect();

        if !Self::is_movie(path, &extensions).unwrap_or(false) {
            return None;
        }

        let name = path
            .file_name()
            .and_then(|fname| fname.to_owned().into_string().ok())?;

        // During rebuild we want to include all valid movies,
        // so we don't filter based on prev_movies
        Some((name, (path.to_path_buf(), false)))
    }

    fn build_movies_lib(
        movies_path: &str,
        prev_movies: Option<&HashMap<String, (PathBuf, bool)>>,
        hash: Option<u64>,
        save_path: &str,
    ) -> Self {
        Self::ffmpeg_init().expect("Failed to initialize ffmpeg");

        let mut movies: HashMap<String, (PathBuf, bool)> = WalkDir::new(movies_path)
            .into_iter()
            .par_bridge()
            .filter_map(Result::ok)
            .filter_map(|entry: walkdir::DirEntry| Self::process_movie_entry(&entry))
            .collect();

        if let Some(prev) = prev_movies {
            for (name, (_, prev_watched)) in prev {
                if let Some((_, watched)) = movies.get_mut(name) {
                    *watched = *prev_watched;
                }
            }
        }

        let save_path = save_path.to_string();
        let movies_path = movies_path.to_string();

        match prev_movies {
            Some(_) => Self {
                movies,
                hash: hash.unwrap(),
                save_path,
                movies_path,
            },
            None => Self {
                movies,
                hash: Self::hash_dir(&movies_path),
                save_path,
                movies_path,
            },
        }
    }

    fn ffmpeg_init() -> Result<(), Box<dyn Error>> {
        ffmpeg_next::init()?;
        ffmpeg_next::util::log::set_level(Quiet);
        Ok(())
    }
}
