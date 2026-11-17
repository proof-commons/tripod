//! Typed local evidence for a complete fixed-form reading.
use std::collections::{BTreeMap, BTreeSet};

pub type EventId = u32;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Triple {
    pub key: [u8; 32],
    pub asset: [u8; 32],
    pub value: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Address(pub [u8; 32]);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ByteSource {
    Event(EventId),
    Retrieved(Address),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Output {
    pub event: EventId,
    pub position: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Handle {
    pub triple: Triple,
    pub output: Output,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Assertions {
    pub inclusion: BTreeSet<Address>,
    pub retrieval: BTreeSet<Address>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Input {
    pub bytes: BTreeMap<EventId, Vec<u8>>,
    pub association: Vec<(EventId, Address)>,
    pub triples: BTreeSet<Triple>,
    pub retrieved: BTreeMap<Address, Vec<u8>>,
    pub assertions: Assertions,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Missing {
    Event(EventId),
    Parent {
        event: EventId,
        input: usize,
        address: Address,
        position: u32,
    },
    Inclusion(Address),
    Retrieval(Address),
    Membership {
        event: EventId,
        input: usize,
        address: Address,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Membership {
    Complete(BTreeSet<Handle>),
    Incomplete(BTreeSet<Missing>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ParentFact {
    Inclusion,
    Unspentness,
    Retrieval,
    Finality,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Obligation {
    pub event: EventId,
    pub input: usize,
    pub address: Address,
    pub position: u32,
    pub fact: ParentFact,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Query {
    UnprovidedTriple(Triple),
    NotHandle,
    MissingEvent(EventId),
    MissingOutput(Output),
    WrongTriple {
        offered: Handle,
        actual: Triple,
    },
    UnresolvedProgram(Output),
    Handle {
        handle: Handle,
        membership: Membership,
    },
}
