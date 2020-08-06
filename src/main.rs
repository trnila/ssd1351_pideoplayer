#![allow(dead_code)]

extern crate spidev;
use std::{thread, time};
use gpio_cdev::{Chip, LineRequestFlags};
use video::{MappedVideo, VideoAnimation};
use ssd1351::*;

mod video;
mod ssd1351;
mod framebuffer;

fn main() -> Result<(), Box<dyn ::std::error::Error>> {
    const GPIO_RST: u32 = 25;
    const GPIO_DC: u32 = 24;
    const WIDTH: usize = 128;
    const HEIGHT: usize = 128;

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
	for frame in anim {
		disp.render(frame)?;
	}
    Ok(())
}
