//! ADR-010 control-plane subprocess coverage for the labels binaries.
//!
//! These run the shipped executables as real subprocesses and pin the uniform
//! control-plane contract: `--help`/`--version` exit 0 with no stdout noise, a
//! missing or unknown argument is usage class 2, and every control-plane
//! record is one JSON object on stderr while stdout stays empty.
//!
//! TTY-refusal behaviour is covered by the PTY harness below (F1-024): a
//! direct-mode checker refuses a terminal stdout before doing any semantic
//! work, while build mode (with an explicit report/stamp destination) does not.

use std::process::{Command, Output};

const BINARIES: [&str; 6] = [
    env!("CARGO_BIN_EXE_check-labels"),
    env!("CARGO_BIN_EXE_check-plans"),
    env!("CARGO_BIN_EXE_census-audit"),
    env!("CARGO_BIN_EXE_check-forbidden-text"),
    env!("CARGO_BIN_EXE_check-hash-citations"),
    env!("CARGO_BIN_EXE_generate-label-registers"),
];

fn run(binary: &str, args: &[&str]) -> Output {
    Command::new(binary)
        .args(args)
        .output()
        .expect("binary runs")
}

fn assert_json_only_stderr(output: &Output) {
    assert!(
        output.stdout.is_empty(),
        "a control-plane result must not write stdout"
    );
    let stderr = String::from_utf8(output.stderr.clone()).expect("stderr is utf8");
    let mut saw_record = false;
    for line in stderr.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let value: serde_json::Value =
            serde_json::from_str(line).expect("every stderr line is one JSON object");
        assert!(value.is_object(), "each control-plane record is an object");
        saw_record = true;
    }
    assert!(
        saw_record,
        "expected at least one JSON control-plane record"
    );
}

#[test]
fn missing_arguments_are_usage_errors() {
    for binary in BINARIES {
        let output = run(binary, &[]);
        assert_eq!(
            output.status.code(),
            Some(2),
            "{binary}: no arguments must be usage class 2",
        );
        assert_json_only_stderr(&output);
    }
}

#[test]
fn check_plans_without_a_subject_is_a_usage_error() {
    // ADR-014 (F3-004): the build system states census membership by
    // argument. An invocation that names a repository root but no
    // subject must be refused as usage class 2, never run with the
    // on-disk walk as the effective census source.
    let output = run(
        env!("CARGO_BIN_EXE_check-plans"),
        &["--repository-root", "."],
    );
    assert_eq!(
        output.status.code(),
        Some(2),
        "check-plans with no --subject must be usage class 2",
    );
    assert_json_only_stderr(&output);
}

#[test]
fn unknown_flag_is_a_usage_error() {
    for binary in BINARIES {
        let output = run(binary, &["--definitely-not-a-real-flag"]);
        assert_eq!(
            output.status.code(),
            Some(2),
            "{binary}: an unknown flag must be usage class 2",
        );
        assert_json_only_stderr(&output);
    }
}

#[test]
fn help_and_version_exit_zero_with_empty_stdout() {
    for binary in BINARIES {
        for flag in ["--help", "--version"] {
            let output = run(binary, &[flag]);
            assert_eq!(
                output.status.code(),
                Some(0),
                "{binary} {flag}: must exit 0",
            );
            assert!(
                output.stdout.is_empty(),
                "{binary} {flag}: help/version is JSON on stderr, not stdout",
            );
        }
    }
}

/// The git program is argument-supplied, so its stderr is arbitrary
/// child output. A failing git must surface only the process status and
/// a generic message — never the raw child stderr (F1-032, ADR-010).
#[cfg(unix)]
#[test]
fn census_audit_omits_untrusted_git_stderr() {
    use std::io::Write;
    use std::os::unix::fs::PermissionsExt;

    let secret = "SHOULD_NOT_APPEAR_git_stderr_secret";

    let dir = tempfile::tempdir().expect("tempdir");
    let script = dir.path().join("fake-git.sh");
    let mut file = std::fs::File::create(&script).expect("create script");
    writeln!(file, "#!/bin/sh").unwrap();
    writeln!(file, "echo '{secret}' >&2").unwrap();
    writeln!(file, "exit 17").unwrap();
    let mut perms = file.metadata().unwrap().permissions();
    perms.set_mode(0o755);
    file.set_permissions(perms).unwrap();
    drop(file);

    let output = run(
        env!("CARGO_BIN_EXE_census-audit"),
        &[
            "--repository-root",
            dir.path().to_str().expect("utf8 path"),
            "--git",
            script.to_str().expect("utf8 path"),
            "--exclude-pattern",
            ".*",
        ],
    );

    assert_eq!(
        output.status.code(),
        Some(1),
        "a failing git program is a runtime failure, not usage",
    );
    assert_eq!(output.stdout, [] as [u8; 0]);

    let stderr = String::from_utf8(output.stderr.clone()).expect("stderr is utf8");
    assert!(
        !stderr.contains(secret),
        "untrusted git stderr leaked into diagnostics: {stderr}",
    );

    // Every emitted record is still one JSON object, and the fixed
    // failure message is present.
    assert_json_only_stderr(&output);
    assert!(
        stderr.contains("git ls-files failed"),
        "expected the generic git-failure message: {stderr}",
    );
}

/// `check-forbidden-text` also takes the git program by argument; a git
/// error (exit > 1) must surface only the status, never the raw child
/// stderr (ADR-010/ADR-015).
#[cfg(unix)]
#[test]
fn check_forbidden_text_omits_untrusted_git_stderr() {
    use std::io::Write;
    use std::os::unix::fs::PermissionsExt;

    let secret = "SHOULD_NOT_APPEAR_grep_stderr_secret";

    let dir = tempfile::tempdir().expect("tempdir");
    let script = dir.path().join("fake-git.sh");
    let mut file = std::fs::File::create(&script).expect("create script");
    writeln!(file, "#!/bin/sh").unwrap();
    writeln!(file, "echo '{secret}' >&2").unwrap();
    writeln!(file, "exit 2").unwrap();
    let mut perms = file.metadata().unwrap().permissions();
    perms.set_mode(0o755);
    file.set_permissions(perms).unwrap();
    drop(file);

    let output = run(
        env!("CARGO_BIN_EXE_check-forbidden-text"),
        &[
            "--repository-root",
            dir.path().to_str().expect("utf8 path"),
            "--git",
            script.to_str().expect("utf8 path"),
        ],
    );

    assert_eq!(
        output.status.code(),
        Some(1),
        "a failing git program is a runtime failure, not usage",
    );
    assert_eq!(output.stdout, [] as [u8; 0]);

    let stderr = String::from_utf8(output.stderr.clone()).expect("stderr is utf8");
    assert!(
        !stderr.contains(secret),
        "untrusted git stderr leaked into diagnostics: {stderr}",
    );

    assert_json_only_stderr(&output);
    assert!(
        stderr.contains("git grep failed"),
        "expected the generic git-failure message: {stderr}",
    );
}

// ---------------------------------------------------------------------------
// PTY-based TTY-refusal coverage (F1-024).
//
// Refusal depends on output *mode*, not binary identity: a direct-mode
// checker with a terminal stdout must refuse (exit 2, one tty_refusal
// record) before touching the filesystem, whereas build mode publishes a
// report/stamp and never refuses. `census-audit` exercises both.
// ---------------------------------------------------------------------------

#[cfg(unix)]
mod pty {
    use std::fs::File;
    use std::io::{Read, Seek, Write};
    use std::os::unix::fs::PermissionsExt;
    use std::process::{Command, ExitStatus, Stdio};

    /// Run `command` with its stdout attached to a real pseudo-terminal.
    /// Returns the child status, the bytes the child wrote to the PTY, and
    /// its captured stderr.
    fn run_with_terminal_stdout(mut command: Command) -> (ExitStatus, Vec<u8>, Vec<u8>) {
        let pty = nix::pty::openpty(None, None).expect("open pty");
        let mut master = File::from(pty.master);
        let slave = File::from(pty.slave);

        let stderr_file = tempfile::tempfile().expect("stderr tempfile");
        let mut stderr_reader = stderr_file.try_clone().expect("clone stderr");

        // The child gets its own dup of the slave; the parent then drops
        // its slave so the only remaining slave is the child's.
        let child_slave = slave.try_clone().expect("clone slave");
        let mut child = command
            .stdin(Stdio::null())
            .stdout(Stdio::from(child_slave))
            .stderr(Stdio::from(stderr_file))
            .spawn()
            .expect("command spawns");
        drop(slave);

        // Drain the master on a helper thread. In this sandbox the master
        // does not reliably report EIO/EOF once the slave closes, so the
        // thread may block on a final read; we collect via a channel with
        // a grace period and never join it, so the test cannot hang.
        let (sender, receiver) = std::sync::mpsc::channel::<Vec<u8>>();
        std::thread::spawn(move || {
            let mut buffer = [0_u8; 4096];
            loop {
                match master.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(read) => {
                        if sender.send(buffer[..read].to_vec()).is_err() {
                            break;
                        }
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {}
                    // EIO once the slave closes, or any other error: done.
                    Err(_) => break,
                }
            }
        });

        let status = child.wait().expect("child exits");

        let mut pty_stdout = Vec::new();
        while let Ok(chunk) = receiver.recv_timeout(std::time::Duration::from_millis(500)) {
            pty_stdout.extend_from_slice(&chunk);
        }

        stderr_reader.rewind().expect("rewind stderr");
        let mut stderr = Vec::new();
        stderr_reader.read_to_end(&mut stderr).expect("read stderr");

        (status, pty_stdout, stderr)
    }

    /// Parse the single JSON control-plane record of the given kind, if any.
    fn find_record(stderr: &[u8], kind: &str) -> Option<serde_json::Value> {
        let text = String::from_utf8(stderr.to_vec()).expect("stderr is utf8");
        for line in text.lines().filter(|line| !line.trim().is_empty()) {
            let value: serde_json::Value =
                serde_json::from_str(line).expect("every stderr line is one JSON object");
            if value["kind"] == kind {
                return Some(value);
            }
        }
        None
    }

    /// A fake git that emits an empty `ls-files -z` listing and succeeds.
    fn empty_git(dir: &std::path::Path) -> std::path::PathBuf {
        let script = dir.join("empty-git.sh");
        let mut file = File::create(&script).expect("create fake git");
        writeln!(file, "#!/bin/sh").unwrap();
        writeln!(file, "exit 0").unwrap();
        let mut perms = file.metadata().unwrap().permissions();
        perms.set_mode(0o755);
        file.set_permissions(perms).unwrap();
        script
    }

    fn census_audit() -> Command {
        Command::new(env!("CARGO_BIN_EXE_census-audit"))
    }

    #[test]
    fn census_audit_refuses_terminal_stdout_in_direct_mode() {
        let dir = tempfile::tempdir().expect("tempdir");

        let mut command = census_audit();
        command.args([
            "--repository-root",
            dir.path().to_str().unwrap(),
            // Never invoked: direct-mode refusal precedes the semantic
            // work, so the git program is irrelevant here.
            "--git",
            "/nonexistent/git",
            "--exclude-pattern",
            ".*",
        ]);

        let (status, pty_stdout, stderr) = run_with_terminal_stdout(command);

        assert_eq!(status.code(), Some(2), "terminal stdout must be refused");
        assert!(pty_stdout.is_empty(), "refusal must not write result data");

        let record = find_record(&stderr, "tty_refusal").expect("one tty_refusal record");
        assert_eq!(record["fields"]["stream"], "stdout");
        assert_eq!(record["fields"]["exit_code"], 2);
    }

    #[test]
    fn census_audit_build_mode_does_not_refuse_terminal_stdout() {
        let dir = tempfile::tempdir().expect("tempdir");
        let git = empty_git(dir.path());
        let report = dir.path().join("census.json");
        let stamp = dir.path().join("census.stamp");

        let mut command = census_audit();
        command.args([
            "--repository-root",
            dir.path().to_str().unwrap(),
            "--git",
            git.to_str().unwrap(),
            "--exclude-pattern",
            ".*",
            "--report",
            report.to_str().unwrap(),
            "--stamp",
            stamp.to_str().unwrap(),
        ]);

        let (status, pty_stdout, stderr) = run_with_terminal_stdout(command);

        // Empty tracked set + empty declared census = a valid audit, so
        // build mode publishes the report and stamp and exits 0 even
        // though stdout is a terminal.
        assert_eq!(
            status.code(),
            Some(0),
            "build mode must not refuse a terminal"
        );
        assert!(pty_stdout.is_empty(), "build mode leaves stdout empty");
        assert!(report.exists(), "build mode publishes the report");
        assert!(stamp.exists(), "build mode touches the stamp");
        assert!(
            find_record(&stderr, "tty_refusal").is_none(),
            "build mode must not emit a tty_refusal record",
        );
    }

    /// A fake git emitting one mode-bearing record for a tracked
    /// symlink. Using a stub keeps the wiring test independent of the
    /// host git's symlink handling; the parsing and policy are unit
    /// tested separately.
    fn symlink_git(dir: &std::path::Path) -> std::path::PathBuf {
        let script = dir.join("symlink-git.sh");
        let mut file = File::create(&script).expect("create fake git");
        writeln!(file, "#!/bin/sh").unwrap();
        writeln!(file, r"printf '120000 aaaa 0\tplans/alias.md\0'").unwrap();
        let mut perms = file.metadata().unwrap().permissions();
        perms.set_mode(0o755);
        file.set_permissions(perms).unwrap();
        script
    }

    #[test]
    fn census_audit_fails_on_a_tracked_symlink_naming_path_and_mode() {
        // ADR-017: repository shape has one central owner. The audit
        // rejects a non-blob entry even when the declared census
        // agrees, and names both the path and the rejected mode.
        let dir = tempfile::tempdir().expect("tempdir");
        let git = symlink_git(dir.path());
        let report = dir.path().join("census.json");
        let stamp = dir.path().join("census.stamp");

        let output = census_audit()
            .args([
                "--repository-root",
                dir.path().to_str().unwrap(),
                "--git",
                git.to_str().unwrap(),
                "--exclude-pattern",
                "^$",
                "--report",
                report.to_str().unwrap(),
                "--stamp",
                stamp.to_str().unwrap(),
                "plans/alias.md",
            ])
            .output()
            .expect("census-audit runs");

        assert_eq!(output.status.code(), Some(1), "a non-blob entry fails");
        assert!(!stamp.exists(), "a failed audit leaves no success stamp");

        let stderr = String::from_utf8(output.stderr).expect("utf-8 diagnostics");
        let record = stderr
            .lines()
            .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
            .find(|value| value["fields"]["mode"] == "120000")
            .expect("one mode-defect record");
        assert_eq!(record["fields"]["path"], "plans/alias.md");
    }
}
