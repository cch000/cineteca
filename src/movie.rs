use std::{
    error::Error,
    path::{Path, PathBuf},
    time::SystemTime,
};

use serde::{Deserialize, Serialize};

const MIN_LENGTH: i64 = 3600;
const EXTENSIONS: [&str; 4] = ["mkv", "mp4", "avi", "mov"];

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Hash)]
pub struct MediaPath(PathBuf);

impl TryFrom<&Path> for MediaPath {
    type Error = Box<dyn Error>;

    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        Ok(Self(
            path.extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| EXTENSIONS.contains(&ext))
                .then(|| path.to_path_buf())
                .ok_or("Invalid or missing file extension")?,
        ))
    }
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Hash)]
struct MovieLength(i64);

impl TryFrom<&Path> for MovieLength {
    type Error = Box<dyn Error>;
    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        let length = ffmpeg_next::format::input(&path)?.duration()
            / i64::from(ffmpeg_next::ffi::AV_TIME_BASE);

        Ok(Self(
            (length > MIN_LENGTH)
                .then_some(length)
                .ok_or("Length below minimum")?,
        ))
    }
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Hash)]
pub struct Movie {
    name: String,
    path: MediaPath,
    length: MovieLength,
    since_watched: Option<SystemTime>,
}

impl Movie {
    pub const fn path(&self) -> &PathBuf {
        &self.path.0
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub const fn since_watched(&self) -> Option<SystemTime> {
        self.since_watched
    }

    pub fn pretty_length(&self) -> String {
        let minutes = self.length.0 / 60 % 60 + 1;
        let hours = self.length.0 / 3600;

        match minutes {
            0 => {
                format!("{hours}h")
            }

            1..60 => {
                format!("{hours}h {minutes}m")
            }
            60 => {
                let correction = hours + 1;
                format!("{correction}h")
            }
            _ => String::new(),
        }
    }

    pub fn pretty_since_watched(&self) -> String {
        self.since_watched.map_or_else(
            || "Not yet".to_string(),
            |time| {
                let hours_since = SystemTime::now()
                    .duration_since(time)
                    .unwrap_or_default()
                    .as_secs()
                    / 3600;

                match hours_since {
                    0 => "<1h ago".to_string(),
                    1 => "1h ago".to_string(),
                    2..24 => format!("{hours_since}h ago"),
                    24.. => format!("{}d ago", hours_since / 24),
                }
            },
        )
    }

    pub fn toggle_watched(&mut self) {
        match self.since_watched {
            Some(_) => self.since_watched = None,
            None => self.since_watched = Some(SystemTime::now()),
        }
    }

    pub fn set_watched(&mut self) {
        self.since_watched = Some(SystemTime::now());
    }
}

impl TryFrom<&Path> for Movie {
    type Error = Box<dyn Error>;

    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        let name = path
            .file_name()
            .ok_or("Path does not have a valid filename")?
            .to_string_lossy()
            .into_owned();

        Ok(Self {
            name,
            path: MediaPath::try_from(path)?,
            length: MovieLength::try_from(path)?,
            since_watched: None,
        })
    }
}
