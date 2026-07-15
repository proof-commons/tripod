//! # `flatten-latex-main` library tests
//!
//! | Test                                       | What it covers                          |
//! |--------------------------------------------|-----------------------------------------|
//! | `extract_braced_returns_inner_value_and_rest` | Parses `\cmd{value}` plus remainder     |
//! | `extract_braced_rejects_other_command`     | Returns None when prefix mismatches     |
//! | `extract_include_handles_input`            | `\input{}` parsed                       |
//! | `extract_include_handles_subfile`          | `\subfile{}` parsed                     |
//! | `flatten_writes_inlined_output`            | End-to-end flatten on a temp tree       |
//! | `flatten_ignores_marked_include`           | `% flatten-ignore` include is skipped   |
//! | `flatten_resolves_include_via_include_dir` | Fallback search dir inlines a macro     |
//! | `flatten_expands_conditional_include`       | Conditional include keeps survivor code |
//! | `flatten_is_reproducible`                   | Same inputs, byte-identical output      |
//! | `flatten_detects_include_cycles`            | Self/mutual includes fail, not recurse  |
//! | `flatten_rejects_trailing_tokens_after_include` | `\input{a}\label{b}` fails, not drops |
//! | `flatten_allows_trailing_comment_after_include` | `\input{a} % note` still flattens   |
//! | `flatten_rejects_embedded_include`          | prefixed/mid-line includes fail         |
//! | `flatten_rejects_nonempty_false_branch`     | absent conditional keeps no silent drop |
//! | `flatten_passes_commented_includes`         | `% \input{a}` is inert                  |
//! | `flatten_failure_leaves_no_partial_output`  | Atomic output on failed flatten         |
//! | `flatten_failure_preserves_existing_output` | Failed flatten keeps prior output bytes |
//! | `concurrent_flattens_never_mix_output`      | Same-output races yield one full result |
//! | `strict_bibliography_fails_on_missing_bib`  | Release mode hard-fails a missing bib   |

use std::fs;

use tempfile::tempdir;

use super::{FlattenOptions, extract_braced_with_rest, extract_include_with_rest, flatten};

#[test]
fn extract_braced_returns_inner_value_and_rest() {
    let parsed = extract_braced_with_rest("\\addbibresource{refs.bib} % note", "\\addbibresource");
    assert_eq!(
        parsed,
        Some(("refs.bib".to_string(), " % note")),
        "value and remainder are both returned"
    );
}

#[test]
fn extract_braced_rejects_other_command() {
    let parsed = extract_braced_with_rest("\\addbibresource{refs.bib}", "\\input");
    assert!(parsed.is_none());
}

#[test]
fn extract_include_handles_input() {
    let parsed = extract_include_with_rest("\\input{section}");
    assert_eq!(parsed, Some(("section".to_string(), "")));
}

#[test]
fn extract_include_handles_subfile() {
    let parsed = extract_include_with_rest("\\subfile{chapter}");
    assert_eq!(parsed, Some(("chapter".to_string(), "")));
}

#[test]
fn flatten_writes_inlined_output() {
    let dir = tempdir().expect("tempdir");
    let root = dir.path();
    let paper_dir = root.join("papers/network");
    fs::create_dir_all(&paper_dir).expect("create paper dir");
    fs::write(
        paper_dir.join("main.tex"),
        "\\documentclass{article}\n\\begin{document}\n\\input{section}\n\\end{document}\n",
    )
    .expect("write main");
    fs::write(
        paper_dir.join("section.tex"),
        "\\documentclass[../main.tex]{subfiles}\n\\begin{document}\nhello world\n\\end{document}\n",
    )
    .expect("write section");

    let output = root.join("flat/blueprint_flat.tex");
    flatten(
        &paper_dir,
        &paper_dir.join("main.tex"),
        &output,
        &[],
        &FlattenOptions::default(),
    )
    .expect("flatten");
    let text = fs::read_to_string(output).expect("read output");
    assert!(text.contains("\\documentclass{article}"));
    assert!(text.contains("% \\documentclass[../main.tex]{subfiles}"));
    assert!(text.contains("\\begin{document}"));
    assert!(text.contains("% \\begin{document}"));
    assert!(text.contains("% --- BEGIN included content from: section ---"));
    assert!(text.contains("hello world"));
    assert!(text.contains("% \\end{document}"));
    assert!(text.contains("% --- END included content from: section ---"));
}

#[test]
fn flatten_ignores_marked_include() {
    let dir = tempdir().expect("tempdir");
    let root = dir.path();
    let paper_dir = root.join("papers/network");
    fs::create_dir_all(&paper_dir).expect("create paper dir");
    // The bridge file deliberately does NOT exist on disk; the flattener must
    // still succeed because the include is annotated with `% flatten-ignore`.
    fs::write(
        paper_dir.join("main.tex"),
        "\\documentclass{article}\n\\begin{document}\n\\input{generated_bridge} % flatten-ignore\n\\end{document}\n",
    )
    .expect("write main");

    let output = root.join("flat/blueprint_flat.tex");
    flatten(
        &paper_dir,
        &paper_dir.join("main.tex"),
        &output,
        &[],
        &FlattenOptions::default(),
    )
    .expect("flatten");
    let text = fs::read_to_string(output).expect("read output");
    assert!(text.contains("% \\input{generated_bridge} % flatten-ignore"));
    assert!(text.contains("flatten-ignore: external include 'generated_bridge' omitted"));
    assert!(!text.contains("BEGIN included content from: generated_bridge"));
}

#[test]
fn flatten_resolves_include_via_include_dir() {
    let dir = tempdir().expect("tempdir");
    let root = dir.path();
    let paper_dir = root.join("papers/network");
    fs::create_dir_all(&paper_dir).expect("create paper dir");
    // The macro file lives only in a separate build/staging directory, not
    // beside the paper sources — it must be resolved via `include_dirs`.
    let staged_dir = root.join("build/papers/network");
    fs::create_dir_all(&staged_dir).expect("create staged dir");
    fs::write(
        paper_dir.join("main.tex"),
        "\\documentclass{article}\n\\input{macros_token.tex}\n\\begin{document}\n\\end{document}\n",
    )
    .expect("write main");
    fs::write(
        staged_dir.join("macros_token.tex"),
        "\\newcommand{\\foo}{bar}\n",
    )
    .expect("write staged macro");

    let output = root.join("flat/blueprint_flat.tex");
    flatten(
        &paper_dir,
        &paper_dir.join("main.tex"),
        &output,
        &[staged_dir],
        &FlattenOptions::default(),
    )
    .expect("flatten");
    let text = fs::read_to_string(output).expect("read output");
    assert!(text.contains("% --- BEGIN included content from: macros_token.tex ---"));
    assert!(text.contains("\\newcommand{\\foo}{bar}"));
}

#[test]
fn flatten_expands_conditional_include() {
    let dir = tempdir().expect("tempdir");
    let root = dir.path();
    let paper_dir = root.join("papers/economic");
    fs::create_dir_all(&paper_dir).expect("create paper dir");
    let staged_dir = root.join("build/papers/economic");
    fs::create_dir_all(&staged_dir).expect("create staged dir");
    fs::write(
        paper_dir.join("main.tex"),
        "\\documentclass{article}\n\\newif\\ifbridge\n\\bridgefalse\n\\IfFileExists{bridge.tex}{\\input{bridge}\\bridgetrue}{}\n\\begin{document}\n\\end{document}\n",
    )
    .expect("write main");
    fs::write(
        staged_dir.join("bridge.tex"),
        "\\newcommand{\\linked}{yes}\n",
    )
    .expect("write staged bridge");

    let output = root.join("flat/economic_flat.tex");
    flatten(
        &paper_dir,
        &paper_dir.join("main.tex"),
        &output,
        &[staged_dir],
        &FlattenOptions::default(),
    )
    .expect("flatten");
    let text = fs::read_to_string(output).expect("read output");
    assert!(text.contains("% \\IfFileExists{bridge.tex}{\\input{bridge}\\bridgetrue}{}"));
    assert!(text.contains("% --- BEGIN included content from: bridge ---"));
    assert!(text.contains("\\newcommand{\\linked}{yes}"));
    assert!(text.contains("% --- END included content from: bridge ---\n\\bridgetrue"));
    assert!(!text.contains("\\IfFileExists{bridge.tex}{%"));
}

#[test]
fn flatten_is_reproducible() {
    let dir = tempdir().expect("tempdir");
    let root = dir.path();
    let paper_dir = root.join("paper");
    fs::create_dir_all(&paper_dir).expect("create paper dir");
    fs::write(
        paper_dir.join("main.tex"),
        "\\documentclass{article}\n\\begin{document}\n\\input{section}\n\\end{document}\n",
    )
    .expect("write main");
    fs::write(paper_dir.join("section.tex"), "content\n").expect("write section");

    let first = root.join("first.tex");
    let second = root.join("second.tex");
    flatten(
        &paper_dir,
        &paper_dir.join("main.tex"),
        &first,
        &[],
        &FlattenOptions::default(),
    )
    .expect("flatten");
    flatten(
        &paper_dir,
        &paper_dir.join("main.tex"),
        &second,
        &[],
        &FlattenOptions::default(),
    )
    .expect("flatten");

    assert_eq!(
        fs::read(first).expect("read first"),
        fs::read(second).expect("read second"),
        "flatten output is not byte-identical across runs",
    );
}

#[test]
fn flatten_detects_include_cycles() {
    let dir = tempdir().expect("tempdir");
    let root = dir.path();
    let paper_dir = root.join("paper");
    fs::create_dir_all(&paper_dir).expect("create paper dir");
    // a -> b -> a: must fail with a cycle diagnostic, not recurse.
    fs::write(
        paper_dir.join("main.tex"),
        "\\documentclass{article}\n\\begin{document}\n\\input{a}\n\\end{document}\n",
    )
    .expect("write main");
    fs::write(paper_dir.join("a.tex"), "\\input{b}\n").expect("write a");
    fs::write(paper_dir.join("b.tex"), "\\input{a}\n").expect("write b");

    let output = root.join("flat.tex");
    let error = flatten(
        &paper_dir,
        &paper_dir.join("main.tex"),
        &output,
        &[],
        &FlattenOptions::default(),
    )
    .expect_err("cycle must fail");
    assert!(error.to_string().contains("include cycle"), "{error}");
}

/// Build a paper directory with `main.tex` holding `body` between the
/// standard document markers, plus a trivial `section.tex`.
fn paper_with_body(root: &std::path::Path, body: &str) -> std::path::PathBuf {
    let paper_dir = root.join("paper");
    fs::create_dir_all(&paper_dir).expect("create paper dir");
    fs::write(
        paper_dir.join("main.tex"),
        format!("\\documentclass{{article}}\n\\begin{{document}}\n{body}\n\\end{{document}}\n"),
    )
    .expect("write main");
    fs::write(paper_dir.join("section.tex"), "hello section\n").expect("write section");
    paper_dir
}

#[test]
fn flatten_rejects_trailing_tokens_after_include() {
    let dir = tempdir().expect("tempdir");
    let paper_dir = paper_with_body(dir.path(), "\\input{section}\\label{after-section}");

    let output = dir.path().join("flat.tex");
    let error = flatten(
        &paper_dir,
        &paper_dir.join("main.tex"),
        &output,
        &[],
        &FlattenOptions::default(),
    )
    .expect_err("trailing tokens after an include must fail, not silently drop");

    assert!(error.to_string().contains("unsupported include syntax"));
    assert!(!output.exists());
}

#[test]
fn flatten_allows_trailing_comment_after_include() {
    let dir = tempdir().expect("tempdir");
    let paper_dir = paper_with_body(dir.path(), "\\input{section} % layer note");

    let output = dir.path().join("flat.tex");
    flatten(
        &paper_dir,
        &paper_dir.join("main.tex"),
        &output,
        &[],
        &FlattenOptions::default(),
    )
    .expect("trailing comment is not semantic content");

    let text = fs::read_to_string(output).expect("read output");
    assert!(text.contains("hello section"));
}

#[test]
fn flatten_rejects_embedded_include() {
    let dir = tempdir().expect("tempdir");
    let paper_dir = paper_with_body(dir.path(), "prefix text \\input{section}");

    let error = flatten(
        &paper_dir,
        &paper_dir.join("main.tex"),
        &dir.path().join("flat.tex"),
        &[],
        &FlattenOptions::default(),
    )
    .expect_err("an include the dispatcher cannot handle must not pass through");

    assert!(error.to_string().contains("unsupported include syntax"));
}

#[test]
fn flatten_rejects_nonempty_false_branch() {
    let dir = tempdir().expect("tempdir");
    let paper_dir = paper_with_body(
        dir.path(),
        "\\IfFileExists{missing-optional.tex}{\\input{missing-optional}}{\\fallbackmacro}",
    );

    let error = flatten(
        &paper_dir,
        &paper_dir.join("main.tex"),
        &dir.path().join("flat.tex"),
        &[],
        &FlattenOptions::default(),
    )
    .expect_err("a dropped nonempty false branch must fail the flatten");

    assert!(error.to_string().contains("false branch"));
}

#[test]
fn flatten_passes_commented_includes() {
    let dir = tempdir().expect("tempdir");
    let paper_dir = paper_with_body(dir.path(), "% \\input{not-really-included}");

    let output = dir.path().join("flat.tex");
    flatten(
        &paper_dir,
        &paper_dir.join("main.tex"),
        &output,
        &[],
        &FlattenOptions::default(),
    )
    .expect("a commented include is inert");

    let text = fs::read_to_string(output).expect("read output");
    assert!(text.contains("% \\input{not-really-included}"));
}

#[test]
fn flatten_failure_leaves_no_partial_output() {
    let dir = tempdir().expect("tempdir");
    let root = dir.path();
    let paper_dir = root.join("paper");
    fs::create_dir_all(&paper_dir).expect("create paper dir");
    fs::write(
        paper_dir.join("main.tex"),
        "\\documentclass{article}\n\\begin{document}\n\\input{missing-section}\n\\end{document}\n",
    )
    .expect("write main");

    let output = root.join("flat.tex");
    flatten(
        &paper_dir,
        &paper_dir.join("main.tex"),
        &output,
        &[],
        &FlattenOptions::default(),
    )
    .expect_err("missing include must fail");

    assert!(!output.exists(), "failed flatten left a partial output");
    assert_eq!(
        staged_leftovers(root),
        0,
        "failed flatten left a temporary file"
    );
}

/// Count leftover staging files (any name marking a temporary) under `root`.
fn staged_leftovers(root: &std::path::Path) -> usize {
    fs::read_dir(root)
        .expect("read root")
        .filter_map(std::result::Result::ok)
        .filter(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            std::path::Path::new(&name)
                .extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("tmp"))
                || name.starts_with(".flatten-staged-")
        })
        .count()
}

#[test]
fn flatten_failure_preserves_existing_output() {
    let dir = tempdir().expect("tempdir");
    let root = dir.path();
    let paper_dir = root.join("paper");
    fs::create_dir_all(&paper_dir).expect("create paper dir");
    fs::write(
        paper_dir.join("main.tex"),
        "\\documentclass{article}\n\\begin{document}\n\\input{missing-section}\n\\end{document}\n",
    )
    .expect("write main");

    let output = root.join("flat.tex");
    fs::write(&output, "previous good output\n").expect("write prior output");

    flatten(
        &paper_dir,
        &paper_dir.join("main.tex"),
        &output,
        &[],
        &FlattenOptions::default(),
    )
    .expect_err("missing include must fail");

    assert_eq!(
        fs::read_to_string(&output).expect("read output"),
        "previous good output\n",
        "failed flatten must not disturb an existing output"
    );
    assert_eq!(
        staged_leftovers(root),
        0,
        "failed flatten left a temporary file"
    );
}

#[test]
fn concurrent_flattens_never_mix_output() {
    let dir = tempdir().expect("tempdir");
    let root = dir.path();

    // Two papers with distinct, multi-line content flattened to the
    // same output path. Any staging-path collision shows up as mixed
    // or truncated bytes; correct staging yields exactly one paper's
    // complete flatten.
    let mut expected = Vec::new();
    for index in 0..2 {
        let paper_dir = root.join(format!("paper-{index}"));
        fs::create_dir_all(&paper_dir).expect("create paper dir");
        let body = (0..200).fold(String::new(), |mut body, line| {
            use std::fmt::Write as _;
            let _ = writeln!(body, "paper {index} line {line}");
            body
        });
        fs::write(
            paper_dir.join("main.tex"),
            format!("\\documentclass{{article}}\n\\begin{{document}}\n{body}\\end{{document}}\n"),
        )
        .expect("write main");

        let reference = root.join(format!("reference-{index}.tex"));
        flatten(
            &paper_dir,
            &paper_dir.join("main.tex"),
            &reference,
            &[],
            &FlattenOptions::default(),
        )
        .expect("reference flatten");
        expected.push(fs::read(reference).expect("read reference"));
    }

    let output = root.join("flat.tex");
    for _round in 0..20 {
        std::thread::scope(|scope| {
            for index in 0..2 {
                let paper_dir = root.join(format!("paper-{index}"));
                let output = output.clone();
                scope.spawn(move || {
                    flatten(
                        &paper_dir,
                        &paper_dir.join("main.tex"),
                        &output,
                        &[],
                        &FlattenOptions::default(),
                    )
                    .expect("concurrent flatten");
                });
            }
        });

        let bytes = fs::read(&output).expect("read output");
        assert!(
            expected.contains(&bytes),
            "concurrent flattens produced mixed output"
        );
    }

    assert_eq!(
        staged_leftovers(root),
        0,
        "concurrent flattens left a temporary file"
    );
}

#[test]
fn strict_bibliography_fails_on_missing_bib() {
    let dir = tempdir().expect("tempdir");
    let root = dir.path();
    let paper_dir = root.join("paper");
    fs::create_dir_all(&paper_dir).expect("create paper dir");
    fs::write(
        paper_dir.join("main.tex"),
        "\\documentclass{article}\n\\addbibresource{references.bib}\n\\begin{document}\n\\end{document}\n",
    )
    .expect("write main");

    let output = root.join("flat.tex");

    // Default mode: warning comment, successful flatten.
    flatten(
        &paper_dir,
        &paper_dir.join("main.tex"),
        &output,
        &[],
        &FlattenOptions::default(),
    )
    .expect("lenient flatten succeeds");
    let text = fs::read_to_string(&output).expect("read output");
    assert!(text.contains("% WARNING: Bibliography file 'references.bib' not found"));

    // Strict (release) mode: hard failure, no output.
    let strict_output = root.join("strict.tex");
    let error = flatten(
        &paper_dir,
        &paper_dir.join("main.tex"),
        &strict_output,
        &[],
        &FlattenOptions {
            strict_bibliography: true,
        },
    )
    .expect_err("strict flatten fails");
    assert!(error.to_string().contains("strict mode"), "{error}");
    assert!(!strict_output.exists());
}
