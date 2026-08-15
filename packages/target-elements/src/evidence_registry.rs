//! Contents of the target evidence requirements, and the reviewed
//! registry of them.
//!
//! The identities live alongside in [`crate::evidence`]; this module
//! gives each one its subject, claim class, required environment, and
//! staleness conditions.
//!
//! Every requirement in the registry is unresolved. None has been
//! evidenced against any node. The registry states what a
//! target-native test would have to demonstrate; it does not stand in
//! for having demonstrated it.

use std::collections::{BTreeMap, BTreeSet};

use crate::evidence::TargetEvidenceRequirementId;

/// What a requirement is about.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum TargetEvidenceSubject {
    /// The execution domain itself.
    ExecutionDomain,
    /// The leaf version selecting it.
    LeafVersion,
    /// The reviewed primitives.
    Primitives,
    /// The field encodings.
    Encodings,
    /// The forms a literal is pushed in.
    ///
    /// Separate from the field encodings because the two are enforced
    /// by different rules and can go stale independently: a field
    /// encoding is what a transaction carries, while a push form is how
    /// a script states a literal, and only the latter has a
    /// standardness dimension.
    LiteralPushes,
    /// Signature and sighash behavior.
    Authorization,
    /// Relative-timelock behavior.
    Timelocks,
    /// Confidential-value behavior.
    ConfidentialValues,
    /// Issuance and reissuance behavior.
    Issuance,
    /// Resource bounds.
    Resources,
}

/// What kind of claim a requirement makes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum EvidenceClaimClass {
    /// That the thing exists and is reachable at all.
    Availability,
    /// That operands and results take the described shapes.
    StackBehavior,
    /// That failures take the described effects.
    FailureBehavior,
    /// That encoded bytes take the described layout.
    EncodingShape,
    /// That a signature binds what the contract says it binds.
    AuthorizationBinding,
    /// That a lock gates on the described conditions.
    TemporalGating,
    /// That value is conserved as described.
    Conservation,
    /// That a bound is enforced at the described figure.
    ResourceBound,
}

/// Where evidence for a requirement must be produced.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum RequiredEvidenceEnvironment {
    /// Any network exercising the reviewed domain.
    AnyNetwork,
    /// A development network under the project's control.
    DevelopmentNetwork,
    /// The network a production deployment would use.
    ProductionNetwork,
}

/// What makes existing evidence stale.
///
/// Staleness is a property of *evidence*, not of the contract. A node
/// upgrade can invalidate a report without changing a single typed
/// fact in this crate, which is exactly why the contract does not
/// carry the node's revision.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum EvidenceStaleCondition {
    /// The typed contract changed.
    TargetContractChange,
    /// The network being evidenced changed.
    NetworkChange,
    /// The activation state changed.
    ActivationChange,
    /// The deployment's policy changed.
    PolicyChange,
    /// The node implementation changed.
    NodeUpgrade,
}

/// One immutable requirement for target-native evidence.
///
/// There is no pass field, no fail field, no report, no timestamp, no
/// endpoint, and no credential. Producing evidence does not change
/// this value, and neither does failing to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TargetEvidenceRequirement {
    id: TargetEvidenceRequirementId,
    subject: TargetEvidenceSubject,
    claim: EvidenceClaimClass,
    environment: RequiredEvidenceEnvironment,
    stale_on: BTreeSet<EvidenceStaleCondition>,
}

impl TargetEvidenceRequirement {
    /// States one requirement.
    #[must_use]
    pub fn new(
        id: TargetEvidenceRequirementId,
        subject: TargetEvidenceSubject,
        claim: EvidenceClaimClass,
        environment: RequiredEvidenceEnvironment,
        stale_on: impl IntoIterator<Item = EvidenceStaleCondition>,
    ) -> Self {
        Self {
            id,
            subject,
            claim,
            environment,
            stale_on: stale_on.into_iter().collect(),
        }
    }

    /// The requirement's stable identity.
    #[must_use]
    pub const fn id(&self) -> TargetEvidenceRequirementId {
        self.id
    }

    /// What it is about.
    #[must_use]
    pub const fn subject(&self) -> TargetEvidenceSubject {
        self.subject
    }

    /// What kind of claim it makes.
    #[must_use]
    pub const fn claim(&self) -> EvidenceClaimClass {
        self.claim
    }

    /// Where it must be evidenced.
    #[must_use]
    pub const fn environment(&self) -> RequiredEvidenceEnvironment {
        self.environment
    }

    /// What makes evidence for it stale.
    #[must_use]
    pub const fn stale_on(&self) -> &BTreeSet<EvidenceStaleCondition> {
        &self.stale_on
    }
}

/// The staleness conditions of a claim about the target's own code.
const CONTRACT_OR_NODE: &[EvidenceStaleCondition] = &[
    EvidenceStaleCondition::TargetContractChange,
    EvidenceStaleCondition::NodeUpgrade,
];

/// The staleness conditions of a claim about what is switched on.
const ACTIVATION: &[EvidenceStaleCondition] = &[
    EvidenceStaleCondition::TargetContractChange,
    EvidenceStaleCondition::NodeUpgrade,
    EvidenceStaleCondition::ActivationChange,
    EvidenceStaleCondition::NetworkChange,
];

/// Builds one requirement.
fn entry(
    id: TargetEvidenceRequirementId,
    subject: TargetEvidenceSubject,
    claim: EvidenceClaimClass,
    environment: RequiredEvidenceEnvironment,
    stale: &[EvidenceStaleCondition],
) -> (TargetEvidenceRequirementId, TargetEvidenceRequirement) {
    (
        id,
        TargetEvidenceRequirement::new(id, subject, claim, environment, stale.iter().copied()),
    )
}

/// Requirements about the domain and the primitives executing in it.
fn execution_requirements() -> Vec<(TargetEvidenceRequirementId, TargetEvidenceRequirement)> {
    use EvidenceClaimClass as K;
    use RequiredEvidenceEnvironment::AnyNetwork;
    use TargetEvidenceRequirementId as R;
    use TargetEvidenceSubject as S;

    vec![
        entry(
            R::TapscriptExecutionDomain,
            S::ExecutionDomain,
            K::Availability,
            AnyNetwork,
            ACTIVATION,
        ),
        entry(
            R::LeafVersionActivation,
            S::LeafVersion,
            K::Availability,
            AnyNetwork,
            ACTIVATION,
        ),
        entry(
            R::OpcodeSemantics,
            S::Primitives,
            K::Availability,
            AnyNetwork,
            CONTRACT_OR_NODE,
        ),
        entry(
            R::InputIntrospectionSemantics,
            S::Primitives,
            K::StackBehavior,
            AnyNetwork,
            CONTRACT_OR_NODE,
        ),
        entry(
            R::OutputIntrospectionSemantics,
            S::Primitives,
            K::StackBehavior,
            AnyNetwork,
            CONTRACT_OR_NODE,
        ),
        entry(
            R::TransactionIntrospectionSemantics,
            S::Primitives,
            K::StackBehavior,
            AnyNetwork,
            CONTRACT_OR_NODE,
        ),
        // A failure-behavior claim rather than a stack-behavior one:
        // the interesting thing to demonstrate about the arithmetic is
        // that overflow retains its operands.
        entry(
            R::ArithmeticSemantics,
            S::Primitives,
            K::FailureBehavior,
            AnyNetwork,
            CONTRACT_OR_NODE,
        ),
        entry(
            R::ComparisonSemantics,
            S::Primitives,
            K::StackBehavior,
            AnyNetwork,
            CONTRACT_OR_NODE,
        ),
        entry(
            R::ConversionSemantics,
            S::Primitives,
            K::FailureBehavior,
            AnyNetwork,
            CONTRACT_OR_NODE,
        ),
        entry(
            R::StreamingHashSemantics,
            S::Primitives,
            K::StackBehavior,
            AnyNetwork,
            CONTRACT_OR_NODE,
        ),
        entry(
            R::EllipticCurveSemantics,
            S::Primitives,
            K::Availability,
            AnyNetwork,
            CONTRACT_OR_NODE,
        ),
    ]
}

/// Requirements about how bytes are laid out and pushed.
fn encoding_requirements() -> Vec<(TargetEvidenceRequirementId, TargetEvidenceRequirement)> {
    use EvidenceClaimClass as K;
    use RequiredEvidenceEnvironment::AnyNetwork;
    use TargetEvidenceRequirementId as R;
    use TargetEvidenceSubject as S;

    vec![
        entry(
            R::EncodingSemantics,
            S::Encodings,
            K::EncodingShape,
            AnyNetwork,
            CONTRACT_OR_NODE,
        ),
        // The push rules straddle the two rule sets: the literal bound
        // and the well-formedness of an encoded push are consensus,
        // while the minimal form is standardness. A policy change can
        // therefore invalidate evidence for this requirement without
        // touching the contract or the node's consensus code, which is
        // why it carries a staleness condition the encoding
        // requirement does not.
        entry(
            R::PushEncodingSemantics,
            S::LiteralPushes,
            K::EncodingShape,
            AnyNetwork,
            &[
                EvidenceStaleCondition::TargetContractChange,
                EvidenceStaleCondition::NodeUpgrade,
                EvidenceStaleCondition::PolicyChange,
            ],
        ),
    ]
}

/// Requirements about authorization, value, issuance, and resources.
fn contract_requirements() -> Vec<(TargetEvidenceRequirementId, TargetEvidenceRequirement)> {
    use EvidenceClaimClass as K;
    use EvidenceStaleCondition as X;
    use RequiredEvidenceEnvironment::{AnyNetwork, DevelopmentNetwork};
    use TargetEvidenceRequirementId as R;
    use TargetEvidenceSubject as S;

    vec![
        entry(
            R::SignatureSemantics,
            S::Authorization,
            K::FailureBehavior,
            AnyNetwork,
            CONTRACT_OR_NODE,
        ),
        entry(
            R::SighashSemantics,
            S::Authorization,
            K::AuthorizationBinding,
            AnyNetwork,
            CONTRACT_OR_NODE,
        ),
        entry(
            R::RelativeTimelockSemantics,
            S::Timelocks,
            K::TemporalGating,
            AnyNetwork,
            CONTRACT_OR_NODE,
        ),
        entry(
            R::ConfidentialValueConservation,
            S::ConfidentialValues,
            K::Conservation,
            AnyNetwork,
            CONTRACT_OR_NODE,
        ),
        entry(
            R::CommitmentEquality,
            S::ConfidentialValues,
            K::Conservation,
            AnyNetwork,
            CONTRACT_OR_NODE,
        ),
        entry(
            R::IssuanceIntrospection,
            S::Issuance,
            K::StackBehavior,
            AnyNetwork,
            CONTRACT_OR_NODE,
        ),
        // The compound-proof substrate. Rearrangement and equality are
        // stack behaviour; the byte-string operations are asked for
        // their failure behaviour too, because the width bound
        // concatenation enforces on a computed result is exactly the
        // kind of rule a reviewed contract can state and a node can
        // contradict.
        entry(
            R::StackRearrangementSemantics,
            S::Primitives,
            K::StackBehavior,
            AnyNetwork,
            CONTRACT_OR_NODE,
        ),
        entry(
            R::ByteStringSemantics,
            S::Primitives,
            K::FailureBehavior,
            AnyNetwork,
            CONTRACT_OR_NODE,
        ),
        entry(
            R::VerificationSemantics,
            S::Primitives,
            K::FailureBehavior,
            AnyNetwork,
            CONTRACT_OR_NODE,
        ),
        entry(
            R::ConsensusResourceLimits,
            S::Resources,
            K::ResourceBound,
            AnyNetwork,
            CONTRACT_OR_NODE,
        ),
        // Policy is deployment-specific by definition, so it cannot be
        // evidenced on "any network": it is not a property of the
        // target at all.
        entry(
            R::PolicyResourceLimits,
            S::Resources,
            K::ResourceBound,
            DevelopmentNetwork,
            &[X::PolicyChange, X::NetworkChange, X::NodeUpgrade],
        ),
    ]
}

/// Builds the reviewed evidence-requirement registry.
pub(crate) fn reviewed_evidence_requirements()
-> BTreeMap<TargetEvidenceRequirementId, TargetEvidenceRequirement> {
    [
        execution_requirements(),
        encoding_requirements(),
        contract_requirements(),
    ]
    .into_iter()
    .flatten()
    .collect()
}
