//! The secretless executor protocol.
//!
//! # Transport
//!
//! Strict newline-delimited JSON over the child's stdin and stdout: one
//! nonempty JSON object, one newline, and nothing else. A blank record, a
//! whitespace-only record, and a record after the exchange has ended are
//! all failures rather than noise to be skipped — a framing that tolerates
//! empty records cannot tell "the executor said nothing here" from "the
//! executor is finished". The child's stdout is protocol data only; its
//! stderr is not read at all, so there is no path by which arbitrary child
//! bytes become first-party diagnostics.
//!
//! Every record is read under an explicit byte bound
//! ([`ProtocolLimits`]). At most `maximum + 1` bytes are taken before the
//! record is refused, so an executor that writes without ever emitting a
//! newline is a typed protocol failure rather than an allocation the
//! harness grows until the host stops it.
//!
//! The exchange is lock-step:
//!
//! ```text
//! startup:   one handshake request  → one handshake response
//!                                   → one environment observation
//! execution: one request per case   → one response per case
//! ```
//!
//! # The environment is observed, not declared
//!
//! [`ExecutorEnvironmentObservation`] is what the executor says it
//! actually ran on: the chain it booted, that chain's genesis, the
//! domains and leaf versions active there. The harness compares it with
//! the validated binding before any case executes, so a run cannot label
//! one chain with another chain's identity. A dishonest executor can of
//! course lie; what the observation removes is the *honest* adapter's
//! ability to inherit a caller's label without ever consulting the node.
//!
//! # Fail closed
//!
//! Every message type refuses unknown fields, so an executor that
//! invents one is a protocol failure rather than a silently ignored
//! difference. A wrong schema, a duplicate case, a case never asked
//! about, a response arriving while another case is outstanding, a
//! missing response, and any data after the final response are all
//! failures, and none of them is recorded as a target verdict.
//!
//! # A handshake is provenance, not identity
//!
//! [`ExecutorHandshake`] carries what the executor says about itself. A
//! dishonest executor can say anything, and this package neither
//! authenticates it nor claims to. The fields exist so a report can
//! record which runner produced it, which is a provenance question; the
//! trust question is answered by the caller's explicit selection of a
//! reviewed nonmock runner, and by nothing here.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::fixture::{NativeCaseId, PrimitiveFixture};

/// The protocol revision this harness speaks.
///
/// Revision 2 adds the environment observation, the separated executor
/// provenance roles, the bounded-record contract, and strict framing. It
/// is not revision 1 with fields appended: a revision-1 executor states
/// provenance this harness can no longer interpret and observes no
/// environment at all, so the two are refused for each other rather than
/// reconciled.
pub const NATIVE_PROTOCOL_SCHEMA: u32 = 2;

/// Which part of the exchange the harness was in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ProtocolPhase {
    /// Starting the executor process.
    Startup,
    /// Exchanging the handshake.
    Handshake,
    /// Reading the executor's environment observation.
    Environment,
    /// Writing an execution request.
    Request,
    /// Reading an execution response.
    Response,
    /// Closing the executor down after the last response.
    Shutdown,
}

impl std::fmt::Display for ProtocolPhase {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::Startup => "startup",
            Self::Handshake => "handshake",
            Self::Environment => "environment",
            Self::Request => "request",
            Self::Response => "response",
            Self::Shutdown => "shutdown",
        };
        formatter.write_str(text)
    }
}

/// The byte bound on each class of protocol record.
///
/// # Why a bound is a correctness property
///
/// A record is read with at most `maximum + 1` bytes taken from the
/// child. Without a bound, an executor that writes and never emits a
/// newline makes the harness allocate until the host intervenes, which
/// turns a typed protocol failure into a resource failure of the process
/// that was supposed to report it. The bound is not containment — the
/// executor already runs with this process's authority — it is the
/// difference between a refusal the harness states and a refusal the
/// kernel states for it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProtocolLimits {
    /// The largest handshake record.
    pub maximum_handshake_bytes: usize,
    /// The largest environment-observation record.
    pub maximum_environment_bytes: usize,
    /// The largest execution-response record.
    pub maximum_response_bytes: usize,
    /// The largest record read while closing the exchange down.
    pub maximum_trailing_record_bytes: usize,
}

impl ProtocolLimits {
    /// The explicit default bounds.
    ///
    /// The response bound is the generous figure: a response restates a
    /// case identity, two stacks, a failure class, and seven resource
    /// figures, and the stacks are the only part whose size the fixture
    /// influences. The handshake and environment records are small and
    /// fixed-shape, and the shutdown record is refused whatever it holds,
    /// so its bound exists only to bound the reading of it.
    pub const DEFAULT: Self = Self {
        maximum_handshake_bytes: 64 * 1024,
        maximum_environment_bytes: 64 * 1024,
        maximum_response_bytes: 4 * 1024 * 1024,
        maximum_trailing_record_bytes: 4 * 1024,
    };

    /// The bound that applies to one phase.
    #[must_use]
    pub const fn for_phase(&self, phase: ProtocolPhase) -> usize {
        match phase {
            ProtocolPhase::Handshake => self.maximum_handshake_bytes,
            ProtocolPhase::Environment => self.maximum_environment_bytes,
            // The startup and request phases read nothing; naming a
            // bound for them keeps the function total without inventing
            // a read that does not happen.
            ProtocolPhase::Startup | ProtocolPhase::Request | ProtocolPhase::Response => {
                self.maximum_response_bytes
            }
            ProtocolPhase::Shutdown => self.maximum_trailing_record_bytes,
        }
    }
}

impl Default for ProtocolLimits {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// The network identity the test-only mock executor states.
///
/// Stated once here so the mock binary and the tests that drive it cannot
/// drift apart. It is an arbitrary nonzero development value: it names no
/// network, authorizes nothing, and is not secret material.
pub const MOCK_EXECUTOR_NETWORK_ID: [u8; 32] = [0x11; 32];

/// The genesis identity the test-only mock executor states.
pub const MOCK_EXECUTOR_GENESIS_ID: [u8; 32] = [0x22; 32];

/// Which class of deployment a run was bound to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum WireEnvironment {
    /// A network under the project's own control.
    Development,
}

/// The execution domain a fixture is stated against, on the wire.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum WireExecutionDomain {
    /// The taproot script-path execution domain.
    Tapscript,
}

impl WireExecutionDomain {
    /// The wire form of one reviewed execution domain.
    ///
    /// `None` cannot arise for a member of the reviewed census — a
    /// census test proves the conversion is total over it — and the
    /// option exists because the identity is non-exhaustive. Answering a
    /// domain this package has never seen with the one it has would be a
    /// guess wearing a conversion's clothes.
    #[must_use]
    pub const fn of(domain: target_elements::ExecutionDomain) -> Option<Self> {
        match domain {
            target_elements::ExecutionDomain::Tapscript => Some(Self::Tapscript),
            _ => None,
        }
    }
}

/// What an executor says it can do.
///
/// A capability here is the executor's own claim about its interface,
/// not a target capability: `target-elements` owns those, and nothing
/// may read one out of the other.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ExecutorCapability {
    /// It reports the final main stack of an accepted execution.
    FinalStackReporting,
    /// It reports the final alternate stack.
    FinalAltstackReporting,
    /// It distinguishes failure classes rather than reporting one
    /// undifferentiated rejection.
    FailureClassReporting,
    /// It accepts a complete transaction context for introspection.
    TransactionContext,
    /// It reports resource observations.
    ResourceObservation,
}

/// The harness's opening message.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HandshakeRequest {
    /// The protocol revision the harness speaks.
    pub schema: u32,
}

impl Default for HandshakeRequest {
    fn default() -> Self {
        Self {
            schema: NATIVE_PROTOCOL_SCHEMA,
        }
    }
}

/// What the executor says about itself.
///
/// Report provenance. Not target identity, and not authenticity.
///
/// # Five provenance roles, kept apart
///
/// One revision string cannot answer five different questions, and a
/// schema that offers only one invites whichever answer is easiest to
/// obtain. These fields separate what the *binary* says it is, what
/// checkout or integration tip the operator *intended* to run, what
/// upstream base that tip derives from, which local topic branches were
/// folded into it, and which adapter and transaction framework built the
/// run at all (ADR-018). A checkout's `HEAD` answers the second question
/// and never the first: it identifies intended source, not a binary that
/// was built at some earlier time from some other state.
///
/// Every field remains what the executor *says*. An executor that cannot
/// establish its provenance states `None` and the report records the
/// absence, which is a different statement from a plausible substitute.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutorHandshake {
    /// The protocol revision the executor speaks.
    pub protocol_schema: u32,

    /// What the adapter driving the node calls itself.
    pub adapter_name: String,
    /// What version that adapter calls itself.
    pub adapter_version: String,
    /// The revision of the transaction framework the adapter builds its
    /// transactions with, where it states one.
    pub framework_revision: Option<String>,

    /// What the executing node calls itself.
    pub node_name: String,
    /// The node's own version line.
    pub node_version: String,
    /// The revision the *binary* reports, where the binary embeds one.
    ///
    /// Never a checkout's `HEAD`. A binary that embeds no revision states
    /// `None` here, and the intended tip below says what the operator
    /// meant to run.
    pub binary_reported_revision: Option<String>,

    /// The integration tip the operator intended to execute.
    pub intended_executed_tip: Option<String>,
    /// The upstream base that tip derives from.
    pub upstream_base: Option<String>,
    /// The local topic branches folded into that tip.
    pub included_local_topics: BTreeSet<String>,

    /// The execution domains it says it can execute in.
    pub supported_domains: BTreeSet<WireExecutionDomain>,
    /// The leaf version bytes it says it accepts.
    pub supported_leaf_versions: BTreeSet<u8>,
    /// What it says its interface offers.
    pub capabilities: BTreeSet<ExecutorCapability>,
}

/// What the executor says it actually ran on.
///
/// The identifier recipes are part of the protocol, not adapter
/// discretion:
///
/// ```text
/// genesis_id   the chain's block-zero hash, as the node reports it,
///              in the byte order the node prints
/// network_id   the identity the deployment binding names for that
///              chain, which the adapter restates only when it has
///              observed the chain it belongs to
/// chain_name   the chain the node was configured to run
/// ```
///
/// A dishonest executor can state anything here, and this package neither
/// authenticates it nor claims to. What the message removes is the case
/// where an honest adapter never looks at its node at all, and the run
/// inherits whichever identifiers the caller happened to type.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutorEnvironmentObservation {
    /// The protocol revision.
    pub schema: u32,
    /// The class of deployment the executor says it ran on.
    pub environment: WireEnvironment,
    /// The chain the node was configured to run.
    pub chain_name: String,
    /// The network identity of that chain.
    pub network_id: [u8; 32],
    /// The observed genesis identity of that chain.
    pub genesis_id: [u8; 32],
    /// The execution domains active there.
    pub active_domains: BTreeSet<WireExecutionDomain>,
    /// The leaf versions active there.
    pub active_leaf_versions: BTreeSet<u8>,
}

/// Where the expected outcome sits in the exchange.
///
/// The preferred design hands a native executor only the execution
/// subject, so that nothing it consults could be the answer. This
/// protocol does not reach it: the request carries the complete fixture
/// DTO, expectation included, because the fixture is one typed value and
/// splitting it would give the harness and the executor two different
/// notions of what was executed. The boundary is therefore stated rather
/// than assumed, is recorded in the report, and is what an adapter's
/// discard-before-execution discipline is judged against.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum RequestExpectationBoundary {
    /// The request carries the fixture's stated expectation, and the
    /// executor is required to discard it before executing.
    FixtureCarriesExpectation,
}

/// One case, handed to the executor.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeExecutionRequest {
    /// The protocol revision.
    pub schema: u32,
    /// The case being asked about.
    pub case: NativeCaseId,
    /// The complete public fixture.
    pub fixture: PrimitiveFixture,
}

/// What the target did with one case.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeExecutionResponse {
    /// The protocol revision.
    pub schema: u32,
    /// The case answered.
    pub case: NativeCaseId,
    /// What the target did.
    pub verdict: NativeVerdict,
    /// The final main stack, where the executor reports one.
    pub final_stack: Option<Vec<Vec<u8>>>,
    /// The final alternate stack, where the executor reports one.
    pub final_altstack: Option<Vec<Vec<u8>>>,
    /// The failure class observed, where the execution failed and the
    /// executor distinguishes classes.
    pub observed_failure: Option<ObservedFailureClass>,
    /// What the execution cost.
    pub resources: NativeResourceObservation,
}

/// How a response contradicts the interface its executor advertised.
///
/// # An accepted response carrying a failure class is not a pass
///
/// The comparison used to read the fields it cared about and ignore the
/// rest, so a response saying both "the target accepted this" and "here
/// is why it failed" compared as an ordinary acceptance. It is neither
/// verdict: it is a message the protocol does not define, and reading a
/// target fact out of it means believing the half that happens to match.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ResponseShapeDefect {
    /// An accepted response named a failure class.
    AcceptedResponseNamesFailure,
    /// A rejected response named no class while the executor advertised
    /// failure-class reporting.
    RejectedResponseOmitsAdvertisedFailure,
    /// A response named a failure class the executor said it does not
    /// distinguish.
    FailureClassWithoutAdvertisedReporting,
    /// A response reporting infrastructure trouble carried a target
    /// observation, which is a claim about a run that did not happen.
    InfrastructureResponseCarriesObservation,
    /// A stack was reported by an executor that said it observes none.
    StackWithoutAdvertisedReporting,
    /// A stack was omitted by an executor that said it observes one.
    AdvertisedStackOmitted,
    /// An interpreter figure was reported by an executor that said it
    /// observes none.
    ResourceWithoutAdvertisedObservation,
}

impl std::fmt::Display for ResponseShapeDefect {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::AcceptedResponseNamesFailure => "an accepted response named a failure class",
            Self::RejectedResponseOmitsAdvertisedFailure => {
                "a rejected response named no failure class, which the executor advertised"
            }
            Self::FailureClassWithoutAdvertisedReporting => {
                "a response named a failure class the executor said it does not distinguish"
            }
            Self::InfrastructureResponseCarriesObservation => {
                "an infrastructure-error response carried a target observation"
            }
            Self::StackWithoutAdvertisedReporting => {
                "a stack was reported by an executor that observes none"
            }
            Self::AdvertisedStackOmitted => {
                "a stack was omitted by an executor that advertised stack reporting"
            }
            Self::ResourceWithoutAdvertisedObservation => {
                "an interpreter figure was reported by an executor that observes none"
            }
        };
        formatter.write_str(text)
    }
}

/// Whether one response is the shape the executor's own handshake
/// promised.
///
/// Checked before any comparison, because a malformed response is a
/// protocol failure and not a target verdict of any kind.
///
/// # Errors
///
/// The first [`ResponseShapeDefect`] the response exhibits.
pub fn validate_response_shape(
    response: &NativeExecutionResponse,
    capabilities: &BTreeSet<ExecutorCapability>,
) -> Result<(), ResponseShapeDefect> {
    let classes = capabilities.contains(&ExecutorCapability::FailureClassReporting);
    let stacks = capabilities.contains(&ExecutorCapability::FinalStackReporting);
    let altstacks = capabilities.contains(&ExecutorCapability::FinalAltstackReporting);
    let resources = capabilities.contains(&ExecutorCapability::ResourceObservation);

    if response.observed_failure.is_some() && !classes {
        return Err(ResponseShapeDefect::FailureClassWithoutAdvertisedReporting);
    }
    match response.verdict {
        NativeVerdict::Accepted => {
            if response.observed_failure.is_some() {
                return Err(ResponseShapeDefect::AcceptedResponseNamesFailure);
            }
        }
        NativeVerdict::Rejected => {
            if classes && response.observed_failure.is_none() {
                return Err(ResponseShapeDefect::RejectedResponseOmitsAdvertisedFailure);
            }
        }
        NativeVerdict::InfrastructureError => {
            // A run that did not happen observed nothing, so every field
            // describing what the target did must be absent.
            if response.observed_failure.is_some()
                || response.final_stack.is_some()
                || response.final_altstack.is_some()
            {
                return Err(ResponseShapeDefect::InfrastructureResponseCarriesObservation);
            }
            return Ok(());
        }
    }

    for (advertised, reported) in [
        (stacks, response.final_stack.is_some()),
        (altstacks, response.final_altstack.is_some()),
    ] {
        if reported && !advertised {
            return Err(ResponseShapeDefect::StackWithoutAdvertisedReporting);
        }
        if advertised && !reported {
            return Err(ResponseShapeDefect::AdvertisedStackOmitted);
        }
    }

    if !resources && response.resources.observes_interpreter() {
        return Err(ResponseShapeDefect::ResourceWithoutAdvertisedObservation);
    }

    Ok(())
}

/// What the target did with one case.
///
/// The three stay distinct. Infrastructure trouble is never an expected
/// rejection, and a rejection is never trouble: collapsing them is how a
/// harness reports a broken environment as target evidence.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum NativeVerdict {
    /// The target accepted the script.
    Accepted,
    /// The target rejected the script.
    Rejected,
    /// The executor could not run the case at all.
    InfrastructureError,
}

/// The class of a rejection, as the executor distinguishes it.
///
/// This is the harness's own vocabulary, stated so that a fixture's
/// expected class and an executor's observed class are comparable
/// values. It deliberately does not restate `target-elements`' failure
/// causes as a mapped type: a mapping over a non-exhaustive census would
/// need a catch-all arm, and a catch-all is exactly the fallback that
/// makes an unclassified failure look classified.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ObservedFailureClass {
    /// Fewer operands were present than the primitive consumes.
    StackUnderflow,
    /// An operand was not the width the primitive requires.
    InvalidOperandWidth,
    /// A script-number operand was oversized or nonminimal.
    MalformedScriptNumber,
    /// A result was not representable as a script number.
    ScriptNumberRangeExceeded,
    /// A fixed-width conversion was refused, without saying which of its
    /// two reviewed causes applied.
    ///
    /// The target answers a conversion whose operand is the wrong width
    /// and one whose result will not fit a script number with one code,
    /// and the reviewed contract names those as separate causes. An
    /// executor reporting either would be naming a cause it did not
    /// observe, so it reports this — and a fixture whose contract cause
    /// is one of the two admits it alongside, which is what the class
    /// set is for.
    FixedWidthConversionRefused,
    /// The primitive is unavailable in the executing domain.
    UnsupportedExecutionDomain,
    /// The introspection context was unavailable.
    IntrospectionContextUnavailable,
    /// An introspection index named no such input or output.
    IntrospectionIndexOutOfRange,
    /// A serialized hash state could not be loaded.
    HashContextLoad,
    /// A hash state could not absorb the offered data.
    HashContextWrite,
    /// A signed fixed-width operation overflowed.
    ArithmeticOverflow,
    /// A division had a zero divisor.
    DivisionByZero,
    /// The offered signature was empty.
    EmptySignature,
    /// The offered signature did not verify.
    InvalidSignature,
    /// A public key was absent or wrongly encoded.
    InvalidPublicKeyEncoding,
    /// An elliptic-curve relation did not hold.
    InvalidCurveRelation,
    /// A relative timelock was not satisfied.
    UnsatisfiedTimelock,
    /// A timelock operand was negative.
    NegativeTimelock,
    /// The script-path validation budget was exhausted.
    ValidationBudgetExhausted,
    /// The script used a byte the target does not execute.
    UnknownOpcode,
    /// A literal was pushed in a form the target refuses.
    MalformedPush,
    /// The leaf version was refused before execution began.
    LeafVersionRejected,
    /// Evaluation completed with a false on top of the stack.
    EvaluatedFalse,
    /// Evaluation completed leaving other than a single stack item.
    ///
    /// The reviewed execution domain requires exactly one item at the
    /// end, so a primitive that pushed the wrong number of results — or
    /// a non-aborting failure that left its operands in place — is
    /// observed here rather than as a failure of the primitive itself.
    NonSingletonFinalStack,
}

/// What one execution cost.
///
/// Observations, not projections. Nothing here is compared with a
/// target-contract bound by this module; that comparison belongs to the
/// report, against expectations the fixture states.
///
/// # Absent is not zero
///
/// Every figure an executor may be unable to observe is optional. A node
/// that validates a transaction reports what the transaction cost, not
/// what its interpreter's stack did on the way, and recording an
/// unobserved peak as zero would turn "not measured" into a measurement.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeResourceObservation {
    /// The script's size in bytes.
    pub script_bytes: u64,
    /// How many items the initial stack held.
    pub initial_stack_items: u64,
    /// The deepest the main stack became, where the executor sees it.
    pub peak_stack_items: Option<u64>,
    /// The deepest the alternate stack became, where the executor sees
    /// it.
    pub peak_altstack_items: Option<u64>,
    /// The largest stack element in bytes, where the executor sees it.
    pub maximum_element_bytes: Option<u64>,
    /// How much validation budget the execution used, where the
    /// executor accounts for it.
    pub validation_budget_used: Option<u64>,
    /// The materialized transaction's weight, where the executor
    /// reports one.
    pub transaction_weight: Option<u64>,
}

impl NativeResourceObservation {
    /// Whether any figure only an interpreter can see was reported.
    ///
    /// The script's size and the initial stack's depth are the fixture's
    /// own and are restated by every executor. The rest are observations
    /// of an execution in progress, and an executor that says it makes
    /// none may not report one.
    #[must_use]
    pub const fn observes_interpreter(&self) -> bool {
        self.peak_stack_items.is_some()
            || self.peak_altstack_items.is_some()
            || self.maximum_element_bytes.is_some()
            || self.validation_budget_used.is_some()
            || self.transaction_weight.is_some()
    }
}
