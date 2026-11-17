//! Exact local refusal boundaries, separate from absent evidence.
use super::{Address, ByteSource, EventId};
use transaction::TransactionRefusal;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Boundary {
    EvidenceConsistency,
    ContentAddressing,
    UnsupportedForm,
    Authentication,
    Clause,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Refusal {
    DuplicateEvent(EventId),
    ConflictingEvent(EventId),
    ConflictingAlias(Address),
    UnassociatedBytes(EventId),
    MissingOutput {
        event: EventId,
        input: usize,
        address: Address,
        position: u32,
    },
    DuplicateConsumption {
        address: Address,
        position: u32,
    },
    Cycle,
    MalformedBytes {
        subject: ByteSource,
        source: TransactionRefusal,
    },
    UnsupportedForm {
        subject: ByteSource,
        source: TransactionRefusal,
    },
    NonCanonical(ByteSource),
    ContentAddress {
        subject: ByteSource,
        offered: Address,
        computed: Address,
    },
    ConflictingParent(Address),
    InvalidTriple(super::Triple),
    Recipe(crate::keyed_fixed_constructor::KeyedFixedConstructorRefusal),
    TargetDefinition,
    LaterBirth {
        event: EventId,
        input: usize,
        address: Address,
        position: u32,
    },
    WrongParentPair {
        event: EventId,
        input: usize,
    },
    WrongSuccessor {
        event: EventId,
        input: usize,
    },
    WrongVersion {
        event: EventId,
        input: usize,
    },
    WrongSuccessorPair {
        event: EventId,
        input: usize,
    },
    ScriptSig {
        event: EventId,
        input: usize,
    },
    WitnessShape {
        event: EventId,
        input: usize,
    },
    UncommittedLeaf {
        event: EventId,
        input: usize,
    },
    ControlVersion {
        event: EventId,
        input: usize,
    },
    WrongParity {
        event: EventId,
        input: usize,
    },
    ControlRecipe {
        event: EventId,
        input: usize,
    },
}

impl Refusal {
    #[must_use]
    pub const fn boundary(&self) -> Boundary {
        match self {
            Self::DuplicateEvent(_)
            | Self::ConflictingEvent(_)
            | Self::ConflictingAlias(_)
            | Self::UnassociatedBytes(_)
            | Self::MissingOutput { .. }
            | Self::DuplicateConsumption { .. }
            | Self::Cycle
            | Self::ConflictingParent(_) => Boundary::EvidenceConsistency,
            Self::MalformedBytes { .. } | Self::NonCanonical(_) | Self::ContentAddress { .. } => {
                Boundary::ContentAddressing
            }
            Self::UnsupportedForm { .. } => Boundary::UnsupportedForm,
            Self::InvalidTriple(_)
            | Self::Recipe(_)
            | Self::TargetDefinition
            | Self::ScriptSig { .. }
            | Self::WitnessShape { .. }
            | Self::UncommittedLeaf { .. }
            | Self::ControlVersion { .. }
            | Self::WrongParity { .. }
            | Self::ControlRecipe { .. } => Boundary::Authentication,
            Self::LaterBirth { .. }
            | Self::WrongParentPair { .. }
            | Self::WrongSuccessor { .. }
            | Self::WrongVersion { .. }
            | Self::WrongSuccessorPair { .. } => Boundary::Clause,
        }
    }
}
