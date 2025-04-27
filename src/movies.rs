use std::{
    collections::HashSet,
    error::Error,
    ffi::OsStr,
    fs,
    hash::{DefaultHasher, Hash, Hasher},
    path::{Path, PathBuf},
};

use ffmpeg_next::log::Level::Quiet;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use rayon::prelude::*;

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct Movie {
    pub name: String,
    pub path: PathBuf,
    pub watched: bool,
}

impl Movie {
    pub fn toggle_watched(&mut self) {
        let movie = self;
        movie.watched = !movie.watched;
    }
}

#[derive(Serialize, Deserialize, Clone)]

pub struct MoviesLib {
    pub movies: Vec<Movie>,
    hash: u64,
}

const SAVE_FILE: &str = "movies.json";
const EXTENSIONS: [&str; 2] = ["mkv", "mp4"];

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

    pub fn get_mut_movie(&mut self, index: usize) -> Option<&mut Movie> {
        self.movies.get_mut(index)
    }
    pub fn get_movie(&self, index: usize) -> Option<&Movie> {
        self.movies.get(index)
    }

    pub fn save_movies(&self) -> Result<(), Box<dyn Error>> {
        let json = serde_json::to_string_pretty(&self)?;
        fs::write(SAVE_FILE, json)?;
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

    fn build_movies_lib(
        movies_path: &str,
        movies_lib: Option<MoviesLib>,
        current_hash: Option<u64>,
    ) -> Result<Self, Box<dyn Error>> {
        ffmpeg_next::init()?;
        ffmpeg_next::util::log::set_level(Quiet);

        let extensions: HashSet<&OsStr> = EXTENSIONS.iter().map(OsStr::new).collect();

        let existing_movies = movies_lib.as_ref().map(|lib| {
            lib.movies
                .par_iter()
                .map(|movie| movie.name.clone())
                .collect::<HashSet<_>>()
        });

        let movies = WalkDir::new(movies_path)
            .into_iter()
            .par_bridge()
            .filter_map(Result::ok)
            .filter_map(|entry| {
                let path = entry.path();

                if !Self::is_movie(path, &extensions).unwrap_or(false) {
                    return None;
                }

                let file_name = path.file_name()?;
                let name = file_name.to_owned().into_string().ok()?;

                if let Some(ref existing) = existing_movies {
                    if existing.contains(&name) {
                        return None;
                    }
                }

                Some(Movie {
                    name,
                    path: path.to_path_buf(),
                    watched: false,
                })
            })
            .collect();

        Ok(match movies_lib {
            Some(mut lib) => {
                lib.movies.extend(movies);
                lib.hash = current_hash.unwrap();
                lib
            }
            None => MoviesLib {
                movies,
                hash: Self::hash_dir(movies_path)?,
            },
        })
    }
}
