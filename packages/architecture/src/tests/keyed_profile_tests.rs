use crate::keyed::{KEYED_ANCHORS, KeyedManifest, KeyedManifestError};
use crate::semantic::{
    KeyedObjectId, KeyedOperationId, SemanticObjectId, SemanticOperationId, SemanticProfileId,
};

#[test]
fn keyed_profile_binding_corruption_has_an_accepted_control() {
    let control = KeyedManifest::probe().unwrap();
    let binding = crate::validate_draft(&control).unwrap().binding();
    assert_eq!(binding.profile, SemanticProfileId::KeyedProvenance);
    let mut subject = control;
    subject.profile = SemanticProfileId::Attestation;
    assert_eq!(
        crate::validate_draft(&subject).unwrap_err(),
        KeyedManifestError::ProfileBinding
    );
}

#[test]
fn a_foreign_operation_cannot_enter_the_keyed_manifest() {
    let control = KeyedManifest::probe().unwrap();
    crate::validate_draft(&control).unwrap();
    let mut subject = control;
    subject.operations[0] = SemanticOperationId::Attestation(crate::OperationId::CreateRequest);
    assert_eq!(
        crate::validate_draft(&subject).unwrap_err(),
        KeyedManifestError::OperationCensus
    );
}

#[test]
fn semantic_keys_keep_namespace_in_equality_ordering_and_encoding() {
    let old_operation = SemanticOperationId::Attestation(crate::OperationId::CreateRequest);
    let new_operation = SemanticOperationId::Keyed(KeyedOperationId::Source);
    assert_ne!(old_operation, new_operation);
    assert_ne!(old_operation.cmp(&new_operation), std::cmp::Ordering::Equal);
    assert_eq!(old_operation.encoding()[1], new_operation.encoding()[1]);
    assert_ne!(old_operation.encoding(), new_operation.encoding());
    let old_object = SemanticObjectId::Attestation(crate::ObjectId::State);
    let new_object = SemanticObjectId::Keyed(KeyedObjectId::Handle);
    assert_ne!(old_object, new_object);
    assert_ne!(old_object.cmp(&new_object), std::cmp::Ordering::Equal);
    assert_eq!(old_object.encoding()[1], new_object.encoding()[1]);
    assert_ne!(old_object.encoding(), new_object.encoding());
}

#[test]
fn keyed_binding_is_order_independent_and_paper_owned() {
    let control = KeyedManifest::probe().unwrap();
    let expected = crate::validate_draft(&control).unwrap().canonical_bytes();
    let mut permuted = control.clone();
    permuted.operations.reverse();
    permuted.clauses.reverse();
    assert_eq!(
        crate::validate_draft(&permuted).unwrap().canonical_bytes(),
        expected
    );
    assert_ne!(
        control.document.anchor_set_hash,
        crate::DOCUMENT.specification.anchor_set_hash.unwrap()
    );
    let sections = [
        include_str!("../../../../papers/lineage/sections/02_model.tex"),
        include_str!("../../../../papers/lineage/sections/03_operations.tex"),
        include_str!("../../../../papers/lineage/sections/01_interface.tex"),
    ]
    .join("\n");
    for anchor in KEYED_ANCHORS {
        assert!(sections.contains(&format!("\\label{{{anchor}}}")));
    }
    let mut subject = control;
    subject.document.anchor_set_hash = crate::DOCUMENT.specification.anchor_set_hash.unwrap();
    assert_eq!(
        crate::validate_draft(&subject).unwrap_err(),
        KeyedManifestError::PaperBinding
    );
}

#[test]
fn keyed_manifest_has_exactly_two_clauses_without_an_input_bound() {
    let control = KeyedManifest::probe().unwrap();
    crate::validate_draft(&control).unwrap();
    let mut subject = control;
    subject.clauses.pop();
    assert_eq!(
        crate::validate_draft(&subject).unwrap_err(),
        KeyedManifestError::ClauseCensus
    );
    assert_eq!(crate::OperationId::ALL.len(), 13);
    assert_eq!(crate::ObjectId::ALL.len(), 14);
    assert_eq!(crate::ARCHITECTURE_SCHEMA_VERSION, 18);
}

#[test]
fn attestation_canonical_bytes_and_publication_equal_the_opening_base() {
    // Frozen bytes were extracted from the opening base's generated artifact,
    // before any semantic source change. See the probe gate provenance record.
    let before: crate::PublishedArchitecture = serde_json::from_str(include_str!(
        "../../../compiler/tests/fixtures/keyed_base_architecture.json"
    ))
    .unwrap();
    before.verify_hashes().unwrap();
    let validated = crate::validate_draft(&crate::ARCHITECTURE).unwrap();
    let before_bytes = serde_json::to_vec(&crate::canonical::canonicalize_json(
        serde_json::to_value(&before.architecture).unwrap(),
    ))
    .unwrap();
    assert_eq!(
        crate::canonical_json_bytes(&validated).unwrap(),
        before_bytes
    );
    assert_eq!(
        crate::semantic_hash_hex(&validated).unwrap(),
        before.semantic_hash
    );
    assert_eq!(
        crate::behavioural_hash_hex(&validated).unwrap(),
        before.behavioural_hash
    );
    assert_eq!(
        crate::ARCHITECTURE_SCHEMA_VERSION,
        before.architecture_schema_version
    );
}

#[test]
fn keyed_validation_tracing_preserves_the_binding_refusal() {
    use super::tracing_capture::{RecordKind, Value, capture, run_isolated};
    if run_isolated(
        "tests::keyed_profile_tests::keyed_validation_tracing_preserves_the_binding_refusal",
    ) {
        return;
    }
    let control = KeyedManifest::probe().unwrap();
    let plain = crate::validate_draft(&control).unwrap().canonical_bytes();
    let (observed, _) = capture(|| crate::validate_draft(&control).unwrap().canonical_bytes());
    assert_eq!(observed, plain);
    let mut subject = control;
    subject.profile = SemanticProfileId::Attestation;
    let plain = crate::validate_draft(&subject).unwrap_err();
    let (observed, records) = capture(|| crate::validate_draft(&subject));
    assert_eq!(observed.unwrap_err(), plain);
    let events = records
        .iter()
        .filter(|record| {
            record.kind == RecordKind::Event && record.target == "architecture::validate"
        })
        .collect::<Vec<_>>();
    assert_eq!(events.len(), 1);
    assert_eq!(
        events[0].fields.get("outcome"),
        Some(&Value::Str("refused".to_owned()))
    );
    assert_eq!(
        events[0].fields.get("refusal_kind"),
        Some(&Value::Str("profile_binding".to_owned()))
    );
}
