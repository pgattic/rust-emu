use std::collections::VecDeque;
use super::MOS6502;

/// Internal state machine responsible for tracking mid-execution information.
///
/// Contains hidden registers:
/// - Instruction register: current instruction being operated on
/// - Address latch: accumulates (16-bit) address to be sent to memory bus
/// - Micro-op queue: representation of the NES's state machine for its current and future jobs
pub struct MOSState {
    pub data_latch: u8,
    pub abs_addr_latch: u16,
    pub zpg_addr_latch: u8,
    pub u_op_queue: VecDeque<fn(&mut MOS6502)>
}

impl MOSState {
    pub fn new() -> Self {
        Self {
            data_latch: 0,
            abs_addr_latch: 0,
            zpg_addr_latch: 0,
            u_op_queue: VecDeque::new(),
        }
    }
}


