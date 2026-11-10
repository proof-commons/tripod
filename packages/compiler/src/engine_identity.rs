//! Compiler-private identity vocabulary shared by both lowering owners.
//!
//! Architecture owns semantic namespace and local identifier encodings. These
//! adapters retain both components; transaction sides have one compiler spelling.
//! Pure encoders and read-only accessors add no telemetry work: their calls
//! remain inside the existing binding and lowering spans.

use architecture::semantic::SemanticProfileId;
use architecture::semantic::{SemanticObjectId, SemanticOperationId};
use realization::TransactionSide;

pub const ATTESTATION: u16 = namespace(SemanticProfileId::Attestation);

pub const fn namespace(profile: SemanticProfileId) -> u16 {
    profile as u16
}

pub const fn operation(id: SemanticOperationId) -> [u16; 2] {
    id.encoding()
}

pub const fn object(id: SemanticObjectId) -> [u16; 2] {
    id.encoding()
}

pub const fn side_code(side: TransactionSide) -> u16 {
    match side {
        TransactionSide::Input => 0,
        TransactionSide::Output => 1,
    }
}

// Equality and ordering are available to the engine; identity contents have
// no accessor. Only constructors accept parts, so nested encoders must work
// from their retained owner values, never decode an existing key.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Identity {
    namespace: u16,
    domain: u16,
    payload: Box<[Part]>,
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Part {
    Word(u64),
    Sequence(Vec<Self>),
}
pub fn word(value: impl Into<u64>) -> Part {
    Part::Word(value.into())
}
pub fn sequence(value: impl IntoIterator<Item = Part>) -> Part {
    Part::Sequence(value.into_iter().collect())
}
fn identity(profile: SemanticProfileId, domain: u16, payload: Vec<Part>) -> Identity {
    Identity {
        namespace: namespace(profile),
        domain,
        payload: payload.into_boxed_slice(),
    }
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CaseKey(Identity);
impl CaseKey {
    pub fn new(profile: SemanticProfileId, payload: Vec<Part>) -> Self {
        Self(identity(profile, 1, payload))
    }
    pub fn part(profile: SemanticProfileId, payload: Vec<Part>) -> Part {
        sequence([word(namespace(profile)), word(1u16), sequence(payload)])
    }
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CarrierKey(Identity);
impl CarrierKey {
    pub fn new(profile: SemanticProfileId, payload: Vec<Part>) -> Self {
        Self(identity(profile, 2, payload))
    }
    pub fn part(profile: SemanticProfileId, payload: Vec<Part>) -> Part {
        sequence([word(namespace(profile)), word(2u16), sequence(payload)])
    }
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct LayoutKey(Identity);
impl LayoutKey {
    pub fn new(profile: SemanticProfileId, payload: Vec<Part>) -> Self {
        Self(identity(profile, 3, payload))
    }
    #[cfg(test)]
    pub fn part(profile: SemanticProfileId, payload: Vec<Part>) -> Part {
        sequence([word(namespace(profile)), word(3u16), sequence(payload)])
    }
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PurposeKey(Identity);
impl PurposeKey {
    pub fn new(profile: SemanticProfileId, payload: Vec<Part>) -> Self {
        Self(identity(profile, 4, payload))
    }
    #[cfg(test)]
    pub fn part(profile: SemanticProfileId, payload: Vec<Part>) -> Part {
        sequence([word(namespace(profile)), word(4u16), sequence(payload)])
    }
}
#[cfg(test)]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SymbolKey(Identity);
#[cfg(test)]
impl SymbolKey {
    pub fn new(profile: SemanticProfileId, payload: Vec<Part>) -> Self {
        Self(identity(profile, 5, payload))
    }
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct AuthorizationKey(Identity);
impl AuthorizationKey {
    pub fn new(profile: SemanticProfileId, payload: Vec<Part>) -> Self {
        Self(identity(profile, 6, payload))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use architecture::semantic::{KeyedObjectId, KeyedOperationId};
    use architecture::{ObjectId, OperationId};

    #[test]
    fn identity_positions_are_distinct_rust_types() {
        use std::{any::TypeId, collections::BTreeSet};
        let positions = [
            TypeId::of::<CaseKey>(),
            TypeId::of::<CarrierKey>(),
            TypeId::of::<LayoutKey>(),
            TypeId::of::<PurposeKey>(),
            TypeId::of::<AuthorizationKey>(),
            TypeId::of::<SymbolKey>(),
        ];
        assert_eq!(positions.into_iter().collect::<BTreeSet<_>>().len(), 6);
    }

    #[test]
    fn structural_keys_keep_owner_domain_and_complete_nested_contents_distinct() {
        let accepted = identity(
            SemanticProfileId::Attestation,
            1,
            vec![word(1u16), sequence([word(2u16)])],
        );
        assert_eq!(
            accepted,
            identity(
                SemanticProfileId::Attestation,
                1,
                vec![word(1u16), sequence([word(2u16)])]
            )
        );
        for subject in [
            identity(
                SemanticProfileId::KeyedProvenance,
                1,
                vec![word(1u16), sequence([word(2u16)])],
            ),
            identity(
                SemanticProfileId::Attestation,
                2,
                vec![word(1u16), sequence([word(2u16)])],
            ),
            identity(
                SemanticProfileId::Attestation,
                1,
                vec![word(1u16), sequence([word(3u16)])],
            ),
            identity(
                SemanticProfileId::Attestation,
                1,
                vec![sequence([word(1u16)]), word(2u16)],
            ),
        ] {
            assert_ne!(accepted, subject);
        }
        // Full-width values and an empty nested sequence are retained.
        assert_ne!(
            identity(SemanticProfileId::Attestation, 3, vec![word(u64::MAX)]),
            identity(SemanticProfileId::Attestation, 3, vec![word(u64::MAX - 1)])
        );
        assert_ne!(
            identity(SemanticProfileId::Attestation, 3, vec![]),
            identity(SemanticProfileId::Attestation, 3, vec![sequence([])])
        );
    }

    #[test]
    fn structural_layout_keys_preserve_owner_order() {
        use std::collections::BTreeSet;
        let input =
            crate::tests::bound_input(&[OperationId::CompactAsh, OperationId::TransferLive]);
        let registered = crate::engine_attestation::cases::lower_registered(&input)
            .expect("accepted owner tables");
        let values = registered
            .layouts
            .values()
            .cloned()
            .collect::<BTreeSet<_>>();
        assert!(!values.is_empty());
        let expected = values
            .iter()
            .map(crate::engine_attestation::identity::attestation_layout)
            .collect::<Vec<_>>();
        let canonical = expected.iter().cloned().collect::<BTreeSet<_>>();
        assert_eq!(expected, canonical.iter().cloned().collect::<Vec<_>>());
        assert_eq!(values.len(), canonical.len());
    }

    #[test]
    fn namespace_and_side_spellings_equal_the_original_literals() {
        assert_eq!(namespace(SemanticProfileId::Attestation), 1);
        assert_eq!(namespace(SemanticProfileId::KeyedProvenance), 2);
        assert_eq!(ATTESTATION, 1);
        assert_eq!(side_code(TransactionSide::Input), 0);
        assert_eq!(side_code(TransactionSide::Output), 1);
        assert_ne!(
            side_code(TransactionSide::Input),
            side_code(TransactionSide::Output)
        );
        // The keyed source lowerer formerly used the enum discriminant directly.
        for side in [TransactionSide::Input, TransactionSide::Output] {
            assert_eq!(side_code(side), side as u16);
        }
    }

    #[test]
    fn all_owner_identity_spellings_preserve_namespace_and_local_code() {
        for &id in OperationId::ALL {
            assert_eq!(
                operation(SemanticOperationId::Attestation(id)),
                [1, id.code()]
            );
            assert_eq!(crate::engine_attestation::operation_key(id).namespace, 1);
            assert_eq!(crate::engine_attestation::operation_key(id).code, id.code());
        }
        for &id in ObjectId::ALL {
            assert_eq!(object(SemanticObjectId::Attestation(id)), [1, id.code()]);
        }
        for (id, code) in [(KeyedOperationId::Source, 1), (KeyedOperationId::Merge, 2)] {
            let semantic = SemanticOperationId::Keyed(id);
            assert_eq!(operation(semantic), [2, code]);
            assert_eq!(crate::engine_keyed::operation_key(semantic).namespace, 2);
            assert_eq!(crate::engine_keyed::operation_key(semantic).code, code);
        }
        assert_eq!(
            object(SemanticObjectId::Keyed(KeyedObjectId::Handle)),
            [2, 1]
        );
    }

    #[test]
    fn identical_local_codes_remain_distinct_across_owners() {
        let attestation = OperationId::ALL
            .iter()
            .copied()
            .find(|id| id.code() == 1)
            .expect("attestation code one");
        let accepted = operation(SemanticOperationId::Attestation(attestation));
        let other_owner = operation(SemanticOperationId::Keyed(KeyedOperationId::Source));
        assert_eq!(accepted[1], other_owner[1]);
        assert_ne!(accepted, other_owner);
        let attestation = ObjectId::ALL
            .iter()
            .copied()
            .find(|id| id.code() == 1)
            .expect("attestation object one");
        let accepted = object(SemanticObjectId::Attestation(attestation));
        let other_owner = object(SemanticObjectId::Keyed(KeyedObjectId::Handle));
        assert_eq!(accepted[1], other_owner[1]);
        assert_ne!(accepted, other_owner);
    }
}
