//! The maturity link, closed over its linked bytes.
//!
//! Every test here reads the decoded announcement leaf and the plan's
//! and record's own statements. None reads a linker field as the fact
//! under test, which is what makes this evidence about the program a
//! spend would run rather than evidence that the linker agrees with
//! itself.
//!
//! The native half is one ignored test. Run it with the adapter and the
//! capture environment set in the invoking shell:
//! ```sh
//! TRIPOD_LIVE_EXECUTOR=/path/to/adapter \
//! cargo test -p tripod-vectors --test maturity_closure -- --ignored --test-threads=1
//! ```

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::sync::LazyLock;

use linker::{StateDischargeClass, StateLinkedCarrier};
use tapscript::{
    StackItem, StateAnnouncementSymbol, StateProgramComponent, StateProgramSymbol,
    TapscriptInstruction, TapscriptProgram,
};
use target_elements::{OpcodeId, ReviewedElementsTapscriptDefinition};
use target_elements_conformance::executor::OperationStep;
use target_elements_conformance::protocol::OperationSubject;
use transaction::bytes::TargetTransaction;
use vectors::maturity_closure::{
    AdoptionCase, DecodedAnnouncementLeaf, KeptCheck, LocatedRow, MaturityClosureRefusal,
    MaturityDeployment, ObservedField, OracleStateCurve, Verdict, adoption_transaction,
    closure_target, decode_announcement_leaf, decoded_deployment, forbidden_program_literals,
    kept_check_site, leaf_literals, locate_discharges, moved_sites, recompute_golden,
    recovered_values, removed_and_kept_checks,
};

/// The golden roots this lane publishes, recomputed and carried alike.
///
/// The demonstration deployment's static root, outer root and output
/// key, then the second deployment's three.
///
/// A pinned constant is what makes a golden a golden: it is checked
/// against the recomputation AND against what the bundle carries in one
/// comparison, so a pin that drifted from either fails here rather than
/// quietly tracking whichever side moved.
const GOLDEN: [&str; 6] = [
    "cf54a9f68d066ca669c447045d01dfc15d2c6c1c33a13a2a11e8d1a52a50eb92",
    "3ddf7410676048756dc1f5750474eafc14f7c2b9e7d1c846da7f29d4cbcd72bf",
    "fa127d5e1bf8f124f66ac48403378ab56a61d95a00ef973566cc01c270c9c2e1",
    "06051be38b1f852856cb4280489fb4246a60d00507a88e8324e873e50819ca09",
    "12b5b81e34fdd1e69429efe53ef44be8be22a1dc2432f3f7097cfd72f18723d6",
    "71f65e4be1eae5842deca56de6c09ca452638ccf2fc02f880bb10057c6c53fa7",
];

type Linked = (
    linker::CandidateLinkedMaturityBundle,
    DecodedAnnouncementLeaf,
);

/// One deployment's linked bundle and decoded leaf, linked once.
fn linked(deployment: MaturityDeployment) -> Linked {
    static DEMONSTRATION: LazyLock<Linked> = LazyLock::new(|| {
        decoded_deployment(MaturityDeployment::Demonstration)
            .expect("the demonstration deployment links")
    });
    static SECOND: LazyLock<Linked> =
        LazyLock::new(|| decoded_deployment(MaturityDeployment::Second).expect("the second links"));
    match deployment {
        MaturityDeployment::Demonstration => DEMONSTRATION.clone(),
        MaturityDeployment::Second => SECOND.clone(),
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
    let (bundle, leaf) = linked(MaturityDeployment::Demonstration);
    let literals = leaf_literals(&leaf, &reviewed).expect("the leaf states its own literals");
    assert_eq!(literals.pinned_input_index(), 0);
    assert_eq!(literals.input_script_version(), 1);
    assert_eq!(literals.output_script_version(), 1);
    assert_eq!(literals.input_asset(), literals.output_asset());
    assert_eq!(literals.input_amount(), literals.output_amount());

    let expected = expected_verdicts();
    for case in AdoptionCase::ALL {
        let vector =
            adoption_transaction(case, &bundle, &reviewed).expect("the vector is buildable");
        assert_eq!(vector.expected(), &expected[&case], "{}", case.name());
        assert_eq!(
            TargetTransaction::decode(vector.bytes()).as_ref(),
            Ok(vector.transaction())
        );
        assert_eq!(vector.spent().len(), vector.transaction().inputs().len());
        assert_eq!(vector.subject().case().step, case.name());
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

// --- (g) The native half, outstanding ----------------------------------

/// The four retained subjects, submitted to a real target.
///
/// The node-free half decides what the leaf's own bytes decide, and
/// stops there. Whether a target accepts the first vector and refuses
/// the other three is a run, and a run needs the toolchain no pod
/// carries today. This test names the submission and leaves it to the
/// capture of record: it reads the adapter the run would use, builds
/// exactly the subjects that run would send, and asserts nothing about
/// any node.
#[test]
#[ignore = "requires the native adapter and capture environment"]
fn the_four_adoption_vectors_are_submitted_to_a_real_target() {
    let executor = std::env::var("TRIPOD_LIVE_EXECUTOR").expect("TRIPOD_LIVE_EXECUTOR is required");
    assert_ne!(executor, "");

    let reviewed = target();
    let (bundle, _) = linked(MaturityDeployment::Demonstration);
    let steps: Vec<OperationStep> = AdoptionCase::ALL
        .into_iter()
        .map(|case| {
            adoption_transaction(case, &bundle, &reviewed)
                .expect("the vector is buildable")
                .subject()
                .clone()
        })
        .collect();

    assert_eq!(steps.len(), 4);
    let names: BTreeSet<&str> = steps.iter().map(|step| step.case().step.as_str()).collect();
    assert_eq!(names.len(), 4);
}
