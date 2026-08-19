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

use crate::fixture::{NativeCaseId, PrimitiveExecutionSubject};
use crate::prototype::{PrototypeCaseId, PrototypeConstruction, PrototypeExecutionSubject};

/// The protocol revision this harness speaks.
///
/// # Revision 3 removes the answer from the question
///
/// A revision-2 request carried the complete fixture, expectation
/// included, and asked the executor to discard it before executing. A
/// revision-3 request carries the execution subject and nothing else, so
/// there is no expectation for an executor to discard, misread, or echo
/// `(´[PLAN-rule:guide11-exec:request-subject]´)`.
///
/// This is a breaking change and is numbered as one. A revision-2
/// executor is handed a record whose shape it has never seen and would
/// answer from a field that is no longer there, so the two revisions are
/// refused for each other at the handshake rather than reconciled: a
/// revision-2 record is historical, and nothing here parses one as a
/// revision-3 record.
///
/// Revision 2 itself added the environment observation, the separated
/// executor provenance roles, the bounded-record contract, and strict
/// framing, and was refused for revision 1 on the same ground.
pub const NATIVE_PROTOCOL_SCHEMA: u32 = 3;

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
    /// It materializes a complete taproot tree stated by a fixture, and
    /// refuses one it cannot build exactly.
    ///
    /// # Why this is a capability and not a schema bump
    ///
    /// A tree-bearing request carries a field an executor of the current
    /// revision need not have seen, and its strict framing would reject
    /// the whole request. That is not a schema incompatibility as long as
    /// no such executor is ever sent one, and this capability is what
    /// makes that true: the harness sends a tree-bearing request only to
    /// an executor that advertised the ability to materialize a tree.
    ///
    /// So a construction adds no revision of its own. The field is
    /// omitted entirely rather than written as null, and an executor that
    /// does not advertise this gets a typed refusal from the harness
    /// instead of a message it cannot parse
    /// `(´[PLAN-rule:guide10:schema-migration]´)`.
    TreeMaterialization,
    /// It accepts a compound-prototype fixture as such.
    ///
    /// # Why the fixture could not be projected onto a primitive one
    ///
    /// A primitive request carries a primitive subject and a primitive
    /// case identity. A compound case is neither: its case is a relation
    /// and a name rather than a group and an ordinal, its outcome is a
    /// spend verdict rather than a stack shape, and its construction is a
    /// requirement to build one exact tree. Squeezing it into the
    /// primitive request would have meant a primitive case identity for a
    /// case that has none, and a report row that counted compound
    /// coverage as primitive coverage
    /// `(´[PLAN-rule:guide10:compound-fixture]´)`.
    ///
    /// So a prototype request is its own record, and this capability is
    /// what keeps it from being sent to an executor that cannot read it:
    /// no executor is ever handed one unless it said it reads them.
    CompoundPrototypeFixtures,
}

impl ExecutorHandshake {
    /// Whether this executor may be sent a tree-bearing request.
    ///
    /// # The gate that keeps a construction from an executor that cannot
    /// read one
    ///
    /// A tree-bearing request carries a field an executor need not have
    /// seen, and its strict framing would reject the whole message. That
    /// is only safe because no such executor is ever sent one, and this
    /// predicate is where that is decided rather than assumed: a
    /// construction goes out only to an executor that said it can
    /// materialize a tree exactly
    /// `(´[PLAN-rule:guide10:schema-migration]´)`.
    ///
    /// It is the executor's own claim, like every other capability here.
    /// An executor that advertises this and then approximates a tree is
    /// dishonest, and the fixture's stated values are what catches that
    /// — the capability decides what may be *sent*, never what is
    /// believed about the answer.
    #[must_use]
    pub fn materializes_trees(&self) -> bool {
        self.capabilities
            .contains(&ExecutorCapability::TreeMaterialization)
    }

    /// Whether this executor may be sent a compound-prototype request.
    ///
    /// The same gate, for the same reason: a prototype request is a
    /// record shape an executor need not have seen, and its strict
    /// framing would refuse the whole message. It is only ever sent to
    /// an executor that said it reads them.
    ///
    /// Materializing a tree is a separate claim and is required as well:
    /// every compound fixture states a construction, and an executor
    /// that reads the record but approximates the tree would be
    /// answering about some other output.
    #[must_use]
    pub fn runs_prototype_fixtures(&self) -> bool {
        self.capabilities
            .contains(&ExecutorCapability::CompoundPrototypeFixtures)
            && self.materializes_trees()
    }
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
/// # The boundary moved, and the old position is kept spelled
///
/// A revision-2 request carried the complete fixture DTO, expectation
/// included, and the boundary was a discipline: the executor was required
/// to discard the answer before executing, and an adapter was judged
/// against that requirement. Revision 3 removes the requirement by
/// removing the field — an executor cannot consult what it was never sent
/// `(´[PLAN-rule:guide11-exec:request-subject]´)`.
///
/// [`Self::FixtureCarriesExpectation`] remains spelled because revision-2
/// reports exist and say so. It is what those documents state about
/// themselves, not a position this harness still offers: nothing here
/// produces it, and report validation refuses a report carrying it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum RequestExpectationBoundary {
    /// The request carried the fixture's stated expectation, and the
    /// executor was required to discard it before executing.
    ///
    /// Historical: revision 2 only.
    FixtureCarriesExpectation,
    /// The request carries the execution subject alone. No expected
    /// verdict, failure class, final stack, resource figure, claim, or
    /// evidence class crosses the boundary.
    ExecutorReceivesSubjectOnly,
}

/// One case, handed to the executor.
///
/// # What it carries, and what it deliberately does not
///
/// The case identity, the execution subject, and — where the case bears
/// one — the taproot construction to materialize. It carries no expected
/// verdict, no expected failure class, no expected final stack, no
/// expected resource figure, no claim set, and no evidence plan class:
/// under revision 3 the answer stays with the harness
/// `(´[PLAN-rule:guide11-exec:request-subject]´)`.
///
/// # The construction is additive and omitted by default
///
/// `skip_serializing_if` leaves the construction out entirely rather than
/// writing a null, so a request for a case bearing none is the shorter
/// record. A tree-bearing request carries it, and goes only to an
/// executor that advertised
/// [`ExecutorCapability::TreeMaterialization`].
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeExecutionRequest {
    /// The protocol revision.
    pub schema: u32,
    /// The case being asked about.
    pub case: NativeCaseId,
    /// Exactly what to execute.
    pub subject: PrimitiveExecutionSubject,
    /// The taproot construction the executor must materialize exactly,
    /// where the case bears one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub construction: Option<PrototypeConstruction>,
}

/// One compound-prototype case, handed to the executor.
///
/// # A record of its own, not a primitive request with extras
///
/// The subject carries its own case identity, script, witness stack, and
/// construction, so the executor receives one typed value rather than a
/// primitive request whose meaning depends on which optional fields are
/// present. The top-level case restates the subject's own, exactly as the
/// primitive request restates its subject's: it is what the lock-step
/// exchange matches responses against.
///
/// Under revision 3 the expected verdict, the expected resource figures,
/// and the claim set stay with the harness, exactly as they do for a
/// primitive case `(´[PLAN-rule:guide11-exec:request-subject]´)`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativePrototypeRequest {
    /// The protocol revision.
    pub schema: u32,
    /// The case being asked about.
    pub case: PrototypeCaseId,
    /// Exactly what to execute, construction included.
    pub subject: PrototypeExecutionSubject,
}

/// What the target did with one compound-prototype case.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativePrototypeResponse {
    /// The protocol revision.
    pub schema: u32,
    /// The case answered.
    pub case: PrototypeCaseId,
    /// What the target did with the spend.
    pub verdict: NativeVerdict,
    /// The final main stack, where the executor reports one.
    pub final_stack: Option<Vec<Vec<u8>>>,
    /// The final alternate stack, where the executor reports one.
    pub final_altstack: Option<Vec<Vec<u8>>>,
    /// The failure class observed, where the spend failed and the
    /// executor distinguishes classes.
    pub observed_failure: Option<ObservedFailureClass>,
    /// What the execution cost.
    pub resources: NativeResourceObservation,
}

impl NativePrototypeResponse {
    /// The same shape rules the primitive responses follow.
    ///
    /// Shared rather than restated: a prototype response that
    /// contradicted its executor's advertised interface would be exactly
    /// as unreadable as a primitive one that did, and two copies of the
    /// rule would eventually disagree.
    ///
    /// # Errors
    ///
    /// The first [`ResponseShapeDefect`] the response exhibits.
    pub fn validate_shape(
        &self,
        capabilities: &BTreeSet<ExecutorCapability>,
    ) -> Result<(), ResponseShapeDefect> {
        validate_observation_shape(
            self.verdict,
            self.final_stack.is_some(),
            self.final_altstack.is_some(),
            self.observed_failure.is_some(),
            &self.resources,
            capabilities,
        )
    }
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
    validate_observation_shape(
        response.verdict,
        response.final_stack.is_some(),
        response.final_altstack.is_some(),
        response.observed_failure.is_some(),
        &response.resources,
        capabilities,
    )
}

/// The shape rules themselves, over the fields they read.
///
/// Stated once, over the observation rather than over one message type,
/// because a primitive response and a prototype response report the same
/// observation and differ only in which case identity they restate. Two
/// copies of these rules would eventually disagree, and the half that
/// disagreed would be the half nobody was reading.
fn validate_observation_shape(
    verdict: NativeVerdict,
    reports_stack: bool,
    reports_altstack: bool,
    names_failure: bool,
    observed: &NativeResourceObservation,
    capabilities: &BTreeSet<ExecutorCapability>,
) -> Result<(), ResponseShapeDefect> {
    let classes = capabilities.contains(&ExecutorCapability::FailureClassReporting);
    let stacks = capabilities.contains(&ExecutorCapability::FinalStackReporting);
    let altstacks = capabilities.contains(&ExecutorCapability::FinalAltstackReporting);
    let resources = capabilities.contains(&ExecutorCapability::ResourceObservation);

    if names_failure && !classes {
        return Err(ResponseShapeDefect::FailureClassWithoutAdvertisedReporting);
    }
    match verdict {
        NativeVerdict::Accepted => {
            if names_failure {
                return Err(ResponseShapeDefect::AcceptedResponseNamesFailure);
            }
        }
        NativeVerdict::Rejected => {
            if classes && !names_failure {
                return Err(ResponseShapeDefect::RejectedResponseOmitsAdvertisedFailure);
            }
        }
        NativeVerdict::InfrastructureError => {
            // A run that did not happen observed nothing, so every field
            // describing what the target did must be absent.
            if names_failure || reports_stack || reports_altstack {
                return Err(ResponseShapeDefect::InfrastructureResponseCarriesObservation);
            }
            return Ok(());
        }
    }

    for (advertised, reported) in [(stacks, reports_stack), (altstacks, reports_altstack)] {
        if reported && !advertised {
            return Err(ResponseShapeDefect::StackWithoutAdvertisedReporting);
        }
        if advertised && !reported {
            return Err(ResponseShapeDefect::AdvertisedStackOmitted);
        }
    }

    if !resources && observed.observes_interpreter() {
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
    /// The signature named a sighash type the target does not admit.
    ///
    /// # Why this is not a signature that failed to verify
    ///
    /// The sighash type is read from the signature's trailing byte
    /// *before* any verification is attempted, and a byte outside the
    /// admitted set ends the operation there. Reporting it as an invalid
    /// signature would say the target computed a message and found the
    /// signature wrong over it, which is a claim about work the target
    /// never did.
    InvalidSignatureHashType,
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
    /// The script itself was longer than the target executes.
    ///
    /// Refused before execution begins, on the script's own length, so it
    /// is neither a budget the execution spent nor a result the execution
    /// computed.
    ScriptSizeLimitExceeded,
    /// The execution performed more operations than the target admits.
    ScriptOperationLimitExceeded,
    /// The main and alternate stacks together grew past the target's
    /// bound.
    ///
    /// # Why the result-size class does not cover this
    ///
    /// [`Self::ResultSizeExceeded`] is one computed byte string being too
    /// wide. This is the *number* of items being too many, which no
    /// element's width establishes and which a primitive pushing one
    /// admissible item at a time still reaches.
    StackSizeLimitExceeded,
    /// The script used a byte the target does not execute.
    UnknownOpcode,
    /// A literal was pushed in a form the target refuses.
    MalformedPush,
    /// The leaf version was refused before execution began.
    LeafVersionRejected,
    /// The control block was not a width the target admits.
    ///
    /// # Why this is not a leaf version the target refused
    ///
    /// Both are refusals of the spend's authentication data before any
    /// script runs, and they are still different observations: a refused
    /// leaf version is a version byte the target declines to execute at,
    /// and this is a block whose length is not one the format defines, so
    /// the target never reaches a version byte to judge at all.
    MalformedControlBlock,
    /// Evaluation completed with a false on top of the stack.
    EvaluatedFalse,
    /// A computed byte string was wider than the target admits.
    ResultSizeExceeded,
    /// A requested slice did not lie within the operand.
    SliceOutOfRange,
    /// Two operands a verifying comparison requires equal were not.
    UnequalOperands,
    /// A verified operand was the target's false.
    FalseVerification,
    /// Two operands a bitwise primitive combines were of different
    /// widths.
    MismatchedOperandWidths,
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
