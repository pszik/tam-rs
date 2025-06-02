mod primitive;
use crate::{
    CP, CT, HT, LB, ST, TamEmulator, TamInstruction,
    errors::{TamError, TamResult},
};

impl TamEmulator {
    fn calc_address(&self, instr: TamInstruction) -> u16 {
        u16::wrapping_add_signed(self.registers[instr.r as usize], instr.d)
    }

    pub(super) fn exec_load(&mut self, instr: TamInstruction) -> TamResult<()> {
        let addr = self.calc_address(instr);

        for i in 0..instr.n {
            let addr = addr + i as u16;
            if addr >= self.registers[ST] && addr <= self.registers[HT] {
                return Err(TamError::DataAccessViolation);
            }

            self.push(self.data_store[addr as usize])?;
        }

        Ok(())
    }

    pub(super) fn exec_loada(&mut self, instr: TamInstruction) -> TamResult<()> {
        let addr = self.calc_address(instr);
        self.push(addr as i16)
    }

    pub(super) fn exec_loadi(&mut self, instr: TamInstruction) -> TamResult<()> {
        let addr = self.pop()? as u16;

        for i in 0..instr.n {
            let addr = addr + i as u16;
            if addr >= self.registers[ST] && addr <= self.registers[HT] {
                return Err(TamError::DataAccessViolation);
            }

            self.push(self.data_store[addr as usize])?;
        }

        Ok(())
    }

    pub(super) fn exec_loadl(&mut self, instr: TamInstruction) -> TamResult<()> {
        self.push(instr.d)
    }

    pub(super) fn exec_store(&mut self, instr: TamInstruction) -> TamResult<()> {
        let mut data = Vec::new();
        for _ in 0..instr.n {
            data.push(self.pop()?);
        }

        let addr = self.calc_address(instr);
        for i in 0..instr.n {
            let addr = addr + i as u16;
            if addr >= self.registers[ST] && addr <= self.registers[HT] {
                return Err(TamError::DataAccessViolation);
            }
            self.data_store[addr as usize] = data.pop().expect("unexpectedly stored too much data");
        }
        Ok(())
    }

    pub(super) fn exec_storei(&mut self, instr: TamInstruction) -> TamResult<()> {
        let addr = self.pop()? as u16;

        let mut data = Vec::new();
        for _ in 0..instr.n {
            data.push(self.pop()?);
        }

        for i in 0..instr.n {
            let addr = addr + i as u16;
            if addr >= self.registers[ST] && addr <= self.registers[HT] {
                return Err(TamError::DataAccessViolation);
            }
            self.data_store[addr as usize] = data.pop().expect("unexpectedly stored too much data");
        }
        Ok(())
    }

    pub(super) fn exec_call_primitive(&mut self, offset: i16) -> TamResult<()> {
        assert!(
            offset > 0 && offset < 29,
            "exec_call_primitive received invalid offset {offset}"
        );

        match offset {
            1 => Ok(()), // id
            2 => self.exec_prim_and(),
            3 => self.exec_prim_or(),
            4 => self.exec_prim_not(),
            _ => Ok(()),
        }
    }

    pub(super) fn exec_call(&mut self, instr: TamInstruction) -> TamResult<()> {
        let static_link = self.registers[instr.n as usize];
        let dynamic_link = self.registers[LB];
        let return_address = self.registers[CP];

        self.push(static_link as i16)?;
        self.push(dynamic_link as i16)?;
        self.push(return_address as i16)?;

        self.registers[LB] = self.registers[ST] - 3;
        let addr = self.calc_address(instr);
        if addr >= self.registers[CT] {
            return Err(TamError::CodeAccessViolation);
        }

        self.registers[CP] = addr;
        Ok(())
    }

    pub(super) fn exec_return(&mut self, instr: TamInstruction) -> TamResult<()> {
        // pop result and save link data
        let mut return_val = Vec::new();
        for _ in 0..instr.n {
            return_val.push(self.pop()?);
        }

        let dynamic_link_addr = self.registers[LB] + 1;
        let dynamic_link = self.data_store[dynamic_link_addr as usize];
        let return_addr_addr = self.registers[LB] + 2;
        let return_addr = self.data_store[return_addr_addr as usize];

        // pop stack frame
        while self.registers[ST] != self.registers[LB] {
            self.pop()?;
        }

        // pop args
        for _ in 0..instr.d {
            self.pop()?;
        }

        // push result
        for _ in 0..instr.n {
            self.push(
                return_val
                    .pop()
                    .expect("return value had wrong number of bytes"),
            )?;
        }

        // update registers
        self.registers[LB] = dynamic_link as u16;
        self.registers[CP] = return_addr as u16;

        Ok(())
    }
}

#[cfg(test)]
mod tests;
