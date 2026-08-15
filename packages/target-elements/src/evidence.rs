//! Target evidence requirement identities.
//!
//! A requirement names a claim that the static typed contract in this
//! crate does *not* establish, and that some future target-native test
//! against a real node must establish instead.
//!
//! # Requirements, never statuses
//!
//! Nothing in this module carries a pass or fail result, a report, a
//! timestamp, an endpoint, or a credential. A requirement identity is
//! immutable: producing evidence for it does not change it, and
//! failing to produce evidence for it does not change it either. A
//! mutable test status would make the target definition depend on when
//! it was last run, which is exactly the coupling this design refuses.
//!
//! So no requirement below carries a status. Whether evidence has been
//! produced about one is a question for the conformance harness that
//! produces it — `tripod-target-elements-conformance`, which
//! has recorded development native evidence — and not a field here.
//! Production target evidence remains absent.

/// A stable key naming one class of required target evidence.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum TargetEvidenceRequirementId {
    /// That the execution domain this contract describes is the domain
    /// a real node actually executes.
    TapscriptExecutionDomain,
    /// That the leaf version this contract requires is active and
    /// carries the described semantics.
    LeafVersionActivation,
    /// That a reviewed opcode's byte and gating match a real node.
    OpcodeSemantics,
    /// That the described encodings are the encodings a real node
    /// produces and accepts.
    EncodingSemantics,
    /// That the described literal-push forms are the forms a real node
    /// decodes, that its maximum literal size is the described one, and
    /// that its relayed-transaction rules require exactly the described
    /// minimal form.
    PushEncodingSemantics,
    /// That input introspection results carry the described shapes.
    InputIntrospectionSemantics,
    /// That output introspection results carry the described shapes.
    OutputIntrospectionSemantics,
    /// That transaction introspection results carry the described
    /// shapes.
    TransactionIntrospectionSemantics,
    /// That fixed-width arithmetic overflows and divides as described,
    /// including its non-aborting failure behavior.
    ArithmeticSemantics,
    /// That fixed-width comparisons order operands as described.
    ComparisonSemantics,
    /// That the conversions between the script number and fixed-width
    /// forms accept and reject exactly the described ranges.
    ConversionSemantics,
    /// That the streaming hash state serialization round-trips as
    /// described.
    StreamingHashSemantics,
    /// That signature verification accepts and rejects as described,
    /// including which failures abort and which push a false.
    SignatureSemantics,
    /// That the sighash commits to exactly the described transaction
    /// dimensions.
    SighashSemantics,
    /// That relative timelocks gate on exactly the described sequence
    /// and transaction-version conditions.
    RelativeTimelockSemantics,
    /// That the elliptic-curve checks verify the described relations.
    EllipticCurveSemantics,
    /// That whole-transaction value conservation holds over the
    /// described value classes.
    ConfidentialValueConservation,
    /// That the described commitment-equality mechanism exists and
    /// behaves as described.
    CommitmentEquality,
    /// That issuance and reissuance fields are introspectable as
    /// described.
    IssuanceIntrospection,
    /// That the described consensus resource limits are the limits a
    /// real node enforces.
    ConsensusResourceLimits,
    /// That the described policy resource limits are the limits a real
    /// deployment enforces.
    PolicyResourceLimits,
}

impl TargetEvidenceRequirementId {
    /// The complete census of requirement identities.
    pub const ALL: &'static [Self] = &[
        Self::TapscriptExecutionDomain,
        Self::LeafVersionActivation,
        Self::OpcodeSemantics,
        Self::EncodingSemantics,
        Self::PushEncodingSemantics,
        Self::InputIntrospectionSemantics,
        Self::OutputIntrospectionSemantics,
        Self::TransactionIntrospectionSemantics,
        Self::ArithmeticSemantics,
        Self::ComparisonSemantics,
        Self::ConversionSemantics,
        Self::StreamingHashSemantics,
        Self::SignatureSemantics,
        Self::SighashSemantics,
        Self::RelativeTimelockSemantics,
        Self::EllipticCurveSemantics,
        Self::ConfidentialValueConservation,
        Self::CommitmentEquality,
        Self::IssuanceIntrospection,
        Self::ConsensusResourceLimits,
        Self::PolicyResourceLimits,
    ];
}
