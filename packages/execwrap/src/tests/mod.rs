//! # `execwrap` library tests
//!
//! | Test                                                | What it covers                          |
//! |-----------------------------------------------------|-----------------------------------------|
//! | `routing_config_detects_combined_redirects`         | `--redirect` triggers both streams      |
//! | `routing_config_detects_single_stream_redirects`    | per-stream redirects only flag one      |
//! | `collect_file_specs_rejects_ambiguous_routing`      | conflicting paths are an error          |
//! | `aliased_redirection_paths_are_rejected`            | `a.log` vs `./a.log` conflict           |
//! | `symlink_and_hardlink_aliases_are_rejected`         | link aliases resolve to one identity    |
//! | `parent_symlink_alias_is_rejected`                  | symlinked parent dirs resolve           |
//! | `dangling_symlink_alias_is_rejected`                | dangling link aliases its target        |
//! | `dangling_symlink_chain_resolves_to_final_target`   | chains resolve to the last target       |
//! | `symlink_loop_is_an_error`                          | link loops error instead of hanging     |
//! | `distinct_redirection_targets_validate`             | different files remain accepted         |
//! | `ambiguous_alias_does_not_truncate_existing_output` | usage failure precedes truncation       |
//! | `dangling_alias_failure_leaves_target_intact`       | preflight only stats, never creates     |
//! | `typesetter_passes_data_through`                    | raw mode writes bytes unchanged         |
//! | `typesetter_preserves_arbitrary_bytes`              | raw mode is byte-exact on binary data   |
//! | `calligraphic_adds_prefix_per_line`                 | prefix mode formats per-line            |
//! | `calligraphic_sanitizes_control_bytes`              | sanitization is calligraphic-only       |
//! | `calligraphic_buffers_partial_lines_until_finalize` | partial lines flush at finalize         |
//! | `calligraphic_preserves_code_point_split_across_two_writes` | chunked multibyte UTF-8 survives |
//! | `calligraphic_preserves_code_point_split_across_three_writes` | byte-at-a-time UTF-8 survives  |
//! | `calligraphic_replaces_invalid_utf8_after_buffering` | genuinely invalid bytes still map to `\u{fffd}` |
//! | `calligraphic_output_is_chunking_invariant`         | same bytes, any chunking, same output   |

use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use super::writer::{StreamProps, Writer};
use super::{ExecError, RoutingConfig, Stream, collect_file_specs, file_identity};

#[test]
fn routing_config_detects_combined_redirects() {
    let cfg = RoutingConfig {
        redirect: Some(PathBuf::from("a.log")),
        ..RoutingConfig::default()
    };
    assert!(cfg.is_stream_redirected(Stream::Stdout));
    assert!(cfg.is_stream_redirected(Stream::Stderr));
}

#[test]
fn routing_config_detects_single_stream_redirects() {
    let cfg = RoutingConfig {
        redirect_output: Some(PathBuf::from("out.log")),
        ..RoutingConfig::default()
    };
    assert!(cfg.is_stream_redirected(Stream::Stdout));
    assert!(!cfg.is_stream_redirected(Stream::Stderr));
}

#[test]
fn collect_file_specs_rejects_ambiguous_routing() {
    let cfg = RoutingConfig {
        redirect: Some(PathBuf::from("a.log")),
        redirect_output: Some(PathBuf::from("a.log")),
        ..RoutingConfig::default()
    };
    let error = collect_file_specs(&cfg).expect_err("expected ambiguity");
    assert!(matches!(error, ExecError::AmbiguousRedirection { .. }));
}

#[test]
fn aliased_redirection_paths_are_rejected() {
    let dir = tempfile::tempdir().expect("tempdir");

    let cfg = RoutingConfig {
        redirect: Some(dir.path().join("a.log")),
        redirect_output: Some(dir.path().join("./a.log")),
        ..RoutingConfig::default()
    };

    assert!(matches!(
        super::preflight_routing(&cfg),
        Err(ExecError::AmbiguousRedirection { .. }),
    ));
}

#[test]
fn symlink_and_hardlink_aliases_are_rejected() {
    let dir = tempfile::tempdir().expect("tempdir");

    let target = dir.path().join("a.log");
    std::fs::write(&target, b"keep").expect("write target");

    let symlink = dir.path().join("sym.log");
    std::os::unix::fs::symlink(&target, &symlink).expect("symlink");

    let symlinked = RoutingConfig {
        redirect: Some(target.clone()),
        redirect_output: Some(symlink),
        ..RoutingConfig::default()
    };

    assert!(matches!(
        super::preflight_routing(&symlinked),
        Err(ExecError::AmbiguousRedirection { .. }),
    ));

    let hard_link = dir.path().join("hard.log");
    std::fs::hard_link(&target, &hard_link).expect("hard link");

    let hard_linked = RoutingConfig {
        redirect: Some(target.clone()),
        redirect_error: Some(hard_link),
        ..RoutingConfig::default()
    };

    assert!(matches!(
        super::preflight_routing(&hard_linked),
        Err(ExecError::AmbiguousRedirection { .. }),
    ));

    // Preflight only stats: the existing target is untouched.
    assert_eq!(std::fs::read(&target).expect("read target"), b"keep");
}

#[test]
fn parent_symlink_alias_is_rejected() {
    let dir = tempfile::tempdir().expect("tempdir");

    let real = dir.path().join("real");
    std::fs::create_dir(&real).expect("create real dir");

    let alias = dir.path().join("alias");
    std::os::unix::fs::symlink(&real, &alias).expect("symlink dir");

    // The target file does not exist yet; only the parents alias.
    let cfg = RoutingConfig {
        redirect: Some(real.join("x.log")),
        redirect_output: Some(alias.join("x.log")),
        ..RoutingConfig::default()
    };

    assert!(matches!(
        super::preflight_routing(&cfg),
        Err(ExecError::AmbiguousRedirection { .. }),
    ));
}

#[test]
fn dangling_symlink_alias_is_rejected() {
    let dir = tempfile::tempdir().expect("tempdir");

    // sym.log dangles: real.log does not exist. File::create on the
    // link would still create/truncate real.log, so the two paths are
    // one identity.
    let real = dir.path().join("real.log");
    let symlink = dir.path().join("sym.log");
    std::os::unix::fs::symlink(&real, &symlink).expect("symlink");

    let cfg = RoutingConfig {
        redirect_output: Some(symlink.clone()),
        redirect_error: Some(real),
        ..RoutingConfig::default()
    };

    assert!(matches!(
        super::preflight_routing(&cfg),
        Err(ExecError::AmbiguousRedirection { .. }),
    ));

    // A dangling link on its own is still a valid single target.
    let alone = RoutingConfig {
        redirect_output: Some(symlink),
        ..RoutingConfig::default()
    };
    let specs = collect_file_specs(&alone).expect("single dangling target is valid");
    assert_eq!(specs.len(), 1);
}

#[test]
fn dangling_symlink_chain_resolves_to_final_target() {
    let dir = tempfile::tempdir().expect("tempdir");

    let real = dir.path().join("real.log");
    let sym1 = dir.path().join("sym1.log");
    let sym2 = dir.path().join("sym2.log");
    std::os::unix::fs::symlink(&real, &sym1).expect("symlink sym1");
    std::os::unix::fs::symlink(&sym1, &sym2).expect("symlink sym2");

    assert_eq!(
        file_identity(&sym2).expect("chain identity"),
        file_identity(&real).expect("target identity"),
        "a dangling chain must share the final target's identity"
    );
}

#[test]
fn symlink_loop_is_an_error() {
    let dir = tempfile::tempdir().expect("tempdir");

    let loop_a = dir.path().join("loop_a.log");
    let loop_b = dir.path().join("loop_b.log");
    std::os::unix::fs::symlink(&loop_a, &loop_b).expect("symlink loop_b");
    std::os::unix::fs::symlink(&loop_b, &loop_a).expect("symlink loop_a");

    assert!(
        file_identity(&loop_a).is_err(),
        "a symlink loop must error, not hang"
    );

    let cfg = RoutingConfig {
        redirect: Some(loop_a),
        ..RoutingConfig::default()
    };
    assert!(matches!(
        super::preflight_routing(&cfg),
        Err(ExecError::Io(_)),
    ));
}

#[test]
fn distinct_redirection_targets_validate() {
    let dir = tempfile::tempdir().expect("tempdir");

    let cfg = RoutingConfig {
        redirect_output: Some(dir.path().join("out.log")),
        redirect_error: Some(dir.path().join("err.log")),
        ..RoutingConfig::default()
    };

    super::preflight_routing(&cfg).expect("distinct targets are valid");
}

#[test]
fn ambiguous_alias_does_not_truncate_existing_output() {
    let dir = tempfile::tempdir().expect("tempdir");

    let target = dir.path().join("a.log");
    std::fs::write(&target, b"previous log contents").expect("write target");

    let symlink = dir.path().join("sym.log");
    std::os::unix::fs::symlink(&target, &symlink).expect("symlink");

    let cfg = RoutingConfig {
        redirect: Some(target.clone()),
        redirect_output: Some(symlink),
        ..RoutingConfig::default()
    };

    let command = [std::ffi::OsString::from("true")];

    assert!(matches!(
        super::run(&command, &cfg, |_notification| {}),
        Err(ExecError::AmbiguousRedirection { .. }),
    ));

    assert_eq!(
        std::fs::read(&target).expect("read target"),
        b"previous log contents",
        "usage failure must precede any truncation"
    );
}

#[test]
fn dangling_alias_failure_leaves_target_intact() {
    let dir = tempfile::tempdir().expect("tempdir");

    let real = dir.path().join("real.log");
    let symlink = dir.path().join("sym.log");
    std::os::unix::fs::symlink(&real, &symlink).expect("symlink");

    let cfg = RoutingConfig {
        redirect_output: Some(symlink),
        redirect_error: Some(real.clone()),
        ..RoutingConfig::default()
    };

    // Preflight only stats: rejecting the dangling alias must not
    // create the missing target.
    assert!(matches!(
        super::preflight_routing(&cfg),
        Err(ExecError::AmbiguousRedirection { .. }),
    ));
    assert!(!real.exists(), "preflight must not create the target");

    // Once the target exists the alias is still rejected, and its
    // contents survive both preflight and a full run attempt.
    std::fs::write(&real, b"previous log contents").expect("write target");

    assert!(matches!(
        super::preflight_routing(&cfg),
        Err(ExecError::AmbiguousRedirection { .. }),
    ));

    let command = [std::ffi::OsString::from("true")];
    assert!(matches!(
        super::run(&command, &cfg, |_notification| {}),
        Err(ExecError::AmbiguousRedirection { .. }),
    ));

    assert_eq!(
        std::fs::read(&real).expect("read target"),
        b"previous log contents",
        "usage failure must precede any truncation"
    );
}

struct SharedSink {
    inner: Arc<Mutex<Vec<u8>>>,
}

impl Write for SharedSink {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut guard = self.inner.lock().expect("test sink lock");
        Write::write(&mut *guard, buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn make_writer(
    add_prefix: bool,
    props: StreamProps,
    wrap_length: usize,
) -> (Writer, Arc<Mutex<Vec<u8>>>) {
    let buffer = Arc::new(Mutex::new(Vec::<u8>::new()));
    let shared = Arc::clone(&buffer);
    let sink = SharedSink { inner: shared };
    let writer = Writer::new(sink, add_prefix, wrap_length, props, false);
    (writer, buffer)
}

#[test]
fn typesetter_passes_data_through() {
    let (mut writer, buffer) = make_writer(false, StreamProps::default(), 80);
    writer
        .write(Stream::Stdout, b"hello\nworld\n")
        .expect("write");
    writer.finalize().expect("finalize");
    let written = buffer.lock().expect("lock").clone();
    assert_eq!(&*written, b"hello\nworld\n");
}

#[test]
fn typesetter_preserves_arbitrary_bytes() {
    // Raw mode is byte-exact: invalid UTF-8, NUL, control bytes, and
    // large chunks pass through unchanged.
    let (mut writer, buffer) = make_writer(false, StreamProps::default(), 80);

    let mut payload: Vec<u8> = vec![0xff, 0xfe, 0x00, 0x01, 0x1b, b'[', b'3', b'1', b'm'];
    payload.extend(std::iter::repeat_n(0xa5_u8, 64 * 1024));
    payload.push(b'\n');

    writer.write(Stream::Stdout, &payload).expect("write");
    writer.write(Stream::Stderr, &payload).expect("write");
    writer.finalize().expect("finalize");

    let written = buffer.lock().expect("lock").clone();
    let mut expected = payload.clone();
    expected.extend_from_slice(&payload);
    assert_eq!(written, expected);
}

#[test]
fn calligraphic_sanitizes_control_bytes() {
    let props = StreamProps {
        line1_prefix_stdout: ">>> ".to_string(),
        ..StreamProps::default()
    };
    let (mut writer, buffer) = make_writer(true, props, 80);
    writer
        .write(Stream::Stdout, b"a\x00b\xffc\n")
        .expect("write");
    writer.finalize().expect("finalize");
    let written = String::from_utf8(buffer.lock().expect("lock").clone()).expect("utf8");
    assert_eq!(written, ">>> a?b\u{fffd}c\n");
}

#[test]
fn calligraphic_adds_prefix_per_line() {
    let props = StreamProps {
        line1_prefix_stdout: ">>> ".to_string(),
        wrap_prefix_stdout: "    ".to_string(),
        ..StreamProps::default()
    };
    let (mut writer, buffer) = make_writer(true, props, 80);
    writer
        .write(Stream::Stdout, b"line one\nline two\n")
        .expect("write");
    writer.finalize().expect("finalize");
    let written = String::from_utf8(buffer.lock().expect("lock").clone()).expect("utf8");
    assert_eq!(written, ">>> line one\n>>> line two\n");
}

#[test]
fn calligraphic_buffers_partial_lines_until_finalize() {
    let props = StreamProps {
        line1_prefix_stdout: ">>> ".to_string(),
        ..StreamProps::default()
    };
    let (mut writer, buffer) = make_writer(true, props, 80);
    writer.write(Stream::Stdout, b"partial").expect("write");
    assert!(buffer.lock().expect("lock").is_empty());
    writer.finalize().expect("finalize");
    let written = String::from_utf8(buffer.lock().expect("lock").clone()).expect("utf8");
    assert_eq!(written, ">>> partial\n");
}

fn prefix_props() -> StreamProps {
    StreamProps {
        line1_prefix_stdout: ">>> ".to_string(),
        ..StreamProps::default()
    }
}

#[test]
fn calligraphic_preserves_code_point_split_across_two_writes() {
    // "€\n" is E2 82 AC 0A; a pipe read may deliver any split of it.
    let (mut writer, buffer) = make_writer(true, prefix_props(), 80);
    writer.write(Stream::Stdout, &[0xe2]).expect("write");
    writer
        .write(Stream::Stdout, &[0x82, 0xac, b'\n'])
        .expect("write");
    writer.finalize().expect("finalize");
    let written = String::from_utf8(buffer.lock().expect("lock").clone()).expect("utf8");
    assert_eq!(written, ">>> \u{20ac}\n");
}

#[test]
fn calligraphic_preserves_code_point_split_across_three_writes() {
    let (mut writer, buffer) = make_writer(true, prefix_props(), 80);
    for byte in [0xe2_u8, 0x82, 0xac, b'\n'] {
        writer.write(Stream::Stdout, &[byte]).expect("write");
    }
    writer.finalize().expect("finalize");
    let written = String::from_utf8(buffer.lock().expect("lock").clone()).expect("utf8");
    assert_eq!(written, ">>> \u{20ac}\n");
}

#[test]
fn calligraphic_replaces_invalid_utf8_after_buffering() {
    // A lone continuation byte and a truncated sequence at end of
    // stream are genuinely invalid and must still become U+FFFD.
    let (mut writer, buffer) = make_writer(true, prefix_props(), 80);
    writer.write(Stream::Stdout, &[b'a', 0xac]).expect("write");
    writer.write(Stream::Stdout, b"b\n").expect("write");
    writer.write(Stream::Stdout, &[0xe2, 0x82]).expect("write");
    writer.finalize().expect("finalize");
    let written = String::from_utf8(buffer.lock().expect("lock").clone()).expect("utf8");
    assert_eq!(written, ">>> a\u{fffd}b\n>>> \u{fffd}\n");
}

#[test]
fn calligraphic_output_is_chunking_invariant() {
    // The same byte stream must render identically no matter how the
    // OS chunks the pipe reads.
    let payload = "una línea con € y 日本語\nsecond line\ntail without newline"
        .as_bytes()
        .to_vec();

    let mut renderings = Vec::new();
    for chunk_size in [1_usize, 2, 3, 5, 7, payload.len()] {
        let (mut writer, buffer) = make_writer(true, prefix_props(), 80);
        for chunk in payload.chunks(chunk_size) {
            writer.write(Stream::Stdout, chunk).expect("write");
        }
        writer.finalize().expect("finalize");
        renderings.push(buffer.lock().expect("lock").clone());
    }

    let first = renderings[0].clone();
    assert!(
        renderings.iter().all(|rendering| *rendering == first),
        "output depends on write chunking"
    );
    let text = String::from_utf8(first).expect("utf8");
    assert!(text.contains('\u{20ac}'));
    assert!(!text.contains('\u{fffd}'));
}
