use crate::{
    HT, ST, TamEmulator, TamInstruction,
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
}
