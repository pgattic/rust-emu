use crate::opcodes;
use super::MOS6502;
use super::instr_def::*;

impl MOS6502 {
    /// Here we define each CPU opcode by what it does during each cycle of its execution. Each
    /// opcode is represented simply by a list of function references. As seen in the definition of
    /// `InstrDef`, the function signatures must be `fn(&mut MOS6502) -> ()`.
    ///
    /// See [6502 Instruction Set](https://www.masswerk.at/6502/6502_instruction_set.html) for info.
    pub fn instruction_table() -> [InstrDef; 256] {
        let mut instrs: [InstrDef; 256] = [InstrDef{cycles: 0, u_ops: [None; MAX_INSTR_CYCLES]}; 256];

        opcodes!(instrs, {
            0x81 => [imm_zal, add_x_zal, ind_lo_aal, ind_hi_aal, aal_sta],      // STA X, ind
            0x84 => [imm_zal, zal_sty],                                         // STY zpg
            0x85 => [imm_zal, zal_sta],                                         // STA zpg
            0x86 => [imm_zal, zal_stx],                                         // STX zpg
            0x8A => [txa],                                                      // TXA impl
            0x8C => [imm_lo_aal, imm_hi_aal, aal_sty],                          // STY abs
            0x8D => [imm_lo_aal, imm_hi_aal, aal_sta],                          // STA abs
            0x8E => [imm_lo_aal, imm_hi_aal, aal_stx],                          // STX abs

            0x91 => [imm_zal, ind_lo_aal, ind_hi_aal, add_y_aal, aal_sta],      // STA ind, Y
            0x94 => [imm_zal, add_x_zal, zal_sty],                              // STY zpg, X
            0x95 => [imm_zal, add_x_zal, zal_sta],                              // STA zpg, X
            0x96 => [imm_zal, add_y_zal, zal_stx],                              // STX zpg, Y
            0x98 => [tya],                                                      // TYA impl
            0x99 => [imm_lo_aal, imm_hi_aal, add_y_aal, aal_sta],               // STA abs, Y
            0x9D => [imm_lo_aal, imm_hi_aal, add_x_aal, aal_sta],               // STA abs, X

            0xA0 => [imm_y],                                                    // LDY #
            0xA1 => [imm_zal, add_x_zal, ind_lo_aal, ind_hi_aal, aal_lda],      // LDA X,ind
            0xA2 => [imm_x],                                                    // LDX #
            0xA4 => [imm_zal, zal_ldy],                                         // LDY zpg
            0xA5 => [imm_zal, zal_lda],                                         // LDA zpg
            0xA6 => [imm_zal, zal_ldx],                                         // LDX zpg
            0xA8 => [tay],                                                      // TAY impl
            0xA9 => [imm_a],                                                    // LDA #
            0xAA => [tax],                                                      // TAX impl
            0xAC => [imm_lo_aal, imm_hi_aal, aal_ldy],                          // LDY abs
            0xAD => [imm_lo_aal, imm_hi_aal, aal_lda],                          // LDA abs
            0xAE => [imm_lo_aal, imm_hi_aal, aal_ldx],                          // LDX abs

            0xB1 => [imm_zal, ind_lo_aal, ind_hi_aal, y_aal_lda],               // LDA ind, Y
            0xB4 => [imm_zal, add_x_zal, zal_ldy],                              // LDY zpg, X
            0xB5 => [imm_zal, add_x_zal, zal_lda],                              // LDA zpg, X
            0xB6 => [imm_zal, add_y_zal, zal_ldx],                              // LDX zpg, Y
            0xB9 => [imm_lo_aal, imm_hi_aal, y_aal_lda],                        // LDA abs, Y
            0xBC => [imm_lo_aal, imm_hi_aal, x_aal_ldy],                        // LDY abs, X
            0xBD => [imm_lo_aal, imm_hi_aal, x_aal_lda],                        // LDA abs, X
            0xBE => [imm_lo_aal, imm_hi_aal, y_aal_ldx],                        // LDX abs, Y

            0xEA => [nop],                                                      // NOP
        });

        instrs
    }
}

