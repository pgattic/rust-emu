use std::sync::{Arc, Mutex};
use crate::MOS6502;

pub struct Ricoh2A03 {
    core: MOS6502,
    //apu:
    clock_speed: f64,
    memory: Arc<Mutex<Vec<u8>>>,
}

impl Ricoh2A03 {
    pub fn new(clock_speed: f64) -> {
    }
}

