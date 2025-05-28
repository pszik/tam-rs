pub mod errors;
mod execute;

use byteorder::{BE, ReadBytesExt};
use errors::*;
use std::{
    fmt::{self, Display},
    io::Cursor,
};

pub const MEMORY_SIZE: usize = 65536;
pub const MEMORY_MAX: usize = MEMORY_SIZE - 1;

pub const CT: usize = 1;
pub const PB: usize = 2;
pub const PT: usize = 3;
pub const SB: usize = 4;
pub const ST: usize = 5;
pub const HB: usize = 6;
pub const HT: usize = 7;
pub const LB: usize = 8;
pub const CP: usize = 15;

/// A single TAM instruction.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TamInstruction {
    /// Opcode
    op: u8,
    /// Register
    r: u8,
    /// Unsigned operand
    n: u8,
    /// Signed operand/offset
    d: i16,
}

impl From<u32> for TamInstruction {
    fn from(value: u32) -> Self {
        let op = (value & 0xf0000000) >> 28;
        let r = (value & 0x0f000000) >> 24;
        let n = (value & 0x00ff0000) >> 16;
        let d = value & 0x0000ffff;
        TamInstruction {
            op: op as u8,
            r: r as u8,
            n: n as u8,
            d: d as i16,
        }
    }
}

impl Display for TamInstruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.op {
            0 => write!(f, "LOAD({}) {}[{}]", self.n, self.d, self.r),
            1 => write!(f, "LOADA {}[{}]", self.d, self.r),
            2 => write!(f, "LOADI({})", self.n),
            3 => write!(f, "LOADL {}", self.d),
            4 => write!(f, "STORE({}) {}[{}]", self.n, self.d, self.r),
            5 => write!(f, "STOREI({})", self.n),
            6 => write!(f, "CALL({}) {}[{}]", self.n, self.d, self.r),
            7 => write!(f, "CALLI"),
            8 => write!(f, "RETURN({}) {}", self.n, self.d),
            10 => write!(f, "PUSH {}", self.d),
            11 => write!(f, "POP({}), {}", self.n, self.d),
            12 => write!(f, "JUMP {}[{}]", self.d, self.r),
            13 => write!(f, "JUMPI"),
            14 => write!(f, "JUMPIF({}) {}[{}]", self.n, self.r, self.d),
            15 => write!(f, "HALT"),
            x => write!(f, "unrecognised opcode {}", x),
        }
    }
}

#[derive(Debug)]
pub struct TamEmulator {
    pub code_store: [u32; MEMORY_SIZE],
    pub data_store: [i16; MEMORY_SIZE],
    pub registers: [u16; 16],
    trace: bool,
}

impl TamEmulator {
    /// Constructs a new TAM emulator with zeroed memory and default registers.
    pub fn new(trace: bool) -> TamEmulator {
        let mut emu = TamEmulator {
            code_store: [0; MEMORY_SIZE],
            data_store: [0; MEMORY_SIZE],
            registers: [0; 16],
            trace,
        };

        emu.registers[HB] = MEMORY_MAX as u16;
        emu.registers[HT] = MEMORY_MAX as u16;
        emu
    }

    /// Sets the program to be executed on this emulator.
    ///
    /// This method zeroes the code store and writes the given bytes into it, beginning
    /// from the first byte. Then the registers `CT`, `PB`, and `PT` are set based on
    /// the size of the given program.
    ///
    /// # Example
    ///
    /// ```
    /// let mut emu = tam_rs::TamEmulator::new(false);
    /// let code = vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
    /// let res = emu.set_program(&code);
    /// assert!(res.is_ok());
    /// assert_eq!(0x01020304, emu.code_store[0]);
    /// assert_eq!(0x05060708, emu.code_store[1]);
    /// ```
    pub fn set_program(&mut self, code: &Vec<u8>) -> TamResult<()> {
        if code.len() / 4 > MEMORY_SIZE {
            return Err(TamError::OutOfMemory);
        }

        self.code_store.fill(0);
        let instr_count = code.len() / 4;

        let mut c = Cursor::new(code);
        let mut i = 0;
        while i < instr_count {
            self.code_store[i] = c.read_u32::<BE>().unwrap();
            i += 1;
        }

        self.registers[CT] = (code.len() / 4) as u16;
        self.registers[PB] = self.registers[CT];
        self.registers[PT] = self.registers[PB] + 29;

        Ok(())
    }

    /// Gets the next instruction to be executed and increments `CP`.
    pub fn fetch_decode(&mut self) -> TamResult<TamInstruction> {
        let addr = self.registers[CP];
        if addr >= self.registers[CT] {
            return Err(TamError::CodeAccessViolation);
        }

        self.registers[CP] += 1;
        let code = self.code_store[addr as usize];
        Ok(TamInstruction::from(code))
    }

    fn push(&mut self, value: i16) -> TamResult<()> {
        let addr = self.registers[ST];
        if addr >= self.registers[HT] {
            return Err(TamError::StackOverflow);
        }

        self.data_store[addr as usize] = value;
        self.registers[ST] += 1;
        Ok(())
    }

    fn pop(&mut self) -> TamResult<i16> {
        if self.registers[ST] == 0 {
            return Err(TamError::StackUnderflow);
        }

        self.registers[ST] -= 1;
        let val = self.data_store[self.registers[ST] as usize];
        Ok(val)
    }

    /// Executes the given instruction.
    pub fn execute(&mut self, instr: TamInstruction) -> TamResult<bool> {
        if self.trace {
            println!("{:#06x}: {}", self.registers[CP] - 1, instr);
        }

        match instr.op {
            0 => self.exec_load(instr)?,
            1 => self.exec_loada(instr)?,
            2 => self.exec_loadi(instr)?,
            3 => self.exec_loadl(instr)?,
            4 => self.exec_store(instr)?,
            5 => self.exec_storei(instr)?,
            6 => self.exec_call(instr)?,
            7 => todo!("exec_calli"),
            8 => todo!("exec_return"),
            10 => todo!("exec_push"),
            11 => todo!("exec_pop"),
            12 => todo!("exec_jump"),
            13 => todo!("exec_jumpi"),
            14 => todo!("exec_jumpif"),
            15 => return Ok(false),
            _ => return Err(TamError::UnknownOpcode(instr.op)),
        }

        Ok(true)
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
    #[case(0x00000000, TamInstruction { op: 0, r: 0, n: 0, d: 0 })]
    #[case(0x12345678, TamInstruction { op: 1, r: 2, n: 52, d: 22136 })]
    #[case(0xa8765432, TamInstruction { op: 10, r: 8, n: 118, d:21554 })]
    #[case(0xffffffff, TamInstruction { op: 15, r: 15, n: 255, d: -1 })]
    fn test_taminstruction_from_u32(#[case] code: u32, #[case] instr: TamInstruction) {
        assert_eq!(instr, TamInstruction::from(code));
    }

    #[rstest]
    fn test_fetch_decode_cp_in_range_ok(mut emulator: TamEmulator) {
        emulator.code_store[0] = 0x12;
        emulator.code_store[1] = 0x12345678;
        emulator.code_store[2] = 0x56;
        emulator.code_store[3] = 0x78;
        emulator.registers[CT] = 4;
        emulator.registers[CP] = 1;

        match emulator.fetch_decode() {
            Err(e) => panic!("unexpected error: {:?}", e),
            Ok(instr) => assert_eq!(
                TamInstruction {
                    op: 1,
                    r: 2,
                    n: 52,
                    d: 22136
                },
                instr
            ),
        }
    }

    #[rstest]
    fn test_fetch_decode_cp_out_of_range_err(mut emulator: TamEmulator) {
        emulator.registers[CP] = 3;

        match emulator.fetch_decode() {
            Ok(_) => panic!("unexpected success"),
            Err(e) => assert_eq!(TamError::CodeAccessViolation, e),
        }
    }

    #[rstest]
    fn test_push_stack_has_space_ok(mut emulator: TamEmulator) {
        let res = emulator.push(23);
        assert!(res.is_ok());
        assert_eq!(23, emulator.data_store[0]);
        assert_eq!(1, emulator.registers[ST]);
    }

    #[rstest]
    fn test_push_stack_full_stack_overflow(mut emulator: TamEmulator) {
        emulator.registers[ST] = 2;
        emulator.registers[HT] = 2;
        let res = emulator.push(-81);
        assert_eq!(TamError::StackOverflow, res.unwrap_err());
    }

    #[rstest]
    fn test_pop_stack_has_data_ok(mut emulator: TamEmulator) {
        emulator.data_store[0] = 77;
        emulator.registers[ST] = 1;
        let res = emulator.pop();
        assert_eq!(77, res.unwrap());
    }

    #[rstest]
    fn test_pop_stack_empty_stack_underflow(mut emulator: TamEmulator) {
        let res = emulator.pop();
        assert_eq!(TamError::StackUnderflow, res.unwrap_err());
    }
}
