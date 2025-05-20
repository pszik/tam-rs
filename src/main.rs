use tam_rs::TamEmulator;

fn main() {
    let mut emu = TamEmulator::new();
    let instr = emu.fetch_decode();
}
