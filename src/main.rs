extern crate spidev;
extern crate byteorder;
use std::io::Write;
use spidev::{Spidev, SpidevOptions, SpidevTransfer, SpiModeFlags};
use gpio_cdev::{Chip, LineRequestFlags, LineHandle};
use byteorder::{LittleEndian, WriteBytesExt};
use std::{thread, time};
use memmap::MmapOptions;
use std::fs::File;
use std::error;
use std::fmt;



struct ssd1351 {
   spi: Spidev,
   dc: LineHandle,
}

enum Command {
   SetColumnAddress(u8, u8),
   SetRowAddress(u8, u8),
   WriteRam,
   ReadRam,
   SetReMapColorDepth{
    verticalAddressIncrement: bool,
    columnAddress127ToSeg0: bool,
    colorSwapped: bool,
    // TODO: xd
   },
   SetDisplayStartLine(u8),
   SetDisplayOffset(u8),
   SetDisplayModeOff,
   SetDisplayModeOn,
   SetDisplayModeNormal,
   SetDisplayModeInverse,
   FunctionSelection{
    intervalRegulator: bool,
    //mode
   },
   Nop,
   SetSleepModeOn,
   SetSleepModeOff,
   Nop2,
   SetResetPreCharge, //
   DisplayEnhancement, //

   FrontClockDivOscilatorFreq {
    frontClkDiv: FrontClockDiv,
    oscilatorFreq: u8,
   },


   SleepOn,
   SetCommandLock(MCUProtection)
}

enum FrontClockDiv {
    Div1,
    Div2,
    Div4,
    Div8,
    Div16,
    Div32,
    Div64,
    Div128,
    Div256,
    Div512,
    Div1024,
}

enum MCUProtection {
    Unlock = 0x12,
    LockCommands = 0x16,
    AdditionalCommandsInaccessible = 0xb0,
    AdditionalCommandsInUnlock = 0x1b,
}

impl ssd1351 {
    fn new(spidev: u32, cs: u32, dc_line: LineHandle) -> Self {
        let mut spi = Spidev::open("/dev/spidev0.0").unwrap();
		let options = SpidevOptions::new()
			 .bits_per_word(8)
			 .max_speed_hz(5_000_000)
			 .mode(SpiModeFlags::SPI_MODE_0)
			 .build();
		spi.configure(&options).unwrap();
        let ret = ssd1351 {
            spi,
            dc: dc_line
        };

        ret
    }

    fn display(&mut self, fb: &FrameBuffer) {
        self.dc.set_value(1).unwrap();
        self.spi.write(&fb.fb);
    }
    fn displaya(&mut self, fb: &[u8]) {
        self.dc.set_value(1).unwrap();
        self.spi.write(&fb);
    }

    fn configa(&mut self, cmd: u8, data: &[u8]) {
        self.dc.set_value(0).unwrap();
        self.spi.write(&[cmd]);

        if !data.is_empty() {
            self.dc.set_value(1).unwrap();
            self.spi.write(&data);
        }

    }

    fn config(&mut self, option: Command) {
        let (cmd, data) = match option {
            Command::SetColumnAddress(start, end) => (0x15, vec![start, end]),
            Command::SetRowAddress(start, end) => (0x75, vec![start, end]),
            Command::WriteRam => (0x5c, Vec::new()),
            Command::ReadRam => (0x5d, Vec::new()),


        


            Command::SleepOn => (0xFF, Vec::new()),

            Command::SetSleepModeOn => (0xae, Vec::new()),
            Command::SetSleepModeOff => (0xaf, Vec::new()),
            Command::SetCommandLock(prot) => (0xfd, vec![prot as u8]),

            Command::FrontClockDivOscilatorFreq{frontClkDiv, oscilatorFreq} => 
                match oscilatorFreq {
                    0..=0b1111 => (0xb3, vec![
                                   frontClkDiv as u8 | ((oscilatorFreq as u8) << 4)
                    ]),
                    _ => unreachable!(),
                }
            ,

            _ => panic!("xd"),
        };        

        self.dc.set_value(0).unwrap();
        self.spi.write(&[cmd]);

        if !data.is_empty() {
            self.dc.set_value(1).unwrap();
            self.spi.write(&data);
        }
    }
}

struct FrameBuffer {
    fb: [u8; 128 * 128 * 2],
}

struct Color (u16);
impl Color {
    fn new(r: u8, g: u8, b: u8) -> Self {
	   Color(((b as u16 >> 3) << 3) | ((r as u16 >> 3) << 8) | (g as u16 >> 3 >> 2) | ((g as u16 >> 5) << 5 << 8))
    }
}

impl FrameBuffer {
    fn new() -> Self {
        FrameBuffer {
            fb: [0u8; 128 * 128 * 2]
        }
    }

    fn set(&mut self, x: usize, y: usize, color: Color) {
        let idx = (y * 128 + x) * 2;
        self.fb[idx + 1] = (color.0 >> 8) as u8;
        self.fb[idx + 0] = color.0 as u8;
    }
}

struct MappedVideo {
	memory: memmap::Mmap,	
	frames: usize,
	frame_bytes: usize,
}

#[derive(Debug)]
enum VideoMapError {
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
	fn new(path: &str, width: usize, height: usize, byte_depth: usize) -> Result<Self, VideoMapError> {
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

	fn frame(&self, num: usize) -> &[u8] {
		let begin = num * self.frame_bytes;
		return &self.memory[begin..begin + self.frame_bytes];
	}
}

struct VideoAnimation<'a> {
	video: &'a MappedVideo,
	frame: usize,
}

impl<'a> VideoAnimation<'a> {
	fn new(video: &'a MappedVideo) -> Self {
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
    let mut disp = ssd1351::new(0, 0, dc);

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


    disp.display(&fb);

	let video = MappedVideo::new("../disp/vsb.raw", WIDTH, HEIGHT, 2).unwrap();
	let anim = VideoAnimation::new(&video);
	for frame in anim {
		disp.displaya(frame);
	}
}
