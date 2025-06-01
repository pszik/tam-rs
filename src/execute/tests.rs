use super::*;
use crate::SB;
use rstest::*;

#[fixture]
fn emulator() -> TamEmulator {
    TamEmulator::new(false)
}

#[inline]
fn set_test_program(emu: &mut TamEmulator, prog: &[u32]) {
    emu.code_store.fill(0);
    emu.code_store[..prog.len()].copy_from_slice(prog);
    emu.registers[CT] = prog.len() as u16;
}

#[inline]
fn set_test_data(emu: &mut TamEmulator, data: &[i16]) {
    emu.data_store.fill(0);
    emu.data_store[..data.len()].copy_from_slice(data);
    emu.registers[ST] = data.len() as u16;
}

#[rstest]
fn test_exec_load_all_in_range_ok(mut emulator: TamEmulator) {
    set_test_data(&mut emulator, &[0x12, 0x98]);

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
    set_test_program(&mut emulator, &[0x12, 0x98]);

    let instr = TamInstruction {
        op: 0,
        r: 4,
        n: 2,
        d: 20,
    };

    let res = emulator.exec_load(instr);
    assert_eq!(TamError::DataAccessViolation, res.unwrap_err());
}

#[rstest]
fn test_exec_load_stack_full_stack_overflow(mut emulator: TamEmulator) {
    emulator.registers[ST] = 2;
    emulator.registers[HT] = 1;

    let res = emulator.exec_load(TamInstruction {
        op: 0,
        r: 0,
        n: 1,
        d: 0,
    });
    assert_eq!(TamError::StackOverflow, res.unwrap_err());
}

#[rstest]
fn text_exec_loada_ok(mut emulator: TamEmulator) {
    emulator.registers[SB] = 5;
    let instr = TamInstruction {
        op: 1,
        r: SB as u8,
        n: 0,
        d: 3,
    };

    let res = emulator.exec_loada(instr);
    assert!(res.is_ok());
    assert_eq!(8, emulator.data_store[0]);
}

#[rstest]
fn test_exec_loada_stack_full_stack_overflow(mut emulator: TamEmulator) {
    emulator.registers[ST] = 3;
    emulator.registers[HT] = 3;

    let res = emulator.exec_loada(TamInstruction {
        op: 1,
        r: 0,
        n: 1,
        d: 0,
    });
    assert_eq!(TamError::StackOverflow, res.unwrap_err());
}

#[rstest]
fn test_exec_loadi_all_in_range_ok(mut emulator: TamEmulator) {
    set_test_data(&mut emulator, &[5, 10, 15, 1]);
    emulator.registers[ST] = 4;

    let instr = TamInstruction {
        op: 2,
        r: SB as u8,
        n: 2,
        d: 1,
    };
    let res = emulator.exec_loadi(instr);

    assert!(res.is_ok());
    assert_eq!(10, emulator.data_store[3], "Wrong first value loaded");
    assert_eq!(15, emulator.data_store[4], "Wrong second value loaded");
    assert_eq!(5, emulator.registers[ST], "ST incorrect");
}

#[rstest]
fn test_exec_loadi_empty_stack_stack_underflow(mut emulator: TamEmulator) {
    let res = emulator.exec_loadi(TamInstruction {
        op: 0,
        r: 0,
        n: 0,
        d: 0,
    });
    assert_eq!(TamError::StackUnderflow, res.unwrap_err());
}

#[rstest]
fn test_exec_loadi_full_stack_stack_overflow(mut emulator: TamEmulator) {
    emulator.registers[ST] = 2;
    emulator.registers[HT] = 2;

    let res = emulator.exec_loadi(TamInstruction {
        op: 0,
        r: 0,
        n: 2,
        d: 0,
    });
    assert_eq!(TamError::StackOverflow, res.unwrap_err());
}

#[rstest]
fn test_exec_loadi_addr_out_of_range_data_access_violation(mut emulator: TamEmulator) {
    emulator.data_store[0] = 25;
    emulator.registers[ST] = 1;

    let res = emulator.exec_loadi(TamInstruction {
        op: 0,
        r: 0,
        n: 1,
        d: 0,
    });
    assert_eq!(TamError::DataAccessViolation, res.unwrap_err());
}

#[rstest]
fn test_exec_loadl_all_in_range_ok(mut emulator: TamEmulator) {
    let instr = TamInstruction {
        op: 3,
        r: 0,
        n: 0,
        d: 84,
    };
    let res = emulator.exec_loadl(instr);

    assert!(res.is_ok());
    assert_eq!(84, emulator.data_store[0]);
    assert_eq!(1, emulator.registers[ST]);
}

#[rstest]
fn test_exec_loadl_stack_full_stack_overflow(mut emulator: TamEmulator) {
    emulator.registers[ST] = 2;
    emulator.registers[HT] = 2;

    let instr = TamInstruction {
        op: 3,
        r: 0,
        n: 0,
        d: 84,
    };
    let res = emulator.exec_loadl(instr);

    assert_eq!(TamError::StackOverflow, res.unwrap_err());
}

#[rstest]
fn test_exec_store_all_in_range_ok(mut emulator: TamEmulator) {
    set_test_data(&mut emulator, &[0, 1, 2, 3, 4, 5]);

    let instr = TamInstruction {
        op: 4,
        r: SB as u8,
        n: 2,
        d: 1,
    };
    let res = emulator.exec_store(instr);

    assert!(res.is_ok());
    assert_eq!(4, emulator.data_store[1]);
    assert_eq!(5, emulator.data_store[2]);
    assert_eq!(4, emulator.registers[ST]);
}

#[rstest]
fn test_exec_store_not_enough_data_stack_underflow(mut emulator: TamEmulator) {
    let instr = TamInstruction {
        op: 4,
        r: SB as u8,
        n: 2,
        d: 1,
    };
    let res = emulator.exec_store(instr);
    assert_eq!(TamError::StackUnderflow, res.unwrap_err());
}

#[rstest]
fn test_exec_store_addr_out_of_range_data_access_violation(mut emulator: TamEmulator) {
    emulator.data_store[0] = 1;
    emulator.registers[ST] = 1;
    let instr = TamInstruction {
        op: 4,
        r: SB as u8,
        n: 1,
        d: 10,
    };
    let res = emulator.exec_store(instr);
    assert_eq!(
        TamError::DataAccessViolation,
        res.expect_err("result should not have been Ok")
    );
}

#[rstest]
fn test_exec_storei_all_in_range_ok(mut emulator: TamEmulator) {
    set_test_data(&mut emulator, &[0, 1, 2, 3, 4, 5, 1]);

    let instr = TamInstruction {
        op: 4,
        r: SB as u8,
        n: 2,
        d: 1,
    };
    let res = emulator.exec_storei(instr);

    assert!(res.is_ok());
    assert_eq!(4, emulator.data_store[1]);
    assert_eq!(5, emulator.data_store[2]);
    assert_eq!(4, emulator.registers[ST]);
}

#[rstest]
fn test_exec_storei_not_enough_data_stack_underflow(mut emulator: TamEmulator) {
    let instr = TamInstruction {
        op: 4,
        r: SB as u8,
        n: 2,
        d: 1,
    };
    let res = emulator.exec_storei(instr);
    assert_eq!(TamError::StackUnderflow, res.unwrap_err());
}

#[rstest]
fn test_exec_storei_addr_out_of_range_data_access_violation(mut emulator: TamEmulator) {
    emulator.data_store[0] = 1;
    emulator.registers[ST] = 1;
    let instr = TamInstruction {
        op: 4,
        r: SB as u8,
        n: 1,
        d: 10,
    };
    let res = emulator.exec_store(instr);
    assert_eq!(
        TamError::DataAccessViolation,
        res.expect_err("result should not have been Ok")
    );
}

#[rstest]
fn test_exec_call_all_in_range_ok(mut emulator: TamEmulator) {
    emulator.registers[CT] = 20;
    emulator.registers[LB] = 3;
    emulator.registers[CP] = 7;

    let instr = TamInstruction {
        op: 6,
        r: 0,
        n: 4,
        d: 2,
    };
    let res = emulator.exec_call(instr);

    assert!(res.is_ok());
    assert_eq!(0, emulator.data_store[0], "wrong static link");
    assert_eq!(3, emulator.data_store[1], "wrong dynamic link");
    assert_eq!(7, emulator.data_store[2], "wrong return address");
    assert_eq!(2, emulator.registers[CP], "jumped to wrong location");
}

#[rstest]
fn test_exec_call_stack_full_stack_overflow(mut emulator: TamEmulator) {
    emulator.registers[CT] = 20;
    emulator.registers[LB] = 3;
    emulator.registers[CP] = 7;
    emulator.registers[ST] = 4;
    emulator.registers[HT] = 4;

    let instr = TamInstruction {
        op: 6,
        r: 0,
        n: 4,
        d: 2,
    };
    let res = emulator.exec_call(instr);

    assert_eq!(TamError::StackOverflow, res.unwrap_err());
}

#[rstest]
fn test_exec_call_invalid_target_code_access_violation(mut emulator: TamEmulator) {
    emulator.registers[CT] = 20;
    emulator.registers[LB] = 3;
    emulator.registers[CP] = 7;

    let instr = TamInstruction {
        op: 6,
        r: 0,
        n: 4,
        d: 22,
    };
    let res = emulator.exec_call(instr);

    assert_eq!(TamError::CodeAccessViolation, res.unwrap_err());
}

#[rstest]
fn test_exec_return_all_valid_ok(mut emulator: TamEmulator) {}
