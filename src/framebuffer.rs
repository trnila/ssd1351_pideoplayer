pub struct FrameBuffer {
    fb: [u8; 128 * 128 * 2],
}

pub struct Color (u16);
impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
	   Color(((b as u16 >> 3) << 3) | ((r as u16 >> 3) << 8) | (g as u16 >> 3 >> 2) | ((g as u16 >> 5) << 5 << 8))
    }
}

impl FrameBuffer {
    pub fn new() -> Self {
        FrameBuffer {
            fb: [0u8; 128 * 128 * 2]
        }
    }

    pub fn set(&mut self, x: usize, y: usize, color: Color) {
        let idx = (y * 128 + x) * 2;
        self.fb[idx + 1] = (color.0 >> 8) as u8;
        self.fb[idx + 0] = color.0 as u8;
    }
}


