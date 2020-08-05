use memmap::MmapOptions;
use std::fs::File;
use std::error;
use std::fmt;

pub struct MappedVideo {
	memory: memmap::Mmap,	
	frames: usize,
	frame_bytes: usize,
}

#[derive(Debug)]
pub enum VideoMapError {
	IOError(std::io::Error),
	CorruptedFile(usize)
}

impl fmt::Display for VideoMapError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			VideoMapError::CorruptedFile(size) => write!(f, "wrong file size: {}", size),
			VideoMapError::IOError(err) => write!(f, "err"),
		}
    }
}
impl From<std::io::Error> for VideoMapError {
	fn from(error: std::io::Error) -> Self {
		VideoMapError::IOError(error)
	}
}
impl MappedVideo {
	pub fn new(path: &str, width: usize, height: usize, byte_depth: usize) -> Result<Self, VideoMapError> {
		let file = File::open(path)?;
		
		let frame_bytes = width * height * byte_depth;
		let file_bytes = file.metadata()?.len() as usize;
		if file_bytes == 0 || file_bytes % frame_bytes != 0 {
			return Err(VideoMapError::CorruptedFile(file_bytes))
		}

		let mmap = unsafe { MmapOptions::new().map(&file)? };

		Ok(MappedVideo {
			memory: mmap,
			frames: file_bytes / frame_bytes,
			frame_bytes: frame_bytes,
		})
	}

	pub fn frame(&self, num: usize) -> &[u8] {
		let begin = num * self.frame_bytes;
		return &self.memory[begin..begin + self.frame_bytes];
	}
}

pub struct VideoAnimation<'a> {
	video: &'a MappedVideo,
	frame: usize,
}

impl<'a> VideoAnimation<'a> {
	pub fn new(video: &'a MappedVideo) -> Self {
		VideoAnimation {
			video,
			frame: 0,
		}
	}
}

impl<'a> Iterator for VideoAnimation<'a> {
	type Item = &'a [u8];
	fn next(&mut self) -> Option<Self::Item> {
		let current = self.frame;
		self.frame = (self.frame + 1) % self.video.frames;
		Some(self.video.frame(current))
	}
}
