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
//! Not read, and — for a reviewed executor — not written either. The
//! executor is handed two files on the spawn, one for its own typed facts
//! and one for raw child text it wants kept, so it has somewhere to say
//! what it saw without saying it on a stream this side would have to
//! either quote or throw away
//! (see [`ExecutorDiagnostics`](crate::executor::ExecutorDiagnostics)).
//! An arbitrary caller-selected executor may still write to stderr; that
//! is why the null device is still on the other end of it.
//!
//! That closure is transitive, and it did not used to be. This side never
//! read the executor's stderr, but the reviewed executor collapsed its
//! *own* child's stderr into the note it then wrote as
//! [`NativeConservationResponse::observed_detail`], so the same class of
//! bytes arrived here anyway — through a field this side was reading
//! rather than through a stream it was not. A detail is now what the
//! adapter or the target *stated*: a method, a status, a target's own
//! answer. Neither a child's stderr nor an operator's configuration path
//! is one `(´[PLAN-rule:guide12-exec:failure-layers]´)`; both are kept in
//! the executor's own quarantine file, where a record number rather than
//! the text itself is what a typed diagnostic names.
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

use crate::conservation::{ConservationRowId, ConservationSubject};
use crate::fixture::{NativeCaseId, PrimitiveExecutionSubject};
use crate::lifecycle::{LifecycleOutcome, PublicHandoff};
use crate::normalization::{
    AuthorizationProfile, ClaimedOutput, NormalizationClaim, NormalizationSubject, ObservedOutput,
};
use crate::prototype::{PrototypeCaseId, PrototypeConstruction, PrototypeExecutionSubject};

/// The protocol revision this harness speaks.
///
/// # Revision 5 states the confidential funding arm
///
/// The confidential funding step adds an untagged [`OperationSubject`]
/// variant and two response members that are NOT defaulted, so a
/// revision-4 executor can neither parse a revision-5 request nor
/// produce a revision-5 answer. That is a breaking change and it is
/// numbered as one.
///
/// The two members are undefaulted deliberately, and the contrast with
/// the sponsor members immediately below them is the whole argument. The
/// sponsor witness and its binding were defaulted precisely so that
/// adding them was not a revision: nothing an executor already wrote
/// changed shape. [`NativeOperationResponse::confidential_funded_outputs`]
/// and [`NativeOperationResponse::mined_readback`] are not defaulted
/// precisely so that adding them IS one, because the alternative is a
/// revision-4 executor answering a confidential request with silence in
/// exactly the members the answer lives in.
///
/// A revision-4 executor is therefore refused at the handshake rather
/// than translated for, on the same ground revision 3 was refused for
/// revision 4. There is ONE binary and one schema constant: no
/// compatibility entry point, no dual-vocabulary serializer, and no
/// per-record revision negotiation. "Old executors receive only their
/// old explicit schema" is honoured by handing a revision-4 executor
/// nothing at all, which is a stronger guarantee than translating for it
/// would be, and "never translate a confidential request backward" is
/// honoured vacuously, because no translation exists to be misused.
///
/// The two implementations bump together in one change — this harness
/// and the reviewed native adapter — for the recorded reason that a
/// revision only one side moved to reproduces the two-sided
/// disagreement the revision mechanism exists to end
/// `(´[PLAN-rule:guide12-exec:protocol-revision]´)`.
///
/// # Revision 4 makes both sides describe the same exchange
///
/// Revision 3 was declared by two implementations that did not agree on
/// what it was. The adapter wrote an `observed_openings` member on every
/// conservation response and the typed reader here declared no such
/// field while refusing unknown ones, so the harness could not parse the
/// answers its own executor produced; and the lifecycle exchange had no
/// typed record on this side at all, so the one workload whose evidence
/// is a public record was read out of an untyped value tree. Neither is
/// a difference of opinion a version number can hold: one revision means
/// one schema, and revision 3 named two
/// `(´[PLAN-rule:guide12-exec:protocol-revision]´)`.
///
/// So revision 4 states the union both sides were already implementing:
/// the openings are a declared member of
/// [`NativeConservationResponse`], and the lifecycle step has a request
/// and a response type here like every other workload. Nothing is
/// tolerated that was not declared, and the two implementations bump
/// together — a revision that only one side moved to would reproduce
/// the fault it exists to close.
///
/// This is a breaking change and is numbered as one. A revision-3
/// executor is refused at the handshake rather than reconciled, on the
/// same ground revision 2 was.
///
/// # The operation records complete revision 4 rather than opening a
/// fifth
///
/// §16.3 names four workloads that must have typed records under one
/// revision: conservation, normalization, lifecycle, and the compact-ASH
/// operation. Revision 4 was minted for exactly that union and carried
/// the first three; [`NativeOperationRequest`] and
/// [`NativeOperationResponse`] are the fourth, and adding them here is
/// finishing the revision rather than superseding it. Nothing that a
/// revision-4 executor already implements changes shape, so no executor
/// is refused for a record it used to be able to write.
///
/// Whether the new records may be *sent* is decided the way every other
/// added record shape has been decided in this protocol: by a capability
/// the executor advertises, not by a revision it declares
/// `(´[PLAN-rule:guide10:schema-migration]´)`. An executor that never
/// heard of an operation step is simply never handed one.
///
/// # Revision 3 removed the answer from the question
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
pub const NATIVE_PROTOCOL_SCHEMA: u32 = 5;

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

/// The byte bound on one handshake request, as the executor enforces it.
///
/// # Why the request bounds are constants and the response bounds are not
///
/// [`ProtocolLimits`] is configuration because the side that enforces it
/// is the side that holds it: this harness reads responses, so a caller
/// tightening a response bound tightens something this process will
/// actually apply.
///
/// The request bounds are enforced by the *executor*, which is a separate
/// program that this interface passes no configuration to. A
/// per-run request bound would therefore be a bound only one side knew —
/// the harness would believe it had tightened the framing while the
/// executor went on reading whatever arrived, which is the two-sided
/// disagreement protocol revision 4 was minted to end. So the request
/// bounds are stated once, here, as part of the contract both
/// implementations declare, and the reviewed adapter mirrors these exact
/// figures `(´[PLAN-rule:guide12-exec:protocol-revision]´)`.
///
/// The values match the response bounds for the same records, because
/// they bound the same shapes: a handshake is small and fixed, and a
/// request carries an execution subject whose script and stack are the
/// only part a fixture's size reaches.
///
/// Changing either figure is a change to what a conforming executor must
/// accept, so it moves in both implementations together or in neither.
pub const MAXIMUM_HANDSHAKE_REQUEST_BYTES: usize = 64 * 1024;

/// The byte bound on one execution request, as the executor enforces it.
///
/// See [`MAXIMUM_HANDSHAKE_REQUEST_BYTES`] for why this is a constant of
/// the contract rather than a member of [`ProtocolLimits`].
pub const MAXIMUM_REQUEST_BYTES: usize = 4 * 1024 * 1024;

/// The request bound that applies to one phase.
///
/// Total over the phases so that a new phase cannot quietly acquire "no
/// bound" by being left out of a match.
#[must_use]
pub const fn maximum_request_bytes(phase: ProtocolPhase) -> usize {
    match phase {
        ProtocolPhase::Handshake => MAXIMUM_HANDSHAKE_REQUEST_BYTES,
        // Nothing is written to the executor in these phases; naming a
        // bound keeps the function total without inventing a write that
        // does not happen.
        ProtocolPhase::Startup
        | ProtocolPhase::Environment
        | ProtocolPhase::Request
        | ProtocolPhase::Response
        | ProtocolPhase::Shutdown => MAXIMUM_REQUEST_BYTES,
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
    /// It materializes a generic confidential transaction and reports
    /// which layer refused it.
    ///
    /// # Why this is a capability rather than a revision
    ///
    /// A conservation request is a record shape an executor need not have
    /// seen, and strict framing would refuse the whole message. The same
    /// gate that protects the tree-bearing and compound records protects
    /// this one: it is sent only to an executor that said it reads them
    /// `(´[PLAN-rule:guide10:schema-migration]´)`.
    ///
    /// The claim is specifically about *materialization plus layer
    /// attribution*. An executor that can build a confidential
    /// transaction but reports one undifferentiated rejection cannot
    /// answer a conservation row, because the row's whole content is
    /// which layer refused.
    ConfidentialConservation,
    /// It builds an owner-authorized normalization and reports both what
    /// the claim named and what the target's decoder read back.
    ///
    /// # Why the two observations are one capability
    ///
    /// A normalization row is refused by the report layer in three of its
    /// nine cases, and a report-layer refusal is a disagreement between
    /// the claim and the transaction. An executor that reported only a
    /// verdict could not produce one, and an executor that reported only
    /// the outputs it *meant* to write could never disagree with itself.
    /// So the capability is the pair: the claim's own outputs, stated
    /// before any mutation, and the observed outputs read back from the
    /// target `(´[PLAN-rule:guide10:schema-migration]´)`.
    ///
    /// It also entails the signing profile. §10.3 requires authorization
    /// committing to the finalized output set, and an executor that
    /// signed under a narrower profile would answer the three
    /// post-signing rows with a refusal that establishes nothing.
    OwnerAuthorizedNormalization,
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
    /// It can be driven through the Guide-11 §13 fresh-process
    /// lifecycle: build a public object in one process, and read it back
    /// from chain data in another that shares no state with the first.
    ///
    /// # Why this variant was missing, and what that broke
    ///
    /// The lifecycle lane is driven from `run-fresh-process-lifecycle.py`
    /// rather than from this harness, so no Rust code ever *sent* a
    /// lifecycle record and the capability had no consumer here. The
    /// adapter advertised it anyway, because the handshake is one list
    /// and an adapter states everything its interface offers.
    ///
    /// That made the two halves of one protocol disagree about the
    /// vocabulary. [`ExecutorHandshake`] refuses unknown members and a
    /// capability list is parsed as a set of *known* values, so any
    /// wallet-enabled adapter — the only kind that advertises this —
    /// produced a handshake this harness could not read at all. The
    /// failure surfaced as `malformed message during the handshake
    /// phase`, which named the symptom and not the missing word, and it
    /// stayed hidden because the one Rust lane that speaks to a real
    /// adapter never enables a wallet.
    ///
    /// Adding the variant is the fix rather than dropping the
    /// advertisement: the adapter's claim is true, and the harness not
    /// having a use for a capability is not a reason to be unable to
    /// hear it. The revision does not move, on the protocol's own rule
    /// that an added capability is how this vocabulary grows
    /// `(´[PLAN-rule:guide10:schema-migration]´)`.
    FreshProcessLifecycle,
    /// It creates spendable outputs at a stated witness program, and
    /// reports where they landed.
    ///
    /// # Why funding is a capability of its own
    ///
    /// Issuing an asset and paying outputs are things a node does with
    /// its own wallet, and an executor that judges transactions perfectly
    /// well may hold no funds and no issuance capability at all. Folding
    /// this into the submission claim would refuse such a runner from a
    /// workload it could answer, and would let a runner that advertised
    /// submission be sent a step it can only fail
    /// `(´[PLAN-rule:guide10:schema-migration]´)`.
    TestFundingCeremony,
    /// It hands the target a complete transaction and reports the layer
    /// the target's answer came from.
    ///
    /// # What the claim is about
    ///
    /// Submission *plus* layer attribution, on exactly the reasoning
    /// [`Self::ConfidentialConservation`] gives: an executor that can
    /// broadcast but reports one undifferentiated refusal cannot answer
    /// a submission step, because the step's whole content is which
    /// layer the target answered from.
    TargetTransactionSubmission,
    /// It acts as a sponsor on this development network: it creates a
    /// coin out of its own reserve holdings at a program it can
    /// authorize, and authorizes one input of a finalized transaction.
    ///
    /// # Why the capability lives on this side of the boundary
    ///
    /// A sponsored transaction carries an input somebody has to
    /// authorize, and the packages that build one hold no key and derive
    /// none. That is a deliberate property rather than a gap: a
    /// constructor that could produce an authorization could also
    /// produce one nobody asked for. So the ability is the executor's,
    /// it is advertised here like every other, and an executor without
    /// it is never sent a step it could only refuse
    /// `(´[PLAN-rule:guide10:schema-migration]´)`.
    ///
    /// # What an executor advertising this is claiming, and what it is
    /// not
    ///
    /// The claim is narrow: on a disposable development network, this
    /// executor holds reserve value and can authorize a spend of a coin
    /// it made. The key material behind it is test-scoped by
    /// construction — it belongs to a chain nobody settles on and
    /// authorizes nothing anywhere else
    /// `(´[ADR015-rule:security:test-material]´)`.
    ///
    /// It is emphatically *not* a claim about a sponsor's authorization
    /// strength, custody, or standing. Evidence produced with this
    /// capability is evidence about what the *candidate* owes — that the
    /// transaction it builds is one the target accepts, and that the
    /// projection read back off the accepted bytes matches the one the
    /// model derived. Reading it as evidence that a sponsor's
    /// authorization was sound would be reading a fact about the
    /// candidate as a fact about a party the run never had.
    TestSponsorAuthorization,
    /// It materializes and mines outputs carrying an explicit protocol
    /// asset and a confidential value, deterministically, under a
    /// selected fixture, and reads the mined bytes back.
    ///
    /// # Why this is not the funding capability with a flag
    ///
    /// [`Self::TestFundingCeremony`] is a claim about creating coins at
    /// a stated program with a stated explicit amount, which every
    /// wallet-bearing node can do. This is a claim about producing an
    /// exact hybrid representation — explicit asset, committed value,
    /// mandatory rangeproof, empty surjection proof — from a public
    /// fixture, with no wallet choice anywhere in it. No reviewed stock
    /// interface produces that form, so an executor that can fund
    /// perfectly well may be unable to answer a single confidential
    /// step, and folding the two claims together would send it one it
    /// could only fail `(´[PLAN-rule:guide10:schema-migration]´)`.
    ///
    /// # What advertising it does not settle
    ///
    /// Which profiles the executor supports. The capability says the
    /// interface exists; [`ConfidentialFundingAdvertisement`] says which
    /// representation, custody, materializer, and reproducibility
    /// contract it will accept, and the two are held consistent at the
    /// handshake — one without the other is refused rather than read as
    /// the more convenient half.
    ///
    /// Advertising constrains the ceremony plan's selection and never
    /// makes it. The plan selects the contract it will be judged under;
    /// an unsupported selection is a typed refusal before construction
    /// rather than a silent downgrade after it.
    ConfidentialValueTestFunding,
    /// It creates a coin of its OWN reserve whose value is committed,
    /// and can still authorize a spend of it afterwards.
    ///
    /// # Why neither existing claim covers this one
    ///
    /// [`Self::ConfidentialValueTestFunding`] is a claim about coins the
    /// executor creates and never spends: a confidential receipt is
    /// funded, mined, and handed to a candidate, and the executor
    /// retains nothing about it. [`Self::TestSponsorAuthorization`] is a
    /// claim about coins the executor DOES spend, and it has always been
    /// a claim about explicit ones, an amount being enough to rebuild
    /// the value field a signature hash commits to.
    ///
    /// A committed sponsor coin needs both halves at once, and the join
    /// is where the work is rather than where the words are. The value
    /// field of such a coin is a point and not a number, so an executor
    /// that retained an amount has nothing it can sign against: it has
    /// to retain the FIELD across the funding step. An executor holding
    /// both existing capabilities may still be unable to do that, which
    /// is why this is a third claim rather than the conjunction of two
    /// others.
    ///
    /// # What advertising it does not claim
    ///
    /// Anything about a sponsor's standing, and anything about what a
    /// target thinks of a transaction built from such a coin. It says
    /// the executor can produce one and authorize a spend of it, and a
    /// target's verdict on the result is a separate observation.
    ConfidentialValueSponsorAuthorization,
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

    /// Whether this executor may be sent a conservation row.
    ///
    /// The same gate once more. Unlike the prototype predicate this
    /// requires no tree capability: a conservation row states a value
    /// flow rather than a taproot commitment, and demanding a tree an
    /// executor does not need would refuse a runner that can answer the
    /// row perfectly well.
    #[must_use]
    pub fn runs_conservation_rows(&self) -> bool {
        self.capabilities
            .contains(&ExecutorCapability::ConfidentialConservation)
    }

    /// Whether this executor may be sent a normalization row.
    ///
    /// The same gate again, and it requires the conservation capability
    /// as well: a normalization row is a confidential transaction whose
    /// refusing layer has to be attributed, so an executor that could not
    /// answer a conservation row could not answer this one either. The
    /// normalization claim is the further capability on top.
    #[must_use]
    pub fn runs_normalization_rows(&self) -> bool {
        self.capabilities
            .contains(&ExecutorCapability::OwnerAuthorizedNormalization)
            && self.runs_conservation_rows()
    }

    /// Whether this executor may be sent one operation step.
    ///
    /// The same gate once more, and asked per step rather than per run:
    /// an operation plan interleaves two kinds of work, and an executor
    /// that can do one of them should be refused only the steps it cannot
    /// do. Refusing the whole workload on the strength of the harder half
    /// would decline a runner that could have answered every step the
    /// caller actually planned.
    #[must_use]
    pub fn runs_operation_step(&self, subject: &OperationSubject) -> bool {
        self.capabilities.contains(&subject.required_capability())
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

    /// Which confidential funding selections it will accept, where it
    /// offers confidential funding at all.
    ///
    /// Absent by default and held consistent with the capability: the
    /// record without the capability, and the capability without the
    /// record, are both refused at the handshake.
    pub confidential_funding: Option<ConfidentialFundingAdvertisement>,
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
    /// An operation response carried members the step kind it answers
    /// does not produce.
    OperationResponseMismatchesStep,
    /// An accepted operation step reported nothing its kind is defined to
    /// produce.
    AcceptedOperationOmitsObservation,
    /// A refused operation step reported what only an acceptance
    /// produces.
    ///
    /// The converse of [`Self::AcceptedOperationOmitsObservation`], and
    /// the more dangerous half. An acceptance with nothing to show for
    /// it is a record a reader can see is empty; a refusal carrying a
    /// created coin, an issued identity, an accepted transaction
    /// identity, or an authorization carries two answers to one
    /// question, and a consumer reading either member alone gets a
    /// different verdict from the same row.
    RefusedOperationCarriesObservation,
    /// A lifecycle step reported an outcome only the other role reaches.
    ///
    /// The two roles are not two configurations of one step: Process A
    /// publishes and Process B reads, so `constructed` is a verdict only
    /// a construct step reaches and `verified` — and every refusal
    /// beneath it — is one only a verify step reaches. A record that
    /// crosses that line is not a run either process made.
    LifecycleOutcomeMismatchesRole,
    /// A lifecycle response carried members the step it answers does not
    /// produce.
    ///
    /// Either the other role's members — a verify step publishing a
    /// handoff, a construct step reporting what the chain said — or its
    /// own role's members under an outcome that does not reach them,
    /// which is how a refusal comes to carry the spend that only a
    /// completed verification builds.
    LifecycleResponseMismatchesStep,
    /// A lifecycle step that ran reported nothing its outcome rests on.
    ///
    /// A construct step exists to publish a record, and a verify step
    /// reaches every one of its verdicts by looking something up. An
    /// outcome with none of that behind it is indistinguishable from an
    /// adapter that named the outcome without doing the work — the same
    /// reasoning [`Self::AcceptedOperationOmitsObservation`] encodes for
    /// the operation record.
    LifecycleStepOmitsObservation,
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
            Self::OperationResponseMismatchesStep => {
                "an operation response carried members the step kind it answers does not produce"
            }
            Self::AcceptedOperationOmitsObservation => {
                "an accepted operation step reported nothing its kind produces"
            }
            Self::RefusedOperationCarriesObservation => {
                "a refused operation step reported what only an acceptance produces"
            }
            Self::LifecycleOutcomeMismatchesRole => {
                "a lifecycle step reported an outcome only the other role reaches"
            }
            Self::LifecycleResponseMismatchesStep => {
                "a lifecycle response carried members the step it answers does not produce"
            }
            Self::LifecycleStepOmitsObservation => {
                "a lifecycle step that ran reported nothing its outcome rests on"
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
            // describing what the target did must be absent. The
            // interpreter figures are part of that, and the line between
            // them and the fixture's own restated script size and
            // initial depth is already drawn by `observes_interpreter`
            // rather than redrawn here.
            //
            // The advertised-observation rule below never sees this arm,
            // so the refusal cannot be left to it: an executor that
            // advertises resource observation would otherwise be allowed
            // to report a peak stack depth for a run it never made.
            if names_failure || reports_stack || reports_altstack || observed.observes_interpreter()
            {
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

/// Where one conservation execution actually ended up.
///
/// # Guide 11 §8.3, and the failure this vocabulary exists to prevent
///
/// A conservation row's entire content is *where* the target refused. A
/// malformed range proof refused before any script runs is CT consensus
/// evidence; the same refusal reported as a script-path rejection would
/// be evidence about an opening script that never executed. And a
/// transaction the adapter failed to build is not a target rejection at
/// all — recording it as one manufactures a consensus fact out of a bug
/// in the test harness.
///
/// So the six layers stay distinct, and the classification is the
/// adapter's *observation* — which RPC refused, at what stage — never the
/// row's expectation. The executor is never told what was expected, so it
/// has nothing to classify toward.
///
/// # Two of these are not target verdicts
///
/// [`Self::FixtureConstructionFailure`] and
/// [`Self::ExecutorInfrastructureFailure`] say the run did not happen.
/// They are reported so a reader can see the row was attempted and why it
/// produced nothing, and a gate must never read either as a rejection the
/// target made.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ObservedOutcomeLayer {
    /// The adapter could not build the stated transaction at all.
    ///
    /// Not a target rejection. The target was never asked.
    FixtureConstructionFailure,
    /// The adapter's environment failed around the execution.
    ///
    /// Not a target rejection either, and kept apart from a construction
    /// failure because they are fixed by different people: one is the
    /// fixture or the materializer, the other is the node or the host.
    ExecutorInfrastructureFailure,
    /// The target refused the transaction before running any script.
    ///
    /// This is where value conservation lives: amounts, commitments,
    /// range proofs, and surjection proofs are checked here, and a script
    /// never runs if they fail.
    ConsensusRejectionBeforeScript,
    /// The target ran the script path and it failed.
    ScriptPathRejection,
    /// The target would relay-refuse an otherwise consensus-valid
    /// transaction.
    RelayPolicyRejection,
    /// The target accepted the transaction.
    Accepted,
}

impl ObservedOutcomeLayer {
    /// Whether this layer is a verdict the target actually reached.
    ///
    /// The two non-verdicts are the whole reason the vocabulary is six
    /// values rather than four, and every consumer that turns a layer
    /// into evidence must ask this first.
    #[must_use]
    pub const fn is_target_verdict(&self) -> bool {
        match self {
            Self::FixtureConstructionFailure | Self::ExecutorInfrastructureFailure => false,
            Self::ConsensusRejectionBeforeScript
            | Self::ScriptPathRejection
            | Self::RelayPolicyRejection
            | Self::Accepted => true,
        }
    }
}

impl std::fmt::Display for ObservedOutcomeLayer {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::FixtureConstructionFailure => "fixture construction failure",
            Self::ExecutorInfrastructureFailure => "executor infrastructure failure",
            Self::ConsensusRejectionBeforeScript => "consensus rejection before script",
            Self::ScriptPathRejection => "script-path rejection",
            Self::RelayPolicyRejection => "relay-policy rejection",
            Self::Accepted => "accepted",
        };
        formatter.write_str(text)
    }
}

/// One conservation row, handed to the executor.
///
/// Carries the row's identity and its subject. No expected layer crosses
/// this boundary `(´[PLAN-rule:guide11-exec:request-subject]´)`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeConservationRequest {
    /// The protocol revision.
    pub schema: u32,
    /// The row being asked about.
    pub case: ConservationRowId,
    /// Exactly what to materialize and judge.
    pub subject: ConservationSubject,
}

/// One confidential output's opening, as the target reported it.
///
/// # Why the target supplies these rather than the harness
///
/// The oracle predicts a commitment from an amount and two blinding
/// factors. The materializer does not choose those factors — the node
/// draws them — so without reading them back there is nothing for the
/// oracle to predict, and §7.4's three-way comparison has no second
/// point to meet at. The comparison then runs one way: the oracle
/// predicts bytes from these openings, and the prediction is checked
/// against the commitment the transaction actually carries. No expected
/// value is ever rewritten to match an observation.
///
/// The blinding factors are the target's own statement about a
/// transaction it built on a disposable development chain, and are
/// evidence rather than credentials `(´[ADR015-rule:security:test-material]´)`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConservationOpening {
    /// Which output of the materialized transaction this opens.
    pub vout: u32,
    /// The explicit amount the output commits to.
    pub amount_satoshis: u64,
    /// The asset the output commits to, in the node's own spelling.
    pub asset: String,
    /// The amount blinding factor the node drew.
    pub amount_blinder: String,
    /// The asset blinding factor the node drew.
    pub asset_blinder: String,
}

/// What the target did with one conservation row.
///
/// # Why this response carries bytes
///
/// Every other response in this protocol reports a verdict and some
/// figures. This one also reports the transaction the adapter built and
/// the output commitments the node read back out of it, for two reasons
/// the wave established:
///
/// - the materializer cannot produce the same bytes twice, so the bytes
///   have to be recorded per run or the row is not reproducible as
///   evidence at all;
/// - the observed commitments are the third leg of the §7.4 three-way
///   comparison, and they have to arrive from the target rather than
///   from the harness's own arithmetic or the comparison is circular.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeConservationResponse {
    /// The protocol revision.
    pub schema: u32,
    /// The row answered.
    pub case: ConservationRowId,
    /// Where the execution ended up, as the adapter observed it.
    pub observed_layer: ObservedOutcomeLayer,
    /// What the target or the adapter said, verbatim and unmapped.
    ///
    /// Recorded rather than classified. This target answers every
    /// conservation failure with one consensus code, and paraphrasing it
    /// into a richer class would invent a distinction the target does not
    /// make.
    pub observed_detail: Option<String>,
    /// The transaction the adapter materialized, where it built one.
    pub transaction_bytes: Option<Vec<u8>>,
    /// The output value commitments the node read back, in output order.
    ///
    /// Empty where the transaction carries no confidential output, or
    /// where it was never built.
    pub observed_value_commitments: Vec<Vec<u8>>,
    /// The output asset commitments the node read back, in output order.
    pub observed_asset_commitments: Vec<Vec<u8>>,
    /// The openings the node reported for the outputs it created.
    ///
    /// Empty where the transaction was never built, where the target did
    /// not accept it, or where it carries no confidential output whose
    /// blinding factors the node holds.
    pub observed_openings: Vec<ConservationOpening>,
}

impl NativeConservationResponse {
    /// Whether this response contradicts itself.
    ///
    /// A response saying the run never happened must carry no target
    /// observation, on exactly the reasoning
    /// [`ResponseShapeDefect::InfrastructureResponseCarriesObservation`]
    /// encodes for the other record shapes: bytes and commitments
    /// describe a transaction that was built and judged, and a
    /// construction failure built nothing.
    ///
    /// The openings are counted among those, and are the clearest case
    /// of the rule rather than a borderline one. An opening is read back
    /// out of a transaction the node created and confirmed, by looking
    /// up the coins that transaction made; a run that reached no target
    /// verdict created no such transaction, so there was nothing to look
    /// up and no factor for the node to have drawn. A blinding factor
    /// beside a layer saying the execution never happened is therefore
    /// not an unusually detailed failure report — it is a value with no
    /// possible provenance, and admitting it would let §7.4's
    /// three-way comparison rest on one.
    ///
    /// # Errors
    ///
    /// [`ResponseShapeDefect`] where the response is not a shape the
    /// protocol defines.
    pub const fn validate_shape(&self) -> Result<(), ResponseShapeDefect> {
        if !self.observed_layer.is_target_verdict()
            && (self.transaction_bytes.is_some()
                || !self.observed_value_commitments.is_empty()
                || !self.observed_asset_commitments.is_empty()
                || !self.observed_openings.is_empty())
        {
            return Err(ResponseShapeDefect::InfrastructureResponseCarriesObservation);
        }
        Ok(())
    }
}

/// One normalization row's identity.
///
/// A single field, and deliberately not the shape any other record
/// carries: a primitive case is a group and an ordinal, a compound one a
/// relation and a name, a conservation row an ordinal and a name. The
/// adapter tells the four apart by shape alone, so a fifth record that
/// reused one of those shapes would be answered by the wrong handler.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NormalizationCaseId {
    /// The mutation this row applies, by its wire spelling.
    pub normalization: String,
}

/// One normalization row, handed to the executor.
///
/// Carries the claim and the mutation. No expected layer crosses this
/// boundary `(´[PLAN-rule:guide11-exec:request-subject]´)`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeNormalizationRequest {
    /// The protocol revision.
    pub schema: u32,
    /// The row being asked about.
    pub case: NormalizationCaseId,
    /// Exactly what to build and judge.
    pub subject: NormalizationSubject,
}

/// What the target did with one normalization row.
///
/// # Why the claim's outputs come back with the observation
///
/// The report layer refuses a row by finding the claim and the
/// transaction in disagreement, so it needs both sides. The claimed side
/// cannot be computed by the harness — which script an owner holds is
/// learned when the coin is created — and the observed side must not be,
/// or the comparison is the harness checking its own intent against
/// itself. So the adapter reports the claim's outputs as it resolved
/// them *before* applying any mutation, and the observed outputs as the
/// target's own decoder read them back.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeNormalizationResponse {
    /// The protocol revision.
    pub schema: u32,
    /// The row answered.
    pub case: NormalizationCaseId,
    /// Where the execution ended up, as the adapter observed it.
    pub observed_layer: ObservedOutcomeLayer,
    /// What the target or the adapter said, verbatim and unmapped.
    pub observed_detail: Option<String>,
    /// The outputs the claim named, resolved before any mutation.
    pub claimed_outputs: Vec<ClaimedOutput>,
    /// The outputs the target's decoder reported.
    pub observed_outputs: Vec<ObservedOutput>,
    /// The authorization profile the owner's signature actually used.
    ///
    /// Reported rather than assumed. §10.3's prerequisite is a claim
    /// about the signature that was made, and an adapter that merely
    /// intended the profile would leave the three post-signing rows
    /// resting on an intention.
    pub authorization_profile: Option<AuthorizationProfile>,
    /// The witness item sizes of each input, in input order.
    ///
    /// The evidence behind the profile: a single 64-byte item is a
    /// taproot key-path signature carrying no sighash byte, which the
    /// reviewed digest reads as the default all-outputs mode.
    pub observed_witness_sizes: Vec<Vec<usize>>,
    /// The transaction the adapter materialized, where it built one.
    pub transaction_bytes: Option<Vec<u8>>,
}

impl NativeNormalizationResponse {
    /// Whether this response contradicts itself.
    ///
    /// A response saying the run never happened must carry no target
    /// observation. The witness sizes are counted among those: they are
    /// read out of the transaction the adapter built, and they are the
    /// evidence the authorization profile rests on, so admitting them
    /// beside a refused profile would leave the profile's own support
    /// standing where the profile may not.
    ///
    /// The claimed outputs are not counted. They are resolved from the
    /// claim before any mutation is applied, so they describe what was
    /// asked rather than what the target did.
    ///
    /// # Errors
    ///
    /// [`ResponseShapeDefect`] where the response is not a shape the
    /// protocol defines.
    pub const fn validate_shape(&self) -> Result<(), ResponseShapeDefect> {
        if !self.observed_layer.is_target_verdict()
            && (self.transaction_bytes.is_some()
                || !self.observed_outputs.is_empty()
                || self.authorization_profile.is_some()
                || !self.observed_witness_sizes.is_empty())
        {
            return Err(ResponseShapeDefect::InfrastructureResponseCarriesObservation);
        }
        Ok(())
    }
}

/// Which half of the §13 lifecycle a step is.
///
/// The two roles are not two configurations of one step. Process A holds
/// owner-private material and publishes; Process B holds none and reads.
/// Naming them in the case identity is what lets the adapter dispatch on
/// the role rather than on which optional subject member happens to be
/// present.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum LifecycleStepRole {
    /// Process A: build the object and publish the record.
    Construct,
    /// Process B: read the record and reconstruct the public fact.
    Verify,
}

/// One lifecycle step's identity.
///
/// A single member, and deliberately not the shape any other record
/// carries, on the reasoning [`NormalizationCaseId`] states: the adapter
/// tells the record kinds apart by shape alone.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LifecycleCaseId {
    /// Which role this step is.
    pub lifecycle: LifecycleStepRole,
}

/// What Process A is asked to build and publish.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LifecycleConstructSubject {
    /// The claim whose normalized output becomes the published object.
    pub claim: NormalizationClaim,
    /// Whether to spend the object afterwards, building the stale row.
    pub supersede: bool,
}

/// What Process B is asked to read.
///
/// The handoff and nothing else. The subject is the whole of what
/// crosses the process boundary, so a member added here would be a
/// member the lifecycle claim does not actually rest on the public
/// record `(´[PLAN-rule:guide11-exec:request-subject]´)`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LifecycleVerifySubject {
    /// The public record, exactly as Process A published it.
    pub handoff: PublicHandoff,
}

/// The subject of one lifecycle step.
///
/// Untagged because the role is already stated in the case identity, and
/// a second discriminator could disagree with the first. The two
/// variants refuse unknown members and share none of their own, so the
/// shapes are distinguishable without one.
///
/// Both subjects are boxed. Each carries a record of its own — a whole
/// claim on one side, a whole published handoff on the other — so an
/// unboxed enum would make every lifecycle request as large as whichever
/// happened to be bigger. The boxes are invisible on the wire.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum LifecycleSubject {
    /// Process A's subject.
    Construct(Box<LifecycleConstructSubject>),
    /// Process B's subject.
    Verify(Box<LifecycleVerifySubject>),
}

/// One lifecycle step, handed to the executor.
///
/// Carries the role and its subject. No expected outcome crosses this
/// boundary `(´[PLAN-rule:guide11-exec:request-subject]´)`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeLifecycleRequest {
    /// The protocol revision.
    pub schema: u32,
    /// The step being asked about.
    pub case: LifecycleCaseId,
    /// Exactly what to build or read.
    pub subject: LifecycleSubject,
}

/// One thing Process B looked for, and what it found.
///
/// # Neither side of a check is a verdict
///
/// A check records an expectation and an observation and whether they
/// agree, and stops there. Whether a run passes is the typed report's
/// judgement, made in the package that owns the matrix — for the reason
/// `G11-W7-06` recorded. An executor that decided a row here would be
/// supplying the answer it is graded against.
///
/// Both sides are carried as text. The values compared are of several
/// kinds — an identifier, an amount, a flag — and a member that were
/// sometimes a number and sometimes a string is the untyped value tree
/// this revision exists to remove. The agreement is determined by the
/// adapter over the values themselves, before either is spelled, so
/// nothing is decided by how they print.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LifecycleCheck {
    /// What was checked.
    pub check: String,
    /// What the record led Process B to expect.
    pub expected: String,
    /// What the chain actually said.
    pub observed: String,
    /// Whether the two agree, as the adapter compared them.
    pub agrees: bool,
}

/// One outpoint, as the target spells it.
///
/// One type rather than one per workload. An outpoint is a single
/// semantic object on this wire, and a second structurally identical
/// record for the same thing would be a lookalike the two halves of the
/// protocol could drift apart at
/// `(´[PLAN-rule:guide12-exec:typed-source]´)`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WireOutpoint {
    /// The transaction.
    pub txid: String,
    /// The output index.
    pub vout: u32,
}

/// The spend Process B built out of its own funds.
///
/// # What this is evidence of
///
/// Not ownership of the published object — the opposite. Process B
/// builds a transaction from funds it located itself, so that a failure
/// to spend the owner's object is a statement about ownership rather
/// than about a process that could not build anything at all.
/// [`Self::consumed_owner_object`] is the member that would falsify the
/// lane, and it is reported rather than assumed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LifecycleSpend {
    /// The transaction Process B built and confirmed.
    pub txid: String,
    /// What it paid, in satoshis.
    pub amount: u64,
    /// The address it paid, derived fresh in this process.
    pub destination: String,
    /// That address's script.
    pub destination_script: String,
    /// The outpoint it actually consumed.
    pub consumed_outpoint: WireOutpoint,
    /// Whether that outpoint was the owner's published object.
    pub consumed_owner_object: bool,
}

/// What one lifecycle step did.
///
/// # One record for both roles
///
/// Process A reports a handoff and what it published; Process B reports
/// checks and a spend. They are one type because they are one exchange,
/// and because the members each role leaves empty are exactly what
/// [`Self::validate_shape`] can then hold to.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeLifecycleResponse {
    /// The protocol revision.
    pub schema: u32,
    /// The step answered.
    pub case: LifecycleCaseId,
    /// What the step did.
    pub outcome: LifecycleOutcome,
    /// The record Process A published, where it published one.
    pub handoff: Option<PublicHandoff>,
    /// The authorization profile the owner's signature used.
    pub authorization_profile: Option<AuthorizationProfile>,
    /// The witness item sizes of each input, in input order.
    pub observed_witness_sizes: Vec<Vec<usize>>,
    /// The outputs the target's decoder reported.
    pub observed_outputs: Vec<ObservedOutput>,
    /// What Process B looked for, in the order it looked.
    pub checks: Vec<LifecycleCheck>,
    /// The spend Process B built, where it got that far.
    pub spend: Option<LifecycleSpend>,
    /// The transaction that consumed the object, where one was built.
    pub superseded_by: Option<String>,
    /// Why the stale row was not built, where it was not.
    pub supersede_failure: Option<String>,
    /// What the adapter said, where the step did not run.
    pub detail: Option<String>,
}

impl NativeLifecycleResponse {
    /// Whether this response contradicts itself, its role, or its own
    /// outcome.
    ///
    /// # A step that did not run observed nothing
    ///
    /// The oldest of the rules here, on exactly the reasoning
    /// [`NativeConservationResponse::validate_shape`] states: a handoff
    /// names a transaction that reached a block, a check reports what
    /// the chain said, and a spend is a transaction that was confirmed.
    /// A step that reached no verdict produced none of them, so any of
    /// them beside such an outcome is a value with no possible
    /// provenance.
    ///
    /// It was also, until `G13-R14`, the *only* rule — which left the
    /// two halves of one record free to answer each other's questions.
    ///
    /// # The census, over both roles and every outcome
    ///
    /// Every member of this record belongs to exactly one role. Process
    /// A publishes the handoff and reports what it signed and created:
    /// the authorization profile, the witness sizes that profile rests
    /// on, the decoded outputs, and the stale row it did or did not
    /// build. Process B reports what it looked for and the spend it
    /// made of its own funds. Only the two failure reasons — the detail
    /// and the reason a stale row was not built — are not observations,
    /// and a failure is entitled to a reason.
    ///
    /// The outcomes divide the same way. `Constructed` is a verdict
    /// only Process A reaches; `Verified` and every refusal beneath it
    /// are verdicts only Process B reaches; the two failures belong to
    /// neither and are reachable from both.
    ///
    /// The match below is over the pair, and it is exhaustive on
    /// purpose. A role or an outcome added later has no shape rule
    /// until one is written here, and the compiler is what says so — a
    /// catch-all arm would let a new pair be admitted, or refused, by a
    /// rule nobody chose for it.
    ///
    /// # What each verdict owes
    ///
    /// A construct step that published owes the handoff, which is the
    /// one thing publishing produces. A verify step owes its checks
    /// whatever it concluded — every refusal is reached by looking
    /// something up and finding it wrong, so a refusal with no checks
    /// behind it is a verdict with no evidence — and a verification
    /// that succeeded owes the spend as well, because building one is
    /// the last of the things `Verified` claims were done.
    ///
    /// # Errors
    ///
    /// [`ResponseShapeDefect`] where the response is not a shape the
    /// protocol defines.
    pub const fn validate_shape(&self) -> Result<(), ResponseShapeDefect> {
        // Process A's members, and Process B's. The two lists together
        // with the two reasons exhaust this record, so a member added
        // later has to be placed in one of them before it can be
        // written down.
        let publisher_members = self.handoff.is_some()
            || self.authorization_profile.is_some()
            || !self.observed_witness_sizes.is_empty()
            || !self.observed_outputs.is_empty()
            || self.superseded_by.is_some();
        let reader_members = !self.checks.is_empty() || self.spend.is_some();

        match (self.case.lifecycle, self.outcome) {
            (LifecycleStepRole::Construct, LifecycleOutcome::Constructed) => {
                if reader_members {
                    return Err(ResponseShapeDefect::LifecycleResponseMismatchesStep);
                }
                if self.handoff.is_none() {
                    return Err(ResponseShapeDefect::LifecycleStepOmitsObservation);
                }
            }
            (LifecycleStepRole::Verify, LifecycleOutcome::Verified) => {
                if publisher_members || self.supersede_failure.is_some() {
                    return Err(ResponseShapeDefect::LifecycleResponseMismatchesStep);
                }
                if self.checks.is_empty() || self.spend.is_none() {
                    return Err(ResponseShapeDefect::LifecycleStepOmitsObservation);
                }
            }
            // A refusal is Process B's too, and it stops short of the
            // spend: the transaction of its own funds is built after
            // everything the record claimed has been confirmed, so a
            // refusal that reports one is reporting work its own
            // verdict says was never reached.
            (
                LifecycleStepRole::Verify,
                LifecycleOutcome::RefusedEvidenceAbsent
                | LifecycleOutcome::RefusedCopiedEvidence
                | LifecycleOutcome::RefusedOutputAbsent
                | LifecycleOutcome::RefusedOutputNotExplicit
                | LifecycleOutcome::RefusedOutputSpent
                | LifecycleOutcome::RefusedWrongChainContext,
            ) => {
                if publisher_members || self.supersede_failure.is_some() || self.spend.is_some() {
                    return Err(ResponseShapeDefect::LifecycleResponseMismatchesStep);
                }
                if self.checks.is_empty() {
                    return Err(ResponseShapeDefect::LifecycleStepOmitsObservation);
                }
            }
            // Neither process reached the chain. The observation rule
            // is applied before the role rule here, and deliberately:
            // "this step did not run" is the more fundamental thing
            // wrong with such a record, and naming the role instead
            // would send a reader looking for a dispatch fault where
            // the fault is a claim about a run that did not happen.
            (
                _,
                LifecycleOutcome::ExecutorInfrastructureFailure
                | LifecycleOutcome::FixtureConstructionFailure,
            ) => {
                if publisher_members || reader_members {
                    return Err(ResponseShapeDefect::InfrastructureResponseCarriesObservation);
                }
                if matches!(self.case.lifecycle, LifecycleStepRole::Verify)
                    && self.supersede_failure.is_some()
                {
                    return Err(ResponseShapeDefect::LifecycleResponseMismatchesStep);
                }
            }
            (
                LifecycleStepRole::Construct,
                LifecycleOutcome::Verified
                | LifecycleOutcome::RefusedEvidenceAbsent
                | LifecycleOutcome::RefusedCopiedEvidence
                | LifecycleOutcome::RefusedOutputAbsent
                | LifecycleOutcome::RefusedOutputNotExplicit
                | LifecycleOutcome::RefusedOutputSpent
                | LifecycleOutcome::RefusedWrongChainContext,
            )
            | (LifecycleStepRole::Verify, LifecycleOutcome::Constructed) => {
                return Err(ResponseShapeDefect::LifecycleOutcomeMismatchesRole);
            }
        }
        Ok(())
    }
}

/// Which kind of target work one operation step asks for.
///
/// # Why the kinds are named on the wire rather than inferred
///
/// The two steps ask a node for entirely different things — create some
/// outputs, or judge a transaction — and they are told apart here rather
/// than by which subject member happens to parse, on exactly the
/// reasoning [`LifecycleStepRole`] states: an adapter that inferred the
/// kind from what was readable would be deciding what it was asked from
/// what happened to succeed.
///
/// # What this vocabulary deliberately does not name
///
/// Nothing in it is compact-ASH vocabulary. Issuing a disposable asset,
/// paying outputs to a witness program, and submitting a transaction are
/// things a caller can ask of a target with no operation semantics
/// whatsoever, and that is the property that lets this package own the
/// exchange without owning what the exchange is evidence *of*
/// `(´[PLAN-rule:guide12-exec:executor-ownership]´)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum OperationStepKind {
    /// Create spendable outputs at a stated witness program.
    Fund,
    /// Hand the target a complete transaction and report what it did.
    Submit,
    /// Create a coin out of the executor's own reserve holdings, at a
    /// program the executor is able to authorize a spend of.
    ///
    /// Distinct from [`Self::Fund`] because the caller does not choose
    /// the program. A funding step names the program its outputs pay to;
    /// this one cannot, because the whole point is that the executor
    /// keeps whatever the program commits to. What comes back is the
    /// program the executor picked, and the caller reads it rather than
    /// dictating it.
    FundSponsor,
    /// Authorize one input of an already-finalized transaction.
    SignSponsor,
    /// Materialize and mine outputs carrying an explicit protocol asset
    /// and a confidential value, under a named public fixture.
    ///
    /// Distinct from [`Self::Fund`] because the two ask for different
    /// objects and carry different subjects. A funding step states a
    /// program, a count, and an amount; this one states destinations and
    /// a binding, carries no amount at all, and asks for a
    /// representation the caller could not obtain by asking a wallet
    /// nicely. Reading one as the other would attach an explicit
    /// observation to a confidential obligation.
    FundConfidential,
    /// Create a coin out of the executor's own reserve whose VALUE is
    /// committed and whose ASSET stays explicit, at a program the
    /// executor is able to authorize a spend of.
    ///
    /// # Why it is neither of the two kinds it sits between
    ///
    /// Not [`Self::FundSponsor`], because that step reports an amount
    /// the caller reads off the chain, and this one has no amount to
    /// report: what comes back is a commitment, and the number behind it
    /// lives in the fixture rather than in the answer. Not
    /// [`Self::FundConfidential`], because that step funds coins of a
    /// protocol asset the caller names and never spends them, while this
    /// one funds coins of a reserve the EXECUTOR names and exists so
    /// that one of them can be spent.
    ///
    /// # Why the subject names no asset
    ///
    /// The reserve is the executor's fact. A caller learns it from an
    /// earlier sponsor-funding answer, and restating it here would be
    /// handing the executor back something it said first. It is not
    /// unchecked for being unstated: the reserve is folded into the
    /// fixture digest, so a caller whose idea of the reserve differs
    /// from the executor's fails the binding rather than proceeding.
    FundConfidentialSponsor,
}

impl std::fmt::Display for OperationStepKind {
    /// # One spelling, and why it has to be the wire's
    ///
    /// The serde representation of this enum is snake case, and an
    /// adapter reads the kind out of the record's own field. A
    /// `Display` that rendered a kind differently would be a second
    /// authored spelling of one word — harmless while a reader is
    /// human, and not harmless at all the moment anything compares the
    /// two `(´[PLAN-rule:guide12-exec:typed-source]´)`. The census test
    /// below holds them equal for every variant.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::Fund => "fund",
            Self::Submit => "submit",
            Self::FundSponsor => "fund_sponsor",
            Self::SignSponsor => "sign_sponsor",
            Self::FundConfidential => "fund_confidential",
            Self::FundConfidentialSponsor => "fund_confidential_sponsor",
        };
        formatter.write_str(text)
    }
}

/// One operation step's identity.
///
/// Two members, and deliberately not the shape any other record carries,
/// on the reasoning [`NormalizationCaseId`] states: the adapter tells the
/// record kinds apart by shape alone. A primitive case is a group and an
/// ordinal, a compound one a relation and a name, a conservation row an
/// ordinal and a name, a normalization row one mutation, a lifecycle step
/// one role; an operation step is a kind and a caller's name for it.
///
/// The name is the caller's, and the harness reads nothing out of it. It
/// exists so a transcript can be indexed by something the caller
/// recognizes without the harness having to understand what the caller
/// recognized.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OperationCaseId {
    /// Which kind of target work this step is.
    pub operation: OperationStepKind,
    /// The caller's own name for this step.
    pub step: String,
}

impl std::fmt::Display for OperationCaseId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}/{}", self.operation, self.step)
    }
}

/// What a funding step is asked to create.
///
/// # Amounts and programs, not roles
///
/// The step states a witness program, a count, and an amount. It does not
/// state what the outputs are *for*: which of them will carry an
/// operation's inputs, and what an operation will do with them, is the
/// caller's question and never travels here.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TargetFundingSubject {
    /// Whether this step must issue the disposable test asset.
    ///
    /// Where false, the asset the outputs carry is named below and was
    /// issued by an earlier step of the same run.
    pub issue_asset: bool,
    /// The asset the outputs must carry, where an earlier step issued
    /// one.
    ///
    /// Absent exactly when [`Self::issue_asset`] is set: a step that
    /// issues the asset cannot also name it, because the target has not
    /// chosen it yet.
    pub asset: Option<String>,
    /// The witness program each created output pays to.
    pub output_program: Vec<u8>,
    /// How many outputs to create at that program.
    pub outputs: u8,
    /// What each output holds, in the asset's smallest unit.
    pub amount_per_output: u64,
}

/// What a submission step is asked to judge.
///
/// The bytes and nothing else. No expected layer, no expected identity,
/// and no class: under revision 4 the answer stays with the caller
/// exactly as it does for every other workload
/// `(´[PLAN-rule:guide11-exec:request-subject]´)`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TargetSubmissionSubject {
    /// The exact transaction to hand the target.
    pub transaction_bytes: Vec<u8>,
}

/// What a sponsor-funding step is asked to create.
///
/// # Why no program is stated
///
/// Every other funding request names the program its outputs pay to.
/// This one deliberately does not, and the omission is the whole record:
/// the coin exists to be spent later by the executor itself, so the
/// program has to be one the executor can authorize. A caller that
/// named it would be choosing where somebody else's key lives, and the
/// only honest answer to "which program can you sign for" comes back
/// from the executor.
///
/// The asset is not stated either, for the same reason. What a
/// development network uses as its reserve is the network's own fact,
/// and the executor reports which asset it paid in.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TargetSponsorFundingSubject {
    /// How many sponsor coins to create.
    pub sponsor_outputs: u8,
    /// What each of them must hold, in the reserve asset's smallest
    /// unit.
    ///
    /// Exact rather than a minimum. A sponsor input that carries more
    /// than the transaction spends leaves the reserve asset unbalanced,
    /// and the caller — not the executor — is the side that knows what
    /// the transaction it is about to build will spend.
    pub amount_per_sponsor_output: u64,
}

/// Which sighash profile a signing step selects, on the wire.
///
/// One variant, and named rather than assumed, on exactly the reasoning
/// `transaction`'s own profile type states: a profile committing to
/// fewer outputs would let an authorization be replayed against a
/// transaction whose protected outputs differ.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum WireSighashProfile {
    /// Commits to every input and every output of the transaction.
    AllInputsAllOutputs,
}

/// What a sponsor-signing step is asked to authorize.
///
/// # The request carries finalized bytes, not a template
///
/// The transaction is complete except for the witness of the input
/// being authorized. That is what makes the echo below checkable: the
/// executor returns the bytes it actually signed, and the caller
/// compares them with the bytes it sent, so an executor that authorized
/// some other transaction is caught by comparison rather than trusted
/// not to.
///
/// # What is deliberately absent
///
/// No expected layer, no expected verdict, and nothing about what the
/// transaction is for. A signing step is not a judgement and the
/// executor is told nothing that would let it produce one
/// `(´[PLAN-rule:guide11-exec:request-subject]´)`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TargetSponsorSigningSubject {
    /// The exact finalized transaction the authorization is about.
    pub finalized_transaction: Vec<u8>,
    /// Which input is being authorized.
    pub sponsor_input_index: u16,
    /// The coin that input spends, so the executor knows which of its
    /// own programs is in question without inferring it.
    pub sponsor_outpoint: WireOutpoint,
    /// The profile the authorization must commit under.
    pub sighash_profile: WireSighashProfile,
}

/// Which representation a confidential funding step asks for.
///
/// # One variant, and why it is an enum
///
/// The one member names the exact hybrid tuple the target admits: an
/// explicit protocol asset paired with a confidential value, which uses
/// the unblinded asset generator, requires a rangeproof, and requires
/// the surjection-proof field to be empty. A second representation is an
/// added variant a peer must advertise before it can be selected, rather
/// than a silent change of what the existing word meant.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum FundingRepresentationProfile {
    /// An explicit protocol asset carrying a confidential value.
    ExplicitAssetConfidentialValue,
}

/// Which custody model a confidential funding step runs under.
///
/// One variant, and it is the accepted model: one party holds every
/// opening, every opening is a published test fixture, and the material
/// is disposable and destroyed with the chain. The three nonrecommended
/// models are closed as directions and are not variants here.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum FundingCustodyProfile {
    /// Deterministic central public fixtures.
    CentralPublicFixtures,
}

/// Which materializer a confidential funding step selects.
///
/// One variant, naming this guide's own deterministic recipe. It is
/// candidate vocabulary: the version in the spelling is the recipe's
/// generation and is not a release identity, an architecture operation,
/// or a schema identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum FundingMaterializerProfile {
    /// The deterministic fixture materializer of this guide.
    GuideCtfDeterministicV1,
}

/// The wire encoding of one [`ReproducibilityContract`].
///
/// # A serialization adapter, not a second vocabulary
///
/// The contract itself lives in `target-elements`, which serializes
/// nothing and depends on nothing, so the encoding has to live on this
/// side. What it must not become is a second spelling: the functions
/// below read and write the contract's OWN code rather than restating
/// the variant names, so there is exactly one word per contract in the
/// workspace and a contract added there needs no edit here to keep the
/// two in step `(´[PLAN-rule:guide12-exec:typed-source]´)`.
mod reproducibility_contract_wire {
    use serde::{Deserialize, Deserializer, Serializer, de::Error as _};
    use target_elements::ReproducibilityContract;

    /// Writes the contract's own code.
    ///
    /// The reference is the serialization framework's calling
    /// convention and not a choice: a `with` module is handed the field
    /// by reference whatever its width.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub fn serialize<S>(
        contract: &ReproducibilityContract,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(contract.code())
    }

    /// Reads one code, refusing any this vocabulary does not hold.
    ///
    /// No default and no fallback: an unknown contract is unknown, and
    /// answering it with the reference contract would move a run onto
    /// guarantees nobody selected.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<ReproducibilityContract, D::Error>
    where
        D: Deserializer<'de>,
    {
        let code = String::deserialize(deserializer)?;
        ReproducibilityContract::from_code(&code)
            .ok_or_else(|| D::Error::custom(format!("unknown reproducibility contract: {code}")))
    }
}

/// The wire encoding of a set of [`ReproducibilityContract`] values.
///
/// The same adapter as [`reproducibility_contract_wire`], for the
/// advertised set. It reads and writes through that module rather than
/// beside it, so the two cannot disagree about a word.
mod reproducibility_contract_set_wire {
    use std::collections::BTreeSet;

    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use target_elements::ReproducibilityContract;

    /// One member, encoded as its code.
    #[derive(Serialize, Deserialize)]
    struct Member(#[serde(with = "super::reproducibility_contract_wire")] ReproducibilityContract);

    /// Writes the advertised contracts, in the set's own order.
    pub fn serialize<S>(
        contracts: &BTreeSet<ReproducibilityContract>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let members: Vec<Member> = contracts.iter().copied().map(Member).collect();
        members.serialize(serializer)
    }

    /// Reads the advertised contracts, refusing any unknown code.
    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<BTreeSet<ReproducibilityContract>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let members = Vec::<Member>::deserialize(deserializer)?;
        Ok(members.into_iter().map(|member| member.0).collect())
    }
}

/// The public identity of one confidential fixture case.
///
/// A public deterministic test identity, and not an adapter-local secret
/// handle. Its spelling encodes no amount, no unit, no asset, no
/// opening, no derivation value, no digest fragment, no retry result,
/// and no transaction identity; it is not a capability, not a secret,
/// and not an architecture or release identity.
///
/// The grammar the registry enforces belongs to the registry, and lookup
/// failure is a construction refusal rather than a target verdict. What
/// travels here is the spelling.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ConfidentialFixtureHandle(String);

impl ConfidentialFixtureHandle {
    /// States one handle.
    #[must_use]
    pub const fn new(spelling: String) -> Self {
        Self(spelling)
    }

    /// The handle's spelling.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ConfidentialFixtureHandle {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// The binding digest of one confidential fixture case.
///
/// A tagged hash over a framed transcript, computed by the registry. It
/// detects drift; it is not an identity anything is minted for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ConfidentialFixtureDigest([u8; 32]);

impl ConfidentialFixtureDigest {
    /// States one digest.
    #[must_use]
    pub const fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// The digest's bytes.
    #[must_use]
    pub const fn bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

/// The four profiles one confidential funding ceremony runs under.
///
/// Each is closed and each is selected before lookup. The reproducibility
/// contract is required, never optional, and never inferred: absence,
/// disagreement between two carriers of it, and comparison across it are
/// all refusals, and none of them is a downgrade.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfidentialFundingProfiles {
    /// The exact representation asked for.
    pub representation: FundingRepresentationProfile,
    /// Who holds the openings.
    pub custody: FundingCustodyProfile,
    /// Which materializer produces the outputs.
    pub materializer: FundingMaterializerProfile,
    /// Which reproducibility contract the run is judged under.
    #[serde(with = "reproducibility_contract_wire")]
    pub reproducibility_contract: target_elements::ReproducibilityContract,
}

/// What the request binds itself to.
///
/// The canonical request is a public-fixture handle and a digest, and it
/// carries the profiles. There is no member an amount could be written
/// into, which is the point rather than an omission.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfidentialFundingBinding {
    /// Which registered case this is.
    pub fixture_handle: ConfidentialFixtureHandle,
    /// The digest that case was registered under.
    pub fixture_digest: ConfidentialFixtureDigest,
    /// The profiles the ceremony selected.
    pub profiles: ConfidentialFundingProfiles,
}

/// One destination a confidential funding step pays.
///
/// The program and nothing else. What each destination HOLDS is a fact
/// of the registered fixture rather than of the request, and the
/// destination count is this list's length.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfidentialFundingDestination {
    /// The witness program this destination pays to.
    pub output_program: Vec<u8>,
}

/// What a confidential funding step is asked to create.
///
/// # Destinations and a binding, and no amount anywhere
///
/// The request carries no amount, no count, no opening, no blinder, and
/// no nonce or proof input. That is the canonical-request ruling
/// expressed as a type rather than as a discipline: the amounts live in
/// the registered fixture the handle names, and the digest is what
/// detects the fixture drifting out from under the request.
///
/// # Why the asset question is spelled exactly as the explicit arm
/// spells it
///
/// [`Self::issue_asset`] and [`Self::asset`] keep the existing arm's
/// meaning and its invariant — the asset is absent exactly when the step
/// issues it — because the protocol asset question is identical in both
/// arms, and a second spelling of it would be a second chance to get it
/// wrong.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TargetConfidentialFundingSubject {
    /// Whether this step must issue the disposable test asset.
    pub issue_asset: bool,
    /// The explicit protocol asset the outputs must carry, where an
    /// earlier step issued one.
    pub asset: Option<String>,
    /// Where the created outputs pay, in fixed order.
    pub destinations: Vec<ConfidentialFundingDestination>,
    /// What the request binds itself to.
    pub binding: ConfidentialFundingBinding,
}

/// The subject of one confidential SPONSOR funding step.
///
/// # Why the asset question is absent rather than optional
///
/// The sibling above asks for coins of a protocol asset, which an
/// earlier step of the same run issued, so the caller states which one.
/// This step asks for coins of the chain's RESERVE, which no step of a
/// run chooses: the executor reads it off the chain it was pointed at
/// and reports it, and a caller stating one would be handing back a fact
/// it learned from that report.
///
/// Unstated is not unchecked. The reserve is folded into the fixture
/// digest this subject binds itself to, so a caller that resolved its
/// own fixture against a different asset presents a digest the executor
/// does not reproduce, and the step refuses at the binding before any
/// commitment is built.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TargetConfidentialSponsorFundingSubject {
    /// Where the created outputs pay, in fixed order.
    ///
    /// The FIRST is the coin that will be spent, and it has to be the
    /// program the executor can authorize a spend of. The executor
    /// checks that rather than trusting it: a sponsor coin paid anywhere
    /// else is one nothing can spend, and the run would meet that two
    /// steps later as a signing refusal that looked like the
    /// candidate's.
    pub destinations: Vec<ConfidentialFundingDestination>,
    /// What the request binds itself to.
    pub binding: ConfidentialFundingBinding,
}

/// The subject of one operation step.
///
/// Untagged because the kind is already stated in the case identity, and
/// a second discriminator could disagree with the first. The two variants
/// refuse unknown members and share none of their own, so the shapes are
/// distinguishable without one — the same construction
/// [`LifecycleSubject`] uses, for the same reason.
///
/// Both subjects are boxed, so an operation request is not as large as
/// whichever variant happens to be bigger. The boxes are invisible on the
/// wire.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OperationSubject {
    /// A funding step's subject.
    Funding(Box<TargetFundingSubject>),
    /// A submission step's subject.
    Submission(Box<TargetSubmissionSubject>),
    /// A sponsor-funding step's subject.
    SponsorFunding(Box<TargetSponsorFundingSubject>),
    /// A sponsor-signing step's subject.
    SponsorSigning(Box<TargetSponsorSigningSubject>),
    /// A confidential funding step's subject.
    ///
    /// # Why the distinct arm is a fifth variant here
    ///
    /// [`TargetFundingSubject`] is not re-shaped into a tagged union.
    /// It is already a variant of an untagged enum, and that enum is
    /// untagged for a stated reason: the kind is in the case identity,
    /// and a second discriminator could disagree with the first. Making
    /// the funding subject itself a tagged union would put a second
    /// discriminator INSIDE a variant of an untagged enum, which is
    /// precisely the shape this file refuses. So the confidential arm is
    /// added exactly as the sponsor steps were — its own subject, its
    /// own step kind, its own capability.
    ///
    /// Untagged deserialization stays unambiguous because the member
    /// sets are disjoint under `deny_unknown_fields`: this subject
    /// carries `destinations` and `binding`, which no other subject
    /// declares, and declares no `output_program`, `outputs`, or
    /// `amount_per_output`. That disjointness is a correctness property
    /// and has its own test rather than a comment.
    ConfidentialFunding(Box<TargetConfidentialFundingSubject>),
    /// A confidential sponsor-funding step's subject.
    ///
    /// # How this stays distinguishable from the arm above it
    ///
    /// By the member the two do not share. Both carry `destinations` and
    /// `binding`; only the protocol arm carries `issue_asset`, and under
    /// `deny_unknown_fields` neither will read the other's record — the
    /// protocol arm refuses one with no `issue_asset` because the member
    /// is required, and this arm refuses one that has it because the
    /// member is unknown here. That is a structural difference rather
    /// than an ordering accident, and it has a test.
    ConfidentialSponsorFunding(Box<TargetConfidentialSponsorFundingSubject>),
}

impl OperationSubject {
    /// The step kind this subject belongs to.
    ///
    /// The one place the correspondence is stated. A request whose case
    /// identity and subject disagree is refused before it is sent, rather
    /// than at whichever end of the exchange noticed first.
    #[must_use]
    pub const fn kind(&self) -> OperationStepKind {
        match self {
            Self::Funding(_) => OperationStepKind::Fund,
            Self::Submission(_) => OperationStepKind::Submit,
            Self::SponsorFunding(_) => OperationStepKind::FundSponsor,
            Self::SponsorSigning(_) => OperationStepKind::SignSponsor,
            Self::ConfidentialFunding(_) => OperationStepKind::FundConfidential,
            Self::ConfidentialSponsorFunding(_) => OperationStepKind::FundConfidentialSponsor,
        }
    }

    /// The executor capability a step of this kind requires.
    ///
    /// The two sponsor steps require one capability rather than two, and
    /// that is a judgement about what is separable rather than an
    /// economy. Funding and submission were split because an executor
    /// can genuinely have one and not the other — a node that judges
    /// transactions perfectly well may hold no funds. Sponsor funding
    /// and sponsor signing are not like that: a coin the executor
    /// created but cannot authorize is a coin nothing can spend, and an
    /// executor that could authorize but holds no reserve has nothing to
    /// authorize a spend of. Splitting them would offer a combination
    /// neither half is usable in.
    #[must_use]
    pub const fn required_capability(&self) -> ExecutorCapability {
        match self {
            Self::Funding(_) => ExecutorCapability::TestFundingCeremony,
            Self::Submission(_) => ExecutorCapability::TargetTransactionSubmission,
            Self::SponsorFunding(_) | Self::SponsorSigning(_) => {
                ExecutorCapability::TestSponsorAuthorization
            }
            // Its own capability rather than the funding one, because
            // the two claims come apart: no reviewed stock interface
            // produces the hybrid representation, so an executor that
            // funds perfectly well may be unable to answer a single
            // confidential step.
            Self::ConfidentialFunding(_) => ExecutorCapability::ConfidentialValueTestFunding,
            // A third capability rather than either neighbour, because
            // the two claims it joins come apart at the retained field:
            // an executor that funds confidential receipts keeps
            // nothing about them, and one that authorizes explicit
            // sponsor coins keeps an amount. Neither is enough to sign a
            // spend of a coin whose value is a point.
            Self::ConfidentialSponsorFunding(_) => {
                ExecutorCapability::ConfidentialValueSponsorAuthorization
            }
        }
    }
}

/// One operation step, handed to the executor.
///
/// Carries the step's identity and its subject, and nothing else
/// `(´[PLAN-rule:guide11-exec:request-subject]´)`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeOperationRequest {
    /// The protocol revision.
    pub schema: u32,
    /// The step being asked for.
    pub case: OperationCaseId,
    /// Exactly what to do.
    pub subject: OperationSubject,
}

/// One output a funding step created.
///
/// # Why the target restates what it was asked for
///
/// The asset and the amount were stated in the request, and they come
/// back because a funding step is only useful if what it created is what
/// a later transaction can spend. The caller compares the two; the
/// executor is never told what the comparison is for, and an executor
/// that echoed the request instead of reading the chain is caught by the
/// script the target itself reports.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FundedOutput {
    /// Where the output is.
    pub outpoint: WireOutpoint,
    /// The asset it holds, in the target's own spelling.
    pub asset: String,
    /// The explicit amount it holds.
    pub amount_satoshis: u64,
    /// The output's script, as the target reports it.
    pub script: String,
}

/// One output a confidential funding step created.
///
/// # Two spellings are inherited rather than improved on
///
/// The script comes back as a string in the target's own rendering and
/// the asset comes back as a string in the target's own spelling, exactly
/// as [`FundedOutput`] carries them. Byte vectors would have been tidier
/// in isolation and would have given the two funding arms two different
/// renderings of one target fact, so that a comparison across arms became
/// a comparison of two authored spellings
/// `(´[PLAN-rule:guide12-exec:typed-source]´)`.
///
/// # The proof fields are declared in the target's serialization order
///
/// Surjection proof first, then rangeproof, so that a reader of this
/// record and a reader of the raw bytes are reading the same order. For
/// the hybrid representation the surjection proof is always empty and the
/// rangeproof is never empty, and both are declared anyway: an absent
/// field cannot be observed to be empty.
///
/// # Why the commitment and the nonce are byte vectors
///
/// Both are thirty-three-byte fields and neither is declared as a
/// thirty-three-byte array, because the serialization framework this
/// protocol is written in implements no array deserializer past
/// thirty-two. The width is therefore checked rather than asserted by
/// the type, and the two typed refusals that check it — an inadmissible
/// commitment encoding and an inadmissible nonce encoding — are exactly
/// the refusals the wire vocabulary already names.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfidentialFundedOutput {
    /// Where the output is.
    pub outpoint: WireOutpoint,
    /// The explicit protocol asset it carries, in the target's own
    /// spelling.
    pub explicit_asset: String,
    /// The serialized value commitment, prefix included.
    pub value_commitment: Vec<u8>,
    /// The serialized nonce field, prefix included.
    pub nonce: Vec<u8>,
    /// The output's script, as the target reports it.
    pub script: String,
    /// Which output-witness entry carries this output's proofs.
    pub output_witness_index: u32,
    /// The surjection proof, which the hybrid representation requires to
    /// be empty.
    pub surjection_proof: Vec<u8>,
    /// The rangeproof, which the hybrid representation requires to be
    /// present.
    pub rangeproof: Vec<u8>,
}

/// What the target holds after a confidential funding step was mined.
///
/// # Why the raw bytes travel
///
/// Because every output fact in a validated record is derived from
/// decoding them and from an independent target readback, and a request
/// echo can never stand in for either. The identities here are the
/// target's own; they are recomputed from these bytes rather than
/// believed `(´[PLAN-rule:guide12-exec:typed-source]´)`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MinedFundingReadback {
    /// The transaction identity the target reports.
    pub transaction_id: String,
    /// The witness transaction identity the target reports.
    pub witness_transaction_id: String,
    /// The block the transaction was mined into.
    pub block_hash: String,
    /// That block's height.
    pub block_height: u32,
    /// The raw mined transaction, witnesses included.
    pub raw_transaction: Vec<u8>,
}

/// What an executor says it can do with confidential funding.
///
/// # Advertisement constrains and never chooses
///
/// The ceremony plan selects the profiles and the reproducibility
/// contract its own report will claim; this record states which
/// selections the executor will accept, so that an unsupported selection
/// is refused BEFORE the request is written rather than downgraded after
/// it. An executor that supported only one contract could otherwise move
/// a run off the reference contract without anyone choosing to.
///
/// # Why it is optional, and why the option cannot disagree with the
/// capability
///
/// It is absent by default, so that an executor which never heard of
/// confidential funding writes a record this harness reads. What it may
/// not do is disagree with itself: advertising
/// [`ExecutorCapability::ConfidentialValueTestFunding`] without this
/// record, or this record without the capability, is refused at the
/// handshake rather than resolved in whichever direction is convenient.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfidentialFundingAdvertisement {
    /// The representations it will materialize.
    pub representation_profiles: BTreeSet<FundingRepresentationProfile>,
    /// The custody models it will run under.
    pub custody_profiles: BTreeSet<FundingCustodyProfile>,
    /// The materializers it implements.
    pub materializer_profiles: BTreeSet<FundingMaterializerProfile>,
    /// The reproducibility contracts it supports.
    #[serde(with = "reproducibility_contract_set_wire")]
    pub reproducibility_contracts: BTreeSet<target_elements::ReproducibilityContract>,
}

/// What the target did with one operation step.
///
/// # One record for both kinds
///
/// A funding step reports what it created; a submission step reports what
/// the target made of a transaction. They are one type because they are
/// one exchange, and because the members each kind leaves empty are
/// exactly what [`Self::validate_shape`] can then hold to — the same
/// construction [`NativeLifecycleResponse`] uses.
///
/// # The layer vocabulary is the shared one
///
/// [`ObservedOutcomeLayer`] already separates the two non-verdicts from
/// the four verdicts, and a submission is precisely the observation that
/// vocabulary was minted for: whether the target refused before any
/// script ran, refused in the script path, would relay-refuse, or
/// accepted. Restating it here under different names would be two
/// authored spellings of one distinction
/// `(´[PLAN-rule:guide12-exec:typed-source]´)`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeOperationResponse {
    /// The protocol revision.
    pub schema: u32,
    /// The step answered.
    pub case: OperationCaseId,
    /// Where the step ended up, as the adapter observed it.
    pub observed_layer: ObservedOutcomeLayer,
    /// What the target or the adapter said, verbatim and unmapped.
    pub observed_detail: Option<String>,
    /// The disposable asset a funding step issued, where it issued one.
    pub issued_asset: Option<String>,
    /// The outputs a funding step created, in creation order.
    pub funded_outputs: Vec<FundedOutput>,
    /// The outputs a confidential funding step created, in creation
    /// order.
    ///
    /// NOT defaulted, and the contrast with the sponsor members below is
    /// the argument: those were defaulted so that adding them was not a
    /// revision, and this one is not defaulted so that adding it IS one.
    /// A revision-4 executor answering a confidential request with
    /// silence in this member is exactly what the revision exists to
    /// prevent.
    pub confidential_funded_outputs: Vec<ConfidentialFundedOutput>,
    /// What the target held after the confidential funding transaction
    /// was mined.
    ///
    /// Not defaulted either, for the same reason.
    pub mined_readback: Option<MinedFundingReadback>,
    /// The identity the target gave a submitted transaction, where it
    /// took one.
    pub accepted_txid: Option<String>,
    /// The witness stack a sponsor-signing step produced, bottom item
    /// first.
    ///
    /// Defaulted, so an executor that predates the sponsor steps and
    /// never writes the member still produces a record this harness
    /// reads. That is what keeps the addition from being a revision:
    /// nothing an executor already wrote changes shape.
    #[serde(default)]
    pub sponsor_witness: Vec<Vec<u8>>,
    /// The exact bytes a sponsor-signing step authorized.
    ///
    /// # Why the executor echoes what it was handed
    ///
    /// So the caller can compare rather than trust. An executor that
    /// authorized some other transaction returns some other bytes, and
    /// the comparison is an exact byte comparison
    /// `(´[PLAN-rule:guide12-exec:typed-source]´)`.
    ///
    /// The echo on its own is weak — an executor could echo without
    /// looking — and it is not what the evidence rests on. What rests on
    /// nothing but the target is the submission: an authorization
    /// produced over different bytes fails the target's own verification
    /// and the transaction is refused. The echo catches the honest
    /// mistake early; the target catches the rest.
    #[serde(default)]
    pub signature_bound_to: Option<Vec<u8>>,
    /// What the target reported the work cost.
    pub resources: NativeResourceObservation,
}

impl NativeOperationResponse {
    /// Whether this response contradicts itself.
    ///
    /// Four rules, and each one closes a way for a report to state a
    /// fact no run produced.
    ///
    /// A step that reached no target verdict observed nothing, on exactly
    /// the reasoning [`NativeConservationResponse::validate_shape`]
    /// states: an outpoint is a coin the target created, an issued asset
    /// is an identity the target chose, and a transaction identity is one
    /// the target computed over bytes it accepted.
    ///
    /// A step's answer must belong to the kind that was asked. A funding
    /// step that named an accepted transaction, or a submission that
    /// reported created outputs, is answering a question it was not
    /// asked, and reading either as evidence would attach one kind of
    /// observation to the other kind's obligation.
    ///
    /// An accepted step must report what its kind is defined to produce.
    /// An acceptance with nothing to show for it is indistinguishable
    /// from an adapter that returned the layer without doing the work.
    ///
    /// A *refused* step must report none of it, which is the rule
    /// `G13-R14` found missing. The three rules above admitted a
    /// rejected submission carrying an accepted transaction identity, a
    /// refused funding step carrying the asset it did not issue and the
    /// coins it did not create, and a refused signing step carrying a
    /// witness stack and the bytes it was bound to. Each of those is one
    /// record answering its own question twice, and a consumer reading
    /// the artifact rather than the layer gets the opposite verdict from
    /// the same row.
    ///
    /// # The member census, by kind
    ///
    /// Each kind owns the members it can produce and carries none of the
    /// others under any outcome:
    ///
    /// ```text
    /// fund               issued_asset, funded_outputs
    /// submit             accepted_txid, mined_readback
    /// fund_sponsor       funded_outputs
    /// sign_sponsor       sponsor_witness, signature_bound_to
    /// fund_confidential  issued_asset, confidential_funded_outputs,
    ///                    mined_readback
    /// ```
    ///
    /// The confidential arm owns two members no other kind may carry,
    /// and it may not carry the explicit arm's outputs. That second half
    /// is what makes an explicit answer to a confidential request a
    /// contradiction in the record's own shape rather than a
    /// disappointment discovered later: a step asked for a committed
    /// value and answered with scalars has answered a question it was
    /// not asked.
    ///
    /// An owned member is present exactly when the target accepted the
    /// step — required of an acceptance, refused of a rejection — with
    /// one exception that is a choice rather than an oversight: a
    /// funding step's issued asset is optional on acceptance, because a
    /// step paying an asset an earlier step already issued creates coins
    /// without choosing an identity.
    ///
    /// The detail is not counted, on the same ground the lifecycle
    /// response gives: a failure is entitled to a reason. Neither are the
    /// resource figures, which are the target's own accounting for bytes
    /// it judged — a refusal is a verdict, and the transaction it refused
    /// still has a weight the node reports.
    ///
    /// This half asks which members belong to a kind at all. What a kind
    /// OWES on an acceptance, and what it may not carry on a refusal, is
    /// the other half and lives in
    /// [`Self::validate_observation_for_kind`].
    ///
    /// # Errors
    ///
    /// [`ResponseShapeDefect`] where the response is not a shape the
    /// protocol defines.
    pub const fn validate_shape(&self) -> Result<(), ResponseShapeDefect> {
        let issues = self.issued_asset.is_some();
        let creates_coins = !self.funded_outputs.is_empty();
        let submits = self.accepted_txid.is_some();
        let authorizes = !self.sponsor_witness.is_empty() || self.signature_bound_to.is_some();
        let creates_confidential_coins = !self.confidential_funded_outputs.is_empty();
        let reads_back = self.mined_readback.is_some();

        if !self.observed_layer.is_target_verdict() {
            return if issues
                || creates_coins
                || submits
                || authorizes
                || creates_confidential_coins
                || reads_back
                || self.resources.observes_interpreter()
            {
                Err(ResponseShapeDefect::InfrastructureResponseCarriesObservation)
            } else {
                Ok(())
            };
        }

        // An authorization belongs to the one step that asks for one.
        // Any other kind reporting one would be attaching an
        // authorization to an obligation that never requested it.
        if !matches!(self.case.operation, OperationStepKind::SignSponsor) && authorizes {
            return Err(ResponseShapeDefect::OperationResponseMismatchesStep);
        }

        // A confidential observation belongs to the steps that ask for
        // one, on exactly the reasoning above. There are two of them
        // now, and they are the two that create coins whose value is a
        // commitment: the protocol arm and the reserve arm. They report
        // through ONE member rather than two, because what they report
        // is the same observation -- an outpoint, an explicit asset, a
        // committed value and the proof that goes with it -- and which
        // step asked for it is already in the case identity.
        if !matches!(
            self.case.operation,
            OperationStepKind::FundConfidential | OperationStepKind::FundConfidentialSponsor
        ) && creates_confidential_coins
        {
            return Err(ResponseShapeDefect::OperationResponseMismatchesStep);
        }

        // A mined readback belongs to the kinds that put bytes on a
        // chain: the two confidential funding steps, which mine what
        // they materialized, and the submission step, which mines the
        // candidate it was handed. The member is shared rather than
        // duplicated because it is one observation — what the node
        // reports for a transaction it has confirmed — and a second
        // member of the same shape would let two kinds drift into two
        // spellings of one fact.
        if !matches!(
            self.case.operation,
            OperationStepKind::FundConfidential
                | OperationStepKind::FundConfidentialSponsor
                | OperationStepKind::Submit
        ) && reads_back
        {
            return Err(ResponseShapeDefect::OperationResponseMismatchesStep);
        }

        self.validate_observation_for_kind()
    }

    /// The per-kind half of [`Self::validate_shape`]: what each step
    /// owes on acceptance, and what it may not carry on a refusal.
    ///
    /// Split from its caller because the two ask different questions.
    /// The caller asks which members belong to a kind AT ALL, which is a
    /// statement about the vocabulary; this asks whether the members a
    /// kind may carry are the ones an acceptance or a refusal of it
    /// entails, which is a statement about one answer. The predicates
    /// are recomputed here rather than passed in: each is one read of
    /// one member, and threading six booleans through a boundary would
    /// make the split look like a shared calculation instead of two
    /// separate questions over the same record.
    ///
    /// The match over the step kinds is exhaustive and stays that way. A
    /// kind added later has no shape rule until one is written here, and
    /// the compiler is what says so — a catch-all arm would let a new
    /// kind be admitted, or refused, by a rule nobody chose for it.
    ///
    /// # Errors
    ///
    /// [`ResponseShapeDefect`] where the response is not a shape the
    /// protocol defines.
    const fn validate_observation_for_kind(&self) -> Result<(), ResponseShapeDefect> {
        let issues = self.issued_asset.is_some();
        let creates_coins = !self.funded_outputs.is_empty();
        let submits = self.accepted_txid.is_some();
        let authorizes = !self.sponsor_witness.is_empty() || self.signature_bound_to.is_some();
        let creates_confidential_coins = !self.confidential_funded_outputs.is_empty();
        let reads_back = self.mined_readback.is_some();
        let accepted = matches!(self.observed_layer, ObservedOutcomeLayer::Accepted);
        match self.case.operation {
            OperationStepKind::Fund => {
                if submits {
                    return Err(ResponseShapeDefect::OperationResponseMismatchesStep);
                }
                if accepted {
                    if !creates_coins {
                        return Err(ResponseShapeDefect::AcceptedOperationOmitsObservation);
                    }
                } else if issues || creates_coins {
                    return Err(ResponseShapeDefect::RefusedOperationCarriesObservation);
                }
            }
            // A submission owes two halves on acceptance, for the same
            // reason the confidential arm does: the identity the target
            // computed over the bytes it took, and the bytes it reports
            // for that identity once the transaction is confirmed. An
            // identity alone cannot be read back, and a consumer that
            // has to re-derive the accepted witness from what it
            // submitted is checking its own value against itself.
            OperationStepKind::Submit => {
                if issues || creates_coins {
                    return Err(ResponseShapeDefect::OperationResponseMismatchesStep);
                }
                if accepted {
                    if !submits || !reads_back {
                        return Err(ResponseShapeDefect::AcceptedOperationOmitsObservation);
                    }
                } else if submits || reads_back {
                    return Err(ResponseShapeDefect::RefusedOperationCarriesObservation);
                }
            }
            // A sponsor-funding step creates coins and issues nothing: a
            // reserve the executor already holds is not an identity it
            // chose, and a coin is not a transaction the target took an
            // identity for.
            OperationStepKind::FundSponsor => {
                if submits || issues {
                    return Err(ResponseShapeDefect::OperationResponseMismatchesStep);
                }
                if accepted {
                    if !creates_coins {
                        return Err(ResponseShapeDefect::AcceptedOperationOmitsObservation);
                    }
                } else if creates_coins {
                    return Err(ResponseShapeDefect::RefusedOperationCarriesObservation);
                }
            }
            // A signing step creates nothing and submits nothing. What it
            // owes on acceptance is both halves of an authorization: the
            // stack, and the bytes that stack was produced against. One
            // without the other is unusable — a stack nobody can bind to
            // a transaction, or a binding with nothing to apply.
            OperationStepKind::SignSponsor => {
                if submits || issues || creates_coins {
                    return Err(ResponseShapeDefect::OperationResponseMismatchesStep);
                }
                if accepted {
                    if self.sponsor_witness.is_empty() || self.signature_bound_to.is_none() {
                        return Err(ResponseShapeDefect::AcceptedOperationOmitsObservation);
                    }
                } else if authorizes {
                    return Err(ResponseShapeDefect::RefusedOperationCarriesObservation);
                }
            }
            // A confidential funding step creates coins and may issue
            // the asset, exactly as the explicit arm may; what it may
            // not do is answer with the explicit arm's outputs, and what
            // it owes on acceptance is both halves of its own
            // observation. Outputs with no mined readback are proofs
            // nobody can check against a chain, and a readback with no
            // outputs is a block identity with nothing in it.
            OperationStepKind::FundConfidential => {
                if submits || creates_coins {
                    return Err(ResponseShapeDefect::OperationResponseMismatchesStep);
                }
                if accepted {
                    if self.confidential_funded_outputs.is_empty() || !reads_back {
                        return Err(ResponseShapeDefect::AcceptedOperationOmitsObservation);
                    }
                } else if issues || creates_confidential_coins || reads_back {
                    return Err(ResponseShapeDefect::RefusedOperationCarriesObservation);
                }
            }
            // The reserve arm owes what the protocol arm owes and one
            // thing less: it may not issue. The protocol arm may,
            // because the asset its coins carry is one a run brings into
            // existence; the reserve is the chain's already, and a step
            // reporting that it issued the reserve would be reporting
            // something no run can do.
            OperationStepKind::FundConfidentialSponsor => {
                if submits || creates_coins || issues {
                    return Err(ResponseShapeDefect::OperationResponseMismatchesStep);
                }
                if accepted {
                    if self.confidential_funded_outputs.is_empty() || !reads_back {
                        return Err(ResponseShapeDefect::AcceptedOperationOmitsObservation);
                    }
                } else if creates_confidential_coins || reads_back {
                    return Err(ResponseShapeDefect::RefusedOperationCarriesObservation);
                }
            }
        }
        Ok(())
    }
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
    ///
    /// One class covers both causes because the target names neither:
    /// a script-number decode failure surfaces as an unnamed error
    /// `(´[PLAN-obs:upstream:eg-004]´)`, which is the string the
    /// reviewed adapter maps here.
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
    ///
    /// The coarse arithmetic string behind it is an upstream friction
    /// `(´[PLAN-obs:upstream:eg-003]´)`. This class exists only because
    /// of it: were the causes ever named apart, the fixtures admitting
    /// this class alongside a contract cause would each narrow to one.
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
/// No RPC exposes that state at all, which is an upstream friction
/// `(´[PLAN-obs:upstream:eg-011]´)`; the optional figures here are its
/// shape in the protocol, and an interface that exposed the interpreter
/// would make them measurements rather than absences.
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
