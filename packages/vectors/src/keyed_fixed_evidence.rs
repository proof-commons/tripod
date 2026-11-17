//! Acceptance is a caller's fold over authenticated SOURCE transactions.
use crate::keyed_fixed_history_reader::{
    Handle, Membership, Missing, ParentFact, Query, Reading, Refusal, Triple,
};
use std::collections::BTreeSet;
use transaction::TargetTransaction;

/// The externally supplied finality claim remains distinct from byte reading.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Finality {
    Unclaimed,
    Supplied,
}

/// One application predicate and its named external premise.
#[derive(Clone, Copy)]
pub struct Request<'a> {
    pub reading: Result<&'a Reading, &'a Refusal>,
    pub handle: Handle,
    pub source_predicate: &'a dyn Fn(&TargetTransaction) -> bool,
    pub finality: Finality,
}

/// A complete set is conditional on the supplied premises; acceptance is separate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Report {
    Complete {
        records: BTreeSet<Handle>,
        accepted_records: BTreeSet<Handle>,
        finality: Finality,
        external: BTreeSet<ParentFact>,
    },
    Incomplete(BTreeSet<Missing>),
    UnprovidedTriple(Triple),
    NotHandle(Query),
    Invalid(Refusal),
}

/// Fold the immutable SOURCE events with the caller's application predicate.
#[must_use]
pub fn report(request: Request<'_>) -> Report {
    let reading = match request.reading {
        Ok(reading) => reading,
        Err(error) => return Report::Invalid(error.clone()),
    };
    match reading.query(request.handle) {
        Query::Handle {
            membership: Membership::Complete(records),
            ..
        } => {
            let accepted_records = records
                .iter()
                .filter(|record| {
                    reading
                        .source_event(**record)
                        .is_some_and(request.source_predicate)
                })
                .copied()
                .collect();
            let external = [
                ParentFact::Inclusion,
                ParentFact::Unspentness,
                ParentFact::Retrieval,
                ParentFact::Finality,
            ]
            .into_iter()
            .collect();
            Report::Complete {
                records,
                accepted_records,
                finality: request.finality,
                external,
            }
        }
        Query::Handle {
            membership: Membership::Incomplete(missing),
            ..
        } => Report::Incomplete(missing),
        Query::MissingEvent(event) => Report::Incomplete(BTreeSet::from([Missing::Event(event)])),
        Query::UnprovidedTriple(triple) => Report::UnprovidedTriple(triple),
        other => Report::NotHandle(other),
    }
}
