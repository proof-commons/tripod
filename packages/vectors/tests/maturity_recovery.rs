//! Public announcement recovery over the accepted archive's exact handoff.
//!
//! The handoff names only published values. This binary checks its own import
//! boundary and the reconstructed program against the stated output bytes.

use realization::AnnouncementLeadBounds;
use realization::Cycle;
use realization::StateRepresentationNonce;
use realization::announce_maturity;
use tapscript::STATE_NUMS_KEY;
use tapscript::StateInternalKeyPolicy;
use tapscript::StateLeafRole;
use tapscript::StateStaticLeaf;
use tapscript::StateStaticNode;
use tapscript::StateStaticSubtree;
use tapscript::StateWitnessSchedule;
use tapscript::TapscriptProgram;
use tapscript::state_metadata_leaf_program;
use target_elements::ReviewedElementsTapscriptDefinition;
use target_elements::TargetContractVersion;
use transaction::bytes::TargetTransaction;
use transaction::taproot::leaf_hash;
use vectors::maturity_closure::MaturityDeployment;
use vectors::maturity_closure::OracleStateCurve;
use vectors::maturity_closure::closure_target;
use vectors::maturity_continuity::MaturityByteSource;
use vectors::maturity_corpus::maturity_variable_run_of_record;
use vectors::maturity_native::MaturityAcceptanceObligation;
use vectors::maturity_recovery::PublicAnnouncementHandoff;
use vectors::maturity_recovery::PublicAnnouncementLocator;
use vectors::maturity_recovery::recover_public_successor;
use vectors::maturity_recovery_report::assemble_maturity_public_recovery_report;
use vectors::maturity_recovery_report::render_maturity_public_recovery_report;
use vectors::maturity_recovery_report::validate_maturity_public_recovery_report;

/// The one published announcement leaf is the whole static subtree:
/// the recipe carries every consumer within it and needs no support leaf.
fn published_subtree(
    target: &ReviewedElementsTapscriptDefinition,
    leaf_bytes: &[u8],
) -> StateStaticSubtree {
    let program = TapscriptProgram::decode(target, leaf_bytes).expect("published leaf");
    StateStaticSubtree::new(
        target,
        Some(StateStaticNode::Leaf {
            identity: 0,
            leaf: StateStaticLeaf {
                role: StateLeafRole::Announcement,
                program,
                version: target.definition().leaf_version().get(),
            },
        }),
    )
    .expect("published one-leaf subtree")
}

fn accepted_handoff() -> PublicAnnouncementHandoff {
    let corpus = maturity_variable_run_of_record().expect("accepted archive");
    let MaturityAcceptanceObligation::Established { readback, .. } =
        corpus.evidence().acceptance_obligation()
    else {
        panic!("accepted archive has an established readback")
    };
    let target = closure_target().expect("reviewed contract");
    let transaction = TargetTransaction::decode(readback.bytes()).expect("accepted transaction");
    let stack = transaction.witnesses()[0].stack();
    let subtree = published_subtree(&target, &stack[7]);
    let (minimum, maximum) = MaturityDeployment::PublishedSignerHeld
        .parameters()
        .expect("published deployment")
        .lead();
    PublicAnnouncementHandoff::new(
        PublicAnnouncementLocator::new(
            MaturityByteSource::ArchivedSubmission {
                run_address: corpus.report().run_address().to_owned(),
            },
            readback.identity(),
        ),
        readback.bytes().to_vec(),
        0,
        corpus.schedule(),
        AnnouncementLeadBounds::new(Cycle::new(minimum), Cycle::new(maximum))
            .expect("published lead bounds"),
        subtree,
        StateInternalKeyPolicy::new(STATE_NUMS_KEY, &OracleStateCurve)
            .expect("published internal key"),
        TargetContractVersion::V2,
    )
}

#[test]
fn the_accepted_archives_handoff_recovers_output_zero_from_public_values() {
    let corpus = maturity_variable_run_of_record().expect("accepted archive");
    let MaturityAcceptanceObligation::Established { readback, .. } =
        corpus.evidence().acceptance_obligation()
    else {
        panic!("accepted archive has an established readback")
    };
    let target = closure_target().expect("reviewed contract");
    let transaction = TargetTransaction::decode(readback.bytes()).expect("accepted transaction");
    let stack = transaction.witnesses()[0].stack();
    assert_eq!(corpus.schedule(), StateWitnessSchedule::VariableMetadata);
    let subtree = published_subtree(&target, &stack[7]);
    assert_eq!(stack[3].as_slice(), subtree.root());
    let (minimum, maximum) = MaturityDeployment::PublishedSignerHeld
        .parameters()
        .expect("published deployment")
        .lead();
    assert_eq!((minimum, maximum), (4, 6));
    assert_eq!(corpus.report().target_contract(), "elements-tapscript-v2");
    let recovered = recover_public_successor(accepted_handoff()).expect("public recovery");
    let requested = Cycle::new(u64::from_be_bytes(
        stack[2].as_slice().try_into().expect("cycle width"),
    ));
    let expected = announce_maturity(
        &recovered.predecessor_metadata().semantic,
        requested,
        AnnouncementLeadBounds::new(Cycle::new(minimum), Cycle::new(maximum)).expect("bounds"),
    )
    .expect("valid announcement");
    assert_eq!(*recovered.successor_semantics(), expected);
    assert_eq!(recovered.requested_cycle(), requested);
    let nonce = u32::from_be_bytes(stack[1].as_slice().try_into().expect("nonce width"));
    assert_eq!(
        recovered.representation(),
        StateRepresentationNonce::new(nonce)
    );
    let leaf_program = state_metadata_leaf_program(&target, recovered.predecessor_metadata())
        .expect("metadata leaf");
    let hash = leaf_hash(
        target.definition().leaf_version(),
        &leaf_program.encode(&target),
    );
    assert_eq!(stack[8].get(stack[8].len() - 32..), Some(hash.as_slice()));
    assert!(recovered.programs_agree());
    assert_eq!(
        recovered.actual_program(),
        transaction.outputs()[0].program()
    );
    assert_eq!(
        recovered.reconstructed_program(),
        recovered.actual_program()
    );
    assert_eq!(recovered.contract(), target.definition().version());
}

#[test]
fn the_accepted_archives_public_recovery_report_validates_from_the_handoff_alone() {
    let handoff = accepted_handoff();
    let report = assemble_maturity_public_recovery_report(handoff).expect("report assembles");
    let independent = accepted_handoff();
    let validated =
        validate_maturity_public_recovery_report(&report, &independent).expect("report validates");
    let rendered = render_maturity_public_recovery_report(&validated);
    assert!(rendered.contains("role state-public-recovery\n"));
    assert!(rendered.contains("programs_agree true\n"));
}

#[test]
fn the_binarys_import_list_excludes_the_bundle_the_identity_and_both_planners() {
    let source = include_str!("maturity_recovery.rs");
    let actual: Vec<_> = source
        .lines()
        .filter(|line| line.starts_with("use "))
        .collect();
    let expected = [
        "use realization::AnnouncementLeadBounds;",
        "use realization::Cycle;",
        "use realization::StateRepresentationNonce;",
        "use realization::announce_maturity;",
        "use tapscript::STATE_NUMS_KEY;",
        "use tapscript::StateInternalKeyPolicy;",
        "use tapscript::StateLeafRole;",
        "use tapscript::StateStaticLeaf;",
        "use tapscript::StateStaticNode;",
        "use tapscript::StateStaticSubtree;",
        "use tapscript::StateWitnessSchedule;",
        "use tapscript::TapscriptProgram;",
        "use tapscript::state_metadata_leaf_program;",
        "use target_elements::ReviewedElementsTapscriptDefinition;",
        "use target_elements::TargetContractVersion;",
        "use transaction::bytes::TargetTransaction;",
        "use transaction::taproot::leaf_hash;",
        "use vectors::maturity_closure::MaturityDeployment;",
        "use vectors::maturity_closure::OracleStateCurve;",
        "use vectors::maturity_closure::closure_target;",
        "use vectors::maturity_continuity::MaturityByteSource;",
        "use vectors::maturity_corpus::maturity_variable_run_of_record;",
        "use vectors::maturity_native::MaturityAcceptanceObligation;",
        "use vectors::maturity_recovery::PublicAnnouncementHandoff;",
        "use vectors::maturity_recovery::PublicAnnouncementLocator;",
        "use vectors::maturity_recovery::recover_public_successor;",
        "use vectors::maturity_recovery_report::assemble_maturity_public_recovery_report;",
        "use vectors::maturity_recovery_report::render_maturity_public_recovery_report;",
        "use vectors::maturity_recovery_report::validate_maturity_public_recovery_report;",
    ];
    assert_eq!(actual, expected);
    for forbidden in [
        ["linker", "::"].concat(),
        ["Candidate", "DeploymentIdentity"].concat(),
        ["TargetOperation", "Planner"].concat(),
        ["MaturityAnnouncement", "Planner"].concat(),
    ] {
        assert!(!source.contains(&forbidden));
    }
}
