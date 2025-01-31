use crate::hardware::*;

/// NES MEMORY BUS
///
/// Determines the hardware to access when given an address, serves as the linking point between
/// all hardware on the system.
pub struct Bus {
    mem: WorkMemory, // $0000-$1FFF (mirrored three times)
    //ppu: PPU, // $2000-3FFF (mirrored every 8 bytes)
    //apu: APU, // $4000-401F ($4018-1F unused)
    rom: Option<ROM>,
}

impl Bus {
    pub fn new() -> Self {
        Self {
            mem: WorkMemory::new(),
            //ppu: PPU::new(),
            //apu: APU::new(),
            rom: None,
        }
    }

    pub fn load_rom(&mut self, rom: ROM) {
        self.rom = Some(rom);
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x1FFF => {
                self.mem.read(address & 0x07FF)
            }
            //0x2000..=0x3FFF => {
            //    self.ppu.read(address & 0x200F)
            //}
            //0x4000..=0x401F => {
            //    self.apu.read(address)
            //}
            0x4020..=0xFFFF => {
                match &self.rom {
                    Some(rom) => rom.read(address),
                    None => 0
                }
            }
            _ => todo!()
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x1FFF => {
                self.mem.write(address & 0x07FF, value)
            }
            //0x2000..=0x3FFF => {
            //    //self.ppu.write(address & 0x200F, value)
            //}
            //0x4000..=0x401F => {
            //    //self.apu.write(address, value)
            //}
            0x4020..=0xFFFF => {
                //self.rom.write(address, value)
            }
            _ => todo!()
        }
    }
}

