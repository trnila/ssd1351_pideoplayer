#![allow(dead_code)]

extern crate spidev;
use std::{thread, time, fmt};
use std::time::{Instant, Duration};
use gpio_cdev::{Chip, LineRequestFlags};
use ssd1351::*;
use video::{MappedVideo, VideoAnimation};

mod framebuffer;
mod ssd1351;
mod video;

#[derive(Debug)]
enum PlayerError {
    HardwareError(ssd1351::TransferError),
    NoFrameAvailable,
}
impl std::error::Error for PlayerError {}

impl fmt::Display for PlayerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "error")
    }
}

impl std::convert::From<ssd1351::TransferError> for PlayerError {
    fn from(err: ssd1351::TransferError) -> Self {
        PlayerError::HardwareError(err)
    }    
}

struct Player<'a, ITER: Iterator<Item = &'a [u8]>> {
    display: Display<'a>,
    frames: ITER,
}

impl<'a, ITER: Iterator<Item = &'a [u8]>> Player<'a, ITER> {
    fn new(display: Display<'a>, frames: ITER) -> Self {
        Player {
            display,
            frames,
        }
    }

    fn render_next_frame(&mut self) -> Result<(), PlayerError> {
        match self.frames.next() {
            Some(frame) => Ok(self.display.render(frame)?),
            None => Err(PlayerError::NoFrameAvailable),
        }
    }
}

fn main() -> Result<(), Box<dyn ::std::error::Error>> {
    const GPIO_RST: u32 = 25;
    const GPIO_DC: u32 = 24;
    const WIDTH: usize = 128;
    const HEIGHT: usize = 128;
    const FPS: u32 = 25;

    let mut chip = Chip::new("/dev/gpiochip0")?;

    let rst = chip
        .get_line(GPIO_RST)?
        .request(LineRequestFlags::OUTPUT, 0, "oled-reset")?;
    thread::sleep(time::Duration::from_millis(1));
    rst.set_value(1)?;

    let dc = chip
        .get_line(GPIO_DC)?
        .request(LineRequestFlags::OUTPUT, 0, "oled-dc")?;

    let video = MappedVideo::new("../disp/vsb.raw", WIDTH, HEIGHT, 2)?;
    let anim = VideoAnimation::new(&video);

    let video = MappedVideo::new("../disp/bunny.raw", WIDTH, HEIGHT, 2)?;
    let anim2 = VideoAnimation::new(&video);


    let mut players = [
        Player::new(Display::new(0, 0, &dc, WIDTH, HEIGHT)?, anim),
        Player::new(Display::new(0, 1, &dc, WIDTH, HEIGHT)?, anim2),
    ];

    let frames_period = Duration::from_secs_f32(1.0 / (FPS as f32));
    loop {
        let start = Instant::now();
        for player in players.iter_mut() {
            player.render_next_frame()?;
        }

        let elapsed = start.elapsed();
        if frames_period > elapsed {
            thread::sleep(frames_period - elapsed);
        }
    }
}
