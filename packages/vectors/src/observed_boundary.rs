//! The one place the observation vocabulary and the boundary vocabulary
//! are put side by side.
//!
//! §1.5 names the boundary a class expects; the adapter records the layer
//! a run actually reached. They are DIFFERENT TYPES on purpose — one is
//! an expectation written before anything executes, the other an
//! observation made after — and a comparison between them is only
//! meaningful through an explicit total mapping.
//!
//! # Why this is a module rather than a method
//!
//! It used to be a private function of [`crate::render`], reachable only
//! by the renderer, while the live evidence classifier — the consumer
//! whose verdicts move the §15 matrix — compared nothing at all and took
//! "a refusal was attributable" for "the row was answered at its own
//! boundary". Two consumers of one semantic rule, one of which had no
//! access to it, is how the seven wrong-boundary rows came to stand as
//! answered. The rule lives here so that BOTH read the same function and
//! neither can drift from the other.
//!
//! [`crate::matrix`] would be the other candidate home and is deliberately
//! not used: that module states in its own contract that nothing in it
//! observes anything, and the observation vocabulary has no business
//! inside a specification transcribed into types.
//!
//! # Fail-closed is the whole point
//!
//! The mapping is total over the observed layer and returns
//! [`Option`] rather than a boundary, because two observed layers are NOT
//! target verdicts at all and no boundary is the right answer for them.
//! [`ObservedOutcomeLayer`] is additionally `#[non_exhaustive]` and owned
//! by another package, so a layer minted tomorrow reaches the wildcard
//! arm and maps to [`None`] — it fails to match every boundary rather
//! than matching one by accident. A layer added upstream must be given a
//! deliberate arm here before any row can be answered at it.

use target_elements_conformance::protocol::ObservedOutcomeLayer;

use crate::matrix::EvidenceBoundary;

/// The §1.5 boundary an observed layer IS, where it is one at all.
///
/// [`None`] means the observation is not a target verdict, so no boundary
/// can be claimed from it:
///
/// - [`ObservedOutcomeLayer::FixtureConstructionFailure`] — the adapter
///   could not build the transaction, so the target was never asked. A
///   boundary read off this would manufacture a target fact out of a
///   harness bug, which is the single confusion §1.5 exists to prevent.
/// - [`ObservedOutcomeLayer::ExecutorInfrastructureFailure`] — the
///   environment failed around the run. [`EvidenceBoundary`] has a member
///   of the same name, and it is deliberately NOT returned here: that
///   member names a boundary a ROW may declare, and an infrastructure
///   failure is never evidence that a row's fault was refused.
///
/// Every other layer is a verdict the target reached, and maps to the one
/// boundary that names it.
#[must_use]
pub const fn observed_boundary(layer: ObservedOutcomeLayer) -> Option<EvidenceBoundary> {
    match layer {
        ObservedOutcomeLayer::ConsensusRejectionBeforeScript => {
            Some(EvidenceBoundary::ConsensusRejectionBeforeScript)
        }
        ObservedOutcomeLayer::KeyPathRejection => Some(EvidenceBoundary::KeyPathRejection),
        ObservedOutcomeLayer::ScriptPathRejection => Some(EvidenceBoundary::ScriptPathRejection),
        ObservedOutcomeLayer::RelayPolicyRejection => Some(EvidenceBoundary::RelayPolicyRejection),
        ObservedOutcomeLayer::Accepted => Some(EvidenceBoundary::AcceptedTransaction),
        // THE FAIL-CLOSED ARM, and it covers two different populations
        // on purpose.
        //
        // The first is the two layers this workspace knows are NOT
        // target verdicts — `FixtureConstructionFailure` and
        // `ExecutorInfrastructureFailure`. They were considered and they
        // map to no boundary, for the reason the type doc gives: the
        // target was never asked, so nothing about any boundary was
        // learned. They are not written as their own arm only because an
        // arm with this arm's body would be the same arm.
        //
        // The second is every layer added upstream tomorrow.
        // `ObservedOutcomeLayer` is `#[non_exhaustive]` and owned by
        // another package, so a new member arrives here rather than
        // breaking the build — and it answers NOTHING until somebody
        // gives it a deliberate arm above. That is the closed direction
        // to fail in: a new layer that silently matched a boundary would
        // answer rows nobody checked.
        _ => None,
    }
}

/// Whether an observed layer is the §1.5 boundary a class expected.
///
/// Exact equality through [`observed_boundary`], and the only comparison
/// either consumer is allowed to make: a layer that maps to no boundary
/// matches no expectation, and a layer that maps to one matches only that
/// one.
#[must_use]
pub fn matches_boundary(expected: EvidenceBoundary, observed: ObservedOutcomeLayer) -> bool {
    observed_boundary(observed) == Some(expected)
}

#[cfg(test)]
mod tests {
    use super::{matches_boundary, observed_boundary};
    use crate::matrix::EvidenceBoundary;
    use target_elements_conformance::protocol::ObservedOutcomeLayer;

    /// Every observed layer this workspace knows.
    const LAYERS: &[ObservedOutcomeLayer] = &[
        ObservedOutcomeLayer::FixtureConstructionFailure,
        ObservedOutcomeLayer::ExecutorInfrastructureFailure,
        ObservedOutcomeLayer::ConsensusRejectionBeforeScript,
        ObservedOutcomeLayer::ScriptPathRejection,
        ObservedOutcomeLayer::KeyPathRejection,
        ObservedOutcomeLayer::RelayPolicyRejection,
        ObservedOutcomeLayer::Accepted,
    ];

    /// Every boundary §1.5 and this workspace's erratum name.
    const BOUNDARIES: &[EvidenceBoundary] = &[
        EvidenceBoundary::SemanticRequestRejection,
        EvidenceBoundary::CompilerPlanRejection,
        EvidenceBoundary::ConstructorDerivationRejection,
        EvidenceBoundary::BackendEmissionRejection,
        EvidenceBoundary::LinkerRejection,
        EvidenceBoundary::AbiConstructionRejection,
        EvidenceBoundary::ExecutorInfrastructureFailure,
        EvidenceBoundary::ConsensusRejectionBeforeScript,
        EvidenceBoundary::KeyPathRejection,
        EvidenceBoundary::ScriptPathRejection,
        EvidenceBoundary::RelayPolicyRejection,
        EvidenceBoundary::AcceptedTransaction,
        EvidenceBoundary::ReportSemanticProjectionRejection,
    ];

    /// The mapping-table regression: EVERY cross pair is false.
    ///
    /// Not a spot check of the five that map. The whole product of layers
    /// and boundaries is walked, and the only pairs allowed to be true
    /// are the ones the mapping itself names — so a mapping that widened
    /// silently, or a boundary that started matching a second layer,
    /// fails here rather than in a census six modules away.
    #[test]
    fn every_cross_layer_boundary_pair_is_false() {
        for &layer in LAYERS {
            for &boundary in BOUNDARIES {
                let expected = observed_boundary(layer) == Some(boundary);
                assert_eq!(
                    matches_boundary(boundary, layer),
                    expected,
                    "{layer:?} against {boundary:?} did not follow the mapping",
                );
            }
        }
    }

    /// The two non-verdict layers map to no boundary at all.
    ///
    /// Including the same-named one: an infrastructure failure must not
    /// satisfy `EvidenceBoundary::ExecutorInfrastructureFailure`, because
    /// that member names what a ROW declared and a failed environment is
    /// not evidence that anything refused the row's fault.
    #[test]
    fn the_non_verdict_layers_answer_no_boundary() {
        assert_eq!(
            observed_boundary(ObservedOutcomeLayer::FixtureConstructionFailure),
            None,
        );
        assert_eq!(
            observed_boundary(ObservedOutcomeLayer::ExecutorInfrastructureFailure),
            None,
        );
        assert!(!matches_boundary(
            EvidenceBoundary::ExecutorInfrastructureFailure,
            ObservedOutcomeLayer::ExecutorInfrastructureFailure,
        ));
    }

    /// Each verdict layer maps to exactly ONE boundary, and to its own.
    #[test]
    fn each_verdict_layer_maps_to_its_own_boundary() {
        let pairs = [
            (
                ObservedOutcomeLayer::ConsensusRejectionBeforeScript,
                EvidenceBoundary::ConsensusRejectionBeforeScript,
            ),
            (
                ObservedOutcomeLayer::ScriptPathRejection,
                EvidenceBoundary::ScriptPathRejection,
            ),
            (
                ObservedOutcomeLayer::KeyPathRejection,
                EvidenceBoundary::KeyPathRejection,
            ),
            (
                ObservedOutcomeLayer::RelayPolicyRejection,
                EvidenceBoundary::RelayPolicyRejection,
            ),
            (
                ObservedOutcomeLayer::Accepted,
                EvidenceBoundary::AcceptedTransaction,
            ),
        ];
        for (layer, boundary) in pairs {
            assert_eq!(observed_boundary(layer), Some(boundary));
            let matching = BOUNDARIES
                .iter()
                .filter(|candidate| matches_boundary(**candidate, layer))
                .count();
            assert_eq!(matching, 1, "{layer:?} matched more than its own boundary");
        }
    }
}
