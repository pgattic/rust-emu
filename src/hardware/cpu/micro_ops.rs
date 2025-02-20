use crate::MOS6502;

impl MOS6502 {
    // CPU SUB-INSTRUCTIONS //
    // I Have no idea if this strat will work long-term. But the model works in my mind.
    // In short, I want to create a function to represent each possible cycle that happens in the
    // CPU, so the opcodes can simply reference a list of these as their spec. (See MOS6502::new)

    // -------- //
    // FETCHERS //
    // -------- //

    // IMMEDIATE //

    /// Immediate fetch into accumulator
    pub fn imm_a(&mut self) {
        self.imm_dl();
        self.a = self.state.data_latch;
        self.upd_nz(self.a);
    }
    /// Immediate fetch into Y reg
    pub fn imm_y(&mut self) {
        self.imm_dl();
        self.y = self.state.data_latch;
        self.upd_nz(self.y);
    }
    /// Immediate fetch into X reg
    pub fn imm_x(&mut self) {
        self.imm_dl();
        self.x = self.state.data_latch;
        self.upd_nz(self.x);
    }
    /// Immediate fetch into zero-page address latch
    pub fn imm_zal(&mut self) {
        self.imm_dl();
        self.state.zpg_addr_latch = self.state.data_latch;
    }
    /// Immediate fetch into low byte of absolute address latch.
    /// Zeroes out the high byte as a side effect.
    pub fn imm_lo_aal(&mut self) {
        self.imm_dl();
        self.state.abs_addr_latch = self.state.data_latch as u16;
    }
    /// Immediate fetch into high byte of absolute address latch.
    /// Preserves the low byte.
    pub fn imm_hi_aal(&mut self) {
        self.imm_dl();
        self.state.abs_addr_latch &= 0xFF; // Make sure the high byte is cleared
        self.state.abs_addr_latch |= (self.state.data_latch as u16) << 8;
    }

    // ABSOLUTE //

    /// Zero-page fetch into accumulator
    pub fn zal_lda(&mut self) {
        self.a = self.bus.borrow_mut().read(self.state.zpg_addr_latch as u16);
        self.upd_nz(self.a);
    }
    /// Zero-page fetch into X register
    pub fn zal_ldx(&mut self) {
        self.x = self.bus.borrow_mut().read(self.state.zpg_addr_latch as u16);
        self.upd_nz(self.x);
    }
    /// Zero-page fetch into Y register
    pub fn zal_ldy(&mut self) {
        self.y = self.bus.borrow_mut().read(self.state.zpg_addr_latch as u16);
        self.upd_nz(self.y);
    }
    /// Absolute fetch into accumulator
    pub fn aal_lda(&mut self) {
        self.a = self.bus.borrow_mut().read(self.state.abs_addr_latch);
        self.upd_nz(self.a);
    }
    /// Absolute fetch into X register
    pub fn aal_ldx(&mut self) {
        self.x = self.bus.borrow_mut().read(self.state.abs_addr_latch);
        self.upd_nz(self.x);
    }
    /// Absolute fetch into Y register
    pub fn aal_ldy(&mut self) {
        self.y = self.bus.borrow_mut().read(self.state.abs_addr_latch);
        self.upd_nz(self.y);
    }
    /// Absolute fetch (plus index stored in X) into accumulator.
    /// Page crossings incur additional cycle.
    pub fn x_aal_lda(&mut self) {
        if self.state.abs_addr_latch & 0xFF + self.x as u16 > 0xFF {
            // Wait an extra cycle.
            // IRL harware takes an extra cycle to resolve the new page.
            self.state.u_op_queue.push_front(Self::nop);
        }
        self.a = self.bus.borrow_mut().read(self.state.abs_addr_latch + self.x as u16);
        self.upd_nz(self.a);
    }
    /// Absolute fetch (plus index stored in Y) into accumulator.
    /// Page crossings incur additional cycle.
    pub fn y_aal_lda(&mut self) {
        if self.state.abs_addr_latch & 0xFF + self.y as u16 > 0xFF {
            // Wait an extra cycle.
            // IRL harware takes an extra cycle to resolve the new page.
            self.state.u_op_queue.push_front(Self::nop);
        }
        self.a = self.bus.borrow_mut().read(self.state.abs_addr_latch + self.y as u16);
        self.upd_nz(self.a);
    }
    /// Absolute fetch (plus index stored in X) into Y register.
    /// Page crossings incur additional cycle.
    pub fn x_aal_ldy(&mut self) {
        if self.state.abs_addr_latch & 0xFF + self.x as u16 > 0xFF {
            // Wait an extra cycle.
            // IRL harware takes an extra cycle to resolve the new page.
            self.state.u_op_queue.push_front(Self::nop);
        }
        self.y = self.bus.borrow_mut().read(self.state.abs_addr_latch + self.x as u16);
        self.upd_nz(self.y);
    }
    /// Absolute fetch (plus index stored in Y) into X register.
    /// Page crossings incur additional cycle.
    pub fn y_aal_ldx(&mut self) {
        if self.state.abs_addr_latch & 0xFF + self.y as u16 > 0xFF {
            // Wait an extra cycle.
            // IRL harware takes an extra cycle to resolve the new page.
            self.state.u_op_queue.push_front(Self::nop);
        }
        self.x = self.bus.borrow_mut().read(self.state.abs_addr_latch + self.y as u16);
        self.upd_nz(self.x);
    }

    // INDIRECT //

    /// Indirect (pointer found with zero-page latch) fetch into low byte of absolute address latch
    /// Zeroes out the high byte as a side effect.
    pub fn ind_lo_aal(&mut self) {
        self.state.abs_addr_latch = self.bus.borrow_mut().read(self.state.zpg_addr_latch as u16) as u16;
    }
    /// Indirect (pointer found with zero-page latch) fetch into high byte of absolute address latch
    /// Preserves the low byte.
    pub fn ind_hi_aal(&mut self) {
        self.state.abs_addr_latch &= 0xFF; // Make sure the high byte is cleared
        self.state.abs_addr_latch |= (self.bus.borrow_mut().read((self.state.zpg_addr_latch + 1) as u16) as u16) << 8;
    }

    // ------- //
    // STORERS //
    // ------- //

    /// Absolute write from Y reg
    pub fn aal_sty(&mut self) {
        self.bus.borrow_mut().write(self.state.abs_addr_latch, self.y);
    }
    /// Absolute write from accumulator
    pub fn aal_sta(&mut self) {
        self.bus.borrow_mut().write(self.state.abs_addr_latch, self.a);
    }
    /// Absolute write from X reg
    pub fn aal_stx(&mut self) {
        self.bus.borrow_mut().write(self.state.abs_addr_latch, self.x);
    }
    /// Zero-page write from Y reg
    pub fn zal_sty(&mut self) {
        self.bus.borrow_mut().write(self.state.zpg_addr_latch as u16, self.y);
    }
    /// Zero-page write from accumulator
    pub fn zal_sta(&mut self) {
        self.bus.borrow_mut().write(self.state.zpg_addr_latch as u16, self.a);
    }
    /// Zero-page write from X reg
    pub fn zal_stx(&mut self) {
        self.bus.borrow_mut().write(self.state.zpg_addr_latch as u16, self.x);
    }

    // Single-operation Instructions //

    /// Transfer Accumulator into X reg
    pub fn tax(&mut self) {
        self.x = self.a;
        self.upd_nz(self.x);
    }
    /// Transfer Accumulator into Y reg
    pub fn tay(&mut self) {
        self.y = self.a;
        self.upd_nz(self.y);
    }
    /// Transfer X reg into Accumulator
    pub fn txa(&mut self) {
        self.a = self.x;
        self.upd_nz(self.a);
    }
    /// Transfer Y reg into Accumulator
    pub fn tya(&mut self) {
        self.a = self.y;
        self.upd_nz(self.a);
    }

    /// Add value stored in reg. X to Zero-page Address Latch
    pub fn add_x_zal(&mut self) {
        self.state.zpg_addr_latch += self.x;
    }
    /// Add value stored in reg. Y to Zero-page Address Latch
    pub fn add_y_zal(&mut self) {
        self.state.zpg_addr_latch += self.y;
    }
    /// Add value stored in reg. X to Absolute Address Latch.
    /// Also perform dummy read from the resultant address
    pub fn add_x_aal(&mut self) {
        self.state.abs_addr_latch += self.x as u16;
        _ = self.bus.borrow_mut().read(self.state.abs_addr_latch);
    }
    /// Add value stored in reg. Y to Absolute Address Latch.
    /// Also perform dummy read from the resultant address
    pub fn add_y_aal(&mut self) {
        self.state.abs_addr_latch += self.y as u16;
        _ = self.bus.borrow_mut().read(self.state.abs_addr_latch);
    }
    /// No-op.
    pub fn nop(&mut self) {}
}

