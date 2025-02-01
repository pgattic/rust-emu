use std::rc::Rc;
use std::collections::VecDeque;
use std::cell::RefCell;
use crate::RustNesError;
use crate::hardware::Bus;

const MAX_INSTR_CYCLES: usize = 8;

#[derive(Clone, Copy)]
struct InstrDef {
    pub cycles: usize,
    pub u_ops: [Option<fn(&mut MOS6502)>; MAX_INSTR_CYCLES]
}

impl InstrDef {
    /// Helper function for generating definitions easily.
    ///
    /// NOTE that the actual processing of an instruction is 1 less cycle than how long it takes on
    /// paper; the first cycle is actually fetching the instruction.
    pub fn from(ops: &[fn(&mut MOS6502)]) -> Self {
        let cycles = ops.len();
        let mut u_ops = [None; MAX_INSTR_CYCLES];
        for (i, op) in ops[..cycles].iter().enumerate() {
            u_ops[i] = Some(*op);
        }
        Self {
            cycles,
            u_ops,
        }
    }
}

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
    //status: u8,
    //stack_ptr: u8,
    state: MOSState,
    instructions: [InstrDef; 256],
}

impl MOS6502 {
    /// Creates a new 6502 CPU. Requires access to a memory bus
    pub fn new(bus: Rc<RefCell<Bus>>) -> Self {

        let mut instrs: [InstrDef; 256] = [InstrDef{cycles: 0, u_ops: [None; MAX_INSTR_CYCLES]}; 256];

        instrs[0x84] = // STY zpg
            InstrDef::from(&[Self::imm_zpg, Self::abs_sty]);
        instrs[0x85] = // STA zpg
            InstrDef::from(&[Self::imm_zpg, Self::abs_sta]);
        instrs[0x86] = // STX zpg
            InstrDef::from(&[Self::imm_zpg, Self::abs_stx]);
        instrs[0xA0] = // LDY #
            InstrDef::from(&[Self::imm_ldy]);
        instrs[0xA2] = // LDX #
            InstrDef::from(&[Self::imm_ldx]);
        instrs[0xA9] = // LDA #
            InstrDef::from(&[Self::imm_lda]);

        Self {
            bus,
            program_counter: 0,
            a: 0, // Accumulator
            x: 0,
            y: 0,
            //status: 0,
            //stack_ptr: 0,
            state: MOSState::new(),
            instructions: instrs
        }
    }

    /// Initializes the CPU to its proper state.
    ///
    /// The ROM must supply the Reset vector, a 16-bit number mapped to address $FFFC-$FFFD that
    /// tells the 6502 what address to initialize its program counter with.
    ///
    /// In addition, the address space from $8000-$FFFF must be mapped to PRG ROM.
    ///
    /// TODO: Rewrite this to actually set the State machine to the correct micro-operations that
    /// perform this, instead of just doing it here. It's supposed to take like 8 cycles I think?
    pub fn init(&mut self) -> Result<(), RustNesError> {
        let bus = self.bus.borrow();
        // Get reset vector
        self.program_counter =
            (bus.read(0xFFFD) as u16) << 8 |
            (bus.read(0xFFFC) as u16);
        Ok(())
    }

    /// Steps the CPU by one clock cycle.
    ///
    /// Famously short function. I know, I know, I'm really cool.
    ///
    /// NOTE that the "fetch" stage always accounts for the first cycle of any instruction. This is
    /// why the branching is bi-conditional; for any instruction, this first cycle is implied.
    pub fn step(&mut self) -> Result<(), RustNesError> {
        match self.state.u_op_queue.pop_front() {
            None => {
                self.imm_f(); // Fetch
                if self.state.fetch == 0 { // BRK (the exception to the rule)
                    return Err(RustNesError::Break)
                }
                let next_instr = self.instructions[self.state.fetch as usize];
                if next_instr.cycles == 0 {
                    return Err(RustNesError::InvalidOpcode(self.state.fetch))
                }
                for i in 0..next_instr.cycles { // Decode
                    self.state.u_op_queue.push_back(next_instr.u_ops[i].unwrap());
                }
            },
            Some(next) => { next(self) }, // Execute
        }
        Ok(())
    }

    // CPU SUB-INSTRUCTIONS //
    // I Have no idea if this strat will work long-term. But the model works in my mind.
    // In short, I want to create a function to represent each possible cycle that happens in the
    // CPU, so the opcodes can simply reference a list of these as their spec. (See MOS6502::new)

    // FETCHERS //

    /// Fetch immediate value from memory, increment PC
    fn imm_f(&mut self) {
        self.state.fetch = self.bus.borrow_mut().read(self.program_counter);
        self.program_counter += 1;
    }
    /// Immediate fetch into accumulator
    fn imm_lda(&mut self) {
        self.imm_f();
        self.a = self.state.fetch;
    }
    /// Immediate fetch into Y reg
    fn imm_ldy(&mut self) {
        self.imm_f();
        self.y = self.state.fetch;
    }
    /// Immediate fetch into X reg
    fn imm_ldx(&mut self) {
        self.imm_f();
        self.x = self.state.fetch;
    }
    /// Immediate zero-page fetch into address latch
    fn imm_zpg(&mut self) {
        self.imm_f();
        self.state.addr_latch = self.state.fetch as u16;
    }

    // STORERS //
    /// Absolute store from fetch
    fn abs_stf(&mut self) {
        self.bus.borrow_mut().write(self.state.addr_latch, self.state.fetch);
    }
    /// Absolute store from Y reg
    fn abs_sty(&mut self) {
        self.state.fetch = self.y;
        self.abs_stf();
    }
    /// Absolute store from accumulator
    fn abs_sta(&mut self) {
        self.state.fetch = self.a;
        self.abs_stf();
    }
    /// Absolute store from X reg
    fn abs_stx(&mut self) {
        self.state.fetch = self.x;
        self.abs_stf();
    }
}

// Thought: I should implement an instruction!() macro that lets me specify the following:
// - Required clock cycles of an instruction
// - What operation(s) to perform (potentially for each clock cycle)
// - Number of bytes the instruction needs (therefore how much to move the PC)

/// Internal state machine responsible for tracking mid-execution information.
///
/// Contains hidden registers:
/// - Instruction register: current instruction being operated on
/// - Address latch: accumulates (16-bit) address to be sent to memory bus
/// - Micro-op queue: representation of the NES's state machine for its current and future jobs
struct MOSState {
    current_instr: u8,
    fetch: u8,
    addr_latch: u16,
    u_op_queue: VecDeque<fn(&mut MOS6502)>
}

impl MOSState {
    pub fn new() -> Self {
        Self {
            current_instr: 0,
            fetch: 0,
            addr_latch: 0,
            u_op_queue: VecDeque::new(),
        }
    }
}

