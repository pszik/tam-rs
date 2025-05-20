const MEMORY_SIZE: usize = 65536;
const MEMORY_MAX: usize = MEMORY_SIZE - 1;

/// Index of CP register
const CP: usize = 15;

pub struct TamEmulator {
    code_store: [u32; MEMORY_SIZE],
    data_store: [i16; MEMORY_SIZE],
    registers: [u16; 16],
}

/// A single TAM instruction.
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
        TamEmulator {
            code_store: [0; MEMORY_SIZE],
            data_store: [0; MEMORY_SIZE],
            registers: [0; 16],
        }
    }

    pub fn fetch_decode(&mut self) -> TamInstruction {
        let code = self.code_store[self.registers[CP] as usize];
        TamInstruction::from(code)
    }
}
