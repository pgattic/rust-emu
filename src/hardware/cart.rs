use crate::header::NESHeader;

pub struct Cart {
    header: NESHeader,
    data: Vec<u8>,
}

impl Cart {
    pub fn new(header: NESHeader, data: &[u8]) -> Self {
        Self {
            header,
            data: data.to_vec(),
        }
    }
    /// Read byte from given (mapped) address.
    pub fn read(&self, address: u16) -> u8 {
        match self.data.get(address as usize - 0x8000) {
            Some(byte) => {*byte},
            None => {
                eprintln!("WARNING: attempted to read unmapped address: {}", address);
                0
            }
        }
    }
    /// Write byte from given (mapped) address.
    pub fn write(&self, _address: u16, _value: u8) {
        eprintln!("Cartridge writing not implemented");
        todo!()
    }
}

