//! Acceptance filters membership; it never rewrites a reader result.
use super::{Amount, Annotation, Asset, AssetValue, Attribution, Epoch, Refusal, Value, values};
use crate::keyed_history_reader::{Handle, Input, Membership, Reading};
use std::collections::{BTreeMap, BTreeSet};

/// Rule identity includes immutable parameters, separately from the observed epoch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Rule {
    AllSourceRecords,
    Fixed {
        asset: Asset,
        threshold: Amount,
    },
    /// Only the named record toggles. There is no monotonicity promise.
    Changing {
        record: Handle,
        enabled_epoch: Epoch,
        disabled_epoch: Epoch,
    },
}
/// Completeness of one dependency closure versus all supplied events.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scope {
    HandleClosure,
    WholeHistory,
}
pub fn failures(reading: &Reading) -> (super::MissingHandles, super::MissingEvents) {
    let handles = reading
        .memberships()
        .iter()
        .filter_map(|(handle, state)| match state {
            Membership::Incomplete(missing) => Some((*handle, missing.clone())),
            Membership::Complete(_) => None,
        })
        .collect();
    let events = reading
        .unexamined()
        .iter()
        .filter(|(_, missing)| !missing.is_empty())
        .map(|(event, missing)| (*event, missing.clone()))
        .collect();
    (handles, events)
}
#[cfg(test)]
pub fn annotations(
    input: &Input,
    records: &BTreeSet<Handle>,
) -> Result<BTreeMap<Handle, Annotation>, Refusal> {
    annotations_indexed(input, records, None)
}
pub fn annotations_indexed(
    input: &Input,
    records: &BTreeSet<Handle>,
    index: Option<&BTreeMap<crate::keyed_history_reader::EventId, transaction::TargetTransaction>>,
) -> Result<BTreeMap<Handle, Annotation>, Refusal> {
    records
        .iter()
        .map(|record| {
            let bytes = input
                .bytes
                .get(&record.output.event)
                .ok_or(Refusal::AnnotationMissing(*record))?;
            let parent = input
                .association
                .iter()
                .find(|(id, _)| *id == record.output.event)
                .ok_or(Refusal::AnnotationMissing(*record))?
                .1;
            let owned;
            let decoded = if let Some(index) = index {
                index
                    .get(&record.output.event)
                    .ok_or(Refusal::AnnotationMissing(*record))?
            } else {
                owned = transaction::TargetTransaction::decode(bytes)
                    .map_err(Refusal::AnnotationDecode)?;
                &owned
            };
            let output = decoded
                .outputs()
                .first()
                .ok_or(Refusal::AnnotationMissing(*record))?;
            Ok((
                *record,
                Annotation {
                    record: *record,
                    creating: parent,
                    creating_bytes: bytes.clone(),
                    value: values::value(output.value()),
                    asset: values::asset(output.asset()),
                    attribution: Attribution::Unassigned,
                },
            ))
        })
        .collect()
}
pub fn accepted(
    rule: &Rule,
    observed_epoch: Epoch,
    annotations: &BTreeMap<Handle, Annotation>,
) -> Result<BTreeSet<Handle>, Refusal> {
    if let Rule::Changing {
        enabled_epoch,
        disabled_epoch,
        ..
    } = rule
    {
        if enabled_epoch == disabled_epoch {
            return Err(Refusal::SameEpoch);
        }
        if observed_epoch != *enabled_epoch && observed_epoch != *disabled_epoch {
            return Err(Refusal::UnknownEpoch);
        }
    }
    Ok(annotations.iter().filter_map(|(handle, annotation)| {
        let include = match rule {
            Rule::AllSourceRecords => true,
            Rule::Fixed { asset, threshold } => matches!((annotation.asset, annotation.value), (AssetValue::Explicit(actual), Value::Explicit(value)) if actual == *asset && value >= *threshold),
            Rule::Changing { record, enabled_epoch, .. } => handle != record || *enabled_epoch == observed_epoch,
        };
        include.then_some(*handle)
    }).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keyed_history_reader::{Address, Assertions, Key, Output};

    #[test]
    fn annotation_internal_missing_and_decode_guards_preserve_owner_errors() {
        // Synthetic helper subject, not a reader-accepted or admitted history.
        let record = Handle {
            key: Key([42; 32]),
            output: Output {
                event: 1,
                position: 0,
            },
        };
        let transaction = transaction::TargetTransaction::new(
            2,
            vec![transaction::TargetInput::new(
                transaction::Outpoint::new(transaction::Txid::from_internal([7; 32]), 0).unwrap(),
                1,
            )],
            vec![transaction::TargetOutput::new(
                transaction::AssetField::Explicit(transaction::AssetId::from_internal([3; 32])),
                transaction::ValueField::Explicit(0),
                transaction::NonceField::Null,
                vec![0x51],
            )],
            0,
            vec![transaction::InputWitness::new(vec![])],
        )
        .unwrap();
        let input = Input {
            bytes: BTreeMap::from([(1, transaction.encode())]),
            association: vec![(1, Address([9; 32]))],
            keys: BTreeSet::from([record.key]),
            retrieved: BTreeMap::new(),
            boundary_premises: BTreeSet::new(),
            assertions: Assertions::default(),
        };
        let records = BTreeSet::from([record]);
        let control = annotations(&input, &records).unwrap();
        assert_eq!(control[&record].value, Value::Explicit(Amount(0)));
        let mut absent = input.clone();
        absent.bytes.clear();
        assert_eq!(
            annotations(&absent, &records),
            Err(Refusal::AnnotationMissing(record))
        );
        let mut unnamed = input.clone();
        unnamed.association.clear();
        assert_eq!(
            annotations(&unnamed, &records),
            Err(Refusal::AnnotationMissing(record))
        );
        let mut malformed = input;
        malformed.bytes.insert(1, Vec::new());
        let error = transaction::TargetTransaction::decode(&[]).unwrap_err();
        assert_eq!(
            annotations(&malformed, &records),
            Err(Refusal::AnnotationDecode(error))
        );
    }
}
