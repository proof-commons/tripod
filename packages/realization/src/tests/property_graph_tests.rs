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
