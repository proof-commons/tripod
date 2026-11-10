//! Aggregate and per-relation evidence cannot silently diverge.
use crate::{
    engine_records::{OperationKey, RelationKey},
    engine_requirement as core,
    engine_source::Evidence,
};
use std::collections::{BTreeMap, BTreeSet};

#[test]
fn proof_bundles_keep_source_and_constructibility_refusals() {
    let input =
        super::engine_coverage_tests::bound(vec![architecture::semantic::KeyedOperationId::Source]);
    let control = crate::input::KeyedOwner::lower_proofs(&input).expect("proof records");
    assert!(core::proof_tables(&control).is_ok());
    for constructibility in [true, false] {
        let mut subject = control.clone();
        let alternative = subject
            .obligations
            .values_mut()
            .find_map(|obligation| match obligation {
                crate::engine_proof::Obligation::Evidence(value) => Some(value),
                _ => None,
            })
            .expect("fixed evidence");
        let expected = if constructibility {
            alternative.constructible = false;
            core::ProofBundleError::Constructibility(alternative.id.clone())
        } else {
            alternative.sources = Err(crate::proof::LocalProofRejection::SourceDerivation);
            core::ProofBundleError::Source {
                alternative: alternative.id.clone(),
                reason: crate::proof::LocalProofRejection::SourceDerivation,
            }
        };
        assert_eq!(core::proof_tables(&subject).err(), Some(expected));
    }
}

#[test]
fn keyed_requirement_tables_join_selected_proofs_and_retain_every_source() {
    use architecture::semantic::KeyedOperationId;
    for (scope, count, evidence) in [
        (vec![KeyedOperationId::Source], 4, 4),
        (vec![KeyedOperationId::Merge], 5, 3),
        (
            vec![KeyedOperationId::Source, KeyedOperationId::Merge],
            9,
            7,
        ),
    ] {
        let input = super::engine_coverage_tests::bound(scope);
        let tables =
            crate::input::KeyedOwner::lower_requirements(&input).expect("tables before selection");
        let (candidates, _) = crate::engine_records::analyze_bound_proofs(
            &input,
            &crate::capability::CapabilityView::Unconstrained,
        )
        .expect("proofs");
        let candidate = &candidates[0];
        let owned = core::derive(&tables, &candidate.proofs).expect("owned requirements");
        assert_eq!(owned.len(), count);
        let aggregate = core::Bundle {
            capabilities: candidate.capabilities.clone(),
            sources: candidate.sources.iter().cloned().collect(),
            evidence: candidate.evidence.clone(),
        };
        assert_eq!(aggregate.evidence.len(), evidence);
        assert_eq!(core::validate_aggregate(&aggregate, &owned), Ok(()));
        if let Some((relation, _)) = candidate.proofs.first_key_value() {
            let mut missing = candidate.proofs.clone();
            missing.remove(relation);
            assert_eq!(
                core::derive(&tables, &missing).err(),
                Some(core::SelectionError::Missing(relation.clone()))
            );
            let mut unapproved = candidate.proofs.clone();
            unapproved.get_mut(relation).expect("selected").kind = u16::MAX;
            assert_eq!(
                core::derive(&tables, &unapproved).err(),
                Some(core::SelectionError::Unapproved(relation.clone()))
            );
        }
        let fixed = tables
            .iter()
            .find(|row| !row.required)
            .expect("evidence table");
        let mut unexpected = candidate.proofs.clone();
        unexpected.insert(
            fixed.relation.clone(),
            crate::engine_proof::AlternativeKey {
                relation: fixed.relation.clone(),
                namespace: 2,
                kind: 0,
            },
        );
        assert_eq!(
            core::derive(&tables, &unexpected).err(),
            Some(core::SelectionError::Unexpected(fixed.relation.clone()))
        );
    }
}

#[test]
fn aggregate_evidence_requires_exact_namespaced_ownership() {
    for namespace in [1, 2] {
        let operation = OperationKey { namespace, code: 1 };
        let relation = RelationKey {
            operation,
            kind: 0,
            subject: Vec::new(),
        };
        let evidence = Evidence::Operator { operation };
        let control = core::Bundle {
            evidence: BTreeSet::from([evidence.clone()]),
            ..core::Bundle::default()
        };
        let owned = BTreeMap::from([(relation.clone(), control.clone())]);
        assert_eq!(core::validate_aggregate(&control, &owned), Ok(()));
        let mut missing = control.clone();
        missing.evidence.clear();
        assert_eq!(
            core::validate_aggregate(&missing, &owned),
            Err(core::Error::Evidence {
                relation: Some(relation),
                evidence: evidence.clone()
            })
        );
        let mut extra = control.clone();
        let foreign = Evidence::Operator {
            operation: OperationKey {
                namespace: 3 - namespace,
                code: 1,
            },
        };
        extra.evidence.insert(foreign.clone());
        assert_eq!(
            core::validate_aggregate(&extra, &owned),
            Err(core::Error::Evidence {
                relation: None,
                evidence: foreign
            })
        );
        assert_eq!(
            core::validate_aggregate(&control, &BTreeMap::new()),
            Err(core::Error::Evidence {
                relation: None,
                evidence
            })
        );
    }
}
