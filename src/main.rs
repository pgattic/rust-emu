pub mod hardware;
pub mod header;
pub mod error;
use crate::header::NESHeader;
use crate::error::RustNesError;

use std::cell::RefCell;
use std::rc::Rc;
use std::fs;
use clap::Parser;

#[derive(Parser)]
#[command(name = "rust-nes", about = "Simple NES emulator in Rust")]
struct Cli {
    /// Path to search in
    #[arg()]
    file: String,
}

fn main() -> Result<(), RustNesError> {
    let args = Cli::parse();

    // Load Cartridge
    let cart = {
        let rom_file = match fs::read(&args.file) {
            Ok(data) => data,
            Err(err) => {
                eprintln!("Error: can't open file '{}': {}", args.file, err);
                std::process::exit(0x02);
            }
        };
        let header = NESHeader::from_bytes(&rom_file[0..15]).ok_or(RustNesError::InvalidHeader)?;
        RefCell::new(hardware::Cart::new(header, &rom_file[16..]))
    };

    // Initialize Hardware
    let my_ppu = RefCell::new(hardware::PPU::new());
    let my_apu = RefCell::new(hardware::APU::new());
    let my_bus = Rc::new(RefCell::new(hardware::Bus::new(my_ppu, my_apu)));
    let mut my_cpu = hardware::MOS6502::new(my_bus.clone());

    // Input cart
    my_bus.borrow_mut().load_cart(cart);

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

    assert_eq!(my_cpu.step(), Err(RustNesError::Break));

    Ok(())
}

