//! The secretless executor protocol.
//!
//! # Transport
//!
//! Newline-delimited JSON over the child's stdin and stdout, one
//! complete object per line. The child's stdout is protocol data only;
//! its stderr is not read at all, so there is no path by which arbitrary
//! child bytes become first-party diagnostics.
//!
//! The exchange is lock-step:
//!
//! ```text
//! startup:   one handshake request  → one handshake response
//! execution: one request per case   → one response per case
//! ```
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
pub const NATIVE_PROTOCOL_SCHEMA: u32 = 1;

/// Which part of the exchange the harness was in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ProtocolPhase {
    /// Starting the executor process.
    Startup,
    /// Exchanging the handshake.
    Handshake,
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
            Self::Request => "request",
            Self::Response => "response",
            Self::Shutdown => "shutdown",
        };
        formatter.write_str(text)
    }
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
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutorHandshake {
    /// The protocol revision the executor speaks.
    pub protocol_schema: u32,
    /// What the executor calls itself.
    pub implementation_name: String,
    /// What version it calls itself.
    pub implementation_version: String,
    /// The upstream revision it says it was built from, where it states
    /// one.
    pub upstream_revision: Option<String>,
    /// The execution domains it says it can execute in.
    pub supported_domains: BTreeSet<WireExecutionDomain>,
    /// The leaf version bytes it says it accepts.
    pub supported_leaf_versions: BTreeSet<u8>,
    /// What it says its interface offers.
    pub capabilities: BTreeSet<ExecutorCapability>,
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
