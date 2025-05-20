mod errors;
use errors::*;

const MEMORY_SIZE: usize = 65536;
const MEMORY_MAX: usize = MEMORY_SIZE - 1;

const CT: usize = 1;
const HB: usize = 6;
const HT: usize = 7;
const CP: usize = 15;

#[derive(Debug)]
pub struct TamEmulator {
    code_store: [u32; MEMORY_SIZE],
    data_store: [i16; MEMORY_SIZE],
    registers: [u16; 16],
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
    /// Construct a new TAM emulator with zeroed memory and default registers.
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

    /// Get the next instruction to be executed. Increments the code pointer when called.
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
