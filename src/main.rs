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
use ssd1351::Display;

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
    let mut disp = Display::new(0, 0, dc);

    disp.configa(0xFD, &[0x12u8]);               // Unlock IC MCU interface
    disp.configa(0xFD, &[0xB1]);               // Command A2,B1,B3,BB,BE,C1 accessible if in unlock state
    disp.configa(0xAE, &[]);                     // Display off
    disp.configa(0xB3, &[0xF1]);               // Clock divider
    disp.configa(0xCA, &[0x7F]);               // Mux ratio
    disp.configa(0x15, &[0x00, WIDTH as u8 - 1]);    // Set column address
    disp.configa(0x75, &[0x00, HEIGHT as u8 - 1]);   // Set row address
    disp.configa(0xA0, &[0x70 | 0x00]);  // Segment remapping
    disp.configa(0xA1, &[0x00]);               // Set Display start line
    disp.configa(0xA2, &[0x00]);               // Set display offset
    disp.configa(0xB5, &[0x00]);               // Set GPIO
    disp.configa(0xAB, &[0x01]);               // Function select (internal - diode drop);
    disp.configa(0xB1, &[0x32]);               // Precharge
    disp.configa(0xB4, &[0xA0, 0xB5, 0x55]);   // Set segment low voltage
    disp.configa(0xBE, &[0x05]);               // Set VcomH voltage
    disp.configa(0xC7, &[0x0F]);               // Contrast master
    disp.configa(0xB6, &[0x01]);               // Precharge2
    disp.configa(0xA6, &[]);                     // Normal display
    disp.configa(0xc1, &[0xff, 0xff, 0xff]);
    disp.configa(0xAf, &[]);                     // Display off
    disp.configa(0x15, &[0, 127]);                     // Write RAM
    disp.configa(0x75, &[0, 127]);                     // Write RAM
    disp.configa(0x5C, &[]);                     // Write RAM

    /*
    disp.config(CommandSetCommandLock(MCUProtection::Unlock));
    disp.config(CommandSetCommandLock(MCUProtection::AdditionalCommandsInUnlock));
    disp.config(CommandSetSleepModeOn); 
    disp.config(CommandSetSleepModeOff); 
    disp.config(CommandSetColumnAddress(0, width - 1));
    disp.config(CommandSetRowAddress(0, height - 1));
    disp.config(CommandWriteRam);
    */

    /*
    disp.config(CommandFrontClockDivOscilatorFreq {
        frontClkDiv: FrontClockDiv::Div1024,
        oscilatorFreq: 
    });
    */

    fb.set(0, 0, Color::new(255, 0, 0));
    for x in 30..50 {
        for y in 30..50 {
            fb.set(x, y, Color::new(255, 0, 0));
        }
    }


//    disp.display(&fb);

	let video = MappedVideo::new("../disp/vsb.raw", WIDTH, HEIGHT, 2).unwrap();
	let anim = VideoAnimation::new(&video);
	for frame in anim {
		disp.displaya(frame);
	}
}
