use std::{
    error::Error,
    fs,
    hash::Hash,
    mem,
    ops::Not,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::movies_collector::MovieCollector;

const SAVE_FILE: &str = ".media.json";

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Hash)]
pub struct Movie {
    pub name: String,
    pub path: PathBuf,
    pub watched: bool,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Hash)]
pub struct Show {
    pub name: String,
    pub seasons: Vec<Season>,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Hash)]
pub struct Season {
    pub name: String,
    pub episodes: Vec<Episode>,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Hash)]
pub struct Episode {
    pub name: String,
    pub path: PathBuf,
    pub watched: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MoviesArchive {
    pub movies: Vec<Movie>,
    hash: u64,

    #[serde(skip)]
    save_path: String,
    #[serde(skip)]
    movies_path: String,
}

impl MoviesArchive {
    pub fn init(movies_path: &str) -> Self {
        let save_path = format!("{movies_path}/{SAVE_FILE}");

        if let Some(mut saved_movies) = Self::load_saved_movies(&save_path) {
            saved_movies.movies_path = movies_path.to_string();
            saved_movies.save_path = save_path;
            saved_movies
        } else {
            Self::build_archive(movies_path, None, None, None, &save_path)
        }
    }

    pub fn refresh(&mut self) {
        let (movies, hash) = MovieCollector::collect_movies(&self.movies_path);
        if hash != self.hash {
            *self = Self::build_archive(
                &self.movies_path,
                Some(&mut self.movies),
                Some(&movies),
                Some(hash),
                &self.save_path,
            );
        }
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

    pub fn save_movies(&self) -> Result<(), Box<dyn Error>> {
        let json = serde_json::to_string_pretty(&self)?;
        fs::write(&self.save_path, json)?;
        Ok(())
    }

    fn build_archive(
        movies_path: &str,
        prev_movies: Option<&mut Vec<Movie>>,
        movies: Option<&Vec<Movie>>,
        hash: Option<u64>,
        save_path: &str,
    ) -> Self {
        let movies_path = movies_path.to_string();
        let save_path = save_path.to_string();

        let Some(prev) = prev_movies else {
            let (movies, hash) = MovieCollector::collect_movies(&movies_path);
            return Self {
                movies,
                hash,
                save_path,
                movies_path,
            };
        };

        let movies = movies.unwrap();

        prev.retain(|item| movies.clone().iter().any(|m| m.name == item.name));

        prev.extend(movies.clone());
        prev.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| b.watched.cmp(&a.watched)));
        prev.dedup_by(|a, b| b.name == a.name);

        Self {
            movies: mem::take(prev),
            hash: hash.unwrap(),
            save_path,
            movies_path,
        }
    }
}
