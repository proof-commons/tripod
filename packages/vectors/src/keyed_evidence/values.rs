//! SOURCE output values at creation, not successor values, credits or entitlements.
use crate::keyed_history_reader::{Address, Handle};

/// Explicit asset identity in internal byte order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Asset(pub [u8; 32]);
/// Explicit amount in the target's base unit; not a record credit.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Amount(pub u64);
/// Observation epoch, not transaction time or confirmation depth.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Epoch(pub u64);
/// Why no explicit value is available.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unknown {
    Committed,
    /// Guard for future variants of the target's non-exhaustive field enums.
    Unsupported,
}
/// Unknown values cannot be confused with explicit zero.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Value {
    Explicit(Amount),
    Unknown(Unknown),
}
/// An asset commitment is not an explicit asset identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssetValue {
    Explicit(Asset),
    Unknown(Unknown),
}
/// This reader assigns no ownership or attribution.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Attribution {
    Unassigned,
}
/// Optional SOURCE output-zero annotation bound to its creating bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Annotation {
    /// Authenticated SOURCE identity, not the currently queried handle.
    pub record: Handle,
    /// Creating transaction's stripped content identity.
    pub creating: Address,
    /// Creating bytes, retained for provenance.
    pub creating_bytes: Vec<u8>,
    /// Explicit value if available, otherwise a named unknown.
    pub value: Value,
    /// Explicit asset if available, otherwise a named unknown.
    pub asset: AssetValue,
    /// No owner is inferred from an absent attribution.
    pub attribution: Attribution,
}
/// These conversions do not introduce confidential construction support.
pub const fn value(value: transaction::ValueField) -> Value {
    match value {
        transaction::ValueField::Explicit(n) => Value::Explicit(Amount(n)),
        transaction::ValueField::Commitment(_) => Value::Unknown(Unknown::Committed),
        _ => Value::Unknown(Unknown::Unsupported),
    }
}
pub const fn asset(value: transaction::AssetField) -> AssetValue {
    match value {
        transaction::AssetField::Explicit(n) => AssetValue::Explicit(Asset(*n.internal())),
        transaction::AssetField::Commitment(_) => AssetValue::Unknown(Unknown::Committed),
        _ => AssetValue::Unknown(Unknown::Unsupported),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keyed_evidence::{Rule, acceptance};
    use crate::keyed_history_reader::{Key, Output};
    use std::collections::{BTreeMap, BTreeSet};

    #[test]
    fn committed_amount_is_unknown_and_never_explicit_zero_or_an_owner() {
        let record = Handle {
            key: Key([42; 32]),
            output: Output {
                event: 1,
                position: 0,
            },
        };
        let annotation = Annotation {
            record,
            creating: Address([7; 32]),
            creating_bytes: vec![],
            value: value(transaction::ValueField::Explicit(0)),
            asset: AssetValue::Explicit(Asset([3; 32])),
            attribution: Attribution::Unassigned,
        };
        let rule = Rule::Fixed {
            asset: Asset([3; 32]),
            threshold: Amount(0),
        };
        let mut annotations = BTreeMap::from([(record, annotation)]);
        assert_eq!(
            acceptance::accepted(&rule, Epoch(0), &annotations),
            Ok(BTreeSet::from([record]))
        );
        annotations.get_mut(&record).unwrap().value =
            value(transaction::ValueField::Commitment([8; 33]));
        assert_eq!(
            annotations[&record].value,
            Value::Unknown(Unknown::Committed)
        );
        assert_eq!(
            acceptance::accepted(&rule, Epoch(0), &annotations),
            Ok(BTreeSet::new())
        );
        assert_eq!(annotations[&record].attribution, Attribution::Unassigned);
        assert_eq!(
            asset(transaction::AssetField::Commitment([10; 33])),
            AssetValue::Unknown(Unknown::Committed)
        );
        assert_eq!(
            acceptance::accepted(&Rule::AllSourceRecords, Epoch(0), &annotations),
            Ok(BTreeSet::from([record]))
        );
    }
}
