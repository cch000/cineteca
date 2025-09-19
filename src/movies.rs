use std::{
    collections::HashSet,
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

use rayon::{
    iter::{IntoParallelIterator, ParallelBridge, ParallelExtend, ParallelIterator},
    slice::ParallelSliceMut,
};

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
    pub movies: Vec<Movie>,
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
                Some(&mut self.movies),
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
        let index = self.get_index(name);
        let movie = self.movies.get_mut(index).unwrap();

        movie.watched = movie.watched.not();
    }

    fn get_index(&self, name: &str) -> usize {
        self.movies
            .binary_search_by_key(&name, |movie| &movie.name)
            .unwrap()
    }

    pub fn get_path(&self, name: &str) -> PathBuf {
        let index = self.get_index(name);
        self.movies.get(index).unwrap().path.clone()
    }

    pub fn set_watched(&mut self, name: &str) {
        let index = self.get_index(name);
        self.movies.get_mut(index).unwrap().watched = true;
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

    fn process_movie_entry(entry: &DirEntry) -> Option<Movie> {
        let path = entry.path();

        let extensions: HashSet<&OsStr> = EXTENSIONS.iter().map(OsStr::new).collect();

        if !Self::is_movie(path, &extensions).unwrap_or(false) {
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

    fn build_movies_lib(
        movies_path: &str,
        prev_movies: Option<&mut Vec<Movie>>,
        hash: Option<u64>,
        save_path: &str,
    ) -> Self {
        Self::ffmpeg_init().expect("Failed to initialize ffmpeg");

        let movies: Vec<Movie> = WalkDir::new(movies_path)
            .into_iter()
            .par_bridge()
            .filter_map(Result::ok)
            .filter_map(|entry: walkdir::DirEntry| Self::process_movie_entry(&entry))
            .collect();

        let movies_path = movies_path.to_string();
        let save_path = save_path.to_string();

        let Some(prev) = prev_movies else {
            return Self {
                movies,
                hash: Self::hash_dir(&movies_path),
                save_path,
                movies_path,
            };
        };

        prev.retain(|item| movies.clone().into_par_iter().any(|m| m.name == item.name));

        prev.par_extend(movies);
        prev.par_sort_by(|a, b| a.name.cmp(&b.name).then_with(|| b.watched.cmp(&a.watched)));
        prev.dedup_by(|a, b| b.name == a.name);

        Self {
            movies: prev.clone(),
            hash: hash.unwrap(),
            save_path,
            movies_path,
        }
    }

    fn ffmpeg_init() -> Result<(), Box<dyn Error>> {
        ffmpeg_next::init()?;
        ffmpeg_next::util::log::set_level(Quiet);
        Ok(())
    }
}
