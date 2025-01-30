use std::sync::{Arc, Mutex};

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
pub struct MOS {
    memory_access: Arc<Mutex<Vec<u8>>>,
    program_counter: u16,
    accumulator: u8,
    x: u8,
    y: u8,
    status: u8,
    stack_ptr: u8,
    state: MOSState,
}

impl MOS {
    /// Creates a new 6502 CPU. Requires access to some form of memory
    pub fn new(memory: Arc<Mutex<Vec<u8>>>) -> Self {
        Self {
            memory_access: memory,
            program_counter: 0,
            accumulator: 0,
            x: 0,
            y: 0,
            status: 0,
            stack_ptr: 0,
            state: MOSState::new(),
        }
    }
    /// Steps the CPU.
    pub fn step(&mut self) -> Result<(), ()> {
        let mut mem = self.memory_access.lock().unwrap();
        let next_instr = mem[self.program_counter as usize];
        match next_instr {
            0x00 => { // BRK impl
                return Err(());
            }
            0x85 => { // STA zpg
                let target = mem[self.program_counter as usize + 1];
                mem[target as usize] = self.accumulator;
                self.program_counter += 2;
            }
            0xA9 => { // LDA #
                self.accumulator = mem[self.program_counter as usize + 1];
                self.program_counter += 2;
            }
            _ => {todo!()}
        }
        Ok(())
    }
}

/// Internal state machine responsible for tracking mid-execution information.
struct MOSState {
    current_instr: u8,
    step: u8,
}

impl MOSState {
    pub fn new() -> Self {
        Self {
            current_instr: 0,
            step: 0,
        }
    }
}

