
/// Errors specific to the project
#[derive(Debug, Eq, PartialEq)]
pub enum RustNesError {
    InvalidHeader,
    Break,
    OutOfBounds,
    InvalidOpcode(u8),
}

