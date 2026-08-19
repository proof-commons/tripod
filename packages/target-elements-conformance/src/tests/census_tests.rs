//! Census closure, determinism, and independent-oracle tests.

use std::collections::{BTreeMap, BTreeSet};

use tapscript::{TapscriptInstruction, TapscriptProgram};
use target_elements::{
    ExecutionDomain, FailureOutcome, OpcodeId, ReviewedElementsTapscriptDefinition,
    SuccessContract, TargetEvidenceRequirementId,
};

use super::support::{development_binding, reviewed_target};
use crate::fixture::{
    FixtureScriptSource, LeafVersionStatus, NativeCaseGroup, PrimitiveFixture, PrimitiveFixtureSet,
    canonical_fixture_set,
};
use crate::report::EvidencePlanClass;
use crate::validate::{guide_nine_evidence_plan, requirements_for_tests};

/// The primitives with no accepting native case, and why.
///
/// Two reasons, both structural rather than incidental.
///
/// A signature over the transaction sighash depends on the transaction
/// the executor materializes, so no static fixture can carry one. The
/// checking form still has a non-aborting case — an empty signature
/// consumes its operands and pushes a false — but the verifying form has
/// nothing that is not an abort, and inventing a signature to close the
/// gap would be inventing evidence.
///
/// The rest push two, three, or six items, and the reviewed domain
/// requires evaluation to finish with exactly one. Nothing in the
/// reviewed primitive census can consume the surplus: there is no
/// equality, no drop, and no verify primitive that takes an arbitrary
/// item, and the one reduction that exists — reading a one-byte prefix
/// as a script number and comparing — needs the item beneath it to be
/// eight bytes wide, which an asset payload, a program, and a digest are
/// not. So their accepting path is unreachable from reviewed primitives
/// alone, and their cases establish the *count* the primitive pushed
/// instead. Admitting the target's base script primitives into the
/// reviewed contract would close this; nothing in this package can.
const WITHOUT_ACCEPTING_CASE: &[OpcodeId] = &[
    OpcodeId::CheckSig,
    OpcodeId::CheckSigVerify,
    OpcodeId::InspectInputOutpoint,
    OpcodeId::InspectInputAsset,
    OpcodeId::InspectInputScriptPubKey,
    OpcodeId::InspectInputIssuance,
    OpcodeId::InspectOutputAsset,
    OpcodeId::InspectOutputScriptPubKey,
    OpcodeId::InspectOutputNonce,
];

/// The primitives with no aborting native case, and why.
///
/// The whole-transaction primitives take no operand and, in the reviewed
/// contract, declare no context failure. The only failure left to them
/// is executing outside the reviewed domain, which needs a domain this
/// contract deliberately does not describe.
const WITHOUT_ABORTING_CASE: &[OpcodeId] = &[OpcodeId::InspectVersion, OpcodeId::InspectLockTime];

/// The canonical census, for the tests below.
fn census() -> PrimitiveFixtureSet {
    let target = reviewed_target();
    let binding = development_binding(&target);
    canonical_fixture_set(&target, &binding)
        .expect("the census is expressible")
        .into_fixtures()
}

#[test]
fn the_census_is_not_empty_and_holds_no_duplicate_identity() {
    let census = census();
    assert!(!census.is_empty());
    let identities: BTreeSet<_> = census.iter().map(PrimitiveFixture::case).collect();
    assert_eq!(
        identities.len(),
        census.len(),
        "a repeated identity would make one case's result ambiguous",
    );
}

#[test]
fn the_census_is_the_same_census_every_time() {
    // Nothing in the authoring depends on iteration order, a clock, or
    // the environment, so two builds are the same value.
    assert_eq!(census(), census());
}

#[test]
fn every_reviewed_primitive_is_natively_covered_both_ways() {
    let census = census();
    let mut accepting: BTreeSet<OpcodeId> = BTreeSet::new();
    let mut aborting: BTreeSet<OpcodeId> = BTreeSet::new();

    for fixture in &census {
        let Some(id) = fixture.case().opcode() else {
            continue;
        };
        if fixture.expected().is_accepting() {
            accepting.insert(id);
        } else if fixture.expected().static_final_stack().is_none() {
            // No final stack at all means no evaluation completed: the
            // case is an abort rather than a false value.
            aborting.insert(id);
        }
    }

    for id in OpcodeId::ALL {
        if !WITHOUT_ACCEPTING_CASE.contains(id) {
            assert!(
                accepting.contains(id),
                "{id:?} has no accepting native case",
            );
        }
        if !WITHOUT_ABORTING_CASE.contains(id) {
            assert!(aborting.contains(id), "{id:?} has no aborting native case");
        }
    }
}

#[test]
fn the_exceptions_are_exceptions_and_not_holes() {
    // Every primitive the closure test excuses must still be covered the
    // other way round, so no primitive is unexercised.
    let census = census();
    let exercised: BTreeSet<OpcodeId> = census
        .iter()
        .filter_map(|fixture| fixture.case().opcode())
        .collect();
    for id in WITHOUT_ACCEPTING_CASE.iter().chain(WITHOUT_ABORTING_CASE) {
        assert!(exercised.contains(id), "{id:?} is not exercised at all");
    }
}

#[test]
fn every_required_evidence_row_has_bearing_cases() {
    let plan = guide_nine_evidence_plan().expect("the plan is a partition");
    let census = census();
    let mut bearing: BTreeMap<TargetEvidenceRequirementId, usize> = BTreeMap::new();

    for fixture in &census {
        let case = fixture.case();
        let mut requirements: BTreeSet<TargetEvidenceRequirementId> =
            requirements_for_tests(case.group(), fixture.enforcement_layer())
                .iter()
                .copied()
                .collect();
        if case.opcode().is_some() {
            requirements.insert(TargetEvidenceRequirementId::OpcodeSemantics);
        }
        for requirement in requirements {
            *bearing.entry(requirement).or_default() += 1;
        }
    }

    for (id, class) in plan.iter() {
        if class != EvidencePlanClass::Required {
            continue;
        }
        assert!(
            bearing.get(&id).copied().unwrap_or_default() > 0,
            "required evidence {id:?} has no case bearing on it",
        );
    }
}

#[test]
fn the_rows_outside_the_required_plan_stay_unattempted() {
    // A case bearing on a row the plan does not require would make the
    // report claim evidence the project deliberately did not gather.
    let plan = guide_nine_evidence_plan().expect("the plan is a partition");
    let census = census();
    for fixture in &census {
        for requirement in
            requirements_for_tests(fixture.case().group(), fixture.enforcement_layer())
        {
            assert_eq!(
                plan.class(*requirement),
                Some(EvidencePlanClass::Required),
                "{requirement:?} is borne on by a case but is not required",
            );
        }
    }
}

#[test]
fn typed_fixtures_carry_bytes_the_typed_language_reads_back() {
    let target = reviewed_target();
    for fixture in &census() {
        match fixture.script_source() {
            FixtureScriptSource::TypedProgram => {
                TapscriptProgram::decode(&target, fixture.script())
                    .expect("a typed fixture's bytes are typed instructions");
            }
            FixtureScriptSource::DeliberatelyMalformed => {
                assert!(
                    TapscriptProgram::decode(&target, fixture.script()).is_err(),
                    "a deliberately malformed fixture must be outside the typed language",
                );
            }
        }
    }
}

#[test]
fn every_fixture_is_stated_against_the_reviewed_contract() {
    let target = reviewed_target();
    let definition = target.definition();
    for fixture in &census() {
        assert_eq!(
            fixture.target_contract_version(),
            definition.version().get()
        );
        match fixture.leaf_version_status() {
            LeafVersionStatus::Reviewed => {
                assert_eq!(fixture.leaf_version(), definition.leaf_version().get());
            }
            LeafVersionStatus::Unreviewed => {
                assert_ne!(fixture.leaf_version(), definition.leaf_version().get());
            }
        }
        // A context can never disagree with the fixture it belongs to.
        if let Some(context) = fixture.context() {
            assert_eq!(context.script_path.script, fixture.script());
            assert_eq!(context.script_path.leaf_version, fixture.leaf_version());
        }
    }
}

#[test]
fn every_stated_final_depth_is_one_the_contract_can_produce() {
    // The independent check on every stated stack: the reviewed
    // contract's own success and non-aborting-failure arithmetic says
    // which depths a program can finish at, and a stated stack of any
    // other depth is a fixture defect rather than a target finding.
    let target = reviewed_target();
    for fixture in &census() {
        let Some(stated) = fixture.expected().static_final_stack() else {
            continue;
        };
        if fixture.script_source() != FixtureScriptSource::TypedProgram {
            continue;
        }
        let program = TapscriptProgram::decode(&target, fixture.script()).expect("typed bytes");
        let depths = reachable_depths(&target, &program, fixture.initial_stack().len());
        let stated = i64::try_from(stated.len()).expect("a stated depth fits");
        assert!(
            depths.contains(&stated),
            "case {} states a final depth of {stated}, which the contract cannot reach: {depths:?}",
            fixture.case(),
        );
    }
}

/// Every main-stack depth the reviewed contract says a program can
/// finish at, starting from one initial depth.
///
/// Written from the contract's success alternatives and its non-aborting
/// failure effects rather than from the abstract validator's states, so
/// that the two are independent statements of the same arithmetic.
fn reachable_depths(
    target: &ReviewedElementsTapscriptDefinition,
    program: &TapscriptProgram,
    initial: usize,
) -> BTreeSet<i64> {
    let mut depths: BTreeSet<i64> = std::iter::once(i64::try_from(initial).unwrap_or(0)).collect();

    for instruction in program.instructions() {
        let id = match instruction {
            TapscriptInstruction::Push(_) => {
                depths = depths.iter().map(|depth| depth + 1).collect();
                continue;
            }
            TapscriptInstruction::Opcode(id) => *id,
        };
        let spec = target
            .definition()
            .opcodes()
            .get(&id)
            .expect("the reviewed contract states every reviewed primitive");
        let declared = spec.stack().operands().len();

        let mut changes: BTreeSet<i64> = BTreeSet::new();
        for case in spec.stack().success().cases() {
            changes.insert(case.effect().depth_change());
        }
        if matches!(
            spec.stack().success(),
            SuccessContract::RetainsOperands { .. }
        ) {
            changes.insert(0);
        }
        for effect in spec.stack().failure().effects() {
            let consumed = i64::try_from(declared).unwrap_or(0);
            match effect.outcome() {
                // A false pushed above the operands, or in their place.
                FailureOutcome::RetainOperandsPushFalse => {
                    changes.insert(1);
                }
                FailureOutcome::ConsumeOperandsPushFalse => {
                    changes.insert(1 - consumed);
                }
                // An abort has no final depth to contribute, and
                // neither does an outcome this test has never seen.
                _ => {}
            }
        }

        depths = depths
            .iter()
            .flat_map(|depth| changes.iter().map(move |change| depth + change))
            .collect();
    }

    depths
}

#[test]
fn the_opcode_bytes_the_fixtures_carry_are_the_reviewed_ones() {
    // An independently written table, compared with what the census's
    // own programs encode to. Asking the registry for both sides would
    // be the registry agreeing with itself (Guide-9 §21.1).
    let expected: &[(OpcodeId, u8)] = &[
        (OpcodeId::CheckSequenceVerify, 0xb2),
        (OpcodeId::CheckSig, 0xac),
        (OpcodeId::CheckSigVerify, 0xad),
        (OpcodeId::CheckSigFromStack, 0xc1),
        (OpcodeId::CheckSigFromStackVerify, 0xc2),
        (OpcodeId::Sha256Initialize, 0xc4),
        (OpcodeId::Sha256Update, 0xc5),
        (OpcodeId::Sha256Finalize, 0xc6),
        (OpcodeId::InspectInputOutpoint, 0xc7),
        (OpcodeId::InspectInputAsset, 0xc8),
        (OpcodeId::InspectInputValue, 0xc9),
        (OpcodeId::InspectInputScriptPubKey, 0xca),
        (OpcodeId::InspectInputSequence, 0xcb),
        (OpcodeId::InspectInputIssuance, 0xcc),
        (OpcodeId::PushCurrentInputIndex, 0xcd),
        (OpcodeId::InspectOutputAsset, 0xce),
        (OpcodeId::InspectOutputValue, 0xcf),
        (OpcodeId::InspectOutputNonce, 0xd0),
        (OpcodeId::InspectOutputScriptPubKey, 0xd1),
        (OpcodeId::InspectVersion, 0xd2),
        (OpcodeId::InspectLockTime, 0xd3),
        (OpcodeId::InspectNumInputs, 0xd4),
        (OpcodeId::InspectNumOutputs, 0xd5),
        (OpcodeId::TxWeight, 0xd6),
        (OpcodeId::Add64, 0xd7),
        (OpcodeId::Sub64, 0xd8),
        (OpcodeId::Mul64, 0xd9),
        (OpcodeId::Div64, 0xda),
        (OpcodeId::Neg64, 0xdb),
        (OpcodeId::LessThan64, 0xdc),
        (OpcodeId::LessThanOrEqual64, 0xdd),
        (OpcodeId::GreaterThan64, 0xde),
        (OpcodeId::GreaterThanOrEqual64, 0xdf),
        (OpcodeId::ScriptNumToLe64, 0xe0),
        (OpcodeId::Le64ToScriptNum, 0xe1),
        (OpcodeId::Le32ToLe64, 0xe2),
        (OpcodeId::EcMulScalarVerify, 0xe3),
        (OpcodeId::TweakVerify, 0xe4),
        (OpcodeId::Verify, 0x69),
        (OpcodeId::DropTwo, 0x6d),
        (OpcodeId::DuplicateTwo, 0x6e),
        (OpcodeId::Drop, 0x75),
        (OpcodeId::Duplicate, 0x76),
        (OpcodeId::RemoveSecond, 0x77),
        (OpcodeId::CopyOver, 0x78),
        (OpcodeId::Rotate, 0x7b),
        (OpcodeId::Swap, 0x7c),
        (OpcodeId::Tuck, 0x7d),
        (OpcodeId::Concatenate, 0x7e),
        (OpcodeId::Substring, 0x7f),
        (OpcodeId::Size, 0x82),
        (OpcodeId::BitwiseAnd, 0x84),
        (OpcodeId::BitwiseXor, 0x86),
        (OpcodeId::Equal, 0x87),
        (OpcodeId::EqualVerify, 0x88),
    ];
    assert_eq!(expected.len(), OpcodeId::ALL.len());

    let target = reviewed_target();
    for (id, byte) in expected {
        let program = TapscriptProgram::new(vec![TapscriptInstruction::Opcode(*id)])
            .expect("one instruction is within the work limit");
        assert_eq!(
            program.encode(&target),
            vec![*byte],
            "{id:?} does not encode to its reviewed byte",
        );
    }
}

#[test]
fn every_group_the_census_uses_is_one_the_plan_requires() {
    let census = census();
    let used: BTreeSet<NativeCaseGroup> = census
        .iter()
        .map(|fixture| fixture.case().group())
        .collect();
    // The two groups the plan does not require are deliberately unused:
    // one needs sighash evidence the project did not attempt, the other
    // needs whole-transaction conservation evidence.
    assert!(!used.contains(&NativeCaseGroup::Sighash));
    assert!(!used.contains(&NativeCaseGroup::ConfidentialValue));
    for group in [
        NativeCaseGroup::ExecutionDomain,
        NativeCaseGroup::LeafVersion,
        NativeCaseGroup::InstructionEncoding,
        NativeCaseGroup::PushEncoding,
        NativeCaseGroup::InputIntrospection,
        NativeCaseGroup::OutputIntrospection,
        NativeCaseGroup::TransactionIntrospection,
        NativeCaseGroup::Arithmetic,
        NativeCaseGroup::Comparison,
        NativeCaseGroup::Conversion,
        NativeCaseGroup::StreamingHash,
        NativeCaseGroup::EllipticCurve,
        NativeCaseGroup::Signature,
        NativeCaseGroup::RelativeTimelock,
        NativeCaseGroup::Issuance,
        NativeCaseGroup::Resource,
    ] {
        assert!(
            used.contains(&group),
            "the {} group has no case",
            group.wire_name(),
        );
    }
}

#[test]
fn the_reviewed_domain_is_the_only_one_the_census_states() {
    let target = reviewed_target();
    assert_eq!(
        target.definition().execution_domain(),
        ExecutionDomain::Tapscript,
    );
}
