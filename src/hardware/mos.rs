use bitflags::bitflags;

use std::rc::Rc;
use std::collections::VecDeque;
use std::cell::RefCell;
use crate::RustNesError;
use crate::hardware::Bus;

const MAX_INSTR_CYCLES: usize = 8;

/// Const-sized struct for storing an instruction definition.
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

    /// Returns the InstrDef's micro-operations as a vector
    /// (Remember that `InstrDef` is const sized)
    pub fn as_vec(&self) -> Vec<fn(&mut MOS6502)> {
        self.u_ops[0..self.cycles].iter().map(|&it| it.unwrap()).collect()
    }
}

bitflags! {
    struct Status: u8 {
        const CARRY     = 0b0000_0001;
        const ZERO      = 0b0000_0010;
        const INTERRUPT = 0b0000_0100;
        const DECIMAL   = 0b0000_1000;
        const BREAK     = 0b0001_0000;
        const UNUSED    = 0b0010_0000;
        const OVERFLOW  = 0b0100_0000;
        const NEGATIVE  = 0b1000_0000;
    }
}

/// Virtual MOS 6502 processor. The roles of `MOS6502` are as follows:
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
    status: Status,
    stack_ptr: u8,
    state: MOSState,
    instructions: [InstrDef; 256],
}

impl MOS6502 {
    /// Constructs a new 6502 CPU (`MOS6502`). Requires access to a memory bus.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::cell::RefCell;
    /// use std::rc::Rc;
    /// use crate::hardware::*;
    ///
    /// let my_ppu = RefCell::new(PPU::new());
    /// let my_apu = RefCell::new(APU::new());
    /// let my_bus = Rc::new(RefCell::new(Bus::new(my_ppu, my_apu)));
    ///
    /// let my_cpu = MOS6502::new(my_bus.clone());
    /// ```
    pub fn new(bus: Rc<RefCell<Bus>>) -> Self {

        let mut instrs: [InstrDef; 256] = [InstrDef{cycles: 0, u_ops: [None; MAX_INSTR_CYCLES]}; 256];

        instrs[0x84] = // STY zpg
            InstrDef::from(&[Self::imm_zal, Self::wr_y_zpg]);
        instrs[0x85] = // STA zpg
            InstrDef::from(&[Self::imm_zal, Self::wr_a_zpg]);
        instrs[0x86] = // STX zpg
            InstrDef::from(&[Self::imm_zal, Self::wr_x_zpg]);

        instrs[0xA0] = // LDY #
            InstrDef::from(&[Self::imm_y]);
        instrs[0xA1] = // LDA x,ind
            InstrDef::from(&[Self::imm_zal, Self::add_x_zal, Self::ind_lo_aal, Self::ind_hi_aal, Self::aal_lda]);
        instrs[0xA2] = // LDX #
            InstrDef::from(&[Self::imm_x]);
        instrs[0xA4] = // LDY zpg
            InstrDef::from(&[Self::imm_zal, Self::zal_ldy]);
        instrs[0xA5] = // LDA zpg
            InstrDef::from(&[Self::imm_zal, Self::zal_lda]);
        instrs[0xA6] = // LDX zpg
            InstrDef::from(&[Self::imm_zal, Self::zal_ldx]);
        instrs[0xA9] = // LDA #
            InstrDef::from(&[Self::imm_a]);
        instrs[0xAC] = // LDY abs
            InstrDef::from(&[Self::imm_lo_aal, Self::imm_hi_aal, Self::aal_ldy]);
        instrs[0xAD] = // LDA abs
            InstrDef::from(&[Self::imm_lo_aal, Self::imm_hi_aal, Self::aal_lda]);
        instrs[0xAE] = // LDX abs
            InstrDef::from(&[Self::imm_lo_aal, Self::imm_hi_aal, Self::aal_ldx]);

        instrs[0xB1] = // LDA ind, y
            InstrDef::from(&[Self::imm_zal, Self::ind_lo_aal, Self::ind_hi_aal, Self::y_aal_lda]);
        // NOTE: The x register is actually added to the zeropage latch, while when similar
        // operations are performed on the Absolute latch, the absolute latch is not modified.
        instrs[0xB4] = // LDY zpg, X
            InstrDef::from(&[Self::imm_zal, Self::add_x_zal /* <- NOTE here */, Self::zal_ldy]);
        instrs[0xB5] = // LDA zpg, X
            InstrDef::from(&[Self::imm_zal, Self::add_x_zal /* <- NOTE here */, Self::zal_lda]);
        instrs[0xB6] = // LDX zpg, Y
            InstrDef::from(&[Self::imm_zal, Self::add_y_zal /* <- NOTE here */, Self::zal_ldx]);
        instrs[0xB9] = // LDA abs, Y
            InstrDef::from(&[Self::imm_lo_aal, Self::imm_hi_aal, Self::y_aal_lda]);
        instrs[0xBC] = // LDY abs, X
            InstrDef::from(&[Self::imm_lo_aal, Self::imm_hi_aal, Self::x_aal_ldy]);
        instrs[0xBD] = // LDA abs, X
            InstrDef::from(&[Self::imm_lo_aal, Self::imm_hi_aal, Self::x_aal_lda]);
        instrs[0xBE] = // LDX abs, Y
            InstrDef::from(&[Self::imm_lo_aal, Self::imm_hi_aal, Self::y_aal_ldx]);

        instrs[0xEA] = // NOP
            InstrDef::from(&[Self::nop]);

        Self {
            bus,
            program_counter: 0,
            a: 0, // Accumulator
            x: 0,
            y: 0,
            status: Status::empty(),
            stack_ptr: 0,
            state: MOSState::new(),
            instructions: instrs
        }
    }

    /// Initializes the CPU to its powered-on state.
    ///
    /// The ROM must supply the Reset vector, a 16-bit number mapped to address $FFFC-$FFFD that
    /// tells the 6502 what address to initialize its program counter with.
    ///
    /// In addition, the address space from $8000-$FFFF must be mapped to PRG ROM.
    ///
    /// The stack pointer is initialized with a default of 0xFD, and the unused flag is always set.
    ///
    /// TODO: Rewrite this to actually set the State machine to the correct micro-operations that
    /// perform this, instead of just doing it here. It's supposed to take like 8 cycles I think?
    pub fn reset(&mut self) -> Result<(), RustNesError> {
        let bus = self.bus.borrow();
        // Get reset vector
        self.program_counter =
            (bus.read(0xFFFD) as u16) << 8 |
            (bus.read(0xFFFC) as u16);

        self.status = Status::empty();
        self.status.insert(Status::UNUSED); // This bit is always 1
        self.stack_ptr = 0xFD;
        Ok(())
    }

    /// Steps the CPU by one clock cycle.
    ///
    /// Famously short function. I know, I know, I'm really cool.
    ///
    /// NOTE that the "fetch" stage always accounts for the first cycle of any instruction.
    /// For any instruction, this first cycle is implied.
    pub fn step(&mut self) -> Result<(), RustNesError> {
        match self.state.u_op_queue.pop_front() {
            None => {
                let next_byte = self.bus.borrow()
                    .read(self.program_counter); // Fetch
                let next_instr = self.instructions[next_byte as usize];
                self.program_counter += 1;
                if next_instr.cycles == 0 { return Err(RustNesError::InvalidOpcode(next_byte)) }
                self.state.u_op_queue = next_instr.as_vec().into(); // Decode
            },
            Some(next) => { next(self) }, // Execute
        }
        Ok(())
    }

    // CPU SUB-INSTRUCTIONS //
    // I Have no idea if this strat will work long-term. But the model works in my mind.
    // In short, I want to create a function to represent each possible cycle that happens in the
    // CPU, so the opcodes can simply reference a list of these as their spec. (See MOS6502::new)

    /// Update N and Z flags (NOT A SUB-INSTRUCTION)
    fn upd_nz(&mut self, number: u8) {
        self.status.set(Status::NEGATIVE, number & 0x80 == 0x80);
        self.status.set(Status::ZERO, number == 0);
    }

    // -------- //
    // FETCHERS //
    // -------- //

    // IMMEDIATE //

    /// Immediate load into data latch, increment PC
    fn imm_dl(&mut self) {
        self.state.data_latch = self.bus.borrow_mut().read(self.program_counter);
        self.program_counter += 1;
    }
    /// Immediate fetch into accumulator
    fn imm_a(&mut self) {
        self.imm_dl();
        self.a = self.state.data_latch;
        self.upd_nz(self.a);
    }
    /// Immediate fetch into Y reg
    fn imm_y(&mut self) {
        self.imm_dl();
        self.y = self.state.data_latch;
        self.upd_nz(self.y);
    }
    /// Immediate fetch into X reg
    fn imm_x(&mut self) {
        self.imm_dl();
        self.x = self.state.data_latch;
        self.upd_nz(self.x);
    }

    /// Immediate fetch into zero-page address latch
    fn imm_zal(&mut self) {
        self.imm_dl();
        self.state.zpg_addr_latch = self.state.data_latch;
    }

    /// Immediate fetch into low-byte of absolute address latch
    fn imm_lo_aal(&mut self) {
        self.imm_dl();
        self.state.abs_addr_latch = self.state.data_latch as u16;
    }
    /// Immediate fetch into high-byte of absolute address latch
    fn imm_hi_aal(&mut self) {
        self.imm_dl();
        self.state.abs_addr_latch &= 0xFF; // Make sure the high byte is cleared
        self.state.abs_addr_latch |= (self.state.data_latch as u16) << 8;
    }
    /// Absolute fetch into accumulator
    fn zal_lda(&mut self) {
        self.a = self.bus.borrow_mut().read(self.state.zpg_addr_latch as u16);
        self.upd_nz(self.a);
    }
    /// Absolute fetch into X register
    fn zal_ldx(&mut self) {
        self.x = self.bus.borrow_mut().read(self.state.zpg_addr_latch as u16);
        self.upd_nz(self.x);
    }
    /// Absolute fetch into Y register
    fn zal_ldy(&mut self) {
        self.y = self.bus.borrow_mut().read(self.state.zpg_addr_latch as u16);
        self.upd_nz(self.y);
    }

    /// Indirect (pointer found in zero-page latch) fetch into low_byte of absolute address latch
    fn ind_lo_aal(&mut self) {
        self.state.abs_addr_latch = self.bus.borrow_mut().read(self.state.zpg_addr_latch as u16) as u16;
    }
    /// Indirect (pointer found in zero-page latch) fetch into high_byte of absolute address latch
    fn ind_hi_aal(&mut self) {
        self.state.abs_addr_latch &= 0xFF; // Make sure the high byte is cleared
        self.state.abs_addr_latch |= (self.bus.borrow_mut().read((self.state.zpg_addr_latch + 1) as u16) as u16) << 8;
    }

    // ABSOLUTE //

    /// Absolute fetch into accumulator
    fn aal_lda(&mut self) {
        self.a = self.bus.borrow_mut().read(self.state.abs_addr_latch);
        self.upd_nz(self.a);
    }
    /// Absolute fetch into X reg
    fn aal_ldx(&mut self) {
        self.x = self.bus.borrow_mut().read(self.state.abs_addr_latch);
        self.upd_nz(self.x);
    }
    /// Absolute fetch into Y reg
    fn aal_ldy(&mut self) {
        self.y = self.bus.borrow_mut().read(self.state.abs_addr_latch);
        self.upd_nz(self.y);
    }

    /// Absolute fetch (plus index stored in x) into accumulator.
    /// Page crossings incur additional cycle
    fn x_aal_lda(&mut self) {
        if self.state.abs_addr_latch & 0xFF + self.x as u16 > 0xFF {
            // Wait an extra cycle.
            // IRL harware takes an extra cycle to resolve the new page.
            self.state.u_op_queue.push_front(Self::nop);
        }
        self.a = self.bus.borrow_mut().read(self.state.abs_addr_latch + self.x as u16);
        self.upd_nz(self.a);
    }
    /// Absolute fetch (plus index stored in x) into accumulator.
    /// Page crossings incur additional cycle
    fn x_aal_ldy(&mut self) {
        if self.state.abs_addr_latch & 0xFF + self.x as u16 > 0xFF {
            // Wait an extra cycle.
            // IRL harware takes an extra cycle to resolve the new page.
            self.state.u_op_queue.push_front(Self::nop);
        }
        self.y = self.bus.borrow_mut().read(self.state.abs_addr_latch + self.x as u16);
        self.upd_nz(self.y);
    }
    /// Absolute fetch (plus index stored in y) into accumulator.
    /// Page crossings incur additional cycle
    fn y_aal_lda(&mut self) {
        if self.state.abs_addr_latch & 0xFF + self.y as u16 > 0xFF {
            // Wait an extra cycle.
            // IRL harware takes an extra cycle to resolve the new page.
            self.state.u_op_queue.push_front(Self::nop);
        }
        self.a = self.bus.borrow_mut().read(self.state.abs_addr_latch + self.y as u16);
        self.upd_nz(self.a);
    }
    /// Absolute fetch (plus index stored in y) into accumulator.
    /// Page crossings incur additional cycle
    fn y_aal_ldx(&mut self) {
        if self.state.abs_addr_latch & 0xFF + self.y as u16 > 0xFF {
            // Wait an extra cycle.
            // IRL harware takes an extra cycle to resolve the new page.
            self.state.u_op_queue.push_front(Self::nop);
        }
        self.x = self.bus.borrow_mut().read(self.state.abs_addr_latch + self.y as u16);
        self.upd_nz(self.x);
    }

    // ------- //
    // STORERS //
    // ------- //

    /// Absolute write from data latch
    fn wr_abs(&mut self) {
        self.bus.borrow_mut().write(self.state.abs_addr_latch, self.state.data_latch);
    }
    /// Absolute write from Y reg
    fn wr_y_abs(&mut self) {
        self.state.data_latch = self.y;
        self.wr_abs();
    }
    /// Absolute write from accumulator
    fn wr_a_abs(&mut self) {
        self.state.data_latch = self.a;
        self.wr_abs();
    }
    /// Absolute write from X reg
    fn wr_x_abs(&mut self) {
        self.state.data_latch = self.x;
        self.wr_abs();
    }

    /// Zero-page store from data latch
    fn wr_zpg(&mut self) {
        self.bus.borrow_mut().write(self.state.zpg_addr_latch as u16, self.state.data_latch);
    }
    /// Zero-page write from Y reg
    fn wr_y_zpg(&mut self) {
        self.state.data_latch = self.y;
        self.wr_zpg();
    }
    /// Zero-page write from accumulator
    fn wr_a_zpg(&mut self) {
        self.state.data_latch = self.a;
        self.wr_zpg();
    }
    /// Zero-page write from X reg
    fn wr_x_zpg(&mut self) {
        self.state.data_latch = self.x;
        self.wr_zpg();
    }

    /// Add value stored in reg. X to Zero-page Address Latch
    fn add_x_zal(&mut self) {
        self.state.zpg_addr_latch += self.x;
    }
    /// Add value stored in reg. Y to Zero-page Address Latch
    fn add_y_zal(&mut self) {
        self.state.zpg_addr_latch += self.y;
    }
    /// No-op.
    fn nop(&mut self) {}
}

/// Internal state machine responsible for tracking mid-execution information.
///
/// Contains hidden registers:
/// - Instruction register: current instruction being operated on
/// - Address latch: accumulates (16-bit) address to be sent to memory bus
/// - Micro-op queue: representation of the NES's state machine for its current and future jobs
struct MOSState {
    data_latch: u8,
    abs_addr_latch: u16,
    zpg_addr_latch: u8,
    u_op_queue: VecDeque<fn(&mut MOS6502)>
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

