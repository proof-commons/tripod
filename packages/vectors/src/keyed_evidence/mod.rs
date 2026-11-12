//! Offline reader rules and internal report framing, not a production acceptance API.
//!
//! Membership and classification belong exclusively to the unchanged history reader.
//! Report validation recomputes from bytes/evidence; no proposed pass bit is trusted.
mod acceptance;
mod premises;
mod refusal;
mod report;
mod validation;
mod values;

use crate::keyed_history_reader::{Input, Membership, Query, Reading};
pub use acceptance::{Rule, Scope};
pub use premises::{
    Assumption, Declaration, DeclarationOffer, Deployment, DeploymentDeclaration, EvidenceSource,
    Finality, FinalityRequest, FinalityRule, HistoryAnchor, HypotheticalDeclaration, PolicyVersion,
    PremiseSources, Provenance,
};
pub use refusal::{Boundary, Field, MissingEvents, MissingHandles, ProvenanceField, Refusal};
pub use report::{
    Absence, AcceptanceBody, AcceptedReading, Body, CompleteReading, Completeness, Dimension,
    EvidenceReferences, Family, Header, Measurement, Observation, Outcome, PaperId, Quantity,
    ReadingBody, ReadingFamily, Report, Request, RequirementId, SCHEMA, Subject, Trace, Unit,
};
pub use values::{Amount, Annotation, Asset, AssetValue, Attribution, Epoch, Unknown, Value};

/// Reuses the reader's membership without changing it, and filters SOURCE records.
/// # Errors
/// Refuses incomplete evidence, invalid provenance/finality/rules, or reader errors.
pub fn reading_report(input: &Input, request: &Request) -> Result<Report, Refusal> {
    let _span = tracing::info_span!(target: "vectors::keyed_evidence", "read_keyed_evidence", event_count = input.bytes.len()).entered();
    let result = derive(input, request);
    match &result {
        Ok(_) => {
            tracing::info!(target: "vectors::keyed_evidence", "keyed evidence reading completed");
        }
        Err(error) => {
            tracing::debug!(target: "vectors::keyed_evidence", refusal_kind = error.refusal_kind(), boundary = ?error.boundary(), inner_refusal_kind = inner_kind(error), "keyed evidence reading refused");
        }
    }
    result
}

/// Recomputes the reading, annotations, filter and framing from underlying evidence.
///
/// This validates the reading/acceptance reports only; other family semantics are
/// deliberately left to step 4, which reuses the frame without trusting this verdict.
/// # Errors
/// Returns a typed evidence, rule, framing or quantity refusal.
pub fn validate_report(input: &Input, request: &Request, proposed: &Report) -> Result<(), Refusal> {
    let _span = tracing::info_span!(target: "vectors::keyed_evidence", "validate_keyed_evidence", event_count = input.bytes.len()).entered();
    let result =
        derive(input, request).and_then(|expected| validation::compare(&expected, proposed));
    match &result {
        Ok(()) => {
            tracing::info!(target: "vectors::keyed_evidence", "keyed evidence report validated");
        }
        Err(error) => {
            tracing::debug!(target: "vectors::keyed_evidence", refusal_kind = error.refusal_kind(), boundary = ?error.boundary(), inner_refusal_kind = inner_kind(error), "keyed evidence report refused");
        }
    }
    result
}

const fn inner_kind(error: &Refusal) -> Option<&'static str> {
    match error {
        Refusal::Reader(inner) => Some(inner.refusal_kind()),
        _ => None,
    }
}
fn derive(input: &Input, request: &Request) -> Result<Report, Refusal> {
    validation::provenance(request)?;
    let reading = crate::keyed_history_reader::read(input).map_err(Refusal::Reader)?;
    from_reading(input, &reading, request)
}

fn from_reading(input: &Input, reading: &Reading, request: &Request) -> Result<Report, Refusal> {
    let (incomplete, unexamined) = acceptance::failures(reading);
    let outcome = match reading.query(request.handle) {
        Query::Handle {
            membership: Membership::Complete(membership),
            ..
        } => {
            if request.scope == Scope::WholeHistory
                && (!incomplete.is_empty() || !unexamined.is_empty())
            {
                return Err(Refusal::WholeHistory {
                    handles: incomplete,
                    events: unexamined,
                });
            }
            Outcome::Complete(membership)
        }
        Query::Handle {
            membership: Membership::Incomplete(missing),
            ..
        } => {
            if request.scope == Scope::WholeHistory {
                return Err(Refusal::WholeHistory {
                    handles: incomplete,
                    events: unexamined,
                });
            }
            if request.completeness == Completeness::Required
                || request.finality == FinalityRequest::Required
            {
                return Err(Refusal::Incomplete {
                    handle: request.handle,
                    missing,
                });
            }
            Outcome::Incomplete {
                handle: request.handle,
                missing,
            }
        }
        Query::NotHandle
            if request.completeness == Completeness::Report
                && request.finality == FinalityRequest::NotRequested =>
        {
            Outcome::NotHandle(request.handle)
        }
        other => return Err(Refusal::Query(other)),
    };
    assemble(input, reading, request, outcome, incomplete, unexamined)
}
fn assemble(
    input: &Input,
    reading: &Reading,
    request: &Request,
    outcome: Outcome<std::collections::BTreeSet<crate::keyed_history_reader::Handle>>,
    incomplete: MissingHandles,
    unexamined: MissingEvents,
) -> Result<Report, Refusal> {
    let subject = subject(input, request.handle)?;
    let byte_count = subject.bytes.len();
    let (family, trace) = report::trace(request.family);
    let mut records = None;
    let mut accepted_count = None;
    let mut standing = Finality::Nonfinal;
    let body = match request.family {
        ReadingFamily::Reading => {
            let outcome = match outcome {
                Outcome::Complete(membership) => {
                    records = Some(membership.len());
                    standing = premises::finality(
                        &request.provenance,
                        &request.sources,
                        request.finality,
                        request.declaration.as_ref(),
                    )?;
                    Outcome::Complete(CompleteReading {
                        scope: request.scope,
                        finality: standing.clone(),
                        membership,
                    })
                }
                Outcome::Incomplete { handle, missing } => Outcome::Incomplete { handle, missing },
                Outcome::Invalid(error) => Outcome::Invalid(error),
                Outcome::NotHandle(handle) => Outcome::NotHandle(handle),
            };
            Body::Reading(ReadingBody {
                trace,
                outcome,
                incomplete,
                unexamined,
            })
        }
        ReadingFamily::Acceptance => {
            let outcome = match outcome {
                Outcome::Complete(membership) => {
                    let annotations = acceptance::annotations(input, &membership)?;
                    let accepted =
                        acceptance::accepted(&request.rule, request.observed_epoch, &annotations)?;
                    records = Some(membership.len());
                    accepted_count = Some(accepted.len());
                    standing = premises::finality(
                        &request.provenance,
                        &request.sources,
                        request.finality,
                        request.declaration.as_ref(),
                    )?;
                    Outcome::Complete(AcceptedReading {
                        reading: CompleteReading {
                            scope: request.scope,
                            finality: standing.clone(),
                            membership,
                        },
                        accepted,
                        annotations,
                    })
                }
                Outcome::Incomplete { handle, missing } => Outcome::Incomplete { handle, missing },
                Outcome::Invalid(error) => Outcome::Invalid(error),
                Outcome::NotHandle(handle) => Outcome::NotHandle(handle),
            };
            Body::Acceptance(AcceptanceBody {
                rule: request.rule.clone(),
                epoch: request.observed_epoch,
                reading: ReadingBody {
                    trace,
                    outcome,
                    incomplete,
                    unexamined,
                },
            })
        }
    };
    let assumptions = premises::assumptions(reading, &input.keys, &request.sources, &standing);
    Ok(Report {
        header: Header {
            schema: SCHEMA,
            family,
            subject: Some(subject),
            provenance: Some(request.provenance.clone()),
            trace: Some(report::header_trace()),
            evidence: Some(EvidenceReferences::from(input)),
            assumptions: Some(assumptions),
            observation: Observation::OfflineReaderEvidence,
            quantities: report::quantities(records, accepted_count, byte_count)?,
        },
        body,
    })
}

fn subject(input: &Input, handle: crate::keyed_history_reader::Handle) -> Result<Subject, Refusal> {
    let bytes = input
        .bytes
        .get(&handle.output.event)
        .ok_or(Refusal::SubjectMissing(handle))?
        .clone();
    let address = input
        .association
        .iter()
        .find(|(id, _)| *id == handle.output.event)
        .ok_or(Refusal::SubjectMissing(handle))?
        .1;
    Ok(Subject {
        handle,
        address,
        bytes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn synthetic_subject_lookup_is_distinct_from_annotation_lookup() {
        use crate::keyed_history_reader::{Address, Assertions, Handle, Key, Output};
        use std::collections::{BTreeMap, BTreeSet};
        let handle = Handle {
            key: Key([1; 32]),
            output: Output {
                event: 1,
                position: 0,
            },
        };
        // Lookup only: these are not a reader-accepted transaction or content address.
        let input = Input {
            bytes: BTreeMap::from([(1, vec![1])]),
            association: vec![(1, Address([2; 32]))],
            keys: BTreeSet::new(),
            retrieved: BTreeMap::new(),
            boundary_premises: BTreeSet::new(),
            assertions: Assertions::default(),
        };
        assert_eq!(
            subject(&input, handle),
            Ok(Subject {
                handle,
                address: Address([2; 32]),
                bytes: vec![1]
            })
        );
        let mut absent = input.clone();
        absent.bytes.clear();
        assert_eq!(
            subject(&absent, handle),
            Err(Refusal::SubjectMissing(handle))
        );
        let mut unnamed = input;
        unnamed.association.clear();
        assert_eq!(
            subject(&unnamed, handle),
            Err(Refusal::SubjectMissing(handle))
        );
    }
}
