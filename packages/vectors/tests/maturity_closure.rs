//! The maturity link, closed over its linked bytes.
//!
//! Every test here reads the decoded announcement leaf and the plan's
//! and record's own statements. None reads a linker field as the fact
//! under test, which is what makes this evidence about the program a
//! spend would run rather than evidence that the linker agrees with
//! itself.
//!
//! The native half is one ignored test. It submits the four retained
//! shapes to a real target and observes where that target's interpreter
//! refuses them. Run it with the adapter and the deployment identities
//! set in the invoking shell:
//! ```sh
//! TRIPOD_LIVE_EXECUTOR=/path/to/adapter TRIPOD_LIVE_NETWORK_ID=NETWORK_HEX \
//! TRIPOD_LIVE_GENESIS_ID=GENESIS_HEX \
//! cargo test -p tripod-vectors --test maturity_closure -- --ignored --test-threads=1
//! ```
//! Network and genesis are 64 lower-case hex digits. No capture is
//! written, so no suite provenance is read; `TRIPOD_LIVE_REPORT_DIR`
//! is honoured where it is set, as the directory the adapter's own
//! diagnostics are written under.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use std::time::Duration;

use linker::{
    CandidateDeploymentIdentity, LinkRefusal, StateDischargeClass, StateLinkRefusal,
    StateLinkedCarrier,
};
use tapscript::{
    StackItem, StateAnnouncementSymbol, StateConstructorRefusal, StateCurveCapability,
    StateLeafRole, StateOperatorSymbol, StateProgramComponent, StateProgramSymbol,
    StateTweakOutcome, TapscriptInstruction, TapscriptProgram,
};
use target_elements::{
    ActivationDeclaration, DeploymentEnvironment, DevelopmentDeploymentBinding, LeafVersion,
    OpcodeId, PayloadWidth, ReviewedElementsTapscriptDefinition, StackValueType,
    TAPSCRIPT_STACK_ITEM_RELAY_LIMIT, reviewed_elements_tapscript,
    validate_reviewed_development_binding,
};
use target_elements_conformance::executor::{
    ExecutorConfiguration, ExecutorDiagnostics, ExecutorTrust, NativeOperationCapture,
    OperationStep, PlanRefused, TargetOperationPlanner, execute_operations_captured,
};
use target_elements_conformance::protocol::{
    NativeOperationResponse, ObservedOutcomeLayer, OperationCaseId, OperationSubject,
    TargetFundingSubject, TargetSponsorFundingSubject, TargetSponsorSigningSubject,
    TargetSubmissionSubject, WireOutpoint, WireSighashProfile,
};
use target_elements_conformance::test_material::PublicTestSignerHandle;
use transaction::bytes::{
    AssetField, AssetId as TargetAssetId, InputWitness, Outpoint, TargetInput, TargetOutput,
    TargetTransaction, Txid, ValueField,
};
use vectors::maturity_closure::{
    AdoptionCase, AdoptionVector, DecodedAnnouncementLeaf, GoldenFigure, KeptCheck, LocatedRow,
    MaturityClosureRefusal, MaturityDeployment, MaturityDeploymentParameters,
    MaturityWitnessSelection, ObservedField, OracleStateCurve, Verdict, adoption_transaction,
    closure_target, decode_announcement_leaf, decoded_deployment, forbidden_program_literals,
    internal_key_from_bytes, kept_check_site, leaf_literals, linked_announcement_bytes,
    linked_maturity_bundle, locate_discharges, maturity_sources, maturity_sources_with,
    moved_sites, recompute_golden, recovered_values, removed_and_kept_checks,
};

#[test]
fn whole_metadata_emission_refuses_at_sources_and_linking() {
    let schedule = tapscript::StateWitnessSchedule::WholeMetadata;
    let selection = MaturityWitnessSelection::Emission(schedule);
    assert!(selection.is_emission());
    assert_eq!(selection.schedule(), schedule);
    for deployment in MaturityDeployment::ALL {
        assert_eq!(
            maturity_sources(deployment, selection).err(),
            Some(MaturityClosureRefusal::ReplayOnlySchedule { schedule })
        );
        assert_eq!(
            linked_maturity_bundle(deployment, selection).err(),
            Some(MaturityClosureRefusal::ReplayOnlySchedule { schedule })
        );
    }
}

#[test]
fn variable_metadata_selections_link_distinct_leaves_and_resolve_the_header() {
    let schedule = tapscript::StateWitnessSchedule::VariableMetadata;
    for selection in [
        MaturityWitnessSelection::Retained(schedule),
        MaturityWitnessSelection::Emission(schedule),
    ] {
        assert_eq!(selection.schedule(), schedule);
        for deployment in MaturityDeployment::ALL {
            let sources =
                maturity_sources(deployment, selection).expect("variable sources compose");
            assert_eq!(sources.record().schedule(), schedule);
            let direct =
                maturity_sources_with(deployment.parameters().expect("parameters"), selection)
                    .expect("supplied variable sources compose");
            assert_eq!(direct.record(), sources.record());
            let bundle =
                linked_maturity_bundle(deployment, selection).expect("variable link resolves");
            let (decoded_bundle, leaf) =
                decoded_deployment(deployment, selection).expect("variable leaf decodes");
            assert_eq!(decoded_bundle, bundle);
            assert_eq!(bundle.resolved().entries().len(), 14);
            assert_eq!(bundle.resolved().program_keys().len(), 7);
            assert_eq!(bundle.resolved().push_site_count(), 16);
            let (whole, whole_leaf) = linked(deployment);
            assert_ne!(leaf.bytes(), whole_leaf.bytes());
            assert_ne!(
                bundle.static_subtree().root(),
                whole.static_subtree().root()
            );
            assert_ne!(
                bundle.instances()[0].constructor().output_key(),
                whole.instances()[0].constructor().output_key()
            );
        }
    }
}

#[test]
fn retained_whole_metadata_sources_preserve_the_linked_announcement_bytes() {
    let selection = MaturityWitnessSelection::retained_whole_metadata();
    assert!(!selection.is_emission());
    assert_eq!(
        selection,
        MaturityWitnessSelection::Retained(tapscript::StateWitnessSchedule::WholeMetadata)
    );
    for deployment in MaturityDeployment::ALL {
        let sources = maturity_sources(deployment, selection).expect("retained sources");
        assert_eq!(sources.record().schedule(), selection.schedule());
        let bundle = sources.link(&OracleStateCurve).expect("retained link");
        let (_, leaf) = linked(deployment);
        assert_eq!(
            linked_announcement_bytes(&bundle, &target()).expect("announcement"),
            leaf.bytes()
        );
    }
}

/// The golden roots this lane publishes, recomputed and carried alike.
///
/// The demonstration deployment's static root, outer root and output
/// key, then the second deployment's three, then the third's.
///
/// A pinned constant is what makes a golden a golden: it is checked
/// against the recomputation AND against what the bundle carries in one
/// comparison, so a pin that drifted from either fails here rather than
/// quietly tracking whichever side moved.
const GOLDEN: [&str; 9] = [
    "cf54a9f68d066ca669c447045d01dfc15d2c6c1c33a13a2a11e8d1a52a50eb92",
    "3ddf7410676048756dc1f5750474eafc14f7c2b9e7d1c846da7f29d4cbcd72bf",
    "fa127d5e1bf8f124f66ac48403378ab56a61d95a00ef973566cc01c270c9c2e1",
    "06051be38b1f852856cb4280489fb4246a60d00507a88e8324e873e50819ca09",
    "12b5b81e34fdd1e69429efe53ef44be8be22a1dc2432f3f7097cfd72f18723d6",
    "71f65e4be1eae5842deca56de6c09ca452638ccf2fc02f880bb10057c6c53fa7",
    "b46e22832bc98707d8ac73408b13f8c4a689aea73a0ce593c9ecad104a43cd42",
    "d3b0e63ccb5b636ccede25c0d0942f0e9c969df1e5af887bbed1265855dcbd7f",
    "c96dd597d1e3f3cf3bca218b32e4f9ba4392218d50578e71f26b4b765ef822e9",
];

type Linked = (
    linker::CandidateLinkedMaturityBundle,
    DecodedAnnouncementLeaf,
);

/// One deployment's linked bundle and decoded leaf, linked once.
fn linked(deployment: MaturityDeployment) -> Linked {
    static DEMONSTRATION: LazyLock<Linked> = LazyLock::new(|| {
        decoded_deployment(
            MaturityDeployment::Demonstration,
            vectors::maturity_closure::MaturityWitnessSelection::retained_whole_metadata(),
        )
        .expect("the demonstration deployment links")
    });
    static SECOND: LazyLock<Linked> = LazyLock::new(|| {
        decoded_deployment(
            MaturityDeployment::Second,
            vectors::maturity_closure::MaturityWitnessSelection::retained_whole_metadata(),
        )
        .expect("the second links")
    });
    static PUBLISHED_SIGNER_HELD: LazyLock<Linked> = LazyLock::new(|| {
        decoded_deployment(
            MaturityDeployment::PublishedSignerHeld,
            vectors::maturity_closure::MaturityWitnessSelection::retained_whole_metadata(),
        )
        .expect("the published signer's deployment links")
    });
    match deployment {
        MaturityDeployment::Demonstration => DEMONSTRATION.clone(),
        MaturityDeployment::Second => SECOND.clone(),
        MaturityDeployment::PublishedSignerHeld => PUBLISHED_SIGNER_HELD.clone(),
    }
}

fn target() -> ReviewedElementsTapscriptDefinition {
    closure_target().expect("the reviewed contract validates")
}

fn hex(bytes: &[u8]) -> String {
    let mut text = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(text, "{byte:02x}");
    }
    text
}

/// The decoded leaf with one instruction removed, rebuilt from the
/// instructions and re-encoded, so the census reads bytes either way.
fn without(
    leaf: &DecodedAnnouncementLeaf,
    target: &ReviewedElementsTapscriptDefinition,
    removed: &[usize],
) -> DecodedAnnouncementLeaf {
    let mut instructions = leaf.program().instructions().to_vec();
    for index in removed.iter().rev() {
        instructions.remove(*index);
    }
    let program = TapscriptProgram::new(instructions).expect("the shortened recipe is a program");
    decode_announcement_leaf(target, &program.encode(target)).expect("the shortened bytes decode")
}

/// The decoded leaf with one instruction inserted at its head.
fn with_inserted(
    leaf: &DecodedAnnouncementLeaf,
    target: &ReviewedElementsTapscriptDefinition,
    instruction: TapscriptInstruction,
) -> DecodedAnnouncementLeaf {
    let mut instructions = leaf.program().instructions().to_vec();
    instructions.insert(0, instruction);
    let program = TapscriptProgram::new(instructions).expect("the widened recipe is a program");
    decode_announcement_leaf(target, &program.encode(target)).expect("the widened bytes decode")
}

/// The decoded leaf with one instruction replaced by another.
fn replacing(
    leaf: &DecodedAnnouncementLeaf,
    target: &ReviewedElementsTapscriptDefinition,
    index: usize,
    instruction: TapscriptInstruction,
) -> DecodedAnnouncementLeaf {
    let mut instructions = leaf.program().instructions().to_vec();
    instructions[index] = instruction;
    let program = TapscriptProgram::new(instructions).expect("the altered recipe is a program");
    decode_announcement_leaf(target, &program.encode(target)).expect("the altered bytes decode")
}

/// Which check the census names absent when one check's instruction is
/// removed.
///
/// The two tweak relations are the same two instructions at two sites,
/// so removing either leaves one and the census reports the second as
/// missing. Removing both is what names the first, and that case is
/// covered separately.
const fn expected_absent(check: KeptCheck) -> KeptCheck {
    match check {
        KeptCheck::PredecessorTweakVerify => KeptCheck::SuccessorTweakVerify,
        other => other,
    }
}

// --- (a) Both deployments link, and their bytes are a program ----------

#[test]
fn both_deployments_link_and_their_linked_bytes_decode_to_the_records_shape() {
    for deployment in MaturityDeployment::ALL {
        let (bundle, leaf) = linked(deployment);
        assert_eq!(
            leaf.program().len(),
            bundle.record().program().len(),
            "{} linked to a different shape",
            deployment.name()
        );
        assert_eq!(
            leaf.program().encode(&target()),
            leaf.bytes(),
            "{} does not re-encode",
            deployment.name()
        );
        // The bytes are the leaf's whole encoding, measured by the
        // encoder rather than by counting them here.
        assert_eq!(
            u64::try_from(leaf.bytes().len()),
            Ok(leaf.program().encoded_length(&target()))
        );
    }
}

// --- (b) The row's own gate, in both directions ------------------------

#[test]
fn the_reduced_leaf_carries_every_kept_check_and_no_removed_primitive() {
    let reviewed = target();
    for deployment in MaturityDeployment::ALL {
        let (bundle, leaf) = linked(deployment);
        let forbidden =
            forbidden_program_literals(&bundle, &reviewed).expect("the forbidden shapes derive");
        let census = removed_and_kept_checks(&leaf, &forbidden, &reviewed)
            .expect("the reduced leaf passes its own census");

        assert_eq!(
            census.kept(),
            &KeptCheck::ALL.into_iter().collect::<BTreeSet<_>>()
        );
        assert_eq!(census.positions(), &BTreeSet::from([0]));
        assert_eq!(census.absent().len(), 3);
        assert_eq!(census.forbidden_literals(), 4);
        assert_eq!(census.introspections(), 7);
        assert_eq!(forbidden.widths(), vec![34, 32, 90, 86]);
    }
}

#[test]
fn a_kept_check_that_goes_missing_is_refused_by_name() {
    let reviewed = target();
    let (bundle, leaf) = linked(MaturityDeployment::Demonstration);
    let forbidden =
        forbidden_program_literals(&bundle, &reviewed).expect("the forbidden shapes derive");

    for check in KeptCheck::ALL {
        let site = kept_check_site(&leaf, &reviewed, check).expect("the check is in the bytes");
        let shortened = without(&leaf, &reviewed, &[site + check.breaking_offset()]);
        assert_eq!(
            removed_and_kept_checks(&shortened, &forbidden, &reviewed),
            Err(MaturityClosureRefusal::KeptCheckAbsent {
                check: expected_absent(check)
            }),
            "removing {} was not refused by name",
            check.name()
        );
    }

    let first = kept_check_site(&leaf, &reviewed, KeptCheck::PredecessorTweakVerify)
        .expect("the predecessor relation is in the bytes");
    let second = kept_check_site(&leaf, &reviewed, KeptCheck::SuccessorTweakVerify)
        .expect("the successor relation is in the bytes");
    let neither = without(&leaf, &reviewed, &[first + 1, second + 1]);
    assert_eq!(
        removed_and_kept_checks(&neither, &forbidden, &reviewed),
        Err(MaturityClosureRefusal::KeptCheckAbsent {
            check: KeptCheck::PredecessorTweakVerify
        })
    );
}

#[test]
fn a_removed_primitive_that_returns_is_refused_by_name() {
    let reviewed = target();
    let (bundle, leaf) = linked(MaturityDeployment::Demonstration);
    let forbidden =
        forbidden_program_literals(&bundle, &reviewed).expect("the forbidden shapes derive");

    for opcode in [
        OpcodeId::InspectNumInputs,
        OpcodeId::InspectNumOutputs,
        OpcodeId::InspectInputIssuance,
    ] {
        let widened = with_inserted(&leaf, &reviewed, TapscriptInstruction::Opcode(opcode));
        assert_eq!(
            removed_and_kept_checks(&widened, &forbidden, &reviewed),
            Err(MaturityClosureRefusal::RemovedPrimitivePresent {
                opcode,
                instruction: 0
            })
        );
    }
}

#[test]
fn an_introspection_that_names_no_literal_zero_is_refused() {
    let reviewed = target();
    let (bundle, leaf) = linked(MaturityDeployment::Demonstration);
    let forbidden =
        forbidden_program_literals(&bundle, &reviewed).expect("the forbidden shapes derive");

    // Remove the position literal the first introspection names. Every
    // other position is free precisely because this literal is there, so
    // a leaf without it is refused before any kept check is looked for.
    let site = kept_check_site(&leaf, &reviewed, KeptCheck::InputZeroAsset)
        .expect("the asset recognition is in the bytes");
    let unnamed = without(&leaf, &reviewed, &[site]);
    assert_eq!(
        removed_and_kept_checks(&unnamed, &forbidden, &reviewed),
        Err(MaturityClosureRefusal::IntrospectionWithoutLiteralZero { instruction: site })
    );
}

#[test]
fn a_predecessor_program_literal_that_returns_is_refused() {
    let reviewed = target();
    let (bundle, leaf) = linked(MaturityDeployment::Demonstration);
    let forbidden =
        forbidden_program_literals(&bundle, &reviewed).expect("the forbidden shapes derive");

    for (shape, width) in forbidden.shapes().iter().zip(forbidden.widths()) {
        let item = StackItem::new(&reviewed, shape.clone()).expect("the shape is a stack item");
        let widened = with_inserted(&leaf, &reviewed, TapscriptInstruction::Push(item));
        assert_eq!(
            removed_and_kept_checks(&widened, &forbidden, &reviewed),
            Err(MaturityClosureRefusal::ProgramLiteralPresent {
                instruction: 0,
                width
            })
        );
    }
}

#[test]
fn a_consumer_site_the_bytes_do_not_push_is_refused() {
    let reviewed = target();
    let (bundle, leaf) = linked(MaturityDeployment::Demonstration);

    let asset = kept_check_site(&leaf, &reviewed, KeptCheck::InputZeroAsset)
        .expect("the asset recognition is in the bytes")
        + 4;
    let silent = replacing(
        &leaf,
        &reviewed,
        asset,
        TapscriptInstruction::Opcode(OpcodeId::Drop),
    );
    assert_eq!(
        recovered_values(&silent, bundle.record()),
        Err(MaturityClosureRefusal::ConsumerSiteIsNotAPush { instruction: asset })
    );

    let key = kept_check_site(&leaf, &reviewed, KeptCheck::PredecessorTweakVerify)
        .expect("the predecessor relation is in the bytes");
    let other = StackItem::new(&reviewed, vec![0x5a; 32]).expect("a distinguishable key");
    let disagreeing = replacing(&leaf, &reviewed, key, TapscriptInstruction::Push(other));
    assert_eq!(
        recovered_values(&disagreeing, bundle.record()),
        Err(MaturityClosureRefusal::ConsumerSitesDisagree {
            symbol: StateProgramSymbol::Semantic(StateAnnouncementSymbol::InternalKey)
        })
    );
}

// --- (c) The discharge table, located ----------------------------------

#[test]
fn every_emitted_discharge_is_found_where_the_closure_claims_it() {
    let reviewed = target();
    for deployment in MaturityDeployment::ALL {
        let (bundle, leaf) = linked(deployment);
        let closure = bundle.carrier_closure();
        let located = locate_discharges(&leaf, closure, bundle.record(), &reviewed)
            .expect("every claimed discharge is located");

        // The located census is checked against the closure's own, and
        // the closure's own against the figure the wave publishes. The
        // located side writes nothing down.
        assert_eq!(located.census(), closure.census());
        assert_eq!(
            closure.census(),
            &BTreeMap::from([
                (StateDischargeClass::Emitted, 13),
                (StateDischargeClass::Deployment, 1),
                (StateDischargeClass::ModelScope, 10),
                (StateDischargeClass::External, 2),
            ])
        );
        assert_eq!(located.census().values().sum::<usize>(), 26);

        let components: BTreeSet<StateProgramComponent> = located
            .rows()
            .iter()
            .filter_map(LocatedRow::component)
            .collect();
        assert!(components.len() <= located.census()[&StateDischargeClass::Emitted]);
        assert!(!components.is_empty());
        for component in &components {
            assert!(bundle.record().components().contains_key(component));
        }
    }
}

#[test]
fn the_rows_that_carry_no_bytes_locate_none() {
    let reviewed = target();
    let (bundle, leaf) = linked(MaturityDeployment::Demonstration);
    let closure = bundle.carrier_closure();
    let located = locate_discharges(&leaf, closure, bundle.record(), &reviewed).expect("located");

    let model_scope: BTreeSet<_> = located
        .rows()
        .iter()
        .filter(|row| row.class() == StateDischargeClass::ModelScope)
        .map(|row| {
            assert_eq!(row.located(), None);
            assert_eq!(row.in_script(), None);
            row.relation().clone()
        })
        .collect();
    assert_eq!(
        model_scope.len(),
        located.census()[&StateDischargeClass::ModelScope]
    );

    for row in closure.rows() {
        if let StateLinkedCarrier::DeploymentFact(role) = row.linked() {
            assert!(bundle.deployment().deployment_facts().contains(role));
        }
    }

    let with_in_script = located
        .rows()
        .iter()
        .filter(|row| row.class() == StateDischargeClass::External)
        .filter(|row| row.in_script().is_some())
        .count();
    let external = located
        .rows()
        .iter()
        .filter(|row| row.class() == StateDischargeClass::External)
        .count();
    assert!(with_in_script > 0 && with_in_script < external);
}

// --- (d) The golden, recomputed ----------------------------------------

#[test]
fn the_golden_roots_are_pinned_recomputed_and_carried_alike() {
    let reviewed = target();
    let mut recomputed = Vec::new();
    let mut carried = Vec::new();
    for deployment in MaturityDeployment::ALL {
        let (bundle, leaf) = linked(deployment);
        let evidence = recompute_golden(&leaf, &bundle, &reviewed, &OracleStateCurve)
            .expect("the golden recomputes and agrees with the bundle");
        let constructor = bundle
            .instances()
            .first()
            .expect("the link retains its application")
            .constructor();
        recomputed.extend([
            hex(evidence.static_root()),
            hex(evidence.merkle_root()),
            hex(evidence.output_key()),
        ]);
        carried.extend([
            hex(bundle.taptree().static_root()),
            hex(bundle.taptree().merkle_root()),
            hex(constructor.output_key()),
        ]);
    }
    let pinned: Vec<String> = GOLDEN.iter().map(|figure| (*figure).to_owned()).collect();
    assert_eq!((&recomputed, &carried), (&pinned, &pinned));
}

#[test]
fn a_second_deployment_moves_only_the_sites_whose_values_moved() {
    let reviewed = target();
    let (first_bundle, first) = linked(MaturityDeployment::Demonstration);
    let (second_bundle, second) = linked(MaturityDeployment::Second);

    let moved = moved_sites(&first, &second, &reviewed).expect("the two links are comparable");
    let census = first_bundle
        .relocations()
        .expect("the announcement leaf carries a relocation census")
        .len();
    assert_eq!(moved.count(), 9);
    assert_eq!(census, 15);
    assert_eq!(census - moved.count(), 6);

    let sites = first_bundle
        .relocations()
        .expect("the census is carried")
        .sites();
    for site in moved.sites() {
        assert!(sites.contains(site));
    }

    assert_ne!(
        first_bundle.taptree().static_root(),
        second_bundle.taptree().static_root()
    );
    assert_ne!(
        first_bundle.taptree().merkle_root(),
        second_bundle.taptree().merkle_root()
    );
    assert_ne!(
        first_bundle
            .instances()
            .first()
            .expect("an application")
            .constructor()
            .output_key(),
        second_bundle
            .instances()
            .first()
            .expect("an application")
            .constructor()
            .output_key()
    );
}

#[test]
fn the_third_deployment_moves_the_same_sites_and_the_operator_key_is_one_of_them() {
    let reviewed = target();
    let (first_bundle, first) = linked(MaturityDeployment::Demonstration);
    let (_, second) = linked(MaturityDeployment::Second);
    let (third_bundle, third) = linked(MaturityDeployment::PublishedSignerHeld);

    let moved = moved_sites(&first, &third, &reviewed).expect("the two links are comparable");
    let second_moved =
        moved_sites(&first, &second, &reviewed).expect("the two links are comparable");
    assert_eq!(moved.count(), 9);
    // A deployment whose committed key is a signer's rather than a fill
    // moves the sites another fill moves, so what the third establishes
    // is about the value at those sites rather than about how far a
    // substitution reaches.
    assert_eq!(moved.sites(), second_moved.sites());

    // The operator key's site, taken from the bytes at the check that
    // consumes it rather than from the relocation census.
    let operator = kept_check_site(&first, &reviewed, KeptCheck::OperatorAuthorization)
        .expect("the operator authorization is in the bytes");
    assert!(moved.sites().contains(&operator));

    assert_ne!(
        first_bundle.taptree().merkle_root(),
        third_bundle.taptree().merkle_root()
    );
}

#[test]
fn the_third_deployments_committed_operator_key_is_the_published_signers_own() {
    let reviewed = target();
    let (bundle, leaf) = linked(MaturityDeployment::PublishedSignerHeld);
    let published = PublicTestSignerHandle::Third
        .x_only_public_key()
        .expect("the published signer resolves to a key");

    let values = recovered_values(&leaf, bundle.record()).expect("the linked values are recovered");
    let committed = values
        .get(&StateProgramSymbol::Operator(
            StateOperatorSymbol::CommittedOperatorKey,
        ))
        .expect("the record names a committed operator key");
    assert_eq!(committed.bytes(), published.as_slice());

    // The operand the verification itself consumes is that same key,
    // read at the site the leaf runs it at rather than at a site a
    // census named.
    let site = kept_check_site(&leaf, &reviewed, KeptCheck::OperatorAuthorization)
        .expect("the operator authorization is in the bytes");
    let TapscriptInstruction::Push(operand) = &leaf.program().instructions()[site] else {
        panic!("the verification's operand is a push");
    };
    assert_eq!(operand.bytes(), published.as_slice());

    // The demonstration commits a fill of the same width there, so the
    // substitution moved the value and not the shape.
    let (_, demonstration) = linked(MaturityDeployment::Demonstration);
    let fill_site = kept_check_site(&demonstration, &reviewed, KeptCheck::OperatorAuthorization)
        .expect("the operator authorization is in the bytes");
    let TapscriptInstruction::Push(fill) = &demonstration.program().instructions()[fill_site]
    else {
        panic!("the verification's operand is a push");
    };
    assert_eq!(fill.bytes().len(), published.len());
    assert_ne!(fill.bytes(), published.as_slice());
}

// --- (e) The adoption gate ---------------------------------------------

fn expected_verdicts() -> BTreeMap<AdoptionCase, Verdict> {
    BTreeMap::from([
        (
            AdoptionCase::AcceptedWithForeignInputsAndOutputs,
            Verdict::Accepted,
        ),
        (
            AdoptionCase::RefusedSingletonAtInputOne,
            Verdict::Refused {
                fields: vec![
                    ObservedField::ExecutingInputIndex,
                    ObservedField::InputZeroAsset,
                    ObservedField::InputZeroAmount,
                ],
            },
        ),
        (
            AdoptionCase::RefusedSingletonAtOutputOne,
            Verdict::Refused {
                fields: vec![
                    ObservedField::OutputZeroAsset,
                    ObservedField::OutputZeroAmount,
                ],
            },
        ),
        (
            AdoptionCase::RefusedShortOutputZero,
            Verdict::Refused {
                fields: vec![ObservedField::OutputZeroAmount],
            },
        ),
    ])
}

#[test]
fn the_four_adoption_vectors_are_decided_by_the_leafs_own_bytes() {
    let reviewed = target();
    let expected = expected_verdicts();
    // Over every deployment, because the vectors are built from the
    // literals each leaf carries: a deployment whose asset, key and lead
    // window are other values decides the same four cases the same way,
    // or the verdict was reading something other than those bytes.
    for deployment in MaturityDeployment::ALL {
        let (bundle, leaf) = linked(deployment);
        let literals = leaf_literals(&leaf, &reviewed).expect("the leaf states its own literals");
        assert_eq!(literals.pinned_input_index(), 0);
        assert_eq!(literals.input_script_version(), 1);
        assert_eq!(literals.output_script_version(), 1);
        assert_eq!(literals.input_asset(), literals.output_asset());
        assert_eq!(literals.input_amount(), literals.output_amount());

        for case in AdoptionCase::ALL {
            let vector =
                adoption_transaction(case, &bundle, &reviewed).expect("the vector is buildable");
            assert_eq!(
                vector.expected(),
                &expected[&case],
                "{} over {}",
                case.name(),
                deployment.name()
            );
            assert_eq!(
                TargetTransaction::decode(vector.bytes()).as_ref(),
                Ok(vector.transaction())
            );
            assert_eq!(vector.spent().len(), vector.transaction().inputs().len());
            assert_eq!(vector.subject().case().step, case.name());
        }
    }
}

#[test]
fn the_accepted_vector_carries_foreign_positions_the_leaf_never_names() {
    let reviewed = target();
    let (bundle, _) = linked(MaturityDeployment::Demonstration);
    let vector = adoption_transaction(
        AdoptionCase::AcceptedWithForeignInputsAndOutputs,
        &bundle,
        &reviewed,
    )
    .expect("the accepted vector is buildable");

    assert_eq!(vector.transaction().inputs().len(), 3);
    assert_eq!(vector.transaction().outputs().len(), 4);
    assert_eq!(vector.observed().executing_input_index(), 0);
    // A fee output that is not last, which the reduction made free.
    assert!(vector.transaction().outputs()[2].is_fee());
    assert!(!vector.transaction().outputs()[3].is_fee());
    assert_eq!(vector.expected(), &Verdict::Accepted);
}

#[test]
fn every_retained_subject_round_trips_through_the_protocol_serializer() {
    let reviewed = target();
    let (bundle, _) = linked(MaturityDeployment::Demonstration);
    let mut seen = BTreeSet::new();
    for case in AdoptionCase::ALL {
        let vector =
            adoption_transaction(case, &bundle, &reviewed).expect("the vector is buildable");
        let text = serde_json::to_string(vector.subject().subject()).expect("the subject writes");
        let decoded: OperationSubject = serde_json::from_str(&text).expect("the subject reads");
        assert_eq!(&decoded, vector.subject().subject());
        assert_eq!(&OperationStep::new(case.name(), decoded), vector.subject());
        assert!(seen.insert(vector.bytes().to_vec()));
    }
    assert_eq!(seen.len(), 4);
}

// --- (f) The outstanding contract, read --------------------------------

#[test]
fn the_outstanding_contract_is_read_from_the_bundle() {
    let (bundle, _) = linked(MaturityDeployment::Demonstration);
    let obligations = bundle.abi_obligations();
    assert_ne!(obligations, []);

    let relations: BTreeSet<_> = bundle
        .carrier_closure()
        .rows()
        .iter()
        .map(|row| row.relation().clone())
        .collect();
    for obligation in obligations {
        assert!(relations.contains(obligation.relation()));
    }

    let outstanding = bundle.obligations();
    assert_eq!(outstanding.obligations().count(), outstanding.count().get());
    assert!(!bundle.evidence().is_empty());
}

// --- (g) The refusal root, reached by name ------------------------------

/// A curve capability that finds no point at all.
///
/// It answers the same way for every key, so it makes no claim about
/// which key it was asked about and needs none. What the two tests using
/// it read is the refusal a caller's own capability produces, rather than
/// an expectation scripted for one input.
struct CurveWithoutPoints;

impl StateCurveCapability for CurveWithoutPoints {
    fn internal_key_is_a_point(&self, _x_only: &[u8; 32]) -> bool {
        false
    }

    fn output_key(&self, _internal_key: &[u8; 32], _merkle_root: &[u8; 32]) -> StateTweakOutcome {
        StateTweakOutcome::InternalKeyNotAPoint
    }
}

/// The leaf with every internal-key site pushing a literal of another
/// width.
///
/// All four sites move together, because one site left alone would be
/// refused as two sites of one consumer disagreeing before any width was
/// read.
fn internal_key_of_another_width(
    pair: &Linked,
    reviewed: &ReviewedElementsTapscriptDefinition,
) -> DecodedAnnouncementLeaf {
    let (bundle, leaf) = pair;
    let narrower = StackItem::new(reviewed, vec![0x5a; 31]).expect("a thirty-one-byte literal");
    let consumer = bundle
        .record()
        .consumers()
        .get(&StateProgramSymbol::Semantic(
            StateAnnouncementSymbol::InternalKey,
        ))
        .expect("the record names the internal key's own sites");
    let mut mutated = leaf.clone();
    for &site in &consumer.sites {
        mutated = replacing(
            &mutated,
            reviewed,
            site,
            TapscriptInstruction::Push(narrower.clone()),
        );
    }
    mutated
}

#[test]
fn a_zero_lead_minimum_leaves_the_sources_unavailable() {
    let supplied = MaturityDeployment::Demonstration
        .parameters()
        .expect("the demonstration's own values resolve");
    // Only the window moves. A zero minimum would admit an announcement
    // for the current cycle, which is not a lead at all, and the window
    // is the one source built from a value a caller supplies.
    let zeroed = MaturityDeploymentParameters::new(
        *supplied.singleton(),
        *supplied.operator_key(),
        (0, supplied.lead().1),
    );
    assert_eq!(
        maturity_sources_with(
            zeroed,
            vectors::maturity_closure::MaturityWitnessSelection::retained_whole_metadata()
        )
        .err(),
        Some(MaturityClosureRefusal::SourcesUnavailable)
    );
}

#[test]
fn a_curve_that_finds_no_point_refuses_the_link_by_the_constructors_own_name() {
    let sources = maturity_sources(
        MaturityDeployment::Demonstration,
        vectors::maturity_closure::MaturityWitnessSelection::retained_whole_metadata(),
    )
    .expect("the sources bind once");
    // The first thing a constructor application asks a curve is whether
    // the internal key is a point, so a capability that finds none is
    // refused there and the link carries that refusal whole rather than
    // flattening it.
    assert_eq!(
        sources.link(&CurveWithoutPoints).err(),
        Some(MaturityClosureRefusal::LinkRefused(LinkRefusal::StateLink(
            StateLinkRefusal::ConstructorApplication(StateConstructorRefusal::InternalKeyNotAPoint)
        )))
    );
}

#[test]
fn bytes_that_end_inside_a_push_do_not_decode() {
    let reviewed = target();
    let (_, leaf) = linked(MaturityDeployment::Demonstration);
    let item = StackItem::new(&reviewed, vec![0x5a; 32]).expect("a thirty-two-byte literal");
    let push = TapscriptProgram::new(vec![TapscriptInstruction::Push(item)])
        .expect("one push is a program")
        .encode(&reviewed);
    // The leaf's own bytes, then a push whose header states a width the
    // payload after it is one byte short of. The header is the encoder's
    // rather than a byte written down here.
    let mut truncated = leaf.bytes().to_vec();
    truncated.extend_from_slice(&push[..push.len() - 1]);
    assert_eq!(
        decode_announcement_leaf(&reviewed, &truncated).err(),
        Some(MaturityClosureRefusal::LeafBytesDoNotDecode)
    );
}

#[test]
fn an_internal_key_of_another_width_is_not_one_key() {
    let reviewed = target();
    let pair = linked(MaturityDeployment::Demonstration);
    let narrowed = internal_key_of_another_width(&pair, &reviewed);
    let (bundle, _) = &pair;
    // Four sites agreeing on a value that is not a key is still not a
    // key, which is what the reader refuses on rather than on the sites
    // disagreeing.
    assert_eq!(
        internal_key_from_bytes(&narrowed, bundle.record()).err(),
        Some(MaturityClosureRefusal::InternalKeySitesDisagree)
    );
}

#[test]
fn an_internal_key_of_another_width_refuses_re_emission() {
    let reviewed = target();
    let pair = linked(MaturityDeployment::Demonstration);
    let narrowed = internal_key_of_another_width(&pair, &reviewed);
    let (bundle, _) = &pair;
    // The semantic bindings admit an internal key of thirty-two bytes and
    // no other width, so re-emitting the components from the recovered
    // values is where a value of another width is refused.
    assert_eq!(
        locate_discharges(
            &narrowed,
            bundle.carrier_closure(),
            bundle.record(),
            &reviewed
        )
        .err(),
        Some(MaturityClosureRefusal::ReEmissionRefused)
    );
}

#[test]
fn a_component_whose_primitive_moved_is_not_located() {
    let reviewed = target();
    let (bundle, leaf) = linked(MaturityDeployment::Demonstration);
    let instructions = leaf.program().instructions();
    // One primitive inside a located component's own claimed range. It is
    // at no consumer site, so the recovered values and the re-emitted
    // fragments are exactly what they were and the located slice is the
    // only thing that moved.
    let (component, claimed, site) = bundle
        .carrier_closure()
        .rows()
        .iter()
        .find_map(|row| match row.linked() {
            StateLinkedCarrier::Component {
                component, range, ..
            } => range
                .clone()
                .find(|&index| {
                    matches!(
                        instructions.get(index),
                        Some(TapscriptInstruction::Opcode(_))
                    )
                })
                .map(|index| (*component, range.clone(), index)),
            _ => None,
        })
        .expect("a located component's range holds a primitive");
    // A primitive the reduction removed, which this file's own census
    // establishes the leaf does not carry, so the replacement differs
    // from whatever stood there.
    let moved = replacing(
        &leaf,
        &reviewed,
        site,
        TapscriptInstruction::Opcode(OpcodeId::InspectNumInputs),
    );
    assert_eq!(
        locate_discharges(&moved, bundle.carrier_closure(), bundle.record(), &reviewed).err(),
        Some(MaturityClosureRefusal::ComponentNotLocated { component, claimed })
    );
}

#[test]
fn a_golden_over_another_deployments_bytes_disagrees_by_name() {
    let reviewed = target();
    let (bundle, _) = linked(MaturityDeployment::Demonstration);
    let (_, elsewhere) = linked(MaturityDeployment::Second);
    // One record is composed for every deployment, so another
    // deployment's leaf recovers the same internal key and the
    // recomputation reaches the leaf hash — the first figure a
    // substitution moves.
    assert_eq!(
        recompute_golden(&elsewhere, &bundle, &reviewed, &OracleStateCurve).err(),
        Some(MaturityClosureRefusal::GoldenDisagrees {
            figure: GoldenFigure::AnnouncementLeafHash
        })
    );
}

#[test]
fn a_curve_that_determines_no_output_key_is_named_with_its_outcome() {
    let reviewed = target();
    let (bundle, leaf) = linked(MaturityDeployment::Demonstration);
    // Every hash comparison ahead of the tweak is taken over the
    // module's own arithmetic, so the curve is asked for an output key
    // only once they have all agreed, and the outcome it answers with is
    // carried rather than summarized.
    assert_eq!(
        recompute_golden(&leaf, &bundle, &reviewed, &CurveWithoutPoints).err(),
        Some(MaturityClosureRefusal::OutputKeyUndetermined(
            StateTweakOutcome::InternalKeyNotAPoint
        ))
    );
}

#[test]
fn two_leaves_of_different_shapes_are_not_comparable() {
    let reviewed = target();
    let (_, leaf) = linked(MaturityDeployment::Demonstration);
    // A site-by-site comparison of two parses of different lengths would
    // have to decide which site stands for which, which is a decision no
    // substitution licenses.
    let shortened = without(&leaf, &reviewed, &[0]);
    assert_eq!(
        moved_sites(&leaf, &shortened, &reviewed).err(),
        Some(MaturityClosureRefusal::LinkedProgramsAreNotComparable)
    );
}

// --- (h) The native half: where a real interpreter refuses -------------

/// What the target prefixes a script-execution failure with.
///
/// The adapter decides the layer by this prefix: a relay reason carrying
/// it names a script the interpreter actually ran, and any other reason
/// names a consensus or policy refusal that never reached one.
const SCRIPT_FAILURE: &str = "mandatory-script-verify-flag-failed (";

/// The check this run expects the interpreter to stop at.
const SIGNATURE_FAILURE: &str = "Schnorr signature";

/// What the target says where a revealed leaf is not the committed one.
const COMMITMENT_FAILURE: &str = "Witness program";

/// What the target says where its amount verification refuses.
const AMOUNT_FAILURE: &str = "bad-txns-in-ne-out";

/// The checks the leaf's own comparisons decide a vector by.
///
/// Every one of them sits after the committed operator key's
/// verification, which is what makes "no compared field was reached" a
/// statement about these bytes rather than about the run.
const COMPARED_CHECKS: [KeptCheck; 7] = [
    KeptCheck::SelfPositionPin,
    KeptCheck::InputZeroAsset,
    KeptCheck::InputZeroExplicitAmount,
    KeptCheck::InputZeroScriptVersion,
    KeptCheck::OutputZeroAsset,
    KeptCheck::OutputZeroExplicitAmount,
    KeptCheck::OutputZeroScriptVersion,
];

/// One funded coin an input of one case spends.
#[derive(Clone, Copy, Debug)]
enum CoinSlot {
    /// The coin of the issued singleton asset funded for this case.
    Singleton(usize),
    /// A coin of the target's own reserve asset, by funding position.
    Foreign(usize),
}

/// One environment read, with the variable named where it is absent.
fn required_environment(name: &str) -> String {
    std::env::var(name)
        .ok()
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| panic!("{name} is required"))
}

/// The bytes one hexadecimal spelling carries.
fn decode_hex(text: &str) -> Vec<u8> {
    assert_eq!(
        text.len() % 2,
        0,
        "a hexadecimal spelling carries whole bytes"
    );
    let (pairs, _) = text.as_bytes().as_chunks::<2>();
    pairs
        .iter()
        .map(|pair| {
            let digits = std::str::from_utf8(pair).expect("the spelling is ascii");
            u8::from_str_radix(digits, 16).expect("the spelling is hexadecimal")
        })
        .collect()
}

/// One identity, in the order the invoking run states it.
fn identity_bytes(text: &str) -> [u8; 32] {
    <[u8; 32]>::try_from(decode_hex(text).as_slice()).expect("a thirty-two byte identity")
}

/// One identity in internal order, from the order the target prints.
fn printed_identity(text: &str) -> [u8; 32] {
    let mut raw = decode_hex(text);
    raw.reverse();
    <[u8; 32]>::try_from(raw.as_slice()).expect("a thirty-two byte identity")
}

/// One outpoint, from the wire form the adapter answers with.
fn outpoint_of(wire: &WireOutpoint) -> Outpoint {
    Outpoint::new(Txid::from_internal(printed_identity(&wire.txid)), wire.vout)
        .expect("the funded outpoint is well formed")
}

/// The deployment this run is bound to.
fn native_identity() -> CandidateDeploymentIdentity {
    CandidateDeploymentIdentity::new(
        identity_bytes(&required_environment("TRIPOD_LIVE_NETWORK_ID")),
        identity_bytes(&required_environment("TRIPOD_LIVE_GENESIS_ID")),
    )
    .expect("nonzero deployment identities")
}

/// Where the adapter writes its own diagnostics.
///
/// The directory the invoking round names when it names one, so the
/// diagnostics sit beside the round that produced them, and a directory
/// under this run's temporary root otherwise. No capture is written
/// here, so the suite provenance variables are not read at all.
fn diagnostics_directory() -> PathBuf {
    let reported = std::env::var("TRIPOD_LIVE_REPORT_DIR").unwrap_or_default();
    if !reported.is_empty() {
        return PathBuf::from(reported);
    }
    let directory =
        std::env::temp_dir().join(format!("maturity-closure-native-{}", std::process::id()));
    std::fs::create_dir_all(&directory).expect("the diagnostics directory is created");
    directory
}

/// The stable name of one observed outcome layer.
const fn layer_name(layer: ObservedOutcomeLayer) -> &'static str {
    match layer {
        ObservedOutcomeLayer::Accepted => "accepted",
        ObservedOutcomeLayer::ScriptPathRejection => "script-path-rejection",
        ObservedOutcomeLayer::KeyPathRejection => "key-path-rejection",
        ObservedOutcomeLayer::ConsensusRejectionBeforeScript => "consensus-rejection-before-script",
        ObservedOutcomeLayer::RelayPolicyRejection => "relay-policy-rejection",
        ObservedOutcomeLayer::FixtureConstructionFailure => "fixture-construction-failure",
        ObservedOutcomeLayer::ExecutorInfrastructureFailure => "executor-infrastructure-failure",
        _ => "unknown",
    }
}

/// One observation, written where the invoking round's log keeps it.
///
/// Written to this process's own error stream rather than through the
/// print macros, which the test harness captures and discards on a pass.
fn observation_line(case: &str, layer: ObservedOutcomeLayer, detail: &str) {
    let mut stream = std::io::stderr();
    writeln!(stream, "{case} --> {} {detail}", layer_name(layer))
        .expect("the observation reaches the run's error stream");
}

/// Whether one asset field is the explicit asset a leaf literal names.
fn names_literal(field: AssetField, literal: &[u8]) -> bool {
    matches!(field, AssetField::Explicit(asset) if asset.internal().as_slice() == literal)
}

/// The explicit amount one retained coin or output carries.
const fn explicit_amount(field: ValueField) -> u64 {
    match field {
        ValueField::Explicit(amount) => amount,
        _ => panic!("a retained vector carries explicit amounts"),
    }
}

/// The width the record's own witness declaration fixes for one item.
const fn declared_width(value: &StackValueType) -> usize {
    match value {
        StackValueType::Bytes { minimum, .. } => *minimum,
        StackValueType::Encoded(class) => match class.v1_shape().payload() {
            PayloadWidth::Exact(width) => width.get(),
            PayloadWidth::Bounded { minimum, .. } => minimum,
            PayloadWidth::Absent => 0,
        },
        _ => panic!("the announcement witness declares byte and encoded items only"),
    }
}

/// Which funded coin every input of every case spends, and what the
/// foreign coins must hold.
fn coin_slots(vectors: &[AdoptionVector], literal: &[u8]) -> (Vec<Vec<CoinSlot>>, Vec<u64>) {
    let mut slots = Vec::new();
    let mut amounts = Vec::new();
    for (case, vector) in vectors.iter().enumerate() {
        let mut spent = Vec::new();
        for entry in vector.spent() {
            if names_literal(entry.asset(), literal) {
                spent.push(CoinSlot::Singleton(case));
            } else {
                amounts.push(explicit_amount(entry.value()));
                spent.push(CoinSlot::Foreign(amounts.len() - 1));
            }
        }
        slots.push(spent);
    }
    (slots, amounts)
}

/// Whether one retained shape carries an output of explicit value zero.
///
/// The target's amount verification refuses such a transaction before
/// it examines any witness, so no shape carrying one reaches the leaf.
/// The node-free half is untouched by that: zero is the only value
/// below the amount the leaf pins, so an output short of it has no
/// other value to carry there.
fn carries_a_zero_output(vector: &AdoptionVector) -> bool {
    vector
        .transaction()
        .outputs()
        .iter()
        .any(|output| explicit_amount(output.value()) == 0)
}

/// The whole step plan, fixed before the first step is sent.
fn plan_steps(slots: &[Vec<CoinSlot>], reserve_coins: usize) -> Vec<RunStep> {
    let mut steps = vec![RunStep::Singleton];
    steps.extend((0..reserve_coins).map(RunStep::Reserve));
    for (case, inputs) in slots.iter().enumerate() {
        for (input, slot) in inputs.iter().enumerate() {
            if matches!(slot, CoinSlot::Foreign(_)) {
                steps.push(RunStep::Authorize(case, input));
            }
        }
        steps.push(RunStep::Submit(case));
    }
    steps
}

/// The amount every case's singleton coin holds, which the leaf pins.
fn singleton_amount(vectors: &[AdoptionVector], literal: &[u8]) -> u64 {
    let amounts: BTreeSet<u64> = vectors
        .iter()
        .flat_map(AdoptionVector::spent)
        .filter(|entry| names_literal(entry.asset(), literal))
        .map(|entry| explicit_amount(entry.value()))
        .collect();
    let mut found = amounts.into_iter();
    let amount = found.next().expect("every case spends the singleton");
    assert_eq!(found.next(), None, "the cases pin one singleton amount");
    amount
}

/// The issued asset one funding answer names.
fn issued_asset(response: &NativeOperationResponse) -> Result<TargetAssetId, PlanRefused> {
    let printed = response.issued_asset.as_deref().ok_or(PlanRefused)?;
    Ok(TargetAssetId::from_internal(printed_identity(printed)))
}

/// The coins one funding answer created, where it created as many as the
/// step asked for.
fn funded_coins(
    response: &NativeOperationResponse,
    wanted: usize,
) -> Result<Vec<Outpoint>, PlanRefused> {
    if response.funded_outputs.len() != wanted {
        return Err(PlanRefused);
    }
    Ok(response
        .funded_outputs
        .iter()
        .map(|coin| outpoint_of(&coin.outpoint))
        .collect())
}

/// One step this run sends, in the order its plan fixes.
#[derive(Clone, Copy, Debug)]
enum RunStep {
    /// Issue the singleton asset and fund one coin of it per case.
    Singleton,
    /// Fund one coin of the target's own reserve asset.
    Reserve(usize),
    /// Authorize one case's spend of one reserve coin.
    Authorize(usize, usize),
    /// Submit one case.
    Submit(usize),
}

/// One native run of the four retained adoption shapes.
///
/// The funding steps come first because every later step names coins
/// they created, each case's authorizations come immediately before its
/// own submission, and the submissions follow the retained cases' own
/// order, each named by its case.
struct AdoptionRun {
    vectors: Vec<AdoptionVector>,
    slots: Vec<Vec<CoinSlot>>,
    steps: Vec<RunStep>,
    foreign_amounts: Vec<u64>,
    singleton_literal: Vec<u8>,
    singleton_amount: u64,
    singleton_program: Vec<u8>,
    witness_widths: Vec<usize>,
    leaf_bytes: Vec<u8>,
    control_block: Vec<u8>,
    position: usize,
    pending: Option<OperationStep>,
    singleton_asset: Option<TargetAssetId>,
    singleton_coins: Vec<Outpoint>,
    reserve_asset: Option<TargetAssetId>,
    foreign_wires: Vec<WireOutpoint>,
    foreign_coins: Vec<Outpoint>,
    authorizations: BTreeMap<(usize, usize), Vec<Vec<u8>>>,
    observations: Vec<(String, ObservedOutcomeLayer, String)>,
}

impl AdoptionRun {
    /// Everything the run needs from the link, read once.
    fn prepare() -> Self {
        let reviewed = target();
        let (bundle, leaf) = linked(MaturityDeployment::Demonstration);
        let literals = leaf_literals(&leaf, &reviewed).expect("the leaf states its own literals");
        let constructor = bundle
            .instances()
            .first()
            .expect("the link retains its application")
            .constructor();
        let control_block = constructor
            .control_recipe(StateLeafRole::Announcement)
            .expect("the announcement leaf has a control path")
            .control_bytes()
            .expect("the control path encodes");
        let vectors: Vec<AdoptionVector> = AdoptionCase::ALL
            .into_iter()
            .map(|case| {
                adoption_transaction(case, &bundle, &reviewed).expect("the vector is buildable")
            })
            .collect();
        let (slots, foreign_amounts) = coin_slots(&vectors, literals.input_asset());
        Self {
            singleton_amount: singleton_amount(&vectors, literals.input_asset()),
            singleton_literal: literals.input_asset().to_vec(),
            singleton_program: constructor.output_program(),
            witness_widths: bundle
                .record()
                .witness()
                .iter()
                .map(|(_, value)| declared_width(value))
                .collect(),
            leaf_bytes: leaf.bytes().to_vec(),
            control_block,
            steps: plan_steps(&slots, foreign_amounts.len()),
            vectors,
            slots,
            foreign_amounts,
            position: 0,
            pending: None,
            singleton_asset: None,
            singleton_coins: Vec::new(),
            reserve_asset: None,
            foreign_wires: Vec::new(),
            foreign_coins: Vec::new(),
            authorizations: BTreeMap::new(),
            observations: Vec::new(),
        }
    }

    /// The stack that spends the singleton coin.
    ///
    /// The six announcement roles at the widths the composed record
    /// declares, in the order it declares them, then the operator
    /// signature, then the leaf and its control block. The order is the
    /// record's own: its declared starting stack is deepest-first, and
    /// the signature is declared last because the committed operator
    /// key's verification is the first instruction pair the leaf runs.
    /// The six roles and the signature are placeholders of the declared
    /// widths, because no layer of this workspace populates them yet,
    /// except where a declared width is wider than the target relays.
    /// Capping such a placeholder at the reviewed relay width costs this
    /// observation nothing, because what it observes is the leaf's first
    /// instruction pair, which no stack item's width reaches.
    fn announcement_stack(&self) -> Vec<Vec<u8>> {
        let mut stack: Vec<Vec<u8>> = self
            .witness_widths
            .iter()
            .map(|width| vec![0_u8; (*width).min(TAPSCRIPT_STACK_ITEM_RELAY_LIMIT)])
            .collect();
        stack.push(self.leaf_bytes.clone());
        stack.push(self.control_block.clone());
        stack
    }

    /// Where one slot's coin was funded.
    fn coin(&self, slot: CoinSlot) -> Outpoint {
        match slot {
            CoinSlot::Singleton(case) => self.singleton_coins[case],
            CoinSlot::Foreign(index) => self.foreign_coins[index],
        }
    }

    /// One retained output over the assets this run has.
    fn retarget(&self, output: &TargetOutput) -> TargetOutput {
        let asset = if names_literal(output.asset(), &self.singleton_literal) {
            self.singleton_asset.expect("the singleton asset is issued")
        } else {
            self.reserve_asset.expect("the reserve asset is funded")
        };
        TargetOutput::new(
            AssetField::Explicit(asset),
            output.value(),
            output.nonce(),
            output.program().to_vec(),
        )
    }

    /// The stack one input of one case carries.
    fn input_witness(&self, case: usize, index: usize, authorized: bool) -> Vec<Vec<u8>> {
        if index == self.vectors[case].observed().executing_input_index() {
            return self.announcement_stack();
        }
        if !authorized {
            return Vec::new();
        }
        self.authorizations
            .get(&(case, index))
            .cloned()
            .unwrap_or_default()
    }

    /// One case, in the retained shape, over the coins this run holds.
    ///
    /// Positions, amounts, programs and the successor's own output are
    /// the vector's; the outpoints, the two assets and the witnesses are
    /// the run's, because those are what a node requires to be real. The
    /// unauthorized form is what a signing step is handed: a signature
    /// over this input commits to the prevouts, the outputs and this
    /// input's own coin, and to no other input's stack, so the two forms
    /// have one signature between them.
    fn candidate(&self, case: usize, authorized: bool) -> TargetTransaction {
        let vector = &self.vectors[case];
        let source = vector.transaction();
        let inputs: Vec<TargetInput> = source
            .inputs()
            .iter()
            .zip(&self.slots[case])
            .map(|(input, slot)| TargetInput::new(self.coin(*slot), input.sequence()))
            .collect();
        let outputs: Vec<TargetOutput> = source
            .outputs()
            .iter()
            .map(|output| self.retarget(output))
            .collect();
        let witnesses: Vec<InputWitness> = (0..inputs.len())
            .map(|index| InputWitness::new(self.input_witness(case, index, authorized)))
            .collect();
        TargetTransaction::new(
            source.version(),
            inputs,
            outputs,
            source.lock_time(),
            witnesses,
        )
        .expect("the funded case is a transaction")
    }

    /// The step that issues the singleton asset and funds it per case.
    fn singleton_subject(&self) -> Result<OperationSubject, PlanRefused> {
        let subject = TargetFundingSubject {
            issue_asset: true,
            asset: None,
            output_program: self.singleton_program.clone(),
            outputs: u8::try_from(self.vectors.len()).map_err(|_| PlanRefused)?,
            amount_per_output: self.singleton_amount,
        };
        Ok(OperationSubject::Funding(Box::new(subject)))
    }

    /// The step that funds one coin of the target's own reserve asset.
    ///
    /// The retained shape's foreign coins are of some asset that is not
    /// the singleton, and the reserve asset is the one such asset a run
    /// can have coins of: a funding step issues an asset once per run,
    /// and the step that pays out an already-issued one pays out that
    /// same issuance. The reserve asset is also the one the target
    /// weighs a fee in, which the accepted shape's third output is.
    ///
    /// The amount is exactly what the retained shape spends, because a
    /// coin holding more would leave that asset unbalanced and the
    /// tally, rather than the leaf, would be what refused the case.
    fn reserve_subject(&self, index: usize) -> Result<OperationSubject, PlanRefused> {
        let subject = TargetSponsorFundingSubject {
            sponsor_outputs: 1,
            amount_per_sponsor_output: *self.foreign_amounts.get(index).ok_or(PlanRefused)?,
        };
        Ok(OperationSubject::SponsorFunding(Box::new(subject)))
    }

    /// The step that authorizes one case's spend of one reserve coin.
    fn authorize_subject(
        &self,
        case: usize,
        input: usize,
    ) -> Result<OperationSubject, PlanRefused> {
        let CoinSlot::Foreign(index) = self.slots[case][input] else {
            return Err(PlanRefused);
        };
        let subject = TargetSponsorSigningSubject {
            finalized_transaction: self.candidate(case, false).encode(),
            sponsor_input_index: u16::try_from(input).map_err(|_| PlanRefused)?,
            sponsor_outpoint: self.foreign_wires.get(index).ok_or(PlanRefused)?.clone(),
            sighash_profile: WireSighashProfile::AllInputsAllOutputs,
        };
        Ok(OperationSubject::SponsorSigning(Box::new(subject)))
    }

    /// The step this run sends next.
    fn make_step(&self) -> Result<Option<OperationStep>, PlanRefused> {
        let Some(step) = self.steps.get(self.position).copied() else {
            return Ok(None);
        };
        let (name, subject) = match step {
            RunStep::Singleton => ("fund-singleton".to_owned(), self.singleton_subject()?),
            RunStep::Reserve(index) => (
                format!("fund-reserve-{index}"),
                self.reserve_subject(index)?,
            ),
            RunStep::Authorize(case, input) => (
                format!(
                    "authorize-{}-input-{input}",
                    self.vectors[case].case().name()
                ),
                self.authorize_subject(case, input)?,
            ),
            RunStep::Submit(case) => (
                self.vectors[case].case().name().to_owned(),
                OperationSubject::Submission(Box::new(TargetSubmissionSubject {
                    transaction_bytes: self.candidate(case, true).encode(),
                })),
            ),
        };
        Ok(Some(OperationStep::new(&name, subject)))
    }

    /// What one answer settles.
    ///
    /// Every step before a submission must have been accepted, because
    /// a later step names what it produced. A submission's verdict is
    /// the datum this run is for and settles nothing.
    fn settle(&mut self, response: &NativeOperationResponse) -> Result<(), PlanRefused> {
        let step = self.steps.get(self.position).copied().ok_or(PlanRefused)?;
        if let RunStep::Submit(case) = step {
            self.observations.push((
                self.vectors[case].case().name().to_owned(),
                response.observed_layer,
                response.observed_detail.clone().unwrap_or_default(),
            ));
            return Ok(());
        }
        if response.observed_layer != ObservedOutcomeLayer::Accepted {
            return Err(PlanRefused);
        }
        match step {
            RunStep::Singleton => {
                self.singleton_asset = Some(issued_asset(response)?);
                self.singleton_coins = funded_coins(response, self.vectors.len())?;
            }
            RunStep::Reserve(_) => self.settle_reserve(response)?,
            RunStep::Authorize(case, input) => {
                if response.sponsor_witness.is_empty() {
                    return Err(PlanRefused);
                }
                self.authorizations
                    .insert((case, input), response.sponsor_witness.clone());
            }
            RunStep::Submit(_) => {}
        }
        Ok(())
    }

    /// The coin one reserve funding step created, and the asset it is
    /// of, which the target names rather than this run.
    fn settle_reserve(&mut self, response: &NativeOperationResponse) -> Result<(), PlanRefused> {
        let [coin] = response.funded_outputs.as_slice() else {
            return Err(PlanRefused);
        };
        let asset = TargetAssetId::from_internal(printed_identity(&coin.asset));
        if self
            .reserve_asset
            .as_ref()
            .is_some_and(|held| *held != asset)
        {
            return Err(PlanRefused);
        }
        self.reserve_asset = Some(asset);
        self.foreign_wires.push(coin.outpoint.clone());
        self.foreign_coins.push(outpoint_of(&coin.outpoint));
        Ok(())
    }
}

impl TargetOperationPlanner for AdoptionRun {
    fn next_step(
        &mut self,
        previous: Option<(&OperationCaseId, &NativeOperationResponse)>,
    ) -> Result<Option<OperationStep>, PlanRefused> {
        match (self.pending.take(), previous) {
            (Some(step), Some((case, response)))
                if step.case() == case && &response.case == case =>
            {
                response.validate_shape().map_err(|_| PlanRefused)?;
                self.settle(response)?;
                self.position += 1;
            }
            (None, None) if self.position == 0 => {}
            _ => return Err(PlanRefused),
        }
        let step = self.make_step()?;
        self.pending.clone_from(&step);
        Ok(step)
    }
}

/// Run the planned operations against the adapter the run names.
fn execute(
    adapter: &Path,
    diagnostics: &Path,
    deployment: &CandidateDeploymentIdentity,
    planner: &mut dyn TargetOperationPlanner,
    capture: &mut NativeOperationCapture,
) -> Result<(), target_elements_conformance::error::NativeConformanceError> {
    let reviewed = reviewed_elements_tapscript().expect("reviewed target");
    let binding = validate_reviewed_development_binding(
        &reviewed,
        DevelopmentDeploymentBinding::new(
            reviewed.definition().version(),
            DeploymentEnvironment::Development,
            *deployment.network_id(),
            *deployment.genesis_id(),
            ActivationDeclaration::new(true, LeafVersion::TAPSCRIPT, []),
            None,
        ),
    )
    .expect("development binding");
    let configuration = ExecutorConfiguration::new(
        adapter,
        ExecutorTrust::ReviewedNonMock,
        Duration::from_secs(300),
        ExecutorDiagnostics::in_directory(&diagnostics.join("diagnostics")),
    );
    execute_operations_captured(&reviewed, &binding, &configuration, planner, capture).map(drop)
}

/// Hold every compared check to sitting after the operator's.
fn compared_checks_follow_the_operator_check(
    leaf: &DecodedAnnouncementLeaf,
    reviewed: &ReviewedElementsTapscriptDefinition,
) {
    let operator = kept_check_site(leaf, reviewed, KeptCheck::OperatorAuthorization)
        .expect("the operator authorization is in the bytes");
    assert_eq!(operator, 0, "the leaf runs another check first");
    for check in COMPARED_CHECKS {
        let site =
            kept_check_site(leaf, reviewed, check).expect("the compared check is in the bytes");
        assert!(site > operator, "{} precedes the signature", check.name());
    }
}

/// The four retained shapes, submitted to a real target.
///
/// What the run establishes: a node's own interpreter accepts the whole
/// chain from the linked leaf's bytes through the static subtree, the
/// output key and the control block — it reveals the leaf at a funded
/// output of the constructor's own program and reaches that leaf's first
/// instruction pair — and then refuses at the committed operator key's
/// verification. The node-free half recomputed that chain; this is a
/// target agreeing with the recomputation by running it.
///
/// What the run cannot establish is the gate's accepted case. Three
/// facts of the tree stand in the way, and each is a fact of the
/// deployment rather than of the run. The committed operator key is
/// thirty-two bytes of public, meaningless material whose scalar nobody
/// holds, and the leaf verifies a signature under it, so no witness
/// satisfies it. The pinned singleton asset is a fixture constant, and a
/// node derives an asset identity from its own issuing outpoint, so a
/// really funded coin never carries the identity the leaf compares
/// against. Output zero's program is authenticated against
/// witness-supplied successor metadata, and the linked bundle's own
/// outstanding set records that nothing here populates a witness or
/// searches a successor's representation nonce. The announcement's
/// witness roles are therefore placeholders: the run submits what it
/// can build, and every case is refused before any compared field is
/// read.
///
/// Two rules of the target's own decide where each case stops, and
/// both are the target's rather than this run's. Its relay policy
/// admits no tapscript stack item as wide as the predecessor-metadata
/// role declares, so a spend carrying that role at its declared width
/// is refused before any script runs at all; the placeholder is capped
/// at the limit, which is what lets the leaf be reached. Its amount
/// verification refuses an output of explicit value zero before it
/// examines any witness, and one retained shape carries one, so that
/// case is refused before the leaf while the other three are refused
/// inside it.
#[test]
fn the_four_adoption_vectors_are_submitted_to_a_real_target() {
    let Ok(executor) = std::env::var("TRIPOD_LIVE_EXECUTOR") else {
        return;
    };
    if executor.is_empty() {
        return;
    }
    let deployment = native_identity();
    let diagnostics = diagnostics_directory();
    let reviewed = target();
    let (_, leaf) = linked(MaturityDeployment::Demonstration);
    compared_checks_follow_the_operator_check(&leaf, &reviewed);

    let mut run = AdoptionRun::prepare();
    let mut capture = NativeOperationCapture::default();
    let outcome = execute(
        Path::new(&executor),
        &diagnostics,
        &deployment,
        &mut run,
        &mut capture,
    );
    for (case, layer, detail) in &run.observations {
        observation_line(case, *layer, detail);
    }
    outcome.expect("the native exchange completed");

    assert_eq!(run.observations.len(), AdoptionCase::ALL.len());
    assert_eq!(capture.operations().len(), run.steps.len());
    for (observed, vector) in run.observations.iter().zip(&run.vectors) {
        let (case, layer, detail) = observed;
        assert_eq!(case.as_str(), vector.case().name());
        if carries_a_zero_output(vector) {
            assert_eq!(
                *layer,
                ObservedOutcomeLayer::ConsensusRejectionBeforeScript,
                "{case}"
            );
            assert!(detail.contains(AMOUNT_FAILURE), "{case}: {detail}");
            continue;
        }
        assert_eq!(*layer, ObservedOutcomeLayer::ScriptPathRejection, "{case}");
        assert!(detail.starts_with(SCRIPT_FAILURE), "{case}: {detail}");
        assert!(detail.contains(SIGNATURE_FAILURE), "{case}: {detail}");
        assert!(!detail.contains(COMMITMENT_FAILURE), "{case}: {detail}");
    }
}
