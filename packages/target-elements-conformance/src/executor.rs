//! The external executor driver.
//!
//! # Selecting an executor is granting execution authority
//!
//! The program named by `--executor` runs with this process's authority
//! (ADR-015, ADR-017). This module starts it, speaks the protocol to it,
//! and stops it. It does not authenticate it, sandbox it, contain it, or
//! claim any of those.
//!
//! # What a failing child is reported as
//!
//! The child's stderr is not captured at all: it is routed to the null
//! device rather than read, so there is no buffer of arbitrary child
//! bytes for a diagnostic to accidentally quote. A failure is reported
//! as a typed error carrying a fixed message, the protocol phase, the
//! child's exit status, and the safe typed case identity — never the
//! executor's path, its argv, or its output
//! `(´[ADR010-rule:output:data-classification]´)`.
//!
//! There is no argument passthrough. The harness passes the executor no
//! arguments at all, which is also why no credential can travel through
//! one: an executor needing its own configuration receives it outside
//! this interface.
//!
//! # A timeout is never a rejection
//!
//! The timeout is explicit typed configuration. When it expires the
//! child is killed and the run fails with
//! [`NativeConformanceError::ExecutorTimeout`]. An executor that ran out
//! of time observed nothing about the target, and recording that as a
//! target rejection would turn a slow machine into target evidence.
//!
//! # One run is one process group
//!
//! A real executor is not one process. The reviewed adapter starts a
//! node, and an arbitrary caller-selected executor may start anything;
//! either descendant inherits the protocol pipe. Stopping only the
//! first process therefore stops neither the tree nor the harness's own
//! wait: a descendant holding stdout open keeps the reader waiting for
//! an end of stream that never comes.
//!
//! So the executor is started in its own process group and stopped as
//! one (Guide-10 §5.8): a graceful signal to the group, a bounded
//! interval in which a well-written adapter runs its own cleanup, a
//! forceful signal to whatever is left, and only then the reap. The
//! order matters in both directions — graceful first, so an adapter can
//! still shut its node down and remove its temporary directory; reap
//! last, so the unreaped child keeps its process-group identifier
//! allocated and the forceful signal cannot land on a recycled group.
//!
//! This is not containment and does not become it. A process that
//! leaves the group, or that the harness has no authority over, is
//! outside what this can reach; the residual is stated in the
//! supervisor's own documentation rather than papered over.

use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

use target_elements::{
    DeploymentEnvironment, ReviewedDevelopmentBinding, ReviewedElementsTapscriptDefinition,
};

use crate::error::NativeConformanceError;
use crate::fixture::{CanonicalPrimitiveFixtureSet, NativeCaseId, PrimitiveFixtureSet};
use crate::protocol::{
    ExecutorCapability, ExecutorEnvironmentObservation, ExecutorHandshake, HandshakeRequest,
    NATIVE_PROTOCOL_SCHEMA, NativeExecutionRequest, NativeExecutionResponse,
    NativePrototypeRequest, NativePrototypeResponse, ProtocolLimits, ProtocolPhase,
    WireEnvironment, WireExecutionDomain, validate_response_shape,
};
use crate::prototype::{CanonicalPrototypeMatrix, CompoundPrototypeFixture, PrototypeCaseId};

/// What one run asks the executor about.
///
/// Two workloads, kept apart in the type rather than in a flag: a
/// primitive census and a compound-prototype matrix are different
/// questions with different case identities, and a run that could carry
/// both would produce a transcript whose rows a report could not tell
/// apart `(´[PLAN-rule:guide10:compound-fixture]´)`.
#[derive(Clone, Copy, Debug)]
pub enum NativeWorkload<'a> {
    /// The canonical primitive census.
    Primitives(&'a PrimitiveFixtureSet),
    /// One compound-prototype case matrix.
    Prototypes(&'a [CompoundPrototypeFixture]),
}

/// How long a run may take before the executor is stopped.
///
/// An explicit typed default rather than an implicit one: the value is
/// stated here, is overridable, and travels as configuration.
pub const DEFAULT_EXECUTOR_TIMEOUT: Duration = Duration::from_secs(300);

/// How long the executor group has to clean up after a graceful signal.
///
/// Long enough for the reviewed adapter to stop a node and remove its
/// temporary directory, and bounded so that an executor which ignores
/// the graceful signal cannot extend the run indefinitely by doing so.
pub const DEFAULT_EXECUTOR_CLEANUP_GRACE: Duration = Duration::from_secs(5);

/// What the caller declares the selected executor to be.
///
/// The declaration is the caller's, recorded as ordinary provenance. It
/// establishes nothing about the program itself — a mock declared
/// reviewed is still a mock — and its only mechanical effect is that a
/// run declaring [`ExecutorTrust::Mock`] can never satisfy the
/// target-native gate.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ExecutorTrust {
    /// A mock, admitted for protocol and failure-path tests only.
    Mock,
    /// The reviewed nonmock runner the gate requires.
    ReviewedNonMock,
}

/// The explicit typed configuration of one executor run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutorConfiguration {
    program: PathBuf,
    timeout: Duration,
    trust: ExecutorTrust,
    limits: ProtocolLimits,
    cleanup_grace: Duration,
}

impl ExecutorConfiguration {
    /// Selects one executor, under the default record bounds.
    #[must_use]
    pub fn new(program: &Path, trust: ExecutorTrust, timeout: Duration) -> Self {
        Self {
            program: program.to_path_buf(),
            timeout,
            trust,
            limits: ProtocolLimits::DEFAULT,
            cleanup_grace: DEFAULT_EXECUTOR_CLEANUP_GRACE,
        }
    }

    /// The same selection under explicit record bounds.
    #[must_use]
    pub fn with_limits(self, limits: ProtocolLimits) -> Self {
        Self { limits, ..self }
    }

    /// The same selection under an explicit cleanup interval.
    #[must_use]
    pub fn with_cleanup_grace(self, cleanup_grace: Duration) -> Self {
        Self {
            cleanup_grace,
            ..self
        }
    }

    /// How long the executor group has to clean up before it is killed.
    #[must_use]
    pub const fn cleanup_grace(&self) -> Duration {
        self.cleanup_grace
    }

    /// How long the run may take.
    #[must_use]
    pub const fn timeout(&self) -> Duration {
        self.timeout
    }

    /// What the caller declares the executor to be.
    #[must_use]
    pub const fn trust(&self) -> ExecutorTrust {
        self.trust
    }

    /// The byte bound on each class of protocol record.
    #[must_use]
    pub const fn limits(&self) -> ProtocolLimits {
        self.limits
    }
}

/// Everything one executor run observed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionTranscript {
    handshake: ExecutorHandshake,
    environment: ExecutorEnvironmentObservation,
    trust: ExecutorTrust,
    responses: BTreeMap<NativeCaseId, NativeExecutionResponse>,
    prototype_responses: BTreeMap<PrototypeCaseId, NativePrototypeResponse>,
}

impl ExecutionTranscript {
    /// What the executor said about itself.
    #[must_use]
    pub const fn handshake(&self) -> &ExecutorHandshake {
        &self.handshake
    }

    /// What the executor said it ran on.
    #[must_use]
    pub const fn environment(&self) -> &ExecutorEnvironmentObservation {
        &self.environment
    }

    /// What the caller declared the executor to be.
    #[must_use]
    pub const fn trust(&self) -> ExecutorTrust {
        self.trust
    }

    /// The responses, in canonical case order.
    #[must_use]
    pub const fn responses(&self) -> &BTreeMap<NativeCaseId, NativeExecutionResponse> {
        &self.responses
    }

    /// The compound-prototype responses, in canonical case order.
    ///
    /// Empty for a primitive run, and empty for a prototype run is a
    /// contradiction the caller can see: the two maps are never both
    /// populated, because one run asks one workload.
    #[must_use]
    pub const fn prototype_responses(&self) -> &BTreeMap<PrototypeCaseId, NativePrototypeResponse> {
        &self.prototype_responses
    }

    /// A transcript assembled directly, for the crate's own tests.
    ///
    /// Not public: a transcript is what an executor said, and a caller
    /// able to state one without an executor could hand the evaluator a
    /// run that never happened.
    #[cfg(test)]
    pub(crate) const fn for_tests(
        handshake: ExecutorHandshake,
        environment: ExecutorEnvironmentObservation,
        trust: ExecutorTrust,
        responses: BTreeMap<NativeCaseId, NativeExecutionResponse>,
    ) -> Self {
        Self {
            handshake,
            environment,
            trust,
            responses,
            prototype_responses: BTreeMap::new(),
        }
    }

    /// A compound-prototype transcript assembled directly, for the
    /// crate's own tests.
    ///
    /// Not public, for the same reason as the primitive one: a
    /// transcript is what an executor said, and a caller able to state
    /// one without an executor could hand the evaluator a run that never
    /// happened.
    #[cfg(test)]
    pub(crate) const fn prototypes_for_tests(
        handshake: ExecutorHandshake,
        environment: ExecutorEnvironmentObservation,
        trust: ExecutorTrust,
        prototype_responses: BTreeMap<PrototypeCaseId, NativePrototypeResponse>,
    ) -> Self {
        Self {
            handshake,
            environment,
            trust,
            responses: BTreeMap::new(),
            prototype_responses,
        }
    }
}

/// Spawn the executor, absorbing the Linux fork/exec text-busy race.
///
/// A caller that stages its executor script immediately before this call
/// can lose the race against an unrelated concurrent fork that still
/// holds the script's write descriptor across its own pre-exec window;
/// the kernel then refuses the exec with a text-file-busy error even
/// though the writer has already closed it. The race self-resolves as
/// soon as the concurrent child completes its exec, so that one cause is
/// retried briefly. Every other spawn failure — an absent program, a
/// permission refusal — is reported on the first attempt.
fn spawn_executor(program: &std::path::Path) -> std::io::Result<std::process::Child> {
    let mut attempts = 0u8;
    loop {
        let mut command = Command::new(program);
        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            // Not captured, not read, not relayed: arbitrary child bytes
            // never become first-party diagnostics.
            .stderr(Stdio::null());
        // The child leads its own process group, so the whole executor
        // tree can be signalled as one. This is the safe standard-library
        // route to it: the equivalent hand-written pre-exec hook would
        // need an unsafe block, which ADR-011 denies workspace-wide.
        #[cfg(unix)]
        std::os::unix::process::CommandExt::process_group(&mut command, 0);
        let result = command.spawn();
        match result {
            Err(error) if error.raw_os_error() == Some(libc_etxtbsy()) && attempts < 5 => {
                attempts += 1;
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            other => return other,
        }
    }
}

/// The text-file-busy errno, named without a libc dependency.
const fn libc_etxtbsy() -> i32 {
    26
}

/// The process group one executor run leads.
///
/// # Why the leader is checked rather than assumed
///
/// Every signal this sends goes to a whole process group, so the one
/// thing that must not be guessed is which group. If the child were not
/// its own group leader it would share this process's group, and a
/// graceful termination of that group would stop the harness — and, in a
/// terminal, whatever else shares it. So the group is verified to be the
/// child's own at spawn, and a run that cannot establish one is refused
/// rather than run unsupervised.
///
/// # What this does not claim
///
/// Group membership is inherited, not enforced. An executor may put a
/// descendant in a group of its own, or hand work to a process this
/// harness never started; neither is reachable here, and neither is
/// claimed to be. This is the harness keeping its own timeout and
/// cleanup contract, not a sandbox (ADR-015, ADR-017).
#[cfg(unix)]
#[derive(Clone, Copy, Debug)]
struct SupervisedGroup {
    /// The group's leader, which is the direct child. The group
    /// identifier has the same value, by construction.
    leader: nix::unistd::Pid,
}

#[cfg(unix)]
impl SupervisedGroup {
    /// Establishes the group the spawned child leads.
    fn establish(child: &Child) -> Result<Self, NativeConformanceError> {
        let raw = i32::try_from(child.id())
            .map_err(|_| NativeConformanceError::ExecutorProcessGroupUnavailable)?;
        let leader = nix::unistd::Pid::from_raw(raw);
        match nix::unistd::getpgid(Some(leader)) {
            Ok(group) if group == leader => Ok(Self { leader }),
            _ => Err(NativeConformanceError::ExecutorProcessGroupUnavailable),
        }
    }

    /// Signals the whole group.
    ///
    /// A group that is already empty answers with `ESRCH`, which is the
    /// outcome the call asked for rather than a failure to report.
    fn signal(self, signal: nix::sys::signal::Signal) {
        let _ignored = nix::sys::signal::killpg(self.leader, signal);
    }

    /// Whether the group leader has exited, leaving it unreaped.
    ///
    /// Not reaping is the point. A reaped child releases its process
    /// identifier, and with it the group identifier derived from it; a
    /// forceful signal sent afterwards could then land on an unrelated
    /// group the host has since created. Observing the exit without
    /// consuming it keeps the identifier allocated until the supervisor
    /// is finished with it.
    #[cfg(any(target_os = "linux", target_os = "android", target_os = "freebsd"))]
    fn leader_exited(self) -> bool {
        use nix::sys::wait::{Id, WaitPidFlag, WaitStatus, waitid};

        let flags = WaitPidFlag::WEXITED | WaitPidFlag::WNOWAIT | WaitPidFlag::WNOHANG;
        match waitid(Id::Pid(self.leader), flags) {
            // An error here is an unobserved state, not an observed
            // exit, and is treated as the former: the cleanup interval
            // simply runs to its bound.
            Ok(WaitStatus::StillAlive) | Err(_) => false,
            Ok(_) => true,
        }
    }

    /// Where no non-reaping wait is available, nothing is observed and
    /// the bounded cleanup interval runs in full.
    #[cfg(not(any(target_os = "linux", target_os = "android", target_os = "freebsd")))]
    const fn leader_exited(self) -> bool {
        false
    }
}

/// Where there is no process group, there is nothing to establish.
#[cfg(not(unix))]
#[derive(Clone, Copy, Debug)]
struct SupervisedGroup;

#[cfg(not(unix))]
impl SupervisedGroup {
    const fn establish(_child: &Child) -> Result<Self, NativeConformanceError> {
        Ok(Self)
    }

    const fn leader_exited(self) -> bool {
        false
    }
}

/// One executor run, started and stopped as a whole process tree.
struct ExecutorSupervisor {
    child: Mutex<Child>,
    group: SupervisedGroup,
    /// Held for the whole of one termination sequence.
    ///
    /// Reaping is taken under the same lock, so the direct child cannot
    /// be consumed part-way through a sequence that still has a group
    /// signal to send. Without that, a watchdog waiting out the cleanup
    /// interval could send its forceful signal after the exchange's own
    /// wait had already released the identifier.
    termination: Mutex<()>,
}

impl ExecutorSupervisor {
    /// Takes ownership of a spawned child and the group it leads.
    fn adopt(child: Child) -> Result<Self, NativeConformanceError> {
        let group = SupervisedGroup::establish(&child)?;
        Ok(Self {
            child: Mutex::new(child),
            group,
            termination: Mutex::new(()),
        })
    }

    /// Stops the whole run: graceful, bounded wait, forceful.
    ///
    /// The forceful signal is sent whether or not the leader has
    /// already gone. A leader that exits promptly says nothing about
    /// the descendants it started, and those are exactly what the group
    /// signal exists to reach.
    fn terminate(&self, grace: Duration) {
        let _sequence = self
            .termination
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        #[cfg(unix)]
        self.group.signal(nix::sys::signal::Signal::SIGTERM);

        let deadline = std::time::Instant::now() + grace;
        while !self.group.leader_exited() {
            if std::time::Instant::now() >= deadline {
                break;
            }
            std::thread::sleep(Duration::from_millis(10));
        }

        #[cfg(unix)]
        self.group.signal(nix::sys::signal::Signal::SIGKILL);

        // The direct child again, for its own sake: on a host with no
        // process groups this is the whole of the termination, and on
        // one with them it costs a signal to an already-dying process.
        if let Ok(mut child) = self.child.lock() {
            let _ignored = child.kill();
        }
    }

    /// The child's exit status, where the host reports one.
    ///
    /// Polled rather than blocked on, so the child is free between
    /// attempts and the watchdog can still stop a run that outstays the
    /// exchange. A signal-terminated child reports no code, which is why
    /// a timeout is decided by the watchdog's flag rather than by this.
    fn wait(&self) -> Option<i32> {
        loop {
            {
                let _sequence = self
                    .termination
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                let mut child = self
                    .child
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                match child.try_wait() {
                    Ok(Some(status)) => return status.code(),
                    Ok(None) => {}
                    Err(_) => return None,
                }
            }
            // A killed child is reaped on one of the next passes: the
            // forceful signal cannot be ignored.
            std::thread::sleep(Duration::from_millis(5));
        }
    }
}

/// Runs every fixture through the selected executor.
///
/// The exchange is lock-step and the request order is the census's
/// canonical one, so the transcript is a function of the fixture set and
/// the executor's answers, not of scheduling.
///
/// # Errors
///
/// Every protocol failure in [`NativeConformanceError`]: a child that
/// will not start, a handshake that does not complete or does not cover
/// the reviewed domain and leaf version, a schema this harness does not
/// speak, a malformed message, a duplicate, unexpected, out-of-order, or
/// missing response, protocol data after the last response, a timeout,
/// and a nonzero exit status.
pub fn execute(
    target: &ReviewedElementsTapscriptDefinition,
    binding: &ReviewedDevelopmentBinding,
    configuration: &ExecutorConfiguration,
    fixtures: &PrimitiveFixtureSet,
) -> Result<ExecutionTranscript, NativeConformanceError> {
    execute_workload(
        target,
        binding,
        configuration,
        NativeWorkload::Primitives(fixtures),
    )
}

/// Runs the canonical census through the selected executor.
///
/// # Why execution itself is not the trust boundary
///
/// Running an arbitrary census is not a way to manufacture evidence: it
/// is a way to ask a node a question. The boundary sits at
/// [`crate::validate::evaluate`], which accepts only the canonical
/// wrapper, so [`execute`] stays open to any census and this entry point
/// exists to make the evidence path read as one canonical sequence from
/// census to gate.
///
/// # Errors
///
/// Every protocol failure [`execute`] states.
pub fn execute_canonical(
    target: &ReviewedElementsTapscriptDefinition,
    binding: &ReviewedDevelopmentBinding,
    configuration: &ExecutorConfiguration,
    fixtures: &CanonicalPrimitiveFixtureSet,
) -> Result<ExecutionTranscript, NativeConformanceError> {
    execute(target, binding, configuration, fixtures.fixtures())
}

/// Runs one compound-prototype matrix through the selected executor.
///
/// The same exchange, the same supervision, and the same refusals: what
/// differs is the record shape and the case identity. The executor is
/// asked whether it reads prototype fixtures before any case is sent,
/// so an executor that does not is a declined workload rather than a
/// stream of messages it cannot parse
/// `(´[PLAN-rule:guide10:schema-migration]´)`.
///
/// # Errors
///
/// Every protocol failure [`execute`] states, plus
/// [`NativeConformanceError::PrototypeFixturesUnsupported`] when the
/// executor does not advertise the capability.
pub fn execute_prototypes(
    target: &ReviewedElementsTapscriptDefinition,
    binding: &ReviewedDevelopmentBinding,
    configuration: &ExecutorConfiguration,
    fixtures: &[CompoundPrototypeFixture],
) -> Result<ExecutionTranscript, NativeConformanceError> {
    execute_workload(
        target,
        binding,
        configuration,
        NativeWorkload::Prototypes(fixtures),
    )
}

/// Runs one canonical prototype matrix through the selected executor.
///
/// The trust boundary sits at [`crate::prototype_validate::evaluate_prototypes`]
/// rather than here, for the reason [`execute_canonical`] gives. This
/// entry point exists so the evidence path reads as one canonical
/// sequence from matrix to gate.
///
/// # Errors
///
/// Every protocol failure [`execute_prototypes`] states.
pub fn execute_canonical_prototypes(
    target: &ReviewedElementsTapscriptDefinition,
    binding: &ReviewedDevelopmentBinding,
    configuration: &ExecutorConfiguration,
    matrix: CanonicalPrototypeMatrix<'_>,
) -> Result<ExecutionTranscript, NativeConformanceError> {
    execute_prototypes(target, binding, configuration, matrix.rows())
}

/// The supervised run, over either workload.
fn execute_workload(
    target: &ReviewedElementsTapscriptDefinition,
    binding: &ReviewedDevelopmentBinding,
    configuration: &ExecutorConfiguration,
    workload: NativeWorkload<'_>,
) -> Result<ExecutionTranscript, NativeConformanceError> {
    let mut child = spawn_executor(&configuration.program)
        .map_err(|_| NativeConformanceError::ExecutorStartupFailed)?;

    let stdin = child
        .stdin
        .take()
        .ok_or(NativeConformanceError::ExecutorStartupFailed)?;
    let stdout = child
        .stdout
        .take()
        .ok_or(NativeConformanceError::ExecutorStartupFailed)?;
    let mut reader = BufReader::new(stdout);

    let supervisor = match ExecutorSupervisor::adopt(child) {
        Ok(supervisor) => Arc::new(supervisor),
        Err(error) => {
            // Nothing is supervised, so nothing may be left running: the
            // pipes are dropped and the run is refused rather than
            // continued outside the cleanup contract.
            return Err(error);
        }
    };
    let watchdog = Watchdog::start(
        Arc::clone(&supervisor),
        configuration.timeout,
        configuration.cleanup_grace,
    );

    let outcome = run_protocol(target, binding, configuration, workload, stdin, &mut reader);

    // A refused exchange ends the run here. The tree is stopped rather
    // than waited for, because it may be mid-way through writing the very
    // record the harness has already refused — an unterminated oversized
    // one, say — and waiting would let the watchdog report a typed
    // protocol failure as a timeout instead.
    if outcome.is_err() {
        supervisor.terminate(configuration.cleanup_grace);
    }

    // The wait polls rather than blocking, so the watchdog can still
    // reach the run: a child that outstays the exchange is a timeout,
    // not a hang of the harness.
    let status = supervisor.wait();
    let expired = watchdog.stop();

    let transcript = match outcome {
        Ok(transcript) => transcript,
        Err(error) => {
            return Err(if expired {
                NativeConformanceError::ExecutorTimeout
            } else {
                error
            });
        }
    };

    if expired {
        return Err(NativeConformanceError::ExecutorTimeout);
    }
    if status != Some(0) {
        return Err(NativeConformanceError::ExecutorExited { status });
    }
    Ok(transcript)
}

/// The protocol exchange itself.
fn run_protocol(
    target: &ReviewedElementsTapscriptDefinition,
    binding: &ReviewedDevelopmentBinding,
    configuration: &ExecutorConfiguration,
    workload: NativeWorkload<'_>,
    mut stdin: impl Write,
    reader: &mut impl BufRead,
) -> Result<ExecutionTranscript, NativeConformanceError> {
    let limits = configuration.limits;
    // The same rule the case loops below follow, and for the same
    // reason: a failed write means the pipe is gone, and *what* that was
    // is decided by the read below and by the child's status rather than
    // guessed from which side of the pipe noticed first.
    //
    // Propagating it instead made the classification a race. A child
    // that exits before the harness writes leaves the handshake write
    // failing with `ExecutorExited`, and a child that exits after it
    // leaves the write succeeding and the read reaching end of stream,
    // which is `ExecutorHandshakeFailed`. The same child, told to die
    // before it speaks, was reported as either one depending on how
    // loaded the machine was. A child that never starts the protocol
    // fails as a handshake failure, always: it is the phase that did not
    // happen that names the failure, and the exit status is what the run
    // reports about a child that *did* speak
    // (´[PLAN-rule:guide10:protocol-handshake]´).
    let _handshake_write = write_message(
        &mut stdin,
        &HandshakeRequest::default(),
        ProtocolPhase::Handshake,
    );
    let handshake: ExecutorHandshake = read_message(reader, ProtocolPhase::Handshake, limits)?
        .ok_or(NativeConformanceError::ExecutorHandshakeFailed)?;

    if handshake.protocol_schema != NATIVE_PROTOCOL_SCHEMA {
        return Err(NativeConformanceError::UnsupportedProtocolSchema {
            offered: handshake.protocol_schema,
        });
    }

    let definition = target.definition();
    let domain = WireExecutionDomain::of(definition.execution_domain())
        .ok_or(NativeConformanceError::TargetContractMismatch)?;
    if !handshake.supported_domains.contains(&domain)
        || !handshake
            .supported_leaf_versions
            .contains(&definition.leaf_version().get())
    {
        return Err(NativeConformanceError::ExecutorProtocolMismatch);
    }
    // A census whose cases read a transaction cannot be answered by an
    // executor that says it accepts no transaction context. Refusing here
    // is the difference between a run that could not happen and a run of
    // cases that quietly executed against no transaction at all.
    match workload {
        NativeWorkload::Primitives(fixtures) => {
            if fixtures.iter().any(|fixture| fixture.context().is_some())
                && !handshake
                    .capabilities
                    .contains(&ExecutorCapability::TransactionContext)
            {
                return Err(NativeConformanceError::ExecutorProtocolMismatch);
            }
        }
        // A prototype request is a record shape a schema-2 executor has
        // never seen, so the gate is what keeps the revision at 2: it
        // decides what may be *sent*, and never what is believed about
        // the answer.
        NativeWorkload::Prototypes(_) => {
            if !handshake.runs_prototype_fixtures() {
                return Err(NativeConformanceError::PrototypeFixturesUnsupported);
            }
        }
    }

    let environment: ExecutorEnvironmentObservation =
        read_message(reader, ProtocolPhase::Environment, limits)?
            .ok_or(NativeConformanceError::MissingEnvironmentObservation)?;
    compare_environment(target, binding, &environment)?;

    let mut responses: BTreeMap<NativeCaseId, NativeExecutionResponse> = BTreeMap::new();
    let mut prototype_responses: BTreeMap<PrototypeCaseId, NativePrototypeResponse> =
        BTreeMap::new();
    match workload {
        NativeWorkload::Primitives(fixtures) => run_primitive_cases(
            fixtures,
            &handshake,
            limits,
            &mut stdin,
            reader,
            &mut responses,
        )?,
        NativeWorkload::Prototypes(matrix) => run_prototype_cases(
            matrix,
            &handshake,
            limits,
            &mut stdin,
            reader,
            &mut prototype_responses,
        )?,
    }

    // Closing stdin is how a well-behaved executor learns the exchange
    // is over; it happens before the read below, because an executor
    // still waiting for input would otherwise never reach its own end of
    // stream and both sides would wait for each other.
    drop(stdin);

    // The protocol is over. Anything further is the executor writing
    // outside the exchange, which fails the run rather than being
    // ignored as harmless noise — a blank trailing record included,
    // since the framing has no empty records to be tolerant of.
    expect_end_of_stream(reader, limits)?;

    Ok(ExecutionTranscript {
        handshake,
        environment,
        trust: configuration.trust,
        responses,
        prototype_responses,
    })
}

/// The primitive half of the exchange.
fn run_primitive_cases(
    fixtures: &PrimitiveFixtureSet,
    handshake: &ExecutorHandshake,
    limits: ProtocolLimits,
    stdin: &mut impl Write,
    reader: &mut impl BufRead,
    responses: &mut BTreeMap<NativeCaseId, NativeExecutionResponse>,
) -> Result<(), NativeConformanceError> {
    for fixture in fixtures {
        let case = fixture.case();
        let request = NativeExecutionRequest {
            schema: NATIVE_PROTOCOL_SCHEMA,
            case,
            fixture: fixture.clone(),
            // A primitive fixture bears no construction, and the field
            // is omitted from the wire entirely rather than written as
            // null, so this request is byte-identical to the one a
            // schema-2 executor has always received.
            construction: None,
        };
        // A failed write means the pipe is gone. What that was — a
        // timeout, an early exit, or an executor that simply stopped
        // answering — is decided by the read below and by the child's
        // status, not guessed here.
        let _write = write_message(&mut *stdin, &request, ProtocolPhase::Request);

        let response: NativeExecutionResponse =
            read_message(reader, ProtocolPhase::Response, limits)?
                .ok_or(NativeConformanceError::MissingCaseResponse(case))?;

        if response.schema != NATIVE_PROTOCOL_SCHEMA {
            return Err(NativeConformanceError::UnsupportedProtocolSchema {
                offered: response.schema,
            });
        }
        if response.case != case {
            // Three different faults, kept apart: an answer to a case
            // already settled, an answer to a case never asked about,
            // and an answer to a case that is still outstanding while
            // another one was asked.
            if responses.contains_key(&response.case) {
                return Err(NativeConformanceError::DuplicateCaseResponse(response.case));
            }
            if !fixtures.contains(response.case) {
                return Err(NativeConformanceError::UnexpectedCaseResponse(
                    response.case,
                ));
            }
            return Err(NativeConformanceError::ResponseOrderViolation { expected: case });
        }
        // Shape before comparison. A response that contradicts its own
        // executor's advertised interface is a protocol failure, and
        // reading a target verdict out of it would mean believing
        // whichever half of the contradiction happens to match.
        validate_response_shape(&response, &handshake.capabilities)
            .map_err(|defect| NativeConformanceError::MalformedResponseShape { case, defect })?;

        responses.insert(case, response);
    }
    Ok(())
}

/// The compound-prototype half of the exchange.
///
/// The same lock-step discipline as the primitive loop, and the same
/// three faults kept apart: an answer to a case already settled, an
/// answer to a case never asked about, and an answer to a case that is
/// still outstanding while another one was asked.
fn run_prototype_cases(
    matrix: &[CompoundPrototypeFixture],
    handshake: &ExecutorHandshake,
    limits: ProtocolLimits,
    stdin: &mut impl Write,
    reader: &mut impl BufRead,
    responses: &mut BTreeMap<PrototypeCaseId, NativePrototypeResponse>,
) -> Result<(), NativeConformanceError> {
    for fixture in matrix {
        let case = fixture.case.clone();
        let request = NativePrototypeRequest {
            schema: NATIVE_PROTOCOL_SCHEMA,
            case: case.clone(),
            fixture: fixture.clone(),
        };
        // A failed write means the pipe is gone. What that was is
        // decided by the read below and by the child's status.
        let _write = write_message(&mut *stdin, &request, ProtocolPhase::Request);

        let response: NativePrototypeResponse =
            read_message(reader, ProtocolPhase::Response, limits)?
                .ok_or_else(|| NativeConformanceError::MissingPrototypeResponse(case.clone()))?;

        if response.schema != NATIVE_PROTOCOL_SCHEMA {
            return Err(NativeConformanceError::UnsupportedProtocolSchema {
                offered: response.schema,
            });
        }
        if response.case != case {
            if responses.contains_key(&response.case) {
                return Err(NativeConformanceError::DuplicatePrototypeResponse(
                    response.case,
                ));
            }
            if !matrix.iter().any(|other| other.case == response.case) {
                return Err(NativeConformanceError::UnexpectedPrototypeResponse(
                    response.case,
                ));
            }
            return Err(NativeConformanceError::PrototypeResponseOrderViolation { expected: case });
        }
        // Shape before comparison, exactly as for a primitive response.
        response
            .validate_shape(&handshake.capabilities)
            .map_err(
                |defect| NativeConformanceError::MalformedPrototypeResponseShape {
                    case: case.clone(),
                    defect,
                },
            )?;

        responses.insert(case, response);
    }
    Ok(())
}

/// Whether the executor ran the chain the binding names.
///
/// Compared before any case executes. A run whose executor observed a
/// different chain from the one the fixtures are stated against has not
/// produced weak evidence about the bound network; it has produced
/// evidence about some other network, and continuing would attach that
/// evidence to this one.
fn compare_environment(
    target: &ReviewedElementsTapscriptDefinition,
    binding: &ReviewedDevelopmentBinding,
    observation: &ExecutorEnvironmentObservation,
) -> Result<(), NativeConformanceError> {
    if observation.schema != NATIVE_PROTOCOL_SCHEMA {
        return Err(NativeConformanceError::UnsupportedProtocolSchema {
            offered: observation.schema,
        });
    }

    let declared = binding.binding();
    let environment_agrees = match declared.environment() {
        DeploymentEnvironment::Development => {
            observation.environment == WireEnvironment::Development
        }
        // No validated production binding exists, so this arm cannot be
        // reached by any binding this crate accepts; it refuses rather
        // than choosing a development answer for a production question.
        _ => false,
    };
    if !environment_agrees || observation.chain_name.is_empty() {
        return Err(NativeConformanceError::EnvironmentBindingMismatch);
    }
    if observation.network_id != declared.network_id() {
        return Err(NativeConformanceError::EnvironmentBindingMismatch);
    }
    if observation.genesis_id != declared.genesis_id() {
        return Err(NativeConformanceError::GenesisObservationMismatch);
    }

    let definition = target.definition();
    let domain = WireExecutionDomain::of(definition.execution_domain())
        .ok_or(NativeConformanceError::TargetContractMismatch)?;
    if !observation.active_domains.contains(&domain)
        || !observation
            .active_leaf_versions
            .contains(&definition.leaf_version().get())
    {
        return Err(NativeConformanceError::ActivationObservationMismatch);
    }

    Ok(())
}

/// Writes one NDJSON message.
fn write_message<T: serde::Serialize>(
    writer: &mut impl Write,
    message: &T,
    phase: ProtocolPhase,
) -> Result<(), NativeConformanceError> {
    let mut bytes = serde_json::to_vec(message)
        .map_err(|_| NativeConformanceError::MalformedResponse { phase })?;
    bytes.push(b'\n');
    // A child that has gone away closes the pipe; the caller decides
    // whether that was a timeout or an early exit.
    writer
        .write_all(&bytes)
        .and_then(|()| writer.flush())
        .map_err(|_| NativeConformanceError::ExecutorExited { status: None })
}

/// Reads one NDJSON message, or `None` at end of stream.
fn read_message<T: serde::de::DeserializeOwned>(
    reader: &mut impl BufRead,
    phase: ProtocolPhase,
    limits: ProtocolLimits,
) -> Result<Option<T>, NativeConformanceError> {
    let Some(line) = read_record(reader, phase, limits)? else {
        return Ok(None);
    };
    serde_json::from_str(&line)
        .map(Some)
        .map_err(|_| NativeConformanceError::MalformedResponse { phase })
}

/// Requires the executor to have finished.
///
/// Any further byte at all is trailing protocol data, including one that
/// makes up a blank record: the framing defines no empty record, so an
/// executor emitting one is writing outside the exchange.
fn expect_end_of_stream(
    reader: &mut impl BufRead,
    limits: ProtocolLimits,
) -> Result<(), NativeConformanceError> {
    let phase = ProtocolPhase::Shutdown;
    let maximum = limits.for_phase(phase);
    let mut buffer = Vec::new();
    let read = bounded_read(reader, maximum, &mut buffer)
        .map_err(|_| NativeConformanceError::MalformedResponse { phase })?;
    if read == 0 {
        return Ok(());
    }
    Err(NativeConformanceError::TrailingProtocolData)
}

/// Reads one complete strict-NDJSON record.
///
/// # Strict, and bounded
///
/// A record is one nonempty JSON object and one newline. A blank or
/// whitespace-only record is refused rather than skipped, because a
/// framing that skips them cannot distinguish an executor that said
/// nothing from an executor that finished. At most `maximum + 1` bytes
/// are read, so a record that never terminates is refused as oversized
/// rather than allocated: the two are told apart by whether the bound was
/// reached, and neither grows the buffer past it.
fn read_record(
    reader: &mut impl BufRead,
    phase: ProtocolPhase,
    limits: ProtocolLimits,
) -> Result<Option<String>, NativeConformanceError> {
    let maximum = limits.for_phase(phase);
    let mut buffer = Vec::new();
    let read = bounded_read(reader, maximum, &mut buffer)
        .map_err(|_| NativeConformanceError::MalformedResponse { phase })?;
    if read == 0 {
        return Ok(None);
    }

    if buffer.last() != Some(&b'\n') {
        // Either the bound was reached without a newline, or the stream
        // ended mid-record. The first is an oversized record; the second
        // is a truncated one, and they are not the same fault.
        return Err(if buffer.len() > maximum {
            NativeConformanceError::ProtocolRecordTooLarge { phase, maximum }
        } else {
            NativeConformanceError::MalformedResponse { phase }
        });
    }
    buffer.pop();

    let record = String::from_utf8(buffer)
        .map_err(|_| NativeConformanceError::MalformedResponse { phase })?;
    if record.trim().is_empty() {
        return Err(NativeConformanceError::BlankProtocolRecord { phase });
    }
    Ok(Some(record))
}

/// Reads up to `maximum + 1` bytes, stopping at the first newline.
fn bounded_read(
    reader: &mut impl BufRead,
    maximum: usize,
    buffer: &mut Vec<u8>,
) -> std::io::Result<usize> {
    let ceiling = u64::try_from(maximum.saturating_add(1)).unwrap_or(u64::MAX);
    reader.by_ref().take(ceiling).read_until(b'\n', buffer)
}

/// Stops the executor tree when its explicit timeout expires.
struct Watchdog {
    finished: Arc<(Mutex<bool>, Condvar)>,
    expired: Arc<AtomicBool>,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl Watchdog {
    /// Starts watching one run.
    fn start(supervisor: Arc<ExecutorSupervisor>, timeout: Duration, grace: Duration) -> Self {
        let finished = Arc::new((Mutex::new(false), Condvar::new()));
        let expired = Arc::new(AtomicBool::new(false));

        let waiting = Arc::clone(&finished);
        let flag = Arc::clone(&expired);
        let handle = std::thread::spawn(move || {
            let (lock, condition) = &*waiting;
            let timed_out = {
                let (done, wait) = condition
                    .wait_timeout_while(
                        lock.lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner),
                        timeout,
                        |done| !*done,
                    )
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                drop(done);
                wait.timed_out()
            };
            if timed_out {
                flag.store(true, Ordering::SeqCst);
                supervisor.terminate(grace);
            }
        });

        Self {
            finished,
            expired,
            handle: Some(handle),
        }
    }

    /// Stops watching, and says whether the timeout had expired.
    fn stop(mut self) -> bool {
        {
            let (lock, condition) = &*self.finished;
            let mut done = lock
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            *done = true;
            drop(done);
            condition.notify_all();
        }
        if let Some(handle) = self.handle.take() {
            let _ignored = handle.join();
        }
        self.expired.load(Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use std::io::{self, Write};
    use std::path::Path;
    use std::time::Duration;

    use target_elements::{
        ActivationDeclaration, DeploymentEnvironment, DevelopmentDeploymentBinding, LeafVersion,
        TargetContractVersion, reviewed_elements_tapscript, validate_reviewed_development_binding,
    };

    use super::{
        ExecutorConfiguration, ExecutorTrust, NativeWorkload, PrimitiveFixtureSet, run_protocol,
    };
    use crate::error::NativeConformanceError;
    use crate::protocol::{MOCK_EXECUTOR_GENESIS_ID, MOCK_EXECUTOR_NETWORK_ID};

    /// A pipe whose far end is already gone.
    struct BrokenPipe;

    impl Write for BrokenPipe {
        fn write(&mut self, _bytes: &[u8]) -> io::Result<usize> {
            Err(io::Error::from(io::ErrorKind::BrokenPipe))
        }

        fn flush(&mut self) -> io::Result<()> {
            Err(io::Error::from(io::ErrorKind::BrokenPipe))
        }
    }

    /// A child that never starts the protocol is a handshake failure.
    ///
    /// The two halves of a died-early child arrive in either order, and
    /// the ordering is the machine's rather than the protocol's: a child
    /// that exits before the harness writes breaks the handshake write,
    /// and one that exits after it leaves the write succeeding and the
    /// read reaching end of stream. Propagating the write's own error
    /// made the classification depend on which happened first, which is
    /// how one mock was reported as an exit under a loaded workspace and
    /// as a handshake failure in isolation.
    ///
    /// This pins the losing ordering directly rather than trying to
    /// provoke it under load: the write fails outright and the reader is
    /// already at end of stream, which is exactly what the harness sees
    /// when the child wins the race. The phase that did not happen names
    /// the failure `(´[PLAN-rule:guide10:protocol-handshake]´)`.
    #[test]
    fn a_broken_handshake_write_is_still_a_handshake_failure() {
        let target = reviewed_elements_tapscript().expect("the reviewed contract validates");
        let binding = validate_reviewed_development_binding(
            &target,
            DevelopmentDeploymentBinding::new(
                TargetContractVersion::V2,
                DeploymentEnvironment::Development,
                MOCK_EXECUTOR_NETWORK_ID,
                MOCK_EXECUTOR_GENESIS_ID,
                ActivationDeclaration::new(true, LeafVersion::TAPSCRIPT, []),
                None,
            ),
        )
        .expect("the binding validates");
        let configuration = ExecutorConfiguration::new(
            Path::new("/nonexistent-executor"),
            ExecutorTrust::Mock,
            Duration::from_secs(1),
        );
        let fixtures = PrimitiveFixtureSet::default();

        let mut reader = io::empty();
        let error = run_protocol(
            &target,
            &binding,
            &configuration,
            NativeWorkload::Primitives(&fixtures),
            BrokenPipe,
            &mut reader,
        )
        .expect_err("a child that never speaks is refused");

        assert!(
            matches!(error, NativeConformanceError::ExecutorHandshakeFailed),
            "expected a handshake failure, got {error}",
        );
    }
}
