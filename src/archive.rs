use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::collector::Collector;
use crate::movie::Movie;

#[cfg(debug_assertions)]
const SAVE_FILE: &str = ".debug_cineteca.json";
#[cfg(not(debug_assertions))]
const SAVE_FILE: &str = ".movies.json";

#[derive(Serialize, Deserialize, Clone)]
pub struct Archive {
    pub movies: Vec<Movie>,
    hash: u64,
    save_path: PathBuf,
    path: PathBuf,
}

impl Archive {
    pub fn init(path: &Path) -> Self {
        let save_path = path.join(SAVE_FILE);
        Self::load_saved(&save_path).unwrap_or_else(|_| Self::new(path, &save_path))
    }

    pub fn update(&mut self, new_movies: Vec<Movie>, new_hash: u64) {
        if new_hash != self.hash {
            self.movies = new_movies;
            self.hash = new_hash;
        }
    }

    fn get_index(&self, name: &str) -> usize {
        self.movies
            .binary_search_by_key(&name, |movie| movie.name())
            .unwrap()
    }

    pub fn get_path(&self, name: &str) -> &Path {
        self.movies.get(self.get_index(name)).unwrap().path()
    }

    pub fn toggle_watched(&mut self, name: &str) {
        let index = self.get_index(name);
        self.movies.get_mut(index).unwrap().toggle_watched();
    }

    pub fn set_watched(&mut self, name: &str) {
        let index = self.get_index(name);
        self.movies.get_mut(index).unwrap().set_watched();
    }

    fn load_saved(save_path: &Path) -> Result<Self, Box<dyn Error>> {
        let json = fs::read_to_string(save_path)?;
        let movies = serde_json::from_str(&json)?;
        Ok(movies)
    }

    pub fn save(&self) -> Result<(), Box<dyn Error>> {
        let json = serde_json::to_string_pretty(&self)?;
        fs::write(&self.save_path, json)?;
        Ok(())
    }

    fn new(path: &Path, save_path: &Path) -> Self {
        let (movies, hash) = Collector::collect(path);
        Self {
            movies,
            hash,
            save_path: save_path.to_path_buf(),
            path: path.to_path_buf(),
        }
    }
}
