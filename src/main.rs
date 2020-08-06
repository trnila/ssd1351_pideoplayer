extern crate spidev;
use std::{thread, time};
use gpio_cdev::{Chip, LineRequestFlags};

mod video;
mod ssd1351;
mod framebuffer;

use video::{MappedVideo, VideoAnimation};
use ssd1351::*;

fn main() -> Result<(), Box<dyn ::std::error::Error>> {
    let mut chip = Chip::new("/dev/gpiochip0")?;
    
    let rst = chip
        .get_line(25)?
        .request(LineRequestFlags::OUTPUT, 1, "oled-reset")?;

    rst.set_value(1)?;
    thread::sleep(time::Duration::from_millis(1));
    rst.set_value(0)?;
    thread::sleep(time::Duration::from_millis(1));
    rst.set_value(1)?;
    

    let dc = chip
        .get_line(24)?
        .request(LineRequestFlags::OUTPUT, 0, "oled-dc")?;

    const WIDTH: usize = 128;
    const HEIGHT: usize = 128;

    let mut disp = Display::new(0, 0, dc)?;
	
	let video = MappedVideo::new("../disp/vsb.raw", WIDTH, HEIGHT, 2)?;
	let anim = VideoAnimation::new(&video);
	for frame in anim {
		disp.render(frame)?;
	}
    Ok(())
}
