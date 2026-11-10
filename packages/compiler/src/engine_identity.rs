//! Compiler-private identity vocabulary shared by both lowering owners.
//!
//! Architecture owns semantic namespace and local identifier encodings. These
//! adapters retain both components; transaction sides have one compiler spelling.
//! Pure const encoders and read-only accessors add no telemetry work: their calls
//! remain inside the existing binding and lowering spans.

use architecture::semantic::SemanticProfileId;
#[cfg(any(test, feature = "keyed-experimental"))]
use architecture::semantic::{SemanticObjectId, SemanticOperationId};
use realization::TransactionSide;

pub const ATTESTATION: u16 = namespace(SemanticProfileId::Attestation);

pub const fn namespace(profile: SemanticProfileId) -> u16 {
    profile as u16
}

#[cfg(any(test, feature = "keyed-experimental"))]
pub const fn operation(id: SemanticOperationId) -> [u16; 2] {
    id.encoding()
}

#[cfg(any(test, feature = "keyed-experimental"))]
pub const fn object(id: SemanticObjectId) -> [u16; 2] {
    id.encoding()
}

pub const fn side_code(side: TransactionSide) -> u16 {
    match side {
        TransactionSide::Input => 0,
        TransactionSide::Output => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use architecture::semantic::{KeyedObjectId, KeyedOperationId};
    use architecture::{ObjectId, OperationId};

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
