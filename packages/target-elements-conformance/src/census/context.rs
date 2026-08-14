//! The generic transaction the introspection cases read.
//!
//! # One transaction, stated once
//!
//! Every introspection case reads the same development transaction, so
//! that the expectation of one primitive can be checked against the field
//! another primitive reads. It carries two inputs and three outputs,
//! which is the smallest shape that still admits a first and a last index
//! on both axes and an out-of-range index that is only just out of range
//! (Guide-9 §13.3, §13.4).
//!
//! # What the executor materializes, and what it supplies
//!
//! Exactly this transaction, with these inputs and outputs in this order,
//! this version, and this locktime, validating the input at
//! [`CURRENT_INPUT_INDEX`] through the fixture's own script. Nothing may
//! be added to it: an extra fee or change output would move every output
//! index these cases state, and the expectations would then describe a
//! transaction that never ran.
//!
//! Several fields are `None`, and that is not laziness. An outpoint names
//! a funding output the executor created; the program of the input under
//! validation commits to the fixture's own leaf script; and which asset a
//! development network issues is the network's fact. A fixture that named
//! any of them would be describing one deployment rather than the target,
//! so it states none and the cases reading them state a verdict rather
//! than bytes.
//!
//! # What is stated is stated exactly
//!
//! The transaction version, the locktime, the input sequences, the output
//! amounts, the absent nonces, and the two counts are the executor's
//! instructions, and the cases that read them state their exact stack
//! bytes. An executor that changes one of them has materialized a
//! different transaction, and the case fails — which is the point.
//!
//! # Deliberately absent
//!
//! No blinded asset, amount, or nonce, and no issuance. The target admits
//! all of them and §13.3, §13.4, and §13.15 ask for them; they are absent
//! because the reviewed executor cannot yet materialize a confidential or
//! issuing transaction, and a case it must answer with infrastructure
//! trouble is not evidence of anything. The gap is recorded as a residual
//! rather than filled with cases that cannot run.

use crate::fixture::{FixtureInput, FixtureOutput, FixtureScriptPath, PrimitiveExecutionContext};

/// The transaction version the census transaction carries.
///
/// Two, because the relative-timelock primitive requires at least that
/// and every other case is indifferent to it.
pub const TRANSACTION_VERSION: u32 = 2;

/// The locktime the census transaction carries.
///
/// # Why this number and not a rounder one
///
/// A lock time below the height threshold names a block height, and a
/// transaction carrying one is not merely unusual until that height is
/// reached — it is *non-final*, and a node refuses it without ever
/// running the script. So the value a fixture states is a chain
/// requirement as much as a field to be introspected, and the executor
/// has to grow a disposable chain past it before any of these cases can
/// reach the interpreter at all.
///
/// The first native run made that concrete: at the round five hundred
/// thousand this census originally stated, every context-bearing case —
/// a hundred and thirty-one of them — came back as infrastructure
/// trouble, because half a million blocks is not a chain a run can
/// build. The height is stated small enough to be reached and large
/// enough to be a distinctive four-byte field, which is all the
/// reviewed contract ever asked of it.
pub const TRANSACTION_LOCKTIME: u32 = 1_000;

/// Which input the census transaction validates.
pub const CURRENT_INPUT_INDEX: u32 = 0;

/// How many inputs the census transaction carries.
pub const INPUT_COUNT: i64 = 2;

/// How many outputs the census transaction carries.
pub const OUTPUT_COUNT: i64 = 3;

/// The index of the census transaction's last input.
pub const LAST_INPUT_INDEX: i64 = INPUT_COUNT - 1;

/// The index of the census transaction's last output.
pub const LAST_OUTPUT_INDEX: i64 = OUTPUT_COUNT - 1;

/// The first input's sequence.
///
/// Every bit set, which is both the largest a field can carry and the
/// value that disables the relative timelock — the timelock cases state
/// their own sequences rather than relying on this one.
pub const FIRST_INPUT_SEQUENCE: u32 = 0xffff_ffff;

/// The second input's sequence, the smallest a field can carry.
pub const SECOND_INPUT_SEQUENCE: u32 = 0x0000_0000;

/// The amount the first output pays.
pub const FIRST_OUTPUT_AMOUNT: u64 = 600_000;

/// The amount the second output pays.
pub const SECOND_OUTPUT_AMOUNT: u64 = 300_000;

/// The amount the last output pays.
pub const LAST_OUTPUT_AMOUNT: u64 = 90_000;

/// The prefix byte of a field carried in the clear.
pub const EXPLICIT_PREFIX: u8 = 0x01;

/// The prefix byte of an absent field.
pub const NULL_PREFIX: u8 = 0x00;

/// One amount carried in the clear, in transaction byte order.
///
/// Most significant byte first, which is the order the field stores and
/// *not* the order the introspection primitives push.
pub fn explicit_amount_field(amount: u64) -> Vec<u8> {
    let mut field = vec![EXPLICIT_PREFIX];
    field.extend_from_slice(&amount.to_be_bytes());
    field
}

/// One absent field, which is its prefix alone.
pub fn null_field() -> Vec<u8> {
    vec![NULL_PREFIX]
}

/// The script path a fixture spends through.
///
/// The leaf version and the script are filled in from the fixture itself
/// when the fixture is stated, so that a context can never claim a leaf
/// or a leaf script other than the one that ran, and the control block is
/// the executor's own.
const fn script_path() -> FixtureScriptPath {
    FixtureScriptPath {
        leaf_version: 0,
        script: Vec::new(),
        control: None,
    }
}

/// One input the executor funds, spent at one sequence.
const fn funded_input(sequence: u32) -> FixtureInput {
    FixtureInput {
        outpoint_txid: None,
        outpoint_index: None,
        spent_asset: None,
        spent_value: None,
        spent_program: None,
        sequence,
        issuance: None,
        witness: Vec::new(),
    }
}

/// One output paying a stated amount with an absent nonce.
fn output(amount: u64) -> FixtureOutput {
    FixtureOutput {
        asset: None,
        value: explicit_amount_field(amount),
        nonce: null_field(),
        program: None,
    }
}

/// The census transaction.
pub fn census_transaction() -> PrimitiveExecutionContext {
    PrimitiveExecutionContext {
        version: TRANSACTION_VERSION,
        locktime: TRANSACTION_LOCKTIME,
        current_input_index: CURRENT_INPUT_INDEX,
        inputs: vec![
            funded_input(FIRST_INPUT_SEQUENCE),
            funded_input(SECOND_INPUT_SEQUENCE),
        ],
        outputs: vec![
            output(FIRST_OUTPUT_AMOUNT),
            output(SECOND_OUTPUT_AMOUNT),
            output(LAST_OUTPUT_AMOUNT),
        ],
        script_path: script_path(),
    }
}

/// The census transaction validating one particular input.
///
/// Only the current-index primitive can tell the difference, and it is
/// the one primitive whose whole result is which input is running.
pub fn census_transaction_at_input(index: u32) -> PrimitiveExecutionContext {
    PrimitiveExecutionContext {
        current_input_index: index,
        ..census_transaction()
    }
}

/// The census transaction at one version, with one sequence on the input
/// being validated.
///
/// The relative-timelock cases vary exactly those two: the transaction
/// version is the primitive's prerequisite, and the input's own sequence
/// is what the operand is compared against.
pub fn timelock_transaction(version: u32, sequence: u32) -> PrimitiveExecutionContext {
    PrimitiveExecutionContext {
        version,
        inputs: vec![funded_input(sequence), funded_input(SECOND_INPUT_SEQUENCE)],
        ..census_transaction()
    }
}
