//! Exact source censuses and explicit owner-produced region policy.
use crate::{
    engine_constructibility::Availability,
    engine_lifecycle::{ModeKey, ObjectKey},
    engine_records::{OperationKey, RelationKey},
    source::RequiredSourceKind,
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct OperandKey {
    pub relation: RelationKey,
    pub role: u16,
    pub payload: Payload,
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Payload {
    None,
    Family { side: u16, object: ObjectKey },
    Object(ObjectKey),
    Bound { namespace: u16, code: u16 },
    Fact(crate::engine_proof::FactKey),
    Expression { relation: RelationKey, role: u16 },
    Evidence(Evidence),
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Evidence {
    Operator {
        operation: OperationKey,
    },
    ConfidentialValue {
        operation: OperationKey,
        asset: u16,
    },
    SubstrateValue {
        operation: OperationKey,
        asset: u16,
    },
    #[cfg(any(test, feature = "keyed-experimental"))]
    HandleClassification {
        operation: OperationKey,
        object: ObjectKey,
        side: u16,
        minimum: u16,
        maximum: Option<u16>,
    },
    #[cfg(any(test, feature = "keyed-experimental"))]
    SourceClassification {
        operation: OperationKey,
        object: ObjectKey,
    },
    #[cfg(any(test, feature = "keyed-experimental"))]
    MembershipReading {
        operation: OperationKey,
        object: ObjectKey,
        recurrence: Recurrence,
        inclusion: bool,
        retrieval: bool,
        finality_for_final_reading: bool,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[cfg(any(test, feature = "keyed-experimental"))]
pub enum Recurrence {
    Singleton,
    ConsumedUnion,
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Activation {
    Always,
    SponsorPresent,
    Representation { object: ObjectKey, mode: ModeKey },
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SourceRow {
    pub operand: OperandKey,
    pub source: RequiredSourceKind,
    pub availability: Availability,
    pub activation: Activation,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceTable {
    pub relation: RelationKey,
    pub proof_namespace: u16,
    pub proof_kind: u16,
    pub expected: Vec<OperandKey>,
    pub rows: Vec<SourceRow>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegionRecord {
    pub operation: OperationKey,
    pub declared_families: BTreeSet<ObjectKey>,
    pub declared_flows: BTreeSet<u16>,
    pub funding_family: Option<ObjectKey>,
    pub erased_amount_family: Option<ObjectKey>,
    pub sponsor_conditional_family: Option<ObjectKey>,
    pub decidable: bool,
}
impl RegionRecord {
    // Direct-stage oracle queries; production consumes the record fields.
    #[cfg(test)]
    pub fn erases_amount(&self, family: ObjectKey) -> bool {
        self.erased_amount_family == Some(family)
    }
    #[cfg(test)]
    pub fn activation(&self, family: ObjectKey) -> Activation {
        if self.sponsor_conditional_family == Some(family) {
            Activation::SponsorPresent
        } else {
            Activation::Always
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceRecords {
    pub tables: Vec<SourceTable>,
    pub regions: Vec<RegionRecord>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SourceError {
    DuplicateOperand(OperandKey),
    MissingOperand(OperandKey),
    ForeignOperand(OperandKey),
    DuplicateRow(OperandKey),
    UndecidableRegion(OperationKey),
}

pub fn analyze_sources(records: &SourceRecords) -> Result<Vec<Vec<SourceRow>>, SourceError> {
    let span = tracing::debug_span!(target: "compiler::engine_source", "analyze_sources", table_count = crate::instrumentation::count(records.tables.len()), region_count = crate::instrumentation::count(records.regions.len()));
    let _entered = span.enter();
    for region in &records.regions {
        if !region.decidable {
            return Err(SourceError::UndecidableRegion(region.operation));
        }
    }
    records.tables.iter().map(assemble_table).collect()
}

fn assemble_table(table: &SourceTable) -> Result<Vec<SourceRow>, SourceError> {
    let mut expected = BTreeSet::new();
    for operand in &table.expected {
        if operand.relation != table.relation {
            return Err(SourceError::ForeignOperand(operand.clone()));
        }
        if !expected.insert(operand.clone()) {
            return Err(SourceError::DuplicateOperand(operand.clone()));
        }
    }
    let mut rows = BTreeMap::new();
    for row in &table.rows {
        if !expected.contains(&row.operand) {
            return Err(SourceError::ForeignOperand(row.operand.clone()));
        }
        if rows.insert(row.operand.clone(), row.clone()).is_some() {
            return Err(SourceError::DuplicateRow(row.operand.clone()));
        }
    }
    for operand in expected {
        if !rows.contains_key(&operand) {
            return Err(SourceError::MissingOperand(operand));
        }
    }
    Ok(rows.into_values().collect())
}
