use crate::opc;
use super::MOS6502;
use super::instr_def::*;

impl MOS6502 {
    /// Here we define each CPU opcode by what it does during each cycle of its execution. Each
    /// opcode is represented simply by a list of function references. As seen in the definition of
    /// `InstrDef`, the function signatures must be `fn(&mut MOS6502) -> ()`.
    ///
    /// I guess in this sense, they're actually procedures since their only purpose is to modify
    /// state.
    ///
    /// See [6502 Instruction Set](https://www.masswerk.at/6502/6502_instruction_set.html) for info.
    pub fn instruction_table() -> [InstrDef; 256] {
        let mut res: [InstrDef; 256] = [InstrDef{cycles: 0, u_ops: [None; MAX_INSTR_CYCLES]}; 256];

        opc!(res, 0x81, [imm_zal, add_x_zal, ind_lo_aal, ind_hi_aal, aal_sta]);     // STA X, ind
        opc!(res, 0x84, [imm_zal, zal_sty]);                                        // STY zpg
        opc!(res, 0x85, [imm_zal, zal_sta]);                                        // STA zpg
        opc!(res, 0x86, [imm_zal, zal_stx]);                                        // STX zpg
        opc!(res, 0x8A, [txa]);                                                     // TXA impl
        opc!(res, 0x8C, [imm_lo_aal, imm_hi_aal, aal_sty]);                         // STY abs
        opc!(res, 0x8D, [imm_lo_aal, imm_hi_aal, aal_sta]);                         // STA abs
        opc!(res, 0x8E, [imm_lo_aal, imm_hi_aal, aal_stx]);                         // STX abs

        opc!(res, 0x91, [imm_zal, ind_lo_aal, ind_hi_aal, add_y_aal, aal_sta]);     // STA ind, Y
        opc!(res, 0x94, [imm_zal, add_x_zal, zal_sty]);                             // STY zpg, X
        opc!(res, 0x95, [imm_zal, add_x_zal, zal_sta]);                             // STA zpg, X
        opc!(res, 0x96, [imm_zal, add_y_zal, zal_stx]);                             // STX zpg, Y
        opc!(res, 0x98, [tya]);                                                     // TYA impl
        opc!(res, 0x99, [imm_lo_aal, imm_hi_aal, add_y_aal, aal_sta]);              // STA abs, Y
        opc!(res, 0x9D, [imm_lo_aal, imm_hi_aal, add_x_aal, aal_sta]);              // STA abs, X

        opc!(res, 0xA0, [imm_y]);                                                   // LDY #
        opc!(res, 0xA1, [imm_zal, add_x_zal, ind_lo_aal, ind_hi_aal, aal_lda]);     // LDA X,ind
        opc!(res, 0xA2, [imm_x]);                                                   // LDX #
        opc!(res, 0xA4, [imm_zal, zal_ldy]);                                        // LDY zpg
        opc!(res, 0xA5, [imm_zal, zal_lda]);                                        // LDA zpg
        opc!(res, 0xA6, [imm_zal, zal_ldx]);                                        // LDX zpg
        opc!(res, 0xA8, [tay]);                                                     // TAY impl
        opc!(res, 0xA9, [imm_a]);                                                   // LDA #
        opc!(res, 0xAA, [tax]);                                                     // TAX impl
        opc!(res, 0xAC, [imm_lo_aal, imm_hi_aal, aal_ldy]);                         // LDY abs
        opc!(res, 0xAD, [imm_lo_aal, imm_hi_aal, aal_lda]);                         // LDA abs
        opc!(res, 0xAE, [imm_lo_aal, imm_hi_aal, aal_ldx]);                         // LDX abs

        opc!(res, 0xB1, [imm_zal, ind_lo_aal, ind_hi_aal, y_aal_lda]);              // LDA ind, Y
        opc!(res, 0xB4, [imm_zal, add_x_zal, zal_ldy]);                             // LDY zpg, X
        opc!(res, 0xB5, [imm_zal, add_x_zal, zal_lda]);                             // LDA zpg, X
        opc!(res, 0xB6, [imm_zal, add_y_zal, zal_ldx]);                             // LDX zpg, Y
        opc!(res, 0xB9, [imm_lo_aal, imm_hi_aal, y_aal_lda]);                       // LDA abs, Y
        opc!(res, 0xBC, [imm_lo_aal, imm_hi_aal, x_aal_ldy]);                       // LDY abs, X
        opc!(res, 0xBD, [imm_lo_aal, imm_hi_aal, x_aal_lda]);                       // LDA abs, X
        opc!(res, 0xBE, [imm_lo_aal, imm_hi_aal, y_aal_ldx]);                       // LDX abs, Y

        opc!(res, 0xEA, [nop]);                                                     // NOP

        res
    }
}

