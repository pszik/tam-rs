use clap::Parser;
use std::{fs::File, io::Read};
use tam_rs::{
    TamEmulator,
    errors::{TamError, TamResult},
};

#[derive(Parser)]
struct Cli {
    /// Name of file to read program from
    prog_file: String,
    /// Print each instruction as it is executed
    #[arg(short, long)]
    trace: bool,
}

fn main() -> TamResult<()> {
    // load program from file
    let cli = Cli::parse();
    let code = read_code_from_file(&cli.prog_file).map_err(|_| TamError::IOError)?;
    let mut emu = TamEmulator::new(cli.trace);
    emu.set_program(&code)?;

    // CPU cycle
    let mut running = true;
    while running {
        let instr = emu.fetch_decode()?;
        running = emu.execute(instr)?;
    }

    Ok(())
}

fn read_code_from_file(filename: &str) -> std::io::Result<Vec<u8>> {
    let f = File::open(filename)?;
    let mut code = Vec::new();
    for b in f.bytes() {
        code.push(b?);
    }

    Ok(code)
}
