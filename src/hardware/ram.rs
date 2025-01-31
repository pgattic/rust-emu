
pub struct WorkMemory {
    memory: [u8; 0x2000],
}

impl WorkMemory {
    pub fn new() -> Self {
        Self {
            memory: [0; 0x2000],
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        self.memory[addr as usize] = value;
    }
}

