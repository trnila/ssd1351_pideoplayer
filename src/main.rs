#![allow(dead_code)]

extern crate spidev;
use std::{thread, time};
use std::time::{Instant, Duration};
use gpio_cdev::{Chip, LineRequestFlags};
use ssd1351::*;
use video::{MappedVideo, VideoAnimation};

mod framebuffer;
mod ssd1351;
mod video;

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

    let mut disp = Display::new(0, 0, dc, WIDTH, HEIGHT)?;

    let video = MappedVideo::new("../disp/vsb.raw", WIDTH, HEIGHT, 2)?;
    let anim = VideoAnimation::new(&video);
    let frames_period = Duration::from_secs_f32(1.0 / (FPS as f32));
    for frame in anim {
        let start = Instant::now();
        disp.render(frame)?;
        thread::sleep(frames_period - start.elapsed());
    }
    Ok(())
}
