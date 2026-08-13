use std::{fs, path::Path};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use petgraph::Direction;

use crate::{
    LabelErrorCode,
    census::{CensusGroup, RepositoryCensus},
    label::{Label, LabelShape},
    latex::harvest_attestation,
    markdown::{InlineCodeContext, scan_markdown},
    model_labels_json,
    owner::{ImportedLabel, LabelOwner},
    repository::{
        CitationClass, LabelGraphEdge, LabelGraphNode, RepositoryLabels, generate_registers,
    },
    rust_source::{RustHarvest, harvest_model},
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
fn malformed_citation_group_with_trailing_prose_is_not_bare() {
    // The close parenthesis arrives only after prose, so the exact
    // citation grammar fails. The occurrence is an attempted citation
    // and must be diagnosed rather than demoted to a mint.
    let scan = scan_markdown(
        Path::new("fixture.md"),
        "See (`sec:fixture`, ordinary supporting prose).\n",
    );
    assert_eq!(
        scan.code_spans[0].context,
        InlineCodeContext::MalformedGroup
    );
}

#[test]
fn malformed_citation_group_with_leading_prose_is_not_bare() {
    let scan = scan_markdown(Path::new("fixture.md"), "(see `sec:fixture`)\n");
    assert_eq!(
        scan.code_spans[0].context,
        InlineCodeContext::MalformedGroup
    );
}

#[test]
fn multiple_citation_groups_on_one_line_are_each_parenthesized() {
    let scan = scan_markdown(
        Path::new("fixture.md"),
        "Both (`sec:first`) and (`sec:second`, `sec:third`) cite.\n",
    );
    assert!(scan.diagnostics.is_empty());
    assert_eq!(scan.code_spans.len(), 3);
    assert!(
        scan.code_spans
            .iter()
            .all(|span| span.context == InlineCodeContext::Parenthesized)
    );
}

#[test]
fn nested_parentheses_around_a_citation_group_stay_a_citation() {
    let scan = scan_markdown(Path::new("fixture.md"), "Aside ((`sec:fixture`)) here.\n");
    assert_eq!(scan.code_spans[0].context, InlineCodeContext::Parenthesized);
}

#[test]
fn unrelated_parentheses_elsewhere_never_change_a_bare_mint() {
    // A parenthesis before and after the occurrence, neither adjacent
    // to it: the occurrence is an ordinary mint, and the far
    // parentheses neither promote nor demote it.
    let scan = scan_markdown(
        Path::new("fixture.md"),
        "Prose (an aside) `sec:fixture` more (another aside).\n",
    );
    assert_eq!(scan.code_spans[0].context, InlineCodeContext::Bare);
}

#[test]
fn a_malformed_citation_never_becomes_the_only_mint_of_a_label() {
    // The label appears exactly once in the document, inside a
    // malformed citation group. Before this rule the later close
    // parenthesis suppressed the diagnostic and the occurrence minted
    // the label, silently moving its conceptual home.
    let directory = fixture_root("# Realization\nSee (`sec:fixture`, supporting prose).\n");
    let labels = RepositoryLabels::harvest_sources(&RepositoryCensus::discover(directory.path()));

    assert!(
        labels
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == LabelErrorCode::MalformedCitationGroup),
        "{:#?}",
        labels.diagnostics,
    );
    assert!(
        !labels
            .registries
            .realization
            .labels()
            .any(|label| label.as_str() == "sec:fixture"),
        "malformed citation minted the label it cites",
    );
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
fn blockquoted_fence_is_rejected_rather_than_silently_scanned() {
    // The accepted grammar carries top-level fences only. A fence
    // behind a container marker is rejected, so a label-shaped token
    // inside it can never participate silently.
    let scan = scan_markdown(
        Path::new("fixture.md"),
        "> ```text\n> `sec:quoted`\n> ```\n",
    );
    let nested: Vec<_> = scan
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == LabelErrorCode::NestedMarkdownFence)
        .map(|diagnostic| (diagnostic.line, diagnostic.column))
        .collect();
    assert_eq!(nested, vec![(1, 3), (3, 3)], "{:#?}", scan.diagnostics);
}

#[test]
fn list_item_and_indented_fences_are_rejected() {
    for source in [
        "- ```text\n`sec:listed`\n",
        "1. ```text\n`sec:ordered`\n",
        "    ```text\n`sec:indented`\n",
    ] {
        let scan = scan_markdown(Path::new("fixture.md"), source);
        assert!(
            scan.diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == LabelErrorCode::NestedMarkdownFence),
            "{source:?} {:#?}",
            scan.diagnostics,
        );
    }
}

#[test]
fn top_level_fences_and_ordinary_containers_are_not_nested_fences() {
    let scan = scan_markdown(
        Path::new("fixture.md"),
        "> quoted prose\n- listed prose\n   ```text\n`sec:fenced`\n   ```\n`sec:visible`\n",
    );
    assert!(scan.diagnostics.is_empty(), "{:#?}", scan.diagnostics);
    assert_eq!(scan.code_spans.len(), 1);
    assert_eq!(scan.code_spans[0].content, "sec:visible");
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
fn duplicate_attestation_mint_diagnostic_names_both_locations() {
    // ADR-013 (F3-007): a duplicate-mint diagnostic identifies both
    // the duplicate occurrence and the first mint, in canonical
    // repository-relative locations, for the attestation owner exactly as for the
    // shared insert_or_diagnose owners.
    let directory = tempfile::tempdir().expect("temporary repository");
    let root = directory.path();
    fs::create_dir_all(root.join("papers/attestation/sections"))
        .expect("attestation sections directory");
    fs::write(root.join("papers/attestation/main.tex"), "% main\n").expect("attestation main");
    fs::write(
        root.join("papers/attestation/sections/first.tex"),
        "\\label{def:model:classes}\n",
    )
    .expect("first section");
    fs::write(
        root.join("papers/attestation/sections/second.tex"),
        "% preamble\n\\label{def:model:classes}\n",
    )
    .expect("second section");

    let (_registry, diagnostics) = harvest_attestation(&RepositoryCensus::discover(root));
    let duplicate = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == LabelErrorCode::DuplicateLatexLabel)
        .expect("duplicate attestation diagnostic");
    // The duplicate occurrence is the diagnostic's own location; the
    // first mint is named in the message.
    assert_eq!(duplicate.path, "papers/attestation/sections/second.tex");
    assert_eq!(duplicate.line, 2);
    assert!(
        duplicate
            .message
            .contains("first minted at papers/attestation/sections/first.tex:1"),
        "{}",
        duplicate.message
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
    assert_eq!(
        harvest
            .citations
            .iter()
            .filter(|citation| citation.class == CitationClass::AuthoredImported)
            .count(),
        1,
    );
    assert!(harvest.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == LabelErrorCode::InvalidImportedCitationForm && diagnostic.line == 5
    }));
}

#[test]
fn plan_local_labels_resolve_while_unknown_imports_fail() {
    let directory = tempfile::tempdir().expect("temporary repository");
    let root = directory.path();
    for directory in [
        "papers/attestation/sections",
        "docs/attestation",
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
        "docs/attestation",
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

#[cfg(unix)]
#[test]
fn unreadable_adr_is_an_io_diagnostic() {
    let directory = fixture_root("# Realization\n`sec:fixture`\n");
    let root = directory.path();
    let adr = root.join("adr/012-unreadable.md");

    fs::write(&adr, "# ADR\n`rule:labels:defined`\n").expect("ADR source");
    fs::set_permissions(&adr, fs::Permissions::from_mode(0o0)).expect("make ADR unreadable");

    if fs::read_to_string(&adr).is_ok() {
        fs::set_permissions(&adr, fs::Permissions::from_mode(0o644)).expect("restore readable ADR");
        return;
    }

    let labels = RepositoryLabels::harvest_sources(&RepositoryCensus::discover(root));

    fs::set_permissions(&adr, fs::Permissions::from_mode(0o644)).expect("restore readable ADR");

    assert!(labels.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == LabelErrorCode::Io && diagnostic.path == "adr/012-unreadable.md"
    }));
}

/// Make `path` unreadable, returning `false` when the process can read
/// it anyway — running as root, or on a filesystem without permission
/// enforcement — so the caller can skip rather than assert falsely.
#[cfg(unix)]
fn make_unreadable(path: &Path) -> bool {
    fs::set_permissions(path, fs::Permissions::from_mode(0o0)).expect("remove permissions");
    if fs::read_dir(path).is_ok() {
        fs::set_permissions(path, fs::Permissions::from_mode(0o755)).expect("restore permissions");
        return false;
    }
    true
}

#[cfg(unix)]
#[test]
fn unreadable_subject_directory_fails_an_empty_declared_group() {
    // The hole this closes: an unreadable directory used to discover
    // as an empty group, so an empty declared group agreed with it and
    // verification passed while subjects sat outside the label graph.
    let directory = fixture_root("# Realization\n`sec:fixture`\n");
    let root = directory.path();
    let adr_dir = root.join("adr");
    fs::write(
        adr_dir.join("012-hidden.md"),
        "# ADR\n`rule:hidden:label`\n",
    )
    .expect("ADR source");
    if !make_unreadable(&adr_dir) {
        return;
    }

    let declared = RepositoryCensus {
        root: root.to_path_buf(),
        ..RepositoryCensus::default()
    };
    let diagnostics = declared.verify(&[CensusGroup::Adr]);

    fs::set_permissions(&adr_dir, fs::Permissions::from_mode(0o755))
        .expect("restore ADR directory");

    assert!(
        diagnostics.iter().any(|diagnostic| {
            diagnostic.code == LabelErrorCode::CensusUnreadable && diagnostic.path == "adr"
        }),
        "{diagnostics:#?}",
    );
}

#[cfg(unix)]
#[test]
fn one_unreadable_entry_is_reported_beside_the_readable_ones() {
    let directory = fixture_root("# Realization\n`sec:fixture`\n");
    let root = directory.path();
    fs::write(root.join("plans/first.md"), "# First\n").expect("plan source");
    fs::write(root.join("plans/second.md"), "# Second\n").expect("plan source");
    let closed = root.join("plans/closed");
    fs::create_dir_all(&closed).expect("nested plan directory");
    fs::write(closed.join("third.md"), "# Third\n").expect("plan source");
    if !make_unreadable(&closed) {
        return;
    }

    let census = RepositoryCensus::discover(root);

    fs::set_permissions(&closed, fs::Permissions::from_mode(0o755))
        .expect("restore plan directory");

    // The readable siblings are still discovered.
    let names: Vec<_> = census
        .plans
        .iter()
        .filter_map(|path| path.file_name().and_then(|name| name.to_str()))
        .collect();
    assert_eq!(names, vec!["first.md", "second.md"]);

    let failures = census
        .traversal
        .get(&CensusGroup::Plan)
        .expect("plan traversal record");
    assert_eq!(failures.len(), 1, "{failures:#?}");
    assert_eq!(failures[0].code, LabelErrorCode::CensusUnreadable);
    assert_eq!(failures[0].path, "plans/closed");
}

#[test]
fn absent_and_empty_directories_are_equally_clean_empty_groups() {
    // Policy: a directory that is absent and a directory that is
    // present but empty both yield an empty group with no traversal
    // diagnostic. Only a directory that exists and cannot be read is a
    // failure — an empty group is knowledge, an unreadable one is not.
    let absent = fixture_root("# Realization\n`sec:fixture`\n");
    fs::remove_dir_all(absent.path().join("adr")).expect("remove ADR directory");
    let empty = fixture_root("# Realization\n`sec:fixture`\n");

    for root in [absent.path(), empty.path()] {
        let census = RepositoryCensus::discover(root);
        assert!(census.adrs.is_empty());
        assert!(
            census
                .traversal
                .get(&CensusGroup::Adr)
                .is_some_and(Vec::is_empty),
            "{:#?}",
            census.traversal,
        );
        assert!(census.verify(&[CensusGroup::Adr]).is_empty());
    }
}

// Whole-repository validity is the job of the Meson-driven
// `check-labels` target (ADR-014): unit tests never discover the live
// checkout, so register generation is exercised on synthetic fixtures
// only.
#[test]
fn register_generation_is_deterministic_in_explicit_outputs() {
    let directory =
        fixture_root("# Realization\n`sec:fixture`\nBody cite (`[A-def:model:known]`).\n");
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
        "docs/attestation",
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
        "docs/attestation",
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
fn status_tags_are_audited_in_their_grammar_class() {
    // Each bracketed token is audited in its own grammar class: live status
    // tags resolve their pin and clause references, template examples are
    // exempted, and malformed tags are typed InvalidStatusTag diagnostics.
    let directory = fixture_root(concat!(
        "# Realization\n",                          // 1
        "`sec:fixture`\n",                          // 2
        "`pin:pins:fixture`\n",                     // 3 — the pin an enforced tag resolves to
        "Enforced `[enforced: P-fixture]` live.\n", // 4 — valid, resolves
        "Invariant `[invariant: 𝗜₅]` live.\n",      // 5 — valid clause ordinal
        "Tag `[accepted residual]` live.\n",        // 6 — valid tag
        "Example `[enforced: P-…]` shown.\n",       // 7 — placeholder → exempt
        "Example `[invariant: 𝗜ₙ]` shown.\n",       // 8 — placeholder → exempt
        "Unminted `[enforced: P-nonesuch]` bad.\n", // 9 — unresolved pin
        "Shapeless `[enforced: burn]` bad.\n",      // 10 — not a P- glyph
        "Range `[invariant: 𝗜₉₉]` bad.\n",          // 11 — clause out of range
        "Wrapped (`[enforced: P-fixture]`) bad.\n", // 12 — round-wrapped
    ));

    let labels = RepositoryLabels::harvest_sources(&RepositoryCensus::discover(directory.path()));
    let realization_diagnostics = labels
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.path == "docs/attestation/realization.md")
        .map(|diagnostic| (diagnostic.line, diagnostic.code))
        .collect::<Vec<_>>();

    assert_eq!(
        realization_diagnostics,
        vec![
            (9, LabelErrorCode::InvalidStatusTag),
            (10, LabelErrorCode::InvalidStatusTag),
            (11, LabelErrorCode::InvalidStatusTag),
            (12, LabelErrorCode::InvalidStatusTag),
        ],
        "{:#?}",
        labels.diagnostics,
    );
}

#[test]
fn repository_analysis_exposes_a_direct_petgraph_graph() {
    let directory = fixture_root("# Realization\n`sec:fixture`\n(`sec:fixture`)\n");
    let labels = RepositoryLabels::harvest_sources(&RepositoryCensus::discover(directory.path()));
    let graph: &petgraph::graph::DiGraph<LabelGraphNode, LabelGraphEdge, u32> = &labels.graph;

    assert!(graph.node_count() > 0);
}

#[test]
fn every_authored_citation_resolves_to_one_mint_edge() {
    let directory = fixture_root(concat!(
        "# Realization\n",
        "`sec:fixture`\n",
        "(`sec:fixture`)\n",
        "Body cite (`[A-def:model:known]`).\n",
    ));
    let labels = RepositoryLabels::harvest_sources(&RepositoryCensus::discover(directory.path()));

    for node in labels.graph.node_indices() {
        let LabelGraphNode::Citation(citation) = &labels.graph[node] else {
            continue;
        };

        if citation.class == CitationClass::SyntheticArchitecture {
            continue;
        }

        assert_eq!(
            labels
                .graph
                .edges_directed(node, Direction::Outgoing)
                .count(),
            1,
        );
    }
}

#[test]
fn self_qualified_import_is_rejected_by_the_graph_builder() {
    let directory = fixture_root("# Realization\n`sec:fixture`\n");
    let root = directory.path();

    fs::write(
        root.join("adr/013-fixture.md"),
        concat!(
            "# ADR\n",
            "`rule:fixture:defined`\n",
            "(`[ADR013-rule:fixture:defined]`)\n",
        ),
    )
    .unwrap();

    let labels = RepositoryLabels::harvest_sources(&RepositoryCensus::discover(root));

    assert!(
        labels
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.code == LabelErrorCode::InvalidImportedCitationForm })
    );
}

// Guards the deliberately retained petgraph `serde-1` feature (P2-002): the
// raw label graph is not yet serialized by product code, but serialization is
// anticipated for noncanonical diagnostics, so the feature and this guard are
// kept while the other petgraph features were dropped as unused.
#[test]
fn petgraph_serde_feature_is_available_for_noncanonical_diagnostics() {
    let mut graph = petgraph::graph::DiGraph::<u8, u8, u32>::new();
    graph.add_node(1);

    let rendered = serde_json::to_string(&graph).unwrap();

    assert!(!rendered.is_empty());
}

#[test]
fn attestation_anchor_set_derives_from_body_citations_only() {
    // A fenced example and the generated upward-citation index must
    // not contribute anchors; an index token without a body citation
    // is a stale index, not a member of the anchor set.
    let directory = fixture_root(concat!(
        "# Realization\n",
        "`sec:fixture`\n",
        "Body cite (`[A-def:model:known]`).\n",
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
fn malformed_attestation_import_is_invalid_label() {
    let directory = fixture_root("# Realization\n`sec:fixture`\n(`[A-def:Bad]`)\n");

    let labels = RepositoryLabels::harvest_sources(&RepositoryCensus::discover(directory.path()));

    assert!(labels.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == LabelErrorCode::InvalidLabel
            && diagnostic.path == "docs/attestation/realization.md"
            && diagnostic.line == 3
    }));
}

#[test]
fn unclosed_attestation_import_is_diagnostic() {
    let directory = fixture_root("# Realization\n`sec:fixture`\n(`[A-def:model:known]\n");

    let labels = RepositoryLabels::harvest_sources(&RepositoryCensus::discover(directory.path()));

    assert!(labels.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == LabelErrorCode::UnclosedInlineCode
            && diagnostic.path == "docs/attestation/realization.md"
            && diagnostic.line == 3
    }));
}

#[test]
fn non_parenthesized_attestation_import_is_rejected() {
    let directory = fixture_root("# Realization\n`sec:fixture`\n`[A-def:model:known]`\n");

    let labels = RepositoryLabels::harvest_sources(&RepositoryCensus::discover(directory.path()));

    assert!(labels.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == LabelErrorCode::InvalidImportedCitationForm
            && diagnostic.path == "docs/attestation/realization.md"
            && diagnostic.line == 3
    }));
}

#[test]
fn raw_text_attestation_token_is_nonparticipating() {
    let directory = fixture_root("# Realization\n`sec:fixture`\nRaw [A-def:model:known].\n");

    let labels = RepositoryLabels::harvest_sources(&RepositoryCensus::discover(directory.path()));

    assert_eq!(labels.imported_citation_count(), 0);
    assert!(!labels.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic.code,
        LabelErrorCode::InvalidImportedCitationForm | LabelErrorCode::UnknownImportedLabel
    )));
}

#[test]
fn double_backtick_attestation_example_is_nonparticipating() {
    let directory =
        fixture_root("# Realization\n`sec:fixture`\nExample ``[A-def:model:missing]``.\n");

    let labels = RepositoryLabels::harvest_sources(&RepositoryCensus::discover(directory.path()));

    assert!(!labels.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == LabelErrorCode::UnknownImportedLabel
            && diagnostic.message.contains("def:model:missing")
    }));
}

#[test]
fn fenced_attestation_example_is_nonparticipating() {
    let directory =
        fixture_root("# Realization\n`sec:fixture`\n```text\n(`[A-def:model:missing]`)\n```\n");

    let labels = RepositoryLabels::harvest_sources(&RepositoryCensus::discover(directory.path()));

    assert!(!labels.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == LabelErrorCode::UnknownImportedLabel
            && diagnostic.message.contains("def:model:missing")
    }));
}

#[test]
fn upward_index_token_does_not_sustain_body_anchor() {
    let directory = fixture_root(concat!(
        "# Realization\n",
        "`sec:fixture`\n",
        "## §17 Upward-citation index · `sec:realization:anchors`\n",
        "| `[A-def:model:known]` | (`sec:fixture`) |\n",
    ));

    let labels = RepositoryLabels::harvest_sources(&RepositoryCensus::discover(directory.path()));

    assert!(labels.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == LabelErrorCode::AttestationIndexStale
            && diagnostic.path == "docs/attestation/realization.md"
    }));
    assert_eq!(labels.imported_citation_count(), 0);
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
fn rust_container_nested_documentation_fence_is_rejected() {
    let harvest = rust_fixture_harvest(concat!(
        "//! > ```text\n",
        "//! > ´def:fixture:quoted´\n",
        "//! > ```\n",
    ));

    assert!(
        harvest
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == LabelErrorCode::NestedMarkdownFence),
        "{:#?}",
        harvest.diagnostics,
    );
}

#[test]
fn rust_malformed_citation_groups_are_diagnosed_not_minted() {
    let harvest = rust_fixture_harvest(concat!(
        "// intended cite (´def:fixture:prose´ with trailing prose)\n",
        "// trailing close (see ´def:fixture:leading´)\n",
        "// unrelated (aside) ´def:fixture:mint´ more (aside)\n",
        "// groups (´def:fixture:first´) and (´def:fixture:second´)\n",
        "// nested ((´def:fixture:nested´))\n",
    ));

    let minted: Vec<_> = harvest.registry.labels().map(ToString::to_string).collect();
    assert_eq!(
        minted,
        vec!["def:fixture:mint".to_owned()],
        "{:#?}",
        harvest.diagnostics,
    );

    let defects: Vec<_> = harvest
        .diagnostics
        .iter()
        .map(|diagnostic| (diagnostic.line, diagnostic.code))
        .collect();
    assert_eq!(
        defects,
        vec![
            (1, LabelErrorCode::MalformedCitationGroup),
            (2, LabelErrorCode::MalformedCitationGroup),
        ],
        "{:#?}",
        harvest.diagnostics,
    );

    // The three well-formed groups became citations, not mints.
    assert_eq!(harvest.citations.len(), 3);
}

#[test]
fn rust_malformed_citation_never_becomes_the_only_mint_of_a_label() {
    let harvest = rust_fixture_harvest("// only occurrence (´def:fixture:home´ and prose)\n");

    assert!(harvest.registry.labels().next().is_none());
    assert!(harvest.citations.is_empty());
    assert!(
        harvest
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == LabelErrorCode::MalformedCitationGroup),
        "{:#?}",
        harvest.diagnostics,
    );
}

fn rust_fixture_harvest(source: &str) -> RustHarvest {
    let directory = tempfile::tempdir().expect("temporary repository");
    let root = directory.path();
    let source_directory = root.join("packages/model/src");
    fs::create_dir_all(&source_directory).expect("model source directory");
    fs::write(source_directory.join("fixture.rs"), source).expect("model source");
    harvest_model(&RepositoryCensus::discover(root))
}

// F2-003 fence scoping: only documentation comments open fences, and a
// fence never outlives its contiguous documentation block.

#[test]
fn plain_comment_fence_markers_do_not_suppress_labels() {
    let harvest = rust_fixture_harvest(concat!(
        "// ```text\n",
        "fn unrelated_code() {}\n",
        "// \u{b4}def:fixture:hidden\u{b4}\n",
        "// ```\n",
    ));

    assert!(
        harvest
            .registry
            .contains(&Label::parse("def:fixture:hidden", LabelShape::Model).expect("valid label")),
        "{:#?}",
        harvest.diagnostics,
    );
    assert!(harvest.diagnostics.is_empty(), "{:#?}", harvest.diagnostics);
}

#[test]
fn outer_doc_fenced_example_is_nonparticipating() {
    let harvest = rust_fixture_harvest(concat!(
        "/// ```text\n",
        "/// \u{b4}def:fixture:outer-fenced\u{b4}\n",
        "/// ```\n",
        "fn documented() {}\n",
    ));

    assert!(!harvest.registry.contains(
        &Label::parse("def:fixture:outer-fenced", LabelShape::Model).expect("valid label")
    ));
    assert!(harvest.diagnostics.is_empty(), "{:#?}", harvest.diagnostics);
}

#[test]
fn block_doc_fenced_example_is_nonparticipating() {
    let harvest = rust_fixture_harvest(concat!(
        "/** docs\n",
        "```text\n",
        "\u{b4}def:fixture:block-fenced\u{b4}\n",
        "```\n",
        "*/\n",
        "fn documented() {}\n",
        "/* ordinary block \u{b4}def:fixture:block-plain\u{b4} */\n",
    ));

    // The documentation block comment hides its fenced example; the
    // ordinary block comment cannot be fenced and its label
    // participates.
    assert!(
        !harvest.registry.contains(
            &Label::parse("def:fixture:block-fenced", LabelShape::Model).expect("valid label")
        ),
        "{:#?}",
        harvest.diagnostics,
    );
    assert!(
        harvest.registry.contains(
            &Label::parse("def:fixture:block-plain", LabelShape::Model).expect("valid label")
        ),
        "{:#?}",
        harvest.diagnostics,
    );
}

#[test]
fn open_fence_stops_at_the_end_of_its_documentation_block() {
    let harvest = rust_fixture_harvest(concat!(
        "/// ```text\n",
        "fn code_between() {}\n",
        "// \u{b4}def:fixture:after-code\u{b4}\n",
        "/// fresh block \u{b4}def:fixture:next-doc\u{b4}\n",
    ));

    // The unclosed fence is diagnosed at its opening documentation
    // line and suppresses nothing outside its own block.
    assert!(harvest.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == LabelErrorCode::UnclosedMarkdownFence && diagnostic.line == 1
    }));
    assert!(
        harvest.registry.contains(
            &Label::parse("def:fixture:after-code", LabelShape::Model).expect("valid label")
        ),
        "{:#?}",
        harvest.diagnostics,
    );
    assert!(
        harvest.registry.contains(
            &Label::parse("def:fixture:next-doc", LabelShape::Model).expect("valid label")
        ),
        "{:#?}",
        harvest.diagnostics,
    );
}

#[test]
fn ordinary_comment_between_fenced_doc_lines_participates() {
    let harvest = rust_fixture_harvest(concat!(
        "/// ```text\n",
        "// \u{b4}def:fixture:interleaved\u{b4}\n",
        "/// ```\n",
    ));

    // The ordinary comment is a different block, so it participates;
    // both severed documentation fragments leave open fences behind
    // (the closing line reads as a new opening fence).
    assert!(
        harvest.registry.contains(
            &Label::parse("def:fixture:interleaved", LabelShape::Model).expect("valid label")
        ),
        "{:#?}",
        harvest.diagnostics,
    );
    assert!(
        harvest
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.code == LabelErrorCode::UnclosedMarkdownFence })
    );
}

#[test]
fn labels_before_and_after_a_fenced_example_participate() {
    let harvest = rust_fixture_harvest(concat!(
        "/// before \u{b4}def:fixture:before\u{b4}\n",
        "/// ```text\n",
        "/// \u{b4}def:fixture:inside\u{b4}\n",
        "/// ```\n",
        "/// after \u{b4}def:fixture:after\u{b4}\n",
    ));

    let minted: Vec<_> = harvest.registry.labels().map(ToString::to_string).collect();
    assert_eq!(
        minted,
        vec![
            "def:fixture:after".to_owned(),
            "def:fixture:before".to_owned(),
        ],
        "{:#?}",
        harvest.diagnostics,
    );
    assert!(harvest.diagnostics.is_empty(), "{:#?}", harvest.diagnostics);
}

#[test]
fn scoped_derivations_ignore_cross_owner_imports_and_index_staleness() {
    // A model comment citing another owner, and a stale §17 index,
    // both fail the full repository check but must not block the
    // scoped register/model-label derivations.
    let directory = fixture_root(concat!(
        "# Realization\n",
        "`sec:fixture`\n",
        "Body cite (`[A-def:model:known]`).\n",
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
        "Body cite (`[A-def:model:known]`).\n",
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
fn malformed_owner_qualified_plan_tokens_fail_exhaustively() {
    // Every bracket-free owner-qualified form a planning author can
    // write, in one file, so the diagnostic set and its order are
    // pinned together: a valid local mint and a valid import stay
    // silent, a known owner without brackets and every unknown or
    // malformed owner fail closed, and ordinary inline code, a
    // double-backtick example, and a fenced example stay out.
    let directory = fixture_root("# Realization\n`sec:fixture`\n");
    let root = directory.path();
    fs::write(
        root.join("plans/tokens.md"),
        concat!(
            "# Tokens\n",
            "`rule:fixture:local`\n",
            "Valid import (`[A-def:model:known]`).\n",
            "No brackets (`A-def:model:known`).\n",
            "Short ADR owner (`ADR01-rule:fixture:local`).\n",
            "Mistyped owner (`PLN-rule:fixture:local`).\n",
            "Unknown owner (`UNKNOWN-rule:fixture:local`).\n",
            "Malformed local label (`ADR013-rule:Bad`).\n",
            "Ordinary code `UTF-8`, `SHA-256`, and `--stamp`.\n",
            "Example ``ADR01-rule:fixture:local``.\n",
            "```text\n",
            "ADR01-rule:fixture:local\n",
            "```\n",
        ),
    )
    .expect("plan token fixture");

    let labels = RepositoryLabels::harvest_sources(&RepositoryCensus::discover(root));

    let reported: Vec<_> = labels
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.path == "plans/tokens.md")
        .map(|diagnostic| (diagnostic.code, diagnostic.line))
        .collect();
    assert_eq!(
        reported,
        vec![
            (LabelErrorCode::InvalidImportedCitationForm, 4),
            (LabelErrorCode::UnknownOwner, 5),
            (LabelErrorCode::UnknownOwner, 6),
            (LabelErrorCode::UnknownOwner, 7),
            (LabelErrorCode::UnknownOwner, 8),
        ],
        "{:#?}",
        labels.diagnostics,
    );
}

#[test]
fn malformed_owner_qualified_doc_token_fails_closed() {
    // The guard belongs to the shared Markdown-owner harvest, so the
    // DOC owner fails the same way the PLAN owner does.
    let directory = fixture_root("# Realization\n`sec:fixture`\n");
    let root = directory.path();
    fs::write(
        root.join("README.md"),
        "# Repository\n`sec:readme:versions`\nCite (`PLN-sec:readme:versions`).\n",
    )
    .expect("repository README");

    let labels = RepositoryLabels::harvest_sources(&RepositoryCensus::discover(root));

    assert!(
        labels.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == LabelErrorCode::UnknownOwner
                && diagnostic.path == "README.md"
                && diagnostic.line == 3
        }),
        "{:#?}",
        labels.diagnostics,
    );
}

#[test]
fn malformed_owner_qualified_adr_tokens_fail_closed() {
    // The ADR owner shares the bracket-free classifier, so a known
    // owner written without brackets and an unknown owner prefix fail
    // there exactly as they do for PLAN and DOC.
    let directory = fixture_root("# Realization\n`sec:fixture`\n");
    let root = directory.path();
    fs::write(
        root.join("adr/012-fixture.md"),
        concat!(
            "# ADR\n",
            "`rule:fixture:local`\n",
            "No brackets (`A-def:model:known`).\n",
            "Mistyped owner (`PLN-rule:fixture:local`).\n",
            "Ordinary code `UTF-8`.\n",
        ),
    )
    .expect("ADR token fixture");

    let labels = RepositoryLabels::harvest_sources(&RepositoryCensus::discover(root));

    let reported: Vec<_> = labels
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.path == "adr/012-fixture.md")
        .map(|diagnostic| (diagnostic.code, diagnostic.line))
        .collect();
    assert_eq!(
        reported,
        vec![
            (LabelErrorCode::InvalidImportedCitationForm, 3),
            (LabelErrorCode::UnknownOwner, 4),
        ],
        "{:#?}",
        labels.diagnostics,
    );
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

#[test]
fn malformed_model_label_does_not_block_upstream_register_generation() {
    let directory =
        fixture_root("# Realization\n`sec:fixture`\nBody cite (`[A-def:model:known]`).\n");
    let root = directory.path();
    fs::write(
        root.join("packages/model/src/fixture.rs"),
        "// ´def:broken´\n",
    )
    .expect("malformed model source");

    let paths = RepositoryCensus::discover(root);
    let output = tempfile::tempdir().expect("temporary register output");
    let specification_output = output.path().join("specification.md");
    let realization_output = output.path().join("realization.md");

    generate_registers(&paths, &specification_output, &realization_output)
        .expect("a malformed model-owned label must not block upstream register generation");

    assert!(specification_output.is_file());
    assert!(realization_output.is_file());

    let model = harvest_model(&paths);
    assert!(
        model.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == LabelErrorCode::InvalidLabel
                && diagnostic.path == "packages/model/src/fixture.rs"
        }),
        "{:#?}",
        model.diagnostics
    );

    let complete = RepositoryLabels::harvest_sources(&paths);
    assert!(
        complete.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == LabelErrorCode::InvalidLabel
                && diagnostic.path == "packages/model/src/fixture.rs"
        }),
        "{:#?}",
        complete.diagnostics
    );
}

#[test]
fn census_audit_welds_declared_lists_to_the_tracked_set() {
    let pattern =
        regex::Regex::new("(^|/)[.]|^archive/|(^|/)meson[.](build|options)$|[.](bib|zip)$")
            .expect("valid exclusion pattern");
    let tracked = [
        ".gitignore",
        "adr/010-fixture.md",
        "adr/meson.build",
        "archive/old.md",
        "papers/attestation/macros.tex",
        "papers/attestation/main.tex",
        "papers/attestation/references.bib",
    ]
    .map(|path| crate::census::TrackedEntry {
        mode: "100644",
        path,
    });

    // Complete census: categorical exclusions and the explicit
    // same-typed exclusion cover everything undeclared.
    let report = crate::census::audit_census(
        tracked,
        ["adr/010-fixture.md", "papers/attestation/main.tex"],
        ["papers/attestation/macros.tex"],
        &pattern,
    );
    assert!(report.valid, "{report:?}");
    assert_eq!((report.tracked, report.subjects), (7, 2));

    // A tracked subject missing from the declared lists fails, naming
    // the path; so does a declared entry that is not tracked.
    let report = crate::census::audit_census(
        tracked,
        ["adr/010-fixture.md", "plans/ghost.md"],
        ["papers/attestation/macros.tex"],
        &pattern,
    );
    assert!(!report.valid);
    assert_eq!(
        report.missing_from_census,
        vec!["papers/attestation/main.tex".to_owned()]
    );
    assert_eq!(report.not_tracked, vec!["plans/ghost.md".to_owned()]);
}

#[test]
fn tracked_listing_parses_modes_and_refuses_malformed_records() {
    use crate::census::{TrackedEntry, parse_tracked_listing};

    let listing = concat!(
        "100644 aaaa 0\tadr/010-fixture.md\u{0}",
        "100755 bbbb 0\tscripts/ci.sh\u{0}",
    );
    assert_eq!(
        parse_tracked_listing(listing).expect("well-formed listing"),
        vec![
            TrackedEntry {
                mode: "100644",
                path: "adr/010-fixture.md",
            },
            TrackedEntry {
                mode: "100755",
                path: "scripts/ci.sh",
            },
        ]
    );

    // A record without a tab separator is an error, never a skipped
    // entry: dropping one would silently remove exactly the tracked
    // symlink this audit exists to catch.
    let error = parse_tracked_listing("100644 aaaa 0 adr/010-fixture.md\u{0}")
        .expect_err("a record without a path separator fails");
    assert!(error.contains("no path separator"), "{error}");
}

#[test]
fn census_audit_rejects_tracked_symlinks_and_gitlinks() {
    use crate::census::{TrackedEntry, TrackedModeDefect};

    let pattern = regex::Regex::new("^(archive|vendor)/").expect("valid exclusion pattern");
    let report = crate::census::audit_census(
        [
            TrackedEntry {
                mode: "100644",
                path: "adr/010-fixture.md",
            },
            TrackedEntry {
                mode: "120000",
                path: "plans/alias.md",
            },
            // Categorically excluded from lint subjects, yet still
            // bound by the repository-shape rule: lint exclusion is
            // not an exemption.
            TrackedEntry {
                mode: "160000",
                path: "archive/vendored",
            },
        ],
        ["adr/010-fixture.md", "plans/alias.md"],
        [],
        &pattern,
    );

    assert!(!report.valid, "{report:?}");
    assert_eq!(
        report.disallowed_modes,
        vec![
            TrackedModeDefect {
                path: "archive/vendored".to_owned(),
                mode: "160000".to_owned(),
            },
            TrackedModeDefect {
                path: "plans/alias.md".to_owned(),
                mode: "120000".to_owned(),
            },
        ]
    );
    assert!(
        report.missing_from_census.is_empty() && report.not_tracked.is_empty(),
        "the census itself agrees; only repository shape fails: {report:?}"
    );
}

#[test]
fn register_generation_repairs_mixed_state_and_preserves_mtimes() {
    // T3: a prior partial publication (one stale register) is repaired
    // without rewriting the already-current member.
    let directory =
        fixture_root("# Realization\n`sec:fixture`\nBody cite (`[A-def:model:known]`).\n");
    let paths = RepositoryCensus::discover(directory.path());
    let output = tempfile::tempdir().expect("temporary output root");
    let specification_output = output.path().join("specification.md");
    let realization_output = output.path().join("realization.md");

    generate_registers(&paths, &specification_output, &realization_output)
        .expect("first register generation");
    let specification_mtime = fs::metadata(&specification_output)
        .expect("attestation metadata")
        .modified()
        .expect("attestation mtime");
    fs::write(&realization_output, b"mixed prior generation").expect("corrupt one register");

    generate_registers(&paths, &specification_output, &realization_output)
        .expect("repair register generation");

    assert_ne!(
        fs::read(&realization_output).expect("repaired register"),
        b"mixed prior generation"
    );
    assert_eq!(
        fs::metadata(&specification_output)
            .expect("attestation metadata")
            .modified()
            .expect("attestation mtime"),
        specification_mtime,
        "the current register must not be rewritten during repair",
    );
}

#[test]
fn register_generation_staging_failure_leaves_the_other_register_unchanged() {
    // T3: the realization destination's parent is a regular file, so
    // its staging fails; the specification register must keep its old bytes.
    let directory =
        fixture_root("# Realization\n`sec:fixture`\nBody cite (`[A-def:model:known]`).\n");
    let paths = RepositoryCensus::discover(directory.path());
    let output = tempfile::tempdir().expect("temporary output root");
    let specification_output = output.path().join("specification.md");
    fs::write(&specification_output, b"previous generation").expect("seed attestation");
    let blocker = output.path().join("blocker");
    fs::write(&blocker, b"file").expect("seed blocker");
    let realization_output = blocker.join("realization.md");

    let error = generate_registers(&paths, &specification_output, &realization_output)
        .expect_err("staging into a file parent must fail");
    assert!(matches!(
        error,
        crate::repository::GenerateError::Publication(_)
    ));
    assert_eq!(
        fs::read(&specification_output).expect("attestation register"),
        b"previous generation",
        "no final destination may change when a later member fails staging",
    );
}

#[test]
fn register_generation_rejects_aliased_outputs() {
    // F2-005: the two registers are distinct assets; one destination
    // serving both roles must fail before derivation or writing.
    let directory =
        fixture_root("# Realization\n`sec:fixture`\nBody cite (`[A-def:model:known]`).\n");
    let paths = RepositoryCensus::discover(directory.path());
    let output = tempfile::tempdir().expect("temporary output root");
    let shared = output.path().join("register.md");

    let error = generate_registers(&paths, &shared, &shared)
        .expect_err("aliased register outputs must be rejected");
    assert!(matches!(
        error,
        crate::repository::GenerateError::AliasedOutputs(_)
    ));
    assert!(!shared.exists(), "nothing may be written on alias failure");
}

#[test]
fn duplicate_attestation_index_row_is_rejected() {
    // S6: the index anchors were stored in a set, so a repeated row
    // collapsed and still matched the body set. The documented
    // contract is one row per distinct anchor.
    let directory = fixture_root(concat!(
        "# Realization\n",
        "`sec:fixture`\n",
        "Body cite (`[A-def:model:known]`).\n",
        "## §17 Upward-citation index · `sec:realization:anchors`\n",
        "| `[A-def:model:known]` | (`sec:fixture`) |\n",
        "| `[A-def:model:known]` | (`sec:fixture`) |\n",
    ));

    let labels = RepositoryLabels::harvest_sources(&RepositoryCensus::discover(directory.path()));

    assert!(
        labels.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == LabelErrorCode::AttestationIndexStale
                && diagnostic.message.contains("repeats attestation anchor")
                && diagnostic.line == 6
        }),
        "{:#?}",
        labels.diagnostics,
    );
}

#[test]
fn repeated_body_citation_needs_only_one_index_row() {
    // The index presents the distinct anchor set, so two body cites of
    // one anchor are served by a single row. This is the case the
    // duplicate check must not break.
    let directory = fixture_root(concat!(
        "# Realization\n",
        "`sec:fixture`\n",
        "First (`[A-def:model:known]`).\n",
        "Second (`[A-def:model:known]`).\n",
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
}

#[test]
fn index_only_and_body_only_anchors_both_stale_the_index() {
    // An index row with no body cite, and a body cite with no index
    // row, are each a stale index in their own direction.
    let index_only = fixture_root(concat!(
        "# Realization\n",
        "`sec:fixture`\n",
        "## §17 Upward-citation index · `sec:realization:anchors`\n",
        "| `[A-def:model:known]` | (`sec:fixture`) |\n",
    ));
    let labels = RepositoryLabels::harvest_sources(&RepositoryCensus::discover(index_only.path()));
    assert!(
        labels
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == LabelErrorCode::AttestationIndexStale)
    );

    let body_only = fixture_root(concat!(
        "# Realization\n",
        "`sec:fixture`\n",
        "Body cite (`[A-def:model:known]`).\n",
        "## §17 Upward-citation index · `sec:realization:anchors`\n",
        "| (`sec:fixture`) |\n",
    ));
    let labels = RepositoryLabels::harvest_sources(&RepositoryCensus::discover(body_only.path()));
    assert!(
        labels
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == LabelErrorCode::AttestationIndexStale)
    );
}
