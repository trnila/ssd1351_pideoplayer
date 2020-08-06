use gpio_cdev::LineHandle;
use spidev::{SpiModeFlags, Spidev, SpidevOptions};
use std::fmt;
use std::io::Write;

pub struct Display {
    spi: Spidev,
    dc: LineHandle,
    width: usize,
    height: usize,
}

pub enum Interface {
    Parallel8 = 0b00,
    Parallel16 = 0b01,
    Parallel18 = 0b11,
}

pub enum PinState {
    HiZInputDisabled,
    HiZInputEnabled,
    OutputLow,
    OutputHigh,
}

pub enum ScrollingInterval {
    TestMode,
    Normal,
    Slow,
    Slowest,
}

pub enum ColorDepth {
    Color65k = 0b00,
    Color262k = 0b10,
    Color262kFmt2 = 0b11,
}

pub enum Command {
    SetColumnAddress(u8, u8),
    SetRowAddress(u8, u8),
    WriteRam,
    ReadRam,
    SetReMapColorDepth {
        vert_increment: bool,
        column_invert: bool,
        swap_colors: bool,
        scan_from_n: bool,
        enable_com_split: bool,
        color_depth: ColorDepth,
    },
    SetDisplayStartLine(u8),
    SetDisplayOffset(u8),
    SetDisplayModeOff,
    SetDisplayModeOn,
    SetDisplayModeNormal,
    SetDisplayModeInverse,
    FunctionSelection {
        internal_regulator: bool,
        interface: Interface,
    },
    Nop,
    SetSleepModeOn,
    SetSleepModeOff,
    SetResetPreCharge {
        phase_1: u8,
        phase_2: u8,
    },
    DisplayEnhancement(bool),
    Clocks {
        divider: ClkDiv,
        osc_freq: u8,
    },
    SetSegmentLowVoltage {
        external: bool,
    },
    SetGPIO {
        gpio0: PinState,
        gpio1: PinState,
    },
    SetSecondPreChargePeriod {
        precharge_period: u8,
        gs_table: [u8; 63],
    },
    UseBuiltInLUT,
    SetPreChargeVoltage(u8),
    SetVComVoltage(u8),
    SetContrast {
        r: u8,
        g: u8,
        b: u8,
    },
    MasterContrast(u8),
    SetMuxRatio(u8),
    SetCommandLock(MCUProtection),
    HorizontalScroll {
        scroll: u8,
        start_row: u8,
        rows: u8,
        interval: ScrollingInterval,
    },
    StopMoving,
    StartMoving,
}

pub enum ClkDiv {
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

pub enum MCUProtection {
    Unlock = 0x12,
    LockCommands = 0x16,
    AdditionalCommandsInaccessible = 0xb0,
    AdditionalCommandsInUnlock = 0xb1,
}

#[derive(Debug)]
pub enum TransferError {
    SPIError(std::io::Error),
    GPIOError(gpio_cdev::errors::Error),
}
impl std::error::Error for TransferError {}

impl fmt::Display for TransferError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "err")
    }
}

impl From<std::io::Error> for TransferError {
    fn from(error: std::io::Error) -> Self {
        TransferError::SPIError(error)
    }
}

impl From<gpio_cdev::errors::Error> for TransferError {
    fn from(error: gpio_cdev::errors::Error) -> Self {
        TransferError::GPIOError(error)
    }
}

impl Display {
    pub fn new(
        spidev: u32,
        cs: u32,
        dc_line: LineHandle,
        width: usize,
        height: usize,
    ) -> Result<Self, TransferError> {
        let mut spi = Spidev::open(format!("/dev/spidev{}.{}", spidev, cs))?;
        let options = SpidevOptions::new()
            .bits_per_word(8)
            .max_speed_hz(20_000_000)
            .mode(SpiModeFlags::SPI_MODE_0)
            .build();
        spi.configure(&options)?;
        let mut disp = Display {
            spi,
            dc: dc_line,
            width,
            height,
        };
        disp.init()?;
        Ok(disp)
    }

    fn init(&mut self) -> Result<(), TransferError> {
        self.config(Command::SetCommandLock(MCUProtection::Unlock))?;
        self.config(Command::SetCommandLock(
            MCUProtection::AdditionalCommandsInUnlock,
        ))?;
        self.config(Command::SetSleepModeOn)?;
        self.config(Command::Clocks {
            divider: ClkDiv::Div2,
            osc_freq: 0xF,
        })?;
        self.config(Command::SetMuxRatio(0x7f))?;
        self.config(Command::SetColumnAddress(0, self.width as u8 - 1))?;
        self.config(Command::SetRowAddress(0, self.height as u8 - 1))?;
        self.config(Command::SetReMapColorDepth {
            vert_increment: false,
            column_invert: false,
            swap_colors: false,
            scan_from_n: true,
            enable_com_split: true,
            color_depth: ColorDepth::Color65k,
        })?;
        self.config(Command::SetDisplayStartLine(0))?;
        self.config(Command::SetDisplayOffset(0))?;
        self.config(Command::SetGPIO {
            gpio0: PinState::HiZInputDisabled,
            gpio1: PinState::HiZInputDisabled,
        })?;
        self.config(Command::FunctionSelection {
            internal_regulator: true,
            interface: Interface::Parallel8,
        })?;

        //
        self.config(Command::SetResetPreCharge {
            phase_1: 0x2,
            phase_2: 0x3,
        })?;
        self.config(Command::SetSegmentLowVoltage { external: true })?;
        self.config(Command::SetVComVoltage(0x05))?;
        self.config(Command::MasterContrast(0x0F))?;
        self.config(Command::SetPreChargeVoltage(0x01))?;
        self.config(Command::SetDisplayModeNormal)?;
        self.config(Command::SetContrast {
            r: 0xff,
            g: 0xff,
            b: 0xff,
        })?;
        self.config(Command::SetSleepModeOff)?;
        self.config(Command::WriteRam)?;
        Ok(())
    }

    pub fn render(&mut self, fb: &[u8]) -> Result<(), TransferError> {
        self.data_mode(true)?;
        self.xmit(fb)
    }

    fn xmit(&mut self, data: &[u8]) -> Result<(), TransferError> {
        self.spi.write(&data)?;
        Ok(())
    }

    fn data_mode(&mut self, data: bool) -> Result<(), TransferError> {
        Ok(self.dc.set_value(data as u8)?)
    }

    pub fn config(&mut self, option: Command) -> Result<(), TransferError> {
        let (cmd, data) = match option {
            Command::SetColumnAddress(start, end) => (0x15, vec![start, end]),
            Command::SetRowAddress(start, end) => (0x75, vec![start, end]),
            Command::WriteRam => (0x5c, Vec::new()),
            Command::ReadRam => (0x5d, Vec::new()),
            Command::SetSleepModeOn => (0xae, Vec::new()),
            Command::SetSleepModeOff => (0xaf, Vec::new()),
            Command::SetCommandLock(prot) => (0xfd, vec![prot as u8]),
            Command::Clocks { divider, osc_freq } => match osc_freq {
                0..=0b1111 => (0xb3, vec![divider as u8 | ((osc_freq as u8) << 4)]),
                _ => unreachable!(),
            },
            Command::SetMuxRatio(ratio) => match ratio {
                15..=127 => (0xca, vec![ratio]),
                _ => unreachable!(),
            },
            Command::SetReMapColorDepth {
                vert_increment,
                column_invert,
                swap_colors,
                scan_from_n,
                enable_com_split,
                color_depth,
            } => (
                0xa0,
                vec![
                    (vert_increment as u8)
                        | ((column_invert as u8) << 1)
                        | ((swap_colors as u8) << 2)
                        | ((scan_from_n as u8) << 4)
                        | ((enable_com_split as u8) << 5)
                        | ((color_depth as u8) << 6),
                ],
            ),
            Command::SetDisplayStartLine(line) => match line {
                0..=127 => (0xA1, vec![line]),
                _ => unreachable!(),
            },
            Command::SetDisplayOffset(line) => match line {
                0..=127 => (0xA2, vec![line]),
                _ => unreachable!(),
            },
            Command::SetGPIO { gpio0, gpio1 } => (0xb5, vec![(gpio0 as u8) | ((gpio1 as u8) >> 2)]),
            Command::FunctionSelection {
                internal_regulator,
                interface,
            } => (
                0xab,
                vec![(internal_regulator as u8) | ((interface as u8) << 6)],
            ),
            Command::SetResetPreCharge { phase_1, phase_2 } => match (phase_1, phase_2) {
                (2..=15, 3..=15) => (0xb1, vec![phase_1 | (phase_2 << 4)]),
                _ => unreachable!(),
            },
            Command::SetSegmentLowVoltage { external } => {
                (0xb4, vec![0xa0 | external as u8, 0b10110101, 0b01010101])
            }
            Command::SetVComVoltage(val) => match val {
                0..=0b111 => (0xbe, vec![val]),
                _ => unreachable!(),
            },
            Command::MasterContrast(val) => match val {
                0..=0b1111 => (0xc7, vec![val]),
                _ => unreachable!(),
            },
            Command::SetPreChargeVoltage(val) => match val {
                0..=0b11111 => (0xbb, vec![val]),
                _ => unreachable!(),
            },
            Command::SetContrast { r, g, b } => (0xc1, vec![r, g, b]),
            Command::SetDisplayModeOff => (0xa4, Vec::new()),
            Command::SetDisplayModeOn => (0xa5, Vec::new()),
            Command::SetDisplayModeNormal => (0xa6, Vec::new()),
            Command::SetDisplayModeInverse => (0xa7, Vec::new()),

            _ => unreachable!(),
        };

        self.data_mode(false)?;
        self.xmit(&[cmd])?;

        if !data.is_empty() {
            self.data_mode(true)?;
            self.xmit(&data)?;
        }
        Ok(())
    }
}
