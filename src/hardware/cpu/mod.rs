pub(crate) mod instr_def;
pub(crate) mod state;
pub(crate) mod micro_ops;

use std::rc::Rc;
use std::cell::RefCell;
use crate::RustNesError;
use crate::hardware::Bus;
use crate::hardware::cpu::instr_def::*;
use crate::hardware::cpu::state::MOSState;


bitflags::bitflags! {
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
/// The 6502 has an internal state machine responsible for tracking information such as which cycle
/// within a given instruction it is on.
///
/// Notably, the 6502 is unaware of any memory mapping, as the 2A03 handles that.
/// As opposed to the CPU itself, the roles of the 2A03 chip include:
///
/// - PPU OAM
/// - I/O registers
/// - Frame counter control
/// - Clock speed
pub struct MOS6502 {
    pub(crate) bus: Rc<RefCell<Bus>>,
    pub(crate) program_counter: u16,
    pub(crate) a: u8,
    pub(crate) x: u8,
    pub(crate) y: u8,
    status: Status,
    stack_ptr: u8,
    pub(crate) state: MOSState,
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
        Self {
            bus,
            program_counter: 0,
            a: 0, // Accumulator
            x: 0,
            y: 0,
            status: Status::empty(),
            stack_ptr: 0,
            state: MOSState::new(),
            instructions: Self::instruction_table()
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
    /// NOTE that the "fetch" stage always accounts for the first cycle of any instruction.
    /// For any instruction, this first cycle is implied.
    pub fn step(&mut self) -> Result<(), RustNesError> {
        match self.state.u_op_queue.pop_front() {
            None => {
                let next_byte = self.get_prg(); // Fetch
                let next_instr = self.instructions[next_byte as usize];
                if next_instr.cycles == 0 { return Err(RustNesError::InvalidOpcode(next_byte)) }
                self.state.u_op_queue = next_instr.as_vec().into(); // Decode
            },
            Some(next) => { next(self) }, // Execute
        }
        Ok(())
    }

    /// Retrieves the next byte in the program, and increments the program counter.
    fn get_prg(&mut self) -> u8 {
        let result = self.bus.borrow_mut().read(self.program_counter);
        self.program_counter += 1;
        result
    }

    // CPU Common functions //
    // Not actually used as sub-instructions, although their function signatures might make them
    // seem so. They are just commonly referenced by sub-instructions.

    /// Update N and Z flags
    pub(crate) fn upd_nz(&mut self, number: u8) {
        self.status.set(Status::NEGATIVE, number & 0x80 == 0x80);
        self.status.set(Status::ZERO, number == 0);
    }
    /// Immediate load into data latch, increment PC
    pub(crate) fn imm_dl(&mut self) {
        self.state.data_latch = self.bus.borrow_mut().read(self.program_counter);
        self.program_counter += 1;
    }

    /// Here we define each CPU opcode by what it does for each cycle of its execution. Each opcode
    /// is represented simply by a list of function references. As seen in the definition of
    /// `InstrDef`, the function signatures must be `fn(&mut MOS6502) -> ()`.
    ///
    /// I guess in this sense, they're actually procedures since their only purpose is to modify
    /// state.
    pub fn instruction_table() -> [InstrDef; 256] {
        let mut instrs: [InstrDef; 256] = [InstrDef{cycles: 0, u_ops: [None; MAX_INSTR_CYCLES]}; 256];

        instrs[0x81] = // STA X, ind
            InstrDef::from(&[Self::imm_zal, Self::add_x_zal, Self::ind_lo_aal, Self::ind_hi_aal, Self::aal_sta]);
        instrs[0x84] = // STY zpg
            InstrDef::from(&[Self::imm_zal, Self::zal_sty]);
        instrs[0x85] = // STA zpg
            InstrDef::from(&[Self::imm_zal, Self::zal_sta]);
        instrs[0x86] = // STX zpg
            InstrDef::from(&[Self::imm_zal, Self::zal_stx]);
        instrs[0x8A] = // TXA impl
            InstrDef::from(&[Self::txa]);
        instrs[0x8C] = // STY abs
            InstrDef::from(&[Self::imm_lo_aal, Self::imm_hi_aal, Self::aal_sty]);
        instrs[0x8D] = // STA abs
            InstrDef::from(&[Self::imm_lo_aal, Self::imm_hi_aal, Self::aal_sta]);
        instrs[0x8E] = // STX abs
            InstrDef::from(&[Self::imm_lo_aal, Self::imm_hi_aal, Self::aal_stx]);

        instrs[0x91] = // STA ind, Y
            InstrDef::from(&[Self::imm_zal, Self::ind_lo_aal, Self::ind_hi_aal, Self::add_y_aal, Self::aal_sta]);
        instrs[0x94] = // STY zpg, X
            InstrDef::from(&[Self::imm_zal, Self::add_x_zal, Self::zal_sty]);
        instrs[0x95] = // STA zpg, X
            InstrDef::from(&[Self::imm_zal, Self::add_x_zal, Self::zal_sta]);
        instrs[0x96] = // STX zpg, Y
            InstrDef::from(&[Self::imm_zal, Self::add_y_zal, Self::zal_stx]);
        instrs[0x98] = // TYA impl
            InstrDef::from(&[Self::tya]);
        instrs[0x99] = // STA abs, Y
            InstrDef::from(&[Self::imm_lo_aal, Self::imm_hi_aal, Self::add_y_aal, Self::aal_sta]);
        instrs[0x9D] = // STA abs, X
            InstrDef::from(&[Self::imm_lo_aal, Self::imm_hi_aal, Self::add_x_aal, Self::aal_sta]);

        instrs[0xA0] = // LDY #
            InstrDef::from(&[Self::imm_y]);
        instrs[0xA1] = // LDA X,ind
            InstrDef::from(&[Self::imm_zal, Self::add_x_zal, Self::ind_lo_aal, Self::ind_hi_aal, Self::aal_lda]);
        instrs[0xA2] = // LDX #
            InstrDef::from(&[Self::imm_x]);
        instrs[0xA4] = // LDY zpg
            InstrDef::from(&[Self::imm_zal, Self::zal_ldy]);
        instrs[0xA5] = // LDA zpg
            InstrDef::from(&[Self::imm_zal, Self::zal_lda]);
        instrs[0xA6] = // LDX zpg
            InstrDef::from(&[Self::imm_zal, Self::zal_ldx]);
        instrs[0xA8] = // TAY impl
            InstrDef::from(&[Self::tay]);
        instrs[0xA9] = // LDA #
            InstrDef::from(&[Self::imm_a]);
        instrs[0xAA] = // TAX impl
            InstrDef::from(&[Self::tax]);
        instrs[0xAC] = // LDY abs
            InstrDef::from(&[Self::imm_lo_aal, Self::imm_hi_aal, Self::aal_ldy]);
        instrs[0xAD] = // LDA abs
            InstrDef::from(&[Self::imm_lo_aal, Self::imm_hi_aal, Self::aal_lda]);
        instrs[0xAE] = // LDX abs
            InstrDef::from(&[Self::imm_lo_aal, Self::imm_hi_aal, Self::aal_ldx]);

        instrs[0xB1] = // LDA ind, Y
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

        instrs
    }
}

