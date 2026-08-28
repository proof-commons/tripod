//! The validated live-transfer safety report (§13.2, §13.5, §13.6).
//!
//! §13.2 asks one question — did every valid transfer preserve the exact
//! semantic relation, and did every required invalid transfer fail at its
//! owning boundary — and names twelve typed carriers the answer travels
//! in. §13.5 says the gate accepts a validated wrapper rather than a raw
//! report, names fourteen possible recomputations, and excludes ten
//! volatile fields from the canonical bytes. This module is all three.
//!
//! # The wrapper is the whole point
//!
//! [`LiveTransferSafetyReport`] is a value a caller can build, and it
//! establishes nothing. [`ValidatedLiveTransferSafetyReport`] has private
//! fields and one constructor, [`validate_live_safety_report`], which
//! records an item only after its comparison ran. A report whose summary
//! disagreed with its own rows is refused rather than published, and an
//! absent run binding cannot manufacture six passing run comparisons.
//!
//! # Timing is not in here, and that is structural
//!
//! §13.6 puts timing in a separate noncanonical diagnostic report, and
//! [`LiveSafetyDiagnostics`] is it. The canonical renderer takes a
//! validated report and nothing else, so the ten volatile fields are not
//! *filtered* out of the bytes — they are not reachable from the value
//! the renderer reads. `the_canonical_bytes_carry_no_volatile_field`
//! checks that no other carrier smuggles one in anyway.
//!
//! # A report is not a verdict about the target
//!
//! A report every one of whose rows is blocked is as valid as one whose
//! every row is discharged. What changes is the completeness token, and
//! §13.5's bar is that a report may not call itself complete while a
//! required row is unanswered. That is the honest state today, and the
//! report says so in a typed field rather than by the absence of
//! failures.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use compiler::live_transfer_plan::LiveTransferRepresentationPlan;
use target_elements::TargetProjection;
use target_elements::opcode::LeafVersion;
use target_elements_conformance::constructor::tagged;
use target_elements_conformance::protocol::ObservedOutcomeLayer;
use transaction::bytes::SerializedFieldLocator;
use transaction::taproot::{CONTROL_BASE_BYTES, DIGEST_BYTES, TAPROOT_LEAF_MASK};
use transaction::{AssetField, TargetTransaction, Txid, ValueField};

use crate::live_evidence::{
    LiveEvidenceCensus, LiveInfrastructureBlocker, LiveRowStanding, LiveTransferEvidencePlan,
    RecordedObservation, ValidatedCorpusEvidence, blocker_census,
};
use crate::live_safety::{LiveReportRequirement, LiveSafetySection};
use crate::matrix::EvidenceBoundary;

/// The schema of the canonical rendered safety report.
///
/// Stated in the bytes so a reader never has to infer which revision a
/// file is: a report whose field set changed under a reader that assumed
/// the old one would be read wrong rather than refused.
///
/// Revision 2 adds the `operation_vocabulary_closed` census line and the
/// `operation-vocabulary-closed` outstanding spelling. A revision-1
/// reader summing the census lines it knows would find them short of the
/// row count, which is exactly the misreading a stated schema exists to
/// turn into a refusal.
///
/// Revision 3 adds the evidence corpus ledger, the
/// `recorded_observation_unbound` census, every observation bucket, and
/// the `native_refusal_at_unexpected_boundary` census and outstanding
/// spelling. It also makes `failed` reachable. Schema 2 is hard-rejected:
/// it has neither the ledger nor the boundary distinction and is never
/// silently reinterpreted as evidence-bearing schema 3.
///
/// Revision 4 separates report-layer requirements from observations,
/// validates those requirements against the exact canonical bytes, and
/// records required and observed report-layer census buckets separately.
/// Schema 3 is hard-rejected because its plan-derived report observations
/// are capability claims rather than validated evidence.
///
/// Revision 5 binds every run to the archive bytes it came from, a closed
/// protocol revision, algorithm-tagged fixture digests, and typed request
/// roles. Schema 4 is hard-rejected because it cannot distinguish a
/// historical-v1 digest from a forward-v2 digest or detect a revision lie.
/// Schema 5 was never persisted as a report document and is RETIRED; it
/// has no historical reader.
///
/// Revision 6 adds the narrow forward-capture vocabulary: one composite
/// two-acceptance observation, two decoded witness locators, reusable
/// accepted-control support links, and one independently proven multi-row
/// semantic witness. It is the sole accepted live safety report schema.
pub const LIVE_SAFETY_REPORT_SCHEMA: u32 = 6;

/// What a safety report is, said in the bytes.
///
/// One variant, and it is named rather than assumed: §13.6 forbids the
/// safety and minimality reports substituting for one another, and a
/// reader holding only the bytes needs to be able to tell which it has.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum LiveSafetyReportRole {
    /// The live-transfer safety report of §13.2.
    LiveTransferSafety,
}

impl LiveSafetyReportRole {
    /// The role's wire spelling.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::LiveTransferSafety => "live-transfer-safety",
        }
    }
}

/// Whether a report may call itself complete (§13.5).
///
/// Three states and no inference. A report never implies completeness by
/// the absence of failures, so the token is explicit and the middle state
/// exists to keep "nothing failed" and "everything ran" apart.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum LiveSafetyCompleteness {
    /// Every required row of §15 is answered at its own boundary.
    CompleteForTheRequiredMatrix,
    /// Some required row is unanswered, and the report names which.
    PartialRequiredRowsOutstanding,
    /// A required row was answered at a boundary other than its own.
    Failed,
}

impl LiveSafetyCompleteness {
    /// The token's wire spelling.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::CompleteForTheRequiredMatrix => "complete-for-the-required-matrix",
            Self::PartialRequiredRowsOutstanding => "partial-required-rows-outstanding",
            Self::Failed => "failed",
        }
    }
}

/// The exact deployment fields one validated target run binds.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiveDeploymentBinding {
    environment: String,
    network_id: [u8; 32],
    genesis_id: [u8; 32],
    target_contract: String,
}

impl LiveDeploymentBinding {
    /// The deployment environment.
    #[must_use]
    pub fn environment(&self) -> &str {
        &self.environment
    }

    /// The network identity in target display order.
    #[must_use]
    pub const fn network_id(&self) -> &[u8; 32] {
        &self.network_id
    }

    /// The genesis identity in target display order.
    #[must_use]
    pub const fn genesis_id(&self) -> &[u8; 32] {
        &self.genesis_id
    }

    /// The target contract revision recorded by the run.
    #[must_use]
    pub fn target_contract(&self) -> &str {
        &self.target_contract
    }
}

/// The stable executor self-description one validated target run binds.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiveExecutorProvenance {
    adapter_name: String,
    adapter_version: String,
    node_name: String,
    node_version: String,
    executed_source_tip: String,
}

impl LiveExecutorProvenance {
    /// The adapter name.
    #[must_use]
    pub fn adapter_name(&self) -> &str {
        &self.adapter_name
    }

    /// The adapter version.
    #[must_use]
    pub fn adapter_version(&self) -> &str {
        &self.adapter_version
    }

    /// The node name.
    #[must_use]
    pub fn node_name(&self) -> &str {
        &self.node_name
    }

    /// The node version.
    #[must_use]
    pub fn node_version(&self) -> &str {
        &self.node_version
    }

    /// The executed source identity.
    #[must_use]
    pub fn executed_source_tip(&self) -> &str {
        &self.executed_source_tip
    }
}

/// The native protocol revision an archived run actually spoke.
///
/// This is archival data. In particular, retaining revision 6 here does
/// not make revision 6 acceptable to the current live executor.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum NativeProtocolRevision {
    /// The historical rerun-day protocol.
    Revision6,
    /// The current forward-capture protocol.
    Revision7,
}

impl NativeProtocolRevision {
    /// The integer carried by the archived handshake.
    #[must_use]
    pub const fn code(self) -> u32 {
        match self {
            Self::Revision6 => 6,
            Self::Revision7 => 7,
        }
    }
}

/// Which fixture-digest algorithm produced one archived value.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum FixtureDigestAlgorithm {
    /// The immutable algorithm used by historical revision-6 runs.
    HistoricalV1,
    /// The sole algorithm used by forward revision-7 capture.
    ForwardV2,
}

impl FixtureDigestAlgorithm {
    /// The canonical report spelling.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::HistoricalV1 => "historical-v1",
            Self::ForwardV2 => "forward-v2",
        }
    }
}

/// One digest value and the algorithm it belongs to.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FixtureDigestFact {
    algorithm: FixtureDigestAlgorithm,
    value: [u8; 32],
}

impl FixtureDigestFact {
    /// The algorithm that produced the archived value.
    #[must_use]
    pub const fn algorithm(self) -> FixtureDigestAlgorithm {
        self.algorithm
    }

    /// The exact archived value.
    #[must_use]
    pub const fn value(self) -> [u8; 32] {
        self.value
    }
}

/// The matrix mutation one refused request stages.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum LiveMutantKind {
    /// The confidential asset-commitment mutant.
    ConfidentialAssetCommitment,
    /// The empty-signature mutant.
    EmptySignature,
    /// The hidden private-U output mutant.
    HiddenPrivateUOutput,
    /// The key-path escape mutant.
    KeyPathEscape,
    /// The malformed range-proof mutant.
    MalformedRangeproof,
    /// The malformed signature mutant.
    MalformedSignature,
    /// The missing sponsor-authorization mutant.
    MissingSponsorAuthorization,
    /// The no-coordinator leaf arrangement.
    NoCoordinator,
    /// The omitted-source mutant.
    OmittedSource,
    /// The one-above input/output tally mutant.
    OutputTotalOneAboveInput,
    /// The one-below input/output tally mutant.
    OutputTotalOneBelowInput,
    /// The private commitment-imbalance mutant.
    PrivateCtImbalance,
    /// The private-output omission mutant.
    PrivateOutputOmitted,
    /// The two-coordinator leaf arrangement.
    TwoCoordinators,
    /// The vault-control entitlement or bare-U program mutant.
    VaultControlEntitlementOrBareUOutput,
    /// The wrong explicit-asset mutant.
    WrongExplicitAsset,
    /// The wrong private blinding-balance mutant.
    WrongPrivateBlindingBalance,
}

impl LiveMutantKind {
    /// The matrix row this typed mutant stages.
    #[must_use]
    pub const fn row(self) -> &'static str {
        match self {
            Self::ConfidentialAssetCommitment => "confidential-asset-commitment",
            Self::EmptySignature => "empty-signature",
            Self::HiddenPrivateUOutput => "hidden-private-u-output",
            Self::KeyPathEscape => "key-path-escape",
            Self::MalformedRangeproof => "malformed-rangeproof",
            Self::MalformedSignature => "malformed-signature",
            Self::MissingSponsorAuthorization => "missing-sponsor-authorization",
            Self::NoCoordinator => "no-coordinator",
            Self::OmittedSource => "omitted-source",
            Self::OutputTotalOneAboveInput => "output-total-one-above-input",
            Self::OutputTotalOneBelowInput => "output-total-one-below-input",
            Self::PrivateCtImbalance => "private-ct-imbalance",
            Self::PrivateOutputOmitted => "private-output-omitted",
            Self::TwoCoordinators => "two-coordinators",
            Self::VaultControlEntitlementOrBareUOutput => {
                "vault-control-entitlement-or-bare-u-output"
            }
            Self::WrongExplicitAsset => "wrong-explicit-asset",
            Self::WrongPrivateBlindingBalance => "wrong-private-blinding-balance",
        }
    }
}

/// Where a typed mutation claims to differ from its accepted control.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LiveMutationLocator {
    /// One encoder-located output field.
    SerializedOutputField(SerializedFieldLocator),
    /// One item of one input witness stack.
    WitnessItem {
        /// The input position.
        input_index: usize,
        /// The witness-item position.
        item_index: usize,
    },
    /// The exact differing range in witnessless target bytes.
    WitnesslessRange {
        /// Inclusive start.
        start: usize,
        /// Exclusive end.
        end: usize,
    },
    /// A structural input/output-census mutation.
    TransactionShape {
        /// Control input count.
        control_inputs: usize,
        /// Mutant input count.
        mutant_inputs: usize,
        /// Control output count.
        control_outputs: usize,
        /// Mutant output count.
        mutant_outputs: usize,
    },
    /// One input whose witness changes between key-path and script-path shape.
    WitnessPathShape {
        /// The input position whose witness changes.
        input_index: usize,
        /// The control witness-stack item count.
        control_stack_items: usize,
        /// The mutant witness-stack item count.
        mutant_stack_items: usize,
        /// Every changed or inserted witness-item position.
        changed_positions: Vec<usize>,
        /// The control's decoded witness path.
        control_role: LiveWitnessPathRole,
        /// The mutant's decoded witness path.
        mutant_role: LiveWitnessPathRole,
        /// Whether the declaration requires equal witnessless bytes.
        witnessless_serialization_equal: bool,
    },
    /// A control and mutant's exact revealed committed-leaf arrangement.
    CommittedLeafArrangement {
        /// Input positions, in strictly increasing order.
        input_indices: Vec<usize>,
        /// Positions in `input_indices` revealing the control coordinator leaf.
        control_coordinator_leaf_indices: Vec<usize>,
        /// Positions in `input_indices` revealing the mutant coordinator leaf.
        mutant_coordinator_leaf_indices: Vec<usize>,
        /// The control's exact revealed committed leaf at every named input.
        control_committed_leaf_programs: Vec<Vec<u8>>,
        /// The mutant's exact revealed committed leaf at every named input.
        mutant_committed_leaf_programs: Vec<Vec<u8>>,
    },
}

/// The path shape recomputed from one decoded input witness.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum LiveWitnessPathRole {
    /// One signature-like item and no revealed leaf or control block.
    KeyPath,
    /// A revealed leaf program followed by a structurally valid control block.
    ScriptPath,
}

/// One exact accepted member of a composite two-acceptance observation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiveCompositeAcceptanceMember {
    ceremony: String,
    run_id: String,
    request_id: String,
    identity: Txid,
    consumed_commitment_output_index: usize,
}

impl LiveCompositeAcceptanceMember {
    /// Name one accepted request and the decoded output carrying its consumed parity.
    #[must_use]
    pub const fn new(
        ceremony: String,
        run_id: String,
        request_id: String,
        identity: Txid,
        consumed_commitment_output_index: usize,
    ) -> Self {
        Self {
            ceremony,
            run_id,
            request_id,
            identity,
            consumed_commitment_output_index,
        }
    }

    /// The distinct named ceremony this member came from.
    #[must_use]
    pub fn ceremony(&self) -> &str {
        &self.ceremony
    }

    /// The bound run identity.
    #[must_use]
    pub fn run_id(&self) -> &str {
        &self.run_id
    }

    /// The exact request identity within the run.
    #[must_use]
    pub fn request_id(&self) -> &str {
        &self.request_id
    }

    /// The accepted target identity.
    #[must_use]
    pub const fn identity(&self) -> Txid {
        self.identity
    }

    /// The decoded output position carrying this member's parity commitment.
    #[must_use]
    pub const fn consumed_commitment_output_index(&self) -> usize {
        self.consumed_commitment_output_index
    }
}

/// Exactly two primary accepted members answering one matrix row.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiveCompositeTwoAcceptance {
    members: [LiveCompositeAcceptanceMember; 2],
}

impl LiveCompositeTwoAcceptance {
    /// Bind exactly two accepted members; validation proves their identities and parity.
    #[must_use]
    pub const fn new(
        first: LiveCompositeAcceptanceMember,
        second: LiveCompositeAcceptanceMember,
    ) -> Self {
        Self {
            members: [first, second],
        }
    }

    /// The structurally exact two-member census.
    #[must_use]
    pub const fn members(&self) -> &[LiveCompositeAcceptanceMember; 2] {
        &self.members
    }
}

/// One accepted-control support link carried by a schema-6 refusal observation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiveSupportLink {
    run_id: String,
    request_id: String,
    request_bytes: Vec<u8>,
    response: LiveTargetResponse,
}

impl LiveSupportLink {
    /// Carry the exact request and response facts the link resolves to.
    #[must_use]
    pub const fn new(
        run_id: String,
        request_id: String,
        request_bytes: Vec<u8>,
        response: LiveTargetResponse,
    ) -> Self {
        Self {
            run_id,
            request_id,
            request_bytes,
            response,
        }
    }

    /// The run containing this support request.
    #[must_use]
    pub fn run_id(&self) -> &str {
        &self.run_id
    }

    /// The accepted-control request identity.
    #[must_use]
    pub fn request_id(&self) -> &str {
        &self.request_id
    }

    /// The exact accepted-control request bytes.
    #[must_use]
    pub fn request_bytes(&self) -> &[u8] {
        &self.request_bytes
    }

    /// The exact accepted-control response facts.
    #[must_use]
    pub const fn response(&self) -> &LiveTargetResponse {
        &self.response
    }
}

/// A closed semantic predicate one row can independently prove from request bytes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LiveRowSemanticPredicate {
    /// The request has one two-item sponsor authorization and one exact fee output.
    Sponsored {
        /// The sponsor input position.
        sponsor_input_index: usize,
        /// The explicit fee output position.
        fee_output_index: usize,
    },
    /// The sponsored request has no output paying the sponsor-change program.
    SponsorChangeAbsent {
        /// The sponsor input position.
        sponsor_input_index: usize,
        /// The explicit fee output position.
        fee_output_index: usize,
        /// The deployment's exact sponsor-change program.
        sponsor_change_program: Vec<u8>,
    },
}

/// One primary accepted request that may answer only its declared semantic row set.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiveMultiRowSemanticWitness {
    run_id: String,
    request_id: String,
    identity: Txid,
    rows: BTreeSet<&'static str>,
    predicates: BTreeMap<&'static str, LiveRowSemanticPredicate>,
}

impl LiveMultiRowSemanticWitness {
    /// Bind one acceptance to a named row set and exactly one predicate per row.
    #[must_use]
    pub const fn new(
        run_id: String,
        request_id: String,
        identity: Txid,
        rows: BTreeSet<&'static str>,
        predicates: BTreeMap<&'static str, LiveRowSemanticPredicate>,
    ) -> Self {
        Self {
            run_id,
            request_id,
            identity,
            rows,
            predicates,
        }
    }

    /// The rows this one request claims to answer.
    #[must_use]
    pub const fn rows(&self) -> &BTreeSet<&'static str> {
        &self.rows
    }

    /// The run carrying the one primary request.
    #[must_use]
    pub fn run_id(&self) -> &str {
        &self.run_id
    }

    /// The one primary request identity.
    #[must_use]
    pub fn request_id(&self) -> &str {
        &self.request_id
    }

    /// The accepted target identity recomputed from the request.
    #[must_use]
    pub const fn identity(&self) -> Txid {
        self.identity
    }

    /// Exactly one closed semantic predicate for every declared row.
    #[must_use]
    pub const fn predicates(&self) -> &BTreeMap<&'static str, LiveRowSemanticPredicate> {
        &self.predicates
    }
}

/// Which half of a paired equality request one archive member is.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum LivePairMember {
    /// The explicit member.
    Explicit,
    /// The private committed member.
    Private,
}

impl LivePairMember {
    /// The canonical report spelling.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Explicit => "explicit",
            Self::Private => "private",
        }
    }
}

/// Public constructor inputs needed to recompute one paired projection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LivePairProjectionInput {
    input_owners: Vec<String>,
    semantic_input_amounts: Vec<u64>,
    destination_programs: BTreeMap<String, Vec<u8>>,
    semantic_destination_amounts: BTreeMap<String, u64>,
}

impl LivePairProjectionInput {
    /// The public constructor facts needed to project one accepted member.
    #[must_use]
    pub const fn new(
        input_owners: Vec<String>,
        semantic_input_amounts: Vec<u64>,
        destination_programs: BTreeMap<String, Vec<u8>>,
        semantic_destination_amounts: BTreeMap<String, u64>,
    ) -> Self {
        Self {
            input_owners,
            semantic_input_amounts,
            destination_programs,
            semantic_destination_amounts,
        }
    }
}

/// What one exact request does inside its ceremony.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LiveRequestFact {
    /// A row-bearing accepted request.
    Acceptance,
    /// An accepted control used to attribute one or more refusals.
    Control,
    /// A refused typed mutant and its same-ceremony control.
    Refusal {
        /// The staged matrix mutation.
        mutant: LiveMutantKind,
        /// The accepted control request in this run.
        control_request_id: String,
        /// The mutation's decoded-byte locator or shape.
        locator: LiveMutationLocator,
    },
    /// One accepted member of a paired projection comparison.
    Paired {
        /// Which representation this request carries.
        member: LivePairMember,
        /// Public inputs needed for the independent projection.
        projection: LivePairProjectionInput,
    },
}

/// One exact target response retained by a validated run binding.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum LiveTargetResponse {
    /// The target accepted the request and assigned this identity.
    Accepted {
        /// The typed target identity.
        identity: Txid,
    },
    /// The target refused the request.
    Refused {
        /// The observed refusal layer.
        observed_layer: ObservedOutcomeLayer,
        /// The target's exact detail.
        detail: String,
        /// The accepted control identity the refusal is attributable against.
        control_identity: Txid,
    },
}

/// One transcript-grade target run in the cumulative corpus.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiveRunBinding {
    run_id: String,
    archive_bytes: Vec<u8>,
    revision: NativeProtocolRevision,
    digest_facts: BTreeMap<String, FixtureDigestFact>,
    deployment: LiveDeploymentBinding,
    executor: LiveExecutorProvenance,
    requests: BTreeMap<String, Vec<u8>>,
    request_facts: BTreeMap<String, LiveRequestFact>,
    responses: BTreeMap<String, LiveTargetResponse>,
}

impl LiveRunBinding {
    /// Parse one archive-owned run and mint its content address.
    ///
    /// This constructor establishes only that the archive fact header is
    /// complete and typed. Request decoding, response linkage, txid
    /// recomputation, mutation attribution and pair projection remain the
    /// validator's work; accepting them here would make construction and
    /// validation one opinion again.
    ///
    /// # Errors
    ///
    /// [`LiveSafetyReportRefusal::MalformedRunArchive`],
    /// [`LiveSafetyReportRefusal::UnknownProtocolRevision`] or
    /// [`LiveSafetyReportRefusal::UnknownDigestAlgorithm`] when the archive
    /// fact bytes are not one complete supported record.
    pub fn from_archive(
        archive_bytes: Vec<u8>,
        requests: BTreeMap<String, Vec<u8>>,
        request_facts: BTreeMap<String, LiveRequestFact>,
        responses: BTreeMap<String, LiveTargetResponse>,
    ) -> Result<Self, LiveSafetyReportRefusal> {
        let parsed = parse_run_archive(&archive_bytes)?;
        let mut run = Self {
            run_id: String::new(),
            archive_bytes,
            revision: parsed.revision,
            digest_facts: parsed.digest_facts,
            deployment: parsed.deployment,
            executor: parsed.executor,
            requests,
            request_facts,
            responses,
        };
        run.run_id = content_address_run(&run);
        Ok(run)
    }

    /// The deterministic corpus identity of this run.
    #[must_use]
    pub fn run_id(&self) -> &str {
        &self.run_id
    }

    /// The exact archived bytes the typed provenance was parsed from.
    #[must_use]
    pub fn archive_bytes(&self) -> &[u8] {
        &self.archive_bytes
    }

    /// The archived protocol revision.
    #[must_use]
    pub const fn revision(&self) -> NativeProtocolRevision {
        self.revision
    }

    /// Every algorithm-tagged fixture digest carried by the run.
    #[must_use]
    pub const fn digest_facts(&self) -> &BTreeMap<String, FixtureDigestFact> {
        &self.digest_facts
    }

    /// The exact deployment binding.
    #[must_use]
    pub const fn deployment(&self) -> &LiveDeploymentBinding {
        &self.deployment
    }

    /// The validated executor provenance.
    #[must_use]
    pub const fn executor(&self) -> &LiveExecutorProvenance {
        &self.executor
    }

    /// Exact sent request bytes, keyed by deterministic request identity.
    #[must_use]
    pub const fn requests(&self) -> &BTreeMap<String, Vec<u8>> {
        &self.requests
    }

    /// The typed role and validation inputs for every exact request.
    #[must_use]
    pub const fn request_facts(&self) -> &BTreeMap<String, LiveRequestFact> {
        &self.request_facts
    }

    /// Exact responses, keyed by the same request identity.
    #[must_use]
    pub const fn responses(&self) -> &BTreeMap<String, LiveTargetResponse> {
        &self.responses
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ParsedRunArchive {
    ceremony_id: Option<String>,
    revision: NativeProtocolRevision,
    digest_facts: BTreeMap<String, FixtureDigestFact>,
    deployment: LiveDeploymentBinding,
    executor: LiveExecutorProvenance,
}

#[derive(Debug, Default)]
struct RunArchiveFields {
    ceremony_id: Option<String>,
    revision: Option<NativeProtocolRevision>,
    environment: Option<String>,
    network_id: Option<[u8; 32]>,
    genesis_id: Option<[u8; 32]>,
    target_contract: Option<String>,
    adapter_name: Option<String>,
    adapter_version: Option<String>,
    node_name: Option<String>,
    node_version: Option<String>,
    executed_source_tip: Option<String>,
    digest_count: Option<usize>,
    digest_facts: BTreeMap<String, FixtureDigestFact>,
}

fn parse_hex_32(text: &str) -> Option<[u8; 32]> {
    if text.len() != 64 {
        return None;
    }
    let mut decoded = [0_u8; 32];
    let (pairs, remainder) = text.as_bytes().as_chunks::<2>();
    if !remainder.is_empty() {
        return None;
    }
    for (slot, pair) in decoded.iter_mut().zip(pairs) {
        let high = match pair[0] {
            b'0'..=b'9' => pair[0] - b'0',
            b'a'..=b'f' => pair[0] - b'a' + 10,
            b'A'..=b'F' => pair[0] - b'A' + 10,
            _ => return None,
        };
        let low = match pair[1] {
            b'0'..=b'9' => pair[1] - b'0',
            b'a'..=b'f' => pair[1] - b'a' + 10,
            b'A'..=b'F' => pair[1] - b'A' + 10,
            _ => return None,
        };
        *slot = (high << 4) | low;
    }
    Some(decoded)
}

fn set_once<T>(slot: &mut Option<T>, value: T) -> Result<(), LiveSafetyReportRefusal> {
    if slot.replace(value).is_some() {
        return Err(LiveSafetyReportRefusal::MalformedRunArchive);
    }
    Ok(())
}

impl RunArchiveFields {
    fn parse_ceremony(&mut self, fields: &[&str]) -> Result<(), LiveSafetyReportRefusal> {
        let ["ceremony_id", value] = fields else {
            return Err(LiveSafetyReportRefusal::MalformedRunArchive);
        };
        if value.is_empty() {
            return Err(LiveSafetyReportRefusal::MalformedRunArchive);
        }
        set_once(&mut self.ceremony_id, (*value).to_owned())
    }

    fn parse_revision(&mut self, fields: &[&str]) -> Result<(), LiveSafetyReportRefusal> {
        let ["protocol_revision", value] = fields else {
            return Err(LiveSafetyReportRefusal::MalformedRunArchive);
        };
        let offered = value
            .parse::<u32>()
            .map_err(|_| LiveSafetyReportRefusal::MalformedRunArchive)?;
        let parsed = match offered {
            6 => NativeProtocolRevision::Revision6,
            7 => NativeProtocolRevision::Revision7,
            unknown => {
                return Err(LiveSafetyReportRefusal::UnknownProtocolRevision(unknown));
            }
        };
        set_once(&mut self.revision, parsed)
    }

    fn parse_deployment(&mut self, fields: &[&str]) -> Result<(), LiveSafetyReportRefusal> {
        match fields {
            ["deployment_environment", value] => {
                set_once(&mut self.environment, (*value).to_owned())
            }
            ["deployment_network", value] => set_once(
                &mut self.network_id,
                parse_hex_32(value).ok_or(LiveSafetyReportRefusal::MalformedRunArchive)?,
            ),
            ["deployment_genesis", value] => set_once(
                &mut self.genesis_id,
                parse_hex_32(value).ok_or(LiveSafetyReportRefusal::MalformedRunArchive)?,
            ),
            ["deployment_target", value] => {
                set_once(&mut self.target_contract, (*value).to_owned())
            }
            _ => Err(LiveSafetyReportRefusal::MalformedRunArchive),
        }
    }

    fn parse_executor(&mut self, fields: &[&str]) -> Result<(), LiveSafetyReportRefusal> {
        match fields {
            ["executor_adapter", value] => set_once(&mut self.adapter_name, (*value).to_owned()),
            ["executor_adapter_version", value] => {
                set_once(&mut self.adapter_version, (*value).to_owned())
            }
            ["executor_node", value] => set_once(&mut self.node_name, (*value).to_owned()),
            ["executor_node_version", value] => {
                set_once(&mut self.node_version, (*value).to_owned())
            }
            ["executor_source_tip", value] => {
                set_once(&mut self.executed_source_tip, (*value).to_owned())
            }
            _ => Err(LiveSafetyReportRefusal::MalformedRunArchive),
        }
    }

    fn parse_digest(&mut self, fields: &[&str]) -> Result<(), LiveSafetyReportRefusal> {
        match fields {
            ["digest_count", value] => {
                let parsed = value
                    .parse::<usize>()
                    .map_err(|_| LiveSafetyReportRefusal::MalformedRunArchive)?;
                set_once(&mut self.digest_count, parsed)
            }
            ["digest", name, algorithm, value] => {
                let algorithm = match *algorithm {
                    "historical-v1" => FixtureDigestAlgorithm::HistoricalV1,
                    "forward-v2" => FixtureDigestAlgorithm::ForwardV2,
                    unknown => {
                        return Err(LiveSafetyReportRefusal::UnknownDigestAlgorithm(
                            unknown.to_owned(),
                        ));
                    }
                };
                let value =
                    parse_hex_32(value).ok_or(LiveSafetyReportRefusal::MalformedRunArchive)?;
                if self
                    .digest_facts
                    .insert((*name).to_owned(), FixtureDigestFact { algorithm, value })
                    .is_some()
                {
                    return Err(LiveSafetyReportRefusal::MalformedRunArchive);
                }
                Ok(())
            }
            _ => Err(LiveSafetyReportRefusal::MalformedRunArchive),
        }
    }

    fn parse_line(&mut self, fields: &[&str]) -> Result<(), LiveSafetyReportRefusal> {
        match fields.first().copied() {
            Some("ceremony_id") => self.parse_ceremony(fields),
            Some("protocol_revision") => self.parse_revision(fields),
            Some(
                "deployment_environment"
                | "deployment_network"
                | "deployment_genesis"
                | "deployment_target",
            ) => self.parse_deployment(fields),
            Some(
                "executor_adapter"
                | "executor_adapter_version"
                | "executor_node"
                | "executor_node_version"
                | "executor_source_tip",
            ) => self.parse_executor(fields),
            Some("digest_count" | "digest") => self.parse_digest(fields),
            _ => Err(LiveSafetyReportRefusal::MalformedRunArchive),
        }
    }

    fn finish(self) -> Result<ParsedRunArchive, LiveSafetyReportRefusal> {
        if self.digest_count != Some(self.digest_facts.len()) {
            return Err(LiveSafetyReportRefusal::MalformedRunArchive);
        }
        Ok(ParsedRunArchive {
            ceremony_id: self.ceremony_id,
            revision: self
                .revision
                .ok_or(LiveSafetyReportRefusal::MalformedRunArchive)?,
            digest_facts: self.digest_facts,
            deployment: LiveDeploymentBinding {
                environment: self
                    .environment
                    .ok_or(LiveSafetyReportRefusal::MalformedRunArchive)?,
                network_id: self
                    .network_id
                    .ok_or(LiveSafetyReportRefusal::MalformedRunArchive)?,
                genesis_id: self
                    .genesis_id
                    .ok_or(LiveSafetyReportRefusal::MalformedRunArchive)?,
                target_contract: self
                    .target_contract
                    .ok_or(LiveSafetyReportRefusal::MalformedRunArchive)?,
            },
            executor: LiveExecutorProvenance {
                adapter_name: self
                    .adapter_name
                    .ok_or(LiveSafetyReportRefusal::MalformedRunArchive)?,
                adapter_version: self
                    .adapter_version
                    .ok_or(LiveSafetyReportRefusal::MalformedRunArchive)?,
                node_name: self
                    .node_name
                    .ok_or(LiveSafetyReportRefusal::MalformedRunArchive)?,
                node_version: self
                    .node_version
                    .ok_or(LiveSafetyReportRefusal::MalformedRunArchive)?,
                executed_source_tip: self
                    .executed_source_tip
                    .ok_or(LiveSafetyReportRefusal::MalformedRunArchive)?,
            },
        })
    }
}

fn parse_run_archive(bytes: &[u8]) -> Result<ParsedRunArchive, LiveSafetyReportRefusal> {
    if bytes.contains(&b'\r') {
        return Err(LiveSafetyReportRefusal::MalformedRunArchive);
    }
    let text =
        std::str::from_utf8(bytes).map_err(|_| LiveSafetyReportRefusal::MalformedRunArchive)?;
    if text.is_empty() || !text.ends_with('\n') {
        return Err(LiveSafetyReportRefusal::MalformedRunArchive);
    }

    let mut parsed = RunArchiveFields::default();
    for line in text.lines() {
        let fields = line.split_ascii_whitespace().collect::<Vec<_>>();
        if fields.join(" ") != line {
            return Err(LiveSafetyReportRefusal::MalformedRunArchive);
        }
        parsed.parse_line(&fields)?;
    }
    parsed.finish()
}

fn append_framed(preimage: &mut Vec<u8>, bytes: &[u8]) {
    let length = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
    preimage.extend_from_slice(&length.to_le_bytes());
    preimage.extend_from_slice(bytes);
}

fn content_address_run(run: &LiveRunBinding) -> String {
    let mut preimage = b"tripod/live-report/run/v1".to_vec();
    append_framed(&mut preimage, &run.archive_bytes);
    for (request_id, bytes) in &run.requests {
        append_framed(&mut preimage, b"request");
        append_framed(&mut preimage, request_id.as_bytes());
        append_framed(&mut preimage, bytes);
    }
    for (request_id, fact) in &run.request_facts {
        append_framed(&mut preimage, b"request-fact");
        append_framed(&mut preimage, request_id.as_bytes());
        append_framed(&mut preimage, format!("{fact:?}").as_bytes());
    }
    for (request_id, response) in &run.responses {
        append_framed(&mut preimage, b"response");
        append_framed(&mut preimage, request_id.as_bytes());
        append_framed(&mut preimage, format!("{response:?}").as_bytes());
    }
    hex_bytes(&tagged::sha256(&preimage))
}

fn recomputed_txid(bytes: &[u8]) -> Option<Txid> {
    let decoded = TargetTransaction::decode(bytes).ok()?;
    if decoded.encode() != bytes {
        return None;
    }
    let first = tagged::sha256(&decoded.encode_without_witness());
    Some(Txid::from_internal(tagged::sha256(&first)))
}

/// A typed historical observation whose run binding is absent.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum LiveRecordedObservation {
    /// A recorded acceptance.
    NativeAcceptance {
        /// The target-computed identity.
        identity: Txid,
    },
    /// A recorded refusal and accepted control.
    NativeRefusal {
        /// The row's declared boundary.
        declared_boundary: EvidenceBoundary,
        /// The recorded observed layer.
        observed_layer: ObservedOutcomeLayer,
        /// The accepted control identity.
        control_identity: Txid,
        /// The target's exact refusal detail.
        detail: &'static str,
    },
    /// A recorded relation over two accepted identities.
    PairedRelation {
        /// The explicit identity.
        explicit_identity: Txid,
        /// The private identity.
        private_identity: Txid,
        /// The recorded relation.
        relation: &'static str,
    },
}

/// One row-level observation in the corpus ledger.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum LiveReportObservation {
    /// A bound target acceptance.
    NativeAcceptance {
        /// The matrix row.
        row: &'static str,
        /// The bound run.
        run_id: String,
        /// The exact request within the run.
        request_id: String,
        /// The target-computed identity.
        identity: Txid,
    },
    /// Two accepted requests from distinct ceremonies proving both parity forms.
    CompositeTwoAcceptance {
        /// The one matrix row the fixed pair answers.
        row: &'static str,
        /// Exactly two primary acceptance links.
        acceptance: LiveCompositeTwoAcceptance,
    },
    /// A bound refusal with an explicitly reusable support link.
    NativeRefusalWithSupport {
        /// The matrix row.
        row: &'static str,
        /// The mutant's bound run.
        run_id: String,
        /// The mutant request within the run.
        request_id: String,
        /// The row's declared boundary.
        declared_boundary: EvidenceBoundary,
        /// The observed layer.
        observed_layer: ObservedOutcomeLayer,
        /// The target's exact refusal detail.
        detail: String,
        /// The accepted control supporting this refusal.
        support: LiveSupportLink,
    },
    /// A bound relation over two accepted members.
    PairedRelation {
        /// The matrix row.
        row: &'static str,
        /// The explicit member's run.
        explicit_run_id: String,
        /// The explicit request within its run.
        explicit_request_id: String,
        /// The explicit identity.
        explicit_identity: Txid,
        /// The private member's run.
        private_run_id: String,
        /// The private request within its run.
        private_request_id: String,
        /// The private identity.
        private_identity: Txid,
        /// The validated relation.
        relation: String,
    },
    /// One accepted request proving more than one named row independently.
    MultiRowSemantic {
        /// The typed row set, predicates, primary link, and response identity.
        witness: LiveMultiRowSemanticWitness,
    },
    /// A preserved historical observation lacking a complete run binding.
    RecordedObservationUnbound {
        /// The matrix row.
        row: &'static str,
        /// The typed recorded fact.
        observation: LiveRecordedObservation,
    },
    /// A first-party determinism observation.
    Determinism {
        /// The matrix row.
        row: &'static str,
        /// What was recomputed.
        recomputed: &'static str,
        /// The site that recomputed it.
        observed_by: &'static str,
    },
    /// A first-party fact.
    FirstPartyFact {
        /// The matrix row.
        row: &'static str,
        /// The fact.
        fact: &'static str,
        /// The site establishing it.
        observed_by: &'static str,
    },
}

/// One report-layer property the evidence plan requires validation to check.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LiveReportLayerRequirement {
    row: &'static str,
    requirement: LiveReportRequirement,
}

impl LiveReportLayerRequirement {
    /// The matrix row requiring the check.
    #[must_use]
    pub const fn row(&self) -> &'static str {
        self.row
    }

    /// The exact property validation must establish.
    #[must_use]
    pub const fn requirement(&self) -> LiveReportRequirement {
        self.requirement
    }
}

/// One report-layer property established against canonical versioned bytes.
///
/// There is deliberately no public constructor. Values live only inside a
/// [`ValidatedLiveTransferSafetyReport`] returned after disclosure validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ValidatedReportLayerObservation {
    row: &'static str,
    requirement: LiveReportRequirement,
    schema: u32,
}

impl ValidatedReportLayerObservation {
    /// The matrix row this observation answers.
    #[must_use]
    pub const fn row(&self) -> &'static str {
        self.row
    }

    /// The exact property observed in the canonical bytes.
    #[must_use]
    pub const fn requirement(&self) -> LiveReportRequirement {
        self.requirement
    }

    /// The canonical schema whose bytes were checked.
    #[must_use]
    pub const fn schema(&self) -> u32 {
        self.schema
    }
}

/// The lifecycle obligations §13.2 makes the report carry (§17.1).
///
/// Transfer is implemented; burn and redemption are not; the candidate is
/// therefore lifecycle-incomplete. Recorded in the report because §7.4
/// requires the status to travel with the candidate rather than being
/// looked up.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiveLifecycleStatus {
    implemented: BTreeSet<&'static str>,
    outstanding: BTreeSet<&'static str>,
    release_complete: bool,
}

impl LiveLifecycleStatus {
    /// The Phase-5 status, which is the only one this candidate has.
    #[must_use]
    pub fn candidate() -> Self {
        Self {
            implemented: BTreeSet::from(["transfer-live-receipts"]),
            outstanding: BTreeSet::from(["burn", "redeem"]),
            release_complete: false,
        }
    }

    /// The exits this candidate implements.
    #[must_use]
    pub const fn implemented(&self) -> &BTreeSet<&'static str> {
        &self.implemented
    }

    /// The exits it does not.
    #[must_use]
    pub const fn outstanding(&self) -> &BTreeSet<&'static str> {
        &self.outstanding
    }

    /// Whether the live-receipt lifecycle is complete, which it is not.
    #[must_use]
    pub const fn release_complete(&self) -> bool {
        self.release_complete
    }
}

/// One volatile field §13.5 keeps out of the canonical bytes.
///
/// Enumerated so the exclusion is a census a test can walk rather than a
/// list in a comment. Every one of them lives on
/// [`LiveSafetyDiagnostics`], which the canonical renderer cannot reach.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum VolatileField {
    /// The wall-clock time the run started.
    WallClockTime,
    /// How long the run took.
    ElapsedTime,
    /// The host the run happened on.
    Hostname,
    /// The account it ran as.
    Username,
    /// The process identifier.
    ProcessId,
    /// A temporary directory path.
    TemporaryPath,
    /// The executor program's path.
    ExecutorPath,
    /// Raw text a child process wrote.
    RawChildStandardError,
    /// An environment variable's value.
    EnvironmentValue,
    /// Test-only private fixture material the schema does not require.
    TestOnlyFixtureMaterial,
}

impl VolatileField {
    /// All ten, in §13.5's order.
    pub const ALL: &'static [Self] = &[
        Self::WallClockTime,
        Self::ElapsedTime,
        Self::Hostname,
        Self::Username,
        Self::ProcessId,
        Self::TemporaryPath,
        Self::ExecutorPath,
        Self::RawChildStandardError,
        Self::EnvironmentValue,
        Self::TestOnlyFixtureMaterial,
    ];
}

/// The noncanonical diagnostic report of §13.6.
///
/// Everything §13.5 excludes from the canonical bytes, in one value that
/// the canonical renderer takes no argument of. Duration is a property of
/// the machine a run happened on rather than of the run's result, and a
/// canonical stream carrying one differs from every other by
/// construction — so two runs of the same matrix could never be compared
/// byte for byte.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LiveSafetyDiagnostics {
    fields: BTreeMap<VolatileField, String>,
}

impl LiveSafetyDiagnostics {
    /// A diagnostic report with one field recorded.
    #[must_use]
    pub fn with(mut self, field: VolatileField, value: impl Into<String>) -> Self {
        self.fields.insert(field, value.into());
        self
    }

    /// One recorded field.
    #[must_use]
    pub fn field(&self, field: VolatileField) -> Option<&str> {
        self.fields.get(&field).map(String::as_str)
    }

    /// Every recorded field.
    #[must_use]
    pub const fn fields(&self) -> &BTreeMap<VolatileField, String> {
        &self.fields
    }

    /// The diagnostic report's own rendering.
    ///
    /// Deliberately a separate function from the canonical one, and
    /// deliberately not called by it.
    ///
    /// # Panics
    ///
    /// Never: the sink is a `String`, whose writes cannot fail.
    #[must_use]
    pub fn render(&self) -> String {
        let mut text = String::new();
        let _ = writeln!(text, "role live-transfer-safety-diagnostics");
        let _ = writeln!(text, "canonical false");
        for (field, value) in &self.fields {
            let _ = writeln!(text, "{field:?} {value}");
        }
        text
    }
}

/// The live-transfer safety report of §13.2.
///
/// A value a caller can build, and one that establishes nothing on its
/// own. The cumulative corpus carriers are here; the conclusions are
/// [`validate_live_safety_report`]'s.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiveTransferSafetyReport {
    schema: u32,
    role: LiveSafetyReportRole,
    target: TargetProjection,
    representations: BTreeSet<LiveTransferRepresentationPlan>,
    runs: Vec<LiveRunBinding>,
    observations: Vec<LiveReportObservation>,
    report_layer_requirements: Vec<LiveReportLayerRequirement>,
    lifecycle: LiveLifecycleStatus,
    census: LiveEvidenceCensus,
    completeness: LiveSafetyCompleteness,
}

impl LiveTransferSafetyReport {
    /// The report's schema.
    #[must_use]
    pub const fn schema(&self) -> u32 {
        self.schema
    }

    /// What kind of report this is.
    #[must_use]
    pub const fn role(&self) -> LiveSafetyReportRole {
        self.role
    }

    /// The reviewed target the matrix is stated against.
    #[must_use]
    pub const fn target(&self) -> &TargetProjection {
        &self.target
    }

    /// The transcript-grade run bindings in the cumulative corpus.
    #[must_use]
    pub fn runs(&self) -> &[LiveRunBinding] {
        &self.runs
    }

    /// The representation plans in scope.
    #[must_use]
    pub const fn representations(&self) -> &BTreeSet<LiveTransferRepresentationPlan> {
        &self.representations
    }

    /// Every row-level observation in matrix order.
    #[must_use]
    pub fn observations(&self) -> &[LiveReportObservation] {
        &self.observations
    }

    /// Every report-layer requirement in matrix order.
    #[must_use]
    pub fn report_layer_requirements(&self) -> &[LiveReportLayerRequirement] {
        &self.report_layer_requirements
    }

    /// The lifecycle obligations.
    #[must_use]
    pub const fn lifecycle(&self) -> &LiveLifecycleStatus {
        &self.lifecycle
    }

    /// The row census.
    #[must_use]
    pub const fn census(&self) -> LiveEvidenceCensus {
        self.census
    }

    /// The completeness token.
    #[must_use]
    pub const fn completeness(&self) -> LiveSafetyCompleteness {
        self.completeness
    }
}

/// Why one report could not be validated.
///
/// Every variant is a disagreement between the report and what this
/// module recomputed from the evidence plan. None of them is a target
/// verdict: a report whose every row is blocked validates perfectly and
/// says so.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum LiveSafetyReportRefusal {
    /// The report states a schema this validator does not read.
    UnsupportedSchema(u32),
    /// The report states a role that is not the safety one.
    ///
    /// §13.6's separation, enforced: a minimality report handed to the
    /// safety gate is refused rather than read as a safety one.
    WrongRole(LiveSafetyReportRole),
    /// The report's target is not the one the plan is stated against.
    TargetDiffers,
    /// The report's representation census is not the plan's.
    RepresentationCensusDiffers,
    /// The report's row census is not the one recomputed from the plan.
    ///
    /// Both censuses are boxed, as [`crate::live_resource_report`] boxes
    /// its own pair and for the same reason: a refusal carrying two wide
    /// aggregates inline would make every `Result` in this module pay for
    /// the one arm the happy path never takes. The first member is what
    /// the report said and the second is what the plan recomputes to.
    CensusDiffers(Box<(LiveEvidenceCensus, LiveEvidenceCensus)>),
    /// The report claims a completeness its own census does not support.
    CompletenessDiffers {
        /// What the report said.
        reported: LiveSafetyCompleteness,
        /// What the census supports.
        recomputed: LiveSafetyCompleteness,
    },
    /// A recorded target identity is not valid target-display txid text.
    MalformedRecordedIdentity {
        /// The matrix row carrying it.
        row: &'static str,
    },
    /// The archive fact header is missing, duplicated or malformed.
    MalformedRunArchive,
    /// The archive names a protocol revision outside the closed set.
    UnknownProtocolRevision(u32),
    /// The archive names a fixture-digest algorithm outside the closed set.
    UnknownDigestAlgorithm(String),
    /// The report and the derived corpus carry different run identities.
    RunCensusDiffers,
    /// One run's typed fields were not parsed from these archive bytes.
    RunArchiveDiffers(String),
    /// One run's typed protocol revision differs from its archive.
    RunProtocolRevisionDiffers(String),
    /// One run's digest values or algorithm tags differ from its archive.
    RunDigestFactsDiffer(String),
    /// One run's stated content address does not address its carriers.
    RunIdDiffers(String),
    /// One run's deployment differs from the validated ledger.
    RunDeploymentDiffers(String),
    /// One run's exact request census or bytes differ.
    RunRequestDiffers(String),
    /// One run's exact response census or facts differ.
    RunResponseDiffers(String),
    /// Requests, typed request facts and responses do not have identical keys.
    RequestResponseCensusDiffers(String),
    /// A request identity is reused by more than one run.
    ReusedRequestId(String),
    /// One exact request does not decode and round-trip byte for byte.
    RequestDecodeRefused(String, String),
    /// An accepted identity differs from the recomputed request txid.
    AcceptedIdentityDiffers(String, String),
    /// A response variant does not fit its request's typed role.
    RequestResponseShapeDiffers(String, String),
    /// A request is cited in a typed role it does not carry.
    RequestRoleDiffers(String, String),
    /// A refusal's accepted control identity does not recompute.
    RefusalControlIdentityDiffers(String, String),
    /// A request fact links to a request belonging to another run.
    CrossRunRequestLink(String, String),
    /// A mutation locator or shape does not describe the decoded byte change.
    MutationLocatorDiffers(String, String),
    /// One run's executor provenance differs.
    RunExecutorProvenanceDiffers(String),
    /// The report's row-level observations or report requirements differ
    /// from the derived ledger.
    ObservationsDiffer,
    /// The report's observation buckets do not cross-foot its census.
    ObservationCensusDiffers,
    /// A bound observation cites no matching run or request.
    BoundObservationUnbacked {
        /// The matrix row carrying the invalid reference.
        row: &'static str,
    },
    /// One request is aliased outside the closed schema-6 reuse shapes.
    ObservationAliased(String),
    /// Two composite acceptance members claim the same named ceremony.
    CompositeCeremonyReused(String),
    /// The two decoded commitment prefixes do not carry opposite parity.
    CompositeParityNotOpposite(&'static str),
    /// A support-link copy differs from the request or response facts it resolves to.
    SupportLinkFactsDiffer(String, String),
    /// A multi-row witness's row set and predicate keys do not agree.
    MultiRowWitnessRowsDiffer,
    /// One named row's semantic predicate did not recompute from request bytes.
    SemanticPredicateNotProven(&'static str),
    /// Run, response, control and observation counts do not cross-foot.
    RunObservationCensusDiffers,
    /// A pair's decoded projections do not prove its stated equality.
    PairedRelationNotRecomputed(&'static str),
    /// A forbidden disclosure key is present in the canonical bytes.
    ReportDisclosurePresent {
        /// The matrix row whose requirement the key violates.
        row: &'static str,
        /// The forbidden canonical field key.
        key: String,
    },
    /// The report's lifecycle status is not the candidate's.
    LifecycleDiffers,
}

/// One run's report, validated: the only thing a safety gate reads.
///
/// # What holding one of these establishes
///
/// That the corpus carriers agree with what this module recomputed from
/// the evidence plan. Its recomputation witness names only comparisons
/// that actually ran; with no transcript-grade target run in the tree it
/// establishes the preserved observations as unbound, not as target
/// verdict evidence. Its report-layer observations and canonical bytes are
/// private fields minted together only after disclosure validation; there is
/// no constructor or intermediate wrapper state that can omit
/// [`RecomputedItem::DisclosureComparison`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedLiveTransferSafetyReport {
    report: LiveTransferSafetyReport,
    report_layer_observations: Vec<ValidatedReportLayerObservation>,
    census: LiveEvidenceCensus,
    recomputed_items: BTreeSet<RecomputedItem>,
    blockers: BTreeMap<LiveInfrastructureBlocker, usize>,
    outstanding: Vec<(&'static str, LiveRowStanding)>,
    scoreboard: BTreeMap<LiveSafetySection, (usize, usize, usize)>,
    canonical_bytes: String,
}

/// One of §13.5's fourteen recomputed items.
///
/// A census rather than prose, so a validation that stopped performing
/// one has to remove it here and fail the count.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RecomputedItem {
    /// The report's role.
    Role,
    /// The report's schema.
    Schema,
    /// The target and deployment binding.
    TargetDeploymentBinding,
    /// The exact request and response census.
    RequestResponseCensus,
    /// The response shape.
    ResponseShape,
    /// The case census.
    CaseCensus,
    /// The relation census.
    RelationCensus,
    /// The mutation links.
    MutationLinks,
    /// The target verdict comparison.
    TargetVerdictComparison,
    /// The semantic projection.
    SemanticProjection,
    /// The disclosure comparison.
    DisclosureComparison,
    /// The resource comparison.
    ResourceComparison,
    /// The executor provenance.
    ExecutorProvenance,
    /// The summary.
    Summary,
}

impl RecomputedItem {
    /// All fourteen, in §13.5's order.
    pub const ALL: &'static [Self] = &[
        Self::Role,
        Self::Schema,
        Self::TargetDeploymentBinding,
        Self::RequestResponseCensus,
        Self::ResponseShape,
        Self::CaseCensus,
        Self::RelationCensus,
        Self::MutationLinks,
        Self::TargetVerdictComparison,
        Self::SemanticProjection,
        Self::DisclosureComparison,
        Self::ResourceComparison,
        Self::ExecutorProvenance,
        Self::Summary,
    ];

    /// Whether this item can only be recomputed for a report about a run.
    ///
    /// Six of the fourteen are about things an executor produced. They do
    /// not enter the validated witness until at least one bound run made
    /// the corresponding comparison possible.
    #[must_use]
    pub const fn needs_a_run(self) -> bool {
        matches!(
            self,
            Self::RequestResponseCensus
                | Self::ResponseShape
                | Self::TargetVerdictComparison
                | Self::SemanticProjection
                | Self::ResourceComparison
                | Self::ExecutorProvenance
        )
    }
}

impl ValidatedLiveTransferSafetyReport {
    /// The report every carrier of which has been recomputed.
    #[must_use]
    pub const fn report(&self) -> &LiveTransferSafetyReport {
        &self.report
    }

    /// The report-layer observations minted by canonical-byte validation.
    #[must_use]
    pub fn report_layer_observations(&self) -> &[ValidatedReportLayerObservation] {
        &self.report_layer_observations
    }

    /// The validated census, including report-layer observations.
    #[must_use]
    pub const fn census(&self) -> LiveEvidenceCensus {
        self.census
    }

    /// The §13.5 items this validation recomputed.
    #[must_use]
    pub const fn recomputed_items(&self) -> &BTreeSet<RecomputedItem> {
        &self.recomputed_items
    }

    /// Every blocker, with how many rows carry it.
    #[must_use]
    pub const fn blockers(&self) -> &BTreeMap<LiveInfrastructureBlocker, usize> {
        &self.blockers
    }

    /// Every required row that is not answered, in §15 order.
    #[must_use]
    pub fn outstanding(&self) -> &[(&'static str, LiveRowStanding)] {
        &self.outstanding
    }
}

/// The completeness one census supports.
///
/// # The mismatch count is asked FIRST, and that ordering is the rule
///
/// A wrong-boundary refusal and a run that has not happened are different
/// conditions, and the enum has carried a distinct token for the first
/// since it was written — `Failed` says a required row was answered at a
/// boundary other than its own. Nothing could reach it, because nothing
/// compared the two boundaries. Now that something does, the mismatch
/// count is tested BEFORE the outstanding count: a matrix with both a
/// contradiction and an ordinary outstanding row is `Failed`, not
/// partial, because a report that renders a contradiction as "still
/// waiting" understates what it found.
const fn completeness_of(census: LiveEvidenceCensus) -> LiveSafetyCompleteness {
    if census.native_refusal_at_unexpected_boundary() > 0 {
        LiveSafetyCompleteness::Failed
    } else if census.every_required_row_is_answered() {
        LiveSafetyCompleteness::CompleteForTheRequiredMatrix
    } else {
        LiveSafetyCompleteness::PartialRequiredRowsOutstanding
    }
}

/// Assemble the safety report one evidence plan determines (§13.2).
///
/// Every carrier is derived from the plan, which is why this returns a
/// report rather than taking one: a constructor accepting the census
/// would let a caller state a conclusion the rows do not support, and
/// [`validate_live_safety_report`] would then be checking the caller
/// against themselves.
///
/// # Errors
///
/// [`LiveSafetyReportRefusal::MalformedRecordedIdentity`] if a historical
/// identity cannot be parsed through the transaction package's target-display
/// API, or [`LiveSafetyReportRefusal::BoundObservationUnbacked`] if a future
/// bound standing arrives without its validated run binding.
pub fn assemble_live_safety_report(
    evidence: &ValidatedCorpusEvidence,
    target: TargetProjection,
) -> Result<LiveTransferSafetyReport, LiveSafetyReportRefusal> {
    let plan = evidence.plan();
    let census = plan.census();
    Ok(LiveTransferSafetyReport {
        schema: LIVE_SAFETY_REPORT_SCHEMA,
        role: LiveSafetyReportRole::LiveTransferSafety,
        target,
        representations: BTreeSet::from([
            LiveTransferRepresentationPlan::Explicit,
            LiveTransferRepresentationPlan::PrivateCommitted,
        ]),
        runs: evidence.runs().to_vec(),
        observations: observations_from_overlay(evidence)?,
        report_layer_requirements: report_layer_requirements_from_plan(plan),
        lifecycle: LiveLifecycleStatus::candidate(),
        census,
        completeness: completeness_of(census),
    })
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ObservationCensus {
    accepted: usize,
    refused: usize,
    determinism: usize,
    paired_relation: usize,
    first_party_fact: usize,
    recorded_unbound: usize,
    unbound_acceptance: usize,
    unbound_refusal: usize,
    unbound_paired_relation: usize,
}

impl ObservationCensus {
    fn from_observations(observations: &[LiveReportObservation]) -> Self {
        let mut census = Self::default();
        for observation in observations {
            match observation {
                LiveReportObservation::NativeAcceptance { .. }
                | LiveReportObservation::CompositeTwoAcceptance { .. } => census.accepted += 1,
                LiveReportObservation::NativeRefusalWithSupport { .. } => census.refused += 1,
                LiveReportObservation::PairedRelation { .. } => census.paired_relation += 1,
                LiveReportObservation::MultiRowSemantic { witness } => {
                    census.accepted += witness.rows.len();
                }
                LiveReportObservation::RecordedObservationUnbound { observation, .. } => {
                    census.recorded_unbound += 1;
                    match observation {
                        LiveRecordedObservation::NativeAcceptance { .. } => {
                            census.unbound_acceptance += 1;
                        }
                        LiveRecordedObservation::NativeRefusal { .. } => {
                            census.unbound_refusal += 1;
                        }
                        LiveRecordedObservation::PairedRelation { .. } => {
                            census.unbound_paired_relation += 1;
                        }
                    }
                }
                LiveReportObservation::Determinism { .. } => census.determinism += 1,
                LiveReportObservation::FirstPartyFact { .. } => census.first_party_fact += 1,
            }
        }
        census
    }

    const fn cross_foots(self, census: LiveEvidenceCensus) -> bool {
        self.accepted == census.native_run_observed()
            && self.refused == census.native_refusal_observed()
            && self.determinism == census.determinism_observed()
            && self.paired_relation == census.paired_relation_observed()
            && self.first_party_fact == census.first_party_fact_observed()
            && census.report_layer_observed() == 0
            && self.recorded_unbound
                == census.recorded_observation_unbound()
                    + census.native_refusal_at_unexpected_boundary()
    }

    const fn total(self) -> usize {
        self.accepted
            + self.refused
            + self.determinism
            + self.paired_relation
            + self.first_party_fact
            + self.recorded_unbound
    }
}

fn parse_recorded_identity(
    row: &'static str,
    identity: &str,
) -> Result<Txid, LiveSafetyReportRefusal> {
    Txid::from_target_display(identity)
        .map_err(|_| LiveSafetyReportRefusal::MalformedRecordedIdentity { row })
}

fn recorded_observation_from_standing(
    row: &'static str,
    recorded: &RecordedObservation,
) -> Result<LiveRecordedObservation, LiveSafetyReportRefusal> {
    match recorded {
        RecordedObservation::NativeAcceptance { accepted_identity } => {
            Ok(LiveRecordedObservation::NativeAcceptance {
                identity: parse_recorded_identity(row, accepted_identity)?,
            })
        }
        RecordedObservation::NativeRefusal {
            declared_boundary,
            observed_layer,
            control_identity,
            refusal_detail,
        } => Ok(LiveRecordedObservation::NativeRefusal {
            declared_boundary: *declared_boundary,
            observed_layer: *observed_layer,
            control_identity: parse_recorded_identity(row, control_identity)?,
            detail: refusal_detail,
        }),
        RecordedObservation::PairedRelation {
            explicit_identity,
            private_identity,
            relation,
        } => Ok(LiveRecordedObservation::PairedRelation {
            explicit_identity: parse_recorded_identity(row, explicit_identity)?,
            private_identity: parse_recorded_identity(row, private_identity)?,
            relation,
        }),
    }
}

fn observations_from_overlay(
    evidence: &ValidatedCorpusEvidence,
) -> Result<Vec<LiveReportObservation>, LiveSafetyReportRefusal> {
    let mut observations = Vec::new();
    let mut emitted_corpus_observations = BTreeSet::new();
    for plan_row in evidence.plan().rows() {
        let row = plan_row.row().name();
        if let Some(attribution) = evidence.attribution_for(row) {
            if emitted_corpus_observations.insert(attribution.observation_index()) {
                observations.push(attribution.observation().clone());
            }
            continue;
        }
        let observation = match plan_row.standing() {
            LiveRowStanding::RecordedObservationUnbound(recorded) => {
                Some(LiveReportObservation::RecordedObservationUnbound {
                    row,
                    observation: recorded_observation_from_standing(row, recorded)?,
                })
            }
            LiveRowStanding::DeterminismObserved {
                recomputed,
                observed_by,
            } => Some(LiveReportObservation::Determinism {
                row,
                recomputed,
                observed_by,
            }),
            LiveRowStanding::FirstPartyFactObserved { fact, observed_by } => {
                Some(LiveReportObservation::FirstPartyFact {
                    row,
                    fact,
                    observed_by,
                })
            }
            LiveRowStanding::NativeRunObserved { .. }
            | LiveRowStanding::NativeRefusalObserved { .. }
            | LiveRowStanding::PairedRelationObserved { .. } => {
                return Err(LiveSafetyReportRefusal::BoundObservationUnbacked { row });
            }
            LiveRowStanding::NativeRefusalAtUnexpectedBoundary {
                declared_boundary,
                observed_layer,
                control_identity,
                refusal_detail,
            } => Some(LiveReportObservation::RecordedObservationUnbound {
                row,
                observation: LiveRecordedObservation::NativeRefusal {
                    declared_boundary: *declared_boundary,
                    observed_layer: *observed_layer,
                    control_identity: parse_recorded_identity(row, control_identity)?,
                    detail: refusal_detail,
                },
            }),
            LiveRowStanding::ReportLayerRequired(_)
            | LiveRowStanding::FirstPartyDischarged { .. }
            | LiveRowStanding::FirstPartyUndischarged(_)
            | LiveRowStanding::NativeRunRequired(_)
            | LiveRowStanding::InfrastructureBlocked(_)
            | LiveRowStanding::OperationVocabularyClosed
            | LiveRowStanding::Experimental => None,
        };
        if let Some(observation) = observation {
            observations.push(observation);
        }
    }
    Ok(observations)
}

fn report_layer_requirements_from_plan(
    plan: &LiveTransferEvidencePlan,
) -> Vec<LiveReportLayerRequirement> {
    plan.rows()
        .iter()
        .filter_map(|evidence| match evidence.standing() {
            LiveRowStanding::ReportLayerRequired(requirement) => Some(LiveReportLayerRequirement {
                row: evidence.row().name(),
                requirement: *requirement,
            }),
            _ => None,
        })
        .collect()
}

#[derive(Debug, Default)]
struct RecomputationProgress {
    completed: BTreeSet<RecomputedItem>,
}

impl RecomputationProgress {
    fn mark(&mut self, item: RecomputedItem) {
        self.completed.insert(item);
    }

    fn finish(self) -> BTreeSet<RecomputedItem> {
        self.completed
    }

    fn completed_with(&self, item: RecomputedItem) -> BTreeSet<RecomputedItem> {
        let mut completed = self.completed.clone();
        completed.insert(item);
        completed
    }
}

// Keep these byte-derived locator rules aligned with the independently proven
// counterparts in `live_corpus_native_v2_r7`.
fn exact_changed_range(left: &[u8], right: &[u8]) -> Option<(usize, usize)> {
    if left == right {
        return None;
    }
    let start = left.iter().zip(right).take_while(|(a, b)| a == b).count();
    let suffix = left
        .iter()
        .rev()
        .zip(right.iter().rev())
        .take_while(|(a, b)| a == b)
        .count()
        .min(left.len().saturating_sub(start))
        .min(right.len().saturating_sub(start));
    Some((start, left.len().saturating_sub(suffix)))
}

fn changed_bytes_are_within(control: &[u8], mutant: &[u8], start: usize, end: usize) -> bool {
    if start >= end {
        return false;
    }
    if end > control.len() {
        return false;
    }
    exact_changed_range(control, mutant).is_some_and(|(changed_start, changed_end)| {
        if changed_start < start {
            return false;
        }
        changed_end <= end
    })
}

fn witness_item_is_exact(
    control: &TargetTransaction,
    mutant: &TargetTransaction,
    input_index: usize,
    item_index: usize,
) -> bool {
    if control.version() != mutant.version()
        || control.inputs() != mutant.inputs()
        || control.outputs() != mutant.outputs()
        || control.lock_time() != mutant.lock_time()
        || control.output_witnesses() != mutant.output_witnesses()
        || control.witnesses().len() != mutant.witnesses().len()
        || control.encode_without_witness() != mutant.encode_without_witness()
    {
        return false;
    }

    let mut differences = 0_usize;
    for (offered_input, (control_witness, mutant_witness)) in control
        .witnesses()
        .iter()
        .zip(mutant.witnesses())
        .enumerate()
    {
        if control_witness.stack().len() != mutant_witness.stack().len() {
            return false;
        }
        for (offered_item, (control_item, mutant_item)) in control_witness
            .stack()
            .iter()
            .zip(mutant_witness.stack())
            .enumerate()
        {
            if control_item != mutant_item {
                if offered_input != input_index || offered_item != item_index {
                    return false;
                }
                differences += 1;
            }
        }
    }
    differences == 1
}

fn sponsor_authorization_is_absent(
    control: &TargetTransaction,
    mutant: &TargetTransaction,
    input_index: usize,
    item_index: usize,
) -> bool {
    if control.version() != mutant.version()
        || control.inputs() != mutant.inputs()
        || control.outputs() != mutant.outputs()
        || control.lock_time() != mutant.lock_time()
        || control.output_witnesses() != mutant.output_witnesses()
        || control.witnesses().len() != mutant.witnesses().len()
        || control.encode_without_witness() != mutant.encode_without_witness()
    {
        return false;
    }
    let mut found = false;
    for (offered_input, (control_witness, mutant_witness)) in control
        .witnesses()
        .iter()
        .zip(mutant.witnesses())
        .enumerate()
    {
        if offered_input != input_index {
            if control_witness != mutant_witness {
                return false;
            }
            continue;
        }
        let control_stack = control_witness.stack();
        let mutant_stack = mutant_witness.stack();
        if item_index >= control_stack.len() || control_stack.len() != mutant_stack.len() {
            return false;
        }
        if control_stack[..item_index] != mutant_stack[..item_index] {
            return false;
        }
        if control_stack[item_index..].iter().any(Vec::is_empty) {
            return false;
        }
        if mutant_stack[item_index..]
            .iter()
            .any(|item| !item.is_empty())
        {
            return false;
        }
        found = true;
    }
    found
}

const TAPSCRIPT_LEAF_VERSION: u8 = LeafVersion::TAPSCRIPT.get();
const MAXIMUM_CONTROL_BLOCK_DEPTH: usize = 128;

const fn control_block_is_structurally_valid(block: &[u8]) -> bool {
    block.len() >= CONTROL_BASE_BYTES
        && (block.len() - CONTROL_BASE_BYTES).is_multiple_of(DIGEST_BYTES)
        && (block.len() - CONTROL_BASE_BYTES) / DIGEST_BYTES <= MAXIMUM_CONTROL_BLOCK_DEPTH
        && block[0] & TAPROOT_LEAF_MASK == TAPSCRIPT_LEAF_VERSION
}

fn decoded_witness_path_role(stack: &[Vec<u8>]) -> Option<LiveWitnessPathRole> {
    if stack.len() == 1 && !stack[0].is_empty() {
        return Some(LiveWitnessPathRole::KeyPath);
    }
    let (control_block, preceding) = stack.split_last()?;
    let leaf_program = preceding.last()?;
    (!leaf_program.is_empty() && control_block_is_structurally_valid(control_block))
        .then_some(LiveWitnessPathRole::ScriptPath)
}

fn exact_changed_witness_positions(control: &[Vec<u8>], mutant: &[Vec<u8>]) -> Vec<usize> {
    (0..control.len().max(mutant.len()))
        .filter(|position| control.get(*position) != mutant.get(*position))
        .collect()
}

fn witness_path_shape_matches(
    control: &TargetTransaction,
    mutant: &TargetTransaction,
    locator: &LiveMutationLocator,
) -> bool {
    let LiveMutationLocator::WitnessPathShape {
        input_index,
        control_stack_items,
        mutant_stack_items,
        changed_positions,
        control_role,
        mutant_role,
        witnessless_serialization_equal,
    } = locator
    else {
        return false;
    };
    if !*witnessless_serialization_equal
        || control.encode_without_witness() != mutant.encode_without_witness()
        || control.output_witnesses() != mutant.output_witnesses()
        || control.witnesses().len() != mutant.witnesses().len()
        || changed_positions
            .windows(2)
            .any(|positions| positions[0] >= positions[1])
    {
        return false;
    }
    let Some(control_witness) = control.witnesses().get(*input_index) else {
        return false;
    };
    let Some(mutant_witness) = mutant.witnesses().get(*input_index) else {
        return false;
    };
    if control
        .witnesses()
        .iter()
        .zip(mutant.witnesses())
        .enumerate()
        .any(|(index, pair)| index != *input_index && pair.0 != pair.1)
    {
        return false;
    }
    let control_stack = control_witness.stack();
    let mutant_stack = mutant_witness.stack();
    control_stack.len() == *control_stack_items
        && mutant_stack.len() == *mutant_stack_items
        && decoded_witness_path_role(control_stack) == Some(*control_role)
        && decoded_witness_path_role(mutant_stack) == Some(*mutant_role)
        && exact_changed_witness_positions(control_stack, mutant_stack).as_slice()
            == changed_positions.as_slice()
        && !changed_positions.is_empty()
}

fn revealed_committed_leaf_program(
    transaction: &TargetTransaction,
    input_index: usize,
) -> Option<Vec<u8>> {
    let stack = transaction.witnesses().get(input_index)?.stack();
    if decoded_witness_path_role(stack) != Some(LiveWitnessPathRole::ScriptPath) {
        return None;
    }
    stack.get(stack.len().checked_sub(2)?).cloned()
}

fn committed_leaf_arrangement_matches(
    control: &TargetTransaction,
    mutant: &TargetTransaction,
    mutant_kind: LiveMutantKind,
    locator: &LiveMutationLocator,
) -> bool {
    let LiveMutationLocator::CommittedLeafArrangement {
        input_indices,
        control_coordinator_leaf_indices,
        mutant_coordinator_leaf_indices,
        control_committed_leaf_programs,
        mutant_committed_leaf_programs,
    } = locator
    else {
        return false;
    };
    if input_indices.is_empty()
        || input_indices
            .windows(2)
            .any(|positions| positions[0] >= positions[1])
        || control.encode_without_witness() != mutant.encode_without_witness()
        || control.output_witnesses() != mutant.output_witnesses()
        || control.witnesses().len() != mutant.witnesses().len()
        || input_indices.len() != control_committed_leaf_programs.len()
        || input_indices.len() != mutant_committed_leaf_programs.len()
    {
        return false;
    }
    let Some(control_programs) = input_indices
        .iter()
        .map(|index| revealed_committed_leaf_program(control, *index))
        .collect::<Option<Vec<_>>>()
    else {
        return false;
    };
    let Some(mutant_programs) = input_indices
        .iter()
        .map(|index| revealed_committed_leaf_program(mutant, *index))
        .collect::<Option<Vec<_>>>()
    else {
        return false;
    };
    if control_programs.as_slice() != control_committed_leaf_programs.as_slice()
        || mutant_programs.as_slice() != mutant_committed_leaf_programs.as_slice()
        || control_programs == mutant_programs
    {
        return false;
    }
    let coordinator_program = &control_programs[0];
    let recomputed_control_indices = input_indices
        .iter()
        .zip(&control_programs)
        .filter_map(|(index, program)| (program == coordinator_program).then_some(*index))
        .collect::<Vec<_>>();
    let recomputed_mutant_indices = input_indices
        .iter()
        .zip(&mutant_programs)
        .filter_map(|(index, program)| (program == coordinator_program).then_some(*index))
        .collect::<Vec<_>>();
    let expected_mutant_coordinators = match mutant_kind {
        LiveMutantKind::TwoCoordinators => 2,
        LiveMutantKind::NoCoordinator => 0,
        _ => return false,
    };
    recomputed_control_indices.as_slice() == control_coordinator_leaf_indices.as_slice()
        && recomputed_control_indices.as_slice() == [input_indices[0]]
        && recomputed_mutant_indices.as_slice() == mutant_coordinator_leaf_indices.as_slice()
        && recomputed_mutant_indices.len() == expected_mutant_coordinators
}

fn mutation_locator_matches(
    control: &TargetTransaction,
    control_bytes: &[u8],
    mutant: &TargetTransaction,
    mutant_bytes: &[u8],
    mutant_kind: LiveMutantKind,
    locator: &LiveMutationLocator,
) -> bool {
    match locator {
        LiveMutationLocator::SerializedOutputField(locator) => {
            let Ok(control_field) = control.locate_serialized_field(*locator) else {
                return false;
            };
            changed_bytes_are_within(
                control_bytes,
                mutant_bytes,
                control_field.range().start,
                control_field.range().end,
            )
        }
        LiveMutationLocator::WitnessItem {
            input_index,
            item_index,
        } => match mutant_kind {
            LiveMutantKind::MissingSponsorAuthorization => {
                sponsor_authorization_is_absent(control, mutant, *input_index, *item_index)
            }
            _ => witness_item_is_exact(control, mutant, *input_index, *item_index),
        },
        LiveMutationLocator::WitnesslessRange { start, end } => {
            exact_changed_range(
                &control.encode_without_witness(),
                &mutant.encode_without_witness(),
            ) == Some((*start, *end))
        }
        LiveMutationLocator::TransactionShape {
            control_inputs,
            mutant_inputs,
            control_outputs,
            mutant_outputs,
        } => {
            control.inputs().len() == *control_inputs
                && mutant.inputs().len() == *mutant_inputs
                && control.outputs().len() == *control_outputs
                && mutant.outputs().len() == *mutant_outputs
                && (control_inputs != mutant_inputs || control_outputs != mutant_outputs)
                && control.version() == mutant.version()
                && control.lock_time() == mutant.lock_time()
        }
        LiveMutationLocator::WitnessPathShape { .. } => {
            witness_path_shape_matches(control, mutant, locator)
        }
        LiveMutationLocator::CommittedLeafArrangement { .. } => {
            committed_leaf_arrangement_matches(control, mutant, mutant_kind, locator)
        }
    }
}

fn validate_archived_run_facts(run: &LiveRunBinding) -> Result<(), LiveSafetyReportRefusal> {
    let parsed = parse_run_archive(&run.archive_bytes)?;
    if run.revision != parsed.revision {
        return Err(LiveSafetyReportRefusal::RunProtocolRevisionDiffers(
            run.run_id.clone(),
        ));
    }
    if run.digest_facts != parsed.digest_facts
        || run.digest_facts.values().any(|fact| {
            !matches!(
                (run.revision, fact.algorithm),
                (
                    NativeProtocolRevision::Revision6,
                    FixtureDigestAlgorithm::HistoricalV1
                ) | (
                    NativeProtocolRevision::Revision7,
                    FixtureDigestAlgorithm::ForwardV2
                )
            )
        })
    {
        return Err(LiveSafetyReportRefusal::RunDigestFactsDiffer(
            run.run_id.clone(),
        ));
    }
    if run.deployment != parsed.deployment {
        return Err(LiveSafetyReportRefusal::RunDeploymentDiffers(
            run.run_id.clone(),
        ));
    }
    if run.executor != parsed.executor {
        return Err(LiveSafetyReportRefusal::RunExecutorProvenanceDiffers(
            run.run_id.clone(),
        ));
    }
    Ok(())
}

fn decode_run_requests(
    run: &LiveRunBinding,
) -> Result<BTreeMap<String, TargetTransaction>, LiveSafetyReportRefusal> {
    if !run.requests.keys().eq(run.request_facts.keys())
        || !run.requests.keys().eq(run.responses.keys())
    {
        return Err(LiveSafetyReportRefusal::RequestResponseCensusDiffers(
            run.run_id.clone(),
        ));
    }

    let mut decoded = BTreeMap::new();
    for (request_id, bytes) in &run.requests {
        let transaction = TargetTransaction::decode(bytes).map_err(|_| {
            LiveSafetyReportRefusal::RequestDecodeRefused(run.run_id.clone(), request_id.clone())
        })?;
        if transaction.encode() != *bytes {
            return Err(LiveSafetyReportRefusal::RequestDecodeRefused(
                run.run_id.clone(),
                request_id.clone(),
            ));
        }
        let response = &run.responses[request_id];
        let fact = &run.request_facts[request_id];
        match (fact, response) {
            (
                LiveRequestFact::Acceptance
                | LiveRequestFact::Control
                | LiveRequestFact::Paired { .. },
                LiveTargetResponse::Accepted { identity },
            ) => {
                let recomputed = recomputed_txid(bytes).ok_or_else(|| {
                    LiveSafetyReportRefusal::RequestDecodeRefused(
                        run.run_id.clone(),
                        request_id.clone(),
                    )
                })?;
                if identity != &recomputed {
                    return Err(LiveSafetyReportRefusal::AcceptedIdentityDiffers(
                        run.run_id.clone(),
                        request_id.clone(),
                    ));
                }
            }
            (LiveRequestFact::Refusal { .. }, LiveTargetResponse::Refused { .. }) => {}
            _ => {
                return Err(LiveSafetyReportRefusal::RequestResponseShapeDiffers(
                    run.run_id.clone(),
                    request_id.clone(),
                ));
            }
        }
        decoded.insert(request_id.clone(), transaction);
    }
    Ok(decoded)
}

fn validate_refusal_links(
    run: &LiveRunBinding,
    request_owners: &BTreeMap<&str, &str>,
    decoded: &BTreeMap<String, TargetTransaction>,
) -> Result<(), LiveSafetyReportRefusal> {
    for (request_id, fact) in &run.request_facts {
        let LiveRequestFact::Refusal {
            mutant,
            control_request_id,
            locator,
        } = fact
        else {
            continue;
        };
        let Some(control_owner) = request_owners.get(control_request_id.as_str()) else {
            return Err(LiveSafetyReportRefusal::RequestRoleDiffers(
                run.run_id.clone(),
                request_id.clone(),
            ));
        };
        if *control_owner != run.run_id.as_str() {
            return Err(LiveSafetyReportRefusal::CrossRunRequestLink(
                request_id.clone(),
                control_request_id.clone(),
            ));
        }
        if run.request_facts.get(control_request_id) != Some(&LiveRequestFact::Control) {
            return Err(LiveSafetyReportRefusal::RequestRoleDiffers(
                run.run_id.clone(),
                control_request_id.clone(),
            ));
        }
        let LiveTargetResponse::Accepted {
            identity: control_identity,
        } = &run.responses[control_request_id]
        else {
            return Err(LiveSafetyReportRefusal::RequestResponseShapeDiffers(
                run.run_id.clone(),
                control_request_id.clone(),
            ));
        };
        let LiveTargetResponse::Refused {
            control_identity: linked_identity,
            ..
        } = &run.responses[request_id]
        else {
            return Err(LiveSafetyReportRefusal::RequestResponseShapeDiffers(
                run.run_id.clone(),
                request_id.clone(),
            ));
        };
        if linked_identity != control_identity {
            return Err(LiveSafetyReportRefusal::RefusalControlIdentityDiffers(
                run.run_id.clone(),
                request_id.clone(),
            ));
        }
        if !mutation_locator_matches(
            &decoded[control_request_id],
            &run.requests[control_request_id],
            &decoded[request_id],
            &run.requests[request_id],
            *mutant,
            locator,
        ) {
            return Err(LiveSafetyReportRefusal::MutationLocatorDiffers(
                run.run_id.clone(),
                request_id.clone(),
            ));
        }
    }
    Ok(())
}

fn validate_run_binding(
    run: &LiveRunBinding,
    request_owners: &BTreeMap<&str, &str>,
) -> Result<(), LiveSafetyReportRefusal> {
    validate_archived_run_facts(run)?;
    let decoded = decode_run_requests(run)?;
    validate_refusal_links(run, request_owners, &decoded)?;
    if run.run_id != content_address_run(run) {
        return Err(LiveSafetyReportRefusal::RunIdDiffers(run.run_id.clone()));
    }
    Ok(())
}

fn validate_run_bindings(runs: &[LiveRunBinding]) -> Result<(), LiveSafetyReportRefusal> {
    if runs.windows(2).any(|pair| pair[0].run_id >= pair[1].run_id) {
        return Err(LiveSafetyReportRefusal::RunCensusDiffers);
    }
    let mut request_owners = BTreeMap::new();
    for run in runs {
        for request_id in run.requests.keys() {
            if request_owners
                .insert(request_id.as_str(), run.run_id.as_str())
                .is_some()
            {
                return Err(LiveSafetyReportRefusal::ReusedRequestId(request_id.clone()));
            }
        }
    }
    for run in runs {
        validate_run_binding(run, &request_owners)?;
    }
    Ok(())
}

fn compare_run_bindings(
    offered: &[LiveRunBinding],
    expected: &[LiveRunBinding],
) -> Result<(), LiveSafetyReportRefusal> {
    if offered.len() != expected.len() {
        return Err(LiveSafetyReportRefusal::RunCensusDiffers);
    }
    validate_run_bindings(offered)?;
    validate_run_bindings(expected)?;
    for (offered, expected) in offered.iter().zip(expected) {
        if offered.run_id != expected.run_id {
            return Err(LiveSafetyReportRefusal::RunCensusDiffers);
        }
        if offered.archive_bytes != expected.archive_bytes {
            return Err(LiveSafetyReportRefusal::RunArchiveDiffers(
                offered.run_id.clone(),
            ));
        }
        if offered.revision != expected.revision {
            return Err(LiveSafetyReportRefusal::RunProtocolRevisionDiffers(
                offered.run_id.clone(),
            ));
        }
        if offered.digest_facts != expected.digest_facts {
            return Err(LiveSafetyReportRefusal::RunDigestFactsDiffer(
                offered.run_id.clone(),
            ));
        }
        if offered.deployment != expected.deployment {
            return Err(LiveSafetyReportRefusal::RunDeploymentDiffers(
                offered.run_id.clone(),
            ));
        }
        if offered.requests != expected.requests || offered.request_facts != expected.request_facts
        {
            return Err(LiveSafetyReportRefusal::RunRequestDiffers(
                offered.run_id.clone(),
            ));
        }
        if offered.responses != expected.responses {
            return Err(LiveSafetyReportRefusal::RunResponseDiffers(
                offered.run_id.clone(),
            ));
        }
        if offered.executor != expected.executor {
            return Err(LiveSafetyReportRefusal::RunExecutorProvenanceDiffers(
                offered.run_id.clone(),
            ));
        }
    }
    Ok(())
}

fn request_for<'run>(
    runs: &'run [LiveRunBinding],
    run_id: &str,
    request_id: &str,
    row: &'static str,
) -> Result<BoundRequest<'run>, LiveSafetyReportRefusal> {
    let run = runs
        .iter()
        .find(|run| run.run_id == run_id)
        .ok_or(LiveSafetyReportRefusal::BoundObservationUnbacked { row })?;
    let fact = run
        .request_facts
        .get(request_id)
        .ok_or(LiveSafetyReportRefusal::BoundObservationUnbacked { row })?;
    let response = run
        .responses
        .get(request_id)
        .ok_or(LiveSafetyReportRefusal::BoundObservationUnbacked { row })?;
    let bytes = run
        .requests
        .get(request_id)
        .ok_or(LiveSafetyReportRefusal::BoundObservationUnbacked { row })?;
    Ok((fact, response, bytes))
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RecomputedPairProjection {
    version: u32,
    lock_time: u32,
    input_owners: BTreeSet<String>,
    semantic_input_amounts: Vec<u64>,
    destinations: Vec<(String, u64)>,
    destination_owners: BTreeSet<String>,
    explicit_asset: Option<[u8; 32]>,
    every_input_authorized: bool,
    sponsor_inputs: usize,
    fee_outputs: usize,
    publishes_every_destination_amount: bool,
    all_outputs_classified: bool,
}

fn recompute_pair_projection(
    bytes: &[u8],
    input: &LivePairProjectionInput,
) -> Option<RecomputedPairProjection> {
    if input.input_owners.len() != input.semantic_input_amounts.len() {
        return None;
    }
    let decoded = TargetTransaction::decode(bytes).ok()?;
    if decoded.encode() != bytes || input.input_owners.len() > decoded.inputs().len() {
        return None;
    }

    let mut semantic_input_amounts = input.semantic_input_amounts.clone();
    semantic_input_amounts.sort_unstable();
    let input_owners = input.input_owners.iter().cloned().collect::<BTreeSet<_>>();
    let mut assets = BTreeSet::new();
    let mut fee_outputs = 0_usize;
    let mut destinations = Vec::new();
    let mut destination_owners = BTreeSet::new();
    let mut publishes_every_destination_amount = true;

    for output in decoded.outputs() {
        if let AssetField::Explicit(asset) = output.asset() {
            assets.insert(*asset.internal());
        }
        if output.is_fee() {
            fee_outputs += 1;
            continue;
        }
        let mut matches = input
            .destination_programs
            .iter()
            .filter(|(_, program)| program.as_slice() == output.program());
        let (owner, _) = matches.next()?;
        if matches.next().is_some() || !destination_owners.insert(owner.clone()) {
            return None;
        }
        let amount = *input.semantic_destination_amounts.get(owner)?;
        match output.value() {
            ValueField::Explicit(published) if published == amount => {}
            ValueField::Commitment(_) => publishes_every_destination_amount = false,
            _ => return None,
        }
        destinations.push((owner.clone(), amount));
    }
    if destination_owners.len() != input.destination_programs.len()
        || !destination_owners
            .iter()
            .eq(input.semantic_destination_amounts.keys())
    {
        return None;
    }
    destinations.sort();
    let explicit_asset = if assets.len() == 1 {
        assets.into_iter().next()
    } else {
        None
    };
    let all_outputs_classified = destination_owners.len() + fee_outputs == decoded.outputs().len();
    Some(RecomputedPairProjection {
        version: decoded.version(),
        lock_time: decoded.lock_time(),
        input_owners,
        semantic_input_amounts,
        destinations,
        destination_owners,
        explicit_asset,
        every_input_authorized: decoded.witnesses().iter().all(|witness| !witness.is_null()),
        sponsor_inputs: decoded.inputs().len() - input.input_owners.len(),
        fee_outputs,
        publishes_every_destination_amount,
        all_outputs_classified,
    })
}

const VALIDATED_PAIR_RELATION: &str = "all-11-terms-equal;withheld-by-private=2";

fn pair_relation_recomputes(
    explicit_bytes: &[u8],
    explicit_input: &LivePairProjectionInput,
    private_bytes: &[u8],
    private_input: &LivePairProjectionInput,
) -> bool {
    let Some(explicit) = recompute_pair_projection(explicit_bytes, explicit_input) else {
        return false;
    };
    let Some(private) = recompute_pair_projection(private_bytes, private_input) else {
        return false;
    };
    let terms = [
        explicit.semantic_input_amounts == private.semantic_input_amounts,
        explicit.destinations == private.destinations,
        explicit.input_owners == private.input_owners,
        !explicit.destinations.is_empty() && !private.destinations.is_empty(),
        explicit.explicit_asset.is_some() && explicit.explicit_asset == private.explicit_asset,
        explicit.every_input_authorized && private.every_input_authorized,
        explicit.version == private.version,
        explicit.lock_time == private.lock_time,
        explicit.destination_owners == private.destination_owners,
        (explicit.sponsor_inputs, explicit.fee_outputs)
            == (private.sponsor_inputs, private.fee_outputs),
        explicit.all_outputs_classified && private.all_outputs_classified,
    ];
    let withheld_terms = [
        explicit.publishes_every_destination_amount,
        !private.publishes_every_destination_amount,
    ]
    .into_iter()
    .filter(|withheld| *withheld)
    .count();
    terms.into_iter().all(|agrees| agrees) && withheld_terms == 2
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ObservationLinkClass {
    OrdinaryAcceptance(&'static str),
    CompositeAcceptanceMember(&'static str),
    NativeRefusal,
    PairMember,
    MultiRowSemantic,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ObservationLinkStanding {
    Single(ObservationLinkClass),
    CompositeAndOrdinaryAcceptance,
}

fn insert_observation_link(
    observed: &mut BTreeMap<(String, String), ObservationLinkStanding>,
    run_id: &str,
    request_id: &str,
    class: ObservationLinkClass,
) -> Result<(), LiveSafetyReportRefusal> {
    let key = (run_id.to_owned(), request_id.to_owned());
    let Some(standing) = observed.get_mut(&key) else {
        observed.insert(key, ObservationLinkStanding::Single(class));
        return Ok(());
    };
    match (*standing, class) {
        (
            ObservationLinkStanding::Single(ObservationLinkClass::OrdinaryAcceptance(
                "private-one-to-one",
            )),
            ObservationLinkClass::CompositeAcceptanceMember("both-commitment-parity-forms"),
        )
        | (
            ObservationLinkStanding::Single(ObservationLinkClass::CompositeAcceptanceMember(
                "both-commitment-parity-forms",
            )),
            ObservationLinkClass::OrdinaryAcceptance("private-one-to-one"),
        ) => {
            *standing = ObservationLinkStanding::CompositeAndOrdinaryAcceptance;
            Ok(())
        }
        _ => Err(LiveSafetyReportRefusal::ObservationAliased(
            request_id.to_owned(),
        )),
    }
}

#[derive(Debug, Default)]
struct BoundObservationValidation {
    compared: usize,
    observed_links: BTreeMap<(String, String), ObservationLinkStanding>,
    support_links: BTreeMap<String, LiveSupportLink>,
    used_requests: BTreeSet<(String, String)>,
}

const CONSERVATION_CONTROL_ROW: &str = "target-ct-conservation";
const CONSERVATION_CONTROL_REQUEST: &str = "conservation-negatives/request-6";

fn ordinary_acceptance_role_matches(row: &str, request_id: &str, fact: &LiveRequestFact) -> bool {
    // `live_corpus_native_v2_r7::add_conservation_material` deliberately
    // cites the accepted control for the conservation-negative family as the
    // positive conservation witness. Keep that typed reuse closed to its
    // exact row and global request; every other ordinary acceptance remains
    // an acceptance-role request.
    if row == CONSERVATION_CONTROL_ROW {
        request_id == CONSERVATION_CONTROL_REQUEST && fact == &LiveRequestFact::Control
    } else {
        fact == &LiveRequestFact::Acceptance
    }
}

impl BoundObservationValidation {
    fn acceptance(
        &mut self,
        observation: &LiveReportObservation,
        runs: &[LiveRunBinding],
    ) -> Result<(), LiveSafetyReportRefusal> {
        let LiveReportObservation::NativeAcceptance {
            row,
            run_id,
            request_id,
            identity,
        } = observation
        else {
            return Ok(());
        };
        insert_observation_link(
            &mut self.observed_links,
            run_id,
            request_id,
            ObservationLinkClass::OrdinaryAcceptance(row),
        )?;
        let (fact, response, _) = request_for(runs, run_id, request_id, row)?;
        if !ordinary_acceptance_role_matches(row, request_id, fact) {
            return Err(LiveSafetyReportRefusal::RequestRoleDiffers(
                run_id.clone(),
                request_id.clone(),
            ));
        }
        let expected = LiveTargetResponse::Accepted {
            identity: *identity,
        };
        if response != &expected {
            return Err(LiveSafetyReportRefusal::RunResponseDiffers(run_id.clone()));
        }
        self.used_requests
            .insert((run_id.clone(), request_id.clone()));
        self.compared += 1;
        Ok(())
    }

    fn composite_two_acceptance(
        &mut self,
        observation: &LiveReportObservation,
        runs: &[LiveRunBinding],
    ) -> Result<(), LiveSafetyReportRefusal> {
        let LiveReportObservation::CompositeTwoAcceptance { row, acceptance } = observation else {
            return Ok(());
        };
        let [first, second] = acceptance.members();
        for member in [first, second] {
            let run = runs
                .iter()
                .find(|run| run.run_id == member.run_id)
                .ok_or(LiveSafetyReportRefusal::BoundObservationUnbacked { row })?;
            let parsed = parse_run_archive(&run.archive_bytes)?;
            if parsed.ceremony_id.as_deref() != Some(member.ceremony.as_str()) {
                return Err(LiveSafetyReportRefusal::RunArchiveDiffers(
                    member.run_id.clone(),
                ));
            }
        }
        if first.ceremony.is_empty()
            || second.ceremony.is_empty()
            || first.ceremony == second.ceremony
            || first.run_id == second.run_id
        {
            return Err(LiveSafetyReportRefusal::CompositeCeremonyReused(
                first.ceremony.clone(),
            ));
        }
        let mut prefixes = Vec::with_capacity(2);
        for member in [first, second] {
            insert_observation_link(
                &mut self.observed_links,
                &member.run_id,
                &member.request_id,
                ObservationLinkClass::CompositeAcceptanceMember(row),
            )?;
            let (fact, response, bytes) =
                request_for(runs, &member.run_id, &member.request_id, row)?;
            if fact != &LiveRequestFact::Acceptance {
                return Err(LiveSafetyReportRefusal::RequestRoleDiffers(
                    member.run_id.clone(),
                    member.request_id.clone(),
                ));
            }
            let expected = LiveTargetResponse::Accepted {
                identity: member.identity,
            };
            if response != &expected {
                return Err(LiveSafetyReportRefusal::RunResponseDiffers(
                    member.run_id.clone(),
                ));
            }
            if recomputed_txid(bytes) != Some(member.identity) {
                return Err(LiveSafetyReportRefusal::AcceptedIdentityDiffers(
                    member.run_id.clone(),
                    member.request_id.clone(),
                ));
            }
            let transaction = TargetTransaction::decode(bytes).map_err(|_| {
                LiveSafetyReportRefusal::RequestDecodeRefused(
                    member.run_id.clone(),
                    member.request_id.clone(),
                )
            })?;
            let prefix = transaction
                .outputs()
                .get(member.consumed_commitment_output_index)
                .and_then(|output| match output.value() {
                    ValueField::Commitment(commitment) => Some(commitment[0]),
                    _ => None,
                })
                .filter(|prefix| matches!(*prefix, 0x08 | 0x09))
                .ok_or(LiveSafetyReportRefusal::CompositeParityNotOpposite(row))?;
            prefixes.push(prefix);
            self.used_requests
                .insert((member.run_id.clone(), member.request_id.clone()));
        }
        if prefixes.as_slice() != [0x08, 0x09] && prefixes.as_slice() != [0x09, 0x08] {
            return Err(LiveSafetyReportRefusal::CompositeParityNotOpposite(row));
        }
        self.compared += 1;
        Ok(())
    }

    fn support_identity(
        &mut self,
        support: &LiveSupportLink,
        mutant_run_id: &str,
        linked_request_id: &str,
        row: &'static str,
        runs: &[LiveRunBinding],
    ) -> Result<Txid, LiveSafetyReportRefusal> {
        if support.run_id != mutant_run_id {
            return Err(LiveSafetyReportRefusal::CrossRunRequestLink(
                linked_request_id.to_owned(),
                support.request_id.clone(),
            ));
        }
        if support.request_id != linked_request_id {
            return Err(LiveSafetyReportRefusal::RequestRoleDiffers(
                support.run_id.clone(),
                support.request_id.clone(),
            ));
        }
        if let Some(previous) = self.support_links.get(&support.request_id) {
            if previous.run_id != support.run_id {
                return Err(LiveSafetyReportRefusal::CrossRunRequestLink(
                    previous.request_id.clone(),
                    support.request_id.clone(),
                ));
            }
            if previous.request_bytes != support.request_bytes
                || previous.response != support.response
            {
                return Err(LiveSafetyReportRefusal::SupportLinkFactsDiffer(
                    support.run_id.clone(),
                    support.request_id.clone(),
                ));
            }
        }
        let (fact, response, bytes) = request_for(runs, &support.run_id, &support.request_id, row)?;
        if fact != &LiveRequestFact::Control {
            return Err(LiveSafetyReportRefusal::RequestRoleDiffers(
                support.run_id.clone(),
                support.request_id.clone(),
            ));
        }
        if bytes != support.request_bytes.as_slice() || response != &support.response {
            return Err(LiveSafetyReportRefusal::SupportLinkFactsDiffer(
                support.run_id.clone(),
                support.request_id.clone(),
            ));
        }
        let LiveTargetResponse::Accepted { identity } = response else {
            return Err(LiveSafetyReportRefusal::RequestResponseShapeDiffers(
                support.run_id.clone(),
                support.request_id.clone(),
            ));
        };
        self.support_links
            .insert(support.request_id.clone(), support.clone());
        self.used_requests
            .insert((support.run_id.clone(), support.request_id.clone()));
        Ok(*identity)
    }

    fn refusal_with_support(
        &mut self,
        observation: &LiveReportObservation,
        runs: &[LiveRunBinding],
    ) -> Result<(), LiveSafetyReportRefusal> {
        let LiveReportObservation::NativeRefusalWithSupport {
            row,
            run_id,
            request_id,
            observed_layer,
            detail,
            declared_boundary,
            support,
        } = observation
        else {
            return Ok(());
        };
        insert_observation_link(
            &mut self.observed_links,
            run_id,
            request_id,
            ObservationLinkClass::NativeRefusal,
        )?;
        let (fact, response, _) = request_for(runs, run_id, request_id, row)?;
        let LiveRequestFact::Refusal {
            mutant,
            control_request_id,
            ..
        } = fact
        else {
            return Err(LiveSafetyReportRefusal::RequestRoleDiffers(
                run_id.clone(),
                request_id.clone(),
            ));
        };
        let boundary = crate::live_safety::required_safety_matrix()
            .into_iter()
            .find(|candidate| candidate.name() == mutant.row())
            .and_then(crate::live_safety::LiveSafetyRow::refusing_layer);
        if mutant.row() != *row || boundary != Some(*declared_boundary) {
            return Err(LiveSafetyReportRefusal::RequestRoleDiffers(
                run_id.clone(),
                request_id.clone(),
            ));
        }
        let control_identity =
            self.support_identity(support, run_id, control_request_id, row, runs)?;
        let expected = LiveTargetResponse::Refused {
            observed_layer: *observed_layer,
            detail: detail.clone(),
            control_identity,
        };
        if response != &expected {
            return Err(LiveSafetyReportRefusal::RunResponseDiffers(run_id.clone()));
        }
        self.used_requests
            .insert((run_id.clone(), request_id.clone()));
        self.compared += 1;
        Ok(())
    }

    fn pair(
        &mut self,
        observation: &LiveReportObservation,
        runs: &[LiveRunBinding],
    ) -> Result<(), LiveSafetyReportRefusal> {
        let LiveReportObservation::PairedRelation {
            row,
            explicit_run_id,
            explicit_request_id,
            explicit_identity,
            private_run_id,
            private_request_id,
            private_identity,
            relation,
        } = observation
        else {
            return Ok(());
        };
        if explicit_run_id != private_run_id {
            return Err(LiveSafetyReportRefusal::CrossRunRequestLink(
                explicit_request_id.clone(),
                private_request_id.clone(),
            ));
        }
        insert_observation_link(
            &mut self.observed_links,
            explicit_run_id,
            explicit_request_id,
            ObservationLinkClass::PairMember,
        )?;
        insert_observation_link(
            &mut self.observed_links,
            private_run_id,
            private_request_id,
            ObservationLinkClass::PairMember,
        )?;
        let explicit = request_for(runs, explicit_run_id, explicit_request_id, row)?;
        let private = request_for(runs, private_run_id, private_request_id, row)?;
        validate_pair_claim(
            row,
            &PairClaimSide {
                run_id: explicit_run_id,
                request_id: explicit_request_id,
                identity: *explicit_identity,
                request: explicit,
            },
            &PairClaimSide {
                run_id: private_run_id,
                request_id: private_request_id,
                identity: *private_identity,
                request: private,
            },
            relation,
        )?;
        self.used_requests
            .insert((explicit_run_id.clone(), explicit_request_id.clone()));
        self.used_requests
            .insert((private_run_id.clone(), private_request_id.clone()));
        self.compared += 1;
        Ok(())
    }

    fn multi_row_semantic(
        &mut self,
        observation: &LiveReportObservation,
        runs: &[LiveRunBinding],
    ) -> Result<(), LiveSafetyReportRefusal> {
        let LiveReportObservation::MultiRowSemantic { witness } = observation else {
            return Ok(());
        };
        let expected_rows = BTreeSet::from(["sponsor-change-absent", "sponsored"]);
        if witness.rows != expected_rows
            || !witness
                .rows
                .iter()
                .copied()
                .eq(witness.predicates.keys().copied())
        {
            return Err(LiveSafetyReportRefusal::MultiRowWitnessRowsDiffer);
        }
        insert_observation_link(
            &mut self.observed_links,
            &witness.run_id,
            &witness.request_id,
            ObservationLinkClass::MultiRowSemantic,
        )?;
        let row = "sponsored";
        let (fact, response, bytes) = request_for(runs, &witness.run_id, &witness.request_id, row)?;
        if fact != &LiveRequestFact::Acceptance {
            return Err(LiveSafetyReportRefusal::RequestRoleDiffers(
                witness.run_id.clone(),
                witness.request_id.clone(),
            ));
        }
        let expected = LiveTargetResponse::Accepted {
            identity: witness.identity,
        };
        if response != &expected || recomputed_txid(bytes) != Some(witness.identity) {
            return Err(LiveSafetyReportRefusal::AcceptedIdentityDiffers(
                witness.run_id.clone(),
                witness.request_id.clone(),
            ));
        }
        let transaction = TargetTransaction::decode(bytes).map_err(|_| {
            LiveSafetyReportRefusal::RequestDecodeRefused(
                witness.run_id.clone(),
                witness.request_id.clone(),
            )
        })?;
        for (predicate_row, predicate) in &witness.predicates {
            if !semantic_predicate_recomputes(predicate_row, predicate, &transaction) {
                return Err(LiveSafetyReportRefusal::SemanticPredicateNotProven(
                    predicate_row,
                ));
            }
        }
        self.used_requests
            .insert((witness.run_id.clone(), witness.request_id.clone()));
        self.compared += witness.rows.len();
        Ok(())
    }
}

fn sponsored_request_shape_recomputes(
    transaction: &TargetTransaction,
    sponsor_input_index: usize,
    fee_output_index: usize,
) -> bool {
    let sponsor_inputs = transaction
        .witnesses()
        .iter()
        .enumerate()
        .filter_map(|(index, witness)| {
            let stack = witness.stack();
            (stack.len() == 2 && stack.iter().all(|item| !item.is_empty())).then_some(index)
        })
        .collect::<Vec<_>>();
    let fee_outputs = transaction
        .outputs()
        .iter()
        .enumerate()
        .filter_map(|(index, output)| output.is_fee().then_some(index))
        .collect::<Vec<_>>();
    sponsor_inputs.as_slice() == [sponsor_input_index]
        && fee_outputs.as_slice() == [fee_output_index]
}

fn semantic_predicate_recomputes(
    row: &'static str,
    predicate: &LiveRowSemanticPredicate,
    transaction: &TargetTransaction,
) -> bool {
    match (row, predicate) {
        (
            "sponsored",
            LiveRowSemanticPredicate::Sponsored {
                sponsor_input_index,
                fee_output_index,
            },
        ) => {
            sponsored_request_shape_recomputes(transaction, *sponsor_input_index, *fee_output_index)
        }
        (
            "sponsor-change-absent",
            LiveRowSemanticPredicate::SponsorChangeAbsent {
                sponsor_input_index,
                fee_output_index,
                sponsor_change_program,
            },
        ) => {
            !sponsor_change_program.is_empty()
                && sponsored_request_shape_recomputes(
                    transaction,
                    *sponsor_input_index,
                    *fee_output_index,
                )
                && transaction
                    .outputs()
                    .iter()
                    .all(|output| output.program() != sponsor_change_program)
        }
        _ => false,
    }
}

type BoundRequest<'a> = (&'a LiveRequestFact, &'a LiveTargetResponse, &'a [u8]);

struct PairClaimSide<'a> {
    run_id: &'a str,
    request_id: &'a str,
    identity: Txid,
    request: BoundRequest<'a>,
}

fn validate_pair_claim(
    row: &'static str,
    explicit: &PairClaimSide<'_>,
    private: &PairClaimSide<'_>,
    relation: &str,
) -> Result<(), LiveSafetyReportRefusal> {
    let (
        LiveRequestFact::Paired {
            member: LivePairMember::Explicit,
            projection: explicit_projection,
        },
        explicit_response,
        explicit_bytes,
    ) = explicit.request
    else {
        return Err(LiveSafetyReportRefusal::RequestRoleDiffers(
            explicit.run_id.to_owned(),
            explicit.request_id.to_owned(),
        ));
    };
    let (
        LiveRequestFact::Paired {
            member: LivePairMember::Private,
            projection: private_projection,
        },
        private_response,
        private_bytes,
    ) = private.request
    else {
        return Err(LiveSafetyReportRefusal::RequestRoleDiffers(
            private.run_id.to_owned(),
            private.request_id.to_owned(),
        ));
    };
    let explicit_expected = LiveTargetResponse::Accepted {
        identity: explicit.identity,
    };
    if explicit_response != &explicit_expected {
        return Err(LiveSafetyReportRefusal::RunResponseDiffers(
            explicit.run_id.to_owned(),
        ));
    }
    let private_expected = LiveTargetResponse::Accepted {
        identity: private.identity,
    };
    if private_response != &private_expected {
        return Err(LiveSafetyReportRefusal::RunResponseDiffers(
            private.run_id.to_owned(),
        ));
    }
    if relation != VALIDATED_PAIR_RELATION
        || !pair_relation_recomputes(
            explicit_bytes,
            explicit_projection,
            private_bytes,
            private_projection,
        )
    {
        return Err(LiveSafetyReportRefusal::PairedRelationNotRecomputed(row));
    }
    Ok(())
}

fn validate_bound_observations(
    observations: &[LiveReportObservation],
    runs: &[LiveRunBinding],
) -> Result<usize, LiveSafetyReportRefusal> {
    validate_run_bindings(runs)?;
    let mut validation = BoundObservationValidation::default();
    for observation in observations {
        match observation {
            LiveReportObservation::NativeAcceptance { .. } => {
                validation.acceptance(observation, runs)?;
            }
            LiveReportObservation::CompositeTwoAcceptance { .. } => {
                validation.composite_two_acceptance(observation, runs)?;
            }
            LiveReportObservation::NativeRefusalWithSupport { .. } => {
                validation.refusal_with_support(observation, runs)?;
            }
            LiveReportObservation::PairedRelation { .. } => {
                validation.pair(observation, runs)?;
            }
            LiveReportObservation::MultiRowSemantic { .. } => {
                validation.multi_row_semantic(observation, runs)?;
            }
            LiveReportObservation::RecordedObservationUnbound { .. }
            | LiveReportObservation::Determinism { .. }
            | LiveReportObservation::FirstPartyFact { .. } => {}
        }
    }
    let request_count = runs.iter().map(|run| run.requests.len()).sum::<usize>();
    if validation.used_requests.len() != request_count {
        return Err(LiveSafetyReportRefusal::RunObservationCensusDiffers);
    }
    Ok(validation.compared)
}

fn validate_report_envelope(
    report: &LiveTransferSafetyReport,
    evidence: &ValidatedCorpusEvidence,
    target: &TargetProjection,
    progress: &mut RecomputationProgress,
) -> Result<(), LiveSafetyReportRefusal> {
    let plan = evidence.plan();
    if report.schema != LIVE_SAFETY_REPORT_SCHEMA {
        return Err(LiveSafetyReportRefusal::UnsupportedSchema(report.schema));
    }
    progress.mark(RecomputedItem::Schema);
    if report.role != LiveSafetyReportRole::LiveTransferSafety {
        return Err(LiveSafetyReportRefusal::WrongRole(report.role));
    }
    progress.mark(RecomputedItem::Role);
    if &report.target != target {
        return Err(LiveSafetyReportRefusal::TargetDiffers);
    }

    let representations: BTreeSet<_> = plan
        .operation_plan()
        .representations()
        .map(compiler::live_transfer_plan::LiveTransferRepresentationProjection::plan)
        .collect();
    if report.representations != representations {
        return Err(LiveSafetyReportRefusal::RepresentationCensusDiffers);
    }

    compare_run_bindings(&report.runs, evidence.runs())?;
    if !report.runs.is_empty() {
        progress.mark(RecomputedItem::TargetDeploymentBinding);
        progress.mark(RecomputedItem::RequestResponseCensus);
        progress.mark(RecomputedItem::ExecutorProvenance);
    }
    Ok(())
}

fn validate_report_observation_ledger(
    report: &LiveTransferSafetyReport,
    evidence: &ValidatedCorpusEvidence,
    progress: &mut RecomputationProgress,
) -> Result<(), LiveSafetyReportRefusal> {
    let plan = evidence.plan();
    let expected_observations = observations_from_overlay(evidence)?;
    let expected_requirements = report_layer_requirements_from_plan(plan);
    if report.observations != expected_observations
        || report.report_layer_requirements != expected_requirements
    {
        return Err(LiveSafetyReportRefusal::ObservationsDiffer);
    }
    let observed = ObservationCensus::from_observations(&report.observations);
    if !observed.cross_foots(plan.census())
        || report.report_layer_requirements.len() != plan.census().report_layer_required()
    {
        return Err(LiveSafetyReportRefusal::ObservationCensusDiffers);
    }

    let bound_compared = validate_bound_observations(&report.observations, &report.runs)?;
    progress.mark(RecomputedItem::CaseCensus);
    progress.mark(RecomputedItem::RelationCensus);
    progress.mark(RecomputedItem::MutationLinks);
    if bound_compared > 0 {
        progress.mark(RecomputedItem::ResponseShape);
        progress.mark(RecomputedItem::TargetVerdictComparison);
    }
    if observed.paired_relation > 0 {
        progress.mark(RecomputedItem::SemanticProjection);
    }
    Ok(())
}

fn validate_report_summary(
    report: &LiveTransferSafetyReport,
    recomputed: LiveEvidenceCensus,
    progress: &mut RecomputationProgress,
) -> Result<(), LiveSafetyReportRefusal> {
    if report.census != recomputed {
        return Err(LiveSafetyReportRefusal::CensusDiffers(Box::new((
            report.census,
            recomputed,
        ))));
    }

    let completeness = completeness_of(recomputed);
    if report.completeness != completeness {
        return Err(LiveSafetyReportRefusal::CompletenessDiffers {
            reported: report.completeness,
            recomputed: completeness,
        });
    }

    if report.lifecycle != LiveLifecycleStatus::candidate() {
        return Err(LiveSafetyReportRefusal::LifecycleDiffers);
    }
    progress.mark(RecomputedItem::Summary);
    Ok(())
}

fn validated_outstanding_rows(
    plan: &LiveTransferEvidencePlan,
) -> Vec<(&'static str, LiveRowStanding)> {
    plan.rows()
        .iter()
        .filter(|row| {
            !row.standing().is_answered()
                && !matches!(
                    row.standing(),
                    LiveRowStanding::ReportLayerRequired(_)
                        | LiveRowStanding::OperationVocabularyClosed
                        | LiveRowStanding::Experimental
                )
        })
        .map(|row| (row.row().name(), row.standing().clone()))
        .collect()
}

fn validated_section_scoreboard(
    plan: &LiveTransferEvidencePlan,
) -> BTreeMap<LiveSafetySection, (usize, usize, usize)> {
    let mut board: BTreeMap<LiveSafetySection, (usize, usize, usize)> = LiveSafetySection::ALL
        .iter()
        .map(|section| (*section, (0, 0, 0)))
        .collect();
    for row in plan.rows() {
        let entry = board.entry(row.row().section()).or_insert((0, 0, 0));
        entry.0 += 1;
        if row.standing().is_answered()
            || matches!(row.standing(), LiveRowStanding::ReportLayerRequired(_))
        {
            entry.1 += 1;
        }
        if row.standing().is_infrastructure_error() {
            entry.2 += 1;
        }
    }
    board
}

struct CanonicalReportView<'a> {
    report: &'a LiveTransferSafetyReport,
    report_layer_requirements: &'a [LiveReportLayerRequirement],
    census: LiveEvidenceCensus,
    recomputed_items: &'a BTreeSet<RecomputedItem>,
    blockers: &'a BTreeMap<LiveInfrastructureBlocker, usize>,
    outstanding: &'a [(&'static str, LiveRowStanding)],
}

/// Validate one safety report against the plan it claims to be about.
///
/// §13.5's possible items, each recorded only after its comparison ran.
/// Run-only items stay absent while the corpus has no transcript-grade
/// run binding.
///
/// Schema 5 was never persisted, so RETIRING it strands no reader. This
/// reader accepts only [`LIVE_SAFETY_REPORT_SCHEMA`] and refuses every
/// other value.
///
/// # Errors
///
/// [`LiveSafetyReportRefusal`], naming the first disagreement found in
/// the order the checks are written.
pub fn validate_live_safety_report(
    report: LiveTransferSafetyReport,
    evidence: &ValidatedCorpusEvidence,
    target: &TargetProjection,
) -> Result<ValidatedLiveTransferSafetyReport, LiveSafetyReportRefusal> {
    let plan = evidence.plan();
    let mut progress = RecomputationProgress::default();
    validate_report_envelope(&report, evidence, target, &mut progress)?;
    validate_report_observation_ledger(&report, evidence, &mut progress)?;
    let recomputed = plan.census();
    validate_report_summary(&report, recomputed, &mut progress)?;

    let validated_census = recomputed
        .with_validated_report_layer_observations(report.report_layer_requirements.len())
        .ok_or(LiveSafetyReportRefusal::ObservationCensusDiffers)?;
    let blockers = blocker_census(plan);
    let outstanding = validated_outstanding_rows(plan);
    let scoreboard = validated_section_scoreboard(plan);
    // The candidate bytes must already contain the final recomputation
    // witness, but `completed_with` does not mark progress. The real
    // witness is marked only after these exact bytes pass disclosure
    // validation, and those same bytes are then sealed into the wrapper.
    let final_items = progress.completed_with(RecomputedItem::DisclosureComparison);
    let canonical_bytes = render_canonical_report(&CanonicalReportView {
        report: &report,
        report_layer_requirements: &report.report_layer_requirements,
        census: validated_census,
        recomputed_items: &final_items,
        blockers: &blockers,
        outstanding: &outstanding,
    });
    validate_report_disclosures(&canonical_bytes, &report.report_layer_requirements)?;
    progress.mark(RecomputedItem::DisclosureComparison);
    let report_layer_observations = report
        .report_layer_requirements
        .iter()
        .map(|requirement| ValidatedReportLayerObservation {
            row: requirement.row,
            requirement: requirement.requirement,
            schema: report.schema,
        })
        .collect();

    Ok(ValidatedLiveTransferSafetyReport {
        report,
        report_layer_observations,
        census: validated_census,
        recomputed_items: progress.finish(),
        blockers,
        outstanding,
        scoreboard,
        canonical_bytes,
    })
}

/// Render the canonical bytes of one validated safety report.
///
/// Takes a validated report and nothing else. The ten volatile fields are
/// not filtered here — they are not reachable from this function's one
/// argument, which is the difference between a rule and a habit.
///
/// # Panics
///
/// Never: the sink is a `String`, whose writes cannot fail.
#[must_use]
pub fn render_live_safety_report(validated: &ValidatedLiveTransferSafetyReport) -> String {
    validated.canonical_bytes.clone()
}

fn render_canonical_report(validated: &CanonicalReportView<'_>) -> String {
    let report = validated.report;
    let mut text = String::new();
    let observation_census = ObservationCensus::from_observations(&report.observations);
    render_report_header(&mut text, report, observation_census);
    render_evidence_census(&mut text, validated.census);
    render_observation_census(
        &mut text,
        observation_census,
        validated.report_layer_requirements.len(),
    );
    render_observations(&mut text, &report.observations);
    render_validated_report_layer_observations(
        &mut text,
        validated.report_layer_requirements,
        report.schema,
    );
    render_validation_summary(&mut text, validated);
    text
}

fn render_report_header(
    text: &mut String,
    report: &LiveTransferSafetyReport,
    observation_census: ObservationCensus,
) {
    let _ = writeln!(text, "schema {}", report.schema);
    let _ = writeln!(text, "role {}", report.role.name());
    let _ = writeln!(text, "operation transfer-live-receipts");
    let _ = writeln!(text, "candidate true");
    let _ = writeln!(text, "target_contract {:?}", report.target.version());
    let _ = writeln!(
        text,
        "execution_domain {:?}",
        report.target.execution_domain()
    );
    for representation in &report.representations {
        let _ = writeln!(text, "representation {representation:?}");
    }
    let _ = writeln!(
        text,
        "target_evidence {}",
        target_evidence_name(observation_census)
    );
    let _ = writeln!(text, "run_bindings {}", report.runs.len());
    render_run_bindings(text, &report.runs);
}

fn render_evidence_census(text: &mut String, census: LiveEvidenceCensus) {
    let _ = writeln!(text, "rows {}", census.rows());
    let _ = writeln!(
        text,
        "first_party_discharged {}",
        census.first_party_discharged()
    );
    let _ = writeln!(
        text,
        "first_party_undischarged {}",
        census.first_party_undischarged()
    );
    let _ = writeln!(text, "native_run_required {}", census.native_run_required());
    let _ = writeln!(
        text,
        "recorded_observation_unbound {}",
        census.recorded_observation_unbound()
    );
    let _ = writeln!(text, "native_run_observed {}", census.native_run_observed());
    let _ = writeln!(
        text,
        "native_refusal_observed {}",
        census.native_refusal_observed()
    );
    let _ = writeln!(
        text,
        "native_refusal_at_unexpected_boundary {}",
        census.native_refusal_at_unexpected_boundary()
    );
    let _ = writeln!(
        text,
        "determinism_observed {}",
        census.determinism_observed()
    );
    let _ = writeln!(
        text,
        "paired_relation_observed {}",
        census.paired_relation_observed()
    );
    let _ = writeln!(
        text,
        "first_party_fact_observed {}",
        census.first_party_fact_observed()
    );
    let _ = writeln!(
        text,
        "infrastructure_blocked {}",
        census.infrastructure_blocked()
    );
    let _ = writeln!(
        text,
        "report_layer_required {}",
        census.report_layer_required()
    );
    let _ = writeln!(
        text,
        "report_layer_observed {}",
        census.report_layer_observed()
    );
    let _ = writeln!(
        text,
        "operation_vocabulary_closed {}",
        census.vocabulary_closed()
    );
    let _ = writeln!(text, "experimental {}", census.experimental());
}

fn render_observation_census(
    text: &mut String,
    census: ObservationCensus,
    report_layer_observations: usize,
) {
    let _ = writeln!(
        text,
        "observations {}",
        census.total() + report_layer_observations
    );
    let _ = writeln!(text, "accepted {}", census.accepted);
    let _ = writeln!(text, "refused {}", census.refused);
    let _ = writeln!(text, "determinism {}", census.determinism);
    let _ = writeln!(text, "paired_relation {}", census.paired_relation);
    let _ = writeln!(text, "first_party_fact {}", census.first_party_fact);
    let _ = writeln!(
        text,
        "report_layer_observations {report_layer_observations}"
    );
    let _ = writeln!(
        text,
        "recorded_unbound_native_acceptance {}",
        census.unbound_acceptance
    );
    let _ = writeln!(
        text,
        "recorded_unbound_native_refusal {}",
        census.unbound_refusal
    );
    let _ = writeln!(
        text,
        "recorded_unbound_paired_relation {}",
        census.unbound_paired_relation
    );
}

fn render_validation_summary(text: &mut String, validated: &CanonicalReportView<'_>) {
    for (blocker, rows) in validated.blockers {
        let _ = writeln!(text, "blocker {blocker:?} {rows}");
    }
    for (row, standing) in validated.outstanding {
        let _ = writeln!(text, "outstanding {row} {}", standing_name(standing));
    }
    for item in validated.recomputed_items {
        let _ = writeln!(text, "recomputed {item:?}");
    }

    for exit in validated.report.lifecycle.implemented() {
        let _ = writeln!(text, "lifecycle_implemented {exit}");
    }
    for exit in validated.report.lifecycle.outstanding() {
        let _ = writeln!(text, "lifecycle_outstanding {exit}");
    }
    let _ = writeln!(
        text,
        "release_complete {}",
        validated.report.lifecycle.release_complete()
    );
    let _ = writeln!(
        text,
        "completeness {}",
        validated.report.completeness.name()
    );
}

const fn target_evidence_name(census: ObservationCensus) -> &'static str {
    let bound = census.accepted + census.refused + census.paired_relation;
    let unbound = census.recorded_unbound;
    if bound == 0 && unbound == 0 {
        "none-recorded"
    } else if bound == 0 {
        "recorded-unbound"
    } else if unbound == 0 {
        "bound"
    } else {
        "mixed-bound-and-unbound"
    }
}

fn hex_bytes(bytes: &[u8]) -> String {
    let mut rendered = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(rendered, "{byte:02x}");
    }
    rendered
}

fn render_request_fact(text: &mut String, request_id: &str, fact: &LiveRequestFact) {
    match fact {
        LiveRequestFact::Acceptance => {
            let _ = writeln!(text, "run_request_fact {request_id} acceptance");
        }
        LiveRequestFact::Control => {
            let _ = writeln!(text, "run_request_fact {request_id} control");
        }
        LiveRequestFact::Refusal {
            mutant,
            control_request_id,
            locator,
        } => {
            let _ = writeln!(
                text,
                concat!(
                    "run_request_fact {} refusal mutant {} ",
                    "control {} locator {:?}",
                ),
                request_id,
                mutant.row(),
                control_request_id,
                locator,
            );
        }
        LiveRequestFact::Paired { member, .. } => {
            // Projection inputs include semantic values the private member
            // withholds. The run ID commits to them and validation consumes
            // them, while canonical report bytes publish only the member and
            // the independently recomputed relation.
            let _ = writeln!(
                text,
                "run_request_fact {request_id} paired {} projection-inputs committed-by-run-id",
                member.name()
            );
        }
    }
}

fn render_run_bindings(text: &mut String, runs: &[LiveRunBinding]) {
    for run in runs {
        let _ = writeln!(text, "run {}", run.run_id);
        let _ = writeln!(text, "run_archive {}", hex_bytes(&run.archive_bytes));
        let _ = writeln!(text, "run_protocol_revision {}", run.revision.code());
        for (name, fact) in &run.digest_facts {
            let _ = writeln!(
                text,
                "run_digest {name} {} {}",
                fact.algorithm.name(),
                hex_bytes(&fact.value)
            );
        }
        let _ = writeln!(
            text,
            "run_deployment_environment {:?}",
            run.deployment.environment
        );
        let _ = writeln!(
            text,
            "run_deployment_network {}",
            hex_bytes(&run.deployment.network_id)
        );
        let _ = writeln!(
            text,
            "run_deployment_genesis {}",
            hex_bytes(&run.deployment.genesis_id)
        );
        let _ = writeln!(
            text,
            "run_deployment_target {:?}",
            run.deployment.target_contract
        );
        let _ = writeln!(text, "run_executor_adapter {:?}", run.executor.adapter_name);
        let _ = writeln!(
            text,
            "run_executor_adapter_version {:?}",
            run.executor.adapter_version
        );
        let _ = writeln!(text, "run_executor_node {:?}", run.executor.node_name);
        let _ = writeln!(
            text,
            "run_executor_node_version {:?}",
            run.executor.node_version
        );
        let _ = writeln!(
            text,
            "run_executor_source_tip {:?}",
            run.executor.executed_source_tip
        );
        for (request_id, bytes) in &run.requests {
            let _ = writeln!(text, "run_request {request_id} {}", hex_bytes(bytes));
        }
        for (request_id, fact) in &run.request_facts {
            render_request_fact(text, request_id, fact);
        }
        for (request_id, response) in &run.responses {
            match response {
                LiveTargetResponse::Accepted { identity } => {
                    let _ = writeln!(
                        text,
                        "run_response {request_id} accepted {}",
                        identity.to_target_display()
                    );
                }
                LiveTargetResponse::Refused {
                    observed_layer,
                    detail,
                    control_identity,
                } => {
                    let _ = writeln!(
                        text,
                        "run_response {request_id} refused {observed_layer:?} control {} detail {detail:?}",
                        control_identity.to_target_display()
                    );
                }
            }
        }
    }
}

fn render_observations(text: &mut String, observations: &[LiveReportObservation]) {
    for observation in observations {
        render_observation(text, observation);
    }
}

fn render_composite_two_acceptance(
    text: &mut String,
    row: &str,
    acceptance: &LiveCompositeTwoAcceptance,
) {
    let [first, second] = acceptance.members();
    let _ = writeln!(
        text,
        "observation {row} composite-two-acceptance first-ceremony {:?} first-run {} first-request {} first-identity {} first-commitment-output {} second-ceremony {:?} second-run {} second-request {} second-identity {} second-commitment-output {}",
        first.ceremony,
        first.run_id,
        first.request_id,
        first.identity.to_target_display(),
        first.consumed_commitment_output_index,
        second.ceremony,
        second.run_id,
        second.request_id,
        second.identity.to_target_display(),
        second.consumed_commitment_output_index,
    );
}

fn render_multi_row_semantic(text: &mut String, witness: &LiveMultiRowSemanticWitness) {
    let rows = witness.rows.iter().copied().collect::<Vec<_>>().join(",");
    let _ = writeln!(
        text,
        "observation multi-row-semantic rows {rows:?} run {} request {} identity {} predicates {:?}",
        witness.run_id,
        witness.request_id,
        witness.identity.to_target_display(),
        witness.predicates,
    );
}

fn render_observation(text: &mut String, observation: &LiveReportObservation) {
    match observation {
        LiveReportObservation::NativeAcceptance {
            row,
            run_id,
            request_id,
            identity,
        } => {
            let _ = writeln!(
                text,
                "observation {row} accepted run {run_id} request {request_id} identity {}",
                identity.to_target_display()
            );
        }
        LiveReportObservation::CompositeTwoAcceptance { row, acceptance } => {
            render_composite_two_acceptance(text, row, acceptance);
        }
        LiveReportObservation::NativeRefusalWithSupport {
            row,
            run_id,
            request_id,
            declared_boundary,
            observed_layer,
            detail,
            support,
        } => {
            let _ = writeln!(
                text,
                "observation {row} refused run {run_id} request {request_id} declared {declared_boundary:?} observed {observed_layer:?} support-run {} support-request {} detail {detail:?}",
                support.run_id, support.request_id,
            );
        }
        LiveReportObservation::PairedRelation {
            row,
            explicit_run_id,
            explicit_request_id,
            explicit_identity,
            private_run_id,
            private_request_id,
            private_identity,
            relation,
        } => {
            let _ = writeln!(
                text,
                "observation {row} paired-relation explicit-run {explicit_run_id} explicit-request {explicit_request_id} explicit-identity {} private-run {private_run_id} private-request {private_request_id} private-identity {} relation {relation:?}",
                explicit_identity.to_target_display(),
                private_identity.to_target_display()
            );
        }
        LiveReportObservation::MultiRowSemantic { witness } => {
            render_multi_row_semantic(text, witness);
        }
        LiveReportObservation::RecordedObservationUnbound { row, observation } => {
            render_recorded_observation_unbound(text, row, observation);
        }
        LiveReportObservation::Determinism {
            row,
            recomputed,
            observed_by,
        } => {
            let _ = writeln!(
                text,
                "observation {row} determinism recomputed {recomputed:?} observed-by {observed_by:?}"
            );
        }
        LiveReportObservation::FirstPartyFact {
            row,
            fact,
            observed_by,
        } => {
            let _ = writeln!(
                text,
                "observation {row} first-party-fact fact {fact:?} observed-by {observed_by:?}"
            );
        }
    }
}

fn render_validated_report_layer_observations(
    text: &mut String,
    requirements: &[LiveReportLayerRequirement],
    schema: u32,
) {
    for requirement in requirements {
        let _ = writeln!(
            text,
            "observation {} report-layer requirement {} validated-by canonical-bytes schema {}",
            requirement.row,
            requirement.requirement.name(),
            schema
        );
    }
}

fn render_recorded_observation_unbound(
    text: &mut String,
    row: &str,
    observation: &LiveRecordedObservation,
) {
    match observation {
        LiveRecordedObservation::NativeAcceptance { identity } => {
            let _ = writeln!(
                text,
                "observation {row} recorded-observation-unbound native-acceptance identity {}",
                identity.to_target_display()
            );
        }
        LiveRecordedObservation::NativeRefusal {
            declared_boundary,
            observed_layer,
            control_identity,
            detail,
        } => {
            let _ = writeln!(
                text,
                "observation {row} recorded-observation-unbound native-refusal declared {declared_boundary:?} observed {observed_layer:?} control {} detail {detail:?}",
                control_identity.to_target_display()
            );
        }
        LiveRecordedObservation::PairedRelation {
            explicit_identity,
            private_identity,
            relation,
        } => {
            let _ = writeln!(
                text,
                "observation {row} recorded-observation-unbound paired-relation explicit-identity {} private-identity {} relation {relation:?}",
                explicit_identity.to_target_display(),
                private_identity.to_target_display()
            );
        }
    }
}

/// One standing's wire spelling.
///
/// Explicit rather than derived from the type's own `Debug`, because a
/// standing carrying a requirement identity would otherwise print the
/// identity into canonical bytes that are meant to summarize.
const fn standing_name(standing: &LiveRowStanding) -> &'static str {
    match standing {
        LiveRowStanding::FirstPartyDischarged { .. } => "first-party-discharged",
        LiveRowStanding::FirstPartyUndischarged(_) => "first-party-undischarged",
        LiveRowStanding::NativeRunRequired(_) => "native-run-required",
        LiveRowStanding::RecordedObservationUnbound(_) => "recorded-observation-unbound",
        // The identity is deliberately not printed here. It is the
        // standing's own payload and belongs in the evidence plan, not
        // in bytes whose job is to summarize.
        LiveRowStanding::NativeRunObserved { .. } => "native-run-observed",
        // Its payload is withheld for the same reason and for one more:
        // the refusal detail is the target's own sentence, and a summary
        // that quoted a verdict would be carrying evidence in bytes
        // whose job is to count.
        LiveRowStanding::NativeRefusalObserved { .. } => "native-refusal-observed",
        // Withheld for the same two reasons as the standing above, and
        // spelled APART from it because the two are different facts.
        // This one says the target refused somewhere other than where
        // the row declared, which is a finding rather than an answer —
        // and it is spelled apart from `native-run-required` too,
        // because a contradicted row is not a row still waiting.
        LiveRowStanding::NativeRefusalAtUnexpectedBoundary { .. } => {
            "native-refusal-at-unexpected-boundary"
        }
        // Withheld for the first reason and not the second: there is no
        // target sentence here to carry, and what the payload names is
        // the first-party test that recomputed the fixture, which
        // belongs in the evidence plan beside the row rather than in
        // bytes whose job is to count.
        LiveRowStanding::DeterminismObserved { .. } => "determinism-observed",
        // TWO identities withheld rather than one, and for the first
        // reason twice over: they are the standing's own payload and
        // belong in the evidence plan beside the row. A summary that
        // printed a pair of identities would be carrying the evidence in
        // bytes whose job is to count it.
        LiveRowStanding::PairedRelationObserved { .. } => "paired-relation-observed",
        // Withheld for the same two reasons: no target sentence exists
        // to carry, and the payload names a first-party site, which
        // belongs beside the row in the evidence plan rather than in
        // bytes whose job is to count.
        LiveRowStanding::FirstPartyFactObserved { .. } => "first-party-fact-observed",
        LiveRowStanding::InfrastructureBlocked(_) => "infrastructure-blocked",
        LiveRowStanding::ReportLayerRequired(_) => "report-layer-required",
        LiveRowStanding::OperationVocabularyClosed => "operation-vocabulary-closed",
        LiveRowStanding::Experimental => "experimental",
    }
}

/// Every key the canonical rendering is forbidden to emit.
///
/// §1.9's list, as keys rather than as substrings. The distinction is
/// load-bearing: §15.5 has a row *named* `wrong-private-blinding-balance`
/// and §15.6 has one named `report-publishes-sponsor-amount`, and a
/// search for the word "blinding" or "sponsor amount" anywhere in the
/// bytes would flag the matrix's own row names. What §1.9 forbids is a
/// sponsor amount being *emitted*, which is a statement about what a
/// field carries.
const SPONSOR_AMOUNT_KEYS: &[&str] = &[
    "sponsor_amount",
    "sponsor_amounts",
    "sponsor_value",
    "sponsor_values",
    "sponsor_change_amount",
    "sponsor_total",
];

const SPONSOR_OPENING_KEYS: &[&str] = &[
    "sponsor_opening",
    "sponsor_openings",
    "sponsor_blinding",
    "blinding_factor",
    "value_blinding",
    "fixture_opening",
];

/// The §15.6 report rows, answered against the canonical bytes.
///
/// Two rows ask whether the report publishes a sponsor amount or a
/// sponsor opening, and this is what answers them: the canonical
/// serialization is rendered and every line's *key* is compared against
/// this module's forbidden-key list. §1.9 forbids an individual sponsor
/// amount from being emitted in a canonical report at all, so the answer
/// must be that no line carries one.
const fn forbidden_keys(requirement: LiveReportRequirement) -> &'static [&'static str] {
    match requirement {
        LiveReportRequirement::SponsorAmountAbsent => SPONSOR_AMOUNT_KEYS,
        LiveReportRequirement::SponsorOpeningAbsent => SPONSOR_OPENING_KEYS,
    }
}

fn validate_report_disclosures(
    canonical_bytes: &str,
    requirements: &[LiveReportLayerRequirement],
) -> Result<(), LiveSafetyReportRefusal> {
    for requirement in requirements {
        for key in canonical_bytes
            .lines()
            .filter_map(|line| line.split_whitespace().next())
        {
            if forbidden_keys(requirement.requirement).contains(&key) {
                return Err(LiveSafetyReportRefusal::ReportDisclosurePresent {
                    row: requirement.row,
                    key: key.to_owned(),
                });
            }
        }
    }
    Ok(())
}

/// The per-section scoreboard a reader of a report wants first.
#[must_use]
pub fn section_scoreboard(
    validated: &ValidatedLiveTransferSafetyReport,
) -> BTreeMap<LiveSafetySection, (usize, usize, usize)> {
    validated.scoreboard.clone()
}

#[cfg(test)]
mod tests {
    use super::{
        FixtureDigestAlgorithm, LIVE_SAFETY_REPORT_SCHEMA, LiveCompositeAcceptanceMember,
        LiveCompositeTwoAcceptance, LiveMultiRowSemanticWitness, LiveMutantKind,
        LiveMutationLocator, LivePairMember, LivePairProjectionInput, LiveRecordedObservation,
        LiveReportObservation, LiveRequestFact, LiveRowSemanticPredicate, LiveRunBinding,
        LiveSafetyCompleteness, LiveSafetyDiagnostics, LiveSafetyReportRefusal,
        LiveSafetyReportRole, LiveSupportLink, LiveTargetResponse, LiveWitnessPathRole,
        NativeProtocolRevision, ObservationLinkClass, RecomputedItem, VALIDATED_PAIR_RELATION,
        VolatileField, assemble_live_safety_report, compare_run_bindings, insert_observation_link,
        observations_from_overlay, recomputed_txid, render_live_safety_report, section_scoreboard,
        target_evidence_name, validate_bound_observations, validate_live_safety_report,
        validate_report_disclosures,
    };
    use crate::live_evidence::derive_live_evidence_plan;
    use crate::live_plan::reviewed_target;
    use crate::live_safety::{LiveReportRequirement, LiveSafetyPolarity, LiveSafetySection};
    use target_elements::TargetProjection;

    fn projection() -> TargetProjection {
        reviewed_target()
            .expect("the reviewed contract validates")
            .projection()
    }

    fn parsed_identity(value: &str) -> transaction::Txid {
        transaction::Txid::from_target_display(value).expect("the test identity parses")
    }

    fn synthetic_archive(revision: u32, algorithm: &str, source_tip: &str) -> Vec<u8> {
        let ceremony = if revision == 7 {
            format!("ceremony_id {source_tip}\n")
        } else {
            String::new()
        };
        format!(
            "{ceremony}protocol_revision {revision}\n\
             deployment_environment development\n\
             deployment_network {}\n\
             deployment_genesis {}\n\
             deployment_target elements-tapscript-v1\n\
             executor_adapter native-adapter\n\
             executor_adapter_version 1\n\
             executor_node elementsd\n\
             executor_node_version 28.99\n\
             executor_source_tip {source_tip}\n\
             digest_count 1\n\
             digest fixture {algorithm} {}\n",
            "11".repeat(32),
            "22".repeat(32),
            "33".repeat(32),
        )
        .into_bytes()
    }

    fn synthetic_transaction(seed: u8, amount: u64, witness_byte: u8) -> Vec<u8> {
        use transaction::{
            AssetField, AssetId, InputWitness, NonceField, Outpoint, TargetInput, TargetOutput,
            TargetTransaction, Txid, ValueField,
        };

        TargetTransaction::new(
            3,
            vec![TargetInput::new(
                Outpoint::new(Txid::from_internal([seed; 32]), 0)
                    .expect("the synthetic outpoint is admissible"),
                u32::MAX,
            )],
            vec![TargetOutput::new(
                AssetField::Explicit(AssetId::from_internal([0xa1; 32])),
                ValueField::Explicit(amount),
                NonceField::Null,
                vec![0x51],
            )],
            0,
            vec![InputWitness::new(vec![vec![witness_byte]])],
        )
        .expect("the synthetic transaction is well formed")
        .encode()
    }

    fn synthetic_committed_transaction(seed: u8, prefix: u8) -> Vec<u8> {
        use transaction::bytes::OutputWitness;
        use transaction::{
            AssetField, AssetId, InputWitness, NonceField, Outpoint, TargetInput, TargetOutput,
            TargetTransaction, Txid, ValueField,
        };

        let mut commitment = [0x44; 33];
        commitment[0] = prefix;
        TargetTransaction::with_output_witnesses(
            3,
            vec![TargetInput::new(
                Outpoint::new(Txid::from_internal([seed; 32]), 0)
                    .expect("the committed synthetic outpoint is admissible"),
                u32::MAX,
            )],
            vec![TargetOutput::new(
                AssetField::Explicit(AssetId::from_internal([0xa1; 32])),
                ValueField::Commitment(commitment),
                NonceField::Null,
                vec![0x51],
            )],
            0,
            vec![InputWitness::new(vec![vec![0x31]])],
            vec![OutputWitness::range_proof_only(vec![0xaa])],
        )
        .expect("the committed synthetic transaction is well formed")
        .encode()
    }

    fn synthetic_acceptance_run(
        ceremony: &str,
        request_id: &str,
        seed: u8,
        prefix: u8,
    ) -> LiveRunBinding {
        let request = synthetic_committed_transaction(seed, prefix);
        let identity = recomputed_txid(&request).expect("the acceptance identity recomputes");
        LiveRunBinding::from_archive(
            synthetic_archive(7, "forward-v2", ceremony),
            std::collections::BTreeMap::from([(request_id.to_owned(), request)]),
            std::collections::BTreeMap::from([(
                request_id.to_owned(),
                LiveRequestFact::Acceptance,
            )]),
            std::collections::BTreeMap::from([(
                request_id.to_owned(),
                LiveTargetResponse::Accepted { identity },
            )]),
        )
        .expect("the synthetic acceptance run parses")
    }

    fn synthetic_control_acceptance_run(request_id: &str) -> LiveRunBinding {
        let source = synthetic_acceptance_run("conservation-negatives", request_id, 0xb3, 0x08);
        LiveRunBinding::from_archive(
            source.archive_bytes,
            source.requests,
            std::collections::BTreeMap::from([(request_id.to_owned(), LiveRequestFact::Control)]),
            source.responses,
        )
        .expect("the synthetic control acceptance run parses")
    }

    fn composite_observation(
        first: &LiveRunBinding,
        first_ceremony: &str,
        first_request: &str,
        second: &LiveRunBinding,
        second_ceremony: &str,
        second_request: &str,
    ) -> LiveReportObservation {
        let LiveTargetResponse::Accepted {
            identity: first_identity,
        } = &first.responses[first_request]
        else {
            panic!("the first composite response is accepted");
        };
        let LiveTargetResponse::Accepted {
            identity: second_identity,
        } = &second.responses[second_request]
        else {
            panic!("the second composite response is accepted");
        };
        LiveReportObservation::CompositeTwoAcceptance {
            row: "both-commitment-parity-forms",
            acceptance: LiveCompositeTwoAcceptance::new(
                LiveCompositeAcceptanceMember::new(
                    first_ceremony.to_owned(),
                    first.run_id.clone(),
                    first_request.to_owned(),
                    *first_identity,
                    0,
                ),
                LiveCompositeAcceptanceMember::new(
                    second_ceremony.to_owned(),
                    second.run_id.clone(),
                    second_request.to_owned(),
                    *second_identity,
                    0,
                ),
            ),
        }
    }

    fn synthetic_witness_transaction(seed: u8, stacks: Vec<Vec<Vec<u8>>>) -> Vec<u8> {
        use transaction::{
            AssetField, AssetId, InputWitness, NonceField, Outpoint, TargetInput, TargetOutput,
            TargetTransaction, Txid, ValueField,
        };

        let inputs = (0..stacks.len())
            .map(|index| {
                let offset = u8::try_from(index).unwrap_or(u8::MAX);
                TargetInput::new(
                    Outpoint::new(Txid::from_internal([seed.wrapping_add(offset); 32]), 0)
                        .expect("the witness synthetic outpoint is admissible"),
                    u32::MAX,
                )
            })
            .collect();
        let witnesses = stacks.into_iter().map(InputWitness::new).collect();
        TargetTransaction::new(
            3,
            inputs,
            vec![TargetOutput::new(
                AssetField::Explicit(AssetId::from_internal([0xa1; 32])),
                ValueField::Explicit(50),
                NonceField::Null,
                vec![0x51],
            )],
            0,
            witnesses,
        )
        .expect("the witness synthetic transaction is well formed")
        .encode()
    }

    fn synthetic_control_block_with_version(version: u8) -> Vec<u8> {
        let mut block = vec![version];
        block.extend_from_slice(&[0x22; 32]);
        block
    }

    fn synthetic_control_block() -> Vec<u8> {
        synthetic_control_block_with_version(target_elements::opcode::LeafVersion::TAPSCRIPT.get())
    }

    fn synthetic_refusal_run(
        source_tip: &str,
        control: Vec<u8>,
        mutant: Vec<u8>,
        mutant_kind: LiveMutantKind,
        locator: LiveMutationLocator,
    ) -> LiveRunBinding {
        use target_elements_conformance::protocol::ObservedOutcomeLayer;

        let control_identity = recomputed_txid(&control).expect("the control identity recomputes");
        LiveRunBinding::from_archive(
            synthetic_archive(7, "forward-v2", source_tip),
            std::collections::BTreeMap::from([
                ("control".to_owned(), control),
                ("mutant".to_owned(), mutant),
            ]),
            std::collections::BTreeMap::from([
                ("control".to_owned(), LiveRequestFact::Control),
                (
                    "mutant".to_owned(),
                    LiveRequestFact::Refusal {
                        mutant: mutant_kind,
                        control_request_id: "control".to_owned(),
                        locator,
                    },
                ),
            ]),
            std::collections::BTreeMap::from([
                (
                    "control".to_owned(),
                    LiveTargetResponse::Accepted {
                        identity: control_identity,
                    },
                ),
                (
                    "mutant".to_owned(),
                    LiveTargetResponse::Refused {
                        observed_layer: ObservedOutcomeLayer::ScriptPathRejection,
                        detail: "synthetic refusal".to_owned(),
                        control_identity,
                    },
                ),
            ]),
        )
        .expect("the schema-six refusal run parses")
    }

    fn witness_path_shape_run() -> LiveRunBinding {
        let signature = vec![0x31; 64];
        let control = synthetic_witness_transaction(0x61, vec![vec![signature.clone()]]);
        let mutant = synthetic_witness_transaction(
            0x61,
            vec![vec![signature, vec![0x51], synthetic_control_block()]],
        );
        synthetic_refusal_run(
            "witness-path-tip",
            control,
            mutant,
            LiveMutantKind::KeyPathEscape,
            LiveMutationLocator::WitnessPathShape {
                input_index: 0,
                control_stack_items: 1,
                mutant_stack_items: 3,
                changed_positions: vec![1, 2],
                control_role: LiveWitnessPathRole::KeyPath,
                mutant_role: LiveWitnessPathRole::ScriptPath,
                witnessless_serialization_equal: true,
            },
        )
    }

    fn missing_sponsor_authorization_run() -> LiveRunBinding {
        let control =
            synthetic_witness_transaction(0x62, vec![vec![vec![0x31; 64], vec![0x32; 64]]]);
        let mutant = synthetic_witness_transaction(0x62, vec![vec![Vec::new(), Vec::new()]]);
        synthetic_refusal_run(
            "missing-sponsor-authorization-tip",
            control,
            mutant,
            LiveMutantKind::MissingSponsorAuthorization,
            LiveMutationLocator::WitnessItem {
                input_index: 0,
                item_index: 0,
            },
        )
    }

    fn committed_leaf_arrangement_run_with_block(block: Vec<u8>) -> LiveRunBinding {
        let coordinator = vec![0x51, 0x00];
        let member = vec![0x51, 0x01];
        let signature = vec![0x31; 64];
        let control = synthetic_witness_transaction(
            0x71,
            vec![
                vec![signature.clone(), coordinator.clone(), block.clone()],
                vec![signature.clone(), member.clone(), block.clone()],
            ],
        );
        let mutant = synthetic_witness_transaction(
            0x71,
            vec![
                vec![signature.clone(), coordinator.clone(), block.clone()],
                vec![signature, coordinator.clone(), block],
            ],
        );
        synthetic_refusal_run(
            "leaf-arrangement-tip",
            control,
            mutant,
            LiveMutantKind::TwoCoordinators,
            LiveMutationLocator::CommittedLeafArrangement {
                input_indices: vec![0, 1],
                control_coordinator_leaf_indices: vec![0],
                mutant_coordinator_leaf_indices: vec![0, 1],
                control_committed_leaf_programs: vec![coordinator.clone(), member],
                mutant_committed_leaf_programs: vec![coordinator.clone(), coordinator],
            },
        )
    }

    fn committed_leaf_arrangement_run() -> LiveRunBinding {
        committed_leaf_arrangement_run_with_block(synthetic_control_block())
    }

    fn repeated_support_run() -> LiveRunBinding {
        use target_elements_conformance::protocol::ObservedOutcomeLayer;

        let control = synthetic_transaction(0x81, 50, 0x31);
        let mutant_a = synthetic_transaction(0x81, 50, 0x32);
        let mutant_b = synthetic_transaction(0x81, 50, 0x33);
        let control_identity = recomputed_txid(&control).expect("the support identity recomputes");
        LiveRunBinding::from_archive(
            synthetic_archive(7, "forward-v2", "support-tip"),
            std::collections::BTreeMap::from([
                ("control".to_owned(), control),
                ("mutant-a".to_owned(), mutant_a),
                ("mutant-b".to_owned(), mutant_b),
            ]),
            std::collections::BTreeMap::from([
                ("control".to_owned(), LiveRequestFact::Control),
                (
                    "mutant-a".to_owned(),
                    LiveRequestFact::Refusal {
                        mutant: LiveMutantKind::MalformedSignature,
                        control_request_id: "control".to_owned(),
                        locator: LiveMutationLocator::WitnessItem {
                            input_index: 0,
                            item_index: 0,
                        },
                    },
                ),
                (
                    "mutant-b".to_owned(),
                    LiveRequestFact::Refusal {
                        mutant: LiveMutantKind::MalformedSignature,
                        control_request_id: "control".to_owned(),
                        locator: LiveMutationLocator::WitnessItem {
                            input_index: 0,
                            item_index: 0,
                        },
                    },
                ),
            ]),
            std::collections::BTreeMap::from([
                (
                    "control".to_owned(),
                    LiveTargetResponse::Accepted {
                        identity: control_identity,
                    },
                ),
                (
                    "mutant-a".to_owned(),
                    LiveTargetResponse::Refused {
                        observed_layer: ObservedOutcomeLayer::ScriptPathRejection,
                        detail: "first synthetic refusal".to_owned(),
                        control_identity,
                    },
                ),
                (
                    "mutant-b".to_owned(),
                    LiveTargetResponse::Refused {
                        observed_layer: ObservedOutcomeLayer::ScriptPathRejection,
                        detail: "second synthetic refusal".to_owned(),
                        control_identity,
                    },
                ),
            ]),
        )
        .expect("the repeated-support run parses")
    }

    fn support_observation(run: &LiveRunBinding, mutant_request_id: &str) -> LiveReportObservation {
        let LiveTargetResponse::Refused {
            observed_layer,
            detail,
            ..
        } = &run.responses[mutant_request_id]
        else {
            panic!("the support fixture mutant is refused");
        };
        LiveReportObservation::NativeRefusalWithSupport {
            row: "malformed-signature",
            run_id: run.run_id.clone(),
            request_id: mutant_request_id.to_owned(),
            declared_boundary: crate::matrix::EvidenceBoundary::ScriptPathRejection,
            observed_layer: *observed_layer,
            detail: detail.clone(),
            support: LiveSupportLink::new(
                run.run_id.clone(),
                "control".to_owned(),
                run.requests["control"].clone(),
                run.responses["control"].clone(),
            ),
        }
    }

    fn synthetic_multi_row_transaction() -> Vec<u8> {
        use transaction::{
            AssetField, AssetId, InputWitness, NonceField, Outpoint, TargetInput, TargetOutput,
            TargetTransaction, Txid, ValueField,
        };

        let receipt = TargetInput::new(
            Outpoint::new(Txid::from_internal([0x91; 32]), 0)
                .expect("the receipt outpoint is admissible"),
            u32::MAX,
        );
        let sponsor = TargetInput::new(
            Outpoint::new(Txid::from_internal([0x92; 32]), 0)
                .expect("the sponsor outpoint is admissible"),
            u32::MAX,
        );
        TargetTransaction::new(
            3,
            vec![receipt, sponsor],
            vec![
                TargetOutput::new(
                    AssetField::Explicit(AssetId::from_internal([0xa1; 32])),
                    ValueField::Explicit(45),
                    NonceField::Null,
                    vec![0x51],
                ),
                TargetOutput::new(
                    AssetField::Explicit(AssetId::from_internal([0xb1; 32])),
                    ValueField::Explicit(5),
                    NonceField::Null,
                    Vec::new(),
                ),
            ],
            0,
            vec![
                InputWitness::new(vec![vec![0x31; 64], vec![0x51], synthetic_control_block()]),
                InputWitness::new(vec![vec![0x32; 72], vec![0x33; 33]]),
            ],
        )
        .expect("the multi-row transaction is well formed")
        .encode()
    }

    fn synthetic_multi_row_run() -> LiveRunBinding {
        let request = synthetic_multi_row_transaction();
        let identity = recomputed_txid(&request).expect("the multi-row identity recomputes");
        LiveRunBinding::from_archive(
            synthetic_archive(7, "forward-v2", "multi-row-tip"),
            std::collections::BTreeMap::from([("sponsored".to_owned(), request)]),
            std::collections::BTreeMap::from([(
                "sponsored".to_owned(),
                LiveRequestFact::Acceptance,
            )]),
            std::collections::BTreeMap::from([(
                "sponsored".to_owned(),
                LiveTargetResponse::Accepted { identity },
            )]),
        )
        .expect("the multi-row run parses")
    }

    fn multi_row_observation(run: &LiveRunBinding) -> LiveReportObservation {
        let LiveTargetResponse::Accepted { identity } = &run.responses["sponsored"] else {
            panic!("the multi-row response is accepted");
        };
        LiveReportObservation::MultiRowSemantic {
            witness: LiveMultiRowSemanticWitness::new(
                run.run_id.clone(),
                "sponsored".to_owned(),
                *identity,
                std::collections::BTreeSet::from(["sponsor-change-absent", "sponsored"]),
                std::collections::BTreeMap::from([
                    (
                        "sponsor-change-absent",
                        LiveRowSemanticPredicate::SponsorChangeAbsent {
                            sponsor_input_index: 1,
                            fee_output_index: 1,
                            sponsor_change_program: vec![0x51, 0x99],
                        },
                    ),
                    (
                        "sponsored",
                        LiveRowSemanticPredicate::Sponsored {
                            sponsor_input_index: 1,
                            fee_output_index: 1,
                        },
                    ),
                ]),
            ),
        }
    }

    fn synthetic_run() -> LiveRunBinding {
        let request = synthetic_transaction(0x41, 50, 0x31);
        let identity = recomputed_txid(&request).expect("the request identity recomputes");
        LiveRunBinding::from_archive(
            synthetic_archive(6, "historical-v1", "recorded-tip"),
            std::collections::BTreeMap::from([("request-a".to_owned(), request)]),
            std::collections::BTreeMap::from([(
                "request-a".to_owned(),
                LiveRequestFact::Acceptance,
            )]),
            std::collections::BTreeMap::from([(
                "request-a".to_owned(),
                LiveTargetResponse::Accepted { identity },
            )]),
        )
        .expect("the synthetic run parses")
    }

    fn synthetic_mutation_run() -> LiveRunBinding {
        use target_elements_conformance::protocol::ObservedOutcomeLayer;

        let control = synthetic_transaction(0x42, 50, 0x31);
        let mutant = synthetic_transaction(0x42, 50, 0x32);
        let control_identity = recomputed_txid(&control).expect("the control identity recomputes");
        LiveRunBinding::from_archive(
            synthetic_archive(6, "historical-v1", "mutation-tip"),
            std::collections::BTreeMap::from([
                ("control".to_owned(), control),
                ("mutant".to_owned(), mutant),
            ]),
            std::collections::BTreeMap::from([
                ("control".to_owned(), LiveRequestFact::Control),
                (
                    "mutant".to_owned(),
                    LiveRequestFact::Refusal {
                        mutant: LiveMutantKind::MalformedSignature,
                        control_request_id: "control".to_owned(),
                        locator: LiveMutationLocator::WitnessItem {
                            input_index: 0,
                            item_index: 0,
                        },
                    },
                ),
            ]),
            std::collections::BTreeMap::from([
                (
                    "control".to_owned(),
                    LiveTargetResponse::Accepted {
                        identity: control_identity,
                    },
                ),
                (
                    "mutant".to_owned(),
                    LiveTargetResponse::Refused {
                        observed_layer: ObservedOutcomeLayer::ScriptPathRejection,
                        detail: "Invalid Schnorr signature".to_owned(),
                        control_identity,
                    },
                ),
            ]),
        )
        .expect("the mutation run parses")
    }

    fn mutation_observation(run: &LiveRunBinding) -> LiveReportObservation {
        let LiveTargetResponse::Refused {
            observed_layer,
            detail,
            ..
        } = &run.responses["mutant"]
        else {
            panic!("the mutant response is a refusal");
        };
        LiveReportObservation::NativeRefusalWithSupport {
            row: "malformed-signature",
            run_id: run.run_id.clone(),
            request_id: "mutant".to_owned(),
            declared_boundary: crate::matrix::EvidenceBoundary::ScriptPathRejection,
            observed_layer: *observed_layer,
            detail: detail.clone(),
            support: LiveSupportLink::new(
                run.run_id.clone(),
                "control".to_owned(),
                run.requests["control"].clone(),
                run.responses["control"].clone(),
            ),
        }
    }

    fn pair_projection() -> LivePairProjectionInput {
        LivePairProjectionInput::new(
            vec!["owner-a".to_owned()],
            vec![50],
            std::collections::BTreeMap::from([("owner-b".to_owned(), vec![0x51])]),
            std::collections::BTreeMap::from([("owner-b".to_owned(), 50)]),
        )
    }

    fn synthetic_pair_transaction(member: LivePairMember) -> Vec<u8> {
        use transaction::bytes::OutputWitness;
        use transaction::{
            AssetField, AssetId, InputWitness, NonceField, Outpoint, TargetInput, TargetOutput,
            TargetTransaction, Txid, ValueField,
        };

        let input = TargetInput::new(
            Outpoint::new(
                Txid::from_internal(
                    [match member {
                        LivePairMember::Explicit => 0x51,
                        LivePairMember::Private => 0x52,
                    }; 32],
                ),
                0,
            )
            .expect("the pair outpoint is admissible"),
            u32::MAX,
        );
        let value = match member {
            LivePairMember::Explicit => ValueField::Explicit(50),
            LivePairMember::Private => ValueField::Commitment([0x08; 33]),
        };
        let output = TargetOutput::new(
            AssetField::Explicit(AssetId::from_internal([0xa1; 32])),
            value,
            NonceField::Null,
            vec![0x51],
        );
        let witness = InputWitness::new(vec![vec![0x31]]);
        match member {
            LivePairMember::Explicit => {
                TargetTransaction::new(3, vec![input], vec![output], 0, vec![witness])
                    .expect("the explicit pair member is well formed")
            }
            LivePairMember::Private => TargetTransaction::with_output_witnesses(
                3,
                vec![input],
                vec![output],
                0,
                vec![witness],
                vec![OutputWitness::range_proof_only(vec![0xaa])],
            )
            .expect("the private pair member is well formed"),
        }
        .encode()
    }

    fn synthetic_pair_run() -> LiveRunBinding {
        let explicit = synthetic_pair_transaction(LivePairMember::Explicit);
        let private = synthetic_pair_transaction(LivePairMember::Private);
        let explicit_identity =
            recomputed_txid(&explicit).expect("the explicit identity recomputes");
        let private_identity = recomputed_txid(&private).expect("the private identity recomputes");
        LiveRunBinding::from_archive(
            synthetic_archive(6, "historical-v1", "pair-tip"),
            std::collections::BTreeMap::from([
                ("explicit".to_owned(), explicit),
                ("private".to_owned(), private),
            ]),
            std::collections::BTreeMap::from([
                (
                    "explicit".to_owned(),
                    LiveRequestFact::Paired {
                        member: LivePairMember::Explicit,
                        projection: pair_projection(),
                    },
                ),
                (
                    "private".to_owned(),
                    LiveRequestFact::Paired {
                        member: LivePairMember::Private,
                        projection: pair_projection(),
                    },
                ),
            ]),
            std::collections::BTreeMap::from([
                (
                    "explicit".to_owned(),
                    LiveTargetResponse::Accepted {
                        identity: explicit_identity,
                    },
                ),
                (
                    "private".to_owned(),
                    LiveTargetResponse::Accepted {
                        identity: private_identity,
                    },
                ),
            ]),
        )
        .expect("the pair run parses")
    }

    fn pair_observation(run: &LiveRunBinding, relation: &str) -> LiveReportObservation {
        let LiveTargetResponse::Accepted {
            identity: explicit_identity,
        } = &run.responses["explicit"]
        else {
            panic!("the explicit response is accepted");
        };
        let LiveTargetResponse::Accepted {
            identity: private_identity,
        } = &run.responses["private"]
        else {
            panic!("the private response is accepted");
        };
        LiveReportObservation::PairedRelation {
            row: "projection-equality-with-paired-explicit",
            explicit_run_id: run.run_id.clone(),
            explicit_request_id: "explicit".to_owned(),
            explicit_identity: *explicit_identity,
            private_run_id: run.run_id.clone(),
            private_request_id: "private".to_owned(),
            private_identity: *private_identity,
            relation: relation.to_owned(),
        }
    }

    #[test]
    fn schema_five_is_hard_rejected() {
        let plan = derive_live_evidence_plan().expect("the evidence plan derives");
        let target = projection();
        let mut report =
            assemble_live_safety_report(&plan, target.clone()).expect("the live report assembles");
        report.schema = 5;

        assert_eq!(
            validate_live_safety_report(report, &plan, &target),
            Err(LiveSafetyReportRefusal::UnsupportedSchema(5)),
        );
    }

    #[test]
    fn the_composite_two_acceptance_is_schema_six_only_and_proves_opposite_parity() {
        let first = synthetic_acceptance_run("primary-ceremony", "primary", 0x51, 0x08);
        let second = synthetic_acceptance_run("balancing-ceremony", "balancing", 0x52, 0x09);
        let observation = composite_observation(
            &first,
            "primary-ceremony",
            "primary",
            &second,
            "balancing-ceremony",
            "balancing",
        );
        let mut runs = vec![first, second];
        runs.sort_by(|left, right| left.run_id.cmp(&right.run_id));
        assert_eq!(validate_bound_observations(&[observation], &runs), Ok(1),);
    }

    #[test]
    fn a_forged_composite_member_identity_refuses() {
        let first = synthetic_acceptance_run("primary-ceremony", "primary", 0x53, 0x08);
        let second = synthetic_acceptance_run("balancing-ceremony", "balancing", 0x54, 0x09);
        let mut runs = vec![first, second];
        runs.sort_by(|left, right| left.run_id.cmp(&right.run_id));
        let forged_identity =
            parsed_identity("75e823f7c5c70ddfbd9584f90f67298f2570907947829828066c7047b21b53b1");
        let forged_run = runs
            .iter_mut()
            .find(|run| run.responses.contains_key("primary"))
            .expect("the primary run is present");
        let forged_run_id = forged_run.run_id.clone();
        forged_run.responses.insert(
            "primary".to_owned(),
            LiveTargetResponse::Accepted {
                identity: forged_identity,
            },
        );
        let primary = runs
            .iter()
            .find(|run| run.responses.contains_key("primary"))
            .expect("the primary run remains present");
        let balancing = runs
            .iter()
            .find(|run| run.responses.contains_key("balancing"))
            .expect("the balancing run remains present");
        let observation = composite_observation(
            primary,
            "primary-ceremony",
            "primary",
            balancing,
            "balancing-ceremony",
            "balancing",
        );
        assert_eq!(
            validate_bound_observations(&[observation], &runs),
            Err(LiveSafetyReportRefusal::AcceptedIdentityDiffers(
                forged_run_id,
                "primary".to_owned(),
            )),
        );
    }

    #[test]
    fn same_ceremony_and_same_parity_composites_refuse_independently() {
        let first = synthetic_acceptance_run("one-ceremony", "primary", 0x55, 0x08);
        let second = synthetic_acceptance_run("one-ceremony", "balancing", 0x56, 0x09);
        let mut same_ceremony = composite_observation(
            &first,
            "one-ceremony",
            "primary",
            &second,
            "one-ceremony",
            "balancing",
        );
        let mut runs = vec![first, second];
        runs.sort_by(|left, right| left.run_id.cmp(&right.run_id));
        assert_eq!(
            validate_bound_observations(std::slice::from_ref(&same_ceremony), &runs),
            Err(LiveSafetyReportRefusal::CompositeCeremonyReused(
                "one-ceremony".to_owned(),
            )),
        );

        let first = synthetic_acceptance_run("primary-ceremony", "primary", 0x57, 0x08);
        let second = synthetic_acceptance_run("balancing-ceremony", "balancing", 0x58, 0x08);
        same_ceremony = composite_observation(
            &first,
            "primary-ceremony",
            "primary",
            &second,
            "balancing-ceremony",
            "balancing",
        );
        let mut runs = vec![first, second];
        runs.sort_by(|left, right| left.run_id.cmp(&right.run_id));
        assert_eq!(
            validate_bound_observations(&[same_ceremony], &runs),
            Err(LiveSafetyReportRefusal::CompositeParityNotOpposite(
                "both-commitment-parity-forms",
            )),
        );
    }

    #[test]
    fn witness_path_shape_is_schema_six_only_and_recomputed() {
        let run = witness_path_shape_run();
        assert_eq!(
            compare_run_bindings(std::slice::from_ref(&run), std::slice::from_ref(&run)),
            Ok(()),
        );
        let mut forged = run;
        let run_id = forged.run_id.clone();
        let LiveRequestFact::Refusal { locator, .. } = forged
            .request_facts
            .get_mut("mutant")
            .expect("the witness-path mutant fact is present")
        else {
            panic!("the witness-path mutant is a refusal");
        };
        let LiveMutationLocator::WitnessPathShape {
            changed_positions, ..
        } = locator
        else {
            panic!("the locator carries a witness-path shape");
        };
        *changed_positions = vec![0];
        assert_eq!(
            compare_run_bindings(std::slice::from_ref(&forged), std::slice::from_ref(&forged),),
            Err(LiveSafetyReportRefusal::MutationLocatorDiffers(
                run_id,
                "mutant".to_owned(),
            )),
        );
    }

    #[test]
    fn witness_path_shape_refuses_the_bitcoin_leaf_version() {
        let signature = vec![0x31; 64];
        let control = synthetic_witness_transaction(0x63, vec![vec![signature.clone()]]);
        let mutant = synthetic_witness_transaction(
            0x63,
            vec![vec![
                signature,
                vec![0x51],
                synthetic_control_block_with_version(0xc0),
            ]],
        );
        let run = synthetic_refusal_run(
            "obsolete-leaf-version-tip",
            control,
            mutant,
            LiveMutantKind::KeyPathEscape,
            LiveMutationLocator::WitnessPathShape {
                input_index: 0,
                control_stack_items: 1,
                mutant_stack_items: 3,
                changed_positions: vec![1, 2],
                control_role: LiveWitnessPathRole::KeyPath,
                mutant_role: LiveWitnessPathRole::ScriptPath,
                witnessless_serialization_equal: true,
            },
        );
        let run_id = run.run_id.clone();

        assert_eq!(
            compare_run_bindings(std::slice::from_ref(&run), std::slice::from_ref(&run)),
            Err(LiveSafetyReportRefusal::MutationLocatorDiffers(
                run_id,
                "mutant".to_owned(),
            )),
        );
    }

    #[test]
    fn witness_path_shape_requires_one_key_path_item_and_equal_witnessless_bytes() {
        let signature = vec![0x31; 64];
        let not_key_path =
            synthetic_witness_transaction(0x64, vec![vec![signature.clone(), vec![0x01]]]);
        let script_path = synthetic_witness_transaction(
            0x64,
            vec![vec![
                signature.clone(),
                vec![0x51],
                synthetic_control_block(),
            ]],
        );
        let locator = LiveMutationLocator::WitnessPathShape {
            input_index: 0,
            control_stack_items: 2,
            mutant_stack_items: 3,
            changed_positions: vec![1, 2],
            control_role: LiveWitnessPathRole::KeyPath,
            mutant_role: LiveWitnessPathRole::ScriptPath,
            witnessless_serialization_equal: true,
        };
        let wrong_key_path = synthetic_refusal_run(
            "wrong-key-path-shape-tip",
            not_key_path,
            script_path,
            LiveMutantKind::KeyPathEscape,
            locator,
        );
        let wrong_key_path_id = wrong_key_path.run_id.clone();
        assert_eq!(
            compare_run_bindings(
                std::slice::from_ref(&wrong_key_path),
                std::slice::from_ref(&wrong_key_path),
            ),
            Err(LiveSafetyReportRefusal::MutationLocatorDiffers(
                wrong_key_path_id,
                "mutant".to_owned(),
            )),
        );

        let key_path = synthetic_witness_transaction(0x65, vec![vec![signature.clone()]]);
        let different_witnessless = synthetic_witness_transaction(
            0x66,
            vec![vec![signature, vec![0x51], synthetic_control_block()]],
        );
        let wrong_witnessless = synthetic_refusal_run(
            "wrong-witnessless-shape-tip",
            key_path,
            different_witnessless,
            LiveMutantKind::KeyPathEscape,
            LiveMutationLocator::WitnessPathShape {
                input_index: 0,
                control_stack_items: 1,
                mutant_stack_items: 3,
                changed_positions: vec![1, 2],
                control_role: LiveWitnessPathRole::KeyPath,
                mutant_role: LiveWitnessPathRole::ScriptPath,
                witnessless_serialization_equal: true,
            },
        );
        let wrong_witnessless_id = wrong_witnessless.run_id.clone();
        assert_eq!(
            compare_run_bindings(
                std::slice::from_ref(&wrong_witnessless),
                std::slice::from_ref(&wrong_witnessless),
            ),
            Err(LiveSafetyReportRefusal::MutationLocatorDiffers(
                wrong_witnessless_id,
                "mutant".to_owned(),
            )),
        );
    }

    #[test]
    fn missing_sponsor_authorization_requires_the_exact_empty_suffix() {
        let run = missing_sponsor_authorization_run();
        assert_eq!(
            compare_run_bindings(std::slice::from_ref(&run), std::slice::from_ref(&run)),
            Ok(()),
        );
        let mut forged = run;
        let run_id = forged.run_id.clone();
        let LiveRequestFact::Refusal { locator, .. } = forged
            .request_facts
            .get_mut("mutant")
            .expect("the sponsor mutant fact is present")
        else {
            panic!("the sponsor mutant is a refusal");
        };
        let LiveMutationLocator::WitnessItem { item_index, .. } = locator else {
            panic!("the sponsor mutant carries a witness-item locator");
        };
        *item_index = 1;
        assert_eq!(
            compare_run_bindings(std::slice::from_ref(&forged), std::slice::from_ref(&forged),),
            Err(LiveSafetyReportRefusal::MutationLocatorDiffers(
                run_id,
                "mutant".to_owned(),
            )),
        );
    }

    #[test]
    fn committed_leaf_arrangement_is_schema_six_only_and_recomputed() {
        let run = committed_leaf_arrangement_run();
        assert_eq!(
            compare_run_bindings(std::slice::from_ref(&run), std::slice::from_ref(&run)),
            Ok(()),
        );
        let mut forged = run;
        let run_id = forged.run_id.clone();
        let LiveRequestFact::Refusal { locator, .. } = forged
            .request_facts
            .get_mut("mutant")
            .expect("the arrangement mutant fact is present")
        else {
            panic!("the arrangement mutant is a refusal");
        };
        let LiveMutationLocator::CommittedLeafArrangement {
            mutant_coordinator_leaf_indices,
            ..
        } = locator
        else {
            panic!("the locator carries a committed-leaf arrangement");
        };
        *mutant_coordinator_leaf_indices = vec![0];
        assert_eq!(
            compare_run_bindings(std::slice::from_ref(&forged), std::slice::from_ref(&forged),),
            Err(LiveSafetyReportRefusal::MutationLocatorDiffers(
                run_id,
                "mutant".to_owned(),
            )),
        );

        let obsolete =
            committed_leaf_arrangement_run_with_block(synthetic_control_block_with_version(0xc0));
        let obsolete_run_id = obsolete.run_id.clone();
        assert_eq!(
            compare_run_bindings(
                std::slice::from_ref(&obsolete),
                std::slice::from_ref(&obsolete),
            ),
            Err(LiveSafetyReportRefusal::MutationLocatorDiffers(
                obsolete_run_id,
                "mutant".to_owned(),
            )),
        );
    }

    #[test]
    fn composite_member_may_share_one_ordinary_acceptance() {
        let first = synthetic_acceptance_run("primary-ceremony", "primary", 0xb1, 0x08);
        let second = synthetic_acceptance_run("balancing-ceremony", "balancing", 0xb2, 0x09);
        let composite = composite_observation(
            &first,
            "primary-ceremony",
            "primary",
            &second,
            "balancing-ceremony",
            "balancing",
        );
        let LiveTargetResponse::Accepted { identity } = &first.responses["primary"] else {
            panic!("the shared response is accepted");
        };
        let ordinary = LiveReportObservation::NativeAcceptance {
            row: "private-one-to-one",
            run_id: first.run_id.clone(),
            request_id: "primary".to_owned(),
            identity: *identity,
        };
        let mut runs = vec![first, second];
        runs.sort_by(|left, right| left.run_id.cmp(&right.run_id));

        assert_eq!(
            validate_bound_observations(&[composite, ordinary], &runs),
            Ok(2),
        );
    }

    #[test]
    fn composite_member_reuse_is_closed_to_one_ordinary_acceptance() {
        let mut observed = std::collections::BTreeMap::new();
        assert_eq!(
            insert_observation_link(
                &mut observed,
                "run",
                "request",
                ObservationLinkClass::OrdinaryAcceptance("private-one-to-one"),
            ),
            Ok(()),
        );
        assert_eq!(
            insert_observation_link(
                &mut observed,
                "run",
                "request",
                ObservationLinkClass::CompositeAcceptanceMember("both-commitment-parity-forms",),
            ),
            Ok(()),
        );
        assert_eq!(
            insert_observation_link(
                &mut observed,
                "run",
                "request",
                ObservationLinkClass::OrdinaryAcceptance("private-one-to-one"),
            ),
            Err(LiveSafetyReportRefusal::ObservationAliased(
                "request".to_owned(),
            )),
        );

        for class in [
            ObservationLinkClass::NativeRefusal,
            ObservationLinkClass::PairMember,
            ObservationLinkClass::MultiRowSemantic,
            ObservationLinkClass::OrdinaryAcceptance("another-row"),
            ObservationLinkClass::CompositeAcceptanceMember("another-row"),
            ObservationLinkClass::CompositeAcceptanceMember("both-commitment-parity-forms"),
        ] {
            let mut observed = std::collections::BTreeMap::new();
            insert_observation_link(
                &mut observed,
                "run",
                "request",
                ObservationLinkClass::CompositeAcceptanceMember("both-commitment-parity-forms"),
            )
            .expect("one composite member is unique");
            assert_eq!(
                insert_observation_link(&mut observed, "run", "request", class),
                Err(LiveSafetyReportRefusal::ObservationAliased(
                    "request".to_owned(),
                )),
            );
        }
    }

    #[test]
    fn the_real_corpus_roles_match_every_typed_observation_form() {
        let evidence = derive_live_evidence_plan().expect("the evidence plan derives");
        let observations =
            observations_from_overlay(&evidence).expect("the overlay observations derive");

        assert_eq!(
            validate_bound_observations(&observations, evidence.runs()),
            Ok(42),
        );
    }

    #[test]
    fn conservation_control_reuse_is_closed_to_its_exact_acceptance() {
        let request_id = "conservation-negatives/request-6";
        let control = synthetic_control_acceptance_run(request_id);
        let observation = |run: &LiveRunBinding, row: &'static str, request: &str| {
            let LiveTargetResponse::Accepted { identity } = &run.responses[request] else {
                panic!("the conservation control response is accepted");
            };
            LiveReportObservation::NativeAcceptance {
                row,
                run_id: run.run_id.clone(),
                request_id: request.to_owned(),
                identity: *identity,
            }
        };
        assert_eq!(
            validate_bound_observations(
                &[observation(&control, "target-ct-conservation", request_id)],
                std::slice::from_ref(&control),
            ),
            Ok(1),
        );
        assert_eq!(
            validate_bound_observations(
                &[observation(&control, "another-row", request_id)],
                std::slice::from_ref(&control),
            ),
            Err(LiveSafetyReportRefusal::RequestRoleDiffers(
                control.run_id.clone(),
                request_id.to_owned(),
            )),
        );

        let wrong_request_id = "conservation-negatives/request-5";
        let wrong_request = synthetic_control_acceptance_run(wrong_request_id);
        assert_eq!(
            validate_bound_observations(
                &[observation(
                    &wrong_request,
                    "target-ct-conservation",
                    wrong_request_id,
                )],
                std::slice::from_ref(&wrong_request),
            ),
            Err(LiveSafetyReportRefusal::RequestRoleDiffers(
                wrong_request.run_id.clone(),
                wrong_request_id.to_owned(),
            )),
        );

        let wrong_role = synthetic_acceptance_run("conservation-negatives", request_id, 0xb4, 0x08);
        assert_eq!(
            validate_bound_observations(
                &[observation(
                    &wrong_role,
                    "target-ct-conservation",
                    request_id,
                )],
                std::slice::from_ref(&wrong_role),
            ),
            Err(LiveSafetyReportRefusal::RequestRoleDiffers(
                wrong_role.run_id.clone(),
                request_id.to_owned(),
            )),
        );
    }

    #[test]
    fn reused_primary_links_refuse() {
        let run = synthetic_run();
        let LiveTargetResponse::Accepted { identity } = &run.responses["request-a"] else {
            panic!("the primary response is accepted");
        };
        let observation = LiveReportObservation::NativeAcceptance {
            row: "row-a",
            run_id: run.run_id.clone(),
            request_id: "request-a".to_owned(),
            identity: *identity,
        };
        assert_eq!(
            validate_bound_observations(
                &[observation.clone(), observation],
                std::slice::from_ref(&run),
            ),
            Err(LiveSafetyReportRefusal::ObservationAliased(
                "request-a".to_owned(),
            )),
        );
    }

    #[test]
    fn one_byte_identical_support_link_may_repeat_within_one_run() {
        let run = repeated_support_run();
        let observations = [
            support_observation(&run, "mutant-a"),
            support_observation(&run, "mutant-b"),
        ];
        assert_eq!(
            validate_bound_observations(&observations, std::slice::from_ref(&run)),
            Ok(2),
        );
    }

    #[test]
    fn a_cross_run_support_link_refuses() {
        let run = repeated_support_run();
        let mut observation = support_observation(&run, "mutant-a");
        let LiveReportObservation::NativeRefusalWithSupport { support, .. } = &mut observation
        else {
            panic!("the observation carries a support link");
        };
        support.run_id = "another-run".to_owned();
        assert_eq!(
            validate_bound_observations(&[observation], std::slice::from_ref(&run)),
            Err(LiveSafetyReportRefusal::CrossRunRequestLink(
                "control".to_owned(),
                "control".to_owned(),
            )),
        );
    }

    #[test]
    fn a_byte_divergent_support_repeat_refuses() {
        let run = repeated_support_run();
        let first = support_observation(&run, "mutant-a");
        let mut second = support_observation(&run, "mutant-b");
        let LiveReportObservation::NativeRefusalWithSupport { support, .. } = &mut second else {
            panic!("the second observation carries a support link");
        };
        support.request_bytes[0] ^= 1;
        assert_eq!(
            validate_bound_observations(&[first, second], std::slice::from_ref(&run)),
            Err(LiveSafetyReportRefusal::SupportLinkFactsDiffer(
                run.run_id.clone(),
                "control".to_owned(),
            )),
        );
    }

    #[test]
    fn the_multi_row_witness_is_schema_six_only_and_proves_each_row() {
        let run = synthetic_multi_row_run();
        let observation = multi_row_observation(&run);
        assert_eq!(
            validate_bound_observations(&[observation], std::slice::from_ref(&run)),
            Ok(2),
        );
    }

    #[test]
    fn a_multi_row_witness_cannot_be_reused_by_an_ordinary_observation() {
        let run = synthetic_multi_row_run();
        let multi_row = multi_row_observation(&run);
        let LiveTargetResponse::Accepted { identity } = &run.responses["sponsored"] else {
            panic!("the multi-row response is accepted");
        };
        let ordinary = LiveReportObservation::NativeAcceptance {
            row: "another-row",
            run_id: run.run_id.clone(),
            request_id: "sponsored".to_owned(),
            identity: *identity,
        };
        assert_eq!(
            validate_bound_observations(&[multi_row, ordinary], std::slice::from_ref(&run)),
            Err(LiveSafetyReportRefusal::ObservationAliased(
                "sponsored".to_owned(),
            )),
        );
    }

    #[test]
    fn a_multi_row_witness_with_one_unproven_predicate_refuses() {
        let run = synthetic_multi_row_run();
        let mut observation = multi_row_observation(&run);
        let LiveReportObservation::MultiRowSemantic { witness } = &mut observation else {
            panic!("the observation carries a multi-row witness");
        };
        witness.predicates.insert(
            "sponsored",
            LiveRowSemanticPredicate::Sponsored {
                sponsor_input_index: 1,
                fee_output_index: 0,
            },
        );
        assert_eq!(
            validate_bound_observations(&[observation], std::slice::from_ref(&run)),
            Err(LiveSafetyReportRefusal::SemanticPredicateNotProven(
                "sponsored",
            )),
        );
    }

    #[test]
    fn an_unexpected_boundary_refusal_is_outstanding_and_fails_the_report() {
        use super::{completeness_of, standing_name};
        use crate::live_evidence::{LiveEvidenceCensus, LiveRowStanding};
        use crate::matrix::EvidenceBoundary;
        use target_elements_conformance::protocol::ObservedOutcomeLayer;

        let standing = LiveRowStanding::NativeRefusalAtUnexpectedBoundary {
            declared_boundary: EvidenceBoundary::ScriptPathRejection,
            observed_layer: ObservedOutcomeLayer::ConsensusRejectionBeforeScript,
            control_identity: "40cb6c4ee284ed38555a4840198c8130d1e2c3246b57b9d8b93842c3c6730029",
            refusal_detail: "bad-txns-in-ne-out",
        };

        // It renders under its OWN spelling, so a reader can tell a
        // contradiction from a row that is merely waiting.
        assert_eq!(
            standing_name(&standing),
            "native-refusal-at-unexpected-boundary",
        );

        // It lands in `outstanding`. That list is built by filtering on
        // exactly this predicate, so asserting the predicate is
        // asserting the membership.
        assert!(!standing.is_answered());

        // And the census carrying one is FAILED rather than partial —
        // the mismatch taking precedence, with every other bucket clear
        // so the verdict can only have come from the mismatch.
        let census = LiveEvidenceCensus::one_unexpected_boundary_for_tests();
        assert!(!census.every_required_row_is_answered());
        assert_eq!(completeness_of(census), LiveSafetyCompleteness::Failed);
        assert_eq!(LiveSafetyCompleteness::Failed.name(), "failed");
    }

    #[test]
    fn a_report_validates_against_the_plan_it_is_about() {
        let plan = derive_live_evidence_plan().expect("the evidence plan derives");
        let target = projection();
        let report = assemble_live_safety_report(&plan, target.clone()).expect("assembles");
        let validated = validate_live_safety_report(report, &plan, &target)
            .expect("the assembled report validates");

        // Only comparisons that actually ran are recorded. The validated
        // corpus drives every safety-report recomputation except the separate
        // resource-report comparison.
        assert_eq!(
            validated.recomputed_items(),
            &std::collections::BTreeSet::from([
                RecomputedItem::Role,
                RecomputedItem::Schema,
                RecomputedItem::TargetDeploymentBinding,
                RecomputedItem::RequestResponseCensus,
                RecomputedItem::ResponseShape,
                RecomputedItem::CaseCensus,
                RecomputedItem::RelationCensus,
                RecomputedItem::MutationLinks,
                RecomputedItem::TargetVerdictComparison,
                RecomputedItem::SemanticProjection,
                RecomputedItem::DisclosureComparison,
                RecomputedItem::ExecutorProvenance,
                RecomputedItem::Summary,
            ]),
        );
        assert_eq!(RecomputedItem::ALL.len(), 14);
        assert!(
            !validated
                .recomputed_items()
                .contains(&RecomputedItem::ResourceComparison)
        );
    }

    #[test]
    fn the_assembled_report_is_the_truthful_bound_corpus_ledger() {
        let plan = derive_live_evidence_plan().expect("the evidence plan derives");
        let target = projection();
        let report = assemble_live_safety_report(&plan, target.clone()).expect("assembles");
        assert_eq!(report.runs().len(), 28);
        assert_eq!(report.runs(), plan.runs());
        assert_eq!(report.observations().len(), 45);
        assert_eq!(report.report_layer_requirements().len(), 2);
        assert_eq!(report.census().report_layer_required(), 2);
        assert_eq!(report.census().report_layer_observed(), 0);
        let validated = validate_live_safety_report(report, &plan, &target).expect("validates");
        let rendered = render_live_safety_report(&validated);

        assert!(rendered.contains("target_evidence bound\n"));
        assert!(rendered.contains("run_bindings 28\n"));
        assert!(!rendered.contains("deployment none"));
        assert!(!rendered.contains("run none"));
        assert!(!rendered.contains("NoRunRequested"));

        for exact in [
            "recorded_observation_unbound 0\n",
            "native_run_observed 24\n",
            "native_refusal_observed 17\n",
            "determinism_observed 1\n",
            "paired_relation_observed 1\n",
            "first_party_fact_observed 3\n",
            "observations 48\n",
            "accepted 24\n",
            "refused 17\n",
            "determinism 1\n",
            "paired_relation 1\n",
            "first_party_fact 3\n",
            "report_layer_required 0\n",
            "report_layer_observed 2\n",
            "report_layer_observations 2\n",
            "recorded_unbound_native_acceptance 0\n",
            "recorded_unbound_native_refusal 0\n",
            "recorded_unbound_paired_relation 0\n",
        ] {
            assert!(rendered.contains(exact), "missing {exact:?}");
        }
        assert_eq!(validated.census().report_layer_required(), 0);
        assert_eq!(validated.census().report_layer_observed(), 2);
        assert_eq!(validated.report_layer_observations().len(), 2);
        assert_eq!(
            validated
                .report_layer_observations()
                .iter()
                .map(|observation| {
                    (
                        observation.row(),
                        observation.requirement(),
                        observation.schema(),
                    )
                })
                .collect::<Vec<_>>(),
            vec![
                (
                    "report-publishes-sponsor-amount",
                    LiveReportRequirement::SponsorAmountAbsent,
                    LIVE_SAFETY_REPORT_SCHEMA,
                ),
                (
                    "report-publishes-sponsor-opening",
                    LiveReportRequirement::SponsorOpeningAbsent,
                    LIVE_SAFETY_REPORT_SCHEMA,
                ),
            ],
        );
        let answered = validated.census().first_party_discharged()
            + validated.census().native_run_observed()
            + validated.census().native_refusal_observed()
            + validated.census().determinism_observed()
            + validated.census().paired_relation_observed()
            + validated.census().first_party_fact_observed()
            + validated.census().report_layer_observed();
        assert_eq!(answered, 82);
        assert_eq!(validated.outstanding().len(), 25);
        assert_eq!(
            super::completeness_of(validated.census()),
            LiveSafetyCompleteness::PartialRequiredRowsOutstanding,
        );
    }

    #[test]
    fn the_envelope_refuses_a_run_set_not_owned_by_the_overlay() {
        let evidence = derive_live_evidence_plan().expect("the evidence plan derives");
        let target = projection();
        let mut report = assemble_live_safety_report(&evidence, target.clone()).expect("assembles");
        report.runs.pop();

        assert_eq!(
            validate_live_safety_report(report, &evidence, &target),
            Err(LiveSafetyReportRefusal::RunCensusDiffers),
        );
    }

    #[test]
    fn every_rendered_target_identity_is_typed_and_shared_identities_keep_their_rows() {
        let plan = derive_live_evidence_plan().expect("the evidence plan derives");
        let target = projection();
        let report = assemble_live_safety_report(&plan, target.clone()).expect("assembles");
        let validated = validate_live_safety_report(report, &plan, &target).expect("validates");
        let rendered = render_live_safety_report(&validated);
        let mut occurrences = std::collections::BTreeMap::new();

        for line in rendered
            .lines()
            .filter(|line| line.starts_with("observation "))
        {
            let words: Vec<_> = line.split_whitespace().collect();
            for pair in words.windows(2) {
                if matches!(
                    pair[0],
                    "identity" | "control" | "explicit-identity" | "private-identity"
                ) {
                    let parsed = transaction::Txid::from_target_display(pair[1])
                        .expect("every rendered identity parses");
                    assert_eq!(parsed.to_target_display(), pair[1]);
                    *occurrences.entry(pair[1]).or_insert(0_usize) += 1;
                }
            }
        }
        assert!(occurrences.values().any(|count| *count > 1));
        assert_eq!(
            rendered
                .lines()
                .filter(|line| line.starts_with("observation "))
                .count(),
            48,
        );
    }

    #[test]
    fn wrong_and_unknown_archival_revision_facts_refuse() {
        let mut wrong = synthetic_run();
        let run_id = wrong.run_id.clone();
        wrong.revision = NativeProtocolRevision::Revision7;
        assert_eq!(
            compare_run_bindings(&[wrong.clone()], &[wrong]),
            Err(LiveSafetyReportRefusal::RunProtocolRevisionDiffers(run_id)),
        );

        let mut unknown = synthetic_run();
        unknown.archive_bytes = synthetic_archive(99, "historical-v1", "recorded-tip");
        assert_eq!(
            compare_run_bindings(&[unknown.clone()], &[unknown]),
            Err(LiveSafetyReportRefusal::UnknownProtocolRevision(99)),
        );
    }

    #[test]
    fn wrong_and_unknown_digest_algorithms_refuse_without_relabeling() {
        let mut wrong = synthetic_run();
        let run_id = wrong.run_id.clone();
        wrong
            .digest_facts
            .get_mut("fixture")
            .expect("the fixture digest is present")
            .algorithm = FixtureDigestAlgorithm::ForwardV2;
        assert_eq!(
            compare_run_bindings(&[wrong.clone()], &[wrong]),
            Err(LiveSafetyReportRefusal::RunDigestFactsDiffer(run_id)),
        );

        let source = synthetic_run();
        let relabeled = LiveRunBinding::from_archive(
            synthetic_archive(6, "forward-v2", "recorded-tip"),
            source.requests,
            source.request_facts,
            source.responses,
        )
        .expect("the relabeled archive is structurally typed");
        assert_eq!(
            compare_run_bindings(
                std::slice::from_ref(&relabeled),
                std::slice::from_ref(&relabeled),
            ),
            Err(LiveSafetyReportRefusal::RunDigestFactsDiffer(
                relabeled.run_id.clone(),
            )),
        );

        let mut unknown = synthetic_run();
        unknown.archive_bytes = synthetic_archive(6, "invented-v3", "recorded-tip");
        assert_eq!(
            compare_run_bindings(&[unknown.clone()], &[unknown]),
            Err(LiveSafetyReportRefusal::UnknownDigestAlgorithm(
                "invented-v3".to_owned(),
            )),
        );
    }

    #[test]
    fn run_ids_are_deterministic_content_addresses() {
        let first = synthetic_run();
        let second = synthetic_run();
        assert_eq!(first.run_id, second.run_id);

        let changed = LiveRunBinding::from_archive(
            synthetic_archive(6, "historical-v1", "different-source-tip"),
            first.requests.clone(),
            first.request_facts.clone(),
            first.responses.clone(),
        )
        .expect("the changed run parses");
        assert_ne!(first.run_id, changed.run_id);
        assert_eq!(
            compare_run_bindings(std::slice::from_ref(&first), std::slice::from_ref(&first),),
            Ok(()),
        );

        let mut forged_address = first;
        forged_address.run_id = "not-the-content-address".to_owned();
        assert_eq!(
            compare_run_bindings(
                std::slice::from_ref(&forged_address),
                std::slice::from_ref(&forged_address),
            ),
            Err(LiveSafetyReportRefusal::RunIdDiffers(
                "not-the-content-address".to_owned(),
            )),
        );
    }

    #[test]
    fn the_live_schema_renders_the_complete_run_field_set() {
        let run = synthetic_run();
        let mut rendered = String::new();
        super::render_run_bindings(&mut rendered, std::slice::from_ref(&run));
        assert!(rendered.contains("run_archive "));
        assert!(rendered.contains("run_protocol_revision 6\n"));
        assert!(rendered.contains(&format!(
            "run_digest fixture historical-v1 {}\n",
            "33".repeat(32),
        )));
        assert!(rendered.contains("run_request request-a "));
        assert!(rendered.contains("run_request_fact request-a acceptance\n"));
        assert!(rendered.contains("run_response request-a accepted "));
    }

    #[test]
    fn a_forged_request_breaks_accepted_txid_recomputation() {
        let mut forged = synthetic_run();
        let run_id = forged.run_id.clone();
        forged.requests.insert(
            "request-a".to_owned(),
            synthetic_transaction(0x43, 51, 0x31),
        );
        assert_eq!(
            compare_run_bindings(&[forged.clone()], &[forged]),
            Err(LiveSafetyReportRefusal::AcceptedIdentityDiffers(
                run_id,
                "request-a".to_owned(),
            )),
        );
    }

    #[test]
    fn a_refusal_with_a_mismatched_control_identity_refuses() {
        let mut run = synthetic_mutation_run();
        let run_id = run.run_id.clone();
        let LiveTargetResponse::Refused {
            control_identity, ..
        } = run
            .responses
            .get_mut("mutant")
            .expect("the mutant response is present")
        else {
            panic!("the mutant response is refused");
        };
        *control_identity =
            parsed_identity("75e823f7c5c70ddfbd9584f90f67298f2570907947829828066c7047b21b53b1");
        assert_eq!(
            compare_run_bindings(&[run.clone()], &[run]),
            Err(LiveSafetyReportRefusal::RefusalControlIdentityDiffers(
                run_id,
                "mutant".to_owned(),
            )),
        );
    }

    #[test]
    fn refusal_layer_and_detail_match_the_bound_observation_exactly() {
        use target_elements_conformance::protocol::ObservedOutcomeLayer;

        let run = synthetic_mutation_run();
        let mut observation = mutation_observation(&run);
        let LiveReportObservation::NativeRefusalWithSupport { detail, .. } = &mut observation
        else {
            panic!("the observation is a refusal");
        };
        *detail = "a matching importer invented this detail".to_owned();
        assert_eq!(
            validate_bound_observations(&[observation], std::slice::from_ref(&run)),
            Err(LiveSafetyReportRefusal::RunResponseDiffers(
                run.run_id.clone(),
            )),
        );

        let mut wrong_layer = mutation_observation(&run);
        let LiveReportObservation::NativeRefusalWithSupport { observed_layer, .. } =
            &mut wrong_layer
        else {
            panic!("the observation is a refusal");
        };
        *observed_layer = ObservedOutcomeLayer::ConsensusRejectionBeforeScript;
        assert_eq!(
            validate_bound_observations(&[wrong_layer], std::slice::from_ref(&run)),
            Err(LiveSafetyReportRefusal::RunResponseDiffers(
                run.run_id.clone(),
            )),
        );

        assert_eq!(
            validate_bound_observations(&[mutation_observation(&run)], std::slice::from_ref(&run),),
            Ok(1),
        );
    }

    #[test]
    fn orphan_responses_and_reused_request_ids_refuse() {
        let mut orphan = synthetic_run();
        let run_id = orphan.run_id.clone();
        orphan.responses.insert(
            "orphan".to_owned(),
            LiveTargetResponse::Accepted {
                identity: parsed_identity(
                    "75e823f7c5c70ddfbd9584f90f67298f2570907947829828066c7047b21b53b1",
                ),
            },
        );
        assert_eq!(
            compare_run_bindings(&[orphan.clone()], &[orphan]),
            Err(LiveSafetyReportRefusal::RequestResponseCensusDiffers(
                run_id,
            )),
        );

        let first = synthetic_run();
        let second = LiveRunBinding::from_archive(
            synthetic_archive(6, "historical-v1", "another-tip"),
            first.requests.clone(),
            first.request_facts.clone(),
            first.responses.clone(),
        )
        .expect("the second run parses");
        let runs = if first.run_id < second.run_id {
            vec![first, second]
        } else {
            vec![second, first]
        };
        assert_eq!(
            compare_run_bindings(&runs, &runs),
            Err(LiveSafetyReportRefusal::ReusedRequestId(
                "request-a".to_owned(),
            )),
        );
    }

    #[test]
    fn cross_run_control_links_refuse() {
        let original = synthetic_mutation_run();
        let mutant_run = LiveRunBinding::from_archive(
            synthetic_archive(6, "historical-v1", "mutant-only-tip"),
            std::collections::BTreeMap::from([(
                "mutant".to_owned(),
                original.requests["mutant"].clone(),
            )]),
            std::collections::BTreeMap::from([(
                "mutant".to_owned(),
                original.request_facts["mutant"].clone(),
            )]),
            std::collections::BTreeMap::from([(
                "mutant".to_owned(),
                original.responses["mutant"].clone(),
            )]),
        )
        .expect("the mutant-only run parses");
        let control_run = LiveRunBinding::from_archive(
            synthetic_archive(6, "historical-v1", "control-only-tip"),
            std::collections::BTreeMap::from([(
                "control".to_owned(),
                original.requests["control"].clone(),
            )]),
            std::collections::BTreeMap::from([("control".to_owned(), LiveRequestFact::Control)]),
            std::collections::BTreeMap::from([(
                "control".to_owned(),
                original.responses["control"].clone(),
            )]),
        )
        .expect("the control-only run parses");
        let runs = if control_run.run_id < mutant_run.run_id {
            vec![control_run, mutant_run]
        } else {
            vec![mutant_run, control_run]
        };
        assert_eq!(
            compare_run_bindings(&runs, &runs),
            Err(LiveSafetyReportRefusal::CrossRunRequestLink(
                "mutant".to_owned(),
                "control".to_owned(),
            )),
        );
    }

    #[test]
    fn the_declared_mutation_locator_is_checked_against_decoded_bytes() {
        let mut run = synthetic_mutation_run();
        let run_id = run.run_id.clone();
        let LiveRequestFact::Refusal { locator, .. } = run
            .request_facts
            .get_mut("mutant")
            .expect("the mutant fact is present")
        else {
            panic!("the mutant fact is a refusal");
        };
        *locator = LiveMutationLocator::WitnessItem {
            input_index: 0,
            item_index: 1,
        };
        assert_eq!(
            compare_run_bindings(&[run.clone()], &[run]),
            Err(LiveSafetyReportRefusal::MutationLocatorDiffers(
                run_id,
                "mutant".to_owned(),
            )),
        );
    }

    #[test]
    fn an_observation_cannot_alias_an_unrelated_row() {
        let run = synthetic_run();
        let LiveTargetResponse::Accepted { identity } = &run.responses["request-a"] else {
            panic!("the synthetic response is accepted");
        };
        let observation = |row: &'static str| LiveReportObservation::NativeAcceptance {
            row,
            run_id: run.run_id.clone(),
            request_id: "request-a".to_owned(),
            identity: *identity,
        };
        assert_eq!(
            validate_bound_observations(
                &[observation("row-a"), observation("row-b")],
                std::slice::from_ref(&run),
            ),
            Err(LiveSafetyReportRefusal::ObservationAliased(
                "request-a".to_owned(),
            )),
        );
    }

    #[test]
    fn run_response_and_observation_counts_cross_foot() {
        let run = synthetic_run();
        assert_eq!(
            validate_bound_observations(&[], std::slice::from_ref(&run)),
            Err(LiveSafetyReportRefusal::RunObservationCensusDiffers),
        );
    }

    #[test]
    fn a_transcript_pair_claim_is_not_proof_of_equality() {
        let run = synthetic_pair_run();
        let observation = pair_observation(&run, "transcript-says-the-pair-is-equal");
        assert_eq!(
            validate_bound_observations(&[observation], std::slice::from_ref(&run)),
            Err(LiveSafetyReportRefusal::PairedRelationNotRecomputed(
                "projection-equality-with-paired-explicit",
            )),
        );

        let recomputed = pair_observation(&run, VALIDATED_PAIR_RELATION);
        assert_eq!(
            validate_bound_observations(&[recomputed], std::slice::from_ref(&run)),
            Ok(1),
        );
    }

    #[test]
    fn a_matching_pair_claim_still_refuses_an_unrecomputed_projection() {
        let source = synthetic_pair_run();
        let mut request_facts = source.request_facts;
        let LiveRequestFact::Paired { projection, .. } = request_facts
            .get_mut("private")
            .expect("the private request fact is present")
        else {
            panic!("the private request fact is paired");
        };
        projection.semantic_input_amounts = vec![51];
        let forged = LiveRunBinding::from_archive(
            source.archive_bytes,
            source.requests,
            request_facts,
            source.responses,
        )
        .expect("the forged pair remains structurally complete");
        let observation = pair_observation(&forged, VALIDATED_PAIR_RELATION);

        assert_eq!(
            validate_bound_observations(&[observation], std::slice::from_ref(&forged)),
            Err(LiveSafetyReportRefusal::PairedRelationNotRecomputed(
                "projection-equality-with-paired-explicit",
            )),
        );
    }

    #[test]
    fn two_matching_lies_no_longer_validate() {
        // Red before the validator repair: today `compare_run_bindings`
        // checks one caller-authored run against another caller-authored
        // copy. These undecodable request bytes, invented identity and
        // matching expected copy therefore return `Ok(())` together.
        let mut invented = synthetic_run();
        invented
            .requests
            .insert("request-a".to_owned(), vec![0x01, 0x02, 0x03]);
        invented.responses.insert(
            "request-a".to_owned(),
            LiveTargetResponse::Accepted {
                identity: parsed_identity(
                    "75e823f7c5c70ddfbd9584f90f67298f2570907947829828066c7047b21b53b1",
                ),
            },
        );
        let offered = vec![invented.clone()];
        let matching_expected_lie = vec![invented];
        assert_eq!(offered, matching_expected_lie);
        assert_eq!(
            compare_run_bindings(&offered, &matching_expected_lie),
            Err(LiveSafetyReportRefusal::RequestDecodeRefused(
                offered[0].run_id.clone(),
                "request-a".to_owned(),
            )),
        );
    }

    #[test]
    fn every_bound_run_fact_has_a_typed_refusal() {
        let expected = vec![synthetic_run()];
        let run_id = expected[0].run_id.clone();

        let mut deployment = expected.clone();
        deployment[0].deployment.genesis_id[0] ^= 1;
        assert_eq!(
            compare_run_bindings(&deployment, &expected),
            Err(LiveSafetyReportRefusal::RunDeploymentDiffers(
                run_id.clone()
            )),
        );

        let mut request = expected.clone();
        request[0]
            .requests
            .insert("request-a".to_owned(), vec![0xff]);
        assert_eq!(
            compare_run_bindings(&request, &expected),
            Err(LiveSafetyReportRefusal::RequestDecodeRefused(
                run_id.clone(),
                "request-a".to_owned(),
            )),
        );

        let mut response = expected.clone();
        response[0].responses.insert(
            "request-a".to_owned(),
            LiveTargetResponse::Refused {
                observed_layer:
                    target_elements_conformance::protocol::ObservedOutcomeLayer::ScriptPathRejection,
                detail: "mutated".to_owned(),
                control_identity: parsed_identity(
                    "40cb6c4ee284ed38555a4840198c8130d1e2c3246b57b9d8b93842c3c6730029",
                ),
            },
        );
        assert_eq!(
            compare_run_bindings(&response, &expected),
            Err(LiveSafetyReportRefusal::RequestResponseShapeDiffers(
                run_id.clone(),
                "request-a".to_owned(),
            )),
        );

        let mut executor = expected.clone();
        executor[0].executor.node_version = "another-version".to_owned();
        assert_eq!(
            compare_run_bindings(&executor, &expected),
            Err(LiveSafetyReportRefusal::RunExecutorProvenanceDiffers(
                run_id
            )),
        );
    }

    #[test]
    fn mutating_a_fresh_identity_refuses_the_report() {
        let plan = derive_live_evidence_plan().expect("the evidence plan derives");
        let target = projection();
        let mut report = assemble_live_safety_report(&plan, target.clone()).expect("assembles");
        let observation = report
            .observations
            .iter_mut()
            .find_map(|observation| match observation {
                LiveReportObservation::NativeAcceptance { identity, .. } => Some(identity),
                _ => None,
            })
            .expect("the corpus carries a fresh acceptance");
        *observation =
            parsed_identity("75e823f7c5c70ddfbd9584f90f67298f2570907947829828066c7047b21b53b1");
        assert_eq!(
            validate_live_safety_report(report, &plan, &target),
            Err(LiveSafetyReportRefusal::ObservationsDiffer),
        );
    }

    #[test]
    fn no_target_evidence_fixture_renders_none_recorded() {
        assert_eq!(
            target_evidence_name(super::ObservationCensus::default()),
            "none-recorded",
        );
        let one = [LiveReportObservation::RecordedObservationUnbound {
            row: "fixture-row",
            observation: LiveRecordedObservation::NativeAcceptance {
                identity: parsed_identity(
                    "75e823f7c5c70ddfbd9584f90f67298f2570907947829828066c7047b21b53b1",
                ),
            },
        }];
        assert_eq!(
            target_evidence_name(super::ObservationCensus::from_observations(&one)),
            "recorded-unbound",
        );
    }

    #[test]
    fn the_private_restart_rows_use_the_schema_six_bound_forms() {
        let plan = derive_live_evidence_plan().expect("the evidence plan derives");
        let target = projection();
        let report = assemble_live_safety_report(&plan, target).expect("assembles");
        let rows = report
            .observations()
            .iter()
            .filter_map(|observation| match observation {
                LiveReportObservation::CompositeTwoAcceptance { row, .. }
                    if *row == "both-commitment-parity-forms" =>
                {
                    Some(*row)
                }
                LiveReportObservation::NativeAcceptance { row, .. }
                    if *row == "private-one-to-one" =>
                {
                    Some(*row)
                }
                _ => None,
            })
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(
            rows,
            std::collections::BTreeSet::from([
                "both-commitment-parity-forms",
                "private-one-to-one",
            ]),
        );
    }

    #[test]
    fn the_report_refuses_to_call_itself_complete_and_names_what_is_outstanding() {
        // §13.5's bar and the honest answer to it. A report that claimed
        // completeness while §15.1 has never run would be the one thing
        // this whole module exists to prevent.
        let plan = derive_live_evidence_plan().expect("the evidence plan derives");
        let target = projection();
        let report = assemble_live_safety_report(&plan, target.clone()).expect("assembles");
        assert_eq!(
            report.completeness(),
            LiveSafetyCompleteness::PartialRequiredRowsOutstanding,
        );
        let validated = validate_live_safety_report(report, &plan, &target).expect("validates");
        assert_ne!(validated.outstanding().len(), 0);
        // And NO blockers, where there used to be one. The report is
        // partial for the honest reasons: some rows need a future run and
        // historical target observations need transcript-grade binding,
        // not because a component they need is missing. The last blocker
        // left when the raw-bypass row was answered by a fact about
        // this workspace, its own gate being whether that path exists.
        //
        // Asserted as an equality rather than dropped, because "nothing
        // is blocked" is a claim this report makes and a reader is
        // entitled to see it checked rather than merely unstated.
        assert_eq!(validated.blockers().len(), 0);
    }

    #[test]
    fn a_report_claiming_a_completeness_its_census_does_not_support_is_refused() {
        // The recomputation made falsifiable: the validator does not read
        // the token, it derives one and compares.
        let plan = derive_live_evidence_plan().expect("the evidence plan derives");
        let target = projection();
        let mut report = assemble_live_safety_report(&plan, target.clone()).expect("assembles");
        report.completeness = LiveSafetyCompleteness::CompleteForTheRequiredMatrix;
        assert_eq!(
            validate_live_safety_report(report, &plan, &target),
            Err(LiveSafetyReportRefusal::CompletenessDiffers {
                reported: LiveSafetyCompleteness::CompleteForTheRequiredMatrix,
                recomputed: LiveSafetyCompleteness::PartialRequiredRowsOutstanding,
            }),
        );
    }

    #[test]
    fn schema_two_is_hard_rejected() {
        // Schema 2 is historical and never a legacy-validated route into
        // the evidence-bearing live report.
        let plan = derive_live_evidence_plan().expect("the evidence plan derives");
        let target = projection();
        let mut report = assemble_live_safety_report(&plan, target.clone()).expect("assembles");
        report.schema = 2;
        assert_eq!(
            validate_live_safety_report(report, &plan, &target),
            Err(LiveSafetyReportRefusal::UnsupportedSchema(2)),
        );
        assert_eq!(
            LiveSafetyReportRole::LiveTransferSafety.name(),
            "live-transfer-safety",
        );
    }

    #[test]
    fn schema_three_is_hard_rejected() {
        let plan = derive_live_evidence_plan().expect("the evidence plan derives");
        let target = projection();
        let mut report = assemble_live_safety_report(&plan, target.clone()).expect("assembles");
        report.schema = 3;
        assert_eq!(
            validate_live_safety_report(report, &plan, &target),
            Err(LiveSafetyReportRefusal::UnsupportedSchema(3)),
        );
    }

    #[test]
    fn non_live_schema_values_are_hard_rejected() {
        let plan = derive_live_evidence_plan().expect("the evidence plan derives");
        let target = projection();
        for schema in [4, 7, u32::MAX] {
            let mut report = assemble_live_safety_report(&plan, target.clone()).expect("assembles");
            report.schema = schema;
            assert_eq!(
                validate_live_safety_report(report, &plan, &target),
                Err(LiveSafetyReportRefusal::UnsupportedSchema(schema)),
            );
        }
    }

    #[test]
    fn the_canonical_bytes_carry_no_volatile_field() {
        // §13.5's exclusion, checked rather than argued. Each of the ten
        // fields is given a value nothing else in the workspace produces,
        // recorded in the diagnostic report, and the canonical bytes are
        // searched for every one of them. The renderer takes no
        // diagnostic argument, so a hit here would mean some *other*
        // carrier had smuggled a volatile value in.
        let plan = derive_live_evidence_plan().expect("the evidence plan derives");
        let target = projection();
        let report = assemble_live_safety_report(&plan, target.clone()).expect("assembles");
        let validated = validate_live_safety_report(report, &plan, &target).expect("validates");
        let canonical = render_live_safety_report(&validated);

        let mut diagnostics = LiveSafetyDiagnostics::default();
        let mut markers = Vec::with_capacity(VolatileField::ALL.len());
        for (index, field) in VolatileField::ALL.iter().enumerate() {
            let marker = format!("volatile-marker-{index}-{field:?}");
            diagnostics = diagnostics.with(*field, marker.clone());
            markers.push(marker);
        }
        assert_eq!(diagnostics.fields().len(), 10);

        for marker in &markers {
            assert!(
                !canonical.contains(marker.as_str()),
                "the canonical bytes carry {marker}",
            );
            // And the diagnostic report really does carry it, so the
            // check above is not passing because nothing was recorded.
            assert!(diagnostics.render().contains(marker.as_str()));
        }

        // The two renderings are different documents, and the canonical
        // one says nothing about being timed.
        assert!(!canonical.contains("wall"));
        assert!(!canonical.contains("elapsed"));
        assert!(diagnostics.render().contains("canonical false"));
    }

    #[test]
    fn the_canonical_bytes_are_stable_across_renderings() {
        // The property timing would have destroyed: two renderings of the
        // same validated report are the same bytes.
        let plan = derive_live_evidence_plan().expect("the evidence plan derives");
        let target = projection();
        let report = assemble_live_safety_report(&plan, target.clone()).expect("assembles");
        let validated = validate_live_safety_report(report, &plan, &target).expect("validates");
        assert_eq!(
            render_live_safety_report(&validated),
            validated.canonical_bytes,
        );
    }

    #[test]
    fn report_disclosures_refuse_with_the_exact_row_and_key() {
        let plan = derive_live_evidence_plan().expect("the evidence plan derives");
        let target = projection();
        let report = assemble_live_safety_report(&plan, target.clone()).expect("assembles");
        let requirements = report.report_layer_requirements.clone();
        let validated = validate_live_safety_report(report, &plan, &target).expect("validates");
        let rendered = render_live_safety_report(&validated);
        for (key, row) in [
            ("sponsor_amount", "report-publishes-sponsor-amount"),
            ("sponsor_opening", "report-publishes-sponsor-opening"),
        ] {
            let leaked = format!("{rendered}{key} 1000\n");
            assert_eq!(
                validate_report_disclosures(&leaked, &requirements),
                Err(LiveSafetyReportRefusal::ReportDisclosurePresent {
                    row,
                    key: key.to_owned(),
                }),
            );
        }
    }

    #[test]
    fn the_scoreboard_partitions_the_matrix_by_section() {
        let plan = derive_live_evidence_plan().expect("the evidence plan derives");
        let target = projection();
        let report = assemble_live_safety_report(&plan, target.clone()).expect("assembles");
        let validated = validate_live_safety_report(report, &plan, &target).expect("validates");
        let board = section_scoreboard(&validated);
        assert_eq!(board.len(), LiveSafetySection::ALL.len());
        let total: usize = board.values().map(|(rows, _, _)| rows).sum();
        assert_eq!(total, crate::live_safety::row_count());

        // Neither positive table waits on a component that does not
        // exist, and the scoreboard exists to say how far each one has
        // actually got rather than to round it away. Both numbers are
        // asserted rather than bounded, because a scoreboard that said
        // "some" would let the next row in without a run.
        //
        // All target-derived rows now preserve their historical observation
        // without counting it as validated evidence. The only positive row
        // still answered is the independently recomputed determinism row.
        let (explicit_rows, explicit_answered, explicit_blocked) =
            board[&LiveSafetySection::PositiveExplicit];
        assert_eq!(explicit_answered, 0, "the explicit table's answered count");
        assert_eq!(explicit_blocked, 0);
        assert_ne!(explicit_rows, 0);

        let (private_rows, private_answered, private_blocked) =
            board[&LiveSafetySection::PositivePrivate];
        assert_eq!(private_answered, 1, "the private table's answered count");
        assert_eq!(private_blocked, 0);
        assert_eq!(private_rows, 10);
        assert_eq!(LiveSafetyPolarity::ALL.len(), 2);
    }

    #[test]
    fn the_lifecycle_status_stays_incomplete() {
        // §17.1: transfer is implemented, burn and redemption are not,
        // and no report may say the candidate is release-complete.
        let plan = derive_live_evidence_plan().expect("the evidence plan derives");
        let target = projection();
        let report = assemble_live_safety_report(&plan, target).expect("assembles");
        assert!(!report.lifecycle().release_complete());
        assert_eq!(report.lifecycle().outstanding().len(), 2);
        assert_eq!(report.lifecycle().implemented().len(), 1);
    }
}
