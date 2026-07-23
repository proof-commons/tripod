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
//! | `flatten_inlines_a_supplied_file_anywhere` | A listed file is matched by name, not dir |
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
//! | `strict_bibliography_fails_on_missing_bib`  | Release mode hard-fails an unlisted bib |
//! | `flatten_embeds_a_supplied_bibliography`     | A listed bib is inlined verbatim        |
//! | `flatten_rejects_absolute_include`          | `\input{/etc/passwd}` refused           |
//! | `flatten_rejects_parent_traversal`          | `\input{../../secret}` refused          |
//! | `flatten_rejects_unlisted_file_on_disk`     | A real adjacent file not on the list is refused |
//! | `flatten_rejects_symlink_escape`            | An unlisted symlink is never followed   |
//! | `flatten_aborts_on_ambiguous_reference`     | A reference matching two listed files aborts |
//! | `flatten_disambiguates_by_folder`           | Folder components resolve a duplicate filename |

// These tests embed LaTeX include literals such as `\input{section}`. When a
// brace's contents happen to name an in-scope binding, clippy suspects a
// missed `format!`; here they are deliberate TeX source, not format strings.
#![allow(clippy::literal_string_with_formatting_args)]

use std::fs;
use std::path::{Path, PathBuf};

use tempfile::tempdir;

use super::{FlattenOptions, extract_braced_with_rest, extract_include_with_rest, flatten};

/// Write `content` to `path`, creating parent directories as needed.
fn write_file(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent dir");
    }
    fs::write(path, content).expect("write file");
}

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
    let main = paper_dir.join("main.tex");
    let section = paper_dir.join("section.tex");
    write_file(
        &main,
        "\\documentclass{article}\n\\begin{document}\n\\input{section}\n\\end{document}\n",
    );
    write_file(
        &section,
        "\\documentclass[../main.tex]{subfiles}\n\\begin{document}\nhello world\n\\end{document}\n",
    );

    let output = root.join("flat/blueprint_flat.tex");
    flatten(&main, &[section], &output, &FlattenOptions::default()).expect("flatten");
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
    let main = root.join("papers/network/main.tex");
    // The bridge file is neither on disk nor on the allowlist; the flattener
    // must still succeed because the include is annotated `% flatten-ignore`.
    write_file(
        &main,
        "\\documentclass{article}\n\\begin{document}\n\\input{generated_bridge} % flatten-ignore\n\\end{document}\n",
    );

    let output = root.join("flat/blueprint_flat.tex");
    flatten(&main, &[], &output, &FlattenOptions::default()).expect("flatten");
    let text = fs::read_to_string(output).expect("read output");
    assert!(text.contains("% \\input{generated_bridge} % flatten-ignore"));
    assert!(text.contains("flatten-ignore: external include 'generated_bridge' omitted"));
    assert!(!text.contains("BEGIN included content from: generated_bridge"));
}

#[test]
fn flatten_inlines_a_supplied_file_anywhere() {
    let dir = tempdir().expect("tempdir");
    let root = dir.path();
    let main = root.join("papers/network/main.tex");
    // The macro file lives in a completely separate directory; the flattener
    // finds it purely by matching the reference name against the supplied
    // list, with no knowledge of either directory.
    let staged = root.join("build/staged/macros_token.tex");
    write_file(
        &main,
        "\\documentclass{article}\n\\input{macros_token.tex}\n\\begin{document}\n\\end{document}\n",
    );
    write_file(&staged, "\\newcommand{\\foo}{bar}\n");

    let output = root.join("flat/blueprint_flat.tex");
    flatten(&main, &[staged], &output, &FlattenOptions::default()).expect("flatten");
    let text = fs::read_to_string(output).expect("read output");
    assert!(text.contains("% --- BEGIN included content from: macros_token.tex ---"));
    assert!(text.contains("\\newcommand{\\foo}{bar}"));
}

#[test]
fn flatten_expands_conditional_include() {
    let dir = tempdir().expect("tempdir");
    let root = dir.path();
    let main = root.join("papers/economic/main.tex");
    let bridge = root.join("build/papers/economic/bridge.tex");
    write_file(
        &main,
        "\\documentclass{article}\n\\newif\\ifbridge\n\\bridgefalse\n\\IfFileExists{bridge.tex}{\\input{bridge}\\bridgetrue}{}\n\\begin{document}\n\\end{document}\n",
    );
    write_file(&bridge, "\\newcommand{\\linked}{yes}\n");

    let output = root.join("flat/economic_flat.tex");
    flatten(&main, &[bridge], &output, &FlattenOptions::default()).expect("flatten");
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
    let main = root.join("paper/main.tex");
    let section = root.join("paper/section.tex");
    write_file(
        &main,
        "\\documentclass{article}\n\\begin{document}\n\\input{section}\n\\end{document}\n",
    );
    write_file(&section, "content\n");

    let first = root.join("first.tex");
    let second = root.join("second.tex");
    let allow = [section];
    flatten(&main, &allow, &first, &FlattenOptions::default()).expect("flatten");
    flatten(&main, &allow, &second, &FlattenOptions::default()).expect("flatten");

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
    let main = root.join("paper/main.tex");
    let a = root.join("paper/a.tex");
    let b = root.join("paper/b.tex");
    // a -> b -> a: must fail with a cycle diagnostic, not recurse.
    write_file(
        &main,
        "\\documentclass{article}\n\\begin{document}\n\\input{a}\n\\end{document}\n",
    );
    write_file(&a, "\\input{b}\n");
    write_file(&b, "\\input{a}\n");

    let output = root.join("flat.tex");
    let error =
        flatten(&main, &[a, b], &output, &FlattenOptions::default()).expect_err("cycle must fail");
    assert!(error.to_string().contains("include cycle"), "{error}");
}

/// Build `main.tex` holding `body` between the standard document markers, plus
/// a trivial `section.tex`. Returns the main path and the `[section.tex]`
/// allowlist.
fn paper_with_body(root: &Path, body: &str) -> (PathBuf, Vec<PathBuf>) {
    let main = root.join("paper/main.tex");
    let section = root.join("paper/section.tex");
    write_file(
        &main,
        &format!("\\documentclass{{article}}\n\\begin{{document}}\n{body}\n\\end{{document}}\n"),
    );
    write_file(&section, "hello section\n");
    (main, vec![section])
}

#[test]
fn flatten_rejects_trailing_tokens_after_include() {
    let dir = tempdir().expect("tempdir");
    let (main, allow) = paper_with_body(dir.path(), "\\input{section}\\label{after-section}");

    let output = dir.path().join("flat.tex");
    let error = flatten(&main, &allow, &output, &FlattenOptions::default())
        .expect_err("trailing tokens after an include must fail, not silently drop");

    assert!(error.to_string().contains("unsupported include syntax"));
    assert!(!output.exists());
}

#[test]
fn flatten_allows_trailing_comment_after_include() {
    let dir = tempdir().expect("tempdir");
    let (main, allow) = paper_with_body(dir.path(), "\\input{section} % layer note");

    let output = dir.path().join("flat.tex");
    flatten(&main, &allow, &output, &FlattenOptions::default())
        .expect("trailing comment is not semantic content");

    let text = fs::read_to_string(output).expect("read output");
    assert!(text.contains("hello section"));
}

#[test]
fn flatten_rejects_embedded_include() {
    let dir = tempdir().expect("tempdir");
    let (main, allow) = paper_with_body(dir.path(), "prefix text \\input{section}");

    let error = flatten(
        &main,
        &allow,
        &dir.path().join("flat.tex"),
        &FlattenOptions::default(),
    )
    .expect_err("an include the dispatcher cannot handle must not pass through");

    assert!(error.to_string().contains("unsupported include syntax"));
}

#[test]
fn flatten_rejects_nonempty_false_branch() {
    let dir = tempdir().expect("tempdir");
    let (main, allow) = paper_with_body(
        dir.path(),
        "\\IfFileExists{missing-optional.tex}{\\input{missing-optional}}{\\fallbackmacro}",
    );

    let error = flatten(
        &main,
        &allow,
        &dir.path().join("flat.tex"),
        &FlattenOptions::default(),
    )
    .expect_err("a dropped nonempty false branch must fail the flatten");

    assert!(error.to_string().contains("false branch"));
}

#[test]
fn flatten_passes_commented_includes() {
    let dir = tempdir().expect("tempdir");
    let (main, allow) = paper_with_body(dir.path(), "% \\input{not-really-included}");

    let output = dir.path().join("flat.tex");
    flatten(&main, &allow, &output, &FlattenOptions::default())
        .expect("a commented include is inert");

    let text = fs::read_to_string(output).expect("read output");
    assert!(text.contains("% \\input{not-really-included}"));
}

#[test]
fn flatten_failure_leaves_no_partial_output() {
    let dir = tempdir().expect("tempdir");
    let root = dir.path();
    let main = root.join("paper/main.tex");
    write_file(
        &main,
        "\\documentclass{article}\n\\begin{document}\n\\input{missing-section}\n\\end{document}\n",
    );

    let output = root.join("flat.tex");
    flatten(&main, &[], &output, &FlattenOptions::default())
        .expect_err("an unlisted include must fail");

    assert!(!output.exists(), "failed flatten left a partial output");
    assert_eq!(
        staged_leftovers(root),
        0,
        "failed flatten left a temporary file"
    );
}

/// Count leftover staging files (any name marking a temporary) under `root`.
fn staged_leftovers(root: &Path) -> usize {
    fs::read_dir(root)
        .expect("read root")
        .filter_map(std::result::Result::ok)
        .filter(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            Path::new(&name)
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
    let main = root.join("paper/main.tex");
    write_file(
        &main,
        "\\documentclass{article}\n\\begin{document}\n\\input{missing-section}\n\\end{document}\n",
    );

    let output = root.join("flat.tex");
    fs::write(&output, "previous good output\n").expect("write prior output");

    flatten(&main, &[], &output, &FlattenOptions::default())
        .expect_err("an unlisted include must fail");

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

    // Two papers with distinct, multi-line content flattened to the same
    // output path. Any staging-path collision shows up as mixed or truncated
    // bytes; correct staging yields exactly one paper's complete flatten.
    let mut mains = Vec::new();
    let mut expected = Vec::new();
    for index in 0..2 {
        let main = root.join(format!("paper-{index}/main.tex"));
        let body = (0..200).fold(String::new(), |mut body, line| {
            use std::fmt::Write as _;
            let _ = writeln!(body, "paper {index} line {line}");
            body
        });
        write_file(
            &main,
            &format!("\\documentclass{{article}}\n\\begin{{document}}\n{body}\\end{{document}}\n"),
        );

        let reference = root.join(format!("reference-{index}.tex"));
        flatten(&main, &[], &reference, &FlattenOptions::default()).expect("reference flatten");
        expected.push(fs::read(reference).expect("read reference"));
        mains.push(main);
    }

    let output = root.join("flat.tex");
    for _round in 0..20 {
        std::thread::scope(|scope| {
            for main in &mains {
                let output = output.clone();
                scope.spawn(move || {
                    flatten(main, &[], &output, &FlattenOptions::default())
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
    let main = root.join("paper/main.tex");
    write_file(
        &main,
        "\\documentclass{article}\n\\addbibresource{references.bib}\n\\begin{document}\n\\end{document}\n",
    );

    // Default mode: warning comment, successful flatten (bib not supplied).
    let output = root.join("flat.tex");
    flatten(&main, &[], &output, &FlattenOptions::default()).expect("lenient flatten succeeds");
    let text = fs::read_to_string(&output).expect("read output");
    assert!(text.contains("% WARNING: Bibliography file 'references.bib' not found"));

    // Strict (release) mode: hard failure, no output.
    let strict_output = root.join("strict.tex");
    let error = flatten(
        &main,
        &[],
        &strict_output,
        &FlattenOptions {
            strict_bibliography: true,
        },
    )
    .expect_err("strict flatten fails");
    assert!(error.to_string().contains("strict mode"), "{error}");
    assert!(!strict_output.exists());
}

#[test]
fn flatten_embeds_a_supplied_bibliography() {
    let dir = tempdir().expect("tempdir");
    let root = dir.path();
    let main = root.join("paper/main.tex");
    let bib = root.join("paper/references.bib");
    write_file(
        &main,
        "\\documentclass{article}\n\\addbibresource{references.bib}\n\\begin{document}\n\\end{document}\n",
    );
    write_file(&bib, "@misc{key, title = {A Title}}\n");

    let output = root.join("flat.tex");
    flatten(
        &main,
        &[bib],
        &output,
        &FlattenOptions {
            strict_bibliography: true,
        },
    )
    .expect("a supplied bibliography is embedded");
    let text = fs::read_to_string(&output).expect("read output");
    assert!(text.contains("% --- BEGIN embedded bibliography from: references.bib ---"));
    assert!(text.contains("\\begin{filecontents*}{references.bib}"));
    assert!(text.contains("@misc{key, title = {A Title}}"));
    assert!(text.contains("\\addbibresource{references.bib}"));
}

// --- Path-confinement regressions (Finding 1 / ADR-015). The flattener does
// no directory lookup, so a reference can only ever name a file on the
// supplied list; an absolute path, a `..` escape, a real adjacent file, and a
// symlink target are all unreachable unless explicitly supplied. ---

#[test]
fn flatten_rejects_absolute_include() {
    let dir = tempdir().expect("tempdir");
    let root = dir.path();
    let main = root.join("paper/main.tex");
    write_file(
        &main,
        "\\documentclass{article}\n\\begin{document}\n\\input{/etc/passwd}\n\\end{document}\n",
    );

    let output = root.join("flat.tex");
    let error = flatten(&main, &[], &output, &FlattenOptions::default())
        .expect_err("an absolute include must be refused");
    assert!(error.to_string().contains("absolute path"), "{error}");
    assert!(!output.exists());
}

#[test]
fn flatten_rejects_parent_traversal() {
    let dir = tempdir().expect("tempdir");
    let root = dir.path();
    let main = root.join("paper/main.tex");
    // A secret exists outside the paper tree; a `..` escape must not reach it.
    write_file(&root.join("outside-secret.tex"), "TOP SECRET\n");
    write_file(
        &main,
        "\\documentclass{article}\n\\begin{document}\n\\input{../../outside-secret}\n\\end{document}\n",
    );

    let output = root.join("flat.tex");
    let error = flatten(&main, &[], &output, &FlattenOptions::default())
        .expect_err("a parent-directory escape must be refused");
    assert!(error.to_string().contains("parent-directory"), "{error}");
    assert!(!output.exists());
}

#[test]
fn flatten_rejects_unlisted_file_on_disk() {
    let dir = tempdir().expect("tempdir");
    let root = dir.path();
    let main = root.join("paper/main.tex");
    // section.tex exists right next to main.tex, but is NOT on the allowlist.
    // A directory-based flattener would inline it; this one must refuse.
    write_file(&root.join("paper/section.tex"), "adjacent but unlisted\n");
    write_file(
        &main,
        "\\documentclass{article}\n\\begin{document}\n\\input{section}\n\\end{document}\n",
    );

    let output = root.join("flat.tex");
    let error = flatten(&main, &[], &output, &FlattenOptions::default())
        .expect_err("a file not on the supplied list must be refused even if it exists on disk");
    assert!(
        error
            .to_string()
            .contains("does not match any file supplied"),
        "{error}"
    );
    assert!(!output.exists());
}

#[cfg(unix)]
#[test]
fn flatten_rejects_symlink_escape() {
    let dir = tempdir().expect("tempdir");
    let root = dir.path();
    let main = root.join("paper/main.tex");
    let secret = root.join("outside-secret.tex");
    write_file(&secret, "TOP SECRET\n");
    // A symlink inside the paper tree pointing outside it. It is not on the
    // allowlist, so it is never followed.
    let link = root.join("paper/link.tex");
    fs::create_dir_all(link.parent().unwrap()).expect("mkdir");
    std::os::unix::fs::symlink(&secret, &link).expect("symlink");
    write_file(
        &main,
        "\\documentclass{article}\n\\begin{document}\n\\input{link.tex}\n\\end{document}\n",
    );

    let output = root.join("flat.tex");
    let error = flatten(&main, &[], &output, &FlattenOptions::default())
        .expect_err("an unlisted symlink must be refused");
    assert!(
        error
            .to_string()
            .contains("does not match any file supplied"),
        "{error}"
    );
    assert!(!output.exists());
    let text = fs::read_to_string(&output).unwrap_or_default();
    assert!(!text.contains("TOP SECRET"));
}

#[test]
fn flatten_aborts_on_ambiguous_reference() {
    let dir = tempdir().expect("tempdir");
    let root = dir.path();
    let main = root.join("paper/main.tex");
    // Two supplied files share the basename referenced by \input, and the
    // reference names only the basename — an ambiguous match must abort.
    let first = root.join("a/dup.tex");
    let second = root.join("b/dup.tex");
    write_file(&first, "first dup\n");
    write_file(&second, "second dup\n");
    write_file(
        &main,
        "\\documentclass{article}\n\\begin{document}\n\\input{dup.tex}\n\\end{document}\n",
    );

    let output = root.join("flat.tex");
    let error = flatten(&main, &[first, second], &output, &FlattenOptions::default())
        .expect_err("an ambiguous reference must abort");
    assert!(error.to_string().contains("ambiguously matches"), "{error}");
    assert!(!output.exists());
}

#[test]
fn flatten_disambiguates_by_folder() {
    let dir = tempdir().expect("tempdir");
    let root = dir.path();
    let main = root.join("paper/main.tex");
    // Two files share a basename, but the reference names a folder component,
    // so the match is unique: filename first, then folder.
    let a_dup = root.join("a/dup.tex");
    let b_dup = root.join("b/dup.tex");
    write_file(&a_dup, "from a\n");
    write_file(&b_dup, "from b\n");
    write_file(
        &main,
        "\\documentclass{article}\n\\begin{document}\n\\input{b/dup.tex}\n\\end{document}\n",
    );

    let output = root.join("flat.tex");
    flatten(&main, &[a_dup, b_dup], &output, &FlattenOptions::default())
        .expect("a folder-qualified reference resolves uniquely");
    let text = fs::read_to_string(output).expect("read output");
    assert!(text.contains("from b"));
    assert!(!text.contains("from a"));
}
