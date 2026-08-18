use std::{fs, path::Path};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use petgraph::Direction;

use crate::{
    LabelDiagnostic, LabelErrorCode, adoption, attestation, attestation_base,
    census::{CensusGroup, RepositoryCensus},
    heads,
    label::{Label, LabelParseError, LabelShape},
    latex::harvest_attestation,
    markdown::{InlineCodeContext, scan_markdown},
    model_labels_json, nearmiss,
    owner::{ImportedLabel, LabelOwner},
    participation,
    registry::{LabelMint, LabelRegistry, RegistrySet},
    repository::{
        CitationClass, CitationOrigin, LabelCitation, LabelGraphEdge, LabelGraphNode,
        RepositoryLabels, build_label_graph, generate_registers, project_label_graph,
    },
    rust_source::{RustHarvest, harvest_model},
    shape,
    source::SourceLocation,
};

#[test]
fn single_backtick_label_is_a_bare_span() {
    let scan = scan_markdown(
        Path::new("fixture.md"),
        "# Fixture · `sec:fixture:division`\n",
    );
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
        "See (`sec:fixture:division`, `sec:fixture:detail`).\n",
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
        "`sec:mint`\n(`sec:fixture:division`, `sec:fixture:detail`)\n",
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
    let scan = scan_markdown(Path::new("fixture.md"), "See (`sec:fixture:division`.\n");
    assert_eq!(scan.code_spans[0].context, InlineCodeContext::Asymmetric);
}

#[test]
fn malformed_citation_group_with_trailing_prose_is_not_bare() {
    // The close parenthesis arrives only after prose, so the exact
    // citation grammar fails. The occurrence is an attempted citation
    // and must be diagnosed rather than demoted to a mint.
    let scan = scan_markdown(
        Path::new("fixture.md"),
        "See (`sec:fixture:division`, ordinary supporting prose).\n",
    );
    assert_eq!(
        scan.code_spans[0].context,
        InlineCodeContext::MalformedGroup
    );
}

#[test]
fn malformed_citation_group_with_leading_prose_is_not_bare() {
    let scan = scan_markdown(Path::new("fixture.md"), "(see `sec:fixture:division`)\n");
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
    let scan = scan_markdown(
        Path::new("fixture.md"),
        "Aside ((`sec:fixture:division`)) here.\n",
    );
    assert_eq!(scan.code_spans[0].context, InlineCodeContext::Parenthesized);
}

#[test]
fn unrelated_parentheses_elsewhere_never_change_a_bare_mint() {
    // A parenthesis before and after the occurrence, neither adjacent
    // to it: the occurrence is an ordinary mint, and the far
    // parentheses neither promote nor demote it.
    let scan = scan_markdown(
        Path::new("fixture.md"),
        "Prose (an aside) `sec:fixture:division` more (another aside).\n",
    );
    assert_eq!(scan.code_spans[0].context, InlineCodeContext::Bare);
}

#[test]
fn a_malformed_citation_never_becomes_the_only_mint_of_a_label() {
    // The label appears exactly once in the document, inside a
    // malformed citation group. Before this rule the later close
    // parenthesis suppressed the diagnostic and the occurrence minted
    // the label, silently moving its conceptual home.
    let directory =
        fixture_root("# Realization\nSee (`sec:fixture:division`, supporting prose).\n");
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
            .any(|label| label.as_str() == "sec:fixture:division"),
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
    let scan = scan_markdown(Path::new("fixture.md"), "broken `sec:fixture:division\n");
    assert!(
        scan.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == LabelErrorCode::UnclosedInlineCode)
    );
}

#[test]
fn owner_specific_label_shapes_parse() {
    let attestation = Label::parse("def:model:classes", LabelShape::Attestation);
    let realization = Label::parse("sec:representation", LabelShape::Realization);
    let adr = Label::parse("rule:labels:decision", LabelShape::Adr);
    assert!(attestation.is_ok() && realization.is_ok() && adr.is_ok());
}

/// The ADR-019 segment amendment, both halves: an area may hyphenate
/// exactly as a name may, and a kind may not, because the kind ranges
/// over the ADR-020 registry of words.
#[test]
fn area_may_hyphenate_and_kind_may_not() {
    let hyphenated_area = Label::parse("sec:labels-index:purpose", LabelShape::Planning);
    assert!(hyphenated_area.is_ok(), "an area may hyphenate");

    let hyphenated_name = Label::parse("rule:labels:external-citation", LabelShape::Planning);
    assert!(hyphenated_name.is_ok(), "a name may hyphenate");

    let hyphenated_kind = Label::parse("sub-sec:labels:purpose", LabelShape::Planning);
    assert!(
        matches!(hyphenated_kind, Err(LabelParseError::Malformed(_))),
        "a kind may not hyphenate"
    );

    // The realization owner's two-part divisions are unaffected.
    let two_part = Label::parse("sec:representation", LabelShape::Realization);
    assert!(two_part.is_ok());
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

    let (registry, diagnostics) = harvest_attestation(&RepositoryCensus::discover(root));
    assert!(diagnostics.is_empty());
    assert!(registry.contains(
        &Label::parse("open:attestation:inner-case", LabelShape::Attestation).expect("valid label")
    ));
}

#[test]
fn duplicate_attestation_mint_diagnostic_names_both_locations() {
    // ADR-019 (F3-007): a duplicate-mint diagnostic identifies both
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
        "// x ´test:fixture:defined´ and (´test:fixture:defined´)\n// ´def:Malformed´\n",
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
        "# Realization\n`sec:fixture:division`\n",
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
        "# Realization\n`sec:fixture:division`\n",
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
    let directory = fixture_root("# Realization\n`sec:fixture:division`\n");
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
    let directory = fixture_root("# Realization\n`sec:fixture:division`\n");
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
    let directory = fixture_root("# Realization\n`sec:fixture:division`\n");
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

#[cfg(unix)]
#[test]
fn scoped_model_derivation_refuses_an_unreadable_scoped_directory() {
    // Scoped generation runs without the full repository audit, so it
    // is exactly where a suppressed traversal failure would pass.
    let directory = fixture_root("# Realization\n`sec:fixture:division`\n");
    let root = directory.path();
    let sources = root.join("packages/model/src");
    fs::write(sources.join("fixture.rs"), "// ´def:fixture:hidden´\n").expect("model source");
    if !make_unreadable(&sources) {
        return;
    }

    let declared = RepositoryCensus {
        root: root.to_path_buf(),
        ..RepositoryCensus::default()
    };
    let error = model_labels_json(&declared).expect_err("unreadable model sources must fail");

    fs::set_permissions(&sources, fs::Permissions::from_mode(0o755))
        .expect("restore model source directory");

    let crate::repository::GenerateError::Validation(diagnostics) = error else {
        panic!("expected a validation failure");
    };
    assert!(
        diagnostics.iter().any(|diagnostic| {
            diagnostic.code == LabelErrorCode::CensusUnreadable
                && diagnostic.path == "packages/model/src"
        }),
        "{diagnostics:#?}",
    );
}

#[test]
fn absent_and_empty_directories_are_equally_clean_empty_groups() {
    // Policy: a directory that is absent and a directory that is
    // present but empty both yield an empty group with no traversal
    // diagnostic. Only a directory that exists and cannot be read is a
    // failure — an empty group is knowledge, an unreadable one is not.
    let absent = fixture_root("# Realization\n`sec:fixture:division`\n");
    fs::remove_dir_all(absent.path().join("adr")).expect("remove ADR directory");
    let empty = fixture_root("# Realization\n`sec:fixture:division`\n");

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
        fixture_root("# Realization\n`sec:fixture:division`\nBody cite (`[A-def:model:known]`).\n");
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
    let directory = fixture_root("# Realization\n`sec:fixture:division`\n");
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
        "# Realization\n`sec:fixture:division`\n",
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
fn malformed_realization_imports_fail_exhaustively() {
    let directory = fixture_root(concat!(
        "# Realization\n",
        "`sec:fixture:division`\n",
        "Unknown owner (`[UNKNOWN-sec:fixture:division]`).\n",
        "Short ADR owner (`[ADR01-rule:fixture:defined]`).\n",
        "Known owner malformed label (`[ADR013-rule:Bad]`).\n",
        "Known owner valid import (`[ADR013-rule:fixture:defined]`).\n",
        "Same owner imported (`[RZ-sec:fixture:division]`).\n",
        "Bare import `[ADR013-rule:fixture:defined]`.\n",
        "Legacy model (`[rule:verification:pure-transition]`).\n",
        "Malformed model (`[rule:Bad]`).\n",
        "Asymmetric (`[ADR013-rule:fixture:defined]`.\n",
        "Example ``[UNKNOWN-sec:fixture:division]``.\n",
        "```text\n",
        "(`[UNKNOWN-sec:fixture:division]`)\n",
        "```\n",
    ));
    let root = directory.path();
    fs::write(
        root.join("adr/013-fixture.md"),
        "# ADR\n`rule:fixture:defined`\n",
    )
    .expect("ADR source");
    fs::write(
        root.join("packages/model/src/fixture.rs"),
        "// ´rule:verification:pure-transition´\n",
    )
    .expect("model source");

    let labels = RepositoryLabels::harvest_sources(&RepositoryCensus::discover(root));
    let realization_diagnostics = labels
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.path == "docs/attestation/realization.md")
        .map(|diagnostic| (diagnostic.line, diagnostic.code))
        .collect::<Vec<_>>();

    assert_eq!(
        realization_diagnostics,
        vec![
            (3, LabelErrorCode::UnknownOwner),
            (4, LabelErrorCode::UnknownOwner),
            (5, LabelErrorCode::InvalidLabel),
            (7, LabelErrorCode::InvalidImportedCitationForm),
            (8, LabelErrorCode::InvalidImportedCitationForm),
            (10, LabelErrorCode::InvalidLabel),
            (11, LabelErrorCode::AsymmetricCitation),
        ],
        "{:#?}",
        labels.diagnostics,
    );
    assert_eq!(labels.imported_citation_count(), 2);
}

#[test]
fn status_tags_are_audited_in_their_grammar_class() {
    // Each bracketed token is audited in its own grammar class: live status
    // tags resolve their pin and clause references, template examples are
    // exempted, and malformed tags are typed InvalidStatusTag diagnostics.
    let directory = fixture_root(concat!(
        "# Realization\n",                          // 1
        "`sec:fixture:division`\n",                 // 2
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
fn duplicate_realization_mint_fails() {
    let directory = fixture_root("# Realization\n`sec:fixture:division`\n`sec:fixture:division`\n");

    let labels = RepositoryLabels::harvest_sources(&RepositoryCensus::discover(directory.path()));
    let duplicate = labels
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == LabelErrorCode::DuplicateMint)
        .expect("duplicate realization mint diagnostic");
    assert_eq!(duplicate.line, 3);
    assert!(duplicate.message.contains("first minted at"));
}

#[test]
fn repository_analysis_exposes_a_direct_petgraph_graph() {
    let directory =
        fixture_root("# Realization\n`sec:fixture:division`\n(`sec:fixture:division`)\n");
    let labels = RepositoryLabels::harvest_sources(&RepositoryCensus::discover(directory.path()));
    let graph: &petgraph::graph::DiGraph<LabelGraphNode, LabelGraphEdge, u32> = &labels.graph;

    assert!(graph.node_count() > 0);
}

#[test]
fn every_authored_citation_resolves_to_one_mint_edge() {
    let directory = fixture_root(concat!(
        "# Realization\n",
        "`sec:fixture:division`\n",
        "(`sec:fixture:division`)\n",
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
    let directory = fixture_root("# Realization\n`sec:fixture:division`\n");
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

#[test]
fn label_graph_projection_ignores_citation_insertion_order() {
    let mut registry = LabelRegistry::default();
    let first = Label::parse("sec:fixture:first", LabelShape::Realization).unwrap();
    let second = Label::parse("sec:fixture:second", LabelShape::Realization).unwrap();

    for label in [first.clone(), second.clone()] {
        registry
            .insert(LabelMint {
                owner: LabelOwner::Realization,
                label,
                location: SourceLocation::new("docs/attestation/realization.md", 1, 1),
                home: None,
            })
            .unwrap();
    }

    let registries = RegistrySet {
        realization: registry,
        ..RegistrySet::default()
    };
    let citations = vec![citation_to(&first, 10), citation_to(&second, 11)];
    let mut reversed = citations.clone();
    reversed.reverse();
    let mut first_diagnostics = Vec::new();
    let mut second_diagnostics = Vec::new();
    let first = build_label_graph(&registries, citations, &mut first_diagnostics);
    let second = build_label_graph(&registries, reversed, &mut second_diagnostics);

    assert_eq!(first_diagnostics, second_diagnostics);
    assert_eq!(
        project_label_graph(&first.0),
        project_label_graph(&second.0)
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

fn citation_to(label: &Label, line: usize) -> LabelCitation {
    LabelCitation {
        source_owner: LabelOwner::Realization,
        target: ImportedLabel {
            owner: LabelOwner::Realization,
            label: label.clone(),
        },
        origin: CitationOrigin::Source {
            owner: LabelOwner::Realization,
            location: SourceLocation::new("docs/attestation/realization.md", line, 1),
        },
        class: CitationClass::AuthoredSameOwner,
    }
}

#[test]
fn attestation_anchor_set_derives_from_body_citations_only() {
    // A fenced example and the generated upward-citation index must
    // not contribute anchors; an index token without a body citation
    // is a stale index, not a member of the anchor set.
    let directory = fixture_root(concat!(
        "# Realization\n",
        "`sec:fixture:division`\n",
        "Body cite (`[A-def:model:known]`).\n",
        "```text\n",
        "[A-def:model:fenced]\n",
        "```\n",
        "## §17 Upward-citation index · `sec:realization:anchors`\n",
        "| `[A-def:model:known]` | (`sec:fixture:division`) |\n",
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
        "`sec:fixture:division`\n",
        "## §17 Upward-citation index · `sec:realization:anchors`\n",
        "| `[A-def:model:known]` | (`sec:fixture:division`) |\n",
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
    let directory = fixture_root("# Realization\n`sec:fixture:division`\n(`[A-def:Bad]`)\n");

    let labels = RepositoryLabels::harvest_sources(&RepositoryCensus::discover(directory.path()));

    assert!(labels.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == LabelErrorCode::InvalidLabel
            && diagnostic.path == "docs/attestation/realization.md"
            && diagnostic.line == 3
    }));
}

#[test]
fn unclosed_attestation_import_is_diagnostic() {
    let directory = fixture_root("# Realization\n`sec:fixture:division`\n(`[A-def:model:known]\n");

    let labels = RepositoryLabels::harvest_sources(&RepositoryCensus::discover(directory.path()));

    assert!(labels.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == LabelErrorCode::UnclosedInlineCode
            && diagnostic.path == "docs/attestation/realization.md"
            && diagnostic.line == 3
    }));
}

#[test]
fn non_parenthesized_attestation_import_is_rejected() {
    let directory = fixture_root("# Realization\n`sec:fixture:division`\n`[A-def:model:known]`\n");

    let labels = RepositoryLabels::harvest_sources(&RepositoryCensus::discover(directory.path()));

    assert!(labels.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == LabelErrorCode::InvalidImportedCitationForm
            && diagnostic.path == "docs/attestation/realization.md"
            && diagnostic.line == 3
    }));
}

#[test]
fn raw_text_attestation_token_is_nonparticipating() {
    let directory =
        fixture_root("# Realization\n`sec:fixture:division`\nRaw [A-def:model:known].\n");

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
        fixture_root("# Realization\n`sec:fixture:division`\nExample ``[A-def:model:missing]``.\n");

    let labels = RepositoryLabels::harvest_sources(&RepositoryCensus::discover(directory.path()));

    assert!(!labels.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == LabelErrorCode::UnknownImportedLabel
            && diagnostic.message.contains("def:model:missing")
    }));
}

#[test]
fn fenced_attestation_example_is_nonparticipating() {
    let directory = fixture_root(
        "# Realization\n`sec:fixture:division`\n```text\n(`[A-def:model:missing]`)\n```\n",
    );

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
        "`sec:fixture:division`\n",
        "## §17 Upward-citation index · `sec:realization:anchors`\n",
        "| `[A-def:model:known]` | (`sec:fixture:division`) |\n",
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
        "`sec:fixture:division`\n",
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
        "`sec:fixture:division`\n",
        "Body cite (`[A-def:model:known]`).\n",
        "A display-only example: ``[A-def:model:example-only]``.\n",
        "## §17 Upward-citation index · `sec:realization:anchors`\n",
        "| `[A-def:model:known]` | (`sec:fixture:division`) |\n",
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
    let directory = fixture_root("# Realization\n`sec:fixture:division`\n");
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
    let directory = fixture_root("# Realization\n`sec:fixture:division`\n");
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
    let directory = fixture_root("# Realization\n`sec:fixture:division`\n");
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
    let directory = fixture_root("# Realization\n`sec:fixture:division`\n");
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
    let directory = fixture_root("# Realization\n`sec:fixture:division`\n");
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
    let directory = fixture_root("# Realization\n`sec:fixture:division`\n");
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
    let directory = fixture_root("# Realization\n`sec:fixture:division`\n");
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
        fixture_root("# Realization\n`sec:fixture:division`\nBody cite (`[A-def:model:known]`).\n");
    let root = directory.path();
    fs::write(
        root.join("packages/model/src/fixture.rs"),
        "// ´def:Broken´\n",
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
        fixture_root("# Realization\n`sec:fixture:division`\nBody cite (`[A-def:model:known]`).\n");
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
        fixture_root("# Realization\n`sec:fixture:division`\nBody cite (`[A-def:model:known]`).\n");
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
        fixture_root("# Realization\n`sec:fixture:division`\nBody cite (`[A-def:model:known]`).\n");
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
        "`sec:fixture:division`\n",
        "Body cite (`[A-def:model:known]`).\n",
        "## §17 Upward-citation index · `sec:realization:anchors`\n",
        "| `[A-def:model:known]` | (`sec:fixture:division`) |\n",
        "| `[A-def:model:known]` | (`sec:fixture:division`) |\n",
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
        "`sec:fixture:division`\n",
        "First (`[A-def:model:known]`).\n",
        "Second (`[A-def:model:known]`).\n",
        "## §17 Upward-citation index · `sec:realization:anchors`\n",
        "| `[A-def:model:known]` | (`sec:fixture:division`) |\n",
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
        "`sec:fixture:division`\n",
        "## §17 Upward-citation index · `sec:realization:anchors`\n",
        "| `[A-def:model:known]` | (`sec:fixture:division`) |\n",
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
        "`sec:fixture:division`\n",
        "Body cite (`[A-def:model:known]`).\n",
        "## §17 Upward-citation index · `sec:realization:anchors`\n",
        "| (`sec:fixture:division`) |\n",
    ));
    let labels = RepositoryLabels::harvest_sources(&RepositoryCensus::discover(body_only.path()));
    assert!(
        labels
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == LabelErrorCode::AttestationIndexStale)
    );
}

// -------------------------------------------------------------------
// The shared participation scanner (DI-003).
// -------------------------------------------------------------------

#[test]
fn prose_participation_marks_fence_markers_and_interiors_as_displayed() {
    let participation = crate::participation::ProseParticipation::of(concat!(
        "prose\n",   // 1
        "```text\n", // 2
        "shown\n",   // 3
        "```\n",     // 4
        "prose\n",   // 5
    ));

    assert!(participation.participates(1));
    // The fence marker line is displayed too: a span cannot straddle a
    // region boundary if the boundary itself never participates.
    assert!(!participation.participates(2));
    assert!(!participation.participates(3));
    assert!(!participation.participates(4));
    assert!(participation.participates(5));
    // Off the end of the source there is no authored text to mean
    // anything.
    assert!(!participation.participates(6));
    assert!(!participation.participates(0));
    assert_eq!(participation.unclosed_fence(), None);
}

#[test]
fn prose_participation_reports_an_unclosed_fence_at_its_opening_line() {
    let participation =
        crate::participation::ProseParticipation::of("prose\n\n```text\nshown\nstill shown\n");

    assert_eq!(participation.unclosed_fence(), Some(3));
    assert!(participation.participates(1));
    assert!(!participation.participates(4));
    assert!(!participation.participates(5));
}

#[test]
fn blanking_preserves_line_numbering() {
    let source = "a\n```\nb\n```\nc\n";
    let participation = crate::participation::ProseParticipation::of(source);
    let blanked = participation.blanked(source);

    assert_eq!(blanked, "a\n\n\n\nc");
    // Every participating line keeps its original one-based number, so
    // a location derived from the blanked text still points at the
    // right line of the original.
    assert_eq!(blanked.lines().nth(4), Some("c"));
}

#[test]
fn the_line_view_and_the_span_scanner_agree_about_fenced_material() {
    // The two used to be separate loops. Any label-shaped token inside
    // a fence must be invisible to both, or a check that consults one
    // disagrees with a check that consults the other.
    let source = concat!(
        "`sec:real`\n",
        "```text\n",
        "`sec:fenced`\n",
        "```\n",
        "`sec:alsoreal`\n",
    );
    let scan = scan_markdown(Path::new("fixture.md"), source);

    let scanned = scan
        .code_spans
        .iter()
        .map(|span| span.content.clone())
        .collect::<Vec<_>>();
    assert_eq!(scanned, vec!["sec:real", "sec:alsoreal"]);

    for span in &scan.code_spans {
        assert!(
            scan.participation.participates(span.location.line),
            "the span scanner emitted a span from a line the line view calls displayed",
        );
    }
}

#[test]
fn double_backtick_spans_are_displayed_not_participating() {
    let scan = scan_markdown(Path::new("fixture.md"), "``sec:shown`` and `sec:meant`\n");

    // Both are recognized as spans; exactly one participates.
    assert_eq!(scan.code_spans.len(), 2);
    let participating = scan
        .participating_spans()
        .map(|span| span.content.clone())
        .collect::<Vec<_>>();
    assert_eq!(participating, vec!["sec:meant"]);
    assert_eq!(
        scan.code_spans[0].displayed(),
        Some(crate::participation::Displayed::DoubleBacktick),
    );
    assert_eq!(scan.code_spans[1].displayed(), None);
}

#[test]
fn a_fenced_index_heading_does_not_open_the_generated_index_region() {
    // DI-F02. The Realization harvest read its index-region
    // boundary from the raw source while reading its spans from the
    // participation view, so a displayed heading could re-partition a
    // document the span scanner had already partitioned the other way.
    //
    // Here the fenced heading used to open the index region, which then
    // stayed open across the fence close and swallowed the real body
    // citation below it. The anchor set derives from body citations, so
    // the body citation went missing and the genuine index below was
    // reported stale. Under the shared scanner the displayed heading is
    // not a heading at all.
    let directory = fixture_root(concat!(
        "# Realization\n",
        "`sec:fixture:division`\n",
        "```text\n",
        "## §17 Upward-citation index · `sec:realization:anchors`\n",
        "```\n",
        "Body cite (`[A-def:model:known]`).\n",
        "## §17 Upward-citation index · `sec:realization:anchors`\n",
        "| `[A-def:model:known]` | (`sec:fixture:division`) |\n",
    ));

    let labels = RepositoryLabels::harvest_sources(&RepositoryCensus::discover(directory.path()));

    assert!(
        !labels
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == LabelErrorCode::AttestationIndexStale),
        "the body citation below a displayed index heading must still be a body citation: {:#?}",
        labels.diagnostics,
    );
}

#[test]
fn latex_participation_strips_comments_and_respects_escaping() {
    let stripped = crate::participation::latex_participating(concat!(
        "\\label{def:kept} % dropped\n",
        "100\\% kept \\label{def:also-kept}\n",
        "% whole line dropped\n",
        "\\\\% dropped: the backslash pair escapes itself\n",
    ));

    assert!(stripped.contains("def:kept"));
    assert!(stripped.contains("def:also-kept"));
    assert!(stripped.contains("100\\%"));
    assert!(!stripped.contains("dropped"));
    // Line structure survives, so a location derived from the stripped
    // text still names the authored line.
    assert_eq!(stripped.lines().count(), 4);
}

// ---------------------------------------------------------------------
// Adoption data, kind vocabulary, and warrant totality (DI-003 W2).
// ---------------------------------------------------------------------

fn repository_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("the repository root resolves from the crate manifest")
}

fn mint_for(owner: LabelOwner, label: &str) -> LabelMint {
    let shape = owner.shape();
    LabelMint {
        owner,
        label: Label::parse(label, shape).expect("fixture label parses"),
        location: crate::source::SourceLocation::new("fixture.md", 1, 1),
        home: None,
    }
}

fn no_place(_mint: &LabelMint) -> Option<adoption::StandardPlace> {
    None
}

/// The committed registry vocabulary is pinned to the archived draft.
/// This test reads the real document, not a fixture: the point is that
/// the checker's table cannot drift from the registry it adopts.
#[test]
fn committed_registry_kinds_match_the_archived_draft() {
    let text = fs::read_to_string(repository_root().join(adoption::REGISTRY_SOURCE))
        .expect("the archived kind registry is readable");
    let parsed = adoption::parse_registry_source(&text);
    let committed: std::collections::BTreeSet<String> = adoption::REGISTRY_KINDS
        .iter()
        .map(|kind| (*kind).to_owned())
        .collect();
    assert_eq!(
        parsed, committed,
        "the committed kind table must equal the draft's Convention tables",
    );
    // The edition in force reports 208 kinds over its own generated
    // headline table, which the extraction must reproduce.
    assert_eq!(
        parsed.len(),
        208,
        "the adopted edition catalogues 208 kinds"
    );
}

/// The committed extension set is pinned to the adopting record's table.
/// ADR-020 is hand-maintained prose, so this is the drift that most
/// needs catching.
#[test]
fn committed_extension_kinds_match_the_adopting_record() {
    let text = fs::read_to_string(repository_root().join(adoption::EXTENSION_SOURCE))
        .expect("the adopting record is readable");
    let parsed = adoption::parse_extension_source(&text);
    let committed: std::collections::BTreeSet<String> = adoption::EXTENSION_KINDS
        .iter()
        .map(|kind| (*kind).to_owned())
        .collect();
    assert_eq!(
        parsed, committed,
        "the committed extension set must equal the record's extension table",
    );
    assert_eq!(parsed.len(), 14, "the record carries fourteen extensions");
}

/// No extension may collide with a registry token: the record requires
/// an extension to be distinct from every registry entry.
#[test]
fn extension_set_is_disjoint_from_the_registry() {
    let collisions: Vec<&&str> = adoption::EXTENSION_KINDS
        .iter()
        .filter(|kind| adoption::REGISTRY_KINDS.contains(kind))
        .collect();
    assert!(
        collisions.is_empty(),
        "an extension may not claim a registry token: {collisions:?}",
    );
}

/// Drift in either direction is reported, naming both sides, so that the
/// committed table can be repaired from the diagnostic alone.
#[test]
fn vocabulary_drift_fails_loudly_and_names_both_sides() {
    let directory = tempfile::tempdir().expect("temporary repository");
    let root = directory.path();
    fs::create_dir_all(root.join("plans/drafts")).expect("drafts directory");
    fs::write(
        root.join(adoption::REGISTRY_SOURCE),
        concat!(
            "**Convention (Invented)**\n\n",
            "| Environment | Kind |\n",
            "| --- | --- |\n",
            "| Invention | `invented` |\n",
        ),
    )
    .expect("doctored registry");

    let diagnostics = adoption::verify_vocabulary_sources(root);
    let drift = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == LabelErrorCode::KindVocabularyDrift)
        .expect("a doctored registry is drift");
    assert!(drift.is_error(), "drift must fail, never warn");
    assert!(
        drift.message.contains("invented"),
        "the token the document carries must be named: {}",
        drift.message,
    );
    assert!(
        drift.message.contains("sec"),
        "the tokens the checker carries alone must be named: {}",
        drift.message,
    );
}

/// The committed pair table is pinned to the archived draft, name column
/// and kind column together. The kind table alone cannot say which name
/// a kind belongs to, which is the whole of what head validation asks.
#[test]
fn committed_registry_pairs_match_the_archived_draft() {
    let text = fs::read_to_string(repository_root().join(adoption::REGISTRY_SOURCE))
        .expect("the archived kind registry is readable");
    let parsed = adoption::parse_registry_pairs(&text);
    let committed: std::collections::BTreeSet<(String, String)> = adoption::REGISTRY_PAIRS
        .iter()
        .map(|(name, kind)| ((*name).to_owned(), (*kind).to_owned()))
        .collect();
    assert_eq!(
        parsed, committed,
        "the committed pair table must equal the draft's Convention rows",
    );
    // The edition in force reports 349 rows over 333 names, which the
    // extraction must reproduce once the daggers are off the names.
    assert_eq!(parsed.len(), 349, "the adopted edition carries 349 rows");
    let names: std::collections::BTreeSet<&String> = parsed.iter().map(|(name, _)| name).collect();
    assert_eq!(names.len(), 333, "the adopted edition carries 333 names");
}

/// The committed extension pairs are pinned to the adopting record.
#[test]
fn committed_extension_pairs_match_the_adopting_record() {
    let text = fs::read_to_string(repository_root().join(adoption::EXTENSION_SOURCE))
        .expect("the adopting record is readable");
    let parsed = adoption::parse_extension_pairs(&text);
    let committed: std::collections::BTreeSet<(String, String)> = adoption::EXTENSION_PAIRS
        .iter()
        .map(|(name, kind)| ((*name).to_owned(), (*kind).to_owned()))
        .collect();
    assert_eq!(
        parsed, committed,
        "the committed extension pairs must equal the record's table",
    );
}

/// Pair drift is reported in both directions, naming the pair each side
/// carries alone — a kind that moved to another name is drift the kind
/// table cannot see, since its token set never changed.
#[test]
fn pair_drift_fails_loudly_and_names_both_sides() {
    let directory = tempfile::tempdir().expect("temporary repository");
    let root = directory.path();
    fs::create_dir_all(root.join("plans/drafts")).expect("drafts directory");
    fs::write(
        root.join(adoption::REGISTRY_SOURCE),
        concat!(
            "**Convention (Doctored)**\n\n",
            "| Environment | Kind |\n",
            "| --- | --- |\n",
            "| Theorem | `tab` |\n",
        ),
    )
    .expect("doctored registry");

    let drift = adoption::verify_vocabulary_sources(root)
        .into_iter()
        .find(|diagnostic| diagnostic.message.contains("name-and-kind pairs"))
        .expect("a doctored pair table is drift");
    assert!(drift.is_error(), "drift must fail, never warn");
    assert_eq!(drift.code, LabelErrorCode::KindVocabularyDrift);
    assert!(
        drift.message.contains("Theorem = tab"),
        "the pair the document carries must be named: {}",
        drift.message,
    );
    assert!(
        drift.message.contains("Theorem = thm"),
        "the pair the checker carries alone must be named: {}",
        drift.message,
    );
}

/// An absent source is not drift: the synthetic fixture repositories of
/// this suite carry neither document, and their absence must not be read
/// as a vocabulary that shrank to nothing.
#[test]
fn absent_vocabulary_sources_are_not_drift() {
    let directory = tempfile::tempdir().expect("temporary repository");
    assert!(
        adoption::verify_vocabulary_sources(directory.path()).is_empty(),
        "a tree without the adopted documents reports no drift",
    );
}

/// A kind in neither source fails, in an owner the registry governs, and
/// the message names both sources so the reader knows where to look.
#[test]
fn unknown_kind_fails_in_a_governed_owner() {
    let adoption_data = adoption::Adoption::repository();
    let mints = vec![mint_for(LabelOwner::Plan, "notakind:area:name")];
    let diagnostics = adoption::validate_warrants(&adoption_data, &mints, &no_place);
    let unknown = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == LabelErrorCode::UnknownKind)
        .expect("an uncatalogued kind fails");
    assert!(unknown.is_error());
    assert!(unknown.message.contains(adoption::REGISTRY_SOURCE));
    assert!(unknown.message.contains(adoption::EXTENSION_SOURCE));
}

/// Both halves of the effective relation admit a mint: the registry's
/// own tokens, and the acceptee's recorded extensions.
#[test]
fn registry_and_extension_kinds_both_pass() {
    let adoption_data = adoption::Adoption::repository();
    let mints = vec![
        mint_for(LabelOwner::Plan, "sec:area:name"),
        mint_for(LabelOwner::Plan, "trap:area:name"),
        mint_for(LabelOwner::Doc, "postc:area:name"),
    ];
    let diagnostics = adoption::validate_warrants(&adoption_data, &mints, &no_place);
    assert!(
        diagnostics.is_empty(),
        "a registry kind and a recorded extension both stand: {diagnostics:#?}",
    );
}

/// The attestation LaTeX surface is in scope since its last unregistered
/// token was adjudicated, so an uncatalogued token there fails exactly
/// as it does under every other owner.
#[test]
fn attestation_unknown_kind_fails_like_any_other_owner() {
    let adoption_data = adoption::Adoption::repository();
    let mints = vec![mint_for(
        LabelOwner::Attestation,
        "abs:attestation:somewhere",
    )];
    let diagnostics = adoption::validate_warrants(&adoption_data, &mints, &no_place);
    assert_eq!(diagnostics.len(), 1);
    assert!(
        diagnostics[0].is_error(),
        "the attestation surface is enforced, not reported",
    );
    assert_eq!(diagnostics[0].code, LabelErrorCode::UnknownKind);
}

/// The kind the migration adjudicated: the paper's abstract is minted
/// under the registry's own `abst`, which the checker admits.
#[test]
fn attestation_abstract_kind_is_registered() {
    let adoption_data = adoption::Adoption::repository();
    let mints = vec![mint_for(
        LabelOwner::Attestation,
        "abst:attestation:abstract",
    )];
    let diagnostics = adoption::validate_warrants(&adoption_data, &mints, &no_place);
    assert!(
        diagnostics.is_empty(),
        "abst is a registry token: {diagnostics:#?}",
    );
}

/// A reserved kind no profile governs admits neither warrant rule, so
/// its bare occurrence is a hard failure. The repository's reserved set
/// is empty, so the enforcement is exercised on test-only adoption data
/// -- which is the point: it must already be live when the set grows.
#[test]
fn reserved_kind_without_a_profile_fails() {
    let mut adoption_data = adoption::Adoption::repository();
    adoption_data.reserved_kinds.insert("suite".to_owned());
    let mints = vec![mint_for(LabelOwner::Model, "suite:area:name")];
    let diagnostics = adoption::validate_warrants(&adoption_data, &mints, &no_place);
    let reserved = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == LabelErrorCode::ReservedKindWithoutProfile)
        .expect("a reserved kind without a profile fails");
    assert!(reserved.is_error());
    assert_eq!(
        adoption_data.ungoverned_reserved_kinds(),
        vec!["suite"],
        "the ungoverned reserved kinds are nameable before any mint is read",
    );
}

/// An inventory-kind token away from its profile's standard place
/// warrants nothing. Vacuous in this repository, which registers no
/// profile, and exercised here on a test-only profile.
#[test]
fn inventory_kind_away_from_the_standard_place_fails() {
    let profile = adoption::Profile {
        kind: "test".to_owned(),
        census: "the cases the harness recognizes".to_owned(),
        classification: "the case's suite".to_owned(),
        name_transformation: "hyphenation of the function identifier".to_owned(),
        standard_place: adoption::StandardPlace::DocumentationComment,
    };
    let adoption_data = adoption::Adoption {
        profiles: vec![profile],
        reserved_kinds: std::iter::once("test".to_owned()).collect(),
    };
    let mints = vec![mint_for(LabelOwner::Model, "test:integration:roundtrip")];

    let diagnostics = adoption::validate_warrants(&adoption_data, &mints, &no_place);
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == LabelErrorCode::InventoryKindOutOfPlace),
        "a derived label away from its standard place is no mint: {diagnostics:#?}",
    );

    // At the standard place the same occurrence stands.
    let at_place = |_mint: &LabelMint| Some(adoption::StandardPlace::DocumentationComment);
    assert!(
        adoption::validate_warrants(&adoption_data, &mints, &at_place).is_empty(),
        "an occurrence at the standard place is warranted by derivation",
    );
}

/// A prefix outside the signature names no owner, so its citation
/// cannot resolve. The registered package prefixes do name owners.
#[test]
fn only_registered_prefixes_name_owners() {
    assert!(
        ImportedLabel::parse("NOTREGISTERED-def:area:name").is_err(),
        "a prefix outside the signature has no owner",
    );
    let imported =
        ImportedLabel::parse("LABELS-def:area:name").expect("a registered package prefix resolves");
    assert_eq!(imported.owner, LabelOwner::Crate("labels".to_owned()));
    assert_eq!(
        adoption::owner_for_prefix("ADR019"),
        Some(LabelOwner::Adr(19)),
        "the numbered-record family derives its prefixes",
    );
    assert_eq!(
        adoption::owner_for_prefix("ADR19"),
        None,
        "the family fixes three digits",
    );
}

/// The package registration is answerable to the census: an unregistered
/// package and a stale registration are both recorded decisions the tree
/// has not taken.
#[test]
fn package_registration_is_checked_against_the_census() {
    let location = crate::source::SourceLocation::new("packages", 1, 1);
    let registered: Vec<&str> = adoption::PACKAGE_OWNERS
        .iter()
        .map(|(_, package)| *package)
        .collect();
    assert!(
        adoption::verify_package_registration(registered.iter().copied(), &location).is_empty(),
        "the registration agrees with itself",
    );

    let with_newcomer: Vec<&str> = registered
        .iter()
        .copied()
        .chain(std::iter::once("newcomer"))
        .collect();
    let diagnostics = adoption::verify_package_registration(with_newcomer, &location);
    assert!(
        diagnostics.iter().any(|diagnostic| {
            diagnostic.code == LabelErrorCode::UnregisteredOwner
                && diagnostic.message.contains("newcomer")
        }),
        "an unregistered package is named: {diagnostics:#?}",
    );

    let missing: Vec<&str> = registered.iter().copied().skip(1).collect();
    let diagnostics = adoption::verify_package_registration(missing, &location);
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == LabelErrorCode::UnregisteredOwner),
        "a registration naming no package is stale: {diagnostics:#?}",
    );
}

/// The prefix derivation rule is injective over the registered packages:
/// dropping hyphens could collide two names, and a collision would make
/// one owner unciteable.
#[test]
fn derived_package_prefixes_are_distinct() {
    let mut seen = std::collections::BTreeMap::new();
    for (prefix, package) in adoption::PACKAGE_OWNERS {
        assert_eq!(&adoption::derive_package_prefix(package), prefix);
        assert!(
            seen.insert(*prefix, *package).is_none(),
            "two packages derive the prefix {prefix}",
        );
        assert!(
            adoption::owner_for_prefix(prefix).is_some(),
            "a registered package prefix resolves through the signature",
        );
    }
}

/// The owner partition is total on the carrier locations the census
/// walks, and the specific rule precedes the general one.
#[test]
fn partition_rules_are_ordered_specific_before_general() {
    let rules = adoption::partition();
    let model = rules
        .iter()
        .position(|rule| rule.path == "packages/model/src")
        .expect("the model crate has a rule");
    let packages = rules
        .iter()
        .position(|rule| rule.path == "packages")
        .expect("the package family has a rule");
    assert!(
        model < packages,
        "the model crate must be matched before the package family",
    );
}

/// The live tree is conformant: every kind minted in an owner the
/// registry governs lies in the effective relation. The attestation surface
/// is outside that scope and reports its two unadjudicated tokens.
#[test]
fn the_corpus_mints_no_ungoverned_kind() {
    let census = RepositoryCensus::discover(repository_root());
    let labels = RepositoryLabels::harvest_sources(&census);
    let offending: Vec<&LabelDiagnostic> = labels
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == LabelErrorCode::UnknownKind)
        .collect();
    assert!(
        offending.iter().all(|diagnostic| !diagnostic.is_error()),
        "no governed owner may mint an uncatalogued kind: {offending:#?}",
    );
}

// ---------------------------------------------------------------------
// The companion attestation register.
// ---------------------------------------------------------------------

/// The base relation derived here is the one the registry states of
/// itself. The draft publishes its own headline counts, so the parse is
/// welded to them: a change to either side that is not a change to both
/// fails here rather than silently re-deriving the register.
#[test]
fn derived_base_relation_matches_the_registry_headline_counts() {
    let base = attestation_fixture();
    let rows = base
        .records
        .iter()
        .filter(|record| record.key.source == attestation::Source::Base)
        .collect::<Vec<_>>();
    let names = rows
        .iter()
        .map(|record| record.key.name.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    let kinds = rows
        .iter()
        .map(|record| record.key.kind.as_str())
        .collect::<std::collections::BTreeSet<_>>();

    assert_eq!(rows.len(), 349, "rows");
    assert_eq!(names.len(), 333, "names");
    assert_eq!(kinds.len(), 208, "kinds");
}

/// The recorded extension rows carry everything an acceptee's own
/// evidence record must: the pair, an exact quoted spelling, a locator,
/// and the sense.
#[test]
fn recorded_extensions_carry_first_hand_evidence() {
    let base = attestation_fixture();
    let extensions = base.extensions().collect::<Vec<_>>();
    assert_eq!(extensions.len(), 14);
    for record in extensions {
        assert_eq!(record.status, attestation::Status::Firm);
        assert!(
            record.spelling.as_deref().is_some_and(|s| s.contains(':')),
            "{record:#?} needs a quoted spelling",
        );
        assert!(
            record.key.locator.contains(':'),
            "{record:#?} needs a locator",
        );
        assert!(record.sense.is_some(), "{record:#?} needs a sense");
    }
}

/// Homonymy is derived from the effective relation, not from the
/// registry alone: a recorded extension that shares a base row's name
/// makes that name homonymous in this corpus and nowhere else.
#[test]
fn homonymy_is_derived_from_the_effective_relation() {
    let base = attestation_fixture();
    let homonyms = base.homonyms();
    let names = homonyms
        .iter()
        .map(|record| record.key.name.as_str())
        .collect::<std::collections::BTreeSet<_>>();

    assert_eq!(homonyms.len(), 34);
    assert_eq!(names.len(), 16);
    // The extension row is what puts Task's third sense in Hom.
    let task = homonyms
        .iter()
        .filter(|record| record.key.name == "Task")
        .map(|record| (record.key.kind.as_str(), record.key.source))
        .collect::<Vec<_>>();
    assert_eq!(
        task,
        vec![
            ("exer", attestation::Source::Base),
            ("job", attestation::Source::Base),
            ("task", attestation::Source::Extension),
        ],
    );

    // Motto is the deliberate case: the recorded deviation sets this
    // corpus's own kind beside the registry's under one name.
    let motto = homonyms
        .iter()
        .filter(|record| record.key.name == "Motto")
        .map(|record| (record.key.kind.as_str(), record.key.source))
        .collect::<Vec<_>>();
    assert_eq!(
        motto,
        vec![
            ("motto", attestation::Source::Extension),
            ("slogan", attestation::Source::Base),
        ],
    );
}

/// The register is totally ordered by name, kind, source, locator, then
/// the record's sequence in its source table — and the order is the
/// derived key's own, so no two records tie.
#[test]
fn register_records_are_totally_ordered() {
    let base = attestation_fixture();
    let keys = base
        .records
        .iter()
        .map(|record| record.key.clone())
        .collect::<Vec<_>>();

    let mut sorted = keys.clone();
    sorted.sort();
    assert_eq!(keys, sorted, "records must be emitted in register order");

    let distinct = keys.iter().collect::<std::collections::BTreeSet<_>>();
    assert_eq!(distinct.len(), keys.len(), "the ordering must be total");
}

/// Generation is deterministic: the same sources yield the same bytes,
/// and the committed publication is those bytes.
#[test]
fn attestation_register_generation_is_deterministic() {
    let census = RepositoryCensus::discover(repository_root());
    let first = attestation_base(&census).expect("the corpus derives a register");
    let second = attestation_base(&census).expect("the corpus derives a register");
    let rendered = crate::render::attestation_register(&first);

    assert_eq!(
        rendered,
        crate::render::attestation_register(&second),
        "two derivations must render the same bytes",
    );
    let committed =
        fs::read_to_string(&census.attestation_register).expect("the register is committed");
    assert_eq!(
        committed, rendered,
        "the committed register must be the generator's current output",
    );
}

/// The register presents its rows and nothing else: every token it
/// copies is displayed, so a register participates in nothing it
/// indexes and can never sustain its own membership.
#[test]
fn the_attestation_register_participates_in_nothing() {
    let census = RepositoryCensus::discover(repository_root());
    let source = fs::read_to_string(&census.attestation_register).expect("the register is read");
    let scan = scan_markdown(Path::new("plans/labels/attestation.md"), &source);

    let minting = scan
        .participating_spans()
        .filter(|span| Label::parse(span.content.trim(), LabelShape::Planning).is_ok())
        .collect::<Vec<_>>();
    assert!(minting.is_empty(), "{minting:#?}");
}

/// Every recorded extension is firm on first-hand evidence because the
/// adopting record says so. The statement is welded here: if the record
/// stops saying it, the generator's status column has lost its warrant.
#[test]
fn the_adopting_record_still_states_that_extensions_are_firm() {
    let source = fs::read_to_string(repository_root().join(adoption::EXTENSION_SOURCE))
        .expect("the adopting record is read");
    assert!(
        source.contains("Every pair of X_A is firm on the evidence located above."),
        "the adopting record no longer states the extension statuses",
    );
}

fn attestation_fixture() -> attestation::AttestationBase {
    attestation::derive(&repository_root(), std::collections::BTreeMap::new())
        .expect("the corpus derives an attestation base")
}

// ---------------------------------------------------------------------
// Near-miss warnings.
// ---------------------------------------------------------------------

/// The three prose families, each classified as itself.
#[test]
fn prose_near_miss_families_are_recognized() {
    assert_eq!(
        nearmiss::classify("Def:Labels:Total-Resolution"),
        Some(nearmiss::NearMiss::Casing),
    );
    assert_eq!(
        nearmiss::classify("def: labels: total-resolution"),
        Some(nearmiss::NearMiss::Spacing),
    );
    assert_eq!(
        nearmiss::classify("(def:labels:total-resolution)"),
        Some(nearmiss::NearMiss::Brackets),
    );
    assert_eq!(
        nearmiss::classify("{[RZ-sec:realization:representation]}"),
        Some(nearmiss::NearMiss::Brackets),
    );
}

/// A near miss is never an occurrence, and ordinary text is never a
/// near miss. The repairs are what separate the two: a form the grammar
/// accepts is unchanged by every repair, so it can never pass a
/// repaired test.
#[test]
fn occurrences_and_ordinary_text_are_not_near_misses() {
    for content in [
        // Occurrences of every shape the grammar accepts.
        "def:labels:total-resolution",
        "  def:labels:total-resolution  ",
        "[A-def:model:classes]",
        "[RZ-sec:realization:representation]",
        // Text: colon-bearing spans whose first segment names no kind.
        "12:30:45",
        "Self::Bare",
        "note: see below",
        "UTF-8",
        "--stamp",
        // A three-segment shape whose kind is outside the vocabulary.
        "zzz:labels:total-resolution",
    ] {
        assert_eq!(
            nearmiss::classify(content),
            None,
            "{content:?} must not warn"
        );
    }
}

/// Displayed spans are silent: a near miss inside a fence, or in a
/// double-backtick span, is shown rather than meant.
#[test]
fn displayed_near_misses_are_silent() {
    let scan = scan_markdown(
        Path::new("fixture.md"),
        concat!(
            "``Def:Labels:Shown``\n",
            "```text\n",
            "Def:Labels:Fenced\n",
            "```\n",
            "and `Def:Labels:Meant`\n",
        ),
    );
    let mut diagnostics = Vec::new();
    nearmiss::prose(&scan, &mut diagnostics);

    assert_eq!(diagnostics.len(), 1, "{diagnostics:#?}");
    assert_eq!(diagnostics[0].line, 5);
    assert_eq!(diagnostics[0].code, LabelErrorCode::NearMissSpan);
    assert!(!diagnostics[0].is_error());
}

/// In scanned comment text the acute carries the label syntax, so a
/// label-shaped backtick span is a near miss. A fenced documentation
/// example is displayed, and a string literal is not comment text; both
/// stay silent.
#[test]
fn comment_backtick_spans_warn_where_the_acute_was_meant() {
    let directory = tempfile::tempdir().expect("temporary repository");
    let root = directory.path();
    fs::create_dir_all(root.join("packages/model/src")).expect("fixture directory");
    let path = root.join("packages/model/src/lib.rs");
    fs::write(
        &path,
        concat!(
            "//! A citation written with the wrong delimiter:\n",
            "//! (`rem:overview:status-tags`).\n",
            "/// ```text\n",
            "/// `rem:overview:shown`\n",
            "/// ```\n",
            "pub const VALUE: &str = \"`rem:overview:literal`\";\n",
        ),
    )
    .expect("fixture source");

    let census = RepositoryCensus {
        root: root.to_path_buf(),
        model_sources: vec![path],
        ..RepositoryCensus::default()
    };
    let harvest = harvest_model(&census);
    let warnings: Vec<&LabelDiagnostic> = harvest
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == LabelErrorCode::NearMissSpan)
        .collect();

    assert_eq!(warnings.len(), 1, "{:#?}", harvest.diagnostics);
    assert_eq!(warnings[0].line, 2);
    assert!(!warnings[0].is_error());
}

// The acute span model: in scanned code text the acute belongs to the
// label syntax and classifies locally. It opens exactly when
// label-shaped text follows it; an opening acute unclosed when its
// region ends is a hard failure; an acute that opens nothing is text.

/// All three occurrence forms parse in the acute syntax, feeding the
/// same registry and citation list the prose syntax feeds.
#[test]
fn the_three_acute_forms_mint_and_cite() {
    let harvest = rust_fixture_harvest(concat!(
        "// \u{b4}def:fixture:minted\u{b4}\n",
        "// see (\u{b4}def:fixture:minted\u{b4})\n",
        "// and (\u{b4}[RZ-sec:realization:overview]\u{b4})\n",
    ));

    assert!(
        harvest
            .registry
            .contains(&Label::parse("def:fixture:minted", LabelShape::Model).expect("valid label")),
        "{:#?}",
        harvest.diagnostics,
    );
    assert_eq!(harvest.citations.len(), 2, "{:#?}", harvest.citations);
    assert!(harvest.diagnostics.is_empty(), "{:#?}", harvest.diagnostics);
}

/// An opening acute whose region ends before it closes is a hard
/// failure, reported at the opening delimiter.
#[test]
fn an_unclosed_opening_acute_fails_at_its_delimiter() {
    let harvest = rust_fixture_harvest(concat!(
        "// intact \u{b4}def:fixture:closed\u{b4}\n",
        "// broken \u{b4}def:fixture:dangling\n",
    ));

    let unclosed: Vec<_> = harvest
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == LabelErrorCode::UnclosedInlineCode)
        .collect();
    assert_eq!(unclosed.len(), 1, "{:#?}", harvest.diagnostics);
    assert_eq!(unclosed[0].line, 2);
    assert_eq!(unclosed[0].column, 11);
    assert!(unclosed[0].is_error());
    // The label the opener declared is not minted: an opening acute is
    // an intent that failed, never a silent mint.
    assert!(
        !harvest.registry.contains(
            &Label::parse("def:fixture:dangling", LabelShape::Model).expect("valid label")
        )
    );
}

/// An acute that opens nothing is ordinary text. A lone one is an
/// apostrophe accident and a pair of them must not swallow the prose
/// between into a label.
#[test]
fn acutes_that_open_nothing_are_silent_text() {
    let harvest = rust_fixture_harvest(concat!(
        "// it\u{b4}s a plain remark\n",
        "// don\u{b4}t let this and it\u{b4}s partner pair up\n",
        "// \u{b4}def:fixture:real\u{b4} still mints\n",
    ));

    assert!(
        harvest
            .registry
            .contains(&Label::parse("def:fixture:real", LabelShape::Model).expect("valid label")),
        "{:#?}",
        harvest.diagnostics,
    );
    assert!(harvest.diagnostics.is_empty(), "{:#?}", harvest.diagnostics);
}

/// A label-shaped interior opens even when it resolves to nothing, so a
/// misspelled kind is still diagnosed rather than demoted to text.
#[test]
fn a_label_shaped_interior_opens_and_is_diagnosed() {
    let harvest = rust_fixture_harvest("// \u{b4}nosuch:fixture:kind\u{b4}\n");

    assert!(
        harvest
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == LabelErrorCode::InvalidLabel),
        "{:#?}",
        harvest.diagnostics,
    );
}

/// A candidate interior that crosses a line break is not label-shaped,
/// so the acute opens nothing: two lines never fabricate one label.
#[test]
fn an_acute_span_never_crosses_a_line_break() {
    let harvest = rust_fixture_harvest(concat!(
        "// \u{b4}def:fixture:split\n",
        "// tail\u{b4} of the region\n",
    ));

    assert!(
        !harvest
            .registry
            .contains(&Label::parse("def:fixture:split", LabelShape::Model).expect("valid label"))
    );
    // The candidate interior runs to the next acute and so carries the
    // line break: it is not label-shaped, the acute opens nothing, and
    // the whole thing is text. Neither acute is an unclosed opener.
    assert!(harvest.diagnostics.is_empty(), "{:#?}", harvest.diagnostics);
}

/// String literals and fenced documentation examples are not scanned
/// code text, so an acute occurrence in either is displayed.
#[test]
fn acutes_outside_scanned_comment_text_are_displayed() {
    let harvest = rust_fixture_harvest(concat!(
        "/// ```text\n",
        "/// \u{b4}def:fixture:fenced\u{b4}\n",
        "/// ```\n",
        "pub const V: &str = \"\u{b4}def:fixture:literal\u{b4}\";\n",
        "// \u{b4}def:fixture:scanned\u{b4}\n",
    ));

    let minted: Vec<_> = harvest.registry.labels().map(ToString::to_string).collect();
    assert_eq!(
        minted,
        vec!["def:fixture:scanned".to_owned()],
        "{:#?}",
        harvest.diagnostics,
    );
    assert!(harvest.diagnostics.is_empty(), "{:#?}", harvest.diagnostics);
}

/// Both concrete syntaxes may appear in one comment region during the
/// migration: the acute occurrence is the participating one, and the
/// backtick spelling of the same label is a near-miss warning that
/// mints nothing and fails nothing.
#[test]
fn a_mixed_syntax_region_harvests_the_acute_and_warns_on_the_backtick() {
    let harvest = rust_fixture_harvest(concat!(
        "//! \u{b4}def:fixture:shared\u{b4}\n",
        "//! cited here (\u{b4}def:fixture:shared\u{b4})\n",
        "//! and mis-delimited here `def:fixture:shared`\n",
    ));

    assert!(
        harvest
            .registry
            .contains(&Label::parse("def:fixture:shared", LabelShape::Model).expect("valid label")),
        "{:#?}",
        harvest.diagnostics,
    );
    assert_eq!(harvest.citations.len(), 1, "{:#?}", harvest.citations);
    let warnings: Vec<_> = harvest
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == LabelErrorCode::NearMissSpan)
        .collect();
    assert_eq!(warnings.len(), 1, "{:#?}", harvest.diagnostics);
    assert_eq!(warnings[0].line, 3);
    assert!(!warnings[0].is_error());
    assert!(
        harvest
            .diagnostics
            .iter()
            .all(|diagnostic| !diagnostic.is_error()),
        "{:#?}",
        harvest.diagnostics,
    );
}

/// The opening test is a lexical silhouette, not a resolution.
#[test]
fn label_shaped_text_admits_labels_and_imports_only() {
    assert!(participation::label_shaped_text("def:fixture:name"));
    assert!(participation::label_shaped_text(
        "[RZ-sec:realization:overview]"
    ));
    assert!(participation::label_shaped_text("nosuch:fixture:kind"));
    assert!(!participation::label_shaped_text(""));
    assert!(!participation::label_shaped_text("t"));
    assert!(!participation::label_shaped_text("s a plain remark"));
    assert!(!participation::label_shaped_text("def:fixture with space"));
    assert!(!participation::label_shaped_text("no-colon-here"));
    assert!(!participation::label_shaped_text("def:fixture:name\nnext"));
}

/// The live tree's near misses are warnings, every one of them.
#[test]
fn live_tree_near_misses_never_fail_a_check() {
    let census = RepositoryCensus::discover(repository_root());
    let labels = RepositoryLabels::harvest_sources(&census);
    assert!(
        labels
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.code == LabelErrorCode::NearMissSpan)
            .all(|diagnostic| !diagnostic.is_error()),
        "a near miss must never fail a check",
    );
}

// ---------------------------------------------------------------------
// Head validation: a head's name against the kind its label declares.
// ---------------------------------------------------------------------

/// A head whose name and kind are an exact row of the registry.
#[test]
fn catalogued_head_pair_validates() {
    assert!(heads::head_validates("Rule", "rule"));
    assert!(heads::head_validates("Definition", "def"));
}

/// A pair of the acceptee's own recorded extension set validates exactly
/// as a registry row does: the judgment consults the effective relation,
/// not the registry alone.
#[test]
fn extension_pair_validates_and_a_near_neighbour_does_not() {
    assert!(heads::head_validates("Trap", "trap"));
    assert!(heads::head_validates("Obligation", "obl"));
    // Pitfall is `warn` and Trap is `trap`; the record turns on their
    // being different genres, so the crossed pairs must both fail.
    assert!(!heads::head_validates("Trap", "warn"));
    assert!(!heads::head_validates("Pitfall", "trap"));
}

/// A name carrying several catalogued senses validates under each of
/// them and under no other: the kind token is the author's declaration
/// of which sense is meant, and validating it is the point.
#[test]
fn homonym_validates_only_under_its_catalogued_senses() {
    assert!(heads::head_validates("Test", "test"));
    assert!(heads::head_validates("Test", "quiz"));
    assert!(!heads::head_validates("Test", "tab"));
}

/// The iterated sub- prefix is presentation: a subsection is a section
/// nested, in either spelling and to any depth.
#[test]
fn sub_prefix_reduces_to_its_base() {
    assert!(heads::head_validates("Subsection", "sec"));
    assert!(heads::head_validates("Sub-section", "sec"));
    assert!(heads::head_validates("Subsubsection", "sec"));
}

/// The catalogued emphasis modifiers are presentation, and classify by
/// their base — but only as whole words, so a name that merely begins
/// with a modifier's letters reduces to nothing.
#[test]
fn catalogued_modifier_reduces_to_its_base() {
    assert!(heads::head_validates("Main theorem", "thm"));
    assert!(heads::head_validates("Key lemma", "lem"));
    assert!(heads::head_validates("Running example", "ex"));
    assert!(!heads::head_validates("Mainstay", "thm"));
}

/// An expressly catalogued overriding row takes precedence over the
/// modifier rule: Working hypothesis reduces to itself and carries the
/// assumptive kind, so the conjectural kind of its base fails.
#[test]
fn overriding_row_beats_the_modifier_rule() {
    assert!(heads::head_validates("Working hypothesis", "assum"));
    assert!(heads::head_validates("Standing hypothesis", "assum"));
    assert!(!heads::head_validates("Working hypothesis", "hyp"));
    // The base itself is untouched by the override.
    assert!(heads::head_validates("Hypothesis", "hyp"));
}

/// A head the catalogue does not pair fails, naming the head, the label,
/// and what the catalogue does carry for the name, so the author can see
/// which side is wrong without opening the registry.
#[test]
fn uncatalogued_head_pair_fails_and_names_the_senses() {
    let source = "**Table (The census)** · `rem:fixture:census`\n";
    let path = Path::new("plans/fixture.md");
    let scan = scan_markdown(path, source);
    let diagnostics = heads::validate_document(path, source, &scan);
    assert_eq!(diagnostics.len(), 1, "one head, one judgment");
    let failure = &diagnostics[0];
    assert!(failure.is_error());
    assert_eq!(failure.code, LabelErrorCode::UncataloguedHeadPair);
    assert_eq!(failure.line, 1);
    assert!(
        failure.message.contains("Table (The census)"),
        "the head is named: {}",
        failure.message,
    );
    assert!(
        failure.message.contains("rem:fixture:census"),
        "the label is named: {}",
        failure.message,
    );
    assert!(
        failure
            .message
            .contains("the catalogued senses of Table are tab"),
        "the name's senses are named: {}",
        failure.message,
    );
}

/// A name in no catalogue row says so, rather than reporting an empty
/// list of senses as though the name were known.
#[test]
fn uncatalogued_head_name_says_the_name_is_unknown() {
    let source = "**Widget (A widget)** · `rem:fixture:widget`\n";
    let path = Path::new("plans/fixture.md");
    let scan = scan_markdown(path, source);
    let diagnostics = heads::validate_document(path, source, &scan);
    assert_eq!(diagnostics.len(), 1);
    assert!(
        diagnostics[0].message.contains("no catalogue row"),
        "an unknown name is reported as unknown: {}",
        diagnostics[0].message,
    );
}

/// Recognition is narrow: bold prose emphasis is not a head, and forms
/// no judgment. A false rejection here would block an author writing
/// ordinary prose, which is the failure this check must never have.
#[test]
fn bold_prose_is_not_a_head() {
    assert!(heads::parse_head("**Round two.** The swap widened it.\n").is_none());
    assert!(heads::parse_head("**Rows that left the set.** Four tokens.\n").is_none());
    assert!(heads::parse_head("**Status:** Decided\n").is_none());
    assert!(heads::parse_head("**Rule (Normative source)** without a label\n").is_none());
    assert!(heads::parse_head("Text before **Rule (X)** · `rule:a:b`\n").is_none());
    assert!(heads::parse_head("**Rule** · `rule:a:b`\n").is_none());
}

/// A head is recognized with its genre and its declared kind apart.
#[test]
fn head_recognition_splits_genre_from_kind() {
    let head = heads::parse_head("**Definition (Presentation)** · `def:kinds:presentation`\n")
        .expect("a well-formed head is recognized");
    assert_eq!(head.genre, "Definition");
    assert_eq!(head.title, "Presentation");
    assert_eq!(head.kind, "def");
    assert_eq!(head.label, "def:kinds:presentation");
}

/// A head inside a fenced block is an example, not a head: the same
/// participation view every other Markdown rule consults says so.
#[test]
fn head_inside_a_fence_forms_no_judgment() {
    let source = concat!(
        "Example:\n\n",
        "```markdown\n",
        "**Table (The census)** · `rem:fixture:census`\n",
        "```\n",
    );
    let path = Path::new("plans/fixture.md");
    let scan = scan_markdown(path, source);
    assert!(
        heads::validate_document(path, source, &scan).is_empty(),
        "a fenced example forms no head judgment",
    );
}

/// Every head of the live governed tree validates, and the check is
/// live: the corpus this repository authors carries no uncatalogued
/// pair, and a future one fails the gate rather than passing unseen.
#[test]
fn live_tree_carries_no_uncatalogued_head_pair() {
    let census = RepositoryCensus::discover(repository_root());
    let labels = RepositoryLabels::harvest_sources(&census);
    let failures: Vec<&LabelDiagnostic> = labels
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == LabelErrorCode::UncataloguedHeadPair)
        .collect();
    assert!(
        failures.is_empty(),
        "the governed tree carries an uncatalogued head pair: {failures:?}",
    );
}

// The three-part shape rule (the `shape` module): a label-intended
// occurrence that is not three-part fails, wherever it is written, and
// the near-miss warnings never double-report what it fails.

/// Harvest one attestation LaTeX fixture body.
fn latex_fixture_harvest(body: &str) -> (LabelRegistry, Vec<LabelDiagnostic>) {
    let directory = tempfile::tempdir().expect("temporary repository");
    let root = directory.path();
    fs::create_dir_all(root.join("papers/attestation/sections"))
        .expect("attestation sections directory");
    fs::write(root.join("papers/attestation/main.tex"), body).expect("attestation source");
    harvest_attestation(&RepositoryCensus::discover(root))
}

/// The gap this rule closes: the LaTeX surface accepted a two-segment
/// label for years, and now fails it with its location and the form a
/// label takes.
#[test]
fn a_two_segment_latex_label_fails_with_the_expected_form() {
    let (registry, diagnostics) = latex_fixture_harvest("% front\n\\label{def:classes}\n");
    assert_eq!(diagnostics.len(), 1, "{diagnostics:#?}");
    assert_eq!(diagnostics[0].code, LabelErrorCode::MalformedLabelShape);
    assert!(diagnostics[0].is_error());
    assert_eq!(diagnostics[0].path, "papers/attestation/main.tex");
    assert_eq!(diagnostics[0].line, 2);
    assert!(
        diagnostics[0].message.contains(shape::EXPECTED_FORM),
        "the diagnostic names the expected form: {}",
        diagnostics[0].message,
    );
    assert_eq!(registry.len(), 0, "a malformed label mints nothing");
}

/// Arity is exact, not a floor: a fourth segment fails like a missing
/// one.
#[test]
fn a_four_segment_latex_label_fails() {
    let (_registry, diagnostics) = latex_fixture_harvest("\\label{def:model:classes:extra}\n");
    assert_eq!(diagnostics.len(), 1, "{diagnostics:#?}");
    assert_eq!(diagnostics[0].code, LabelErrorCode::MalformedLabelShape);
}

/// The form the corpus writes passes, so the rule rejects nothing an
/// author writes correctly.
#[test]
fn a_three_part_latex_label_passes() {
    let (registry, diagnostics) = latex_fixture_harvest("\\label{def:model:classes}\n");
    assert!(diagnostics.is_empty(), "{diagnostics:#?}");
    assert_eq!(registry.len(), 1);
}

/// A macro-generated label is checked at the arity of what it
/// synthesizes: the optional-argument form prefixes `open:attestation`, so an
/// argument carrying its own colon overshoots the form.
#[test]
fn a_macro_generated_label_of_the_wrong_arity_fails() {
    let (_registry, diagnostics) = latex_fixture_harvest("\\OpenSubProblem[inner:case]{A case}\n");
    assert_eq!(diagnostics.len(), 1, "{diagnostics:#?}");
    assert_eq!(diagnostics[0].code, LabelErrorCode::MalformedLabelShape);
}

/// A macro body is not an occurrence. The parameter token in a macro
/// definition names no label -- each use of the macro is the occurrence
/// -- so the harvest declines it before the shape rule can reach it.
#[test]
fn a_macro_parameter_body_is_not_an_occurrence() {
    let (registry, diagnostics) =
        latex_fixture_harvest("\\newcommand{\\Investigation}[2]{\\label{invest:attestation:#2}}\n");
    assert!(diagnostics.is_empty(), "{diagnostics:#?}");
    assert_eq!(registry.len(), 0, "a macro definition mints nothing");
}

/// The attestation owner no longer admits the two-segment form its surface
/// carried before the migration.
#[test]
fn the_attestation_shape_admits_three_segments_only() {
    assert!(matches!(
        Label::parse("sec:overview", LabelShape::Attestation),
        Err(LabelParseError::Arity(_)),
    ));
    assert!(Label::parse("sec:attestation:overview", LabelShape::Attestation).is_ok());
}

/// The realization contract's twenty divisions are frozen by name, not
/// admitted by a looser shape: an unlisted two-segment token fails
/// there exactly as it does anywhere else.
#[test]
fn the_realization_divisions_are_frozen_by_name() {
    assert!(Label::parse("sec:representation", LabelShape::Realization).is_ok());
    assert!(matches!(
        Label::parse("sec:invented", LabelShape::Realization),
        Err(LabelParseError::Arity(_)),
    ));
}

/// The coordination rule, in one fixture. In scanned comment text the
/// acute carries the label syntax and the backtick does not, so a
/// two-segment acute span is a shape defect and never a near miss,
/// while a label-shaped backtick span is a near miss and never a shape
/// defect.
#[test]
fn a_span_is_never_both_a_near_miss_and_a_shape_defect() {
    let harvest = rust_fixture_harvest(concat!(
        "// a shape defect: \u{b4}def:fixture\u{b4}\n",
        "// a near miss: `rem:overview:status-tags`\n",
        "// neither: `def:fixture`\n",
    ));
    let mut codes: Vec<(LabelErrorCode, usize)> = harvest
        .diagnostics
        .iter()
        .map(|diagnostic| (diagnostic.code, diagnostic.line))
        .collect();
    codes.sort_by_key(|(_, line)| *line);
    assert_eq!(
        codes,
        vec![
            (LabelErrorCode::MalformedLabelShape, 1),
            (LabelErrorCode::NearMissSpan, 2),
        ],
        "{:#?}",
        harvest.diagnostics,
    );
}

/// The rule is live over the tree it governs: no occurrence anywhere in
/// the corpus is label-intended and of the wrong arity.
#[test]
fn the_live_tree_carries_no_malformed_label_shape() {
    let census = RepositoryCensus::discover(repository_root());
    let labels = RepositoryLabels::harvest_sources(&census);
    let failures: Vec<&LabelDiagnostic> = labels
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == LabelErrorCode::MalformedLabelShape)
        .collect();
    assert!(failures.is_empty(), "{failures:?}");
}
