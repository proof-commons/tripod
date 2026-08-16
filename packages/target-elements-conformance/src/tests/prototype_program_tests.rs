//! The emitted constructor prototype, and what it costs.

use tapscript::instruction::TapscriptInstruction;
use target_elements::{OpcodeId, ResourceDimension, StackValueType, reviewed_elements_tapscript};

use crate::constructor::internal_key::UNSPENDABLE_INTERNAL_KEY;
use crate::constructor::tagged::{TAP_BRANCH_TAG, TAP_LEAF_TAG, TAP_TWEAK_TAG, sha256};
use crate::prototype_program::{
    PrototypeKind, PrototypeProgram, PrototypeProgramRelation, PrototypeStatus,
};

/// The reviewed contract, for a test that needs one.
fn target() -> target_elements::ReviewedElementsTapscriptDefinition {
    reviewed_elements_tapscript().expect("the reviewed contract validates")
}

/// The admitted continuity prototype.
fn prototype() -> PrototypeProgram {
    PrototypeProgram::continuity(&target()).expect("the continuity prototype is admitted")
}

#[test]
fn the_continuity_prototype_is_admitted_against_the_reviewed_contracts() {
    // The whole of tranche B in one assertion: the constructor emitted
    // the program, every primitive it names is in the reviewed census,
    // the abstract validator accepted the emitted bytes' schedule, that
    // schedule ends in exactly one true item with no surviving failure
    // state, and no resource exceeds a bound the contract states.
    let program = prototype();

    assert_eq!(program.kind(), PrototypeKind::MetadataConstructorContinuity);
    assert_eq!(
        program.relation(),
        PrototypeProgramRelation::MetadataConstructorContinuity
    );
    // The constructor never sets the accepting status: nothing it
    // checked was a node's verdict.
    assert_eq!(program.status(), PrototypeStatus::Experimental);
    assert_eq!(program.target().contract_version(), 2);
    assert_eq!(
        program.target().leaf_version(),
        target().definition().leaf_version().get()
    );
}

#[test]
fn the_emitted_program_carries_the_real_constants_and_not_placeholders() {
    // A schedule carries literals of the right width. This program
    // carries the values a spend does, and the difference is checkable:
    // every tagged hash's prefix is the tag's own digest twice, and the
    // internal key is the published unspendable point.
    let program = prototype();
    let script = program.encode(&target());

    for tag in [TAP_LEAF_TAG, TAP_BRANCH_TAG, TAP_TWEAK_TAG] {
        let digest = sha256(tag.as_bytes());
        let mut prefix = digest.to_vec();
        prefix.extend_from_slice(&digest);
        assert!(
            script.windows(prefix.len()).any(|window| window == prefix),
            "the emitted script carries the {tag} prefix"
        );
    }

    assert!(
        script
            .windows(UNSPENDABLE_INTERNAL_KEY.len())
            .any(|window| window == UNSPENDABLE_INTERNAL_KEY),
        "the emitted script carries the published internal key"
    );

    // And no run of thirty-two zero bytes, which is what a placeholder
    // literal of digest width would have left behind.
    assert!(
        !script.windows(32).any(|window| window == [0_u8; 32]),
        "no placeholder literal survives in the emitted program"
    );
}

#[test]
fn the_emitted_program_carries_both_curve_checks_and_no_model_of_one() {
    // The Wave-5 schedules modelled the curve step's declared stack
    // effect because the tweak operand was declared as one exact
    // encoding. The operand now says what the target says, so the
    // emitted program carries the primitive itself — twice, once per
    // constructor derivation.
    let program = prototype();
    let curve = program
        .program()
        .instructions()
        .iter()
        .filter(|instruction| {
            matches!(
                instruction,
                TapscriptInstruction::Opcode(OpcodeId::TweakVerify)
            )
        })
        .count();
    assert_eq!(curve, 2);
    assert_eq!(program.resources().curve_operations(), 2);
}

#[test]
fn the_emitted_program_round_trips_through_the_reviewed_parser() {
    // Determinism and encodability in one check: the bytes parse back
    // into the same typed program, so nothing the emitter pushed is
    // outside the reviewed subset and no push is in a non-minimal form
    // (Guide-10 `rule:guide10:stack-schedule`).
    let target = target();
    let program = prototype();
    let script = program.encode(&target);
    let parsed = tapscript::program::TapscriptProgram::decode(&target, &script)
        .expect("the emitted script is in the reviewed subset");
    assert_eq!(&parsed, program.program());

    // And emitting twice gives the same bytes.
    assert_eq!(script, program.encode(&target));
}

#[test]
fn the_resource_table_is_the_measured_one() {
    // Guide-10 §20.1, as a typed table rather than prose. Every number
    // is measured off the emitted program and the witness it consumes;
    // none is a hand count.
    //
    // ```text
    // dimension              measured   reviewed bound   share
    // script bytes                593   unbounded            -
    // witness bytes               862   unbounded            -
    // peak main stack               9   1000             0.9 %
    // largest element             103   520             19.8 %
    // hash primitives              16   unbounded            -
    // curve checks                  2   unbounded            -
    // validation budget           100   witness + 50    11.0 %
    // control path nodes            1   128              0.8 %
    // witness weight              862   400000 policy    0.2 %
    // ```
    //
    // # Which limit binds first
    //
    // The stack element bound, at five hundred and twenty bytes. The
    // widest element the execution ever holds is the streaming-hash
    // context at one hundred and three bytes, and no other dimension
    // reaches a fifth of its bound: the peak stack is nine items against
    // one thousand, the control path is one node against one hundred and
    // twenty-eight, and the witness weighs eight hundred and sixty-two
    // units against a four-hundred-thousand policy ceiling.
    //
    // The validation budget is not the binding one, which is worth
    // stating because it is the dimension a reader expects to bind. The
    // reviewed domain funds it from the witness size plus an offset of
    // fifty, so this witness funds nine hundred and twelve units and the
    // two curve checks charge one hundred. A construction that added
    // curve checks without adding witness would move that ratio, and a
    // construction that widened a stack element would hit five hundred
    // and twenty first.
    //
    // Neither the script bytes nor the operation cost is bounded by the
    // reviewed execution domain at all, which is a reviewed fact rather
    // than an unfilled row.
    let program = prototype();
    let measured = program.resources();

    assert_eq!(measured.script_bytes(), 593);
    assert_eq!(measured.witness_bytes(), 862);
    assert_eq!(measured.peak_main_stack(), 9);
    assert_eq!(measured.largest_element_bytes(), 103);
    assert_eq!(measured.hash_operations(), 16);
    assert_eq!(measured.curve_operations(), 2);
    assert_eq!(measured.validation_budget(), 100);

    // The projection is internally consistent: the reviewed domain
    // weighs a witness byte as one unit.
    assert_eq!(measured.witness_weight(), measured.witness_bytes());

    // And every bounded dimension is inside its bound, with the element
    // width the closest of them. Read from the contract rather than
    // restated, so a reviewed change to a bound reaches this table.
    let reviewed = target();
    let bounds = reviewed.definition().resources().consensus().bounds();
    let maximum = |dimension: ResourceDimension| {
        bounds
            .get(&dimension)
            .and_then(|bound| bound.maximum())
            .expect("the reviewed contract bounds this dimension")
    };

    assert!(measured.largest_element_bytes() <= maximum(ResourceDimension::StackElementBytes));
    assert!(measured.peak_main_stack() <= maximum(ResourceDimension::PeakStackItems));

    // The element bound is the tightest: the share of it this program
    // uses exceeds the share of every other bounded dimension.
    let element_share = share(
        measured.largest_element_bytes(),
        maximum(ResourceDimension::StackElementBytes),
    );
    let stack_share = share(
        measured.peak_main_stack(),
        maximum(ResourceDimension::PeakStackItems),
    );
    let path_share = share(1, maximum(ResourceDimension::ControlPathDepth));
    assert!(element_share > stack_share);
    assert!(element_share > path_share);

    // The budget the witness funds, against what the curve checks
    // charge. The offset is a reviewed number rather than a restated
    // one.
    let offset = reviewed
        .definition()
        .resources()
        .consensus()
        .validation_budget_offset();
    let funded = measured.witness_bytes() + offset;
    assert!(measured.validation_budget() < funded);
    assert!(share(measured.validation_budget(), funded) < element_share);
}

/// What share of a bound a measurement uses, in parts per thousand.
///
/// Integer arithmetic on purpose: the comparison the table makes is an
/// ordering between shares, and a floating-point ratio would introduce a
/// representation question the ordering does not need.
fn share(measured: u64, maximum: u64) -> u64 {
    measured.saturating_mul(1_000) / maximum.max(1)
}

#[test]
fn the_resource_profile_does_not_move_with_the_metadata_or_the_parity() {
    // The §20.1 table asks for a minimum, a representative, and a
    // maximum metadata object, and for both output-key parities. This
    // construction has one row for all of them, and the reason is worth
    // stating rather than leaving as an apparent omission.
    //
    // Every witness item is fixed-width by the schema: the metadata
    // objects are exactly the schema's object width, the static root is
    // a digest, and each output key is a compressed point. So there is
    // no minimum or maximum metadata object to measure — there is one
    // width — and the parity bit rides inside the compressed key's
    // leading byte without changing any width at all.
    //
    // The program is a constant too: it pushes no witness-dependent
    // literal, so the emitted bytes are the same for every instance.
    let program = prototype();
    let script = program.encode(&target());

    // Nothing in the emitted program depends on an instance.
    assert_eq!(script, prototype().encode(&target()));

    // And the initial stack contract fixes every item's width, which is
    // what makes the single row honest.
    for value in program.initial_stack().main() {
        let settled = match value {
            StackValueType::Bytes { minimum, maximum } => minimum == maximum,
            StackValueType::Encoded(_) => true,
            _ => false,
        };
        assert!(settled, "every witness item has one width: {value:?}");
    }
}

#[test]
fn the_emitted_program_does_not_yet_carry_the_metadata_transition() {
    // The residual this wave found and did not close, recorded as a
    // checked absence rather than left for a reader to notice.
    //
    // The metadata transition proof — the counter incremented by exactly
    // one, every other field pinned, both reserved fields zero —
    // schedules on its own, and so does the continuity proof. Composing
    // them into one program is not currently expressible: the transition
    // needs the two metadata objects adjacent, the continuity proof
    // needs the one static root between them so it can outlive both
    // derivations, and no reviewed primitive reaches past the third item
    // to reorder them.
    //
    // So the emitted program proves that two constructors share one
    // static root, and it does not prove that the successor's metadata
    // is the predecessor's successor. The counter increment is the
    // visible marker of that gap: the transition is the only part of the
    // construction that does arithmetic, and this program does none.
    let program = prototype();
    let arithmetic = program
        .program()
        .instructions()
        .iter()
        .filter(|instruction| {
            matches!(
                instruction,
                TapscriptInstruction::Opcode(
                    OpcodeId::Add64 | OpcodeId::Sub64 | OpcodeId::Mul64 | OpcodeId::Div64
                )
            )
        })
        .count();
    assert_eq!(
        arithmetic, 0,
        "the transition proof is not composed into the continuity program"
    );
}

#[test]
fn a_prototype_program_compares_by_its_typed_content() {
    // Two independently constructed prototypes are equal, and equality
    // is the whole comparison: there is no digest of a prototype
    // program and no field reserved for one.
    assert_eq!(prototype(), prototype());
}
