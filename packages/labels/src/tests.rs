use std::{fs, path::Path};

use crate::{
    LabelErrorCode,
    check::check_repository,
    label::{Label, LabelShape},
    latex::harvest_attestation,
    markdown::{InlineCodeContext, scan_markdown},
    owner::{ImportedLabel, LabelOwner},
    repository::{RepositoryLabels, RepositoryPaths, generate_registers},
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
fn open_subproblem_optional_label_is_harvested() {
    let directory = tempfile::tempdir().expect("temporary repository");
    let root = directory.path();
    fs::create_dir_all(root.join("papers/attestation/sections"))
        .expect("attestation sections directory");
    fs::write(
        root.join("papers/attestation/main.tex"),
        "\\OpenSubProblem[inner-case]{An inner case}\n% \\OpenSubProblem[ignored]{Comment}\n",
    )
    .expect("attestation source");

    let (registry, diagnostics) = harvest_attestation(&RepositoryPaths::from_root(root));
    assert!(diagnostics.is_empty());
    assert!(
        registry.contains(
            &Label::parse("open:inner-case", LabelShape::Attestation).expect("valid label")
        )
    );
}

#[test]
fn model_harvest_uses_token_columns_and_rejects_malformed_known_labels() {
    let directory = tempfile::tempdir().expect("temporary repository");
    let root = directory.path();
    let source_directory = root.join("packages/model/src");
    fs::create_dir_all(&source_directory).expect("model source directory");
    fs::write(
        source_directory.join("fixture.rs"),
        "// x ´test:fixture:defined´ and ´sec:fixture´\n// ´def:malformed´\n",
    )
    .expect("model source");

    let harvest = harvest_model(&RepositoryPaths::from_root(root));
    assert!(harvest.registry.contains(
        &Label::parse("test:fixture:defined", LabelShape::Model).expect("valid model label")
    ));
    assert_eq!(harvest.realization_citations.len(), 1);
    assert_eq!(harvest.realization_citations[0].1.column, 33);
    let malformed = harvest
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == LabelErrorCode::InvalidLabel)
        .expect("malformed model label diagnostic");
    assert_eq!((malformed.line, malformed.column), (2, 4));
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

    let labels = RepositoryLabels::harvest_sources(&RepositoryPaths::from_root(root));
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

    let labels = RepositoryLabels::harvest_sources(&RepositoryPaths::from_root(root));
    assert_eq!(labels.imported_citation_count(), 1);
    assert!(!labels.diagnostics.iter().any(|diagnostic| {
        matches!(
            diagnostic.code,
            LabelErrorCode::InvalidImportedCitationForm | LabelErrorCode::MissingMint
        )
    }));
}

#[test]
fn workspace_sources_pass_the_document_and_architecture_weld() {
    let paths = RepositoryPaths::workspace_default();
    let before = [
        fs::read(&paths.specification_register).expect("Layer-0 register"),
        fs::read(&paths.realization_register).expect("realization register"),
        fs::read(&paths.model_labels_json).expect("model label publication"),
    ];
    let (report, diagnostics) = check_repository(&paths);
    assert!(diagnostics.is_empty(), "{diagnostics:#?}");
    assert!(report.valid);
    assert!(report.imported_citations > 0);
    let after = [
        fs::read(&paths.specification_register).expect("Layer-0 register"),
        fs::read(&paths.realization_register).expect("realization register"),
        fs::read(&paths.model_labels_json).expect("model label publication"),
    ];
    assert_eq!(before, after, "the repository check must not write outputs");
}

#[test]
fn register_generation_is_deterministic_in_an_explicit_output_root() {
    let paths = RepositoryPaths::workspace_default();
    let output = tempfile::tempdir().expect("temporary output root");

    generate_registers(&paths, output.path()).expect("first register generation");
    let first_specification = fs::read(output.path().join("plans/labels/specification.md"))
        .expect("generated Layer-0 register");
    let first_realization = fs::read(output.path().join("plans/labels/realization.md"))
        .expect("generated realization register");

    generate_registers(&paths, output.path()).expect("second register generation");
    assert_eq!(
        first_specification,
        fs::read(output.path().join("plans/labels/specification.md"))
            .expect("generated Layer-0 register")
    );
    assert_eq!(
        first_realization,
        fs::read(output.path().join("plans/labels/realization.md"))
            .expect("generated realization register")
    );
}
