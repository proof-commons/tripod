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
use target_elements_conformance::executor::{
    ExecutorConfiguration, ExecutorDiagnostics, ExecutorTrust, execute,
};
use target_elements_conformance::fixture::{
    ExpectedPrimitiveOutcome, NativeCaseGroup, NativeCaseId, PrimitiveFixture, PrimitiveFixtureSet,
};
use target_elements_conformance::protocol::{MOCK_EXECUTOR_GENESIS_ID, MOCK_EXECUTOR_NETWORK_ID};

/// How long the run may take before the tree is stopped.
///
/// A run that has to set up a live descendant first may be given a
/// multiple of this; `run_until_set_up` says why.
const TIMEOUT: Duration = Duration::from_millis(400);

/// How long the tree has to clean up before it is killed, where the
/// executor does not clean up: one that ignores the graceful signal, or
/// one already gone.
///
/// An executor that ignores the signal never exits on its own, so the
/// harness waits this out in full by design; it is kept short because
/// nothing is gained by lengthening it.
const GRACE: Duration = Duration::from_millis(400);

/// How long an executor that does clean up has to finish before it is
/// killed.
///
/// The harness stops waiting the moment the executor exits: its cleanup
/// loop polls `SupervisedGroup::leader_exited`, a non-reaping wait on the
/// direct child, and sends the forceful signal as soon as that answers
/// (`ExecutorSupervisor::terminate` in `src/executor.rs`). So this bound
/// costs nothing on an idle host and is an honest bound on a busy one,
/// where the executor's shell still has to be scheduled to start its own
/// cleanup command. Only a cleanup still unfinished ten seconds after
/// the graceful signal can fail it; like the disappearance budget, the
/// bound is the host's, not a property of the harness. Where the host
/// offers no non-reaping wait, `leader_exited` answers false and the
/// bound is waited out in full.
const CLEANUP_BUDGET: Duration = Duration::from_secs(10);

/// How long a stopped descendant may take to disappear.
///
/// Not a property of the harness: once the group has been signalled,
/// the host still has to schedule the dying processes and their reaper.
const DISAPPEARANCE_BUDGET: Duration = Duration::from_secs(10);

/// How many runs a test makes before it concludes that none of them can
/// show its scenario.
const SETUP_ATTEMPTS: u32 = 5;

/// How long a finished run has to show its descendant's record.
///
/// Every run is judged for survivors whether or not its descendant
/// recorded itself (`judge_every_run`), so this wait decides only
/// whether a run is repeated, never a verdict: a record that appears
/// after it costs one more run, not a missed survivor.
const RECORD_SETTLE: Duration = Duration::from_secs(1);

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

/// Where an executor records its own identifier, which is also its
/// process group's, before it does anything else.
///
/// The harness spawns the script itself as the executor and makes it
/// lead a new process group (`spawn_executor` in `src/executor.rs`,
/// through `CommandExt::process_group(0)`), and it refuses a run whose
/// child does not lead its own group (`SupervisedGroup::establish`). The
/// interpreter named by the script's first line runs in that same
/// process, so the shell's `$$` is the group's identifier.
fn leader_marker(directory: &Path) -> PathBuf {
    directory.join("leader.pid")
}

/// The identifier a shell wrote with `echo $$`, once its line is
/// complete.
///
/// The shell creates the file before it writes to it, and a process
/// stopped in between leaves a file that will never be filled.
fn complete_record(marker: &Path) -> Option<i32> {
    let text = std::fs::read_to_string(marker).ok()?;
    text.strip_suffix('\n')?.parse().ok()
}

/// Reads the identifier a descendant recorded for itself, if it did.
fn recorded_descendant(marker: &Path) -> Option<i32> {
    let deadline = Instant::now() + RECORD_SETTLE;
    loop {
        if let Some(pid) = complete_record(marker) {
            return Some(pid);
        }
        if Instant::now() >= deadline {
            return None;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// One row of the host's process table, as far as these tests read it.
struct ProcessRow {
    pid: i32,
    zombie: bool,
    parent: Option<u32>,
    group: Option<i32>,
}

/// The host's process table, read from `/proc`; empty where there is
/// none.
fn process_table() -> Vec<ProcessRow> {
    let Ok(entries) = std::fs::read_dir("/proc") else {
        return Vec::new();
    };
    entries
        .flatten()
        .filter_map(|entry| {
            let pid = entry.file_name().to_str()?.parse::<i32>().ok()?;
            let stat = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
            // The fields after the parenthesised command name, which may
            // itself contain spaces or parentheses, are the state letter,
            // the parent's identifier and the process group's.
            let (_, rest) = stat.rsplit_once(") ")?;
            let mut fields = rest.split_whitespace();
            let zombie = fields.next() == Some("Z");
            let parent = fields.next().and_then(|text| text.parse::<u32>().ok());
            let group = fields.next().and_then(|text| text.parse::<i32>().ok());
            Some(ProcessRow {
                pid,
                zombie,
                parent,
                group,
            })
        })
        .collect()
}

/// Waits for a process group to have no running member, and returns
/// the members still running once the disappearance budget is spent.
///
/// A member that has exited but is not yet collected is not running,
/// for the reason `is_running` gives. The group's identifier cannot
/// name another group while a member of this one exists.
fn waited_for_empty_group(group: i32) -> Vec<i32> {
    let deadline = Instant::now() + DISAPPEARANCE_BUDGET;
    loop {
        let running: Vec<i32> = process_table()
            .into_iter()
            .filter(|row| !row.zombie && row.group == Some(group))
            .map(|row| row.pid)
            .collect();
        if running.is_empty() || Instant::now() >= deadline {
            return running;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// Runs one shell executor to the given timeout and cleanup grace.
fn run(
    program: &Path,
    timeout: Duration,
    grace: Duration,
) -> (Result<(), NativeConformanceError>, Duration) {
    let configuration = ExecutorConfiguration::new(
        program,
        ExecutorTrust::Mock,
        timeout,
        ExecutorDiagnostics::in_directory(
            program
                .parent()
                .expect("the executor sits in the test directory"),
        ),
    )
    .with_cleanup_grace(grace);
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
    process_table()
        .into_iter()
        .filter(|row| row.zombie && row.parent == Some(ours))
        .map(|row| row.pid)
        .collect()
}

/// What every run shows, whether or not its descendant recorded itself.
///
/// None of these executors answers, so a run that started ends in its
/// timeout, and the harness's clock cannot reach the timeout early: the
/// watchdog reports expiry only once its whole duration has passed, and
/// this side's clock started before the watchdog's. A run whose spawn
/// the host refused started nothing. Whatever the run set up, nothing of
/// its process group may remain running: a descendant started too late
/// to record itself is still a member.
fn judge_every_run(
    outcome: &Result<(), NativeConformanceError>,
    elapsed: Duration,
    timeout: Duration,
    directory: &Path,
    this_run: &str,
) {
    match outcome {
        Err(NativeConformanceError::ExecutorTimeout) => assert!(
            elapsed >= timeout,
            "the tree was stopped before its timeout ({this_run})",
        ),
        Err(NativeConformanceError::ExecutorStartupFailed) => {}
        other => panic!("a run that never answers ends in its timeout, got {other:?} ({this_run})"),
    }
    // A leader that never recorded itself never reached the line that
    // starts the descendant, so its run had no descendant to leave.
    if let Some(leader) = complete_record(&leader_marker(directory)) {
        let running = waited_for_empty_group(leader);
        assert!(
            running.is_empty(),
            "processes {running:?} of group {leader} outlived the run ({this_run})",
        );
    }
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

    let (outcome, _elapsed) = run(&program, TIMEOUT, GRACE);
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
/// with the harness still waiting to read. Like every executor here that
/// starts a descendant, it first records itself at `leader_marker`, so
/// that every run can be checked for a surviving member of its group.
fn stubborn_tree(directory: &Path, marker: &Path) -> PathBuf {
    script(
        directory,
        "stubborn-tree",
        &format!(
            "echo $$ > {leader}\n\
             trap '' TERM\n\
             sh -c 'trap \"\" TERM; echo $$ > {marker}; while : ; do sleep 1; done' &\n\
             while : ; do sleep 1; done\n",
            leader = leader_marker(directory).display(),
            marker = marker.display(),
        ),
    )
}

/// An executor in the shape the reviewed adapter takes: it starts a
/// descendant and removes what it created on the way out. The graceful
/// signal is what makes that exit path reachable at all, which is why it
/// is sent before the forceful one.
///
/// The trap is set on the line before the descendant starts, and the
/// shell runs the lines in order, so a run whose descendant recorded
/// itself is one whose executor had its cleanup in place.
fn tidy_tree(directory: &Path, marker: &Path) -> PathBuf {
    let datadir = tidy_datadir(directory);
    std::fs::create_dir(&datadir).expect("create the executor's temporary directory");
    script(
        directory,
        "tidy-tree",
        &format!(
            "echo $$ > {leader}\n\
             trap 'rm -rf {datadir}; exit 0' TERM\n\
             sh -c 'echo $$ > {marker}; while : ; do sleep 1; done' &\n\
             while : ; do sleep 1; done\n",
            leader = leader_marker(directory).display(),
            datadir = datadir.display(),
            marker = marker.display(),
        ),
    )
}

/// The directory the tidy executor removes on its way out.
fn tidy_datadir(directory: &Path) -> PathBuf {
    directory.join("node-datadir")
}

/// One run whose executor's descendant recorded itself: the only kind of
/// run the scenario's own assertions judge.
///
/// The record shows the descendant started, not when: an executor that
/// ignores the graceful signal can start it during the cleanup grace.
struct SetUpRun {
    /// The run's own directory, kept while the test inspects it.
    directory: tempfile::TempDir,
    outcome: Result<(), NativeConformanceError>,
    /// The timeout this run was given.
    timeout: Duration,
    /// How long the harness took to return from this run.
    elapsed: Duration,
    /// The identifier the descendant recorded for itself.
    descendant: i32,
    /// This run and every earlier one, for the failure messages.
    context: String,
}

/// Runs an executor that starts a self-recording descendant until one
/// run has actually set that scenario up.
///
/// The timeout is the harness's own clock, started once the spawned
/// child is under supervision, and nothing the script does can delay
/// it: reordering the script only moves the window. On a busy host the
/// timeout can therefore arrive before the shell has started its
/// descendant or before the descendant has recorded itself, and the
/// host can refuse the spawn outright. Such a run cannot show the
/// scenario, so the scenario's assertions are not made on it; what any
/// run can show is judged on every run (`judge_every_run`), and the run
/// is repeated in a fresh directory with twice the timeout, a bounded
/// number of times.
///
/// Lengthening the timeout changes nothing the tests assert. The error
/// kind, the executor's own cleanup and the descendant's disappearance
/// are owed at any length; a longer one only gives the host more time to
/// set the scenario up. A test none of whose runs was set up fails
/// listing every run's outcome, so a refused spawn and a late shell can
/// be told apart.
///
/// The cleanup grace is the caller's, because it belongs to the kind of
/// executor: `GRACE` for one that ignores the graceful signal, and
/// `CLEANUP_BUDGET` for one whose own cleanup is under test.
fn run_until_set_up(executor: fn(&Path, &Path) -> PathBuf, grace: Duration) -> SetUpRun {
    let mut earlier: Vec<String> = Vec::new();
    for attempt in 1..=SETUP_ATTEMPTS {
        let timeout = TIMEOUT * 2_u32.pow(attempt - 1);
        let directory = tempfile::tempdir().expect("tempdir");
        let marker = directory.path().join("descendant.pid");
        let program = executor(directory.path(), &marker);
        let (outcome, elapsed) = run(&program, timeout, grace);
        let this_run = format!(
            "run {attempt} of at most {SETUP_ATTEMPTS}, with a {timeout:?} timeout and a \
             {grace:?} cleanup grace, ended {outcome:?} after {elapsed:?}"
        );
        judge_every_run(&outcome, elapsed, timeout, directory.path(), &this_run);
        if let Some(descendant) = recorded_descendant(&marker) {
            let context = if earlier.is_empty() {
                this_run
            } else {
                let earlier = earlier.join("; ");
                format!("{this_run}; before it, without a descendant record: {earlier}")
            };
            return SetUpRun {
                directory,
                outcome,
                timeout,
                elapsed,
                descendant,
                context,
            };
        }
        earlier.push(this_run);
    }
    let earlier = earlier.join("; ");
    panic!("no run had its descendant record itself, so none could show the scenario: {earlier}");
}

#[test]
fn a_descendant_ignoring_graceful_termination_does_not_survive_the_timeout() {
    let set_up = run_until_set_up(stubborn_tree, GRACE);
    let context = &set_up.context;

    let Err(error) = set_up.outcome else {
        panic!("a run that never answers is refused ({context})");
    };
    assert!(
        matches!(error, NativeConformanceError::ExecutorTimeout),
        "a timeout must be its own failure, got {error} ({context})",
    );

    let descendant = set_up.descendant;
    assert!(
        waited_for_exit(descendant),
        "descendant {descendant} outlived the run it belonged to ({context})",
    );
}

#[test]
fn a_descendant_holding_stdout_cannot_hang_the_harness() {
    let set_up = run_until_set_up(stubborn_tree, GRACE);
    let context = &set_up.context;

    // The bound is what the configuration itself allows: the run, the
    // cleanup interval, and room for the host to schedule the reap. A
    // harness that waited on the inherited pipe would not return at all,
    // so the assertion is about returning within the contract rather
    // than about being quick.
    let ceiling = set_up.timeout + GRACE + Duration::from_secs(20);
    let elapsed = set_up.elapsed;
    assert!(
        set_up.outcome.is_err(),
        "a run that never answers is refused ({context})",
    );
    assert!(
        elapsed < ceiling,
        "the harness took {elapsed:?}, past the {ceiling:?} its own timeout and cleanup allow \
         ({context})",
    );
}

#[test]
fn a_descendant_that_exits_gracefully_gets_the_chance_to_clean_up() {
    // The executor exits once its cleanup is done, so the long grace is
    // waited only for as long as that takes; a short one would turn a
    // shell the host scheduled late into a missing cleanup.
    let set_up = run_until_set_up(tidy_tree, CLEANUP_BUDGET);
    let context = &set_up.context;

    let Err(error) = &set_up.outcome else {
        panic!("a run that never answers is refused ({context})");
    };
    assert!(
        matches!(error, NativeConformanceError::ExecutorTimeout),
        "a timeout must be its own failure, got {error} ({context})",
    );
    assert!(
        !tidy_datadir(set_up.directory.path()).exists(),
        "the graceful signal did not reach the executor's own cleanup ({context})",
    );

    let descendant = set_up.descendant;
    assert!(
        waited_for_exit(descendant),
        "descendant {descendant} outlived the run it belonged to ({context})",
    );
}
