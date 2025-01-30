pub mod mos;
pub mod error;
pub mod memory;
use crate::mos::MOS6502;
use crate::error::MOSError;

use std::sync::{Arc, Mutex};

fn main() -> Result<(), MOSError> {
    let initial_mem = Arc::new(Mutex::new(vec![0; 0xFFFF]));

    {
        let mut mem_access = initial_mem.lock().unwrap();

        // Reset vector
        mem_access[0xFFFC] = 0x00;
        mem_access[0xFFFD] = 0x80;

        // Program code
        mem_access[0x8000] = 0xA9; // LDA #
        mem_access[0x8001] = 69;
        mem_access[0x8002] = 0x85; // STA zpg
        mem_access[0x8003] = 0x00; // zero-page address
        mem_access[0x8004] = 0x00; // BRK
    }

    let mut my_cpu = MOS6502::new(initial_mem.clone());

    my_cpu.init()?;
    println!("Program counter is now 0x{:x}", my_cpu.program_counter);
    my_cpu.step()?;
    my_cpu.step()?;
    my_cpu.step()?;
    my_cpu.step()?;
    my_cpu.step()?;

    {
        let mem_access = initial_mem.lock().unwrap();
        println!("The value at the address 0x00 is: {}", mem_access[0x00]);
    }

    assert_eq!(my_cpu.step(), Err(MOSError::Break));

    Ok(())
}


