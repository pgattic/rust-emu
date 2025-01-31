
pub struct NESHeader {
    pub prg_size: usize,
    pub chr_size: usize,
    pub mapper_number: usize,
    pub nes2: bool,
    pub battery: bool,
    pub trainer: bool,
    pub alt_nametables: bool,
    pub nametable_layout: NameTableLayout,
    pub console_type: ConsoleType,
    pub timing_mode: TimingMode,
}

impl NESHeader {
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if !(bytes[0] == b'N' && bytes[1] == b'E' && bytes[2] == b'S' && bytes[3] == 0x1A) {
            return None;
        }
        let prg_size: usize = if bytes[9] & 0x0F == 0x0F {
            2_i32.pow(bytes[4] as u32 >> 2) as usize * ((bytes[4] as usize & 3) * 2 + 1)
        } else {
            bytes[4] as usize | bytes[9] as usize & 0x0F << 8
        };
        let chr_size: usize = if bytes[9] & 0xF0 == 0xF0 {
            2_i32.pow(bytes[5] as u32 >> 2) as usize * ((bytes[5] as usize & 3) * 2 + 1)
        } else {
            bytes[5] as usize | bytes[9] as usize & 0xF0 << 4
        };
        let nes2 = (bytes[7] & 0x0C) == 0x08;
        let mapper_number = bytes[6] as usize >> 4 | bytes[7] as usize & 0b11110000 | bytes[8] as usize & 0b1111 << 8;
        let battery = bytes[6] & 2 == 2;
        let trainer = bytes[6] & 4 == 4;
        let alt_nametables = bytes[6] & 8 == 8;
        let nametable_layout = match bytes[6] & 1 {
            0 => NameTableLayout::Vertical,
            1 => NameTableLayout::Horizontal,
            _ => unreachable!(),
        };
        let console_type = match bytes[7] & 3 {
            0 => ConsoleType::NESFami,
            1 => ConsoleType::VsSystem(bytes[13] & 0x0F, bytes[13] & 0xF0),
            2 => ConsoleType::Playchoice,
            3 => ConsoleType::Extended(bytes[13] & 0x0F),
            _ => unreachable!(),
        };
        let timing_mode = match bytes[12] & 3 {
            0 => TimingMode::NTSC,
            1 => TimingMode::PAL,
            2 => TimingMode::Multi,
            3 => TimingMode::Dendy,
            _ => unreachable!(),
        };

        Some(Self {
            prg_size,
            chr_size,
            mapper_number,
            nes2,
            battery,
            trainer,
            alt_nametables,
            nametable_layout,
            console_type,
            timing_mode,
        })
    }
}

pub enum NameTableLayout {
    Vertical,
    Horizontal,
}

pub enum ConsoleType {
    NESFami,
    VsSystem(u8, u8),
    Playchoice,
    Extended(u8),
}

pub enum TimingMode {
    NTSC,
    PAL,
    Multi,
    Dendy,
}

