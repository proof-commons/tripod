//! Offline conformance reader, not a production chain reader or an admission API.
//!
//! Classification and recurrence use only canonical bytes and explicit evidence.
//! Public recipe/curve authentication, hashes and bounded byte evaluation are
//! shared substrate, not independent recurrence evidence. Boundary names never
//! establish SOURCE status. All four parent facts remain external premises.
mod authentication;
mod evidence;
mod fold;
mod identity;
mod refusal;

pub use evidence::{
    Address, Assertions, ByteSource, EventId, Handle, Input, Key, LeafEvidence, Member, Membership,
    Missing, Obligation, Output, ParentFact, Query, Relation, RelationEvidence,
};
pub use refusal::{Boundary, Refusal};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq)]
enum Class {
    Ordinary,
    Keyed(Key),
    Unresolved,
}

/// Immutable local reading. Complete membership is not chain acceptance.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reading {
    outputs: BTreeMap<EventId, Vec<Class>>,
    memberships: BTreeMap<Handle, Membership>,
    live: BTreeSet<Handle>,
    successors: BTreeMap<Handle, Handle>,
    relations: Vec<RelationEvidence>,
    obligations: BTreeSet<Obligation>,
    unexamined: BTreeMap<EventId, BTreeSet<Missing>>,
    leaves: Vec<LeafEvidence>,
    boundary_premises: BTreeSet<EventId>,
}
impl Reading {
    /// Every recognized handle, including incomplete membership closures.
    #[must_use]
    pub const fn memberships(&self) -> &BTreeMap<Handle, Membership> {
        &self.memberships
    }
    /// Locally unconsumed handles; not a chain unspentness observation.
    #[must_use]
    pub const fn live(&self) -> &BTreeSet<Handle> {
        &self.live
    }
    /// Byte-derived continuation edges, independent of builder coverage.
    /// A source handle may have no membership when its event bytes are unsupplied.
    #[must_use]
    pub const fn successors(&self) -> &BTreeMap<Handle, Handle> {
        &self.successors
    }
    /// Two authenticated model clauses per consumed keyed input.
    #[must_use]
    pub fn relations(&self) -> &[RelationEvidence] {
        &self.relations
    }
    /// Per-instance external parent facts, never discharged by local hashing.
    #[must_use]
    pub const fn obligations(&self) -> &BTreeSet<Obligation> {
        &self.obligations
    }
    /// Missing evidence for every event without a recognized keyed first output.
    #[must_use]
    pub const fn unexamined(&self) -> &BTreeMap<EventId, BTreeSet<Missing>> {
        &self.unexamined
    }
    /// Whole-leaf counters once per consumed member.
    #[must_use]
    pub fn leaves(&self) -> &[LeafEvidence] {
        &self.leaves
    }
    /// Fixture cuts, retained as provenance only.
    #[must_use]
    pub const fn boundary_premises(&self) -> &BTreeSet<EventId> {
        &self.boundary_premises
    }
    /// Distinguishes absent evidence, wrong identity and a genuine non-handle.
    /// # Panics
    /// Panics only if a recognized first output lacks its fold result, which the
    /// private immutable construction of this value cannot arrange.
    #[must_use]
    pub fn query(&self, offered: Handle) -> Query {
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
            Class::Ordinary => Query::NotHandle,
            Class::Unresolved => Query::UnresolvedKey(offered.output),
            Class::Keyed(actual) if *actual != offered.key => Query::WrongKey {
                offered,
                actual: *actual,
            },
            Class::Keyed(_) => Query::Handle {
                handle: offered,
                membership: self
                    .memberships
                    .get(&offered)
                    .expect("folded first output")
                    .clone(),
            },
        }
    }
}

/// Reconstructs a finite conformance history without reading producer roles.
/// # Errors
/// Returns a named consistency, syntax, authentication or model boundary failure.
/// Missing evidence is retained per handle as `Membership::Incomplete`, or in
/// `Reading::unexamined` for events without a keyed first output, never SOURCE.
/// # Panics
/// Panics only if the private canonical index loses a checked parent/output or a
/// completed dependency during the fold; callers cannot mutate that index.
pub fn read(input: &Input) -> Result<Reading, Refusal> {
    let _span = tracing::info_span!(target: "vectors::keyed_history_reader", "read_keyed_history", event_count = input.bytes.len(), key_count = input.keys.len()).entered();
    let result = derive(input);
    match &result {
        Ok(reading) => {
            tracing::info!(target: "vectors::keyed_history_reader", handle_count = reading.memberships.len(), incomplete_count = reading.memberships.values().filter(|value| matches!(value, Membership::Incomplete(_))).count(), relation_count = reading.relations.len(), "keyed history read completed");
        }
        Err(error) => {
            tracing::debug!(target: "vectors::keyed_history_reader", refusal_kind = error.refusal_kind(), boundary = ?error.boundary(), "keyed history read refused");
        }
    }
    result
}
fn derive(input: &Input) -> Result<Reading, Refusal> {
    let index = identity::load(input)?;
    let recipes = authentication::recipes(&input.keys)?;
    fold::reconstruct(input, &index, &recipes)
}
