//! The emitted wide-floor program, and what admitting it establishes.

use std::collections::BTreeSet;

use tapscript::instruction::TapscriptInstruction;
use target_elements::OpcodeId;

use crate::prototype_program::{
    PrototypeKind, PrototypeProgram, PrototypeProgramRelation, PrototypeStatus,
};
use crate::tests::support::reviewed_target;
use crate::wide_floor::candidate::CandidateComparison;
use crate::wide_floor::candidate::{
    ComparisonBasis, WideFloorCandidate, comparison, disposition, emitted_instruction_count,
    selected,
};
use crate::wide_floor::schedule::{
    AMOUNT_AT, PACKED_AMOUNTS_BYTES, PACKED_PROOF_BYTES, QUOTIENT_AT, initial_stack_types,
    proof_instructions,
};

#[test]
fn the_wide_floor_program_is_admitted_against_the_reviewed_contract() {
    let target = reviewed_target();
    let emitted = PrototypeProgram::wide_floor(&target).expect("the schedule is admitted");
    assert_eq!(emitted.kind(), PrototypeKind::WideFloorRelation);
    assert_eq!(
        emitted.relation(),
        PrototypeProgramRelation::WideFloorRelation
    );
    // The constructor never sets the accepting status: nothing it checks
    // is a node's verdict.
    assert_eq!(emitted.status(), PrototypeStatus::Experimental);
    assert_eq!(emitted.target().contract_version(), 2);
}

#[test]
fn the_emitted_bytes_are_the_same_on_every_emission() {
    let target = reviewed_target();
    let first = PrototypeProgram::wide_floor(&target).expect("admitted");
    let second = PrototypeProgram::wide_floor(&target).expect("admitted");
    assert_eq!(first, second);
    assert_eq!(first.encode(&target), second.encode(&target));
}

#[test]
fn the_witness_is_five_canonical_fixed_width_amounts() {
    let target = reviewed_target();
    let emitted = PrototypeProgram::wide_floor(&target).expect("admitted");
    assert_eq!(emitted.initial_stack().main().len(), 5);
    assert_eq!(emitted.initial_stack().main(), initial_stack_types());
    assert_eq!(AMOUNT_AT.len(), 5);
    assert_eq!(PACKED_AMOUNTS_BYTES, 40);
}

#[test]
fn the_packed_layout_is_internally_consistent() {
    // Every field is eight bytes and the layout is append-only, so each
    // offset is its predecessor's plus a width and the final width is
    // the last offset plus one more.
    for window in AMOUNT_AT.windows(2) {
        assert_eq!(window[1] - window[0], 8);
    }
    assert_eq!(AMOUNT_AT[0], 0);
    assert_eq!(QUOTIENT_AT, 16);
    assert_eq!(PACKED_PROOF_BYTES % 8, 0);
    // Five amounts, ten limbs, and twelve normalization results.
    assert_eq!(PACKED_PROOF_BYTES, 8 * (5 + 10 + 12));
}

#[test]
fn the_program_uses_no_primitive_outside_the_arithmetic_and_byte_string_groups() {
    let target = reviewed_target();
    let emitted = PrototypeProgram::wide_floor(&target).expect("admitted");
    let admitted: BTreeSet<OpcodeId> = BTreeSet::from([
        OpcodeId::Add64,
        OpcodeId::Mul64,
        OpcodeId::Div64,
        OpcodeId::LessThan64,
        OpcodeId::GreaterThan64,
        OpcodeId::GreaterThanOrEqual64,
        OpcodeId::Concatenate,
        OpcodeId::Substring,
        OpcodeId::Duplicate,
        OpcodeId::DuplicateTwo,
        OpcodeId::CopyOver,
        OpcodeId::Swap,
        OpcodeId::Rotate,
        OpcodeId::RemoveSecond,
        OpcodeId::Drop,
        OpcodeId::EqualVerify,
        OpcodeId::Verify,
    ]);
    for instruction in emitted.program().instructions() {
        if let TapscriptInstruction::Opcode(id) = instruction {
            assert!(admitted.contains(id), "unexpected primitive {id:?}");
        }
    }
}

#[test]
fn every_arithmetic_flag_is_verified_on_the_next_instruction() {
    let target = reviewed_target();
    let emitted = PrototypeProgram::wide_floor(&target).expect("admitted");
    let instructions = emitted.program().instructions();
    let flagged = [OpcodeId::Add64, OpcodeId::Mul64, OpcodeId::Div64];
    let mut seen = 0_usize;
    for (index, instruction) in instructions.iter().enumerate() {
        let TapscriptInstruction::Opcode(id) = instruction else {
            continue;
        };
        if !flagged.contains(id) {
            continue;
        }
        seen += 1;
        assert_eq!(
            instructions.get(index + 1),
            Some(&TapscriptInstruction::Opcode(OpcodeId::Verify)),
            "the flag of {id:?} at {index} is not verified immediately"
        );
    }
    // Three divisions per amount decomposition and per normalization
    // step, plus every product and sum: a schedule with no flagged
    // primitive at all would pass the loop above vacuously.
    assert!(seen >= 20, "only {seen} flagged primitives were found");
}

#[test]
fn the_pattern_ends_in_the_authenticated_quotient_before_the_framing() {
    let target = reviewed_target();
    let emitted = PrototypeProgram::wide_floor(&target).expect("admitted");
    let boundary = proof_instructions(&target).expect("the schedule is emittable");
    let instructions = emitted.program().instructions();
    assert_eq!(boundary + 2, instructions.len());
    // The framing is exactly the drop of the value nothing composes with
    // and the domain's required true item.
    assert_eq!(
        instructions[boundary],
        TapscriptInstruction::Opcode(OpcodeId::Drop)
    );
    let TapscriptInstruction::Push(item) = &instructions[boundary + 1] else {
        panic!("the final instruction is a literal");
    };
    assert_eq!(item.bytes(), &[1_u8]);
    // The instruction before the boundary discharges the proof-local
    // packed item, leaving the quotient alone.
    assert_eq!(
        instructions[boundary - 1],
        TapscriptInstruction::Opcode(OpcodeId::RemoveSecond)
    );
}

#[test]
fn the_measured_resources_are_inside_every_stated_bound() {
    let target = reviewed_target();
    let emitted = PrototypeProgram::wide_floor(&target).expect("admitted");
    let resources = emitted.resources();
    // Admission already refused anything above a stated bound; these
    // assert the figures are measurements rather than zeroes.
    // The §20.2 measurement, pinned. A change to any of these is a
    // change to what the prototype costs, and is meant to be read in a
    // diff rather than discovered later.
    assert_eq!(
        (
            resources.script_bytes(),
            resources.witness_bytes(),
            resources.peak_main_stack(),
            resources.largest_element_bytes(),
            resources.witness_weight(),
            emitted.program().len(),
        ),
        (523, 606, 7, 216, 606, 304)
    );
    assert!(resources.witness_bytes() > 0);
    assert!(resources.peak_main_stack() >= 5);
    assert_eq!(
        resources.largest_element_bytes(),
        u64::try_from(PACKED_PROOF_BYTES).expect("the packed width is representable")
    );
    // No hashing and no curve operation: the pattern is arithmetic.
    assert_eq!(resources.hash_operations(), 0);
    assert_eq!(resources.curve_operations(), 0);
    assert_eq!(resources.validation_budget(), 0);
}

#[test]
fn the_comparison_covers_every_candidate_exactly_once() {
    let target = reviewed_target();
    let rows = comparison(&target).expect("the emitted candidate is admitted");
    let named: BTreeSet<WideFloorCandidate> =
        rows.iter().map(CandidateComparison::candidate).collect();
    assert_eq!(named.len(), rows.len());
    assert_eq!(named, WideFloorCandidate::ALL.iter().copied().collect());
    for candidate in WideFloorCandidate::ALL {
        assert_ne!(disposition(*candidate), "");
    }
}

#[test]
fn exactly_one_row_is_measured_and_it_is_the_selected_candidate() {
    let target = reviewed_target();
    let rows = comparison(&target).expect("admitted");
    let measured: Vec<_> = rows
        .iter()
        .filter(|row| row.basis() == ComparisonBasis::Measured)
        .collect();
    assert_eq!(measured.len(), 1);
    assert_eq!(measured[0].candidate(), selected());
    assert_eq!(selected(), WideFloorCandidate::DerivedLimbs);
    assert!(measured[0].measured().is_some());
    for row in &rows {
        if row.basis() == ComparisonBasis::LowerBound {
            assert!(row.measured().is_none());
        }
    }
}

#[test]
fn the_taken_candidate_is_smaller_on_the_dimensions_that_decided_it() {
    let target = reviewed_target();
    let rows = comparison(&target).expect("admitted");
    let row = |candidate: WideFloorCandidate| {
        *rows
            .iter()
            .find(|row| row.candidate() == candidate)
            .expect("every candidate has a row")
    };
    let taken = row(WideFloorCandidate::DerivedLimbs);
    let witnessed = row(WideFloorCandidate::WitnessedLimbs);
    let sandwich = row(WideFloorCandidate::Sandwich);

    // Against the witnessed form: five times the witness, and more
    // arithmetic besides. Its one advantage — no divisions — does not
    // offset either.
    assert!(witnessed.witness_items() > taken.witness_items() * 5);
    assert_eq!(witnessed.counts().divisions, 0);
    assert!(witnessed.counts().multiplications > taken.counts().multiplications);
    assert!(witnessed.counts().comparisons > taken.counts().comparisons);

    // Against the sandwich: a smaller witness and more of every
    // arithmetic class, which is the trade Guide 10 warns about.
    assert!(sandwich.witness_items() < taken.witness_items());
    assert!(sandwich.counts().multiplications > taken.counts().multiplications);
    assert!(sandwich.counts().divisions > taken.counts().divisions);
    assert!(sandwich.counts().comparisons > taken.counts().comparisons);
}

#[test]
fn the_measured_counts_are_the_emitted_programs_own() {
    let target = reviewed_target();
    let rows = comparison(&target).expect("admitted");
    let taken = rows
        .iter()
        .find(|row| row.candidate() == WideFloorCandidate::DerivedLimbs)
        .expect("the taken candidate has a row");
    let counts = taken.counts();
    // Two sides, four partial products each.
    assert_eq!(counts.multiplications, 8);
    // Five amount decompositions and six normalization steps.
    assert_eq!(counts.divisions, 11);
    // Four limb equalities.
    assert!(counts.verifications > counts.multiplications);
    assert_eq!(
        emitted_instruction_count(&target),
        Some(proof_instructions(&target).expect("emittable") + 2)
    );
}
