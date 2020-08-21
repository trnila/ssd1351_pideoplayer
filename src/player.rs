use std::{fs, io};
use thiserror::Error;
use crate::video::{Video, VideoError};
use crate::ssd1351::{Display, TransferError};


#[derive(Error, Debug)]
pub enum PlaybackError {
    #[error("No video in playlist")]
    NoVideo,

    #[error(transparent)]
    IOError(#[from] std::io::Error),

    #[error(transparent)]
    VideoError(#[from] VideoError),

    #[error(transparent)]
    TransferError(#[from] TransferError),
}


struct Playlist {
    path: String,
    files: Vec<String>,
    index: usize,
}

impl Playlist {
    fn from_dir(path: &str) -> Self {
        Playlist {
            path: path.to_string(),
            files: vec!(),
            index: usize::MAX,
        }
    }

    fn discover(&mut self) -> Result<(), std::io::Error> {
        self.files = fs::read_dir(&self.path)?
            .map(|res| res.map(|e| e.path().into_os_string().into_string().unwrap()))
            .collect::<Result<Vec<_>, io::Error>>()?;
        Ok(())
    }

    fn next(&mut self) -> Result<Video, VideoError> {
        if self.index >= self.files.len() {
            self.discover()?;
            self.index = 0;
        }

        let video = Video::new(&self.files[self.index])?;
        self.index += 1;
        Ok(video)
    }
}

pub struct Player<'a> {
    display: Display<'a>,
    playlist: Playlist, 
    current_video: Option<Video>,
    current_frame: usize,
}

impl<'a> Player<'a> {
    pub fn new(display: Display<'a>, playlist_dir: &str) -> Self {
        Player {
            display,
            playlist: Playlist::from_dir(playlist_dir),
            current_video: None,
            current_frame: 0,
        }
    }

    pub fn render_next_frame(&mut self) -> Result<(), PlaybackError> {
        if match &self.current_video {
            Some(video) => self.current_frame >= video.frames(),
            None => true,
        } {
            self.current_video = Some(self.playlist.next()?);
            self.current_frame = 0;
        }

        match self.current_video {
            Some(ref mut video) => {
                self.display.render(&video.read_frame(self.current_frame as u64)?)?;
                Ok(())
            },
            None => Err(PlaybackError::NoVideo),
        }
    }
}
