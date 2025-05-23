use tam_rs::{TamEmulator, errors::TamResult};

pub fn cpu_cycle(emu: &mut TamEmulator) -> TamResult<bool> {
    let instr = emu.fetch_decode()?;
    emu.execute(instr)
}
