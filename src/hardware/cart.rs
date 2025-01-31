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
    pub fn read(&self, address: u16) -> u8 {
        self.data[address as usize - 0x8000]
    }
}

