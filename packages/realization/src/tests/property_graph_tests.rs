use architecture::{ARCHITECTURE, OperationId};
use proptest::prelude::*;

use crate::{RealizationScope, derive, project_scoped_realization};

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 256,
        .. ProptestConfig::default()
    })]

    #[test]
    fn phase1_scope_permutations_preserve_projection(reverse in any::<bool>()) {
        let operations = if reverse {
            vec![OperationId::CompactAsh, OperationId::TransferLive]
        } else {
            vec![OperationId::TransferLive, OperationId::CompactAsh]
        };
        let actual = derive(&ARCHITECTURE, RealizationScope::from_operations(operations).unwrap()).unwrap();
        let expected = derive(&ARCHITECTURE, RealizationScope::phase1_pilots()).unwrap();

        prop_assert_eq!(
            project_scoped_realization(&actual),
            project_scoped_realization(&expected),
        );
    }
}

proptest! {
    #[test]
    fn three_operation_permutations_preserve_projection(keys in any::<[u8; 3]>()) {
        let operations = [OperationId::AnnounceMaturity, OperationId::CompactAsh, OperationId::TransferLive];
        let mut keyed = keys.into_iter().zip(operations).collect::<Vec<_>>();
        keyed.sort();
        let actual = derive(&ARCHITECTURE,
            RealizationScope::from_operations(keyed.into_iter().map(|(_, operation)| operation)).unwrap()).unwrap();
        let expected = derive(&ARCHITECTURE, RealizationScope::from_operations(operations).unwrap()).unwrap();
        super::announce_maturity_tests::assert_permutations(&actual);
        prop_assert_eq!(actual.project(), expected.project());
    }
}
