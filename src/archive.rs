use std::{
    error::Error,
    fs, mem,
    path::{Path, PathBuf},
    time::SystemTime,
};

use serde::{Deserialize, Serialize};

use crate::collector::{Collector, Movie};

const SAVE_FILE: &str = ".movies.json";

#[derive(Serialize, Deserialize, Clone)]
pub struct Archive {
    pub movies: Vec<Movie>,
    hash: u64,

    #[serde(skip)]
    save_path: PathBuf,
    #[serde(skip)]
    path: PathBuf,
}

impl Archive {
    pub fn init(path: &Path) -> Self {
        let save_path = PathBuf::from(format!("{}/{SAVE_FILE}", path.to_str().unwrap()));

        if let Some(mut saved) = Self::load_saved(&save_path) {
            saved.path = path.to_path_buf();
            saved.save_path = save_path;
            saved
        } else {
            Self::build_archive(path, None, None, None, &save_path)
        }
    }

    pub fn update(&mut self, data: &[Movie], hash: u64) {
        if hash != self.hash {
            *self = Self::build_archive(
                &self.path,
                Some(&mut self.movies),
                Some(data),
                Some(hash),
                &self.save_path,
            );
        }
    }

    pub fn toggle_watched(&mut self, name: &str) {
        let index = self.get_index(name);
        let movie = self.movies.get_mut(index).unwrap();

        match movie.date_watched {
            Some(_) => movie.date_watched = None,
            None => movie.date_watched = Some(SystemTime::now()),
        }
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
        self.movies.get_mut(index).unwrap().date_watched = Some(SystemTime::now());
    }

    fn load_saved(save_path: &Path) -> Option<Self> {
        if Path::new(save_path).exists() {
            let json = fs::read_to_string(save_path).unwrap();

            let movies = serde_json::from_str(&json).expect("Error parsing json");
            Some(movies)
        } else {
            None
        }
    }

    pub fn save(&self) -> Result<(), Box<dyn Error>> {
        let json = serde_json::to_string_pretty(&self)?;
        fs::write(&self.save_path, json)?;
        Ok(())
    }

    fn build_archive(
        path: &Path,
        prev: Option<&mut Vec<Movie>>,
        movies: Option<&[Movie]>,
        hash: Option<u64>,
        save_path: &Path,
    ) -> Self {
        let Some(prev) = prev else {
            let (movies, hash) = Collector::collect(path);
            return Self {
                movies,
                hash,
                save_path: save_path.to_path_buf(),
                path: path.to_path_buf(),
            };
        };

        let movies = movies.unwrap();

        prev.retain(|item| movies.iter().any(|m| m.name == item.name));

        prev.extend(movies.to_vec());
        prev.sort_by(|a, b| a.name.cmp(&b.name));
        prev.dedup_by(|a, b| b.name == a.name);

        Self {
            movies: mem::take(prev),
            hash: hash.unwrap(),
            save_path: save_path.to_path_buf(),
            path: path.to_path_buf(),
        }
    }
}
