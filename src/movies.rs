use std::{
    collections::HashSet,
    error::Error,
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

use ffmpeg_next::log::Level::Quiet;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

#[derive(Ord, PartialOrd, Eq, Serialize, Deserialize, PartialEq)]
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

pub struct MoviesLib {
    pub movies: Vec<Movie>,
}

const SAVE_FILE: &str = "movies.json";
const EXTENSIONS: [&str; 3] = ["mkv", "mp4", "rs"];

impl MoviesLib {
    pub fn init(movies_path: &str) -> Result<MoviesLib, Box<dyn std::error::Error>> {
        let movies = if let Ok(saved_movies) = Self::load_movies_save() {
            saved_movies
        } else {
            Self::read_movies_dir(movies_path)?
        };
        Ok(MoviesLib { movies })
    }

    pub fn get_mut_movie(&mut self, index: usize) -> &mut Movie {
        self.movies.get_mut(index).unwrap()
    }
    pub fn get_movie(&self, index: usize) -> &Movie {
        self.movies.get(index).unwrap()
    }

    pub fn save_movies(&self) -> Result<(), Box<dyn Error>> {
        let json = serde_json::to_string_pretty(&self.movies)?;
        fs::write(SAVE_FILE, json)?;
        Ok(())
    }
    fn is_movie(path: &Path, extensions: &HashSet<&OsStr>) -> Result<bool, Box<dyn Error>> {
        if !path
            .extension()
            .is_some_and(|ext| extensions.contains(&ext))
        {
            return Ok(false);
        }

        let duration = ffmpeg_next::format::input(path)?.duration() as f64
            / f64::from(ffmpeg_next::ffi::AV_TIME_BASE);

        Ok(duration.ge(&3600.0))
    }

    fn load_movies_save() -> Result<Vec<Movie>, Box<dyn Error>> {
        if Path::new(SAVE_FILE).exists() {
            let json = fs::read_to_string(SAVE_FILE)?;
            let movies: Vec<Movie> = serde_json::from_str(&json)?;
            return Ok(movies);
        }
        Err("No saved movies or empty save file".into())
    }

    fn read_movies_dir(movies_path: &str) -> Result<Vec<Movie>, Box<dyn Error>> {
        ffmpeg_next::init()?;
        ffmpeg_next::util::log::set_level(Quiet);

        let extensions: HashSet<&OsStr> = EXTENSIONS.iter().map(OsStr::new).collect();

        let movies: Vec<Movie> = WalkDir::new(movies_path)
            .max_depth(4)
            .into_iter()
            .filter_map(|res| res.ok())
            .filter_map(|entry| {
                let path = entry.path();
                if Self::is_movie(path, &extensions).is_ok_and(|result| result) {
                    Some(Movie {
                        name: path.file_name()?.to_owned().into_string().ok()?,
                        path: path.to_path_buf(),
                        watched: false,
                    })
                } else {
                    None
                }
            })
            .collect();
        Ok(movies)
    }
}
