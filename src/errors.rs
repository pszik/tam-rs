#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TamError {
    OutOfMemory,
    CodeAccessViolation,
    UnknownOpcode(u8),
}

pub type TamResult<T> = Result<T, TamError>;
