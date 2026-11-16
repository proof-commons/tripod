use crate::keyed_fixed::{KeyedFixedManifest, KeyedFixedManifestError as Error};
use crate::semantic::SemanticProfileId;

#[test]
fn fixed_form_literal_grammar_and_digest() {
    use sha2::{Digest as _, Sha256};
    let mut anchors = [
        "def:kpcfixed-model:history",
        "def:kpcfixed-model:keyed-membership",
        "def:kpcfixed-model:first-output",
        "op:kpcfixed-operations:source",
        "op:kpcfixed-operations:merge",
        "def:kpcfixed-operations:classification",
        "def:kpcfixed-interface:membership-reading",
        "rem:kpcfixed-interface:record-evidence",
        "sec:kpcfixed:interface",
        "thm:kpcfixed-interface:membership-correctness",
    ];
    anchors.sort_unstable();
    let mut anchor_hash = Sha256::new();
    anchor_hash.update(b"tripod layer-0 anchor set v2\n");
    anchor_hash.update(anchors.join("\n").as_bytes());
    let mut bytes = b"tripod keyed provenance fixed-form manifest schema 3\n".to_vec();
    bytes.extend([0, 0, 0, 3, 0, 3]);
    for text in ["papers/lineage-fixed-form/", "2.1.0"] {
        bytes.extend(u64::try_from(text.len()).unwrap().to_be_bytes());
        bytes.extend(text.as_bytes());
    }
    bytes.extend(anchor_hash.finalize());
    for rows in [
        &[[0_u16, 1], [0, 2], [0, 3]][..],
        &[[3, 1]],
        &[[3, 1], [3, 2]],
        &[[0, 1], [0, 2], [0, 3]],
    ] {
        bytes.extend(u64::try_from(rows.len()).unwrap().to_be_bytes());
        for row in rows {
            for component in row {
                bytes.extend(component.to_be_bytes());
            }
        }
    }
    let manifest = KeyedFixedManifest::contract().unwrap();
    let validated = crate::validate_draft(&manifest).unwrap();
    assert_eq!(validated.canonical_bytes(), bytes);
    let digest: [u8; 32] = Sha256::digest(&bytes).into();
    assert_eq!(validated.binding().digest, digest);
    // The literal pin is independently computed from this grammar.
    assert_eq!(digest, FIXED_DIGEST);
    assert_eq!(bytes.len(), 206);
    let sections = [
        include_str!("../../../../papers/lineage-fixed-form/sections/01_interface.tex"),
        include_str!("../../../../papers/lineage-fixed-form/sections/02_model.tex"),
        include_str!("../../../../papers/lineage-fixed-form/sections/03_operations.tex"),
    ]
    .join("\n");
    for anchor in anchors {
        assert!(sections.contains(&format!("\\label{{{anchor}}}")));
    }
}

#[test]
fn fixed_manifest_refuses_every_changed_field_and_cross_schema() {
    let control = KeyedFixedManifest::contract().unwrap();
    let bytes = crate::validate_draft(&control).unwrap().canonical_bytes();
    for coordinate in 0..11 {
        let mut subject = control.clone();
        let expected = match coordinate {
            0 => {
                subject.profile = SemanticProfileId::KeyedProvenance;
                Error::ProfileBinding
            }
            1 => {
                subject.schema_version = 2;
                Error::SchemaVersion
            }
            2 => {
                subject.document.paper = "papers/lineage/".into();
                Error::PaperBinding
            }
            3 => {
                subject.document.version = "2.0.0".into();
                Error::PaperBinding
            }
            4 => {
                subject.document.anchors.pop();
                Error::PaperBinding
            }
            5 => {
                subject.document.anchor_set_hash[0] ^= 1;
                Error::PaperBinding
            }
            6 => {
                subject.parameters.pop();
                Error::ParameterCensus
            }
            7 => {
                subject.parameters.push(subject.parameters[0]);
                Error::ParameterCensus
            }
            8 => {
                subject.objects.clear();
                Error::ObjectCensus
            }
            9 => {
                subject.operations.pop();
                Error::OperationCensus
            }
            _ => {
                subject.clauses.pop();
                Error::ClauseCensus
            }
        };
        assert_eq!(crate::validate_draft(&subject).unwrap_err(), expected);
    }
    let mut permuted = control;
    permuted.parameters.reverse();
    permuted.operations.reverse();
    permuted.clauses.reverse();
    assert_eq!(
        crate::validate_draft(&permuted).unwrap().canonical_bytes(),
        bytes
    );
    let mut legacy = crate::keyed::KeyedManifest::contract().unwrap();
    crate::validate_draft(&legacy).unwrap();
    legacy.schema_version = 3;
    assert_eq!(
        crate::validate_draft(&legacy).unwrap_err(),
        crate::keyed::KeyedManifestError::SchemaVersion
    );
    legacy.schema_version = 2;
    legacy.profile = SemanticProfileId::KeyedFixedForm;
    assert_eq!(
        crate::validate_draft(&legacy).unwrap_err(),
        crate::keyed::KeyedManifestError::ProfileBinding
    );
}

#[test]
fn namespaces_are_injective_and_legacy_encodings_are_unchanged() {
    use crate::keyed_fixed::{KeyedFixedObjectId, KeyedFixedOperationId};
    use crate::semantic::{KeyedObjectId, KeyedOperationId, SemanticObjectId, SemanticOperationId};
    let fixed = [
        KeyedFixedOperationId::Source.encoding(),
        KeyedFixedOperationId::Merge.encoding(),
    ];
    assert_eq!(fixed, [[3, 1], [3, 2]]);
    assert_eq!(KeyedFixedObjectId::Handle.encoding(), [3, 1]);
    for (id, expected) in [
        (KeyedOperationId::Source, [2, 1]),
        (KeyedOperationId::Merge, [2, 2]),
    ] {
        assert_eq!(SemanticOperationId::Keyed(id).encoding(), expected);
        assert!(!fixed.contains(&expected));
    }
    assert_eq!(
        SemanticObjectId::Keyed(KeyedObjectId::Handle).encoding(),
        [2, 1]
    );
    for id in crate::OperationId::ALL {
        assert_eq!(
            SemanticOperationId::Attestation(*id).encoding(),
            [1, id.code()]
        );
        assert!(!fixed.contains(&[1, id.code()]));
    }
    for id in crate::ObjectId::ALL {
        assert_eq!(
            SemanticObjectId::Attestation(*id).encoding(),
            [1, id.code()]
        );
        assert_ne!(KeyedFixedObjectId::Handle.encoding(), [1, id.code()]);
    }
}

const FIXED_DIGEST: [u8; 32] = [
    168, 83, 97, 146, 4, 54, 228, 90, 227, 38, 214, 131, 218, 63, 196, 87, 34, 102, 156, 162, 181,
    242, 65, 139, 201, 120, 241, 131, 243, 66, 47, 193,
];

#[test]
fn keyed_fixed_validation_tracing_preserves_the_binding_refusal() {
    use super::tracing_capture::{RecordKind, Value, capture, run_isolated};
    if run_isolated(
        "tests::keyed_fixed_profile_tests::keyed_fixed_validation_tracing_preserves_the_binding_refusal",
    ) {
        return;
    }
    let control = KeyedFixedManifest::contract().unwrap();
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
