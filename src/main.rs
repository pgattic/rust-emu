pub mod hardware;
pub mod header;
pub mod error;
use crate::error::MOSError;

use std::cell::RefCell;
use std::rc::Rc;

fn main() -> Result<(), MOSError> {
    // Initialize Hardware
    let my_bus = Rc::new(RefCell::new(hardware::Bus::new()));
    let mut my_cpu = hardware::MOS6502::new(Rc::clone(&my_bus));

    // Example "cartridge" (currently not mapped correctly but eh)
    let mut cart = vec![0; 0xFFFF];
    // Reset vector
    cart[0xFFFC] = 0x00;
    cart[0xFFFD] = 0x80;
    // Program code
    cart[0x8000] = 0xA9; // LDA #
    cart[0x8001] = 69;
    cart[0x8002] = 0x85; // STA zpg
    cart[0x8003] = 0x00; // zero-page address
    cart[0x8004] = 0x00; // BRK

    {
        let mut bus_access = my_bus.borrow_mut();
        bus_access.load_rom(hardware::ROM::new(cart));
    }

    my_cpu.init()?;
    println!("Program counter is now 0x{:x}", my_cpu.program_counter);
    my_cpu.step()?;
    my_cpu.step()?;
    my_cpu.step()?;
    my_cpu.step()?;
    my_cpu.step()?;

    {
        let bus_access = my_bus.borrow();
        println!("The value at the address 0x00 is: {}", bus_access.read(0x00));
    }

    assert_eq!(my_cpu.step(), Err(MOSError::Break));

    Ok(())
}

