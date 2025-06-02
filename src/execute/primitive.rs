use crate::{TamEmulator, errors::TamResult};

impl TamEmulator {
    pub(super) fn exec_prim_and(&mut self) -> TamResult<()> {
        let op2 = self.pop()?;
        let op1 = self.pop()?;
        self.push(if op1 * op2 != 0 { 1 } else { 0 })
    }

    pub(super) fn exec_prim_or(&mut self) -> TamResult<()> {
        let op2 = self.pop()?;
        let op1 = self.pop()?;
        self.push(if op1 + op2 != 0 && op1 != -op2 { 1 } else { 0 })
    }

    pub(super) fn exec_prim_not(&mut self) -> TamResult<()> {
        let op = self.pop()?;
        self.push(if op == 0 { 1 } else { 0 })
    }
}
