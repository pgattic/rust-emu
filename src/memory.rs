
pub type Memory = [u8; 65536];

pub fn get(memory: &Memory, address: u16) -> u8 {
    memory[address as usize]
}

