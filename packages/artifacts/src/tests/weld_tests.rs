//! Document-weld conformance tests (M0.4): the realization document's
//! attached manifest and masthead identities are machine-checked
//! against the typed architecture. Changing any appended TOML byte or
//! any masthead identity fails here deterministically.

use architecture::{
    ARCHITECTURE, PublishedArchitecture, behavioural_hash_hex, canonical, semantic_hash_hex,
};

use crate::weld::{extract_appendix_toml, masthead};

/// The committed realization document, embedded at compile time so
/// cargo tracks the fixture and the test never resolves a repository
/// path at runtime (ADR-014 hermetic-test rule).
const REALIZATION_DOCUMENT: &str = include_str!("../../../../docs/attestation/realization.md");

/// The committed generated manifest, embedded the same way.
const COMMITTED_ARCHITECTURE_TOML: &str =
    include_str!("../../../model/generated/architecture.toml");

/// The appendix TOML is byte-for-byte the generated artifact, parses
/// as the supported envelope, validates both hashes, and equals the
/// complete typed expected value.
#[test]
fn appendix_toml_is_the_generated_manifest_verbatim() {
    let attached = extract_appendix_toml(REALIZATION_DOCUMENT).expect("appendix extracts");

    assert_eq!(
        attached, COMMITTED_ARCHITECTURE_TOML,
        "the app:realization:architecture appendix is not byte-identical to \
         packages/model/generated/architecture.toml",
    );

    let published: PublishedArchitecture =
        toml::from_str(&attached).expect("attached manifest parses");
    published.validate_envelope().expect("envelope validates");

    let expected = PublishedArchitecture::from_architecture(&ARCHITECTURE).expect("derives");
    assert_eq!(published, expected);
}

/// Every masthead identity matches the typed architecture: schema,
/// semantic hash, behavioural hash, specification version, anchor-set
/// hash, realization version, and publication status.
#[test]
fn masthead_identities_match_the_typed_architecture() {
    let masthead = masthead(REALIZATION_DOCUMENT).expect("masthead extracts");

    let schema = ARCHITECTURE.document.architecture_schema_version;
    assert!(
        masthead.contains(&format!("architecture schema {schema}")),
        "masthead does not state architecture schema {schema}",
    );

    let semantic = semantic_hash_hex(&ARCHITECTURE).unwrap();
    assert!(
        masthead.contains(&format!("semantic hash* `{semantic}`")),
        "masthead semantic hash does not match the typed architecture",
    );

    let behavioural = behavioural_hash_hex(&ARCHITECTURE).unwrap();
    assert!(
        masthead.contains(&format!("behavioural hash* `{behavioural}`")),
        "masthead behavioural hash does not match the typed architecture",
    );

    let specification = ARCHITECTURE.document.specification.version;
    assert!(
        masthead.contains(&format!("**v{specification}**")),
        "masthead does not pin the specification version v{specification}",
    );

    let anchor_set = ARCHITECTURE
        .document
        .specification
        .anchor_set_hash
        .expect("release manifest pins the anchor set");
    assert!(
        masthead.contains(&canonical::hex(&anchor_set)),
        "masthead anchor-set hash does not match the typed architecture",
    );

    let realization = ARCHITECTURE.document.realization_version;
    assert!(
        masthead.contains(&format!("`realization_version = \"{realization}\"`")),
        "masthead does not carry the tracked realization version binding",
    );

    let status = ARCHITECTURE.document.status.as_str();
    assert!(
        masthead.contains(&format!("`publication_status = \"{status}\"`")),
        "masthead publication status does not match the typed architecture",
    );
}

/// The extractor is structural: a duplicated appendix heading or a
/// second toml fence is a hard failure, not a silent first match.
#[test]
fn appendix_extraction_rejects_ambiguity() {
    let document = REALIZATION_DOCUMENT;

    let duplicated_heading =
        format!("{document}\n## Appendix — duplicate · `app:realization:architecture`\n");
    assert!(extract_appendix_toml(&duplicated_heading).is_err());

    let second_fence = format!("{document}\n```toml\nx = 1\n```\n");
    assert!(extract_appendix_toml(&second_fence).is_err());

    let unclosed = {
        let heading = "## Appendix — Typed architecture manifest · `app:realization:architecture`";
        let index = document.find(heading).expect("heading exists");
        let mut truncated = document[..index].to_owned();
        truncated.push_str(heading);
        truncated.push_str("\n\n```toml\npublication_status = \"final\"\n");
        truncated
    };
    assert!(extract_appendix_toml(&unclosed).is_err());
}
