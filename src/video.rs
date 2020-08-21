use std::fs::File;
use std::io::SeekFrom;
use std::io::{Seek, Read};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VideoError {
    #[error("file size is not divisible by frame size: {file_size}")]
    CorruptedFile {
        file_size: u64,
    },
    #[error(transparent)]
    IOError(#[from] std::io::Error),
}

const FRAME_SIZE: usize = 128 * 128 * 2;

pub struct Frame(pub [u8; FRAME_SIZE]);

impl Frame {
    pub fn new() -> Self {
        Frame([0u8; FRAME_SIZE])
    }
}

pub struct Video {
    handle: File,
    frames: usize,
    pos: u64,
}

impl Video {
    pub fn new(path: &str) -> Result<Self, VideoError> {
        let file = File::open(path)?;

        let file_size = file.metadata()?.len();
        if file_size == 0 || file_size % FRAME_SIZE as u64 != 0 {
            return Err(VideoError::CorruptedFile{file_size});
        }

        Ok(Video {
            handle: File::open(path)?,
            pos: 0,
            frames: file_size as usize / FRAME_SIZE,
        })
    }

    pub fn read_frame(&mut self, n: u64) -> Result<Frame, std::io::Error> {
        let mut frame = Frame::new();
        let offset = n * frame.0.len() as u64;

        self.pos = match self.pos {
            current_pos if current_pos == offset => offset,
            _ => self.handle.seek(SeekFrom::Start(n * 128 * 128))? 
        } + frame.0.len() as u64;

        self.handle.read_exact(&mut frame.0)?;
        Ok(frame)
    }

    pub fn frames(&self) -> usize {
        self.frames
    }
}
