use tam_rs::{CP, ST};

mod common;
use common::cpu_cycle;

#[test]
fn simple_cpu_cycle_test() {
    let mut emulator = tam_rs::TamEmulator::new(false);
    emulator
        .set_program(&vec![0x30, 0x00, 0x12, 0x34])
        .expect("failed to set program");

    let running = cpu_cycle(&mut emulator).expect("CPU cycle failed");
    assert!(running);
    assert_eq!(4660, emulator.data_store[0]);
    assert_eq!(1, emulator.registers[CP]);
    assert_eq!(1, emulator.registers[ST]);
}
