//! A sealed conformance context owns the evidence, never a proposed family's verdict.
//! Reader and family indexes each decode once. Canonical decoding is shared substrate.
use super::{
    ArchivedObservation, BirthBody, Body, Completeness, ContinuationBody, Family, FamilyCondition,
    FamilyRefusal, Observation, Outcome, PaperId, ReadingBody, ReadingFamily, Recurrence,
    RecurrenceBody, Report, Request, RequirementId, Trace, family_resources, from_reading, report,
    validation,
};
use crate::keyed_history_reader::{self as r, Input, Reading, Relation};
use std::collections::{BTreeMap, BTreeSet};
use transaction::TargetTransaction;

/// Validated underlying bytes and one private family index, reused across validators.
/// No caller can supply a cached reading or edit this context after preparation.
#[derive(Clone, Debug)]
pub struct FamilyContext {
    pub(super) input: Input,
    pub(super) reading: Reading,
    pub(super) decoded: BTreeMap<r::EventId, TargetTransaction>,
    pub(super) archives:
        Result<Vec<crate::keyed_cost::CarrierCost>, crate::keyed_cost::CostRefusal>,
    pub(super) parents: BTreeMap<r::Address, TargetTransaction>,
    second_pass: std::time::Duration,
}
impl FamilyContext {
    /// Measured offline family-index construction wall, excluding the reader pass.
    #[must_use]
    pub const fn second_pass_wall(&self) -> std::time::Duration {
        self.second_pass
    }
    /// Exact underlying byte offers; no expected model sets are present.
    #[must_use]
    pub const fn input(&self) -> &Input {
        &self.input
    }
    /// Full admitted rows, including recorded disagreement and absent native dimensions.
    /// # Errors
    /// Preserves the archive owner's admission refusal; unrelated families still work.
    pub fn archive_rows(
        &self,
    ) -> Result<&[crate::keyed_cost::CarrierCost], crate::keyed_cost::CostRefusal> {
        self.archives.as_ref().map(Vec::as_slice).map_err(|e| *e)
    }
}
/// Decode the immutable subject twice as adopted, never once per member or validator.
/// Archives are loaded from their admitted sources, not supplied as report claims.
/// # Errors
/// Preserves reader refusals before any family checks. Archive admission failure is
/// retained in the context and refused only when a resources report is requested.
pub fn prepare_families(input: &Input) -> Result<FamilyContext, FamilyRefusal> {
    let _span = tracing::info_span!(target: "vectors::keyed_evidence", "prepare_keyed_families", event_count=input.bytes.len()).entered();
    let result = prepare(input).map_err(|mut error| {
        error.family = None;
        error.outer = super::FamilyBoundary::Preparation;
        error
    });
    event(&result, "prepare");
    result
}
fn prepare(input: &Input) -> Result<FamilyContext, FamilyRefusal> {
    let family = Family::Reading;
    let reading =
        r::read(input).map_err(|error| refusal(family, FamilyCondition::Reader(error)))?;
    let start = std::time::Instant::now();
    let decoded = input
        .bytes
        .iter()
        .map(|(id, bytes)| {
            TargetTransaction::decode(bytes)
                .map(|tx| (*id, tx))
                .map_err(|error| refusal(family, FamilyCondition::Decode(error)))
        })
        .collect::<Result<BTreeMap<r::EventId, TargetTransaction>, _>>()?;
    let mut parents: BTreeMap<_, _> = input
        .association
        .iter()
        .filter_map(|(event, address)| decoded.get(event).map(|tx| (*address, tx.clone())))
        .collect();
    for (address, bytes) in &input.retrieved {
        if !parents.contains_key(address) {
            parents.insert(
                *address,
                TargetTransaction::decode(bytes)
                    .map_err(|e| refusal(family, FamilyCondition::Decode(e)))?,
            );
        }
    }
    let second_pass = start.elapsed();
    // Admission is held once, including a failure. Only resources consumes it.
    let archives = crate::keyed_cost::cost_packet().map(|packet| packet.carriers);
    Ok(FamilyContext {
        input: input.clone(),
        reading,
        decoded,
        archives,
        second_pass,
        parents,
    })
}
pub(super) const fn refusal(family: Family, condition: FamilyCondition) -> FamilyRefusal {
    FamilyRefusal {
        family: Some(family),
        condition,
        outer: super::FamilyBoundary::FamilyReportValidation,
    }
}
fn event<T>(result: &Result<T, FamilyRefusal>, operation: &'static str) {
    match result {
        Ok(_) => {
            tracing::info!(target: "vectors::keyed_evidence", operation, "keyed family completed");
        }
        Err(error) => {
            tracing::debug!(target: "vectors::keyed_evidence", operation, family=?error.family, refusal_kind=error.refusal_kind(), observation_boundary=?error.observation_boundary(), "keyed family refused");
        }
    }
}
/// Produce one family's claims from its immutable bytes/evidence context.
/// # Errors
/// Returns the family's typed framing or observation refusal.
pub fn family_report(
    context: &FamilyContext,
    family: Family,
    request: &Request,
) -> Result<Report, FamilyRefusal> {
    let _span =
        tracing::info_span!(target: "vectors::keyed_evidence", "keyed_family_report", ?family)
            .entered();
    let result = derive(context, family, request);
    event(&result, "report");
    result
}
fn derive(c: &FamilyContext, family: Family, request: &Request) -> Result<Report, FamilyRefusal> {
    let mut request = request.clone();
    request.family = if family == Family::Acceptance {
        ReadingFamily::Acceptance
    } else {
        ReadingFamily::Reading
    };
    // Relation/resource reports do not demand a complete membership closure.
    if matches!(
        family,
        Family::Birth | Family::Continuation | Family::Resources
    ) {
        if request.finality == super::FinalityRequest::Required {
            return Err(refusal(family, FamilyCondition::FinalityUnsupported));
        }
        request.completeness = Completeness::Report;
        request.finality = super::FinalityRequest::NotRequested;
        request.declaration = None;
    }
    validation::provenance(&request)
        .map_err(|e| refusal(family, FamilyCondition::Observation(e)))?;
    let mut report = from_reading(&c.input, &c.reading, &request, Some(&c.decoded))
        .map_err(|e| refusal(family, FamilyCondition::Observation(e)))?;
    report.header.family = family;
    if matches!(
        family,
        Family::Birth | Family::Continuation | Family::Resources
    ) {
        match c.reading.query(request.handle) {
            r::Query::Handle { .. } => (),
            other => {
                return Err(refusal(
                    family,
                    FamilyCondition::Observation(super::Refusal::Query(other)),
                ));
            }
        }
    }
    let event_id = request.handle.output.event;
    report.body = match family {
        Family::Reading => {
            if let Body::Reading(body) = &mut report.body {
                body.trace = trace(Family::Reading);
            }
            return Ok(report);
        }
        Family::Acceptance => return Ok(report),
        Family::Birth => Body::Birth(BirthBody {
            trace: trace(family),
            first_outputs: c
                .reading
                .memberships()
                .keys()
                .filter(|h| h.output.event == event_id)
                .copied()
                .collect(),
            relations: super::family_execution::relations(c, event_id, Relation::BornFirst)
                .map_err(|e| refusal(family, e))?,
        }),
        Family::Continuation => Body::Continuation(ContinuationBody {
            trace: trace(family),
            relations: super::family_execution::relations(c, event_id, Relation::ContinueFirst)
                .map_err(|e| refusal(family, e))?,
            successors: c
                .reading
                .successors()
                .iter()
                .filter(|(_, h)| h.output.event == event_id)
                .map(|(a, b)| (*a, *b))
                .collect(),
        }),
        Family::Recurrence => Body::Recurrence(recurrence(c, &request, report.body)?),
        Family::Resources => {
            let body = family_resources::derive(c, event_id).map_err(|e| refusal(family, e))?;
            report.header.observation = Observation::OfflineResources {
                archives: archive_references(c, body.members).map_err(|e| refusal(family, e))?,
            };
            family_resources::header(&mut report.header, &body).map_err(|e| refusal(family, e))?;
            Body::Resources(body)
        }
    };
    if matches!(family, Family::Birth | Family::Continuation) {
        let bytes = c.input.bytes[&event_id].len();
        report.header.quantities = report::quantities(None, None, bytes)
            .map_err(|e| refusal(family, FamilyCondition::Observation(e)))?;
    }
    Ok(report)
}
fn archive_references(
    c: &FamilyContext,
    members: u64,
) -> Result<Vec<ArchivedObservation>, FamilyCondition> {
    use target_elements_conformance::constructor::tagged::sha256;
    Ok(c.archives
        .as_ref()
        .map_err(|e| FamilyCondition::Archive(*e))?
        .iter()
        .filter(|row| {
            row.witnesses
                .iter()
                .filter(|w| w.item_bytes == [52, 33])
                .count() as u64
                == members
        })
        .map(|row| ArchivedObservation {
            archive: row.archive.clone(),
            step: row.step.clone(),
            digest: super::ArchiveDigest(sha256(&row.transaction)),
        })
        .collect())
}
// Consistency guard: re-apply the reader fold to its own memberships.
// Independent recurrence evidence comes from the graph/literal harness.
fn recurrence(
    c: &FamilyContext,
    request: &Request,
    body: Body,
) -> Result<RecurrenceBody, FamilyRefusal> {
    let Body::Reading(reading) = body else {
        return Err(refusal(
            Family::Recurrence,
            FamilyCondition::RecurrenceShape,
        ));
    };
    let outcome = match reading.outcome {
        Outcome::Complete(complete) => {
            let consumed: BTreeMap<_, _> = c
                .reading
                .successors()
                .iter()
                .filter(|(_, next)| **next == request.handle)
                .map(|(h, _)| {
                    let state = c.reading.memberships().get(h).ok_or_else(|| {
                        refusal(
                            Family::Recurrence,
                            FamilyCondition::MissingConsumedMembership,
                        )
                    })?;
                    if !matches!(state, r::Membership::Complete(_)) {
                        return Err(refusal(
                            Family::Recurrence,
                            FamilyCondition::MissingConsumedMembership,
                        ));
                    }
                    Ok((*h, state.clone()))
                })
                .collect::<Result<BTreeMap<_, _>, FamilyRefusal>>()?;
            let source = consumed.is_empty().then_some(request.handle);
            let mut union = source.into_iter().collect::<BTreeSet<_>>();
            for state in consumed.values() {
                if let r::Membership::Complete(records) = state {
                    union.extend(records);
                }
            }
            if union != complete.membership {
                return Err(refusal(Family::Recurrence, FamilyCondition::Recurrence));
            }
            Outcome::Complete(Recurrence {
                source,
                consumed,
                records: union,
            })
        }
        Outcome::Incomplete { handle, missing } => Outcome::Incomplete { handle, missing },
        Outcome::Invalid(e) => Outcome::Invalid(e),
        Outcome::NotHandle(h) => Outcome::NotHandle(h),
    };
    Ok(ReadingBody {
        trace: trace(Family::Recurrence),
        outcome,
        incomplete: reading.incomplete,
        unexamined: reading.unexamined,
    })
}
pub(super) fn trace(family: Family) -> Trace {
    let (requirements, paper): (&[u8], &[&str]) = match family {
        Family::Birth => (
            &[1, 2],
            &[
                "def:kpc-model:first-output",
                "prop:kpc-model:single-birth",
                "thm:kpc-invariants:unique-successor",
            ],
        ),
        Family::Continuation => (
            &[1, 2, 3],
            &[
                "def:kpc-model:first-output",
                "prop:kpc-model:single-birth",
                "thm:kpc-invariants:unique-successor",
                "def:kpc-model:keyed-membership",
                "thm:kpc-invariants:key-isolation",
            ],
        ),
        Family::Recurrence => (
            &[4, 5],
            &[
                "op:kpc-operations:source",
                "thm:kpc-invariants:source-provenance",
                "op:kpc-operations:merge",
                "def:kpc-operations:classification",
                "thm:kpc-invariants:set-membership-growth",
            ],
        ),
        Family::Reading => (
            &[5, 6],
            &[
                "op:kpc-operations:merge",
                "def:kpc-operations:classification",
                "thm:kpc-invariants:set-membership-growth",
                "def:kpc-interface:membership-reading",
                "thm:kpc-interface:membership-correctness",
                "cor:kpc-interface:path-reading",
            ],
        ),
        Family::Acceptance => return report::trace(ReadingFamily::Acceptance).1,
        Family::Resources => (&[], &[]), // requirement 8 belongs to every header.
    };
    Trace {
        requirements: requirements.iter().copied().map(RequirementId).collect(),
        paper: paper.iter().map(|label| PaperId((*label).into())).collect(),
    }
}
fn validate(
    c: &FamilyContext,
    family: Family,
    request: &Request,
    proposed: &Report,
) -> Result<(), FamilyRefusal> {
    let _span =
        tracing::info_span!(target: "vectors::keyed_evidence", "validate_keyed_family", ?family)
            .entered();
    let result =
        derive(c, family, request).and_then(|expected| compare(family, &expected, proposed));
    event(&result, "validate");
    result
}
fn compare(family: Family, expected: &Report, proposed: &Report) -> Result<(), FamilyRefusal> {
    validation::header(&expected.header, &proposed.header)
        .map_err(|e| refusal(family, FamilyCondition::Framing(e)))?;
    let condition = match (&expected.body, &proposed.body) {
        (Body::Reading(_), Body::Reading(_)) | (Body::Acceptance(_), Body::Acceptance(_)) => {
            return validation::compare(expected, proposed)
                .map_err(|e| refusal(family, FamilyCondition::Framing(e)));
        }
        (Body::Birth(a), Body::Birth(b)) => {
            if a.trace != b.trace {
                Some(FamilyCondition::Trace)
            } else if a.first_outputs != b.first_outputs {
                Some(FamilyCondition::FirstOutputs)
            } else {
                compare_relations(&a.relations, &b.relations).err()
            }
        }
        (Body::Continuation(a), Body::Continuation(b)) => {
            if a.trace != b.trace {
                Some(FamilyCondition::Trace)
            } else if let Err(error) = compare_relations(&a.relations, &b.relations) {
                Some(error)
            } else if a.successors != b.successors {
                Some(FamilyCondition::Successors)
            } else {
                None
            }
        }
        (Body::Recurrence(a), Body::Recurrence(b)) => compare_recurrence(a, b).err(),
        (Body::Resources(a), Body::Resources(b)) => family_resources::compare(a, b).err(),
        _ => Some(FamilyCondition::Body),
    };
    condition.map_or(Ok(()), |e| Err(refusal(family, e)))
}

/// Recompute birth claims from underlying evidence, never another report's verdict.
/// # Errors
/// Refuses mismatched framing or family claims, preserving observation refusals.
pub fn validate_birth(
    context: &FamilyContext,
    request: &Request,
    proposed: &Report,
) -> Result<(), FamilyRefusal> {
    validate(context, Family::Birth, request, proposed)
}

/// Recompute continuation claims from underlying evidence, never another report's verdict.
/// # Errors
/// Refuses mismatched framing or family claims, preserving observation refusals.
pub fn validate_continuation(
    context: &FamilyContext,
    request: &Request,
    proposed: &Report,
) -> Result<(), FamilyRefusal> {
    validate(context, Family::Continuation, request, proposed)
}

/// Recompute recurrence claims from underlying evidence, never another report's verdict.
/// # Errors
/// Refuses mismatched framing or family claims, preserving observation refusals.
pub fn validate_recurrence(
    context: &FamilyContext,
    request: &Request,
    proposed: &Report,
) -> Result<(), FamilyRefusal> {
    validate(context, Family::Recurrence, request, proposed)
}

/// Recompute reading claims from underlying evidence, never another report's verdict.
/// # Errors
/// Refuses mismatched framing or family claims, preserving observation refusals.
pub fn validate_reading(
    context: &FamilyContext,
    request: &Request,
    proposed: &Report,
) -> Result<(), FamilyRefusal> {
    validate(context, Family::Reading, request, proposed)
}

/// Recompute acceptance claims from underlying evidence, never another report's verdict.
/// # Errors
/// Refuses mismatched framing or family claims, preserving observation refusals.
pub fn validate_acceptance(
    context: &FamilyContext,
    request: &Request,
    proposed: &Report,
) -> Result<(), FamilyRefusal> {
    validate(context, Family::Acceptance, request, proposed)
}

/// Recompute resources claims from underlying evidence, never another report's verdict.
/// # Errors
/// Refuses mismatched framing or family claims, preserving observation refusals.
pub fn validate_resources(
    context: &FamilyContext,
    request: &Request,
    proposed: &Report,
) -> Result<(), FamilyRefusal> {
    validate(context, Family::Resources, request, proposed)
}

fn compare_relations(
    a: &[r::RelationEvidence],
    b: &[r::RelationEvidence],
) -> Result<(), FamilyCondition> {
    if a.len() != b.len() {
        return Err(FamilyCondition::Relations);
    }
    for (a, b) in a.iter().zip(b) {
        if a.event != b.event {
            return Err(FamilyCondition::RelationEvent);
        }
        if a.input != b.input {
            return Err(FamilyCondition::RelationInput);
        }
        if a.member != b.member {
            return Err(FamilyCondition::RelationMember);
        }
        if a.relation != b.relation {
            return Err(FamilyCondition::RelationKind);
        }
        if a.leaf != b.leaf {
            return Err(FamilyCondition::RelationLeaf);
        }
        if a.control != b.control {
            return Err(FamilyCondition::RelationControl);
        }
        if (a.primitive_count, a.peak, a.maximum_width)
            != (b.primitive_count, b.peak, b.maximum_width)
        {
            return Err(FamilyCondition::RelationCounters);
        }
    }
    Ok(())
}
fn compare_recurrence(a: &RecurrenceBody, b: &RecurrenceBody) -> Result<(), FamilyCondition> {
    if a.trace != b.trace {
        return Err(FamilyCondition::Trace);
    }
    if a.incomplete != b.incomplete {
        return Err(FamilyCondition::Framing(super::Refusal::IncompleteMismatch));
    }
    if a.unexamined != b.unexamined {
        return Err(FamilyCondition::Framing(super::Refusal::UnexaminedMismatch));
    }
    match (&a.outcome, &b.outcome) {
        (Outcome::Complete(a), Outcome::Complete(b)) => {
            if a.source != b.source {
                return Err(FamilyCondition::RecurrenceSource);
            }
            if a.consumed != b.consumed {
                return Err(FamilyCondition::RecurrenceInputs);
            }
            if a.records != b.records {
                return Err(FamilyCondition::RecurrenceRecords);
            }
        }
        (a, b) if a == b => (),
        _ => return Err(FamilyCondition::Framing(super::Refusal::OutcomeMismatch)),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn archive_failure_is_retained_without_refusing_preparation() {
        let input = Input {
            bytes: BTreeMap::new(),
            association: Vec::new(),
            keys: BTreeSet::new(),
            retrieved: BTreeMap::new(),
            boundary_premises: BTreeSet::new(),
            assertions: r::Assertions::default(),
        };
        let mut context = prepare_families(&input).unwrap();
        assert!(context.archive_rows().is_ok());
        assert!(archive_references(&context, 0).is_ok());
        let before = context.reading.clone();
        context.archives = Err(crate::keyed_cost::CostRefusal::Admission);
        assert_eq!(
            context.archive_rows(),
            Err(crate::keyed_cost::CostRefusal::Admission)
        );
        assert_eq!(
            archive_references(&context, 0),
            Err(FamilyCondition::Archive(
                crate::keyed_cost::CostRefusal::Admission
            ))
        );
        assert_eq!(context.reading, before);
        let error = refusal(
            Family::Resources,
            FamilyCondition::Archive(crate::keyed_cost::CostRefusal::Admission),
        );
        assert_eq!(
            error.boundary(),
            super::super::FamilyBoundary::FamilyReportValidation
        );
        assert_eq!(error.observation_boundary(), None);
    }
}
