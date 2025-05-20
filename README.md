# tam-rs

A Triangle Abstract Machine emulator written in Rust.

The executable `tam-rs` expects the name of a file containing TAM bytecode. It will 
load the program from this file and then run it. Optionally, the emulator can be 
instructed to print each instruction as it executes using the `trace` option.

See `tam-rs -h` for full instructions.
