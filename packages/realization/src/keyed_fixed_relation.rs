//! Fixed-form relation vocabulary; no attestation or legacy relation bodies are extended.

/// Fixed-form Keyed Provenance Covenant relation vocabulary. Kept distinct from attestation recognition,
/// which requires an asset and its protocol object-shape laws.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum KeyedFixedRelation {
    /// Cardinality of authenticated same-triple handles only. Other inputs and
    /// outputs are unconstrained; a later covenant-shaped output is no handle.
    HandleFamily {
        side: crate::TransactionSide,
        cardinality: KeyedFixedCardinality,
    },
    /// Each consumed handle was born at position zero in its creating event.
    BornFirst,
    /// The spent program and explicit asset/value equal the constant triple.
    OwnQualification,
    /// Output zero has the constant program and exact explicit asset/value pair.
    SuccessorQualification,
    /// Complete input classification proves absence of same-triple handles.
    /// This is history evidence, never a covenant-input predicate.
    SourceClassification,
    MembershipReading {
        recurrence: RecordRecurrence,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum KeyedFixedCardinality {
    Zero,
    One,
    OneOrMore,
}

/// Derived record sets: neither variant is a stored set or an accumulator.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RecordRecurrence {
    /// The singleton record identity is (the manifest's fixed triple, SOURCE handle).
    SourceSingleton,
    /// Union of consumed handles' reconstructed sets; introduces no record.
    ConsumedUnion,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u16)]
pub enum KeyedFixedRelationKind {
    InputFamily = 1,
    OutputFamily = 2,
    BornFirst = 3,
    OwnQualification = 4,
    SuccessorQualification = 5,
    SourceClassification = 6,
    MembershipReading = 7,
}

impl KeyedFixedRelation {
    pub const fn kind(self) -> KeyedFixedRelationKind {
        match self {
            Self::HandleFamily {
                side: crate::TransactionSide::Input,
                ..
            } => KeyedFixedRelationKind::InputFamily,
            Self::HandleFamily {
                side: crate::TransactionSide::Output,
                ..
            } => KeyedFixedRelationKind::OutputFamily,
            Self::BornFirst => KeyedFixedRelationKind::BornFirst,
            Self::OwnQualification => KeyedFixedRelationKind::OwnQualification,
            Self::SuccessorQualification => KeyedFixedRelationKind::SuccessorQualification,
            Self::SourceClassification => KeyedFixedRelationKind::SourceClassification,
            Self::MembershipReading { .. } => KeyedFixedRelationKind::MembershipReading,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct KeyedFixedRelationId {
    pub operation: architecture::keyed_fixed::KeyedFixedOperationId,
    pub object: architecture::keyed_fixed::KeyedFixedObjectId,
    pub kind: KeyedFixedRelationKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct KeyedFixedRelationDeclaration {
    pub id: KeyedFixedRelationId,
    pub relation: KeyedFixedRelation,
}

impl KeyedFixedRelationId {
    /// Namespace-qualified operation/object and explicit kind (including side).
    pub const fn encoding(self) -> [u16; 5] {
        let operation = self.operation.encoding();
        let object = self.object.encoding();
        [
            operation[0],
            operation[1],
            object[0],
            object[1],
            self.kind as u16,
        ]
    }
}
