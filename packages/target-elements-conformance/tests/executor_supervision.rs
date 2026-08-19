//! Timeout cleanup against a real executor tree.
//!
//! The cases here are the ones a mock cannot pose. A mock is one
//! process that answers or does not; a real executor adapter starts a
//! node, and an arbitrary caller-selected executor may start anything.
//! What the harness promises on timeout is that the run is over — no
//! descendant left running, no inherited pipe left open, and no wait of
//! its own left outstanding — and only a child that actually forks,
//! actually holds stdout, and actually ignores a graceful signal can
//! show whether that promise is kept (Guide-10 §5.8).
//!
//! The executors here are shell scripts rather than compiled binaries
//! because that is the shape the residual takes in practice: the
//! reviewed adapter is a script that starts a node and cleans it up in
//! its own exit path.

#![cfg(unix)]

use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use tapscript::{StackItem, TapscriptInstruction, TapscriptProgram};
use target_elements::{
    ActivationDeclaration, DeploymentEnvironment, DevelopmentDeploymentBinding, LeafVersion,
    OpcodeId, ReviewedDevelopmentBinding, ReviewedElementsTapscriptDefinition,
    TargetContractVersion, reviewed_elements_tapscript, validate_reviewed_development_binding,
};
use target_elements_conformance::error::NativeConformanceError;
use target_elements_conformance::executor::{ExecutorConfiguration, ExecutorTrust, execute};
use target_elements_conformance::fixture::{
    ExpectedPrimitiveOutcome, NativeCaseGroup, NativeCaseId, PrimitiveFixture, PrimitiveFixtureSet,
};
use target_elements_conformance::protocol::{MOCK_EXECUTOR_GENESIS_ID, MOCK_EXECUTOR_NETWORK_ID};

/// How long the run may take before the tree is stopped.
const TIMEOUT: Duration = Duration::from_millis(400);

/// How long the tree has to clean up before it is killed.
const GRACE: Duration = Duration::from_millis(400);

/// How long a stopped descendant may take to disappear.
///
/// Not a property of the harness: once the group has been signalled,
/// the host still has to schedule the dying processes and their reaper.
const DISAPPEARANCE_BUDGET: Duration = Duration::from_secs(10);

fn reviewed_target() -> ReviewedElementsTapscriptDefinition {
    reviewed_elements_tapscript().expect("the reviewed contract validates")
}

fn development_binding(target: &ReviewedElementsTapscriptDefinition) -> ReviewedDevelopmentBinding {
    let binding = DevelopmentDeploymentBinding::new(
        TargetContractVersion::V2,
        DeploymentEnvironment::Development,
        MOCK_EXECUTOR_NETWORK_ID,
        MOCK_EXECUTOR_GENESIS_ID,
        ActivationDeclaration::new(true, LeafVersion::TAPSCRIPT, []),
        None,
    );
    validate_reviewed_development_binding(target, binding).expect("the binding validates")
}

/// One fixture: none of these executors ever answers it.
fn fixtures() -> PrimitiveFixtureSet {
    let target = reviewed_target();
    let binding = development_binding(&target);
    let program = TapscriptProgram::new(vec![TapscriptInstruction::Opcode(OpcodeId::Add64)])
        .expect("the program is within the work limit");
    let stack = [
        StackItem::signed_le64(&target, 2),
        StackItem::signed_le64(&target, 3),
    ];
    let fixture = PrimitiveFixture::new(
        &target,
        &binding,
        NativeCaseId::new(NativeCaseGroup::Arithmetic, Some(OpcodeId::Add64), 0),
        &program,
        &stack,
        None,
        ExpectedPrimitiveOutcome::accept(Some(vec![
            StackItem::signed_le64(&target, 5).bytes().to_vec(),
        ])),
    )
    .expect("the reviewed domain has a wire spelling");
    PrimitiveFixtureSet::new([fixture]).expect("one case")
}

/// Writes one executable shell executor.
fn script(directory: &Path, name: &str, body: &str) -> PathBuf {
    let path = directory.join(format!("{name}.sh"));
    let mut file = std::fs::File::create(&path).expect("create executor");
    write!(file, "#!/bin/sh\n{body}").expect("write executor");
    let mut permissions = file.metadata().expect("metadata").permissions();
    permissions.set_mode(0o755);
    file.set_permissions(permissions).expect("set mode");
    drop(file);
    path
}

/// Whether a process identifier still names a running process.
///
/// A process that has exited but not yet been collected is not running.
/// It is reported separately from a live one because a run whose
/// descendants are all dead but momentarily uncollected has kept the
/// cleanup contract, and a test that could not tell the two apart would
/// depend on whichever process happens to reap orphans on the host.
fn is_running(pid: i32) -> bool {
    let alive = nix::sys::signal::kill(nix::unistd::Pid::from_raw(pid), None).is_ok();
    if !alive {
        return false;
    }
    let Ok(stat) = std::fs::read_to_string(format!("/proc/{pid}/stat")) else {
        // No process table to read: the signal answer is all there is.
        return true;
    };
    // The state letter is the field after the parenthesised command
    // name, which may itself contain spaces or parentheses.
    stat.rsplit_once(") ")
        .is_none_or(|(_, rest)| !rest.starts_with('Z'))
}

/// Waits for a descendant to disappear, and says whether it did.
fn waited_for_exit(pid: i32) -> bool {
    let deadline = Instant::now() + DISAPPEARANCE_BUDGET;
    while Instant::now() < deadline {
        if !is_running(pid) {
            return true;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    !is_running(pid)
}

/// Reads the identifier a descendant recorded for itself.
fn recorded_descendant(marker: &Path) -> i32 {
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        if let Ok(text) = std::fs::read_to_string(marker)
            && let Ok(pid) = text.trim().parse::<i32>()
        {
            return pid;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    panic!("the executor never recorded a descendant");
}

/// Runs one shell executor to its timeout.
fn run(program: &Path) -> (Result<(), NativeConformanceError>, Duration) {
    let configuration =
        ExecutorConfiguration::new(program, ExecutorTrust::Mock, TIMEOUT).with_cleanup_grace(GRACE);
    let target = reviewed_target();
    let binding = development_binding(&target);
    let started = Instant::now();
    let outcome = execute(&target, &binding, &configuration, &fixtures());
    (outcome.map(|_| ()), started.elapsed())
}

/// Every zombie child this process currently has.
///
/// Read from the process table rather than waited for: a `wait` would
/// collect the very evidence the assertion is about, and would also
/// collect children belonging to other tests running beside this one.
fn zombie_children() -> Vec<i32> {
    let ours = std::process::id();
    let mut zombies = Vec::new();
    let Ok(entries) = std::fs::read_dir("/proc") else {
        return zombies;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(pid) = name.to_str().and_then(|text| text.parse::<i32>().ok()) else {
            continue;
        };
        let Ok(stat) = std::fs::read_to_string(format!("/proc/{pid}/stat")) else {
            continue;
        };
        // The fields after the parenthesised command name are the state
        // letter and then the parent's identifier.
        let Some((_, rest)) = stat.rsplit_once(") ") else {
            continue;
        };
        let mut fields = rest.split_whitespace();
        let state = fields.next().unwrap_or_default();
        let parent = fields.next().and_then(|text| text.parse::<u32>().ok());
        if state == "Z" && parent == Some(ours) {
            zombies.push(pid);
        }
    }
    zombies
}

/// `G11-R13`: a run refused at startup leaves no unreaped child.
///
/// The executor here exits the instant it starts, which is the common
/// shape of the race the finding names: the harness has spawned a
/// process that is already gone by the time anything is established.
/// However the run is refused — and it is refused, since nothing
/// answered the handshake — the process this harness started must have
/// been both stopped and collected.
///
/// The bounded wait is for the host, not for the harness: a child that
/// has been reaped is gone immediately, while one that was leaked stays
/// in the table for as long as this process lives, so the loop
/// distinguishes them without depending on scheduling.
#[test]
fn a_run_refused_at_startup_leaves_no_unreaped_child() {
    let directory = tempfile::tempdir().expect("tempdir");
    let program = script(directory.path(), "exits-at-once", "exit 0\n");

    let (outcome, _elapsed) = run(&program);
    assert!(
        outcome.is_err(),
        "an executor that answers nothing is refused",
    );

    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let zombies = zombie_children();
        if zombies.is_empty() {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "the run left unreaped children behind: {zombies:?}",
        );
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// An executor that ignores graceful termination, forks a descendant
/// that does the same, and lets that descendant inherit stdout.
///
/// Nothing here answers the protocol, so the run reaches its timeout
/// with the harness still waiting to read.
fn stubborn_tree(directory: &Path, marker: &Path) -> PathBuf {
    let marker = marker.display();
    script(
        directory,
        "stubborn-tree",
        &format!(
            "trap '' TERM\n\
             sh -c 'trap \"\" TERM; echo $$ > {marker}; while : ; do sleep 1; done' &\n\
             while : ; do sleep 1; done\n"
        ),
    )
}

#[test]
fn a_descendant_ignoring_graceful_termination_does_not_survive_the_timeout() {
    let directory = tempfile::tempdir().expect("tempdir");
    let marker = directory.path().join("descendant.pid");
    let program = stubborn_tree(directory.path(), &marker);

    let (outcome, _elapsed) = run(&program);
    let error = outcome.expect_err("a run that never answers is refused");
    assert!(
        matches!(error, NativeConformanceError::ExecutorTimeout),
        "a timeout must be its own failure, got {error}",
    );

    let descendant = recorded_descendant(&marker);
    assert!(
        waited_for_exit(descendant),
        "descendant {descendant} outlived the run it belonged to",
    );
}

#[test]
fn a_descendant_holding_stdout_cannot_hang_the_harness() {
    let directory = tempfile::tempdir().expect("tempdir");
    let marker = directory.path().join("descendant.pid");
    let program = stubborn_tree(directory.path(), &marker);

    // The bound is what the configuration itself allows: the run, the
    // cleanup interval, and room for the host to schedule the reap. A
    // harness that waited on the inherited pipe would not return at all,
    // so the assertion is about returning within the contract rather
    // than about being quick.
    let ceiling = TIMEOUT + GRACE + Duration::from_secs(20);
    let (outcome, elapsed) = run(&program);
    assert!(outcome.is_err(), "a run that never answers is refused");
    assert!(
        elapsed < ceiling,
        "the harness took {elapsed:?}, past the {ceiling:?} its own timeout and cleanup allow",
    );
}

#[test]
fn a_descendant_that_exits_gracefully_gets_the_chance_to_clean_up() {
    let directory = tempfile::tempdir().expect("tempdir");
    let marker = directory.path().join("descendant.pid");
    let datadir = directory.path().join("node-datadir");
    std::fs::create_dir(&datadir).expect("create the executor's temporary directory");
    // This executor is the shape the reviewed adapter takes: it starts
    // a descendant and removes what it created on the way out. The
    // graceful signal is what makes that exit path reachable at all,
    // which is why it is sent before the forceful one.
    let program = script(
        directory.path(),
        "tidy-tree",
        &format!(
            "trap 'rm -rf {datadir}; exit 0' TERM\n\
             sh -c 'echo $$ > {marker}; while : ; do sleep 1; done' &\n\
             while : ; do sleep 1; done\n",
            datadir = datadir.display(),
            marker = marker.display(),
        ),
    );

    let descendant = {
        let (outcome, _elapsed) = run(&program);
        assert!(outcome.is_err(), "a run that never answers is refused");
        recorded_descendant(&marker)
    };

    assert!(
        !datadir.exists(),
        "the graceful signal did not reach the executor's own cleanup",
    );
    assert!(
        waited_for_exit(descendant),
        "descendant {descendant} outlived the run it belonged to",
    );
}
