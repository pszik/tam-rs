#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TamError {
    OutOfMemory,
    CodeAccessViolation,
    UnknownOpcode(u8),
    IOError,
}

pub type TamResult<T> = Result<T, TamError>;
