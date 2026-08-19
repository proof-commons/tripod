//! Why each newly public fact on a candidate path is public.
//!
//! Guide 11 §14.4 requires that every fact a candidate makes public
//! carries a typed reason, and that three kinds of reason stay apart:
//! semantic disclosure, target-safety disclosure, and deployment-policy
//! disclosure. The distinction is the guide's central honesty
//! requirement for this phase — an explicit-only policy may add
//! deployment-policy disclosure without claiming the semantic relation
//! intrinsically requires explicit encoding, and a path that reported
//! the second while doing the first would be describing a policy choice
//! as a mathematical fact.
//!
//! # The reuse the guide prefers is not available, for two reasons
//!
//! §14.4 says to prefer existing realization-owned disclosure types
//! where they already express the required distinction, and
//! `tripod-realization` does own one: its `DisclosureReason`
//! carries `PublicState`, `PublicEvent`, `PublicInterface`,
//! `PermissionlessConstructibility`, `TargetSafety`, and
//! `DeploymentPolicy`. The distinction is there. Reuse still does not
//! follow, and the reasons are independent — either alone would settle
//! it:
//!
//! - **The dependency is refused.** This package names exactly three
//!   first-party dependencies, and `realization` is absent by decision
//!   rather than by oversight: naming it would import a publication
//!   boundary this harness has no asset for. Guide 11 §6.1 and §6.2 draw
//!   the same line, and a disclosure vocabulary is not a reason to
//!   redraw it.
//! - **The payloads name identities that do not exist here.** Two of
//!   realization's variants carry an `OperationId` and a `RelationId`.
//!   A conformance run has neither: it has a transaction, a target
//!   verdict, and a candidate. Importing the type would mean inventing
//!   identities to fill it, and an invented identity in an evidence
//!   record is worse than a duplicated enum.
//!
//! So the vocabulary here is §14.4's own, stated in this package, and
//! the correspondence with realization's is deliberate: the two express
//! the same distinction for the same reason, and a later wave that has
//! both a realization identity and a target run may unify them.
//!
//! # `PublicInterface` is not carried
//!
//! Realization's enum has it and §14.4's does not. It is a realization
//! concern — a fact is public because an interface publishes it — and
//! nothing on a candidate path discloses for that reason. Adding it
//! would be vocabulary no disclosure could ever use.

use serde::{Deserialize, Serialize};

/// Why one newly public fact is public.
///
/// Exactly Guide 11 §14.4's set. The variants are ordered as the guide
/// orders them, so a reader can check the enum against the text without
/// matching prose.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum DeclassificationReason {
    /// The fact is part of a public state the protocol already publishes.
    PublicState,
    /// The fact is part of a public event the protocol already emits.
    PublicEvent,
    /// An unrelated party could not construct a successor without it.
    PermissionlessConstructibility,
    /// The boundary's own arithmetic needs the value in the clear.
    BoundaryArithmetic,
    /// The target cannot be shown safe without the fact being public.
    TargetSafety,
    /// A deployment chose a representation that publishes it.
    ///
    /// The load-bearing variant for this phase. It says the fact is
    /// public because of a policy, and it says nothing at all about
    /// whether the semantic relation needs it public — which is the
    /// separation §14.4 exists to hold.
    DeploymentPolicy,
}

impl DeclassificationReason {
    /// The complete census of reasons.
    pub const ALL: &'static [Self] = &[
        Self::PublicState,
        Self::PublicEvent,
        Self::PermissionlessConstructibility,
        Self::BoundaryArithmetic,
        Self::TargetSafety,
        Self::DeploymentPolicy,
    ];

    /// Whether this reason asserts that the semantics require disclosure.
    ///
    /// # The question a reader of a report actually has
    ///
    /// A disclosure list answers "what became public". This answers the
    /// harder one: "would a different deployment have kept it private?".
    /// A reason that is not semantic leaves that door open, and a report
    /// that could not distinguish the two would let a policy choice
    /// harden into an apparent necessity across waves.
    #[must_use]
    pub const fn is_semantic_necessity(&self) -> bool {
        match self {
            Self::PublicState
            | Self::PublicEvent
            | Self::PermissionlessConstructibility
            | Self::BoundaryArithmetic => true,
            Self::TargetSafety | Self::DeploymentPolicy => false,
        }
    }
}

/// One fact a candidate path makes public, and why.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Declassification {
    /// What became public.
    pub fact: String,
    /// Why.
    pub reason: DeclassificationReason,
    /// What a deployment that did not want this public would have to do.
    ///
    /// Carried for the two non-semantic reasons and empty for the rest.
    /// A deployment-policy disclosure with no stated alternative is
    /// indistinguishable from a necessity, which is the confusion §14.4
    /// is drawn to prevent, so this module's tests require one.
    pub avoidable_by: String,
}

/// What the normalization path makes public, and why.
///
/// # Every entry is deployment policy, and that is the finding
///
/// Normalization publishes an amount. It publishes one because the
/// public representation chosen for the boundary is explicit, and the
/// choice is a deployment's: Guide 11 §9.3 records that an explicit-only
/// policy cannot be described as disclosure-minimal, and §14.4 requires
/// that the policy not be dressed as arithmetic. Nothing in the
/// normalization relation needs the amount in the clear — the target
/// conserves value over commitments perfectly well without it, which the
/// §8.4 matrix established — so no entry here claims
/// [`DeclassificationReason::BoundaryArithmetic`], and none may be added
/// without evidence that the relation itself cannot close otherwise.
#[must_use]
pub fn normalization_declassifications() -> Vec<Declassification> {
    vec![
        Declassification {
            fact: "the semantic amount of the normalized output".to_owned(),
            reason: DeclassificationReason::DeploymentPolicy,
            avoidable_by: "publishing the output as a public committed \
                 representation instead, which keeps the commitment and \
                 publishes an opening; that representation is deferred against \
                 the three reviewed opening blockers rather than unavailable in \
                 principle"
                .to_owned(),
        },
        Declassification {
            fact: "the explicit asset identity of the normalized output".to_owned(),
            reason: DeclassificationReason::DeploymentPolicy,
            avoidable_by: "the same public committed representation; the target \
                 supports a blinded asset generator, and the §8.4 matrix \
                 exercises one on every balanced confidential row"
                .to_owned(),
        },
        Declassification {
            fact: "that a normalization occurred at all, and which coin it \
                   consumed"
                .to_owned(),
            reason: DeclassificationReason::DeploymentPolicy,
            avoidable_by: "nothing available on this target: a spend is public \
                 chain data. Recorded as policy rather than as necessity \
                 because the necessity belongs to the target's transaction \
                 model rather than to the normalization relation, and a later \
                 target that hid spends would not inherit this disclosure"
                .to_owned(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_normalization_path_claims_no_semantic_necessity() {
        // Guide 11 §14.4's own example, made checkable: an explicit-only
        // policy adds deployment-policy disclosure and must not be read as
        // claiming the relation intrinsically requires explicit encoding.
        for entry in normalization_declassifications() {
            assert!(!entry.reason.is_semantic_necessity(), "{}", entry.fact);
            assert_eq!(entry.reason, DeclassificationReason::DeploymentPolicy);
        }
    }

    #[test]
    fn a_non_semantic_disclosure_states_its_alternative() {
        for entry in normalization_declassifications() {
            assert!(!entry.avoidable_by.is_empty(), "{}", entry.fact);
        }
    }

    #[test]
    fn the_amount_is_disclosed_and_it_is_the_first_thing_listed() {
        // The amount is the disclosure a reader comes for. If it ever
        // stopped being listed, the report would be describing a path that
        // publishes an amount without saying so.
        let entries = normalization_declassifications();
        assert!(entries[0].fact.contains("amount"));
    }

    #[test]
    fn the_census_is_the_guides_six() {
        assert_eq!(DeclassificationReason::ALL.len(), 6);
        assert!(DeclassificationReason::ALL.contains(&DeclassificationReason::BoundaryArithmetic));
    }
}
