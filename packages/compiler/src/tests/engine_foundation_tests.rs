//! The same predicate join rejects one-coordinate subjects in either namespace.
use crate::{
    engine_foundation as core,
    engine_records::{OperationKey, RelationKey, RelationOwner},
};
use std::collections::BTreeMap;

#[test]
fn predicate_join_checks_existence_type_and_complete_scope_identity() {
    for namespace in [1, 2] {
        let operation = OperationKey { namespace, code: 1 };
        let relation = RelationKey {
            operation,
            kind: 0,
            subject: Vec::new(),
        };
        let expression = core::ExpressionKey::Relation {
            relation: relation.clone(),
            role: 0,
        };
        let control = core::Records {
            predicates: vec![(relation.clone(), expression.clone())],
            expressions: BTreeMap::from([(
                expression.clone(),
                core::Expression {
                    operation: Some(operation),
                    boolean: true,
                },
            )]),
        };
        assert_eq!(core::validate_predicates(&control), Ok(()));
        let mut missing = control.clone();
        missing.expressions.clear();
        assert_eq!(
            core::validate_predicates(&missing),
            Err(core::Error::UnknownExpression {
                relation: relation.clone(),
                expression: expression.clone()
            })
        );
        let mut non_boolean = control.clone();
        non_boolean
            .expressions
            .get_mut(&expression)
            .expect("control expression")
            .boolean = false;
        assert_eq!(
            core::validate_predicates(&non_boolean),
            Err(core::Error::NonBoolean {
                relation: relation.clone(),
                expression: expression.clone()
            })
        );
        let mut foreign = control.clone();
        foreign
            .expressions
            .get_mut(&expression)
            .expect("control expression")
            .operation = Some(OperationKey {
            namespace: 3 - namespace,
            code: 1,
        });
        assert_eq!(
            core::validate_predicates(&foreign),
            Err(core::Error::OutsideScope {
                relation,
                expression: expression.clone()
            })
        );
        let mut independent = control;
        independent
            .expressions
            .get_mut(&expression)
            .expect("control expression")
            .operation = None;
        assert_eq!(core::validate_predicates(&independent), Ok(()));
    }
}

#[test]
fn bound_owners_explicitly_lower_the_empty_predicate_stage() {
    use architecture::{
        OperationId,
        keyed::KeyedManifest,
        semantic::{KeyedOperationId, SemanticOperationId},
    };
    let attestation = super::bound_input(&[OperationId::CompactAsh]);
    let records = crate::input::AttestationOwner::lower_predicates(&attestation)
        .expect("attestation predicates");
    assert_eq!(records.predicates, []);
    assert_eq!(core::validate_predicates(&records), Ok(()));
    let manifest = KeyedManifest::contract().expect("manifest");
    let realization = realization::keyed::KeyedRealizationDraft::derive(&manifest)
        .expect("derive")
        .validate(&manifest)
        .expect("validate");
    let keyed = crate::bind_input(
        &manifest,
        realization,
        vec![
            SemanticOperationId::Keyed(KeyedOperationId::Source),
            SemanticOperationId::Keyed(KeyedOperationId::Merge),
        ],
        attestation.policy(),
    )
    .expect("bind");
    let records = crate::input::KeyedOwner::lower_predicates(&keyed).expect("keyed predicates");
    assert_eq!(records.predicates, []);
    assert!(records.expressions.is_empty());
    assert_eq!(core::validate_predicates(&records), Ok(()));
}
