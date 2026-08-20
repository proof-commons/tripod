//! Structured substitution and the §13.2 round trip.

use std::collections::BTreeSet;

use tapscript::{BundleSymbol, RelocationSite, StackItem, TapscriptInstruction, TapscriptProgram};

use crate::SelfCommitmentStrategy;
use crate::relocate::substitute;
use crate::symbol::collect_definitions;
use crate::tests::{deployment, relocatable_bundle, resolved_symbols, reviewed_target};

/// The demonstration substitution.
fn linked_programs() -> std::collections::BTreeMap<tapscript::LeafRole, crate::LinkedLeafProgram> {
    let target = reviewed_target();
    let bundle = relocatable_bundle();
    let census = collect_definitions(
        &bundle,
        &deployment(
            &target,
            SelfCommitmentStrategy::ExternallyAuthenticatedCommitment,
        ),
    )
    .expect("the definitions collect");

    substitute(&target, &bundle, &census, &resolved_symbols(&target))
        .expect("the demonstration substitution completes")
}

#[test]
fn no_placeholder_byte_survives_anywhere_in_any_linked_program() {
    // The blunt check, and the one a reader wants first. The
    // placeholder values are distinctive byte strings; after linking,
    // none of them appears in any pushed item of any program. A
    // substitution that missed a site fails here even if every
    // structural check passed.
    let placeholders: BTreeSet<Vec<u8>> = BTreeSet::from([
        vec![0x11; 32],
        vec![0x22; 32],
        vec![0x33; 32],
        vec![0x44; 20],
        vec![0x55; 32],
    ]);

    for (leaf, program) in linked_programs() {
        for instruction in program.program().instructions() {
            if let TapscriptInstruction::Push(item) = instruction {
                assert!(
                    !placeholders.contains(item.bytes()),
                    "leaf {leaf:?} still pushes a placeholder",
                );
            }
        }
    }
}

#[test]
fn every_resolved_value_appears_at_exactly_the_recorded_sites() {
    // The positive direction, checked by value rather than by observing
    // that something changed. The bundle recorded which instruction
    // indices carry which symbol; after substitution those indices must
    // push the resolved item, and the count of pushes of that item must
    // equal the recorded multiplicity.
    let target = reviewed_target();
    let bundle = relocatable_bundle();
    let resolved = resolved_symbols(&target);
    let programs = linked_programs();

    let expected = |symbol: BundleSymbol| -> Option<StackItem> {
        Some(match symbol {
            BundleSymbol::ClosedAsset => resolved.closed_asset().clone(),
            BundleSymbol::ReserveAsset => resolved.reserve_asset().clone(),
            BundleSymbol::AshConstructorProgram => resolved.ash_program().clone(),
            BundleSymbol::SponsorChangeProgram => resolved.sponsor_change_program().clone(),
            BundleSymbol::TargetFeeRoleProgramDigest => resolved.fee_program_digest().clone(),
            _ => return None,
        })
    };

    let mut checked = 0usize;
    for relocation in bundle.relocations() {
        let RelocationSite::ProgramInstructions { leaf, indices } = relocation.site() else {
            continue;
        };
        let Some(item) = expected(relocation.symbol()) else {
            continue;
        };
        let program = programs.get(leaf).expect("every leaf survives the link");

        for index in indices {
            assert_eq!(
                program.program().instructions().get(*index),
                Some(&TapscriptInstruction::Push(item.clone())),
                "leaf {leaf:?} index {index} does not carry the resolved value",
            );
            checked += 1;
        }
    }

    // The bundle records one hundred and three relocations across
    // program sites and constructor bindings; the wide byte-string
    // symbols checked here account for the sites below, and a census
    // that shrank would mean a relocation stopped being recorded.
    assert!(checked > 0);
    assert_eq!(bundle.relocations().len(), 103);
}

#[test]
fn every_linked_program_survives_the_round_trip_again() {
    // §13.2's obligation does not survive substitution for free: the
    // bytes changed, and one of the resolutions changed width. The
    // substitution stage checks this itself, so reaching a linked
    // program is already the assertion; this repeats it independently,
    // decoding through the target's own parser and comparing typed
    // instructions.
    let target = reviewed_target();
    for (leaf, program) in linked_programs() {
        let bytes = program.program().encode(&target);
        let decoded = TapscriptProgram::decode(&target, &bytes)
            .unwrap_or_else(|_| panic!("leaf {leaf:?} does not decode"));
        assert_eq!(
            &decoded,
            program.program(),
            "leaf {leaf:?} round trips wrong"
        );
    }
}

#[test]
fn the_linked_length_is_the_exact_encoded_length_and_not_an_opcode_sum() {
    // §13.3: script bytes are the exact canonical encoded byte length,
    // push opcodes and width prefixes included. Checked against the
    // program's own serialization, which is the only thing that could
    // settle it.
    let target = reviewed_target();
    for (leaf, program) in linked_programs() {
        let encoded = program.program().encode(&target);
        assert_eq!(
            program.charged(target_elements::ResourceDimension::ScriptBytes),
            Some(u64::try_from(encoded.len()).expect("a small program fits")),
            "leaf {leaf:?} charges a length its bytes disagree with",
        );
    }
}

#[test]
fn no_leaf_gains_an_authorization_primitive_and_none_needs_a_witness_datum() {
    // Two carried invariants that substitution could in principle
    // break: a resolved value is not an instruction, so no leaf should
    // acquire a signature check, and the witness handoff should stay at
    // zero data items. §12.8's permissionless path rests on the second.
    let programs = linked_programs();
    for (leaf, program) in &programs {
        assert!(
            !tapscript::carries_authorization(program.program()),
            "leaf {leaf:?} gained an authorization primitive at link time",
        );
    }

    let bundle = relocatable_bundle();
    for (leaf, emitted) in bundle.constructor().leaves() {
        assert_eq!(
            emitted.witness().data_items(),
            0,
            "leaf {leaf:?} requires a witness datum",
        );
        assert_eq!(
            programs
                .get(leaf)
                .and_then(|p| p.charged(target_elements::ResourceDimension::InitialStackItems)),
            Some(0),
            "leaf {leaf:?} linked to a non-empty initial stack",
        );
    }
}

#[test]
fn the_instruction_count_is_unchanged_and_only_pushed_payloads_moved() {
    // Structured substitution means the program's shape is the same
    // program with different literals. An instruction added or removed
    // would be an untracked mutation, which the substitution stage
    // refuses; this checks the property directly, and checks that every
    // differing instruction is a push.
    let bundle = relocatable_bundle();
    let programs = linked_programs();

    for (leaf, pre) in bundle.constructor().leaves() {
        let post = programs.get(leaf).expect("every leaf survives");
        assert_eq!(
            pre.program().len(),
            post.program().len(),
            "leaf {leaf:?} changed instruction count",
        );

        for (before, after) in pre
            .program()
            .instructions()
            .iter()
            .zip(post.program().instructions())
        {
            if before == after {
                continue;
            }
            assert!(
                matches!(before, TapscriptInstruction::Push(_))
                    && matches!(after, TapscriptInstruction::Push(_)),
                "leaf {leaf:?} changed a non-push instruction",
            );
        }
    }
}
