//! Canonical export, semantic-hash stability, and generated-artifact
//! verification tests.

use crate::*;

#[test]
fn semantic_hash_is_stable_within_one_build() {
    assert_eq!(
        canonical_json_bytes(&super::validated(&ARCHITECTURE)).unwrap(),
        canonical_json_bytes(&super::validated(&ARCHITECTURE)).unwrap(),
    );

    assert_eq!(
        semantic_hash(&super::validated(&ARCHITECTURE)).unwrap(),
        semantic_hash(&super::validated(&ARCHITECTURE)).unwrap(),
    );
}

#[test]
fn envelope_metadata_does_not_change_body_hash() {
    // Publication status, the schema version, and the realization
    // version are all envelope metadata: flipping any of them must
    // leave both body hashes verifying. This is what makes changes to
    // envelope metadata hash-invisible. Hash-invisibility is
    // verify_hashes' claim; supported-envelope validation is separate
    // and stricter.
    let mut published =
        PublishedArchitecture::from_architecture(&super::validated(&ARCHITECTURE)).unwrap();

    published.publication_status = PublicationStatus::Final.as_str().to_owned();
    published.architecture_schema_version += 1;
    published.realization_version = "13z".to_owned();

    assert!(published.verify_body_hash().unwrap());
    assert!(published.verify_behavioural_hash().unwrap());
    published.verify_hashes().unwrap();
}

#[test]
fn envelope_validation_rejects_unsupported_metadata() {
    // Hash exclusion does not mean acceptance: an artifact claiming an
    // unknown schema, an unrecognized status, or a blank realization
    // version must fail full envelope validation even though its
    // digests verify.
    let published =
        PublishedArchitecture::from_architecture(&super::validated(&ARCHITECTURE)).unwrap();
    published.validate_envelope().unwrap();

    let mut wrong_schema = published.clone();
    wrong_schema.architecture_schema_version += 1;
    wrong_schema.verify_hashes().unwrap();
    assert!(matches!(
        wrong_schema.validate_envelope(),
        Err(EnvelopeError::UnsupportedSchemaVersion(_)),
    ));

    let mut wrong_status = published.clone();
    wrong_status.publication_status = "compromised".to_owned();
    wrong_status.verify_hashes().unwrap();
    assert!(matches!(
        wrong_status.validate_envelope(),
        Err(EnvelopeError::UnrecognizedPublicationStatus(_)),
    ));

    let mut blank_version = published.clone();
    blank_version.realization_version = "  ".to_owned();
    blank_version.verify_hashes().unwrap();
    assert!(matches!(
        blank_version.validate_envelope(),
        Err(EnvelopeError::MissingRealizationVersion),
    ));

    // The field accepts only the tracked-version form.
    let mut malformed_version = published;
    malformed_version.realization_version = "13z".to_owned();
    malformed_version.verify_hashes().unwrap();
    assert!(matches!(
        malformed_version.validate_envelope(),
        Err(EnvelopeError::MalformedRealizationVersion(_)),
    ));
}

#[test]
fn release_envelope_requires_final_status() {
    let mut published =
        PublishedArchitecture::from_architecture(&super::validated(&ARCHITECTURE)).unwrap();

    published.publication_status = PublicationStatus::Final.as_str().to_owned();
    published.validate_release_envelope().unwrap();

    published.publication_status = PublicationStatus::Draft.as_str().to_owned();
    published.validate_envelope().unwrap();
    assert!(matches!(
        published.validate_release_envelope(),
        Err(EnvelopeError::NotFinal),
    ));
}

#[test]
fn self_consistent_forgery_passes_envelope_but_fails_expected_identity() {
    // The envelope hashes are unkeyed: a mutated body with recomputed
    // digests still passes release-envelope validation, which is a
    // self-consistency check only. Only the complete-value comparison
    // against an independently derived expected publication turns
    // ingestion into an authenticity claim.
    let expected =
        PublishedArchitecture::from_architecture(&super::validated(&ARCHITECTURE)).unwrap();

    let mut forged = expected.clone();
    forged.publication_status = PublicationStatus::Final.as_str().to_owned();
    forged.architecture.operations[0].inputs[0].minimum += 1;
    forged.semantic_hash = crate::canonical::export_body_hash_hex(&forged.architecture).unwrap();
    forged.behavioural_hash =
        crate::canonical::export_behavioural_hash_hex(&forged.architecture).unwrap();

    forged.validate_release_envelope().unwrap();

    assert!(matches!(
        forged.validate_against_expected(&expected),
        Err(EnvelopeError::UnexpectedPublication),
    ));

    // The untampered publication passes trusted-identity validation.
    expected.validate_against_expected(&expected).unwrap();
}

#[test]
fn calibrated_default_mutation_moves_only_the_full_hash() {
    // A bound flagged `requires_deployment_calibration = true` carries
    // a draft default outside the abstract denotation
    // (´[RZ-def:versioning:denotation-law]´): changing it must move the
    // semantic (full) hash but leave the behavioural hash fixed.
    let mut modified =
        PublishedArchitecture::from_architecture(&super::validated(&ARCHITECTURE)).unwrap();

    let bound = &mut modified.architecture.bounds[0];
    assert!(
        bound.requires_deployment_calibration,
        "test premise: bound 0 is deployment-calibrated",
    );
    bound.default_value = Some(bound.default_value.unwrap_or(0) + 1);

    assert!(!modified.verify_body_hash().unwrap());
    assert!(modified.verify_behavioural_hash().unwrap());
}

#[test]
fn fixed_bound_value_mutation_moves_both_hashes() {
    // An uncalibrated bound's value is part of the denotation. The
    // current architecture declares no fixed bounds, so this is
    // exercised on the export DTO: flipping the calibration flag off
    // pulls the value into the behavioural projection.
    let mut modified =
        PublishedArchitecture::from_architecture(&super::validated(&ARCHITECTURE)).unwrap();

    let bound = &mut modified.architecture.bounds[0];
    bound.requires_deployment_calibration = false;

    assert!(!modified.verify_body_hash().unwrap());
    assert!(!modified.verify_behavioural_hash().unwrap());
}

#[test]
fn operation_cardinality_mutation_moves_both_hashes() {
    let mut modified =
        PublishedArchitecture::from_architecture(&super::validated(&ARCHITECTURE)).unwrap();

    modified.architecture.operations[0].inputs[0].minimum += 1;

    assert!(!modified.verify_body_hash().unwrap());
    assert!(!modified.verify_behavioural_hash().unwrap());
}

#[test]
fn fixed_amount_limit_mutation_moves_both_hashes() {
    // `amount_limits` are declared protocol values, not calibration
    // placeholders: they stay inside the behavioural projection.
    let mut modified =
        PublishedArchitecture::from_architecture(&super::validated(&ARCHITECTURE)).unwrap();

    modified.architecture.amount_limits[0].value += 1;

    assert!(!modified.verify_body_hash().unwrap());
    assert!(!modified.verify_behavioural_hash().unwrap());
}

#[test]
fn witness_document_label_rename_moves_only_the_full_hash() {
    // A witness's `semantic_tag` is the realization document's
    // citation label — presentation under the Denotation Law, guarded
    // by the sixteen-string weld. Renaming it must move the full
    // semantic hash (the export body changed) but not the behavioural
    // hash: a label rename leaves the denotation and its behavioural
    // hash unchanged.
    let mut modified =
        PublishedArchitecture::from_architecture(&super::validated(&ARCHITECTURE)).unwrap();

    modified.architecture.witnesses[0].semantic_tag = "lem:invariant:renamed".to_owned();

    assert!(!modified.verify_body_hash().unwrap());
    assert!(modified.verify_behavioural_hash().unwrap());
}

#[test]
fn clause_document_label_rename_moves_only_the_full_hash() {
    // The clause registry's `id` is likewise the document's frozen
    // citation label.
    let mut modified =
        PublishedArchitecture::from_architecture(&super::validated(&ARCHITECTURE)).unwrap();

    modified.architecture.clauses[0].id = "inv:invariant:renamed".to_owned();

    assert!(!modified.verify_body_hash().unwrap());
    assert!(modified.verify_behavioural_hash().unwrap());
}

#[test]
fn witness_semantic_identity_mutation_moves_both_hashes() {
    // The witness id and code are semantic identity, not presentation:
    // both stay behavioural-hash inputs.
    let mut renamed =
        PublishedArchitecture::from_architecture(&super::validated(&ARCHITECTURE)).unwrap();
    renamed.architecture.witnesses[0].id = "renamed-witness".to_owned();

    assert!(!renamed.verify_body_hash().unwrap());
    assert!(!renamed.verify_behavioural_hash().unwrap());

    let mut recoded =
        PublishedArchitecture::from_architecture(&super::validated(&ARCHITECTURE)).unwrap();
    recoded.architecture.witnesses[0].code += 100;

    assert!(!recoded.verify_body_hash().unwrap());
    assert!(!recoded.verify_behavioural_hash().unwrap());
}

#[test]
fn clause_code_mutation_moves_both_hashes() {
    let mut modified =
        PublishedArchitecture::from_architecture(&super::validated(&ARCHITECTURE)).unwrap();

    modified.architecture.clauses[0].code += 100;

    assert!(!modified.verify_body_hash().unwrap());
    assert!(!modified.verify_behavioural_hash().unwrap());
}

#[test]
fn dependency_mutation_moves_only_the_full_hash() {
    // Dependencies are normative-descriptive: outside the behavioural
    // hash, inside the full hash. This asymmetry is the versioning
    // law's letter/major cut, mechanized.
    let mut modified =
        PublishedArchitecture::from_architecture(&super::validated(&ARCHITECTURE)).unwrap();

    modified.architecture.dependencies[0].rationale = "reworded".to_owned();

    assert!(!modified.verify_body_hash().unwrap());
    assert!(modified.verify_behavioural_hash().unwrap());
}

#[test]
fn json_and_toml_export_the_same_manifest() {
    let published =
        PublishedArchitecture::from_architecture(&super::validated(&ARCHITECTURE)).unwrap();

    let json = serde_json::to_string_pretty(&published).unwrap();
    let toml = toml::to_string_pretty(&published).unwrap();

    let from_json: PublishedArchitecture = serde_json::from_str(&json).unwrap();
    let from_toml: PublishedArchitecture = toml::from_str(&toml).unwrap();

    assert_eq!(from_json, from_toml);
    assert_eq!(from_json, published);
}

#[test]
fn both_exports_verify_the_semantic_hash() {
    let published =
        PublishedArchitecture::from_architecture(&super::validated(&ARCHITECTURE)).unwrap();

    assert!(published.verify_body_hash().unwrap());
    published.validate_envelope().unwrap();

    let json = serde_json::to_string_pretty(&published).unwrap();
    let from_json: PublishedArchitecture = serde_json::from_str(&json).unwrap();
    from_json.validate_envelope().unwrap();

    let toml = toml::to_string_pretty(&published).unwrap();
    let from_toml: PublishedArchitecture = toml::from_str(&toml).unwrap();
    from_toml.validate_envelope().unwrap();
}

#[test]
fn pretty_printing_does_not_change_the_semantic_hash() {
    let published =
        PublishedArchitecture::from_architecture(&super::validated(&ARCHITECTURE)).unwrap();

    let compact = serde_json::to_string(&published).unwrap();
    let pretty = serde_json::to_string_pretty(&published).unwrap();

    let compact_parsed: PublishedArchitecture = serde_json::from_str(&compact).unwrap();
    let pretty_parsed: PublishedArchitecture = serde_json::from_str(&pretty).unwrap();

    assert_eq!(compact_parsed.semantic_hash, pretty_parsed.semantic_hash);

    assert!(compact_parsed.verify_body_hash().unwrap());
    assert!(pretty_parsed.verify_body_hash().unwrap());
}

#[test]
fn semantic_change_changes_hash() {
    let mut modified =
        PublishedArchitecture::from_architecture(&super::validated(&ARCHITECTURE)).unwrap();

    modified.architecture.target_network = "different-network".to_owned();

    assert!(!modified.verify_body_hash().unwrap());

    assert!(matches!(
        modified.validate_envelope(),
        Err(EnvelopeError::HashMismatch),
    ));
}

#[test]
fn unknown_fields_are_rejected() {
    let published =
        PublishedArchitecture::from_architecture(&super::validated(&ARCHITECTURE)).unwrap();

    let mut value = serde_json::to_value(published).unwrap();

    value
        .as_object_mut()
        .unwrap()
        .insert("unexpected".to_owned(), serde_json::Value::Bool(true));

    let bytes = serde_json::to_vec(&value).unwrap();

    assert!(serde_json::from_slice::<PublishedArchitecture>(&bytes).is_err());
}

// Every externally deserialized DTO shape must reject unknown fields,
// not only the top-level publication object: a nested unknown field
// that serde silently discards would survive body-hash verification
// (the hash covers the parsed DTO) and typed-equality validation (the
// field no longer exists in memory), defeating the generated-artifact
// policy for tagged enum variants.
fn nested_object_paths(value: &serde_json::Value) -> Vec<Vec<String>> {
    let architecture = &value["architecture"];
    let mut paths = Vec::new();
    let mut push = |base: &[&str], list: &str, item: usize, tail: &[&str]| {
        let mut path: Vec<String> = base.iter().map(ToString::to_string).collect();
        path.push(list.to_owned());
        path.push(item.to_string());
        path.extend(tail.iter().map(ToString::to_string));
        paths.push(path);
    };
    for (index, object) in architecture["objects"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
    {
        let base = [
            "architecture".to_owned(),
            "objects".to_owned(),
            index.to_string(),
        ];
        let base: Vec<&str> = base.iter().map(String::as_str).collect();
        for list in ["allocators", "deallocators", "authorization_paths"] {
            for item in 0..object[list].as_array().unwrap().len() {
                push(&base, list, item, &[]);
            }
        }
    }
    for (index, operation) in architecture["operations"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
    {
        let base = [
            "architecture".to_owned(),
            "operations".to_owned(),
            index.to_string(),
        ];
        let base: Vec<&str> = base.iter().map(String::as_str).collect();
        for list in ["inputs", "outputs", "data_outputs"] {
            for item in 0..operation[list].as_array().unwrap().len() {
                push(&base, list, item, &["maximum"]);
            }
        }
    }
    for (index, quantity) in architecture["quantities"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
    {
        let base = [
            "architecture".to_owned(),
            "quantities".to_owned(),
            index.to_string(),
        ];
        let base: Vec<&str> = base.iter().map(String::as_str).collect();
        for item in 0..quantity["readers"].as_array().unwrap().len() {
            push(&base, "readers", item, &[]);
        }
    }
    assert!(
        !paths.is_empty(),
        "the fixture architecture exercises no nested variants"
    );
    paths
}

fn with_unknown_field_at(value: &serde_json::Value, path: &[String]) -> serde_json::Value {
    let mut mutated = value.clone();
    let mut cursor = &mut mutated;
    for step in path {
        cursor = match cursor {
            serde_json::Value::Array(items) => &mut items[step.parse::<usize>().unwrap()],
            other => other.get_mut(step).unwrap(),
        };
    }
    cursor
        .as_object_mut()
        .unwrap()
        .insert("unexpected".to_owned(), serde_json::Value::Bool(true));
    mutated
}

#[test]
fn nested_unknown_fields_are_rejected_in_json() {
    let published =
        PublishedArchitecture::from_architecture(&super::validated(&ARCHITECTURE)).unwrap();
    let value = serde_json::to_value(published).unwrap();

    for path in nested_object_paths(&value) {
        let mutated = with_unknown_field_at(&value, &path);
        let bytes = serde_json::to_vec(&mutated).unwrap();
        assert!(
            serde_json::from_slice::<PublishedArchitecture>(&bytes).is_err(),
            "nested unknown field silently accepted at {}",
            path.join("."),
        );
    }
}

#[test]
fn nested_unknown_fields_are_rejected_in_toml() {
    let published =
        PublishedArchitecture::from_architecture(&super::validated(&ARCHITECTURE)).unwrap();
    let value = serde_json::to_value(published).unwrap();

    for path in nested_object_paths(&value) {
        let mutated = with_unknown_field_at(&value, &path);
        let rendered = toml::to_string(&mutated).unwrap();
        assert!(
            toml::from_str::<PublishedArchitecture>(&rendered).is_err(),
            "nested unknown field silently accepted at {}",
            path.join("."),
        );
    }
}

#[test]
fn incorrect_algorithm_identifier_invalidates_envelope() {
    // The body hash intentionally excludes envelope metadata, so the
    // body still verifies; publication-envelope validation must
    // nevertheless reject the unknown algorithm identifier.
    let mut modified =
        PublishedArchitecture::from_architecture(&super::validated(&ARCHITECTURE)).unwrap();
    modified.semantic_hash_algorithm = "renamed".to_owned();

    assert!(modified.verify_body_hash().unwrap());

    assert!(matches!(
        modified.validate_envelope(),
        Err(EnvelopeError::UnsupportedHashAlgorithm),
    ));
}

// The committed generated artifacts must satisfy the full conformance
// ladder, not merely embed the right hash: (1) parse (with unknown
// fields rejected by the DTO), (2) validate the publication envelope,
// (3) equal the complete typed expected value, (4) equal the canonical
// presentation bytes. A stale body carrying a fresh hash field fails
// (3) and (4) even though it would pass a hash-only comparison.

#[test]
fn generated_json_equals_typed_architecture_completely() {
    let generated = include_str!("../../../model/generated/architecture.json");

    let published: PublishedArchitecture = serde_json::from_str(generated).unwrap();
    published.validate_envelope().unwrap();

    let expected =
        PublishedArchitecture::from_architecture(&super::validated(&ARCHITECTURE)).unwrap();
    assert_eq!(published, expected);
    assert_eq!(
        published.semantic_hash,
        semantic_hash_hex(&super::validated(&ARCHITECTURE)).unwrap(),
    );
    assert_eq!(
        published.behavioural_hash,
        behavioural_hash_hex(&super::validated(&ARCHITECTURE)).unwrap(),
    );

    assert_eq!(
        generated,
        expected.to_artifact_json().unwrap(),
        "committed architecture.json is not the canonical presentation; \
         run `meson compile -C build generate-artifacts`",
    );
}

#[test]
fn generated_toml_equals_typed_architecture_completely() {
    let generated = include_str!("../../../model/generated/architecture.toml");

    let published: PublishedArchitecture = toml::from_str(generated).unwrap();
    published.validate_envelope().unwrap();

    let expected =
        PublishedArchitecture::from_architecture(&super::validated(&ARCHITECTURE)).unwrap();
    assert_eq!(published, expected);
    assert_eq!(
        published.semantic_hash,
        semantic_hash_hex(&super::validated(&ARCHITECTURE)).unwrap(),
    );
    assert_eq!(
        published.behavioural_hash,
        behavioural_hash_hex(&super::validated(&ARCHITECTURE)).unwrap(),
    );

    assert_eq!(
        generated,
        expected.to_artifact_toml().unwrap(),
        "committed architecture.toml is not the canonical presentation; \
         run `meson compile -C build generate-artifacts`",
    );
}

#[test]
fn generated_artifacts_use_companion_authorization_vocabulary() {
    let json = include_str!("../../../model/generated/architecture.json");
    let toml = include_str!("../../../model/generated/architecture.toml");

    for artifact in [json, toml] {
        assert!(artifact.contains("covenant-companion"));
        assert!(artifact.contains("input_authorization_evidence"));
        assert!(artifact.contains("operation_authorization_evidence"));
        assert!(!artifact.contains("branch-authorization"));
    }
}

#[test]
fn anchor_set_hash_is_order_and_duplication_insensitive() {
    let sorted = ValidatedAnchorSet::new([
        "def:model:classes",
        "open:model:leverage-timing",
        "rem:model:calibration",
    ])
    .expect("three anchor names");
    let shuffled_with_repeats = ValidatedAnchorSet::new([
        "rem:model:calibration",
        "def:model:classes",
        "open:model:leverage-timing",
        "def:model:classes",
    ])
    .expect("the same three anchor names");

    assert_eq!(
        anchor_set_hash(&sorted),
        anchor_set_hash(&shuffled_with_repeats),
    );
}

#[test]
fn anchor_set_hash_matches_the_published_recipe() {
    // sha256 of the domain prefix "tripod layer-0 anchor set
    // v2\n" followed by
    // "def:model:classes\nopen:model:leverage-timing\nrem:model:calibration",
    // computed independently of this crate:
    //
    //   printf 'tripod layer-0 anchor set v2\n\
    //   def:model:classes\nopen:model:leverage-timing\n\
    //   rem:model:calibration' | sha256sum
    //
    // Re-pinned by the DI-004 recipe migration; the retired `…-v1`
    // recipe hashed the joined names alone.
    //
    // The vector's middle name gained its area segment when `G12-R02`
    // made the anchor-name grammar a type: the old two-part `open:…`
    // string is not an anchor name and never was one, so a recipe test
    // could no longer stand on it. The recipe itself did not move —
    // the same command over the *old* vector still returns the digest
    // this test carried before, 331071bbade9fce3fb4cc1386e5e7ff5701b\
    // 6185e0e05e9b11143f9b26b00dd1.
    let expected = "577cee1e2a1901f4ae7d956c6e7845fdb8b2b3fed78a943da87b032410efa812";

    let anchors = ValidatedAnchorSet::new([
        "open:model:leverage-timing",
        "rem:model:calibration",
        "def:model:classes",
    ])
    .expect("three anchor names");

    assert_eq!(canonical::hex(&anchor_set_hash(&anchors)), expected);
}

/// Retired identifiers stay retired: reusing one for a new recipe is
/// the silent redefinition `(´[ADR021-rule:identity:recipe-permanence]´)` forbids.
#[test]
fn retired_identity_algorithms_stay_retired() {
    assert_eq!(SEMANTIC_HASH_ALGORITHM, "sha256-canonical-json-v3");
    assert!(RETIRED_SEMANTIC_HASH_ALGORITHMS.contains(&"sha256-canonical-json-v2"));
    assert!(!RETIRED_SEMANTIC_HASH_ALGORITHMS.contains(&SEMANTIC_HASH_ALGORITHM));

    assert_eq!(ANCHOR_SET_HASH_ALGORITHM, "sha256-anchor-set-v2");
    assert!(RETIRED_ANCHOR_SET_HASH_ALGORITHMS.contains(&"sha256-anchor-set-v1"));
    assert!(!RETIRED_ANCHOR_SET_HASH_ALGORITHMS.contains(&ANCHOR_SET_HASH_ALGORITHM));
}

/// The semantic hash is domain-separated: it is not the bare digest of
/// the canonical body, so the retired `…-v2` measurement cannot be
/// reintroduced by an unprefixed rehash of the same projection.
#[test]
fn the_semantic_hash_is_domain_separated() {
    use sha2::{Digest, Sha256};

    let validated = super::validated(&ARCHITECTURE);
    let body = canonical_json_bytes(&validated).unwrap();

    assert_ne!(
        semantic_hash(&validated).unwrap().as_slice(),
        Sha256::digest(&body).as_slice(),
    );
}

#[test]
fn authorization_evidence_rows_are_declaration_order_independent() {
    // F3-008: the exported evidence tables sort by stable
    // discriminant, so reversing the source iteration must not move
    // one row, and the manifest arrays must equal the canonical
    // helper output exactly.
    let forward =
        export::input_authorization_evidence_rows(InputAuthorization::ALL.iter().copied());
    let reversed =
        export::input_authorization_evidence_rows(InputAuthorization::ALL.iter().rev().copied());
    assert_eq!(forward, reversed);

    let forward_classes =
        export::operation_authorization_evidence_rows(PermissionClass::ALL.iter().copied());
    let reversed_classes =
        export::operation_authorization_evidence_rows(PermissionClass::ALL.iter().rev().copied());
    assert_eq!(forward_classes, reversed_classes);

    let exported = ArchitectureExport::from_architecture(&ARCHITECTURE);
    assert_eq!(exported.input_authorization_evidence, forward);
    assert_eq!(exported.operation_authorization_evidence, forward_classes);
}

// --- R2-N03: identity is reachable only through validation ---
//
// The adopted discipline puts validation before identity
// `(´[ADR021-rule:identity:admission-order]´)`, and rehashing is
// explicitly not revalidation. `ValidatedDraftArchitecture` is the only public
// input to `semantic_hash`, `behavioural_hash`, `canonical_json_bytes`,
// and `PublishedArchitecture::from_architecture`, and `validate_draft`
// is its only constructor, so an invalid architecture has no public
// identity path at all — the cases below record that the gate is the
// validator, not a convention followed by each caller.

#[test]
fn an_invalid_architecture_cannot_reach_a_public_semantic_identity() {
    let mut architecture = ARCHITECTURE;
    architecture.document.realization_version = "  ";

    assert!(
        validate_draft(&architecture).is_err(),
        "the fixture must be invalid for this case to mean anything"
    );

    // The recipe itself is unchanged: the crate-private projection
    // still hashes the same bytes. What changed is who may call it.
    assert!(crate::canonical::unchecked_semantic_hash(&architecture).is_ok());
}

#[test]
fn an_invalid_architecture_cannot_produce_a_publication() {
    let mut architecture = ARCHITECTURE;
    architecture.assets = &[];

    assert!(validate_draft(&architecture).is_err());
}

#[test]
fn validation_does_not_move_any_published_identity() {
    // The wrapper changes the callable surface, not the hashed body:
    // every identity taken through the validated path must equal the
    // unchecked recipe applied to the same architecture.
    let validated = super::validated(&ARCHITECTURE);

    assert_eq!(
        semantic_hash(&validated).unwrap(),
        crate::canonical::unchecked_semantic_hash(&ARCHITECTURE).unwrap(),
    );
    assert_eq!(
        behavioural_hash(&validated).unwrap(),
        crate::canonical::unchecked_behavioural_hash(&ARCHITECTURE).unwrap(),
    );
    assert_eq!(
        canonical_json_bytes(&validated).unwrap(),
        crate::canonical::unchecked_canonical_json_bytes(&ARCHITECTURE).unwrap(),
    );
}

#[test]
fn the_release_state_carries_the_same_identity_as_its_draft() {
    // Release validation subsumes draft validation, and the release
    // obligations (attestation anchor-set pin, final status) are envelope metadata
    // outside the hashed body, so both states share one identity.
    let release = validate_architecture_release(&ARCHITECTURE).unwrap();

    assert_eq!(
        semantic_hash(&release.draft()).unwrap(),
        semantic_hash(&super::validated(&ARCHITECTURE)).unwrap(),
    );
}
