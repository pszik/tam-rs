#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TamError {
    CodeAccessViolation,
}

pub type TamResult<T> = Result<T, TamError>;
