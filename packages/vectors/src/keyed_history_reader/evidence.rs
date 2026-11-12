//! Internal conformance report vocabulary; no maintained interchange format.
use std::collections::{BTreeMap, BTreeSet};

/// Public semantic identity, not a curve point.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Key(
    /// Exact public semantic key bytes.
    pub [u8; 32],
);
impl From<[u8; 32]> for Key {
    fn from(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}
/// Fixture association name, checked against content addressing.
pub type EventId = u32;
/// Stripped transaction digest in internal byte order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Address(
    /// Exact stripped transaction digest bytes.
    pub [u8; 32],
);
impl From<[u8; 32]> for Address {
    fn from(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}
/// Identity of untrusted bytes at the decoding/content boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ByteSource {
    /// Bytes offered under a history event name.
    Event(EventId),
    /// Retrieval bytes offered under their content address.
    Retrieved(Address),
}
/// A byte-derived output position under a checked association.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Output {
    /// Creating event name.
    pub event: EventId,
    /// Actual serialized output index.
    pub position: usize,
}
/// A key-qualified first output; no membership assertion is implied by the type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Handle {
    /// Authenticated recipe key.
    pub key: Key,
    /// Output identity.
    pub output: Output,
}
/// Presence of hypothetical fixture assertions, never proof of their truth.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Assertions {
    /// Addresses for which inclusion has been asserted externally.
    pub inclusion: BTreeSet<Address>,
    /// Addresses for which retrieval has been asserted externally.
    pub retrieval: BTreeSet<Address>,
}
/// Untrusted byte/evidence offers. No graph, expected sets or input-role flags.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Input {
    /// Supplied event bytes.
    pub bytes: BTreeMap<EventId, Vec<u8>>,
    /// Event-name/content-address offers; duplicates are checked.
    pub association: Vec<(EventId, Address)>,
    /// Public recipe keys under examination.
    pub keys: BTreeSet<Key>,
    /// Retrieval bytes by offered address; no role, event or selected-output assertion.
    pub retrieved: BTreeMap<Address, Vec<u8>>,
    /// Names at the fixture cut; never SOURCE assertions.
    pub boundary_premises: BTreeSet<EventId>,
    /// Explicit evidence presence, with truth remaining an external premise.
    pub assertions: Assertions,
}
/// Evidence still needed for membership reconstruction.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Missing {
    /// Association has no bytes for this event.
    Event(EventId),
    /// Input parent is absent; the event cannot be classified as SOURCE.
    Parent {
        event: EventId,
        input: usize,
        address: Address,
        position: u32,
    },
    /// No inclusion assertion was supplied for available bytes.
    Inclusion(Address),
    /// No retrieval assertion was supplied for available bytes.
    Retrieval(Address),
    /// A recognized keyed parent has no associated history event.
    Membership {
        event: EventId,
        input: usize,
        address: Address,
    },
}
/// Complete only relative to the supplied evidence and named external premises.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Membership {
    /// Independently reconstructed SOURCE records.
    Complete(BTreeSet<Handle>),
    /// No complete set is published while any dependency is unresolved.
    Incomplete(BTreeSet<Missing>),
}
/// All four remain external even when fixture assertions are present.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ParentFact {
    Inclusion,
    Unspentness,
    Retrieval,
    Finality,
}
/// Per-input external obligation, not a fact established by local hashes.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Obligation {
    /// Spender event.
    pub event: EventId,
    /// Actual input position.
    pub input: usize,
    /// Actual parent digest.
    pub address: Address,
    /// Actual spent output index.
    pub position: u32,
    /// External fact still owed.
    pub fact: ParentFact,
}
/// Clause identity independent of producer assignments.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Relation {
    BornFirst,
    ContinueFirst,
}
/// Content-addressed member identity, independent of fixture aliases.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Member {
    /// Public recipe key.
    pub key: Key,
    /// Actual spent parent address.
    pub address: Address,
    /// Actual spent output position.
    pub position: u32,
}
/// Local model evidence from an authenticated leaf and serialized context.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelationEvidence {
    /// Spender event.
    pub event: EventId,
    /// Actual member input index.
    pub input: usize,
    /// Consumed handle identity.
    pub member: Member,
    /// Evaluated clause.
    pub relation: Relation,
    /// Exact leaf extracted from the input witness.
    pub leaf: Vec<u8>,
    /// Exact authenticated control bytes.
    pub control: Vec<u8>,
    /// This clause fragment's primitive count from the bounded byte model.
    pub primitive_count: u64,
    /// Combined model stack peak.
    pub peak: usize,
    /// Maximum model element width.
    pub maximum_width: usize,
}
/// Whole-leaf counters once per authenticated member, not additive clause costs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LeafEvidence {
    /// Spender event.
    pub event: EventId,
    /// Actual member input index.
    pub input: usize,
    /// Authenticated consumed identity.
    pub member: Member,
    /// Whole-leaf primitive count.
    pub primitive_count: u64,
    /// Whole-leaf combined peak.
    pub peak: usize,
    /// Whole-leaf maximum element width.
    pub maximum_width: usize,
}
/// Query result separates absent evidence, invalid association and non-handles.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Query {
    /// Available output has no keyed first-output membership.
    NotHandle,
    /// Named event was not supplied.
    MissingEvent(EventId),
    /// Available event has no such output.
    MissingOutput(Output),
    /// First output authenticates another offered key.
    WrongKey { offered: Handle, actual: Key },
    /// Version-one output has no authenticated key among the offered recipes.
    UnresolvedKey(Output),
    /// Local handle and its complete or incomplete reconstruction.
    Handle {
        handle: Handle,
        membership: Membership,
    },
}
