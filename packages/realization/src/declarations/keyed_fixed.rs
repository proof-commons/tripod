//! The fixed-form Keyed Provenance Covenant SOURCE and MERGE declarations, with no execution invented for SOURCE.

use crate::TransactionSide;
use crate::keyed_fixed::KeyedFixedOperationRealization;
use crate::keyed_fixed_relation::{
    KeyedFixedCardinality, KeyedFixedRelation, KeyedFixedRelationDeclaration, KeyedFixedRelationId,
    RecordRecurrence,
};
use architecture::keyed_fixed::{KeyedFixedObjectId, KeyedFixedOperationId};

pub fn source() -> KeyedFixedOperationRealization {
    declaration(
        KeyedFixedOperationId::Source,
        &[
            KeyedFixedRelation::HandleFamily {
                side: TransactionSide::Input,
                cardinality: KeyedFixedCardinality::Zero,
            },
            KeyedFixedRelation::HandleFamily {
                side: TransactionSide::Output,
                cardinality: KeyedFixedCardinality::One,
            },
            KeyedFixedRelation::SourceClassification,
            KeyedFixedRelation::MembershipReading {
                recurrence: RecordRecurrence::SourceSingleton,
            },
        ],
    )
}

pub fn merge() -> KeyedFixedOperationRealization {
    declaration(
        KeyedFixedOperationId::Merge,
        &[
            KeyedFixedRelation::HandleFamily {
                side: TransactionSide::Input,
                cardinality: KeyedFixedCardinality::OneOrMore,
            },
            KeyedFixedRelation::HandleFamily {
                side: TransactionSide::Output,
                cardinality: KeyedFixedCardinality::One,
            },
            KeyedFixedRelation::BornFirst,
            KeyedFixedRelation::OwnQualification,
            KeyedFixedRelation::SuccessorQualification,
            KeyedFixedRelation::MembershipReading {
                recurrence: RecordRecurrence::ConsumedUnion,
            },
        ],
    )
}

fn declaration(
    operation: KeyedFixedOperationId,
    relations: &[KeyedFixedRelation],
) -> KeyedFixedOperationRealization {
    KeyedFixedOperationRealization {
        operation,
        relations: relations
            .iter()
            .map(|relation| KeyedFixedRelationDeclaration {
                id: KeyedFixedRelationId {
                    operation,
                    object: KeyedFixedObjectId::Handle,
                    kind: relation.kind(),
                },
                relation: *relation,
            })
            .collect(),
        expressions: Vec::new(),
        relation_dependencies: Vec::new(),
        constructibility_nodes: Vec::new(),
        constructibility_edges: Vec::new(),
        lifecycle_nodes: Vec::new(),
        lifecycle_edges: Vec::new(),
        disclosure_nodes: Vec::new(),
        disclosure_edges: Vec::new(),
        disclosure_seeds: Vec::new(),
    }
}
