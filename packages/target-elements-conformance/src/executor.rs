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
//! (`[ADR010-rule:output:data-classification]`).
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
use crate::fixture::{NativeCaseId, PrimitiveFixtureSet};
use crate::protocol::{
    ExecutorCapability, ExecutorEnvironmentObservation, ExecutorHandshake, HandshakeRequest,
    NATIVE_PROTOCOL_SCHEMA, NativeExecutionRequest, NativeExecutionResponse, ProtocolLimits,
    ProtocolPhase, WireEnvironment, WireExecutionDomain, validate_response_shape,
};

/// How long a run may take before the executor is stopped.
///
/// An explicit typed default rather than an implicit one: the value is
/// stated here, is overridable, and travels as configuration.
pub const DEFAULT_EXECUTOR_TIMEOUT: Duration = Duration::from_secs(300);

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
        }
    }

    /// The same selection under explicit record bounds.
    #[must_use]
    pub fn with_limits(self, limits: ProtocolLimits) -> Self {
        Self { limits, ..self }
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
        let result = Command::new(program)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            // Not captured, not read, not relayed: arbitrary child bytes
            // never become first-party diagnostics.
            .stderr(Stdio::null())
            .spawn();
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

    let child = Arc::new(Mutex::new(child));
    let watchdog = Watchdog::start(Arc::clone(&child), configuration.timeout);

    let outcome = run_protocol(target, binding, configuration, fixtures, stdin, &mut reader);

    // A refused exchange ends the run here. The child is stopped rather
    // than waited for, because it may be mid-way through writing the very
    // record the harness has already refused — an unterminated oversized
    // one, say — and waiting would let the watchdog report a typed
    // protocol failure as a timeout instead.
    if outcome.is_err()
        && let Ok(mut child) = child.lock()
    {
        let _ignored = child.kill();
    }

    // The wait polls rather than blocking, so the watchdog can still
    // reach the child: a child that outstays the exchange is a timeout,
    // not a hang of the harness.
    let status = wait_for(&child);
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
    fixtures: &PrimitiveFixtureSet,
    mut stdin: impl Write,
    reader: &mut impl BufRead,
) -> Result<ExecutionTranscript, NativeConformanceError> {
    let limits = configuration.limits;
    write_message(
        &mut stdin,
        &HandshakeRequest::default(),
        ProtocolPhase::Handshake,
    )?;
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
    if fixtures.iter().any(|fixture| fixture.context().is_some())
        && !handshake
            .capabilities
            .contains(&ExecutorCapability::TransactionContext)
    {
        return Err(NativeConformanceError::ExecutorProtocolMismatch);
    }

    let environment: ExecutorEnvironmentObservation =
        read_message(reader, ProtocolPhase::Environment, limits)?
            .ok_or(NativeConformanceError::MissingEnvironmentObservation)?;
    compare_environment(target, binding, &environment)?;

    let mut responses: BTreeMap<NativeCaseId, NativeExecutionResponse> = BTreeMap::new();
    for fixture in fixtures {
        let case = fixture.case();
        let request = NativeExecutionRequest {
            schema: NATIVE_PROTOCOL_SCHEMA,
            case,
            fixture: fixture.clone(),
        };
        // A failed write means the pipe is gone. What that was — a
        // timeout, an early exit, or an executor that simply stopped
        // answering — is decided by the read below and by the child's
        // status, not guessed here.
        let _write = write_message(&mut stdin, &request, ProtocolPhase::Request);

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
    })
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

/// Stops the executor when its explicit timeout expires.
struct Watchdog {
    finished: Arc<(Mutex<bool>, Condvar)>,
    expired: Arc<AtomicBool>,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl Watchdog {
    /// Starts watching one child.
    fn start(child: Arc<Mutex<Child>>, timeout: Duration) -> Self {
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
                if let Ok(mut child) = child.lock() {
                    let _ignored = child.kill();
                }
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

/// The child's exit status, where the host reports one.
///
/// Polled rather than blocked on, so the child mutex is free between
/// attempts and the watchdog can still stop a child that outstays the
/// exchange. A signal-terminated child reports no code, which is why the
/// timeout is decided by the watchdog's flag rather than by the status.
fn wait_for(child: &Arc<Mutex<Child>>) -> Option<i32> {
    loop {
        {
            let mut child = child
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            match child.try_wait() {
                Ok(Some(status)) => return status.code(),
                Ok(None) => {}
                Err(_) => return None,
            }
        }
        // A killed child is reaped on one of the next passes: the
        // signal the watchdog sends cannot be ignored.
        std::thread::sleep(Duration::from_millis(5));
    }
}
