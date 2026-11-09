//! Source records retain evidence and declaration-owned region policy.
use crate::{
    engine_records::{self, RelationOwner},
    engine_source as core,
    source::RequiredSourceKind,
};
use architecture::{
    OperationId,
    keyed::KeyedManifest,
    semantic::{KeyedOperationId, SemanticOperationId},
};

fn keyed(scope: Vec<KeyedOperationId>) -> crate::BoundCompilerInput<crate::input::KeyedOwner> {
    let manifest = KeyedManifest::probe().expect("binding");
    let source = realization::keyed::KeyedRealizationDraft::derive(&manifest)
        .expect("declaration")
        .validate(&manifest)
        .expect("validation");
    crate::bind_input(
        &manifest,
        source,
        scope.into_iter().map(SemanticOperationId::Keyed).collect(),
        super::bound_input(&[OperationId::CompactAsh]).policy(),
    )
    .expect("binding")
}

#[test]
fn source_and_merge_preserve_evidence_without_a_sponsor_default() {
    for (scope, census, evidence) in [
        (vec![KeyedOperationId::Source], 4, 4),
        (vec![KeyedOperationId::Merge], 5, 3),
        (
            vec![KeyedOperationId::Source, KeyedOperationId::Merge],
            9,
            7,
        ),
    ] {
        let bound = keyed(scope);
        let records = bound.owner().lower_sources().expect("lowering");
        assert_eq!(
            records,
            bound.owner().lower_sources().expect("re-derivation")
        );
        let analyzed = engine_records::analyze_bound_sources(&bound).expect("shared census");
        assert_eq!(analyzed.len(), census);
        assert_eq!(
            analyzed
                .iter()
                .flatten()
                .filter(|row| row.source == RequiredSourceKind::ExternalEvidence)
                .count(),
            evidence
        );
        for region in &records.regions {
            assert_eq!(region.funding_family, None);
            assert_eq!(region.declared_flows, std::collections::BTreeSet::default());
            assert_eq!(region.declared_families.len(), 1);
            for family in &region.declared_families {
                assert!(!region.erases_amount(*family));
                assert_eq!(region.activation(*family), core::Activation::Always);
            }
        }
        assert!(analyzed.iter().flatten().any(|row| matches!(
            row.operand.payload,
            core::Payload::Evidence(core::Evidence::MembershipReading {
                inclusion: true,
                retrieval: true,
                finality_for_final_reading: true,
                ..
            })
        )));
    }
    let bound = super::bound_input(&[OperationId::CompactAsh, OperationId::TransferLive]);
    let records = bound.owner().lower_sources().expect("attestation lowering");
    assert_ne!(
        engine_records::analyze_bound_sources(&bound)
            .expect("same census")
            .len(),
        0
    );
    for region in records.regions {
        let family = region.funding_family.expect("declared fee sponsor");
        assert!(region.erases_amount(family));
        assert_eq!(region.activation(family), core::Activation::SponsorPresent);
    }
}

#[test]
fn source_census_refuses_missing_duplicate_and_foreign_rows() {
    let bound = keyed(vec![KeyedOperationId::Source]);
    let control = bound.owner().lower_sources().expect("lowering");
    assert!(core::analyze_sources(&control).is_ok());
    let mut missing = control.clone();
    let removed = missing.tables[0].rows.remove(0);
    assert_eq!(
        core::analyze_sources(&missing).err(),
        Some(core::SourceError::MissingOperand(removed.operand))
    );
    let mut duplicate = control.clone();
    let row = duplicate.tables[0].rows[0].clone();
    duplicate.tables[0].rows.push(row.clone());
    assert_eq!(
        core::analyze_sources(&duplicate).err(),
        Some(core::SourceError::DuplicateRow(row.operand))
    );
    let mut foreign = control.clone();
    foreign.tables[0].rows[0]
        .operand
        .relation
        .operation
        .namespace = 1;
    assert_eq!(
        core::analyze_sources(&foreign).err(),
        Some(core::SourceError::ForeignOperand(
            foreign.tables[0].rows[0].operand.clone()
        ))
    );
    let mut undecidable = control;
    undecidable.regions[0].decidable = false;
    assert_eq!(
        core::analyze_sources(&undecidable).err(),
        Some(core::SourceError::UndecidableRegion(
            undecidable.regions[0].operation
        ))
    );
}
