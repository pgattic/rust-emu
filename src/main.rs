use std::sync::{Arc, Mutex};

pub mod mos;
pub mod memory;
use crate::mos::MOS;

fn main() {
    let initial_mem = Arc::new(Mutex::new(vec![0; 65535]));

    {
        let mut mem_access = initial_mem.lock().unwrap();

        mem_access[0] = 0xA9; // LDA #
        mem_access[1] = 69;
        mem_access[2] = 0x85; // STA zpg
        mem_access[3] = 0x68; // zero-page address
        mem_access[4] = 0x00; // BRK
    }

    let mut my_cpu = MOS::new(initial_mem.clone());

    my_cpu.step().unwrap();
    my_cpu.step().unwrap();

    {
        let mem_access = initial_mem.lock().unwrap();
        println!("The value at the address 0x68 is: {}", mem_access[0x68]);
    }
}


