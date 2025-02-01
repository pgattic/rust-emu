
pub struct APU;

impl APU {
    pub fn new() -> Self {
        Self
    }
    pub fn read(&mut self, address: u16) -> u8 {
        eprintln!("APU address {} not implemented", address);
        todo!()
    }
    pub fn write(&mut self, address: u16, _value: u8) {
        eprintln!("APU address {} not implemented", address);
        todo!()
    }
}

