#![allow(dead_code)]

extern crate spidev;
use std::{thread, time};
use std::time::{Instant, Duration};
use gpio_cdev::{Chip, LineRequestFlags};
use ssd1351::*;
use player::Player;

mod framebuffer;
mod ssd1351;
mod video;
mod player;

fn main() -> Result<(), Box<dyn ::std::error::Error>> {
    const GPIO_RST: u32 = 24;
    const GPIO_DC: u32 = 25;
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

    let mut players = [
        Player::new(Display::new(0, 0, &dc, WIDTH, HEIGHT)?, "1"),
        Player::new(Display::new(0, 1, &dc, WIDTH, HEIGHT)?, "2"),
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
