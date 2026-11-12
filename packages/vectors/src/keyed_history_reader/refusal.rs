//! Observation boundaries, never relabelled native covenant refusals.
use super::evidence::{Address, ByteSource, EventId};
use transaction::TransactionRefusal;

/// Named boundary of a conformance observation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Boundary {
    /// Contradictory declarations, dependencies or consumption.
    EvidenceConsistency,
    /// Canonical bytes and their offered address.
    ContentAddressing,
    /// Decoder's explicitly unsupported serialization envelope.
    UnsupportedForm,
    /// Public recipe construction or key association.
    KeyAuthentication,
    /// Birth position observation, not native execution.
    BornFirst,
    /// Successor version/program observation, not native execution.
    Continuation,
    /// Authenticated script/control path.
    CommitmentAuthentication,
    /// Bounded model evaluation after authentication.
    ModelEvaluation,
}
/// One condition per observation refusal; incomplete evidence is a membership outcome.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// Repeated identical name/address declaration.
    DuplicateEvent(EventId),
    /// One name offered with different content addresses.
    ConflictingEvent(EventId),
    /// Different names alias one content address.
    ConflictingAlias(Address),
    /// Bytes have no offered identity association.
    UnassociatedBytes(EventId),
    /// Available parent does not contain the named output.
    MissingOutput {
        event: EventId,
        input: usize,
        address: Address,
        position: u32,
    },
    /// One outpoint is consumed more than once in the supplied history.
    DuplicateConsumption { address: Address, position: u32 },
    /// Dependency graph is cyclic before content authentication.
    Cycle,
    /// Byte syntax fails canonical decoding.
    MalformedBytes {
        subject: ByteSource,
        source: TransactionRefusal,
    },
    /// The decoder refuses issuance, peg-in or unsupported proof serialization.
    UnsupportedForm {
        subject: ByteSource,
        source: TransactionRefusal,
    },
    /// A successful decode does not reproduce the offered bytes.
    NonCanonical(ByteSource),
    /// Stripped serialization does not hash to the offered address.
    ContentAddress {
        subject: ByteSource,
        offered: Address,
        computed: Address,
    },
    /// Supplied funding parent bytes conflict at one offered address.
    ConflictingParent(Address),
    /// Public recipe construction failed in the permitted substrate.
    Recipe(crate::keyed_constructor::KeyedConstructorRefusal),
    /// Reviewed target definition could not be obtained.
    TargetDefinition,
    /// A later keyed output was consumed.
    LaterBirth {
        event: EventId,
        input: usize,
        address: Address,
        position: u32,
    },
    /// Output zero does not equal the consumed program.
    WrongSuccessor { event: EventId, input: usize },
    /// Output zero has a different witness version.
    WrongVersion { event: EventId, input: usize },
    /// Covenant witness is not exactly leaf plus control.
    WitnessShape { event: EventId, input: usize },
    /// Witness leaf differs from the canonical committed leaf.
    UncommittedLeaf { event: EventId, input: usize },
    /// Control byte has the wrong leaf version.
    ControlVersion { event: EventId, input: usize },
    /// Control byte has the wrong tweaked-key parity.
    WrongParity { event: EventId, input: usize },
    /// Internal key, path or control length differs.
    ControlRecipe { event: EventId, input: usize },
    /// Bounded byte model disagrees after the checked canonical schedule.
    Model {
        event: EventId,
        input: usize,
        kind: &'static str,
    },
}
impl Refusal {
    /// Observation owner, not a native script classification.
    #[must_use]
    pub const fn boundary(&self) -> Boundary {
        match self {
            Self::MalformedBytes { .. } | Self::NonCanonical(_) | Self::ContentAddress { .. } => {
                Boundary::ContentAddressing
            }
            Self::UnsupportedForm { .. } => Boundary::UnsupportedForm,
            Self::Recipe(_) | Self::TargetDefinition => Boundary::KeyAuthentication,
            Self::LaterBirth { .. } => Boundary::BornFirst,
            Self::WrongSuccessor { .. } | Self::WrongVersion { .. } => Boundary::Continuation,
            Self::WitnessShape { .. }
            | Self::UncommittedLeaf { .. }
            | Self::ControlVersion { .. }
            | Self::WrongParity { .. }
            | Self::ControlRecipe { .. } => Boundary::CommitmentAuthentication,
            Self::Model { .. } => Boundary::ModelEvaluation,
            Self::DuplicateEvent(_)
            | Self::ConflictingEvent(_)
            | Self::ConflictingAlias(_)
            | Self::UnassociatedBytes(_)
            | Self::MissingOutput { .. }
            | Self::DuplicateConsumption { .. }
            | Self::Cycle
            | Self::ConflictingParent(_) => Boundary::EvidenceConsistency,
        }
    }
    /// Stable structured trace field.
    #[must_use]
    pub const fn refusal_kind(&self) -> &'static str {
        match self {
            Self::DuplicateEvent(_) => "duplicate_event",
            Self::ConflictingEvent(_) => "conflicting_event",
            Self::ConflictingAlias(_) => "conflicting_alias",
            Self::UnassociatedBytes(_) => "unassociated_bytes",
            Self::MissingOutput { .. } => "missing_output",
            Self::DuplicateConsumption { .. } => "duplicate_consumption",
            Self::Cycle => "cycle",
            Self::MalformedBytes { .. } => "malformed_bytes",
            Self::UnsupportedForm { .. } => "unsupported_form",
            Self::NonCanonical(_) => "noncanonical",
            Self::ContentAddress { .. } => "content_address",
            Self::ConflictingParent(_) => "conflicting_parent",
            Self::Recipe(_) => "recipe",
            Self::TargetDefinition => "target_definition",
            Self::LaterBirth { .. } => "later_birth",
            Self::WrongSuccessor { .. } => "wrong_successor",
            Self::WrongVersion { .. } => "wrong_version",
            Self::WitnessShape { .. } => "witness_shape",
            Self::UncommittedLeaf { .. } => "uncommitted_leaf",
            Self::ControlVersion { .. } => "control_version",
            Self::WrongParity { .. } => "wrong_parity",
            Self::ControlRecipe { .. } => "control_recipe",
            Self::Model { .. } => "model",
        }
    }
}
