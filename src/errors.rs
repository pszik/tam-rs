#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TamError {
    OutOfMemory,
    CodeAccessViolation,
    DataAccessViolation,
    StackOverflow,
    StackUnderflow,
    UnknownOpcode(u8),
    IOError,
}

pub type TamResult<T> = Result<T, TamError>;
