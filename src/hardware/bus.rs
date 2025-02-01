use crate::hardware::*;
use std::cell::RefCell;

/// NES MEMORY BUS
///
/// Determines the hardware to access when given an address, serves as the linking point between
/// all hardware on the system.
pub struct Bus {
    mem: WorkMemory, // $0000-$1FFF (mirrored three times)
    ppu: RefCell<PPU>, // $2000-3FFF (mirrored every 8 bytes)
    apu: RefCell<APU>, // $4000-401F ($4018-1F unused)
    cart: Option<RefCell<Cart>>,
}

impl Bus {
    pub fn new(ppu: RefCell<PPU>, apu: RefCell<APU>) -> Self {
        Self {
            mem: WorkMemory::new(),
            ppu,
            apu,
            cart: None,
        }
    }

    pub fn load_cart(&mut self, cart: RefCell<Cart>) {
        self.cart = Some(cart);
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x1FFF => {
                self.mem.read(address & 0x07FF)
            }
            0x2000..=0x3FFF => {
                self.ppu.borrow_mut().read(address & 0x200F)
            }
            0x4000..=0x401F => {
                self.apu.borrow_mut().read(address)
            }
            0x4020..=0xFFFF => {
                match &self.cart {
                    Some(rom) => rom.borrow_mut().read(address),
                    None => 0
                }
            }
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x1FFF => {
                self.mem.write(address & 0x07FF, value)
            }
            0x2000..=0x3FFF => {
                self.ppu.borrow_mut().write(address & 0x200F, value)
            }
            0x4000..=0x401F => {
                self.apu.borrow_mut().write(address, value)
            }
            0x4020..=0xFFFF => {
                if let Some(cart) = &self.cart {
                    cart.borrow_mut().write(address, value)
                }
            }
        }
    }
}

