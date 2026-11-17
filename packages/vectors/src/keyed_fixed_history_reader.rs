//! Offline complete reading of one or more public fixed-form triples.
mod authentication;
mod evidence;
mod fold;
mod identity;
mod refusal;

pub use evidence::{
    Address, Assertions, ByteSource, EventId, Handle, Input, Membership, Missing, Obligation,
    Output, ParentFact, Query, Triple,
};
pub use refusal::{Boundary, Refusal};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Class {
    Ordinary,
    Qualified(Triple),
    Unqualified(Triple),
    Unresolved,
}

/// Local membership and record sets, conditional on supplied external premises.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reading {
    triples: BTreeSet<Triple>,
    outputs: BTreeMap<EventId, Vec<Class>>,
    memberships: BTreeMap<Handle, Membership>,
    live: BTreeSet<Handle>,
    successors: BTreeMap<Handle, Handle>,
    obligations: BTreeSet<Obligation>,
    unexamined: BTreeMap<EventId, BTreeSet<Missing>>,
    transactions: BTreeMap<EventId, transaction::TargetTransaction>,
    event_bytes: usize,
    edges: usize,
}

impl Reading {
    #[must_use]
    pub const fn memberships(&self) -> &BTreeMap<Handle, Membership> {
        &self.memberships
    }
    #[must_use]
    pub const fn live(&self) -> &BTreeSet<Handle> {
        &self.live
    }
    #[must_use]
    pub const fn successors(&self) -> &BTreeMap<Handle, Handle> {
        &self.successors
    }
    #[must_use]
    pub const fn obligations(&self) -> &BTreeSet<Obligation> {
        &self.obligations
    }
    #[must_use]
    pub const fn unexamined(&self) -> &BTreeMap<EventId, BTreeSet<Missing>> {
        &self.unexamined
    }
    #[must_use]
    pub const fn event_bytes(&self) -> usize {
        self.event_bytes
    }
    #[must_use]
    pub const fn edges(&self) -> usize {
        self.edges
    }
    #[must_use]
    pub fn source_event(&self, record: Handle) -> Option<&transaction::TargetTransaction> {
        self.transactions.get(&record.output.event)
    }

    /// Distinguishes missing output, nonmembership, another triple and a record set.
    /// # Panics
    /// Panics only if private folding omits a classified complete handle.
    #[must_use]
    pub fn query(&self, offered: Handle) -> Query {
        if !self.triples.contains(&offered.triple) {
            return Query::UnprovidedTriple(offered.triple);
        }
        let Some(outputs) = self.outputs.get(&offered.output.event) else {
            return Query::MissingEvent(offered.output.event);
        };
        let Some(class) = outputs.get(offered.output.position) else {
            return Query::MissingOutput(offered.output);
        };
        if offered.output.position != 0 {
            return Query::NotHandle;
        }
        match class {
            Class::Ordinary | Class::Unqualified(_) => Query::NotHandle,
            Class::Unresolved => Query::UnresolvedProgram(offered.output),
            Class::Qualified(actual) if *actual != offered.triple => Query::WrongTriple {
                offered,
                actual: *actual,
            },
            Class::Qualified(_) => Query::Handle {
                handle: offered,
                membership: self
                    .memberships
                    .get(&offered)
                    .expect("folded handle")
                    .clone(),
            },
        }
    }
}

/// Decode all supplied events, authenticate their exact fields and fold complete dependencies.
/// # Errors
/// Refuses contradictory identity, malformed bytes, authenticated clause failures or a bad triple.
/// Missing evidence is returned as `Membership::Incomplete` or `Reading::unexamined`.
/// # Panics
/// Panics only if an immutable checked index loses its parent or topological dependency.
pub fn read(input: &Input) -> Result<Reading, Refusal> {
    let _span = tracing::info_span!(target: "vectors::keyed_fixed_history_reader", "read_keyed_fixed_history", event_count = input.bytes.len(), triple_count = input.triples.len()).entered();
    let index = identity::load(input)?;
    let recipes = authentication::recipes(&input.triples)?;
    let result = fold::reconstruct(input, &index, &recipes);
    match &result {
        Ok(reading) => {
            tracing::info!(target: "vectors::keyed_fixed_history_reader", handle_count = reading.memberships.len(), edge_count = reading.edges, byte_count = reading.event_bytes, "fixed history read completed");
        }
        Err(error) => {
            tracing::debug!(target: "vectors::keyed_fixed_history_reader", boundary = ?error.boundary(), refusal = ?error, "fixed history read refused");
        }
    }
    result
}
