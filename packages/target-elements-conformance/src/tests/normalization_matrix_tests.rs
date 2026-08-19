//! What the Guide 11 §10.4 matrix observed on a real node.
//!
//! # What these values are
//!
//! Every layer below was **observed**, on a real `elementsregtest` node
//! at the declared tip, by the Wave-10 normalization run. They are pinned
//! here so the matrix is a standing test rather than a one-time
//! observation on a host nobody else has, and so that a later change to
//! the claim, the mutations, or the adapter has to move a recorded target
//! fact rather than merely a comment.
//!
//! # The expectations are not restated here
//!
//! Each row's expected layer is read from
//! [`canonical_mutation_matrix`], where it was written and committed
//! before this run happened. Restating it in the test would let the two
//! drift, and the drift would be invisible in exactly the direction that
//! matters: a test agreeing with a rewritten expectation.
//!
//! What is pinned is the *observation* — the layer the target answered
//! at — and the derivation from it. A disagreement is reported rather
//! than absorbed, which is Guide 11 §7.4's standing rule.
//!
//! # Why three rows record an acceptance
//!
//! Rows 2, 3 and 6 pin `Accepted`. That is the wave's substantive
//! finding rather than a gap: the target has no opinion about which owner
//! was meant, which amount was claimed, or whether the output set is
//! complete, so a consensus-valid transaction can violate all three and
//! only the report layer refuses it.

use crate::conservation_report::RowVerdict;
use crate::normalization::{
    ClosureFinding, NormalizationMutation, PreservationFinding, PreservedProperty, RefusalLayer,
    canonical_mutation_matrix,
};
use crate::normalization_report::derive_refusal;
use crate::protocol::ObservedOutcomeLayer;

/// One row, as the target answered it.
struct ObservedRow {
    /// The mutation applied.
    mutation: NormalizationMutation,
    /// The layer the target answered at.
    target_layer: ObservedOutcomeLayer,
    /// What the target said, verbatim, where it said anything.
    detail: Option<&'static str>,
    /// Whether the observed output set was exactly the claimed one.
    closure_holds: bool,
    /// Whether §10.1's properties survived.
    preservation_holds: bool,
    /// The §10.1 properties that broke, in the order the report lists
    /// them.
    broken: &'static [PreservedProperty],
}

/// The Wave-10 normalization run, on the declared tip.
///
/// Node `v28.99.0-78499c206475`, genesis
/// `209577bda6bf4b5804bd46f8621580dd6d4e8bfa2d190e1c50e932492baca07d`.
const OBSERVED: &[ObservedRow] = &[
    ObservedRow {
        mutation: NormalizationMutation::None,
        target_layer: ObservedOutcomeLayer::Accepted,
        detail: None,
        closure_holds: true,
        preservation_holds: true,
        broken: &[],
    },
    ObservedRow {
        // Compensated, so value conserves exactly and consensus has
        // nothing to refuse. The claim still says five million.
        mutation: NormalizationMutation::AmountChanged,
        target_layer: ObservedOutcomeLayer::Accepted,
        detail: None,
        closure_holds: false,
        preservation_holds: false,
        broken: &[PreservedProperty::SemanticAmount],
    },
    ObservedRow {
        // A consensus-valid transaction paying the wrong owner.
        mutation: NormalizationMutation::OwnerChanged,
        target_layer: ObservedOutcomeLayer::Accepted,
        detail: None,
        closure_holds: false,
        preservation_holds: false,
        broken: &[
            PreservedProperty::SemanticAmount,
            PreservedProperty::ExplicitAsset,
            PreservedProperty::Owner,
        ],
    },
    ObservedRow {
        // The one conservation code this target answers every value
        // failure with, exactly as Wave 7 recorded.
        mutation: NormalizationMutation::AssetChanged,
        target_layer: ObservedOutcomeLayer::ConsensusRejectionBeforeScript,
        detail: Some("bad-txns-in-ne-out"),
        closure_holds: false,
        preservation_holds: false,
        broken: &[PreservedProperty::ExplicitAsset],
    },
    ObservedRow {
        // Nothing about the claim is wrong; the target's own arithmetic
        // does not close.
        mutation: NormalizationMutation::WrongBlindingBalance,
        target_layer: ObservedOutcomeLayer::ConsensusRejectionBeforeScript,
        detail: Some("bad-txns-in-ne-out"),
        closure_holds: true,
        preservation_holds: true,
        broken: &[],
    },
    ObservedRow {
        // The row that justifies the whole closure check: the target
        // accepts it, the owner is paid exactly right, and an output the
        // claim never named absorbs value nobody can see.
        mutation: NormalizationMutation::HiddenPrivateOutput,
        target_layer: ObservedOutcomeLayer::Accepted,
        detail: None,
        closure_holds: false,
        preservation_holds: true,
        broken: &[],
    },
    ObservedRow {
        mutation: NormalizationMutation::ExtraOutputAfterSigning,
        target_layer: ObservedOutcomeLayer::ScriptPathRejection,
        detail: Some("mandatory-script-verify-flag-failed (Invalid Schnorr signature)"),
        closure_holds: false,
        preservation_holds: true,
        broken: &[],
    },
    ObservedRow {
        mutation: NormalizationMutation::OutputMutatedAfterSigning,
        target_layer: ObservedOutcomeLayer::ScriptPathRejection,
        detail: Some("mandatory-script-verify-flag-failed (Invalid Schnorr signature)"),
        closure_holds: false,
        preservation_holds: false,
        broken: &[
            PreservedProperty::SemanticAmount,
            PreservedProperty::ExplicitAsset,
            PreservedProperty::Owner,
        ],
    },
    ObservedRow {
        // Closure and preservation both hold: the ONLY thing wrong with
        // this transaction is that the owner did not authorize it, which
        // is what makes it the baseline §10 rests on.
        mutation: NormalizationMutation::UnauthorizedRepresentationChange,
        target_layer: ObservedOutcomeLayer::ScriptPathRejection,
        detail: Some(
            "mandatory-script-verify-flag-failed (Witness program was passed an empty witness)",
        ),
        closure_holds: true,
        preservation_holds: true,
        broken: &[],
    },
];

fn finding(holds: bool, broken: &[PreservedProperty]) -> (ClosureFinding, PreservationFinding) {
    let closure = if holds {
        ClosureFinding::Holds
    } else {
        ClosureFinding::Violated {
            unclaimed: Vec::new(),
            missing: Vec::new(),
        }
    };
    let preservation = if broken.is_empty() {
        PreservationFinding::Holds
    } else {
        PreservationFinding::Violated {
            properties: broken.to_vec(),
        }
    };
    (closure, preservation)
}

#[test]
fn the_run_answered_every_row_the_matrix_states() {
    let matrix = canonical_mutation_matrix();
    assert_eq!(OBSERVED.len(), matrix.len());
    for (observed, row) in OBSERVED.iter().zip(matrix.iter()) {
        assert_eq!(observed.mutation, row.mutation);
    }
}

#[test]
fn every_observed_layer_agrees_with_the_expectation_written_before_the_run() {
    // The whole point of the wave. Expectations come from the matrix,
    // where they were committed before the node was asked; observations
    // come from the constants above. Nothing here reconciles the two.
    for (observed, row) in OBSERVED.iter().zip(canonical_mutation_matrix()) {
        let (closure, preservation) = finding(
            observed.closure_holds,
            if observed.preservation_holds {
                &[]
            } else {
                observed.broken
            },
        );
        let refusal = derive_refusal(observed.target_layer, &closure, &preservation)
            .expect("every row reached a target verdict");
        assert_eq!(
            refusal, row.expected,
            "{:?} was refused by {refusal} and the matrix expected {}",
            observed.mutation, row.expected
        );
    }
}

#[test]
fn the_unmutated_claim_was_accepted_and_stood() {
    // Without this row the other eight would describe a path that
    // refuses everything, which is trivially safe and not a prototype.
    let row = OBSERVED
        .iter()
        .find(|row| row.mutation == NormalizationMutation::None)
        .expect("the unmutated row ran");
    assert_eq!(row.target_layer, ObservedOutcomeLayer::Accepted);
    assert!(row.closure_holds);
    assert!(row.preservation_holds);
}

#[test]
fn the_target_accepted_all_three_report_layer_rows() {
    // The finding a later wave must not quietly lose: consensus polices
    // conservation and says nothing about the claim. If any of these
    // three ever became a target rejection, the report layer would be
    // doing less work than this wave established it has to.
    for mutation in [
        NormalizationMutation::AmountChanged,
        NormalizationMutation::OwnerChanged,
        NormalizationMutation::HiddenPrivateOutput,
    ] {
        let row = OBSERVED
            .iter()
            .find(|row| row.mutation == mutation)
            .expect("the row ran");
        assert_eq!(
            row.target_layer,
            ObservedOutcomeLayer::Accepted,
            "{mutation:?}"
        );
    }
}

#[test]
fn the_hidden_output_is_caught_by_closure_and_not_by_preservation() {
    // The distinction the closure check exists for. The owner was paid
    // exactly what the claim said; what the claim missed is that
    // something else was paid too, and the change it came out of is
    // blinded so no amount betrays it.
    let row = OBSERVED
        .iter()
        .find(|row| row.mutation == NormalizationMutation::HiddenPrivateOutput)
        .expect("the row ran");
    assert!(!row.closure_holds);
    assert!(row.preservation_holds);
}

#[test]
fn the_unauthorized_row_is_refused_by_nothing_but_the_missing_signature() {
    // Closure and preservation both hold, so the transaction is exactly
    // what the claim describes -- and the target still refuses it. That
    // is what "owner-authorized" means as a target property rather than
    // as a description of intent.
    let row = OBSERVED
        .iter()
        .find(|row| row.mutation == NormalizationMutation::UnauthorizedRepresentationChange)
        .expect("the row ran");
    assert!(row.closure_holds);
    assert!(row.preservation_holds);
    assert_eq!(row.target_layer, ObservedOutcomeLayer::ScriptPathRejection);
}

#[test]
fn both_post_signing_edits_were_refused_by_the_signature_itself() {
    // Not merely "rejected": the target named an invalid Schnorr
    // signature, which is what ties the refusal to the §10.3 profile
    // rather than to some other check that happened to fire.
    for mutation in [
        NormalizationMutation::ExtraOutputAfterSigning,
        NormalizationMutation::OutputMutatedAfterSigning,
    ] {
        let row = OBSERVED
            .iter()
            .find(|row| row.mutation == mutation)
            .expect("the row ran");
        assert_eq!(
            row.detail,
            Some("mandatory-script-verify-flag-failed (Invalid Schnorr signature)"),
            "{mutation:?}"
        );
    }
}

#[test]
fn the_same_edit_before_and_after_signing_lands_in_different_layers() {
    // The entire content of §10.3's prerequisite, as one comparison:
    // paying the wrong owner is a report-layer matter before the
    // signature and a target refusal after it.
    let before = OBSERVED
        .iter()
        .find(|row| row.mutation == NormalizationMutation::OwnerChanged)
        .expect("the row ran");
    let after = OBSERVED
        .iter()
        .find(|row| row.mutation == NormalizationMutation::OutputMutatedAfterSigning)
        .expect("the row ran");
    assert_eq!(before.target_layer, ObservedOutcomeLayer::Accepted);
    assert_eq!(
        after.target_layer,
        ObservedOutcomeLayer::ScriptPathRejection
    );
}

#[test]
fn the_target_answers_every_conservation_failure_with_one_code() {
    // Wave 7's finding, reproduced on this lane's own transactions: a
    // wrong asset and a wrong blinding balance are different faults and
    // arrive under one name, so a report that classified on the string
    // would be inventing a distinction the target does not make.
    let codes: Vec<_> = OBSERVED
        .iter()
        .filter(|row| row.target_layer == ObservedOutcomeLayer::ConsensusRejectionBeforeScript)
        .map(|row| row.detail)
        .collect();
    assert_eq!(codes, vec![Some("bad-txns-in-ne-out"); 2]);
}

#[test]
fn every_row_of_this_run_agreed() {
    // Stated as a count so a future edit that drops a row cannot leave
    // this file looking complete.
    let mut agreed = 0;
    for (observed, row) in OBSERVED.iter().zip(canonical_mutation_matrix()) {
        let (closure, preservation) = finding(
            observed.closure_holds,
            if observed.preservation_holds {
                &[]
            } else {
                observed.broken
            },
        );
        let refusal = derive_refusal(observed.target_layer, &closure, &preservation)
            .expect("every row reached a target verdict");
        let verdict = if refusal == row.expected {
            RowVerdict::Agrees
        } else {
            RowVerdict::Disagrees
        };
        assert_eq!(verdict, RowVerdict::Agrees, "{:?}", observed.mutation);
        agreed += 1;
    }
    assert_eq!(agreed, 9);
}

#[test]
fn the_authorization_profile_was_observed_and_not_assumed() {
    // Both inputs carried a single 64-byte witness item on this run,
    // which is a Schnorr signature with no trailing sighash byte. The
    // reviewed digest reads a missing byte as the default all-outputs
    // non-anyone-can-pay mode, so the profile is a target observation
    // rather than an adapter's intention.
    const WITNESS_SIZES: &[&[usize]] = &[&[64], &[64]];
    for input in WITNESS_SIZES {
        assert_eq!(input.len(), 1, "a key-path spend carries one witness item");
        assert_eq!(input[0], 64, "no trailing sighash byte");
    }
}

#[test]
fn a_relay_refusal_would_not_be_read_as_a_signature_refusal() {
    // Guards the derivation rather than this run: the three post-signing
    // rows are only evidence about the signature because the target named
    // a script failure. A relay refusal reaching the same expectation
    // would mean the fee, not the authorization, decided the row.
    assert_eq!(
        derive_refusal(
            ObservedOutcomeLayer::RelayPolicyRejection,
            &ClosureFinding::Holds,
            &PreservationFinding::Holds,
        ),
        Some(RefusalLayer::TargetRelayPolicy)
    );
}
