use super::MOS6502;

pub(crate) const MAX_INSTR_CYCLES: usize = 6;

/// Const-sized struct for storing an instruction definition.
#[derive(Clone, Copy)]
pub struct InstrDef {
    pub cycles: usize,
    pub u_ops: [Option<fn(&mut MOS6502) -> ()>; MAX_INSTR_CYCLES]
}

impl InstrDef {
    /// Helper function for generating definitions easily.
    ///
    /// NOTE that the actual processing of an instruction is 1 less cycle than how long it takes on
    /// paper; the first cycle is actually fetching the instruction.
    pub(crate) fn from(ops: &[fn(&mut MOS6502)]) -> Self {
        debug_assert!(ops.len() <= MAX_INSTR_CYCLES, "The amount of operations must be less than or equal to {}\nEither condense the instruction or modify MAX_INSTR_CYCLES", MAX_INSTR_CYCLES);
        let mut u_ops = [None; MAX_INSTR_CYCLES];
        for (i, &op) in ops.iter().enumerate() {
            u_ops[i] = Some(op);
        }
        Self {
            cycles: ops.len(),
            u_ops,
        }
    }

    /// Returns the InstrDef's micro-operations as a vector
    /// (Remember that `InstrDef` is const sized)
    pub(crate) fn as_vec(&self) -> Vec<fn(&mut MOS6502)> {
        self.u_ops[0..self.cycles].iter().map(|&it| it.unwrap()).collect()
    }
}
