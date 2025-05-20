mod errors;

use byteorder::{BE, ReadBytesExt};
use errors::*;
use std::io::Cursor;

const MEMORY_SIZE: usize = 65536;
const MEMORY_MAX: usize = MEMORY_SIZE - 1;

const CT: usize = 1;
const PB: usize = 2;
const PT: usize = 3;
const SB: usize = 4;
const ST: usize = 5;
const HB: usize = 6;
const HT: usize = 7;
const CP: usize = 15;

#[derive(Debug)]
pub struct TamEmulator {
    pub code_store: [u32; MEMORY_SIZE],
    pub data_store: [i16; MEMORY_SIZE],
    pub registers: [u16; 16],
}

/// A single TAM instruction.
#[derive(Copy, Clone, Debug)]
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

impl TamEmulator {
    /// Constructs a new TAM emulator with zeroed memory and default registers.
    pub fn new() -> TamEmulator {
        let mut emu = TamEmulator {
            code_store: [0; MEMORY_SIZE],
            data_store: [0; MEMORY_SIZE],
            registers: [0; 16],
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
    /// let mut emu = tam_rs::TamEmulator::new();
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
}
