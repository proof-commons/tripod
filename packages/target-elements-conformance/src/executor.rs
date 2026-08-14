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
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

use target_elements::ReviewedElementsTapscriptDefinition;

use crate::error::NativeConformanceError;
use crate::fixture::{NativeCaseId, PrimitiveFixtureSet};
use crate::protocol::{
    ExecutorHandshake, HandshakeRequest, NATIVE_PROTOCOL_SCHEMA, NativeExecutionRequest,
    NativeExecutionResponse, ProtocolPhase, WireExecutionDomain,
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
}

impl ExecutorConfiguration {
    /// Selects one executor.
    #[must_use]
    pub fn new(program: &Path, trust: ExecutorTrust, timeout: Duration) -> Self {
        Self {
            program: program.to_path_buf(),
            timeout,
            trust,
        }
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
}

/// Everything one executor run observed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionTranscript {
    handshake: ExecutorHandshake,
    trust: ExecutorTrust,
    responses: BTreeMap<NativeCaseId, NativeExecutionResponse>,
}

impl ExecutionTranscript {
    /// What the executor said about itself.
    #[must_use]
    pub const fn handshake(&self) -> &ExecutorHandshake {
        &self.handshake
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
    configuration: &ExecutorConfiguration,
    fixtures: &PrimitiveFixtureSet,
) -> Result<ExecutionTranscript, NativeConformanceError> {
    let mut child = Command::new(&configuration.program)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        // Not captured, not read, not relayed: arbitrary child bytes
        // never become first-party diagnostics.
        .stderr(Stdio::null())
        .spawn()
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

    let outcome = run_protocol(target, configuration, fixtures, stdin, &mut reader);

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
    configuration: &ExecutorConfiguration,
    fixtures: &PrimitiveFixtureSet,
    mut stdin: impl Write,
    reader: &mut impl BufRead,
) -> Result<ExecutionTranscript, NativeConformanceError> {
    write_message(
        &mut stdin,
        &HandshakeRequest::default(),
        ProtocolPhase::Handshake,
    )?;
    let handshake: ExecutorHandshake = read_message(reader, ProtocolPhase::Handshake)?
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

        let response: NativeExecutionResponse = read_message(reader, ProtocolPhase::Response)?
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

        responses.insert(case, response);
    }

    // Closing stdin is how a well-behaved executor learns the exchange
    // is over; it happens before the read below, because an executor
    // still waiting for input would otherwise never reach its own end of
    // stream and both sides would wait for each other.
    drop(stdin);

    // The protocol is over. Anything further is the executor writing
    // outside the exchange, which fails the run rather than being
    // ignored as harmless noise.
    if read_line(reader, ProtocolPhase::Shutdown)?.is_some() {
        return Err(NativeConformanceError::TrailingProtocolData);
    }

    Ok(ExecutionTranscript {
        handshake,
        trust: configuration.trust,
        responses,
    })
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
) -> Result<Option<T>, NativeConformanceError> {
    let Some(line) = read_line(reader, phase)? else {
        return Ok(None);
    };
    serde_json::from_str(&line)
        .map(Some)
        .map_err(|_| NativeConformanceError::MalformedResponse { phase })
}

/// Reads one nonempty line, or `None` at end of stream.
fn read_line(
    reader: &mut impl BufRead,
    phase: ProtocolPhase,
) -> Result<Option<String>, NativeConformanceError> {
    loop {
        let mut line = String::new();
        let read = reader
            .read_line(&mut line)
            .map_err(|_| NativeConformanceError::MalformedResponse { phase })?;
        if read == 0 {
            return Ok(None);
        }
        if !line.trim().is_empty() {
            return Ok(Some(line));
        }
    }
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
