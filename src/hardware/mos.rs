//use std::sync::{Arc, Mutex};
use std::rc::Rc;
use std::cell::RefCell;
use crate::MOSError;
use crate::hardware::Bus;

/// Virtual MOS 6502 processor. The roles of the 6502 are as follows:
///
/// - Manage program state (program counter, stack)
/// - Fetch, decode and execute instructions
/// - Perform arithmetic operations through integrated ALU
///
/// The 6502, being a CISC architecture, has an internal state machine responsible for tracking
/// information such as which cycle within a given instruction it is on.
///
/// Notably, the 6502 is unaware of any memory mapping, as the 2A03 handles that.
/// As opposed to the CPU itself, the roles of the 2A03 chip include:
///
/// - PPU OAM
/// - I/O registers
/// - Frame counter control
/// - Clock speed
pub struct MOS6502 {
    bus: Rc<RefCell<Bus>>,
    pub program_counter: u16,
    a: u8,
    x: u8,
    y: u8,
    status: u8,
    stack_ptr: u8,
    state: MOSState,
}

impl MOS6502 {
    /// Creates a new 6502 CPU. Requires access to some form of memory
    pub fn new(bus: Rc<RefCell<Bus>>) -> Self {
        Self {
            bus,
            program_counter: 0,
            a: 0, // Accumulator
            x: 0,
            y: 0,
            status: 0,
            stack_ptr: 0,
            state: MOSState::new(),
        }
    }

    /// Initializes the CPU to its proper state.
    ///
    /// The ROM must supply the Reset vector, a 16-bit number mapped to address $FFFC-$FFFD that
    /// tells the 6502 what address to initialize its program counter with.
    ///
    /// In addition, the address space from $8000-$FFFF must be mapped to PRG ROM.
    pub fn init(&mut self) -> Result<(), MOSError> {
        let bus = self.bus.borrow();
        // Get reset vector
        self.program_counter =
            (bus.read(0xFFFD) as u16) << 8 |
            (bus.read(0xFFFC) as u16);
        Ok(())
    }

    /// Steps the CPU by one clock cycle.
    pub fn step(&mut self) -> Result<(), MOSError> {
        if self.state.rem_cycles == 0 {
            // Fetch
            let next_instr = self.read(self.program_counter);
            self.state.current_instr = next_instr;
            // Decode
            match self.state.current_instr {
                // Execute
                0x00 => { // BRK impl
                    self.state.rem_cycles = 7;
                    return Err(MOSError::Break);
                }
                0x84 => { // STY zpg
                    self.state.rem_cycles = 3;
                    let target = self.read(self.program_counter + 1);
                    self.write(target.into(), self.y);
                    self.program_counter += 2;
                }
                0x85 => { // STA zpg
                    self.state.rem_cycles = 3;
                    let target = self.read(self.program_counter + 1);
                    self.write(target.into(), self.a);
                    self.program_counter += 2;
                }
                0x86 => { // STX zpg
                    self.state.rem_cycles = 3;
                    let target = self.read(self.program_counter + 1);
                    self.write(target.into(), self.x);
                    self.program_counter += 2;
                }
                0xA0 => { // LDY #
                    self.state.rem_cycles = 2;
                    self.y = self.read(self.program_counter + 1);
                    self.program_counter += 2;
                }
                0xA2 => { // LDX #
                    self.state.rem_cycles = 2;
                    self.x = self.read(self.program_counter + 1);
                    self.program_counter += 2;
                }
                0xA9 => { // LDA #
                    self.state.rem_cycles = 2;
                    self.a = self.read(self.program_counter + 1);
                    self.program_counter += 2;
                }
                _ => {todo!()}
            }
        }
        self.state.rem_cycles -= 1;
        Ok(())
    }

    fn read(&self, address: u16) -> u8 {
        self.bus.borrow().read(address)
    }

    fn write(&self, address: u16, value: u8) {
        self.bus.borrow_mut().write(address, value)
    }
}

// Thought: I should implement an instruction!() macro that lets me specify the following:
// - Required clock cycles of an instruction
// - What operation(s) to perform (potentially for each clock cycle)
// - Number of bytes the instruction needs (therefore how much to move the PC)
// - Whether or not the instruction needs memory access (to help limit lock usage)

/// Internal state machine responsible for tracking mid-execution information.
struct MOSState {
    current_instr: u8,
    rem_cycles: u8,
}

impl MOSState {
    pub fn new() -> Self {
        Self {
            current_instr: 0,
            rem_cycles: 0,
        }
    }
}

