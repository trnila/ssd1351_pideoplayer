use std::io::Write;
use spidev::{Spidev, SpidevOptions, SpidevTransfer, SpiModeFlags};
use gpio_cdev::{Chip, LineRequestFlags, LineHandle};
use byteorder::{LittleEndian, WriteBytesExt};

pub struct Display {
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

impl Display {
    pub fn new(spidev: u32, cs: u32, dc_line: LineHandle) -> Self {
        let mut spi = Spidev::open("/dev/spidev0.0").unwrap();
		let options = SpidevOptions::new()
			 .bits_per_word(8)
			 .max_speed_hz(5_000_000)
			 .mode(SpiModeFlags::SPI_MODE_0)
			 .build();
		spi.configure(&options).unwrap();
        Display {
            spi,
            dc: dc_line
        }
    }

    pub fn displaya(&mut self, fb: &[u8]) {
        self.dc.set_value(1).unwrap();
        self.spi.write(&fb);
    }

    pub fn configa(&mut self, cmd: u8, data: &[u8]) {
        self.dc.set_value(0).unwrap();
        self.spi.write(&[cmd]);

        if !data.is_empty() {
            self.dc.set_value(1).unwrap();
            self.spi.write(&data);
        }

    }

    pub fn config(&mut self, option: Command) {
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
