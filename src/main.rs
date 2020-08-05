extern crate spidev;
extern crate byteorder;
use std::{thread, time};
use memmap::MmapOptions;
use std::fs::File;
use std::error;
use std::fmt;
use gpio_cdev::{Chip, LineRequestFlags, LineHandle};

mod video;
mod ssd1351;
mod framebuffer;

use video::{MappedVideo, VideoAnimation};
use framebuffer::{FrameBuffer, Color};
use ssd1351::*;

fn main() {
    let mut chip = Chip::new("/dev/gpiochip0").unwrap();
    
    let rst = chip
        .get_line(25).unwrap()
        .request(LineRequestFlags::OUTPUT, 1, "reset").unwrap();

    rst.set_value(1).unwrap();
    thread::sleep(time::Duration::from_millis(1));
    rst.set_value(0).unwrap();
    thread::sleep(time::Duration::from_millis(1));
    rst.set_value(1).unwrap();
    

    let dc = chip
        .get_line(24).unwrap()
        .request(LineRequestFlags::OUTPUT, 0, "dc").unwrap();

    const WIDTH: usize = 128;
    const HEIGHT: usize = 128;

    let mut fb = FrameBuffer::new();
    let mut disp = Display::new(0, 0, dc).unwrap();
    
	
	let video = MappedVideo::new("../disp/vsb.raw", WIDTH, HEIGHT, 2).unwrap();
	let anim = VideoAnimation::new(&video);
	for frame in anim {
		disp.render(frame);
	}
}
