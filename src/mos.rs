use std::sync::{Arc, Mutex};
use crate::MOSError;

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
    memory_access: Arc<Mutex<Vec<u8>>>,
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
    pub fn new(memory: Arc<Mutex<Vec<u8>>>) -> Self {
        Self {
            memory_access: memory,
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
        let mem = self.memory_access.lock().unwrap();
        // Get reset vector
        self.program_counter =
            (*mem.get(0xFFFD).ok_or(MOSError::OutOfBounds)? as u16) << 8 |
            *mem.get(0xFFFC).ok_or(MOSError::OutOfBounds)? as u16;
        Ok(())
    }

    /// Steps the CPU by one clock cycle.
    pub fn step(&mut self) -> Result<(), MOSError> {
        let mut mem = self.memory_access.lock().unwrap();
        if self.state.rem_cycles == 0 {
            // Fetch
            let next_instr = mem.get(self.program_counter as usize).ok_or(MOSError::OutOfBounds)?.clone();
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
                    let target = mem[self.program_counter as usize + 1];
                    mem[target as usize] = self.y;
                    self.program_counter += 2;
                }
                0x85 => { // STA zpg
                    self.state.rem_cycles = 3;
                    let target = mem[self.program_counter as usize + 1];
                    mem[target as usize] = self.a;
                    self.program_counter += 2;
                }
                0x86 => { // STX zpg
                    self.state.rem_cycles = 3;
                    let target = mem[self.program_counter as usize + 1];
                    mem[target as usize] = self.x;
                    self.program_counter += 2;
                }
                0xA0 => { // LDY #
                    self.state.rem_cycles = 2;
                    self.y = mem[self.program_counter as usize + 1];
                    self.program_counter += 2;
                }
                0xA2 => { // LDX #
                    self.state.rem_cycles = 2;
                    self.x = mem[self.program_counter as usize + 1];
                    self.program_counter += 2;
                }
                0xA9 => { // LDA #
                    self.state.rem_cycles = 2;
                    self.a = mem[self.program_counter as usize + 1];
                    self.program_counter += 2;
                }
                _ => {todo!()}
            }
        }
        self.state.rem_cycles -= 1;
        Ok(())
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

