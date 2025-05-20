#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TamError {
    OutOfMemory,
    CodeAccessViolation,
}

pub type TamResult<T> = Result<T, TamError>;
