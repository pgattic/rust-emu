
/// Errors that can lead to undefined behavior or cause CPU reset
#[derive(Debug, Eq, PartialEq)]
pub enum RustNesError {
    InvalidHeader,
    Break,
    OutOfBounds,
    InvalidOpcode,
}

