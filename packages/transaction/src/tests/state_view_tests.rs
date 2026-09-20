//! The public current-STATE view: its two laws, its one check, and its
//! residual.
//!
//! # The bundle under test is a real one, and it is the shared one
//!
//! The view's eighth entry is a linked maturity bundle, and the one used
//! here is the fixture beside this file: derived, planned, composed,
//! bound and linked through the public builders, the way an external
//! consumer would reach it. A hand-assembled bundle would let the
//! reconstruction agree with a policy and a static subtree that no link
//! produced, which is the one thing the check is for.
//!
//! It lives in its own module because the announcement ABI's tests need
//! the same bundle. Two constructions of it would be two deployments
//! answering to one name, and a test reading the second while its subject
//! read the first would pass or fail for a reason nobody could see.
//!
//! # What stays here
//!
//! The two helpers below are this file's own: committing a pair the way
//! the check commits it, and finding a nonce at which a metadata commits.
//! Both exist to build the negatives, and neither is a fact about the
//! deployment, so neither belongs in the shared fixture.

use linker::CandidateLinkedMaturityBundle;
use realization::{Cycle, EncodedStateMetadata, StateMetadata, StateRepresentationNonce};
use tapscript::{StateConstructorRefusal, state_output_program_at_nonce};

use super::state_support::{
    FixtureStateCurve, demonstration_view, linked_bundle, linked_pair, state_metadata, statements,
};
use crate::error::TransactionRefusal;
use crate::state_view::{MaturityViewEntry, MaturityViewResidual, PublicMaturityStateView};

/// Commit one metadata and nonce the way the view's check commits them.
fn commit(
    bundle: &CandidateLinkedMaturityBundle,
    metadata: StateMetadata,
    representation: StateRepresentationNonce,
) -> Result<Vec<u8>, StateConstructorRefusal> {
    state_output_program_at_nonce(
        &super::reviewed_target(),
        &EncodedStateMetadata {
            semantic: metadata,
            representation,
        },
        bundle.static_subtree(),
        bundle.policy().internal_key(),
        &FixtureStateCurve,
    )
}

/// The first nonce within the reviewed scan's reach at which `metadata`
/// commits, other than `besides`.
fn nonce_that_commits(
    bundle: &CandidateLinkedMaturityBundle,
    metadata: StateMetadata,
    besides: Option<StateRepresentationNonce>,
) -> StateRepresentationNonce {
    (0..64)
        .map(StateRepresentationNonce::new)
        .find(|nonce| Some(*nonce) != besides && commit(bundle, metadata, *nonce).is_ok())
        .expect("one of the first sixty-four nonces commits")
}

#[test]
fn a_view_over_the_demonstration_bundle_validates_and_names_freshness_as_its_residual() {
    let view = demonstration_view();
    let validated = view
        .validate(&super::reviewed_target(), &FixtureStateCurve)
        .expect("the stated pair reproduces the stated program");
    assert_eq!(
        validated.residual(),
        MaturityViewResidual::CurrentStateRootFreshness
    );
    assert_eq!(validated.view(), &view);
}

#[test]
fn the_entry_census_is_the_guides_ten_entries_in_its_order() {
    let names: Vec<_> = MaturityViewEntry::ALL
        .iter()
        .map(|entry| entry.name())
        .collect();
    assert_eq!(
        names,
        vec![
            "current-state-outpoint",
            "asset-and-amount",
            "predecessor-metadata",
            "predecessor-representation-nonce",
            "current-root-binding",
            "predecessor-program",
            "current-cycle",
            "accepted-linked-bundle",
            "operator-public-identity",
            "deployment-symbols",
        ]
    );
}

#[test]
fn the_three_projected_entries_are_read_from_the_entries_they_project() {
    let bundle = linked_bundle();
    let view = demonstration_view();
    let projected: Vec<_> = MaturityViewEntry::ALL
        .iter()
        .filter_map(|entry| entry.projected_from().map(|source| (*entry, source)))
        .collect();
    assert_eq!(
        projected,
        vec![
            (
                MaturityViewEntry::CurrentCycle,
                MaturityViewEntry::PredecessorMetadata
            ),
            (
                MaturityViewEntry::OperatorPublicIdentity,
                MaturityViewEntry::AcceptedLinkedBundle
            ),
            (
                MaturityViewEntry::DeploymentSymbols,
                MaturityViewEntry::AcceptedLinkedBundle
            ),
        ]
    );
    assert_eq!(view.current_cycle(), state_metadata().cycle);
    assert_eq!(
        view.operator_public_identity(),
        bundle.deployment().operator().key()
    );
    assert_eq!(view.deployment_symbols(), bundle.resolved());
    assert_eq!(view.accepted_linked_bundle(), &bundle);
}

#[test]
fn a_second_statement_of_any_stated_entry_refuses_naming_that_entry() {
    let bundle = linked_bundle();
    let (nonce, program) = linked_pair(&bundle);
    let stated = statements(&bundle, state_metadata(), nonce, program);
    for statement in &stated {
        let mut offered = stated.clone();
        offered.push(statement.clone());
        assert_eq!(
            PublicMaturityStateView::new(offered).unwrap_err(),
            TransactionRefusal::DuplicateMaturityViewEntry(statement.entry())
        );
    }
}

#[test]
fn an_entry_never_stated_refuses_naming_that_entry() {
    let bundle = linked_bundle();
    let (nonce, program) = linked_pair(&bundle);
    let stated = statements(&bundle, state_metadata(), nonce, program);
    for (position, statement) in stated.iter().enumerate() {
        let mut offered = stated.clone();
        offered.remove(position);
        assert_eq!(
            PublicMaturityStateView::new(offered).unwrap_err(),
            TransactionRefusal::MissingMaturityViewEntry(statement.entry())
        );
    }
}

#[test]
fn another_nonce_that_commits_refuses_by_name_with_both_lengths() {
    let bundle = linked_bundle();
    let (selected, program) = linked_pair(&bundle);
    let other = nonce_that_commits(&bundle, state_metadata(), Some(selected));
    let view = PublicMaturityStateView::new(statements(
        &bundle,
        state_metadata(),
        other,
        program.clone(),
    ))
    .expect("the seven statements name distinct entries");
    assert_eq!(
        view.validate(&super::reviewed_target(), &FixtureStateCurve)
            .unwrap_err(),
        TransactionRefusal::MaturityViewProgramNotReconstructed {
            supplied: program.len(),
            recomputed: program.len(),
        }
    );
}

#[test]
fn a_nonce_that_commits_to_nothing_refuses_carrying_the_constructors_own_refusal() {
    let bundle = linked_bundle();
    let program = linked_pair(&bundle).1;
    let (nonce, refusal) = (0..64)
        .map(StateRepresentationNonce::new)
        .find_map(|nonce| {
            commit(&bundle, state_metadata(), nonce)
                .err()
                .map(|refusal| (nonce, refusal))
        })
        .expect("the fixed outer branch side goes unsatisfied at one of the first sixty-four");
    let view = PublicMaturityStateView::new(statements(&bundle, state_metadata(), nonce, program))
        .expect("the seven statements name distinct entries");
    assert_eq!(
        view.validate(&super::reviewed_target(), &FixtureStateCurve)
            .unwrap_err(),
        TransactionRefusal::MaturityViewCommitmentRefused { refusal }
    );
}

#[test]
fn metadata_at_another_cycle_refuses_by_name_and_moves_the_projected_cycle() {
    let bundle = linked_bundle();
    let program = linked_pair(&bundle).1;
    let altered = StateMetadata {
        cycle: Cycle::new(6),
        ..state_metadata()
    };
    let nonce = nonce_that_commits(&bundle, altered, None);
    let view = PublicMaturityStateView::new(statements(&bundle, altered, nonce, program.clone()))
        .expect("the seven statements name distinct entries");
    assert_eq!(view.current_cycle(), Cycle::new(6));
    assert_eq!(
        view.validate(&super::reviewed_target(), &FixtureStateCurve)
            .unwrap_err(),
        TransactionRefusal::MaturityViewProgramNotReconstructed {
            supplied: program.len(),
            recomputed: program.len(),
        }
    );
}
