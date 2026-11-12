//! Versioned internal report framing; not a serialization or interchange contract.
use super::{
    Annotation, Assumption, DeclarationOffer, Finality, FinalityRequest, PremiseSources,
    Provenance, Rule, Scope,
};
use crate::keyed_history_reader::{Address, EventId, Handle, Input};
use std::collections::{BTreeMap, BTreeSet};

/// Internal schema version, pinned by the framing controls.
pub const SCHEMA: u32 = 1;
/// The six appendix evidence families. Their semantic validators land in step 4.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Family {
    Birth,
    Continuation,
    Recurrence,
    Reading,
    Acceptance,
    Resources,
}
/// This step implements reading and acceptance reports, not the other families.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReadingFamily {
    Reading,
    Acceptance,
}
/// Paper label identity, separate from a delivery requirement identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaperId(pub String);
/// Delivery requirement identity from the eight-row trace.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RequirementId(pub u8);
/// Trace identifiers are fixed by the selected family, not arbitrary pass labels.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Trace {
    pub paper: Vec<PaperId>,
    pub requirements: Vec<RequirementId>,
}
/// Inputs to the acceptance layer. All provenance remains an external reading.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Request {
    pub handle: Handle,
    pub scope: Scope,
    pub family: ReadingFamily,
    pub completeness: Completeness,
    pub rule: Rule,
    /// Time-varying input to the rule, separate from its immutable identity.
    pub observed_epoch: super::Epoch,
    pub provenance: Provenance,
    pub sources: PremiseSources,
    pub finality: FinalityRequest,
    pub declaration: Option<DeclarationOffer>,
}
/// Whether incomplete and non-handle observations are reported or refused.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Completeness {
    Required,
    Report,
}
/// Exact creating bytes and identity of the queried handle.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Subject {
    pub handle: Handle,
    pub address: Address,
    pub bytes: Vec<u8>,
}
/// Exact referenced evidence, including witness bytes which txids do not bind.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EvidenceReferences {
    pub events: BTreeMap<EventId, Vec<u8>>,
    pub association: Vec<(EventId, Address)>,
    pub retrieved: BTreeMap<Address, Vec<u8>>,
    pub assertions: crate::keyed_history_reader::Assertions,
    pub boundary_premises: BTreeSet<EventId>,
    pub keys: BTreeSet<crate::keyed_history_reader::Key>,
}
impl From<&Input> for EvidenceReferences {
    fn from(input: &Input) -> Self {
        let mut association = input.association.clone();
        association.sort();
        Self {
            events: input.bytes.clone(),
            association,
            retrieved: input.retrieved.clone(),
            assertions: input.assertions.clone(),
            boundary_premises: input.boundary_premises.clone(),
            keys: input.keys.clone(),
        }
    }
}
/// Observation vocabulary is open for the archived observations of later families.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Observation {
    OfflineReaderEvidence,
}
/// Only a complete reading has a record set.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompleteReading {
    pub scope: Scope,
    pub finality: Finality,
    pub membership: BTreeSet<Handle>,
}
/// Acceptance claims exist only on a complete reading.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AcceptedReading {
    pub reading: CompleteReading,
    pub accepted: BTreeSet<Handle>,
    pub annotations: BTreeMap<Handle, Annotation>,
}
/// Noncomplete outcomes have no record-set field, including in acceptance reports.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome<T> {
    Complete(T),
    Incomplete {
        handle: Handle,
        missing: BTreeSet<crate::keyed_history_reader::Missing>,
    },
    Invalid(crate::keyed_history_reader::Refusal),
    NotHandle(Handle),
}
/// The reading family has no acceptance rule.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReadingBody<T = CompleteReading> {
    pub trace: Trace,
    pub outcome: Outcome<T>,
    pub incomplete: BTreeMap<Handle, BTreeSet<crate::keyed_history_reader::Missing>>,
    pub unexamined: BTreeMap<EventId, BTreeSet<crate::keyed_history_reader::Missing>>,
}
/// Rule identity and epoch belong only to the acceptance family.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AcceptanceBody {
    pub rule: Rule,
    pub epoch: super::Epoch,
    pub reading: ReadingBody<AcceptedReading>,
}
/// Later families add their own bodies, including relation/member evidence.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Body {
    Reading(ReadingBody),
    Acceptance(AcceptanceBody),
}
/// Units cannot be silently interchanged even when represented by integers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unit {
    Records,
    Bytes,
    Operations,
    Items,
    WeightUnits,
    Microseconds,
    Charge,
    Bound,
}
/// Shared framing dimensions; absent resource observations are not zero.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Dimension {
    Records,
    AcceptedRecords,
    SubjectBytes,
    Operations,
    StackItems,
    Weight,
    NativeWall,
    NativeCpu,
    NativeStall,
    ValidationCharge,
    DeploymentBound,
    PackageBound,
}
/// Explicitly absent evidence, distinct from an observed zero.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Absence {
    NotObservedHere,
    NoIndependentPrediction,
}
/// One observed or predicted quantity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Measurement {
    Present { value: u64, unit: Unit },
    Absent(Absence),
}
/// Prediction and observation are separate even when equal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Quantity {
    pub observed: Measurement,
    pub predicted: Measurement,
}
/// Untrusted internal report offer. Optional required fields admit missing-field subjects.
/// Acceptance by the framing validator does not validate the four future families.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report {
    pub header: Header,
    pub body: Body,
}
/// Shared subject-bound frame; no acceptance rule or record set lives here.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Header {
    pub schema: u32,
    pub family: Family,
    pub subject: Option<Subject>,
    pub provenance: Option<Provenance>,
    /// Requirement 8 and its evidence-assumption anchors, common to every family.
    pub trace: Option<Trace>,
    pub evidence: Option<EvidenceReferences>,
    pub assumptions: Option<Vec<Assumption>>,
    pub observation: Observation,
    pub quantities: BTreeMap<Dimension, Quantity>,
}
pub fn header_trace() -> Trace {
    Trace {
        paper: vec![
            PaperId("sec:kpc:interface".into()),
            PaperId("sec:kpc:limitations".into()),
        ],
        requirements: vec![RequirementId(8)],
    }
}
pub fn trace(family: ReadingFamily) -> (Family, Trace) {
    match family {
        ReadingFamily::Reading => (
            Family::Reading,
            Trace {
                paper: vec![
                    PaperId("def:kpc-interface:membership-reading".into()),
                    PaperId("thm:kpc-interface:membership-correctness".into()),
                    PaperId("cor:kpc-interface:path-reading".into()),
                ],
                requirements: vec![RequirementId(6)],
            },
        ),
        ReadingFamily::Acceptance => (
            Family::Acceptance,
            Trace {
                paper: vec![PaperId("rem:kpc-interface:record-evidence".into())],
                requirements: vec![RequirementId(7)],
            },
        ),
    }
}
const fn next_dimension(dimension: Dimension) -> Option<Dimension> {
    match dimension {
        Dimension::Records => Some(Dimension::AcceptedRecords),
        Dimension::AcceptedRecords => Some(Dimension::SubjectBytes),
        Dimension::SubjectBytes => Some(Dimension::Operations),
        Dimension::Operations => Some(Dimension::StackItems),
        Dimension::StackItems => Some(Dimension::Weight),
        Dimension::Weight => Some(Dimension::NativeWall),
        Dimension::NativeWall => Some(Dimension::NativeCpu),
        Dimension::NativeCpu => Some(Dimension::NativeStall),
        Dimension::NativeStall => Some(Dimension::ValidationCharge),
        Dimension::ValidationCharge => Some(Dimension::DeploymentBound),
        Dimension::DeploymentBound => Some(Dimension::PackageBound),
        Dimension::PackageBound => None,
    }
}
pub fn quantities(
    records: Option<usize>,
    accepted: Option<usize>,
    bytes: usize,
) -> Result<BTreeMap<Dimension, Quantity>, super::Refusal> {
    std::iter::successors(Some(Dimension::Records), |dimension| {
        next_dimension(*dimension)
    })
    .map(|dimension| {
        let value = match dimension {
            Dimension::Records => records.map(|n| (n, Unit::Records)),
            Dimension::AcceptedRecords => accepted.map(|n| (n, Unit::Records)),
            Dimension::SubjectBytes => Some((bytes, Unit::Bytes)),
            Dimension::Operations
            | Dimension::StackItems
            | Dimension::Weight
            | Dimension::NativeWall
            | Dimension::NativeCpu
            | Dimension::NativeStall
            | Dimension::ValidationCharge
            | Dimension::DeploymentBound
            | Dimension::PackageBound => None,
        };
        let observed = match value {
            Some((value, unit)) => Measurement::Present {
                value: count(dimension, value as u128)?,
                unit,
            },
            None => Measurement::Absent(Absence::NotObservedHere),
        };
        Ok((
            dimension,
            Quantity {
                observed,
                predicted: Measurement::Absent(Absence::NoIndependentPrediction),
            },
        ))
    })
    .collect()
}

fn count(dimension: Dimension, value: u128) -> Result<u64, super::Refusal> {
    u64::try_from(value).map_err(|_| super::Refusal::QuantityOverflow(dimension))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn report_quantity_conversion_refuses_overflow_without_saturating() {
        assert_eq!(
            count(Dimension::Records, u128::from(u64::MAX)),
            Ok(u64::MAX)
        );
        assert_eq!(
            count(Dimension::Records, u128::from(u64::MAX) + 1),
            Err(crate::keyed_evidence::Refusal::QuantityOverflow(
                Dimension::Records
            ))
        );
    }
}
