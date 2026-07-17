use std::{fs, path::Path};

use crate::{
    LabelErrorCode,
    census::{CensusGroup, RepositoryCensus},
    label::{Label, LabelShape},
    markdown::{InlineCodeContext, scan_markdown},
    model_labels_json,
    owner::{ImportedLabel, LabelOwner},
    repository::{RepositoryLabels, generate_registers},
    rust_source::harvest_model,
};

#[test]
fn single_backtick_label_is_a_bare_span() {
    let scan = scan_markdown(Path::new("fixture.md"), "# Fixture · `sec:fixture`\n");
    assert!(scan.diagnostics.is_empty());
    assert_eq!(scan.code_spans.len(), 1);
    assert_eq!(scan.code_spans[0].context, InlineCodeContext::Bare);
    assert_eq!(scan.code_spans[0].delimiter_len, 1);
}

#[test]
fn parenthesized_import_is_one_span() {
    let scan = scan_markdown(
        Path::new("fixture.md"),
        "See (`[RZ-sec:realization:representation]`).\n",
    );
    assert!(scan.diagnostics.is_empty());
    assert_eq!(scan.code_spans[0].context, InlineCodeContext::Parenthesized);
    assert_eq!(
        scan.code_spans[0].content,
        "[RZ-sec:realization:representation]"
    );
}

#[test]
fn parenthesized_citation_groups_mark_every_span_as_internal() {
    let scan = scan_markdown(
        Path::new("fixture.md"),
        "See (`sec:fixture`, `sec:fixture:detail`).\n",
    );
    assert!(scan.diagnostics.is_empty());
    assert!(
        scan.code_spans
            .iter()
            .all(|span| span.context == InlineCodeContext::Parenthesized)
    );
}

#[test]
fn parenthesized_context_is_preserved_across_lines() {
    let scan = scan_markdown(
        Path::new("fixture.md"),
        "`sec:mint`\n(`sec:fixture`, `sec:fixture:detail`)\n",
    );
    assert_eq!(scan.code_spans[0].context, InlineCodeContext::Bare);
    assert!(
        scan.code_spans[1..]
            .iter()
            .all(|span| span.context == InlineCodeContext::Parenthesized)
    );
}

#[test]
fn one_sided_label_parenthesis_is_asymmetric() {
    let scan = scan_markdown(Path::new("fixture.md"), "See (`sec:fixture`.\n");
    assert_eq!(scan.code_spans[0].context, InlineCodeContext::Asymmetric);
}

#[test]
fn labels_in_mixed_parenthetical_prose_remain_bare() {
    let scan = scan_markdown(
        Path::new("fixture.md"),
        "See (`sec:fixture`, ordinary supporting prose).\n",
    );
    assert_eq!(scan.code_spans[0].context, InlineCodeContext::Bare);
}

#[test]
fn fences_and_double_backticks_hide_examples() {
    let scan = scan_markdown(
        Path::new("fixture.md"),
        "```text\n`sec:not:a-label`\n```\n`` (`sec:not:a-label`) ``\n",
    );
    assert!(scan.diagnostics.is_empty());
    assert_eq!(scan.code_spans.len(), 1);
    assert_eq!(scan.code_spans[0].delimiter_len, 2);
}

#[test]
fn fenced_blocks_require_at_most_three_leading_spaces() {
    let scan = scan_markdown(
        Path::new("fixture.md"),
        "   ```text\n`sec:ignored`\n   ```\n    ```text\n",
    );
    assert_eq!(scan.code_spans.len(), 0);
    assert!(
        scan.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == LabelErrorCode::UnclosedInlineCode)
    );
    assert!(
        !scan
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == LabelErrorCode::UnclosedMarkdownFence)
    );
}

#[test]
fn fence_close_must_match_the_opening_delimiter_length() {
    let scan = scan_markdown(
        Path::new("fixture.md"),
        "````text\n`sec:ignored`\n```\n````\n`sec:visible`\n",
    );
    assert!(scan.diagnostics.is_empty());
    assert_eq!(scan.code_spans.len(), 1);
    assert_eq!(scan.code_spans[0].content, "sec:visible");
}

#[test]
fn unclosed_inline_code_rejects() {
    let scan = scan_markdown(Path::new("fixture.md"), "broken `sec:fixture\n");
    assert!(
        scan.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == LabelErrorCode::UnclosedInlineCode)
    );
}

#[test]
fn imported_owner_and_local_label_parse() {
    let imported = ImportedLabel::parse("ADR012-rule:labels:decision").expect("ADR token parses");
    assert_eq!(imported.owner, LabelOwner::Adr(12));
    assert_eq!(imported.label.as_str(), "rule:labels:decision");
}

#[test]
fn model_harvest_uses_token_columns_and_rejects_malformed_known_labels() {
    let directory = tempfile::tempdir().expect("temporary repository");
    let root = directory.path();
    let source_directory = root.join("packages/model/src");
    fs::create_dir_all(&source_directory).expect("model source directory");
    fs::write(
        source_directory.join("fixture.rs"),
        "// x ´test:fixture:defined´ and (´test:fixture:defined´)\n// ´def:malformed´\n",
    )
    .expect("model source");

    let harvest = harvest_model(&RepositoryCensus::discover(root));
    assert!(harvest.registry.contains(
        &Label::parse("test:fixture:defined", LabelShape::Model).expect("valid model label")
    ));
    assert_eq!(harvest.citations.len(), 1);
    let malformed = harvest
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == LabelErrorCode::InvalidLabel)
        .expect("malformed model label diagnostic");
    assert_eq!((malformed.line, malformed.column), (2, 4));
}

#[test]
fn model_harvest_enforces_owner_relative_forms() {
    let directory = tempfile::tempdir().expect("temporary repository");
    let root = directory.path();
    let source_directory = root.join("packages/model/src");
    fs::create_dir_all(&source_directory).expect("model source directory");
    fs::write(
        source_directory.join("fixture.rs"),
        concat!(
            "// ´def:fixture:defined´\n",
            "// ´def:fixture:defined´\n",
            "// ´sec:representation´\n",
            "// (´[RZ-sec:realization:representation]´)\n",
            "// ´[RZ-sec:realization:representation]´\n",
        ),
    )
    .expect("model source");

    let harvest = harvest_model(&RepositoryCensus::discover(root));

    // A repeated bare label is a duplicate mint, never a silent repeat.
    let duplicate = harvest
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == LabelErrorCode::DuplicateMint)
        .expect("duplicate model mint diagnostic");
    assert_eq!(duplicate.line, 2);

    // A bare realization-shaped token must name its owner explicitly.
    assert!(harvest.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == LabelErrorCode::InvalidImportedCitationForm && diagnostic.line == 3
    }));

    // The parenthesized imported form is harvested; the bare imported
    // form fails.
    assert_eq!(harvest.imports.len(), 1);
    assert!(harvest.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == LabelErrorCode::InvalidImportedCitationForm && diagnostic.line == 5
    }));
}

#[test]
fn plan_local_labels_are_ignored_but_unknown_imports_fail() {
    let directory = tempfile::tempdir().expect("temporary repository");
    let root = directory.path();
    for directory in [
        "papers/attestation/sections",
        "adr",
        "plans",
        "packages/model/src",
    ] {
        fs::create_dir_all(root.join(directory)).expect("fixture directory");
    }
    fs::write(
        root.join("papers/attestation/main.tex"),
        "\\label{def:model:known}\n",
    )
    .expect("attestation source");
    fs::write(
        root.join("docs/attestation/realization.md"),
        "# Realization\n`sec:fixture`\n",
    )
    .expect("realization source");
    fs::write(
        root.join("plans/fixture.md"),
        "`rule:planning:local`\n(`rule:planning:local`)\n(`[A-def:model:missing]`)\n",
    )
    .expect("plan source");

    let labels = RepositoryLabels::harvest_sources(&RepositoryCensus::discover(root));
    assert!(
        labels
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == LabelErrorCode::UnknownImportedLabel)
    );
    assert!(
        !labels
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == LabelErrorCode::DuplicateMint)
    );
}

#[test]
fn adr_imports_are_validated_once_and_internal_citations_resolve() {
    let directory = tempfile::tempdir().expect("temporary repository");
    let root = directory.path();
    for directory in [
        "papers/attestation/sections",
        "adr",
        "plans",
        "packages/model/src",
    ] {
        fs::create_dir_all(root.join(directory)).expect("fixture directory");
    }
    fs::write(
        root.join("papers/attestation/main.tex"),
        "\\label{def:model:known}\n",
    )
    .expect("attestation source");
    fs::write(
        root.join("docs/attestation/realization.md"),
        "# Realization\n`sec:fixture`\n",
    )
    .expect("realization source");
    fs::write(
        root.join("adr/012-fixture.md"),
        "# ADR\n`rule:labels:defined`\n(`rule:labels:defined`)\n(`[A-def:model:known]`)\n",
    )
    .expect("ADR source");

    let labels = RepositoryLabels::harvest_sources(&RepositoryCensus::discover(root));
    assert_eq!(labels.imported_citation_count(), 1);
    assert!(!labels.diagnostics.iter().any(|diagnostic| {
        matches!(
            diagnostic.code,
            LabelErrorCode::InvalidImportedCitationForm | LabelErrorCode::MissingMint
        )
    }));
}

// Whole-repository validity is the job of the Meson-driven
// `check-labels` target (ADR-014): unit tests never discover the live
// checkout, so register generation is exercised on synthetic fixtures
// only.
#[test]
fn register_generation_is_deterministic_in_explicit_outputs() {
    let directory = fixture_root("# Realization\n`sec:fixture`\nBody cite [A-def:model:known].\n");
    let paths = RepositoryCensus::discover(directory.path());
    let output = tempfile::tempdir().expect("temporary output root");
    let specification_output = output.path().join("specification.md");
    let realization_output = output.path().join("realization.md");

    generate_registers(&paths, &specification_output, &realization_output)
        .expect("first register generation");
    let first_specification =
        fs::read(&specification_output).expect("generated specification register");
    let first_realization = fs::read(&realization_output).expect("generated realization register");
    assert!(!first_specification.is_empty());

    generate_registers(&paths, &specification_output, &realization_output)
        .expect("second register generation");
    assert_eq!(
        first_specification,
        fs::read(&specification_output).expect("generated specification register")
    );
    assert_eq!(
        first_realization,
        fs::read(&realization_output).expect("generated realization register")
    );
}

#[test]
fn stale_census_is_a_hard_failure_naming_the_path() {
    let directory = fixture_root("# Realization\n`sec:fixture`\n");
    let root = directory.path();

    // The argument census predates a newly added plan file: the
    // verifier must fail with the path, never silently reclassify.
    let paths = RepositoryCensus::discover(root);
    fs::write(root.join("plans/late-addition.md"), "# Late\n").expect("late plan");

    let diagnostics = paths.verify(CensusGroup::ALL);
    let stale = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == LabelErrorCode::CensusStale)
        .expect("stale census diagnostic");
    assert_eq!(stale.path, "plans/late-addition.md");

    // A census entry deleted on disk is equally stale.
    let paths = RepositoryCensus::discover(root);
    fs::remove_file(root.join("plans/late-addition.md")).expect("remove plan");
    let diagnostics = paths.verify(CensusGroup::ALL);
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == LabelErrorCode::CensusStale
                && diagnostic.path == "plans/late-addition.md")
    );

    // Scoped verification ignores the unrelated planning group.
    let paths = RepositoryCensus::discover(root);
    fs::write(root.join("plans/other.md"), "# Other\n").expect("other plan");
    assert!(paths.verify(CensusGroup::SCOPED).is_empty());
}

#[test]
fn model_label_derivation_ignores_invalid_planning_imports() {
    let directory = tempfile::tempdir().expect("temporary repository");
    let root = directory.path();
    for directory in [
        "papers/attestation/sections",
        "adr",
        "plans",
        "packages/model/src",
    ] {
        fs::create_dir_all(root.join(directory)).expect("fixture directory");
    }
    fs::write(
        root.join("papers/attestation/main.tex"),
        "\\label{def:model:known}\n",
    )
    .expect("attestation source");
    fs::write(
        root.join("docs/attestation/realization.md"),
        "# Realization\n`sec:fixture`\n",
    )
    .expect("realization source");
    fs::write(
        root.join("packages/model/src/fixture.rs"),
        "// ´test:fixture:defined´\n",
    )
    .expect("model source");
    fs::write(root.join("plans/invalid.md"), "(`[A-def:model:missing]`)\n")
        .expect("invalid planning import");

    let paths = RepositoryCensus::discover(root);
    let labels = model_labels_json(&paths).expect("model labels ignore planning imports");
    assert!(labels.contains("test:fixture:defined"));
    assert!(RepositoryLabels::harvest_sources(&paths).has_errors());
}

fn fixture_root(realization: &str) -> tempfile::TempDir {
    let directory = tempfile::tempdir().expect("temporary repository");
    let root = directory.path();
    for child in [
        "papers/attestation/sections",
        "adr",
        "plans",
        "packages/model/src",
    ] {
        fs::create_dir_all(root.join(child)).expect("fixture directory");
    }
    fs::write(
        root.join("papers/attestation/main.tex"),
        "\\label{def:model:known}\n",
    )
    .expect("attestation source");
    fs::write(root.join("docs/attestation/realization.md"), realization)
        .expect("realization source");
    directory
}

#[test]
fn attestation_anchor_set_derives_from_body_citations_only() {
    // A fenced example and the generated upward-citation index must
    // not contribute anchors; an index token without a body citation
    // is a stale index, not a member of the anchor set.
    let directory = fixture_root(concat!(
        "# Realization\n",
        "`sec:fixture`\n",
        "Body cite [A-def:model:known].\n",
        "```text\n",
        "[A-def:model:fenced]\n",
        "```\n",
        "## §17 Upward-citation index · `sec:realization:anchors`\n",
        "| `[A-def:model:known]` | (`sec:fixture`) |\n",
    ));

    let labels = RepositoryLabels::harvest_sources(&RepositoryCensus::discover(directory.path()));
    assert!(
        !labels
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == LabelErrorCode::AttestationIndexStale),
        "{:#?}",
        labels.diagnostics,
    );

    let stale = fixture_root(concat!(
        "# Realization\n",
        "`sec:fixture`\n",
        "## §17 Upward-citation index · `sec:realization:anchors`\n",
        "| `[A-def:model:known]` | (`sec:fixture`) |\n",
    ));
    let labels = RepositoryLabels::harvest_sources(&RepositoryCensus::discover(stale.path()));
    assert!(
        labels
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == LabelErrorCode::AttestationIndexStale)
    );
}

#[test]
fn rust_scanner_processes_comments_only() {
    let directory = tempfile::tempdir().expect("temporary repository");
    let root = directory.path();
    let source_directory = root.join("packages/model/src");
    fs::create_dir_all(&source_directory).expect("model source directory");
    fs::write(
        source_directory.join("fixture.rs"),
        concat!(
            "// ´def:fixture:comment´\n",
            "/* block ´def:fixture:block´ */\n",
            "const A: &str = \"´def:fixture:string´\";\n",
            "const B: &str = r#\"´def:fixture:raw´ and \"quoted\"´def:fixture:raw-two´\"#;\n",
            "const C: char = '´';\n",
            "fn lifetimes<'a>(_value: &'a str) {}\n",
            "//! ```text\n",
            "//! ´def:fixture:fenced´\n",
            "//! ```\n",
            "/// escaped ´def:fixture:doc´ and \\\"´def:fixture:escaped´\n",
        ),
    )
    .expect("model source");

    let harvest = harvest_model(&RepositoryCensus::discover(root));
    let minted: Vec<_> = harvest.registry.labels().map(ToString::to_string).collect();
    assert_eq!(
        minted,
        vec![
            "def:fixture:block".to_owned(),
            "def:fixture:comment".to_owned(),
            "def:fixture:doc".to_owned(),
            "def:fixture:escaped".to_owned(),
        ],
        "{:#?}",
        harvest.diagnostics,
    );
    assert!(harvest.diagnostics.is_empty(), "{:#?}", harvest.diagnostics);
}

#[test]
fn rust_scanner_matches_the_compilers_block_comment_nesting() {
    let directory = tempfile::tempdir().expect("temporary repository");
    let root = directory.path();
    let source_directory = root.join("packages/model/src");
    fs::create_dir_all(&source_directory).expect("model source directory");
    // `/* /* */* */` is a terminated comment under rustc's lexer: each
    // `/*`/`*/` consumes both characters, so the `/` of the inner
    // close cannot pair with the following `*`.
    fs::write(
        source_directory.join("fixture.rs"),
        "/* /* */* */\n/* outer /* inner */ ´def:fixture:nested´ */\n",
    )
    .expect("model source");

    let harvest = harvest_model(&RepositoryCensus::discover(root));
    assert!(harvest.diagnostics.is_empty(), "{:#?}", harvest.diagnostics);
    assert!(harvest.registry.contains(
        &Label::parse("def:fixture:nested", LabelShape::Model).expect("valid model label")
    ));
}

#[test]
fn rust_scanner_ignores_tilde_fenced_examples_and_rejects_asymmetric_parens() {
    let directory = tempfile::tempdir().expect("temporary repository");
    let root = directory.path();
    let source_directory = root.join("packages/model/src");
    fs::create_dir_all(&source_directory).expect("model source directory");
    fs::write(
        source_directory.join("fixture.rs"),
        concat!(
            "//! ~~~\n",
            "//! ´def:fixture:tilde-fenced´\n",
            "//! ~~~\n",
            "// see (´def:fixture:typo´\n",
            "// prose (with ´def:fixture:plain´ inside parens)\n",
        ),
    )
    .expect("model source");

    let harvest = harvest_model(&RepositoryCensus::discover(root));

    // The tilde fence is nonparticipating, exactly like a backtick
    // fence in Markdown.
    assert!(
        !harvest.registry.contains(
            &Label::parse("def:fixture:tilde-fenced", LabelShape::Model).expect("valid label")
        ),
        "{:#?}",
        harvest.diagnostics,
    );

    // A one-sided parenthesis is a citation typo, not a silent mint.
    assert!(harvest.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == LabelErrorCode::AsymmetricCitation && diagnostic.line == 4
    }));
    assert!(
        !harvest
            .registry
            .contains(&Label::parse("def:fixture:typo", LabelShape::Model).expect("valid label"))
    );

    // Ordinary parenthetical prose around a bare span stays a mint.
    assert!(
        harvest
            .registry
            .contains(&Label::parse("def:fixture:plain", LabelShape::Model).expect("valid label"))
    );
}

#[test]
fn scoped_derivations_ignore_cross_owner_imports_and_index_staleness() {
    // A model comment citing another owner, and a stale §17 index,
    // both fail the full repository check but must not block the
    // scoped register/model-label derivations.
    let directory = fixture_root(concat!(
        "# Realization\n",
        "`sec:fixture`\n",
        "Body cite [A-def:model:known].\n",
        "## §17 Upward-citation index · `sec:realization:anchors`\n",
    ));
    let root = directory.path();
    fs::write(
        root.join("packages/model/src/fixture.rs"),
        "// ´test:fixture:defined´\n// (´[ADR010-rule:fixture:missing]´)\n",
    )
    .expect("model source");

    let paths = RepositoryCensus::discover(root);
    assert!(RepositoryLabels::harvest_sources(&paths).has_errors());

    let labels = model_labels_json(&paths).expect("scoped derivation ignores unrelated owners");
    assert!(labels.contains("test:fixture:defined"));
}

#[test]
fn anchor_scan_ignores_double_backtick_examples() {
    let directory = fixture_root(concat!(
        "# Realization\n",
        "`sec:fixture`\n",
        "Body cite [A-def:model:known].\n",
        "A display-only example: ``[A-def:model:example-only]``.\n",
        "## §17 Upward-citation index · `sec:realization:anchors`\n",
        "| `[A-def:model:known]` | (`sec:fixture`) |\n",
    ));

    let labels = RepositoryLabels::harvest_sources(&RepositoryCensus::discover(directory.path()));
    // The example token contributes neither an anchor nor an import,
    // so the index weld and import resolution both stay green.
    assert!(
        !labels.diagnostics.iter().any(|diagnostic| matches!(
            diagnostic.code,
            LabelErrorCode::AttestationIndexStale | LabelErrorCode::UnknownImportedLabel
        )),
        "{:#?}",
        labels.diagnostics,
    );
}

#[test]
fn crate_owners_are_independent_and_duplicates_fail() {
    let directory = fixture_root("# Realization\n`sec:fixture`\n");
    let root = directory.path();
    for child in ["packages/execwrap/src", "packages/artifacts/src"] {
        fs::create_dir_all(root.join(child)).expect("crate source directory");
    }
    fs::write(
        root.join("packages/execwrap/src/lib.rs"),
        "// ´def:fixture:shared´\n// (´def:fixture:shared´)\n",
    )
    .expect("execwrap source");
    fs::write(
        root.join("packages/artifacts/src/lib.rs"),
        concat!(
            "// ´def:fixture:shared´\n",
            "// ´def:fixture:shared´\n",
            "// (´def:fixture:missing´)\n",
        ),
    )
    .expect("artifacts source");

    let labels = RepositoryLabels::harvest_sources(&RepositoryCensus::discover(root));

    // Same label text under two crate owners does not collide; a
    // repeat within one crate does, and citations resolve per crate.
    assert_eq!(labels.registries.crates.len(), 2);
    let codes: Vec<_> = labels
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            matches!(
                diagnostic.code,
                LabelErrorCode::DuplicateMint | LabelErrorCode::MissingMint
            )
        })
        .map(|diagnostic| (diagnostic.code, diagnostic.path.clone(), diagnostic.line))
        .collect();
    assert_eq!(
        codes,
        vec![
            (
                LabelErrorCode::DuplicateMint,
                "packages/artifacts/src/lib.rs".to_owned(),
                2
            ),
            (
                LabelErrorCode::MissingMint,
                "packages/artifacts/src/lib.rs".to_owned(),
                3
            ),
        ],
        "{:#?}",
        labels.diagnostics,
    );
}

#[test]
fn planning_labels_resolve_across_files_and_duplicates_fail() {
    let directory = fixture_root("# Realization\n`sec:fixture`\n");
    let root = directory.path();
    fs::write(
        root.join("plans/first.md"),
        "# First\n`rule:planning:defined`\n(`rule:planning:missing`)\n",
    )
    .expect("first plan");
    fs::write(
        root.join("plans/second.md"),
        "# Second\n(`rule:planning:defined`)\n`rule:planning:defined`\n",
    )
    .expect("second plan");

    let labels = RepositoryLabels::harvest_sources(&RepositoryCensus::discover(root));

    // The cross-file citation resolves; a citation typo and a second
    // mint are hard failures.
    let codes: Vec<_> = labels
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            matches!(
                diagnostic.code,
                LabelErrorCode::MissingMint | LabelErrorCode::DuplicateMint
            )
        })
        .map(|diagnostic| (diagnostic.code, diagnostic.path.clone(), diagnostic.line))
        .collect();
    assert_eq!(
        codes,
        vec![
            (LabelErrorCode::MissingMint, "plans/first.md".to_owned(), 3),
            (
                LabelErrorCode::DuplicateMint,
                "plans/second.md".to_owned(),
                3
            ),
        ],
        "{:#?}",
        labels.diagnostics,
    );
}

#[test]
fn doc_owner_mints_resolve_and_are_importable() {
    let directory = fixture_root("# Realization\n`sec:fixture`\n");
    let root = directory.path();
    fs::write(
        root.join("README.md"),
        "# Repository\n`sec:readme:versions`\n(`sec:readme:versions`)\n",
    )
    .expect("repository README");
    fs::write(
        root.join("plans/citing.md"),
        "# Citing\n(`[DOC-sec:readme:versions]`)\n",
    )
    .expect("plan citing a DOC label");

    let labels = RepositoryLabels::harvest_sources(&RepositoryCensus::discover(root));
    // The fixture architecture weld necessarily fails against a
    // one-label realization document; only citation resolution is
    // under test here.
    assert!(
        !labels.diagnostics.iter().any(|diagnostic| matches!(
            diagnostic.code,
            LabelErrorCode::DuplicateMint
                | LabelErrorCode::MissingMint
                | LabelErrorCode::UnknownOwner
                | LabelErrorCode::UnknownImportedLabel
                | LabelErrorCode::InvalidImportedCitationForm
        )),
        "{:#?}",
        labels.diagnostics,
    );
    assert_eq!(labels.registries.doc.len(), 1);
}

#[test]
fn register_generation_ignores_unrelated_adr_defects() {
    let directory = fixture_root("# Realization\n`sec:fixture`\n");
    let root = directory.path();
    fs::write(
        root.join("adr/012-fixture.md"),
        "# ADR\n(`[ADR012-rule:labels:missing]`)\n",
    )
    .expect("ADR with an unresolved citation");

    let paths = RepositoryCensus::discover(root);
    assert!(RepositoryLabels::harvest_sources(&paths).has_errors());

    let output = tempfile::tempdir().expect("temporary output root");
    generate_registers(
        &paths,
        &output.path().join("specification.md"),
        &output.path().join("realization.md"),
    )
    .expect("upstream registers regenerate despite the ADR defect");
}
