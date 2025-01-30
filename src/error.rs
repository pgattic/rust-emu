
/// Errors that can lead to undefined behavior or cause CPU reset
#[derive(Debug, Eq, PartialEq)]
pub enum MOSError {
    Break,
    OutOfBounds,
    InvalidOpcode,
}

