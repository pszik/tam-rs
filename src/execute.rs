use crate::{
    HT, SB, ST, TamEmulator, TamInstruction,
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[fixture]
    fn emulator() -> TamEmulator {
        TamEmulator::new(false)
    }

    #[rstest]
    fn test_exec_load_all_in_range_ok(mut emulator: TamEmulator) {
        emulator.data_store[0] = 0x12;
        emulator.data_store[1] = 0x98;
        emulator.registers[ST] = 2;

        let instr = TamInstruction {
            op: 0,
            r: 4,
            n: 2,
            d: 0,
        };

        let res = emulator.exec_load(instr);
        assert!(res.is_ok());

        assert_eq!(4, emulator.registers[ST], "ST register incorrect");
        assert_eq!(
            0x12, emulator.data_store[2],
            "first value incorrectly loaded"
        );
        assert_eq!(
            0x98, emulator.data_store[3],
            "second value incorrectly loaded"
        );
    }

    #[rstest]
    fn test_exec_load_addr_out_of_range_data_access_violation(mut emulator: TamEmulator) {
        emulator.data_store[0] = 0x12;
        emulator.data_store[1] = 0x98;
        emulator.registers[ST] = 2;

        let instr = TamInstruction {
            op: 0,
            r: 4,
            n: 2,
            d: 20,
        };

        let res = emulator.exec_load(instr);
        assert_eq!(TamError::DataAccessViolation, res.unwrap_err());
    }

    #[rstest]
    fn test_exec_load_stack_full_stack_overflow(mut emulator: TamEmulator) {
        emulator.registers[ST] = 2;
        emulator.registers[HT] = 1;

        let res = emulator.exec_load(TamInstruction {
            op: 0,
            r: 0,
            n: 1,
            d: 0,
        });
        assert_eq!(TamError::StackOverflow, res.unwrap_err());
    }

    #[rstest]
    fn text_exec_loada_ok(mut emulator: TamEmulator) {
        emulator.registers[SB] = 5;
        let instr = TamInstruction {
            op: 1,
            r: SB as u8,
            n: 0,
            d: 3,
        };

        let res = emulator.exec_loada(instr);
        assert!(res.is_ok());
        assert_eq!(8, emulator.data_store[0]);
    }

    #[rstest]
    fn test_exec_loada_stack_full_stack_overflow(mut emulator: TamEmulator) {
        emulator.registers[ST] = 3;
        emulator.registers[HT] = 3;

        let res = emulator.exec_loada(TamInstruction {
            op: 1,
            r: 0,
            n: 1,
            d: 0,
        });
        assert_eq!(TamError::StackOverflow, res.unwrap_err());
    }

    #[rstest]
    fn test_exec_loadi_all_in_range_ok(mut emulator: TamEmulator) {
        emulator.data_store[0] = 5;
        emulator.data_store[1] = 10;
        emulator.data_store[2] = 15;
        emulator.data_store[3] = 1;
        emulator.registers[ST] = 4;

        let instr = TamInstruction {
            op: 2,
            r: SB as u8,
            n: 2,
            d: 1,
        };
        let res = emulator.exec_loadi(instr);

        assert!(res.is_ok());
        assert_eq!(10, emulator.data_store[3], "Wrong first value loaded");
        assert_eq!(15, emulator.data_store[4], "Wrong second value loaded");
        assert_eq!(5, emulator.registers[ST], "ST incorrect");
    }

    #[rstest]
    fn test_exec_loadi_empty_stack_stack_underflow(mut emulator: TamEmulator) {
        let res = emulator.exec_loadi(TamInstruction {
            op: 0,
            r: 0,
            n: 0,
            d: 0,
        });
        assert_eq!(TamError::StackUnderflow, res.unwrap_err());
    }

    #[rstest]
    fn test_exec_loadi_full_stack_stack_overflow(mut emulator: TamEmulator) {
        emulator.registers[ST] = 2;
        emulator.registers[HT] = 2;

        let res = emulator.exec_loadi(TamInstruction {
            op: 0,
            r: 0,
            n: 2,
            d: 0,
        });
        assert_eq!(TamError::StackOverflow, res.unwrap_err());
    }

    #[rstest]
    fn test_exec_loadi_addr_out_of_range_data_access_violation(mut emulator: TamEmulator) {
        emulator.data_store[0] = 25;
        emulator.registers[ST] = 1;

        let res = emulator.exec_loadi(TamInstruction {
            op: 0,
            r: 0,
            n: 1,
            d: 0,
        });
        assert_eq!(TamError::DataAccessViolation, res.unwrap_err());
    }

    #[rstest]
    fn test_exec_loadl_all_in_range_ok(mut emulator: TamEmulator) {
        let instr = TamInstruction {
            op: 3,
            r: 0,
            n: 0,
            d: 84,
        };
        let res = emulator.exec_loadl(instr);

        assert!(res.is_ok());
        assert_eq!(84, emulator.data_store[0]);
        assert_eq!(1, emulator.registers[ST]);
    }

    #[rstest]
    fn test_exec_loadl_stack_full_stack_overflow(mut emulator: TamEmulator) {
        emulator.registers[ST] = 2;
        emulator.registers[HT] = 2;

        let instr = TamInstruction {
            op: 3,
            r: 0,
            n: 0,
            d: 84,
        };
        let res = emulator.exec_loadl(instr);

        assert_eq!(TamError::StackOverflow, res.unwrap_err());
    }

    #[rstest]
    fn test_exec_store_all_in_range_ok(mut emulator: TamEmulator) {
        emulator.data_store[0] = 0;
        emulator.data_store[1] = 1;
        emulator.data_store[2] = 2;
        emulator.data_store[3] = 3;
        emulator.data_store[4] = 4;
        emulator.data_store[5] = 5;
        emulator.registers[ST] = 6;

        let instr = TamInstruction {
            op: 4,
            r: SB as u8,
            n: 2,
            d: 1,
        };
        let res = emulator.exec_store(instr);

        assert!(res.is_ok());
        assert_eq!(4, emulator.data_store[1]);
        assert_eq!(5, emulator.data_store[2]);
        assert_eq!(4, emulator.registers[ST]);
    }

    #[rstest]
    fn test_exec_store_not_enough_data_stack_underflow(mut emulator: TamEmulator) {
        let instr = TamInstruction {
            op: 4,
            r: SB as u8,
            n: 2,
            d: 1,
        };
        let res = emulator.exec_store(instr);
        assert_eq!(TamError::StackUnderflow, res.unwrap_err());
    }

    #[rstest]
    fn test_exec_store_addr_out_of_range_data_access_violation(mut emulator: TamEmulator) {
        emulator.data_store[0] = 1;
        emulator.registers[ST] = 1;
        let instr = TamInstruction {
            op: 4,
            r: SB as u8,
            n: 1,
            d: 10,
        };
        let res = emulator.exec_store(instr);
        assert_eq!(
            TamError::DataAccessViolation,
            res.expect_err("result should not have been Ok")
        );
    }

    #[rstest]
    fn test_exec_storei_all_in_range_ok(mut emulator: TamEmulator) {
        emulator.data_store[0] = 0;
        emulator.data_store[1] = 1;
        emulator.data_store[2] = 2;
        emulator.data_store[3] = 3;
        emulator.data_store[4] = 4;
        emulator.data_store[5] = 5;
        emulator.data_store[6] = 1;
        emulator.registers[ST] = 7;

        let instr = TamInstruction {
            op: 4,
            r: SB as u8,
            n: 2,
            d: 1,
        };
        let res = emulator.exec_storei(instr);

        assert!(res.is_ok());
        assert_eq!(4, emulator.data_store[1]);
        assert_eq!(5, emulator.data_store[2]);
        assert_eq!(4, emulator.registers[ST]);
    }

    #[rstest]
    fn test_exec_storei_not_enough_data_stack_underflow(mut emulator: TamEmulator) {
        let instr = TamInstruction {
            op: 4,
            r: SB as u8,
            n: 2,
            d: 1,
        };
        let res = emulator.exec_storei(instr);
        assert_eq!(TamError::StackUnderflow, res.unwrap_err());
    }

    #[rstest]
    fn test_exec_storei_addr_out_of_range_data_access_violation(mut emulator: TamEmulator) {
        emulator.data_store[0] = 1;
        emulator.registers[ST] = 1;
        let instr = TamInstruction {
            op: 4,
            r: SB as u8,
            n: 1,
            d: 10,
        };
        let res = emulator.exec_store(instr);
        assert_eq!(
            TamError::DataAccessViolation,
            res.expect_err("result should not have been Ok")
        );
    }
}
