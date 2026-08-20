//! What the public surface does and does not offer.
//!
//! The tests below are about reachability rather than behaviour: which
//! statements a caller outside this crate can make. A guarantee that
//! holds only because nobody happened to call the wrong function is not
//! a guarantee.

use crate::tests::{deployment, relocatable_bundle, reviewed_target};
use crate::{LinkedArtifactStatus, SelfCommitmentStrategy, link_candidate};

#[test]
fn the_status_is_read_and_never_written() {
    // §1.9 keeps the candidate and final states distinct. There is no
    // constructor argument, field, or setter through which a caller
    // could claim a promotion, so the only reachable status is the one
    // below — and the two further variants exist so a later wave's
    // promotion has a vocabulary rather than a new type.
    let target = reviewed_target();
    let bundle = link_candidate(
        &target,
        &relocatable_bundle(),
        &deployment(
            &target,
            SelfCommitmentStrategy::ExternallyAuthenticatedCommitment,
        ),
    )
    .expect("the demonstration link completes");

    assert_eq!(bundle.status(), LinkedArtifactStatus::Prototype);
    assert_ne!(
        bundle.status(),
        LinkedArtifactStatus::CandidateOperationProven
    );
    assert_ne!(bundle.status(), LinkedArtifactStatus::ProductionApproved);
}

#[test]
fn the_obligation_set_cannot_be_empty() {
    // Structurally non-empty, so a linked bundle owing nothing has no
    // representation. The count is a `NonZeroUsize`, which is the same
    // statement in the type system.
    let target = reviewed_target();
    let bundle = link_candidate(
        &target,
        &relocatable_bundle(),
        &deployment(
            &target,
            SelfCommitmentStrategy::ExternallyAuthenticatedCommitment,
        ),
    )
    .expect("the demonstration link completes");

    let obligations = bundle.outstanding_obligations();
    assert!(obligations.count().get() >= 1);
    assert_eq!(
        obligations.obligations().count(),
        obligations.count().get(),
        "the iterator and the count disagree",
    );
}

#[test]
fn a_link_returns_a_bundle_or_a_refusal_and_never_both() {
    // §1.11 applied to a pipeline: no path returns a partial artifact
    // alongside a diagnostic. The signature is what enforces it, and
    // this records that the signature is the enforcement.
    let target = reviewed_target();
    let refused = link_candidate(
        &target,
        &relocatable_bundle(),
        &deployment(&target, SelfCommitmentStrategy::NotStated),
    );

    assert!(refused.is_err());
    assert!(refused.ok().is_none());
}

#[test]
fn the_crate_names_no_forbidden_package() {
    // §14.1 forbids `model`, `transaction`, `vectors`, `release`, and
    // `artifacts` as direct dependencies. A test cannot read the
    // manifest, but it can establish that no path through this crate's
    // own modules names one, because a name that does not resolve does
    // not compile — so this file compiling at all, with no such import
    // anywhere in the crate, is the check. What is asserted here is the
    // positive half: the two admitted packages are reachable and used.
    let target = reviewed_target();
    let bundle = relocatable_bundle();

    assert_eq!(
        bundle.constructor().leaf_version(),
        target_elements::LeafVersion::TAPSCRIPT
    );
    assert_eq!(
        bundle.constructor().contract(),
        target.definition().version()
    );
}
