//! The Keyed Provenance Covenant SOURCE and MERGE declarations, with no execution invented for SOURCE.

use crate::TransactionSide;
use crate::keyed::KeyedOperationRealization;
use crate::relation::{
    KeyedCardinality, KeyedRelation, KeyedRelationDeclaration, KeyedRelationId, RecordRecurrence,
};
use architecture::semantic::{
    KeyedObjectId, KeyedOperationId, SemanticObjectId, SemanticOperationId,
};

pub fn source() -> KeyedOperationRealization {
    declaration(
        KeyedOperationId::Source,
        &[
            KeyedRelation::HandleFamily {
                side: TransactionSide::Input,
                cardinality: KeyedCardinality::Zero,
            },
            KeyedRelation::HandleFamily {
                side: TransactionSide::Output,
                cardinality: KeyedCardinality::One,
            },
            KeyedRelation::SourceClassification,
            KeyedRelation::MembershipReading {
                recurrence: RecordRecurrence::SourceSingleton,
            },
        ],
    )
}

pub fn merge() -> KeyedOperationRealization {
    declaration(
        KeyedOperationId::Merge,
        &[
            KeyedRelation::HandleFamily {
                side: TransactionSide::Input,
                cardinality: KeyedCardinality::OneOrMore,
            },
            KeyedRelation::HandleFamily {
                side: TransactionSide::Output,
                cardinality: KeyedCardinality::One,
            },
            KeyedRelation::BornFirst,
            KeyedRelation::ContinueFirst,
            KeyedRelation::MembershipReading {
                recurrence: RecordRecurrence::ConsumedUnion,
            },
        ],
    )
}

fn declaration(
    operation: KeyedOperationId,
    relations: &[KeyedRelation],
) -> KeyedOperationRealization {
    let operation = SemanticOperationId::Keyed(operation);
    KeyedOperationRealization {
        operation,
        relations: relations
            .iter()
            .map(|relation| KeyedRelationDeclaration {
                id: KeyedRelationId {
                    operation,
                    object: SemanticObjectId::Keyed(KeyedObjectId::Handle),
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
