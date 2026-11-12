//! Field binding against a fresh derivation, not a report digest or builder total.
use super::{
    AcceptedReading, Body, CompleteReading, Field, Header, Measurement, Outcome, ProvenanceField,
    ReadingBody, Refusal, Report, Request,
};

pub fn provenance(request: &Request) -> Result<(), Refusal> {
    for (field, value) in [
        (
            ProvenanceField::Deployment,
            &request.provenance.deployment.0,
        ),
        (
            ProvenanceField::PolicyVersion,
            &request.provenance.policy_version.0,
        ),
        (
            ProvenanceField::HistorySource,
            &request.provenance.history_source.0,
        ),
        (
            ProvenanceField::PolicySource,
            &request.provenance.policy_source.0,
        ),
        (ProvenanceField::Inclusion, &request.sources.inclusion.0),
        (ProvenanceField::Unspentness, &request.sources.unspentness.0),
        (ProvenanceField::Retrieval, &request.sources.retrieval.0),
        (ProvenanceField::Finality, &request.sources.finality.0),
    ] {
        if value.is_empty() {
            return Err(Refusal::EmptyProvenance(field));
        }
    }
    Ok(())
}

pub fn compare(expected: &Report, offered: &Report) -> Result<(), Refusal> {
    header(&expected.header, &offered.header)?;
    body(&expected.body, &offered.body)
}
fn header(expected: &Header, offered: &Header) -> Result<(), Refusal> {
    if offered.schema != expected.schema {
        return Err(Refusal::SchemaMismatch);
    }
    if offered.family != expected.family {
        return Err(Refusal::FamilyMismatch);
    }
    let subject = offered
        .subject
        .as_ref()
        .ok_or(Refusal::MissingField(Field::Subject))?;
    let expected_subject = expected
        .subject
        .as_ref()
        .ok_or(Refusal::MissingField(Field::Subject))?;
    if subject.bytes != expected_subject.bytes {
        return Err(Refusal::SubjectBytesMismatch);
    }
    if subject.address != expected_subject.address {
        return Err(Refusal::SubjectIdentityMismatch);
    }
    if subject.handle != expected_subject.handle {
        return Err(Refusal::SubjectHandleMismatch);
    }
    components(expected, offered)?;
    if offered.observation != expected.observation {
        return Err(Refusal::ObservationMismatch);
    }
    for (dimension, expected) in &expected.quantities {
        let actual = offered
            .quantities
            .get(dimension)
            .ok_or(Refusal::QuantityMissing(*dimension))?;
        quantity(*dimension, &expected.observed, &actual.observed)?;
        quantity(*dimension, &expected.predicted, &actual.predicted)?;
    }
    Ok(())
}

fn components(expected: &Header, offered: &Header) -> Result<(), Refusal> {
    for (field, present) in [
        (Field::Provenance, offered.provenance.is_some()),
        (Field::Trace, offered.trace.is_some()),
        (Field::Evidence, offered.evidence.is_some()),
        (Field::Assumptions, offered.assumptions.is_some()),
    ] {
        if !present {
            return Err(Refusal::MissingField(field));
        }
    }
    if offered.provenance != expected.provenance {
        return Err(Refusal::ProvenanceMismatch);
    }
    if offered.trace != expected.trace {
        return Err(Refusal::TraceMismatch);
    }
    if offered.evidence != expected.evidence {
        return Err(Refusal::EvidenceMismatch);
    }
    if offered.assumptions != expected.assumptions {
        return Err(Refusal::AssumptionsMismatch);
    }
    Ok(())
}
fn quantity(
    dimension: super::Dimension,
    expected: &Measurement,
    actual: &Measurement,
) -> Result<(), Refusal> {
    match (expected, actual) {
        (
            Measurement::Present { value, unit },
            Measurement::Present {
                value: other_value,
                unit: other_unit,
            },
        ) => {
            if unit != other_unit {
                return Err(Refusal::QuantityUnit(dimension));
            }
            if value != other_value {
                return Err(Refusal::QuantityValue(dimension));
            }
        }
        (Measurement::Absent(a), Measurement::Absent(b)) if a == b => (),
        (Measurement::Absent(_) | Measurement::Present { .. }, _) => {
            return Err(Refusal::QuantityAvailability(dimension));
        }
    }
    Ok(())
}

fn body(expected: &Body, offered: &Body) -> Result<(), Refusal> {
    match (expected, offered) {
        (Body::Reading(a), Body::Reading(b)) => reading(a, b, complete),
        (Body::Acceptance(a), Body::Acceptance(b)) => {
            if a.rule != b.rule {
                return Err(Refusal::RuleMismatch);
            }
            if a.epoch != b.epoch {
                return Err(Refusal::ObservationMismatch);
            }
            reading(&a.reading, &b.reading, accepted)
        }
        _ => Err(Refusal::BodyMismatch),
    }
}
fn reading<T: PartialEq>(
    a: &ReadingBody<T>,
    b: &ReadingBody<T>,
    compare: fn(&T, &T) -> Result<(), Refusal>,
) -> Result<(), Refusal> {
    if a.trace != b.trace {
        return Err(Refusal::TraceMismatch);
    }
    if a.incomplete != b.incomplete {
        return Err(Refusal::IncompleteMismatch);
    }
    if a.unexamined != b.unexamined {
        return Err(Refusal::UnexaminedMismatch);
    }
    match (&a.outcome, &b.outcome) {
        (Outcome::Complete(a), Outcome::Complete(b)) => compare(a, b),
        (a, b) if a == b => Ok(()),
        _ => Err(Refusal::OutcomeMismatch),
    }
}
pub(super) fn complete(a: &CompleteReading, b: &CompleteReading) -> Result<(), Refusal> {
    if a.scope != b.scope || a.finality != b.finality {
        return Err(Refusal::OutcomeMismatch);
    }
    if a.membership != b.membership {
        return Err(Refusal::MembershipMismatch);
    }
    Ok(())
}
fn accepted(a: &AcceptedReading, b: &AcceptedReading) -> Result<(), Refusal> {
    complete(&a.reading, &b.reading)?;
    if a.accepted != b.accepted {
        return Err(Refusal::AcceptedMismatch);
    }
    if a.annotations != b.annotations {
        return Err(Refusal::AnnotationsMismatch);
    }
    Ok(())
}
