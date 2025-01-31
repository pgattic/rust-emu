
pub struct ROM {
    data: Vec<u8>
}

impl ROM {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            data
        }
    }
    pub fn read(&self, address: u16) -> u8 {
        self.data[address as usize]
    }
}

