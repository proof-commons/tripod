//! A separately validated STATE constructor-continuity report.
//!
//! Assembly reads exact-byte projections; validation recomputes every claim from
//! those projections and independently supplied envelope premises. Neither a
//! successful host comparison nor a caller-stated branch authenticates freshness
//! or accepted readback. Safety, continuity, history, recovery and resources answer
//! separate questions; none satisfies another by implication.
//!
//! Canonical bytes exclude all eleven categories in `MaturityVolatileField`.
//! Timing is emitted separately by `MaturityReportTiming`; its inherited safety
//! timing label is unchanged and never enters this report's canonical bytes.
//!
//! Key order is header, acceptance routes, residuals, census, completeness,
//! recomputed items, then source entries in input order. Within an entry: source
//! bindings, transaction sizes, predecessor and successor constructors, leastness,
//! static results, semantic results, tweak evidence, controls, class and quantifier.
//! Each binary operand uses lowercase hexadecimal; text provenance is hex encoded.

use std::collections::BTreeSet;
use std::fmt::Write as _;

use linker::{CandidateDeploymentIdentity, StateLinkRefusal};
use realization::{AnnouncementLeadBounds, Cycle, StateMetadata, StateRepresentationNonce};
use tapscript::{
    StateConstructorRefusal, StateControlRecipe, StateFieldCommitment, state_metadata_leaf_program,
};
use transaction::operator_right::BranchContext;
use transaction::taproot::leaf_hash;

use crate::maturity_closure::closure_target;
use crate::maturity_continuity::{
    MaturityByteComparison, MaturityByteSource, MaturityCheckedPrefix,
    MaturityConstructorProjection, MaturityControlObservation, MaturityControlPaths,
    MaturityFieldComparison, MaturityFundedPredecessor, MaturityLeastnessResidual,
    MaturityNonceLeastness, MaturityTweakEvidence, ValidatedMaturityContinuity,
};
use crate::maturity_evidence::{
    MaturityExecutorProvenanceExpectation, MaturityGuaranteeQuantifier, MaturityObservationClass,
    MaturityTargetBinding,
};
use crate::maturity_native::{MaturityAcceptanceObligation, MaturityAcceptanceRoute};
use crate::maturity_report::MaturitySafetyCompleteness;

/// Schema understood by this canonical continuity report.
pub const MATURITY_CONTINUITY_REPORT_SCHEMA: u32 = 1;

/// A continuity report has its own evidence role.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MaturityContinuityReportRole {
    /// Host constructor comparisons over exact submitted bytes.
    StateConstructorContinuity,
}
impl MaturityContinuityReportRole {
    /// Stable wire spelling, distinct from the safety report.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::StateConstructorContinuity => "state-constructor-continuity",
        }
    }
}

/// The constructor side of a report entry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MaturityContinuityReportSide {
    /// The funded input's witnessed constructor.
    Predecessor,
    /// The submitted output's reconstructed constructor.
    Successor,
}
impl MaturityContinuityReportSide {
    /// Stable side label.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Predecessor => "predecessor",
            Self::Successor => "successor",
        }
    }
}

/// Limits which successful byte comparisons do not discharge.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MaturityContinuityReportResidual {
    /// Output commitment inequality does not identify its cause.
    CommitmentMismatchWithoutCauseAttribution,
    /// Bounded host search does not exclude later admissible nonces.
    HostSearchOnly,
    /// Descriptor equality compares one retained tree with itself.
    SingleRetainedTreeComparison,
    /// Branch operands are caller-stated context, not a freshness proof.
    CallerStatedBranchWithoutFreshness,
}
impl MaturityContinuityReportResidual {
    /// Every residual, in canonical order.
    pub const ALL: &'static [Self] = &[
        Self::CommitmentMismatchWithoutCauseAttribution,
        Self::HostSearchOnly,
        Self::SingleRetainedTreeComparison,
        Self::CallerStatedBranchWithoutFreshness,
    ];
    /// Stable residual spelling.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::CommitmentMismatchWithoutCauseAttribution => {
                "commitment-mismatch-without-cause-attribution"
            }
            Self::HostSearchOnly => "host-search-only",
            Self::SingleRetainedTreeComparison => "single-retained-tree-comparison",
            Self::CallerStatedBranchWithoutFreshness => "caller-stated-branch-without-freshness",
        }
    }
}

/// Sizes recomputed from the submitted transaction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MaturityContinuityByteFacts {
    submitted: usize,
    stripped: usize,
    weight: u64,
    witness_widths: Vec<usize>,
}
impl MaturityContinuityByteFacts {
    /// Number of exact submitted bytes.
    #[must_use]
    pub const fn submitted(&self) -> usize {
        self.submitted
    }
    /// Number of bytes without witnesses.
    #[must_use]
    pub const fn stripped(&self) -> usize {
        self.stripped
    }
    /// Decoded transaction weight.
    #[must_use]
    pub const fn weight(&self) -> u64 {
        self.weight
    }
    /// Widths of the predecessor witness items.
    #[must_use]
    pub fn witness_widths(&self) -> &[usize] {
        &self.witness_widths
    }
}

/// One witnessed constructor, without substituting the host minimum.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MaturityContinuityConstructorFacts {
    semantic: StateMetadata,
    nonce: StateRepresentationNonce,
    metadata_bytes: Vec<u8>,
    metadata_leaf_bytes: Vec<u8>,
    metadata_leaf_hash: [u8; 32],
    metadata_leaf_agrees: bool,
    static_root: [u8; 32],
    outer_root: [u8; 32],
    internal_key: [u8; 32],
    tweak_digest: [u8; 32],
    output_key: [u8; 32],
    parity: bool,
    output_program: Vec<u8>,
    control_recipe: StateControlRecipe,
    field_commitments: Vec<StateFieldCommitment>,
}
impl MaturityContinuityConstructorFacts {
    /// Semantic tuple at this side.
    #[must_use]
    pub const fn semantic(&self) -> StateMetadata {
        self.semantic
    }
    /// Witnessed representation nonce.
    #[must_use]
    pub const fn nonce(&self) -> StateRepresentationNonce {
        self.nonce
    }
    /// Canonical semantic and representation encoding.
    #[must_use]
    pub fn metadata_bytes(&self) -> &[u8] {
        &self.metadata_bytes
    }
    /// Publicly rebuilt metadata script at the witnessed nonce.
    #[must_use]
    pub fn metadata_leaf_bytes(&self) -> &[u8] {
        &self.metadata_leaf_bytes
    }
    /// Witnessed candidate's independently checked metadata digest.
    #[must_use]
    pub const fn metadata_leaf_hash(&self) -> &[u8; 32] {
        &self.metadata_leaf_hash
    }
    /// Whether rebuilding the script reproduces the checked digest.
    #[must_use]
    pub const fn metadata_leaf_agrees(&self) -> bool {
        self.metadata_leaf_agrees
    }
    /// Retained static subtree root.
    #[must_use]
    pub const fn static_root(&self) -> &[u8; 32] {
        &self.static_root
    }
    /// Metadata and static branch root.
    #[must_use]
    pub const fn outer_root(&self) -> &[u8; 32] {
        &self.outer_root
    }
    /// Constructor internal key.
    #[must_use]
    pub const fn internal_key(&self) -> &[u8; 32] {
        &self.internal_key
    }
    /// Tagged tweak digest.
    #[must_use]
    pub const fn tweak_digest(&self) -> &[u8; 32] {
        &self.tweak_digest
    }
    /// Output x coordinate.
    #[must_use]
    pub const fn output_key(&self) -> &[u8; 32] {
        &self.output_key
    }
    /// Output point oddness.
    #[must_use]
    pub const fn parity(&self) -> bool {
        self.parity
    }
    /// Output witness program.
    #[must_use]
    pub fn output_program(&self) -> &[u8] {
        &self.output_program
    }
    /// Announcement control reconstruction recipe.
    #[must_use]
    pub const fn control_recipe(&self) -> &StateControlRecipe {
        &self.control_recipe
    }
    /// Semantic field commitments and byte ranges.
    #[must_use]
    pub fn field_commitments(&self) -> &[StateFieldCommitment] {
        &self.field_commitments
    }
}

/// Bounded host leastness beside the witnessed representation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MaturityContinuityLeastnessFacts {
    witnessed_nonce: StateRepresentationNonce,
    selected_nonce: StateRepresentationNonce,
    host_least: bool,
    rejected_count: usize,
    reconstructed_facts_match: Option<bool>,
    residual: MaturityLeastnessResidual,
    checked: MaturityCheckedPrefix,
}
impl MaturityContinuityLeastnessFacts {
    /// Nonce named by submitted bytes.
    #[must_use]
    pub const fn witnessed_nonce(&self) -> StateRepresentationNonce {
        self.witnessed_nonce
    }
    /// First admissible host candidate.
    #[must_use]
    pub const fn selected_nonce(&self) -> StateRepresentationNonce {
        self.selected_nonce
    }
    /// Whether witnessed and host-selected nonces agree.
    #[must_use]
    pub const fn host_least(&self) -> bool {
        self.host_least
    }
    /// Number of rejected lower candidates.
    #[must_use]
    pub const fn rejected_count(&self) -> usize {
        self.rejected_count
    }
    /// Constructor agreement when witnessed and selected nonces agree.
    #[must_use]
    pub const fn reconstructed_facts_match(&self) -> Option<bool> {
        self.reconstructed_facts_match
    }
    /// Bounded host-search limitation.
    #[must_use]
    pub const fn residual(&self) -> MaturityLeastnessResidual {
        self.residual
    }
    /// Every checked rejection, selection and witnessed candidate.
    #[must_use]
    pub const fn checked(&self) -> &MaturityCheckedPrefix {
        &self.checked
    }
}

/// Four independent static comparisons.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MaturityContinuityStaticFacts {
    root_binding: MaturityByteComparison,
    control_binding: MaturityByteComparison,
    descriptor_result: Result<(), Box<StateLinkRefusal>>,
    constructor_result: Result<(), Box<StateConstructorRefusal>>,
}
impl MaturityContinuityStaticFacts {
    /// Submitted static root against the retained root.
    #[must_use]
    pub const fn root_binding(&self) -> &MaturityByteComparison {
        &self.root_binding
    }
    /// Submitted predecessor control against its reconstruction.
    #[must_use]
    pub const fn control_binding(&self) -> &MaturityByteComparison {
        &self.control_binding
    }
    /// Whole retained-descriptor equality result.
    #[must_use]
    pub const fn descriptor_result(&self) -> &Result<(), Box<StateLinkRefusal>> {
        &self.descriptor_result
    }
    /// Constructor continuity result.
    #[must_use]
    pub const fn constructor_result(&self) -> &Result<(), Box<StateConstructorRefusal>> {
        &self.constructor_result
    }
}

/// Transition agreement kept apart from reconstruction consistency.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MaturityContinuitySemanticFacts {
    transition_agrees: bool,
    reconstruction_agrees: Option<bool>,
    expected_successor: StateMetadata,
    requested_cycle: Cycle,
    bounds: AnnouncementLeadBounds,
    predecessor_fields: Vec<MaturityFieldComparison>,
    successor_fields: Option<Vec<MaturityFieldComparison>>,
}
impl MaturityContinuitySemanticFacts {
    /// Whether the decoded predecessor transitions to the expected successor.
    #[must_use]
    pub const fn transition_agrees(&self) -> bool {
        self.transition_agrees
    }
    /// Whether reconstructed fields match the expected tuple.
    #[must_use]
    pub const fn reconstruction_agrees(&self) -> Option<bool> {
        self.reconstruction_agrees
    }
    /// Realization-produced successor tuple.
    #[must_use]
    pub const fn expected_successor(&self) -> StateMetadata {
        self.expected_successor
    }
    /// Cycle requested in submitted bytes.
    #[must_use]
    pub const fn requested_cycle(&self) -> Cycle {
        self.requested_cycle
    }
    /// Lead bounds from the retained recipe.
    #[must_use]
    pub const fn bounds(&self) -> AnnouncementLeadBounds {
        self.bounds
    }
    /// Predecessor fields against expected successor fields.
    #[must_use]
    pub fn predecessor_fields(&self) -> &[MaturityFieldComparison] {
        &self.predecessor_fields
    }
    /// Reconstructed successor against expected fields.
    #[must_use]
    pub const fn successor_fields(&self) -> &Option<Vec<MaturityFieldComparison>> {
        &self.successor_fields
    }
}

/// One exact-byte source and its independently retained comparison results.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MaturityContinuityReportEntry {
    position: usize,
    source: MaturityByteSource,
    byte_identity: [u8; 32],
    funded: MaturityFundedPredecessor,
    branch: BranchContext,
    identity: CandidateDeploymentIdentity,
    bytes: MaturityContinuityByteFacts,
    predecessor: MaturityContinuityConstructorFacts,
    successor: MaturityContinuityConstructorFacts,
    predecessor_leastness: MaturityContinuityLeastnessFacts,
    successor_leastness: MaturityContinuityLeastnessFacts,
    static_facts: MaturityContinuityStaticFacts,
    semantic_facts: MaturityContinuitySemanticFacts,
    predecessor_tweak: MaturityTweakEvidence,
    successor_tweak: MaturityTweakEvidence,
    controls: MaturityControlPaths,
    observation_class: MaturityObservationClass,
    quantifier: MaturityGuaranteeQuantifier,
}
impl MaturityContinuityReportEntry {
    /// Source position in the assembly input.
    #[must_use]
    pub const fn position(&self) -> usize {
        self.position
    }
    /// Caller-stated byte provenance.
    #[must_use]
    pub const fn source(&self) -> &MaturityByteSource {
        &self.source
    }
    /// SHA-256 identity of the exact submitted bytes.
    #[must_use]
    pub const fn byte_identity(&self) -> &[u8; 32] {
        &self.byte_identity
    }
    /// Supplied funding facts bound by the projector.
    #[must_use]
    pub const fn funded(&self) -> &MaturityFundedPredecessor {
        &self.funded
    }
    /// Caller-stated branch identity and checkpoint.
    #[must_use]
    pub const fn branch(&self) -> BranchContext {
        self.branch
    }
    /// Deployment identity checked against the retained recipe.
    #[must_use]
    pub const fn identity(&self) -> &CandidateDeploymentIdentity {
        &self.identity
    }
    /// Measured transaction sizes.
    #[must_use]
    pub const fn bytes(&self) -> &MaturityContinuityByteFacts {
        &self.bytes
    }
    /// Input constructor at its witnessed nonce.
    #[must_use]
    pub const fn predecessor(&self) -> &MaturityContinuityConstructorFacts {
        &self.predecessor
    }
    /// Output constructor at its witnessed nonce.
    #[must_use]
    pub const fn successor(&self) -> &MaturityContinuityConstructorFacts {
        &self.successor
    }
    /// Input bounded host search.
    #[must_use]
    pub const fn predecessor_leastness(&self) -> &MaturityContinuityLeastnessFacts {
        &self.predecessor_leastness
    }
    /// Output bounded host search.
    #[must_use]
    pub const fn successor_leastness(&self) -> &MaturityContinuityLeastnessFacts {
        &self.successor_leastness
    }
    /// Independent static comparisons.
    #[must_use]
    pub const fn static_facts(&self) -> &MaturityContinuityStaticFacts {
        &self.static_facts
    }
    /// Independent semantic comparisons.
    #[must_use]
    pub const fn semantic_facts(&self) -> &MaturityContinuitySemanticFacts {
        &self.semantic_facts
    }
    /// Input hash and point comparisons with their premises.
    #[must_use]
    pub const fn predecessor_tweak(&self) -> &MaturityTweakEvidence {
        &self.predecessor_tweak
    }
    /// Output hash and point comparisons with their premises.
    #[must_use]
    pub const fn successor_tweak(&self) -> &MaturityTweakEvidence {
        &self.successor_tweak
    }
    /// Submitted input control and derived output control.
    #[must_use]
    pub const fn controls(&self) -> &MaturityControlPaths {
        &self.controls
    }
    /// Intended transition class supplied by the validated projection.
    #[must_use]
    pub const fn observation_class(&self) -> MaturityObservationClass {
        self.observation_class
    }
    /// Two admitted classes conditional on no class-three observation.
    #[must_use]
    pub const fn quantifier(&self) -> MaturityGuaranteeQuantifier {
        self.quantifier
    }
}

/// Counts over the exact comparison inventory.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MaturityContinuityReportCensus {
    sources: usize,
    comparisons: usize,
    agreements: usize,
}
impl MaturityContinuityReportCensus {
    /// Number of supplied projections.
    #[must_use]
    pub const fn sources(&self) -> usize {
        self.sources
    }
    /// Number of individual comparison results.
    #[must_use]
    pub const fn comparisons(&self) -> usize {
        self.comparisons
    }
    /// Number of true or successful results in that inventory.
    #[must_use]
    pub const fn agreements(&self) -> usize {
        self.agreements
    }
}

/// Unvalidated assembly; private fields alone establish no claim.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MaturityContinuityReport {
    schema: u32,
    role: MaturityContinuityReportRole,
    binding: MaturityTargetBinding,
    provenance: MaturityExecutorProvenanceExpectation,
    entries: Vec<MaturityContinuityReportEntry>,
    acceptance: MaturityAcceptanceObligation,
    residuals: Vec<MaturityContinuityReportResidual>,
    census: MaturityContinuityReportCensus,
    completeness: MaturitySafetyCompleteness,
}
impl MaturityContinuityReport {
    /// Schema stated by assembly.
    #[must_use]
    pub const fn schema(&self) -> u32 {
        self.schema
    }
    /// Continuity-specific evidence role.
    #[must_use]
    pub const fn role(&self) -> MaturityContinuityReportRole {
        self.role
    }
    /// Caller-stated target and deployment premise.
    #[must_use]
    pub const fn binding(&self) -> &MaturityTargetBinding {
        &self.binding
    }
    /// Caller-stated executor premise, unauthenticated by projections.
    #[must_use]
    pub const fn provenance(&self) -> &MaturityExecutorProvenanceExpectation {
        &self.provenance
    }
    /// Entries in supplied-source order.
    #[must_use]
    pub fn entries(&self) -> &[MaturityContinuityReportEntry] {
        &self.entries
    }
    /// Unmet accepted-readback condition with both routes.
    #[must_use]
    pub const fn acceptance(&self) -> MaturityAcceptanceObligation {
        self.acceptance
    }
    /// Limits retained beside all successful comparisons.
    #[must_use]
    pub fn residuals(&self) -> &[MaturityContinuityReportResidual] {
        &self.residuals
    }
    /// Recomputed source and comparison counts.
    #[must_use]
    pub const fn census(&self) -> &MaturityContinuityReportCensus {
        &self.census
    }
    /// Partial while accepted readback is absent.
    #[must_use]
    pub const fn completeness(&self) -> MaturitySafetyCompleteness {
        self.completeness
    }
}

/// Report whose claims were recomputed against independent inputs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedMaturityContinuityReport {
    report: MaturityContinuityReport,
    recomputed_items: BTreeSet<MaturityContinuityRecomputedItem>,
}
impl ValidatedMaturityContinuityReport {
    /// The checked report.
    #[must_use]
    pub const fn report(&self) -> &MaturityContinuityReport {
        &self.report
    }
    /// Items recorded only after their comparisons succeeded.
    #[must_use]
    pub const fn recomputed_items(&self) -> &BTreeSet<MaturityContinuityRecomputedItem> {
        &self.recomputed_items
    }
}

/// One complete claim recomputed from projections or independent envelope inputs.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MaturityContinuityRecomputedItem {
    /// Recompute schema.
    Schema,
    /// Recompute role.
    Role,
    /// Recompute source count.
    SourceCount,
    /// Recompute source order.
    SourceOrder,
    /// Recompute target binding.
    TargetBinding,
    /// Recompute executor provenance expectation.
    ExecutorProvenanceExpectation,
    /// Recompute source class.
    SourceClass,
    /// Recompute byte identity.
    ByteIdentity,
    /// Recompute funded predecessor.
    FundedPredecessor,
    /// Recompute branch context.
    BranchContext,
    /// Recompute deployment identity.
    DeploymentIdentity,
    /// Recompute submitted length.
    SubmittedLength,
    /// Recompute stripped length.
    StrippedLength,
    /// Recompute transaction weight.
    TransactionWeight,
    /// Recompute witness widths.
    WitnessWidths,
    /// Recompute predecessor semantic metadata.
    PredecessorSemanticMetadata,
    /// Recompute predecessor nonce.
    PredecessorNonce,
    /// Recompute predecessor metadata bytes.
    PredecessorMetadataBytes,
    /// Recompute predecessor metadata leaf bytes.
    PredecessorMetadataLeafBytes,
    /// Recompute predecessor metadata leaf hash.
    PredecessorMetadataLeafHash,
    /// Recompute predecessor metadata leaf agreement.
    PredecessorMetadataLeafAgreement,
    /// Recompute predecessor static root.
    PredecessorStaticRoot,
    /// Recompute predecessor outer root.
    PredecessorOuterRoot,
    /// Recompute predecessor internal key.
    PredecessorInternalKey,
    /// Recompute predecessor tweak digest.
    PredecessorTweakDigest,
    /// Recompute predecessor output key.
    PredecessorOutputKey,
    /// Recompute predecessor parity.
    PredecessorParity,
    /// Recompute predecessor output program.
    PredecessorOutputProgram,
    /// Recompute predecessor control recipe.
    PredecessorControlRecipe,
    /// Recompute predecessor field commitments.
    PredecessorFieldCommitments,
    /// Recompute predecessor witnessed nonce.
    PredecessorWitnessedNonce,
    /// Recompute predecessor selected nonce.
    PredecessorSelectedNonce,
    /// Recompute predecessor host least.
    PredecessorHostLeast,
    /// Recompute predecessor rejected count.
    PredecessorRejectedCount,
    /// Recompute predecessor reconstructed facts match.
    PredecessorReconstructedFactsMatch,
    /// Recompute predecessor leastness residual.
    PredecessorLeastnessResidual,
    /// Recompute predecessor checked prefix.
    PredecessorCheckedPrefix,
    /// Recompute successor semantic metadata.
    SuccessorSemanticMetadata,
    /// Recompute successor nonce.
    SuccessorNonce,
    /// Recompute successor metadata bytes.
    SuccessorMetadataBytes,
    /// Recompute successor metadata leaf bytes.
    SuccessorMetadataLeafBytes,
    /// Recompute successor metadata leaf hash.
    SuccessorMetadataLeafHash,
    /// Recompute successor metadata leaf agreement.
    SuccessorMetadataLeafAgreement,
    /// Recompute successor static root.
    SuccessorStaticRoot,
    /// Recompute successor outer root.
    SuccessorOuterRoot,
    /// Recompute successor internal key.
    SuccessorInternalKey,
    /// Recompute successor tweak digest.
    SuccessorTweakDigest,
    /// Recompute successor output key.
    SuccessorOutputKey,
    /// Recompute successor parity.
    SuccessorParity,
    /// Recompute successor output program.
    SuccessorOutputProgram,
    /// Recompute successor control recipe.
    SuccessorControlRecipe,
    /// Recompute successor field commitments.
    SuccessorFieldCommitments,
    /// Recompute successor witnessed nonce.
    SuccessorWitnessedNonce,
    /// Recompute successor selected nonce.
    SuccessorSelectedNonce,
    /// Recompute successor host least.
    SuccessorHostLeast,
    /// Recompute successor rejected count.
    SuccessorRejectedCount,
    /// Recompute successor reconstructed facts match.
    SuccessorReconstructedFactsMatch,
    /// Recompute successor leastness residual.
    SuccessorLeastnessResidual,
    /// Recompute successor checked prefix.
    SuccessorCheckedPrefix,
    /// Recompute static root binding.
    StaticRootBinding,
    /// Recompute static control binding.
    StaticControlBinding,
    /// Recompute descriptor result.
    DescriptorResult,
    /// Recompute constructor result.
    ConstructorResult,
    /// Recompute transition agreement.
    TransitionAgreement,
    /// Recompute reconstruction agreement.
    ReconstructionAgreement,
    /// Recompute expected successor.
    ExpectedSuccessor,
    /// Recompute requested cycle.
    RequestedCycle,
    /// Recompute lead bounds.
    LeadBounds,
    /// Recompute predecessor field comparisons.
    PredecessorFieldComparisons,
    /// Recompute successor field comparisons.
    SuccessorFieldComparisons,
    /// Recompute predecessor tweak side.
    PredecessorTweakSide,
    /// Recompute predecessor tweak internal.
    PredecessorTweakInternal,
    /// Recompute predecessor tweak metadata.
    PredecessorTweakMetadata,
    /// Recompute predecessor tweak branch.
    PredecessorTweakBranch,
    /// Recompute predecessor tweak digest comparison.
    PredecessorTweakDigestComparison,
    /// Recompute predecessor tweak key.
    PredecessorTweakKey,
    /// Recompute predecessor tweak parity.
    PredecessorTweakParity,
    /// Recompute predecessor tweak program.
    PredecessorTweakProgram,
    /// Recompute predecessor tweak premises.
    PredecessorTweakPremises,
    /// Recompute successor tweak side.
    SuccessorTweakSide,
    /// Recompute successor tweak internal.
    SuccessorTweakInternal,
    /// Recompute successor tweak metadata.
    SuccessorTweakMetadata,
    /// Recompute successor tweak branch.
    SuccessorTweakBranch,
    /// Recompute successor tweak digest comparison.
    SuccessorTweakDigestComparison,
    /// Recompute successor tweak key.
    SuccessorTweakKey,
    /// Recompute successor tweak parity.
    SuccessorTweakParity,
    /// Recompute successor tweak program.
    SuccessorTweakProgram,
    /// Recompute successor tweak premises.
    SuccessorTweakPremises,
    /// Recompute control recipes.
    ControlRecipes,
    /// Recompute control submitted binding.
    ControlSubmittedBinding,
    /// Recompute control derived bytes.
    ControlDerivedBytes,
    /// Recompute control observation kinds.
    ControlObservationKinds,
    /// Recompute control lengths.
    ControlLengths,
    /// Recompute control inner.
    ControlInner,
    /// Recompute control outer.
    ControlOuter,
    /// Recompute control first byte relation.
    ControlFirstByteRelation,
    /// Recompute observation class.
    ObservationClass,
    /// Recompute guarantee quantifier.
    GuaranteeQuantifier,
    /// Recompute acceptance disposition.
    AcceptanceDisposition,
    /// Recompute attribution residual.
    AttributionResidual,
    /// Recompute host search residual.
    HostSearchResidual,
    /// Recompute retained tree residual.
    RetainedTreeResidual,
    /// Recompute branch context residual.
    BranchContextResidual,
    /// Recompute census sources.
    CensusSources,
    /// Recompute census comparisons.
    CensusComparisons,
    /// Recompute census agreements.
    CensusAgreements,
    /// Recompute completeness.
    Completeness,
}
impl MaturityContinuityRecomputedItem {
    /// Exhaustive validation order; each item is recorded after all sources pass.
    pub const ALL: &'static [Self] = &[
        Self::Schema,
        Self::Role,
        Self::SourceCount,
        Self::SourceOrder,
        Self::TargetBinding,
        Self::ExecutorProvenanceExpectation,
        Self::SourceClass,
        Self::ByteIdentity,
        Self::FundedPredecessor,
        Self::BranchContext,
        Self::DeploymentIdentity,
        Self::SubmittedLength,
        Self::StrippedLength,
        Self::TransactionWeight,
        Self::WitnessWidths,
        Self::PredecessorSemanticMetadata,
        Self::PredecessorNonce,
        Self::PredecessorMetadataBytes,
        Self::PredecessorMetadataLeafBytes,
        Self::PredecessorMetadataLeafHash,
        Self::PredecessorMetadataLeafAgreement,
        Self::PredecessorStaticRoot,
        Self::PredecessorOuterRoot,
        Self::PredecessorInternalKey,
        Self::PredecessorTweakDigest,
        Self::PredecessorOutputKey,
        Self::PredecessorParity,
        Self::PredecessorOutputProgram,
        Self::PredecessorControlRecipe,
        Self::PredecessorFieldCommitments,
        Self::PredecessorWitnessedNonce,
        Self::PredecessorSelectedNonce,
        Self::PredecessorHostLeast,
        Self::PredecessorRejectedCount,
        Self::PredecessorReconstructedFactsMatch,
        Self::PredecessorLeastnessResidual,
        Self::PredecessorCheckedPrefix,
        Self::SuccessorSemanticMetadata,
        Self::SuccessorNonce,
        Self::SuccessorMetadataBytes,
        Self::SuccessorMetadataLeafBytes,
        Self::SuccessorMetadataLeafHash,
        Self::SuccessorMetadataLeafAgreement,
        Self::SuccessorStaticRoot,
        Self::SuccessorOuterRoot,
        Self::SuccessorInternalKey,
        Self::SuccessorTweakDigest,
        Self::SuccessorOutputKey,
        Self::SuccessorParity,
        Self::SuccessorOutputProgram,
        Self::SuccessorControlRecipe,
        Self::SuccessorFieldCommitments,
        Self::SuccessorWitnessedNonce,
        Self::SuccessorSelectedNonce,
        Self::SuccessorHostLeast,
        Self::SuccessorRejectedCount,
        Self::SuccessorReconstructedFactsMatch,
        Self::SuccessorLeastnessResidual,
        Self::SuccessorCheckedPrefix,
        Self::StaticRootBinding,
        Self::StaticControlBinding,
        Self::DescriptorResult,
        Self::ConstructorResult,
        Self::TransitionAgreement,
        Self::ReconstructionAgreement,
        Self::ExpectedSuccessor,
        Self::RequestedCycle,
        Self::LeadBounds,
        Self::PredecessorFieldComparisons,
        Self::SuccessorFieldComparisons,
        Self::PredecessorTweakSide,
        Self::PredecessorTweakInternal,
        Self::PredecessorTweakMetadata,
        Self::PredecessorTweakBranch,
        Self::PredecessorTweakDigestComparison,
        Self::PredecessorTweakKey,
        Self::PredecessorTweakParity,
        Self::PredecessorTweakProgram,
        Self::PredecessorTweakPremises,
        Self::SuccessorTweakSide,
        Self::SuccessorTweakInternal,
        Self::SuccessorTweakMetadata,
        Self::SuccessorTweakBranch,
        Self::SuccessorTweakDigestComparison,
        Self::SuccessorTweakKey,
        Self::SuccessorTweakParity,
        Self::SuccessorTweakProgram,
        Self::SuccessorTweakPremises,
        Self::ControlRecipes,
        Self::ControlSubmittedBinding,
        Self::ControlDerivedBytes,
        Self::ControlObservationKinds,
        Self::ControlLengths,
        Self::ControlInner,
        Self::ControlOuter,
        Self::ControlFirstByteRelation,
        Self::ObservationClass,
        Self::GuaranteeQuantifier,
        Self::AcceptanceDisposition,
        Self::AttributionResidual,
        Self::HostSearchResidual,
        Self::RetainedTreeResidual,
        Self::BranchContextResidual,
        Self::CensusSources,
        Self::CensusComparisons,
        Self::CensusAgreements,
        Self::Completeness,
    ];
    /// Stable spelling used by the canonical recomputation inventory.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Schema => "schema",
            Self::Role => "role",
            Self::SourceCount => "source-count",
            Self::SourceOrder => "source-order",
            Self::TargetBinding => "target-binding",
            Self::ExecutorProvenanceExpectation => "executor-provenance-expectation",
            Self::SourceClass => "source-class",
            Self::ByteIdentity => "byte-identity",
            Self::FundedPredecessor => "funded-predecessor",
            Self::BranchContext => "branch-context",
            Self::DeploymentIdentity => "deployment-identity",
            Self::SubmittedLength => "submitted-length",
            Self::StrippedLength => "stripped-length",
            Self::TransactionWeight => "transaction-weight",
            Self::WitnessWidths => "witness-widths",
            Self::PredecessorSemanticMetadata => "predecessor-semantic-metadata",
            Self::PredecessorNonce => "predecessor-nonce",
            Self::PredecessorMetadataBytes => "predecessor-metadata-bytes",
            Self::PredecessorMetadataLeafBytes => "predecessor-metadata-leaf-bytes",
            Self::PredecessorMetadataLeafHash => "predecessor-metadata-leaf-hash",
            Self::PredecessorMetadataLeafAgreement => "predecessor-metadata-leaf-agreement",
            Self::PredecessorStaticRoot => "predecessor-static-root",
            Self::PredecessorOuterRoot => "predecessor-outer-root",
            Self::PredecessorInternalKey => "predecessor-internal-key",
            Self::PredecessorTweakDigest => "predecessor-tweak-digest",
            Self::PredecessorOutputKey => "predecessor-output-key",
            Self::PredecessorParity => "predecessor-parity",
            Self::PredecessorOutputProgram => "predecessor-output-program",
            Self::PredecessorControlRecipe => "predecessor-control-recipe",
            Self::PredecessorFieldCommitments => "predecessor-field-commitments",
            Self::PredecessorWitnessedNonce => "predecessor-witnessed-nonce",
            Self::PredecessorSelectedNonce => "predecessor-selected-nonce",
            Self::PredecessorHostLeast => "predecessor-host-least",
            Self::PredecessorRejectedCount => "predecessor-rejected-count",
            Self::PredecessorReconstructedFactsMatch => "predecessor-reconstructed-facts-match",
            Self::PredecessorLeastnessResidual => "predecessor-leastness-residual",
            Self::PredecessorCheckedPrefix => "predecessor-checked-prefix",
            Self::SuccessorSemanticMetadata => "successor-semantic-metadata",
            Self::SuccessorNonce => "successor-nonce",
            Self::SuccessorMetadataBytes => "successor-metadata-bytes",
            Self::SuccessorMetadataLeafBytes => "successor-metadata-leaf-bytes",
            Self::SuccessorMetadataLeafHash => "successor-metadata-leaf-hash",
            Self::SuccessorMetadataLeafAgreement => "successor-metadata-leaf-agreement",
            Self::SuccessorStaticRoot => "successor-static-root",
            Self::SuccessorOuterRoot => "successor-outer-root",
            Self::SuccessorInternalKey => "successor-internal-key",
            Self::SuccessorTweakDigest => "successor-tweak-digest",
            Self::SuccessorOutputKey => "successor-output-key",
            Self::SuccessorParity => "successor-parity",
            Self::SuccessorOutputProgram => "successor-output-program",
            Self::SuccessorControlRecipe => "successor-control-recipe",
            Self::SuccessorFieldCommitments => "successor-field-commitments",
            Self::SuccessorWitnessedNonce => "successor-witnessed-nonce",
            Self::SuccessorSelectedNonce => "successor-selected-nonce",
            Self::SuccessorHostLeast => "successor-host-least",
            Self::SuccessorRejectedCount => "successor-rejected-count",
            Self::SuccessorReconstructedFactsMatch => "successor-reconstructed-facts-match",
            Self::SuccessorLeastnessResidual => "successor-leastness-residual",
            Self::SuccessorCheckedPrefix => "successor-checked-prefix",
            Self::StaticRootBinding => "static-root-binding",
            Self::StaticControlBinding => "static-control-binding",
            Self::DescriptorResult => "descriptor-result",
            Self::ConstructorResult => "constructor-result",
            Self::TransitionAgreement => "transition-agreement",
            Self::ReconstructionAgreement => "reconstruction-agreement",
            Self::ExpectedSuccessor => "expected-successor",
            Self::RequestedCycle => "requested-cycle",
            Self::LeadBounds => "lead-bounds",
            Self::PredecessorFieldComparisons => "predecessor-field-comparisons",
            Self::SuccessorFieldComparisons => "successor-field-comparisons",
            remaining => remaining.tail_name(),
        }
    }

    const fn tail_name(self) -> &'static str {
        match self {
            Self::PredecessorTweakSide => "predecessor-tweak-side",
            Self::PredecessorTweakInternal => "predecessor-tweak-internal",
            Self::PredecessorTweakMetadata => "predecessor-tweak-metadata",
            Self::PredecessorTweakBranch => "predecessor-tweak-branch",
            Self::PredecessorTweakDigestComparison => "predecessor-tweak-digest-comparison",
            Self::PredecessorTweakKey => "predecessor-tweak-key",
            Self::PredecessorTweakParity => "predecessor-tweak-parity",
            Self::PredecessorTweakProgram => "predecessor-tweak-program",
            Self::PredecessorTweakPremises => "predecessor-tweak-premises",
            Self::SuccessorTweakSide => "successor-tweak-side",
            Self::SuccessorTweakInternal => "successor-tweak-internal",
            Self::SuccessorTweakMetadata => "successor-tweak-metadata",
            Self::SuccessorTweakBranch => "successor-tweak-branch",
            Self::SuccessorTweakDigestComparison => "successor-tweak-digest-comparison",
            Self::SuccessorTweakKey => "successor-tweak-key",
            Self::SuccessorTweakParity => "successor-tweak-parity",
            Self::SuccessorTweakProgram => "successor-tweak-program",
            Self::SuccessorTweakPremises => "successor-tweak-premises",
            Self::ControlRecipes => "control-recipes",
            Self::ControlSubmittedBinding => "control-submitted-binding",
            Self::ControlDerivedBytes => "control-derived-bytes",
            Self::ControlObservationKinds => "control-observation-kinds",
            Self::ControlLengths => "control-lengths",
            Self::ControlInner => "control-inner",
            Self::ControlOuter => "control-outer",
            Self::ControlFirstByteRelation => "control-first-byte-relation",
            Self::ObservationClass => "observation-class",
            Self::GuaranteeQuantifier => "guarantee-quantifier",
            Self::AcceptanceDisposition => "acceptance-disposition",
            Self::AttributionResidual => "attribution-residual",
            Self::HostSearchResidual => "host-search-residual",
            Self::RetainedTreeResidual => "retained-tree-residual",
            Self::BranchContextResidual => "branch-context-residual",
            Self::CensusSources => "census-sources",
            Self::CensusComparisons => "census-comparisons",
            Self::CensusAgreements => "census-agreements",
            Self::Completeness => "completeness",
            _ => "",
        }
    }
}

/// First disagreement, including structural failures before payload comparisons.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MaturityContinuityReportRefusal {
    /// The schema is not understood.
    UnsupportedSchema {
        /// Schema carried by the report.
        stated: u32,
    },
    /// The report and supplied sources have different lengths.
    SourceCountDiffers {
        /// Number carried by the report.
        stated: usize,
        /// Number supplied for independent validation.
        recomputed: usize,
    },
    /// Entries or matching source identities have been reordered.
    SourceOrderDiffers {
        /// First position that disagrees.
        position: usize,
    },
    /// A recomputed claim differs.
    ItemDiffers {
        /// The first failed comparison.
        item: MaturityContinuityRecomputedItem,
        /// Source position, or none for a report-wide claim.
        source_index: Option<usize>,
    },
}
impl MaturityContinuityReportRefusal {
    /// The recomputation which refused, including the structural preflight.
    #[must_use]
    pub const fn failed_item(&self) -> MaturityContinuityRecomputedItem {
        match self {
            Self::UnsupportedSchema { .. } => MaturityContinuityRecomputedItem::Schema,
            Self::SourceCountDiffers { .. } => MaturityContinuityRecomputedItem::SourceCount,
            Self::SourceOrderDiffers { .. } => MaturityContinuityRecomputedItem::SourceOrder,
            Self::ItemDiffers { item, .. } => *item,
        }
    }
}

/// Rebuild a script at the witnessed nonce; a failed builder cannot validate.
///
/// Both builders already succeeded for a validated projection. Retaining false
/// on an unexpected failure keeps assembly total without turning that failure
/// into a successful comparison or substituting the host-selected representation.
fn metadata_leaf(
    projection: &MaturityConstructorProjection,
    checked: &MaturityCheckedPrefix,
) -> (Vec<u8>, bool) {
    let Ok(target) = closure_target() else {
        return (Vec::new(), false);
    };
    let Ok(program) = state_metadata_leaf_program(&target, projection.encoded_metadata()) else {
        return (Vec::new(), false);
    };
    let bytes = program.encode(&target);
    let agrees =
        leaf_hash(target.definition().leaf_version(), &bytes) == *checked.witnessed().leaf_digest();
    (bytes, agrees)
}

fn constructor_facts(
    projection: &MaturityConstructorProjection,
    checked: &MaturityCheckedPrefix,
) -> MaturityContinuityConstructorFacts {
    let (metadata_leaf_bytes, metadata_leaf_agrees) = metadata_leaf(projection, checked);
    MaturityContinuityConstructorFacts {
        semantic: projection.encoded_metadata().semantic,
        nonce: projection.nonce(),
        metadata_bytes: projection.metadata_bytes().to_vec(),
        metadata_leaf_bytes,
        metadata_leaf_hash: *checked.witnessed().leaf_digest(),
        metadata_leaf_agrees,
        static_root: *projection.static_subtree().root(),
        outer_root: *projection.merkle_root(),
        internal_key: projection.control_recipe().internal_key,
        tweak_digest: *projection.tweak_hash(),
        output_key: *projection.output_key(),
        parity: projection.parity(),
        output_program: projection.output_program().to_vec(),
        control_recipe: projection.control_recipe().clone(),
        field_commitments: projection.field_commitments().to_vec(),
    }
}

fn leastness_facts(
    least: &MaturityNonceLeastness,
    checked: &MaturityCheckedPrefix,
) -> MaturityContinuityLeastnessFacts {
    MaturityContinuityLeastnessFacts {
        witnessed_nonce: least.witnessed_nonce(),
        selected_nonce: least.evidence().selected,
        host_least: least.is_host_least(),
        rejected_count: least.evidence().rejected.len(),
        reconstructed_facts_match: least.reconstructed_facts_match(),
        residual: checked.residual(),
        checked: checked.clone(),
    }
}

fn entry_of(
    position: usize,
    source: &ValidatedMaturityContinuity,
) -> MaturityContinuityReportEntry {
    let static_comparison = source.static_comparison();
    let semantic = source.semantic_comparison();
    MaturityContinuityReportEntry {
        position,
        source: source.source().clone(),
        byte_identity: *source.byte_identity(),
        funded: source.funded().clone(),
        branch: source.branch(),
        identity: source.identity().clone(),
        bytes: MaturityContinuityByteFacts {
            submitted: source.submitted_bytes().len(),
            stripped: source.transaction().encode_without_witness().len(),
            weight: source.transaction().weight(),
            witness_widths: source
                .transaction()
                .witnesses()
                .iter()
                .flat_map(|witness| witness.stack().iter().map(Vec::len))
                .collect(),
        },
        predecessor: constructor_facts(source.predecessor(), source.checked_predecessor()),
        successor: constructor_facts(source.successor(), source.checked_successor()),
        predecessor_leastness: leastness_facts(
            source.predecessor_leastness(),
            source.checked_predecessor(),
        ),
        successor_leastness: leastness_facts(
            source.successor_leastness(),
            source.checked_successor(),
        ),
        static_facts: MaturityContinuityStaticFacts {
            root_binding: static_comparison.static_root().clone(),
            control_binding: static_comparison.control_block().clone(),
            descriptor_result: static_comparison.descriptor_result().clone(),
            constructor_result: static_comparison.constructor_result().clone(),
        },
        semantic_facts: MaturityContinuitySemanticFacts {
            transition_agrees: semantic.transition_agrees(),
            reconstruction_agrees: semantic.reconstruction_agrees(),
            expected_successor: *semantic.expected_successor(),
            requested_cycle: semantic.requested_cycle(),
            bounds: source.bounds(),
            predecessor_fields: semantic.predecessor_against_expected().to_vec(),
            successor_fields: semantic.reconstructed_against_expected().map(<[_]>::to_vec),
        },
        predecessor_tweak: source.predecessor_tweak().clone(),
        successor_tweak: source.successor_tweak().clone(),
        controls: source.controls().clone(),
        observation_class: source.observation_class(),
        quantifier: source.quantifier(),
    }
}

/// Exact inventory: four static, up to two semantic summaries, individual
/// semantic fields, fourteen tweak results and five control-byte results.
fn comparison_results(source: &ValidatedMaturityContinuity) -> Vec<bool> {
    let static_comparison = source.static_comparison();
    let semantic = source.semantic_comparison();
    let controls = source.controls();
    let mut results = vec![
        static_comparison.static_root().agrees(),
        static_comparison.control_block().agrees(),
        static_comparison.descriptor_result().is_ok(),
        static_comparison.constructor_result().is_ok(),
        semantic.transition_agrees(),
    ];
    results.extend(semantic.reconstruction_agrees());
    results.extend(
        semantic
            .predecessor_against_expected()
            .iter()
            .map(MaturityFieldComparison::agrees),
    );
    if let Some(fields) = semantic.reconstructed_against_expected() {
        results.extend(fields.iter().map(MaturityFieldComparison::agrees));
    }
    for evidence in [source.predecessor_tweak(), source.successor_tweak()] {
        results.extend(tweak_comparisons(evidence).map(|(_, comparison)| comparison.agrees()));
    }
    results.extend([controls.observed().agrees(), controls.inner().agrees()]);
    results.extend(controls.outer().iter().map(MaturityByteComparison::agrees));
    let (actual, predicted) = controls.first_byte_relation();
    results.push(actual == predicted);
    results
}

fn census_of(sources: &[&ValidatedMaturityContinuity]) -> MaturityContinuityReportCensus {
    let mut comparisons = 0;
    let mut agreements = 0;
    for source in sources {
        let results = comparison_results(source);
        comparisons += results.len();
        agreements += results.iter().filter(|result| **result).count();
    }
    MaturityContinuityReportCensus {
        sources: sources.len(),
        comparisons,
        agreements,
    }
}

fn acceptance_of(sources: &[&ValidatedMaturityContinuity]) -> MaturityAcceptanceObligation {
    sources.first().map_or(
        MaturityAcceptanceObligation::Outstanding {
            routes: [
                MaturityAcceptanceRoute::RelayWitnessRestructure,
                MaturityAcceptanceRoute::BlockLayerSubmissionSubject,
            ],
        },
        |source| source.acceptance_obligation(),
    )
}

/// Assemble claims solely from validated projections and caller-stated premises.
///
/// Binding and executor provenance are envelope premises, not authenticated
/// conclusions of the projector. Validation requires their independent originals.
/// All possible input lengths remain partial: this input vocabulary contains no
/// accepted readback with target-computed identity.
#[must_use]
pub fn assemble_maturity_continuity_report(
    sources: &[&ValidatedMaturityContinuity],
    binding: MaturityTargetBinding,
    provenance: MaturityExecutorProvenanceExpectation,
) -> MaturityContinuityReport {
    MaturityContinuityReport {
        schema: MATURITY_CONTINUITY_REPORT_SCHEMA,
        role: MaturityContinuityReportRole::StateConstructorContinuity,
        binding,
        provenance,
        entries: sources
            .iter()
            .enumerate()
            .map(|(position, source)| entry_of(position, source))
            .collect(),
        acceptance: acceptance_of(sources),
        residuals: MaturityContinuityReportResidual::ALL.to_vec(),
        census: census_of(sources),
        completeness: MaturitySafetyCompleteness::PartialRequiredRowsOutstanding,
    }
}

fn preflight(
    report: &MaturityContinuityReport,
    sources: &[&ValidatedMaturityContinuity],
) -> Result<(), MaturityContinuityReportRefusal> {
    if report.schema != MATURITY_CONTINUITY_REPORT_SCHEMA {
        return Err(MaturityContinuityReportRefusal::UnsupportedSchema {
            stated: report.schema,
        });
    }
    if report.entries.len() != sources.len() {
        return Err(MaturityContinuityReportRefusal::SourceCountDiffers {
            stated: report.entries.len(),
            recomputed: sources.len(),
        });
    }
    for (position, (entry, source)) in report.entries.iter().zip(sources).enumerate() {
        if entry.position != position
            || (entry.byte_identity != *source.byte_identity()
                && report.entries.iter().enumerate().any(|(other, candidate)| {
                    other != position && candidate.byte_identity == *source.byte_identity()
                }))
        {
            return Err(MaturityContinuityReportRefusal::SourceOrderDiffers { position });
        }
    }
    Ok(())
}

fn envelope_agrees(
    item: MaturityContinuityRecomputedItem,
    report: &MaturityContinuityReport,
    sources: &[&ValidatedMaturityContinuity],
    binding: &MaturityTargetBinding,
    provenance: &MaturityExecutorProvenanceExpectation,
    census: &MaturityContinuityReportCensus,
) -> Option<bool> {
    use MaturityContinuityRecomputedItem as Item;
    Some(match item {
        Item::Schema => report.schema == MATURITY_CONTINUITY_REPORT_SCHEMA,
        Item::Role => report.role == MaturityContinuityReportRole::StateConstructorContinuity,
        Item::SourceCount => report.entries.len() == sources.len(),
        Item::SourceOrder => report.entries.iter().enumerate().all(|(position, entry)| entry.position == position),
        Item::TargetBinding => &report.binding == binding,
        Item::ExecutorProvenanceExpectation => &report.provenance == provenance,
        Item::AcceptanceDisposition => report.acceptance == acceptance_of(sources)
            && sources.iter().all(|source| source.acceptance_obligation() == report.acceptance),
        Item::AttributionResidual => report.residuals.len() == MaturityContinuityReportResidual::ALL.len()
            && report.residuals.first() == MaturityContinuityReportResidual::ALL.first()
            && sources.iter().all(|source| matches!(source.successor_attribution(), crate::maturity_continuity::MaturitySuccessorAttribution::CommitmentMismatchWithoutCauseAttribution)),
        Item::HostSearchResidual => report.residuals.get(1) == MaturityContinuityReportResidual::ALL.get(1)
            && sources.iter().all(|source| matches!(source.checked_predecessor().residual(), MaturityLeastnessResidual::HostSearchOnly)
                && matches!(source.checked_successor().residual(), MaturityLeastnessResidual::HostSearchOnly)),
        Item::RetainedTreeResidual => report.residuals.get(2) == MaturityContinuityReportResidual::ALL.get(2),
        Item::BranchContextResidual => report.residuals.get(3) == MaturityContinuityReportResidual::ALL.get(3),
        Item::CensusSources => report.census.sources == census.sources,
        Item::CensusComparisons => report.census.comparisons == census.comparisons,
        Item::CensusAgreements => report.census.agreements == census.agreements,
        Item::Completeness => report.completeness == MaturitySafetyCompleteness::PartialRequiredRowsOutstanding,
        _ => return None,
    })
}

fn entry_agrees_0(
    item: MaturityContinuityRecomputedItem,
    stated: &MaturityContinuityReportEntry,
    expected: &MaturityContinuityReportEntry,
) -> Option<bool> {
    use MaturityContinuityRecomputedItem as Item;
    Some(match item {
        Item::SourceClass => stated.source == expected.source,
        Item::ByteIdentity => stated.byte_identity == expected.byte_identity,
        Item::FundedPredecessor => stated.funded == expected.funded,
        Item::BranchContext => stated.branch == expected.branch,
        Item::DeploymentIdentity => stated.identity == expected.identity,
        Item::SubmittedLength => stated.bytes.submitted == expected.bytes.submitted,
        Item::StrippedLength => stated.bytes.stripped == expected.bytes.stripped,
        Item::TransactionWeight => stated.bytes.weight == expected.bytes.weight,
        Item::WitnessWidths => stated.bytes.witness_widths == expected.bytes.witness_widths,
        Item::PredecessorSemanticMetadata => {
            stated.predecessor.semantic == expected.predecessor.semantic
        }
        Item::PredecessorNonce => stated.predecessor.nonce == expected.predecessor.nonce,
        Item::PredecessorMetadataBytes => {
            stated.predecessor.metadata_bytes == expected.predecessor.metadata_bytes
        }
        Item::PredecessorMetadataLeafBytes => {
            stated.predecessor.metadata_leaf_bytes == expected.predecessor.metadata_leaf_bytes
        }
        Item::PredecessorMetadataLeafHash => {
            stated.predecessor.metadata_leaf_hash == expected.predecessor.metadata_leaf_hash
        }
        Item::PredecessorMetadataLeafAgreement => {
            stated.predecessor.metadata_leaf_agrees == expected.predecessor.metadata_leaf_agrees
                && expected.predecessor.metadata_leaf_agrees
        }
        Item::PredecessorStaticRoot => {
            stated.predecessor.static_root == expected.predecessor.static_root
        }
        _ => return None,
    })
}

fn entry_agrees_1(
    item: MaturityContinuityRecomputedItem,
    stated: &MaturityContinuityReportEntry,
    expected: &MaturityContinuityReportEntry,
) -> Option<bool> {
    use MaturityContinuityRecomputedItem as Item;
    Some(match item {
        Item::PredecessorOuterRoot => {
            stated.predecessor.outer_root == expected.predecessor.outer_root
        }
        Item::PredecessorInternalKey => {
            stated.predecessor.internal_key == expected.predecessor.internal_key
        }
        Item::PredecessorTweakDigest => {
            stated.predecessor.tweak_digest == expected.predecessor.tweak_digest
        }
        Item::PredecessorOutputKey => {
            stated.predecessor.output_key == expected.predecessor.output_key
        }
        Item::PredecessorParity => stated.predecessor.parity == expected.predecessor.parity,
        Item::PredecessorOutputProgram => {
            stated.predecessor.output_program == expected.predecessor.output_program
        }
        Item::PredecessorControlRecipe => {
            stated.predecessor.control_recipe == expected.predecessor.control_recipe
        }
        Item::PredecessorFieldCommitments => {
            stated.predecessor.field_commitments == expected.predecessor.field_commitments
        }
        Item::PredecessorWitnessedNonce => {
            stated.predecessor_leastness.witnessed_nonce
                == expected.predecessor_leastness.witnessed_nonce
        }
        Item::PredecessorSelectedNonce => {
            stated.predecessor_leastness.selected_nonce
                == expected.predecessor_leastness.selected_nonce
        }
        Item::PredecessorHostLeast => {
            stated.predecessor_leastness.host_least == expected.predecessor_leastness.host_least
        }
        Item::PredecessorRejectedCount => {
            stated.predecessor_leastness.rejected_count
                == expected.predecessor_leastness.rejected_count
        }
        Item::PredecessorReconstructedFactsMatch => {
            stated.predecessor_leastness.reconstructed_facts_match
                == expected.predecessor_leastness.reconstructed_facts_match
        }
        Item::PredecessorLeastnessResidual => {
            stated.predecessor_leastness.residual == expected.predecessor_leastness.residual
        }
        Item::PredecessorCheckedPrefix => {
            stated.predecessor_leastness.checked == expected.predecessor_leastness.checked
        }
        Item::SuccessorSemanticMetadata => stated.successor.semantic == expected.successor.semantic,
        _ => return None,
    })
}

fn entry_agrees_2(
    item: MaturityContinuityRecomputedItem,
    stated: &MaturityContinuityReportEntry,
    expected: &MaturityContinuityReportEntry,
) -> Option<bool> {
    use MaturityContinuityRecomputedItem as Item;
    Some(match item {
        Item::SuccessorNonce => stated.successor.nonce == expected.successor.nonce,
        Item::SuccessorMetadataBytes => {
            stated.successor.metadata_bytes == expected.successor.metadata_bytes
        }
        Item::SuccessorMetadataLeafBytes => {
            stated.successor.metadata_leaf_bytes == expected.successor.metadata_leaf_bytes
        }
        Item::SuccessorMetadataLeafHash => {
            stated.successor.metadata_leaf_hash == expected.successor.metadata_leaf_hash
        }
        Item::SuccessorMetadataLeafAgreement => {
            stated.successor.metadata_leaf_agrees == expected.successor.metadata_leaf_agrees
                && expected.successor.metadata_leaf_agrees
        }
        Item::SuccessorStaticRoot => stated.successor.static_root == expected.successor.static_root,
        Item::SuccessorOuterRoot => stated.successor.outer_root == expected.successor.outer_root,
        Item::SuccessorInternalKey => {
            stated.successor.internal_key == expected.successor.internal_key
        }
        Item::SuccessorTweakDigest => {
            stated.successor.tweak_digest == expected.successor.tweak_digest
        }
        Item::SuccessorOutputKey => stated.successor.output_key == expected.successor.output_key,
        Item::SuccessorParity => stated.successor.parity == expected.successor.parity,
        Item::SuccessorOutputProgram => {
            stated.successor.output_program == expected.successor.output_program
        }
        Item::SuccessorControlRecipe => {
            stated.successor.control_recipe == expected.successor.control_recipe
        }
        Item::SuccessorFieldCommitments => {
            stated.successor.field_commitments == expected.successor.field_commitments
        }
        Item::SuccessorWitnessedNonce => {
            stated.successor_leastness.witnessed_nonce
                == expected.successor_leastness.witnessed_nonce
        }
        Item::SuccessorSelectedNonce => {
            stated.successor_leastness.selected_nonce == expected.successor_leastness.selected_nonce
        }
        _ => return None,
    })
}

fn entry_agrees_3(
    item: MaturityContinuityRecomputedItem,
    stated: &MaturityContinuityReportEntry,
    expected: &MaturityContinuityReportEntry,
) -> Option<bool> {
    use MaturityContinuityRecomputedItem as Item;
    Some(match item {
        Item::SuccessorHostLeast => {
            stated.successor_leastness.host_least == expected.successor_leastness.host_least
        }
        Item::SuccessorRejectedCount => {
            stated.successor_leastness.rejected_count == expected.successor_leastness.rejected_count
        }
        Item::SuccessorReconstructedFactsMatch => {
            stated.successor_leastness.reconstructed_facts_match
                == expected.successor_leastness.reconstructed_facts_match
        }
        Item::SuccessorLeastnessResidual => {
            stated.successor_leastness.residual == expected.successor_leastness.residual
        }
        Item::SuccessorCheckedPrefix => {
            stated.successor_leastness.checked == expected.successor_leastness.checked
        }
        Item::StaticRootBinding => {
            stated.static_facts.root_binding == expected.static_facts.root_binding
        }
        Item::StaticControlBinding => {
            stated.static_facts.control_binding == expected.static_facts.control_binding
        }
        Item::DescriptorResult => {
            stated.static_facts.descriptor_result == expected.static_facts.descriptor_result
        }
        Item::ConstructorResult => {
            stated.static_facts.constructor_result == expected.static_facts.constructor_result
        }
        Item::TransitionAgreement => {
            stated.semantic_facts.transition_agrees == expected.semantic_facts.transition_agrees
        }
        Item::ReconstructionAgreement => {
            stated.semantic_facts.reconstruction_agrees
                == expected.semantic_facts.reconstruction_agrees
        }
        Item::ExpectedSuccessor => {
            stated.semantic_facts.expected_successor == expected.semantic_facts.expected_successor
        }
        Item::RequestedCycle => {
            stated.semantic_facts.requested_cycle == expected.semantic_facts.requested_cycle
        }
        Item::LeadBounds => stated.semantic_facts.bounds == expected.semantic_facts.bounds,
        Item::PredecessorFieldComparisons => {
            stated.semantic_facts.predecessor_fields == expected.semantic_facts.predecessor_fields
        }
        Item::SuccessorFieldComparisons => {
            stated.semantic_facts.successor_fields == expected.semantic_facts.successor_fields
        }
        _ => return None,
    })
}

fn entry_agrees_4(
    item: MaturityContinuityRecomputedItem,
    stated: &MaturityContinuityReportEntry,
    expected: &MaturityContinuityReportEntry,
) -> Option<bool> {
    use MaturityContinuityRecomputedItem as Item;
    Some(match item {
        Item::PredecessorTweakSide => {
            stated.predecessor_tweak.side() == expected.predecessor_tweak.side()
        }
        Item::PredecessorTweakInternal => {
            stated.predecessor_tweak.internal() == expected.predecessor_tweak.internal()
        }
        Item::PredecessorTweakMetadata => {
            stated.predecessor_tweak.metadata() == expected.predecessor_tweak.metadata()
        }
        Item::PredecessorTweakBranch => {
            stated.predecessor_tweak.branch() == expected.predecessor_tweak.branch()
        }
        Item::PredecessorTweakDigestComparison => {
            stated.predecessor_tweak.digest() == expected.predecessor_tweak.digest()
        }
        Item::PredecessorTweakKey => {
            stated.predecessor_tweak.key() == expected.predecessor_tweak.key()
        }
        Item::PredecessorTweakParity => {
            stated.predecessor_tweak.oddness() == expected.predecessor_tweak.oddness()
        }
        Item::PredecessorTweakProgram => {
            stated.predecessor_tweak.program() == expected.predecessor_tweak.program()
        }
        Item::PredecessorTweakPremises => {
            stated.predecessor_tweak.premises() == expected.predecessor_tweak.premises()
        }
        Item::SuccessorTweakSide => {
            stated.successor_tweak.side() == expected.successor_tweak.side()
        }
        Item::SuccessorTweakInternal => {
            stated.successor_tweak.internal() == expected.successor_tweak.internal()
        }
        Item::SuccessorTweakMetadata => {
            stated.successor_tweak.metadata() == expected.successor_tweak.metadata()
        }
        Item::SuccessorTweakBranch => {
            stated.successor_tweak.branch() == expected.successor_tweak.branch()
        }
        Item::SuccessorTweakDigestComparison => {
            stated.successor_tweak.digest() == expected.successor_tweak.digest()
        }
        Item::SuccessorTweakKey => stated.successor_tweak.key() == expected.successor_tweak.key(),
        Item::SuccessorTweakParity => {
            stated.successor_tweak.oddness() == expected.successor_tweak.oddness()
        }
        _ => return None,
    })
}

fn entry_agrees_5(
    item: MaturityContinuityRecomputedItem,
    stated: &MaturityContinuityReportEntry,
    expected: &MaturityContinuityReportEntry,
) -> Option<bool> {
    use MaturityContinuityRecomputedItem as Item;
    Some(match item {
        Item::SuccessorTweakProgram => {
            stated.successor_tweak.program() == expected.successor_tweak.program()
        }
        Item::SuccessorTweakPremises => {
            stated.successor_tweak.premises() == expected.successor_tweak.premises()
        }
        Item::ControlRecipes => {
            stated.controls.before() == expected.controls.before()
                && stated.controls.after() == expected.controls.after()
        }
        Item::ControlSubmittedBinding => stated.controls.observed() == expected.controls.observed(),
        Item::ControlDerivedBytes => stated.controls.derived() == expected.controls.derived(),
        Item::ControlObservationKinds => {
            stated.controls.observations() == expected.controls.observations()
        }
        Item::ControlLengths => stated.controls.lengths() == expected.controls.lengths(),
        Item::ControlInner => stated.controls.inner() == expected.controls.inner(),
        Item::ControlOuter => stated.controls.outer() == expected.controls.outer(),
        Item::ControlFirstByteRelation => {
            stated.controls.first_byte_relation() == expected.controls.first_byte_relation()
        }
        Item::ObservationClass => stated.observation_class == expected.observation_class,
        Item::GuaranteeQuantifier => stated.quantifier == expected.quantifier,
        _ => return None,
    })
}

fn entry_agrees(
    item: MaturityContinuityRecomputedItem,
    stated: &MaturityContinuityReportEntry,
    expected: &MaturityContinuityReportEntry,
) -> bool {
    entry_agrees_0(item, stated, expected)
        .or_else(|| entry_agrees_1(item, stated, expected))
        .or_else(|| entry_agrees_2(item, stated, expected))
        .or_else(|| entry_agrees_3(item, stated, expected))
        .or_else(|| entry_agrees_4(item, stated, expected))
        .or_else(|| entry_agrees_5(item, stated, expected))
        .is_some_and(|agrees| agrees)
}

/// Recompute every claim against projections and independent envelope originals.
///
/// The envelope's equality is caller-premise consistency, not authentication.
/// Schema, count and order refuse before any payload comparison. Each item is
/// marked only after every source comparison succeeds; rendering cannot consume
/// a partially checked value.
///
/// # Errors
///
/// Returns the first structural refusal or disagreeing census item.
pub fn validate_maturity_continuity_report(
    report: &MaturityContinuityReport,
    sources: &[&ValidatedMaturityContinuity],
    binding: &MaturityTargetBinding,
    provenance: &MaturityExecutorProvenanceExpectation,
) -> Result<ValidatedMaturityContinuityReport, MaturityContinuityReportRefusal> {
    preflight(report, sources)?;
    let expected: Vec<_> = sources
        .iter()
        .enumerate()
        .map(|(position, source)| entry_of(position, source))
        .collect();
    let census = census_of(sources);
    let mut recomputed_items = BTreeSet::new();
    for &item in MaturityContinuityRecomputedItem::ALL {
        if let Some(agrees) = envelope_agrees(item, report, sources, binding, provenance, &census) {
            if !agrees {
                return Err(MaturityContinuityReportRefusal::ItemDiffers {
                    item,
                    source_index: None,
                });
            }
        } else {
            for (source_index, (stated, derived)) in
                report.entries.iter().zip(&expected).enumerate()
            {
                if !entry_agrees(item, stated, derived) {
                    return Err(MaturityContinuityReportRefusal::ItemDiffers {
                        item,
                        source_index: Some(source_index),
                    });
                }
            }
        }
        recomputed_items.insert(item);
    }
    Ok(ValidatedMaturityContinuityReport {
        report: report.clone(),
        recomputed_items,
    })
}

fn hex(bytes: &[u8]) -> String {
    let mut text = String::new();
    for byte in bytes {
        let _ = write!(text, "{byte:02x}");
    }
    text
}

const fn tweak_comparisons(
    evidence: &MaturityTweakEvidence,
) -> [(&'static str, &MaturityByteComparison); 7] {
    [
        ("internal", evidence.internal()),
        ("metadata", evidence.metadata()),
        ("branch", evidence.branch()),
        ("digest", evidence.digest()),
        ("key", evidence.key()),
        ("parity", evidence.oddness()),
        ("program", evidence.program()),
    ]
}

fn render_comparison(text: &mut String, key: &str, comparison: &MaturityByteComparison) {
    let _ = writeln!(
        text,
        "{key} {} {} {}",
        hex(comparison.witnessed()),
        hex(comparison.reconstructed()),
        comparison.agrees()
    );
}

fn render_metadata(text: &mut String, key: &str, metadata: StateMetadata) {
    let maturity = match metadata.maturity {
        realization::Maturity::Unannounced => "unannounced".to_owned(),
        realization::Maturity::Complete => "complete".to_owned(),
        realization::Maturity::Announced { cycle } => format!("announced:{}", cycle.get()),
    };
    let _ = writeln!(
        text,
        "{key} {} {} {} {} {} {maturity}",
        metadata.omega.get(),
        metadata.y_l.get(),
        metadata.y_t.get(),
        metadata.q.get(),
        metadata.cycle.get()
    );
}

fn render_recipe(text: &mut String, key: &str, recipe: &StateControlRecipe) {
    let _ = write!(
        text,
        "{key} {:?} {} {} {} {}",
        recipe.role,
        recipe.leaf_version.get(),
        recipe.parity,
        hex(&recipe.internal_key),
        hex(&recipe.executing_leaf_hash)
    );
    for sibling in &recipe.siblings {
        let _ = write!(text, " {}", hex(sibling));
    }
    text.push('\n');
}

fn render_fields(text: &mut String, key: &str, fields: &[StateFieldCommitment]) {
    for field in fields {
        let _ = writeln!(
            text,
            "{key} {:?} {} {} {} {}",
            field.side,
            field.field.name(),
            field.range.start,
            field.range.end,
            hex(&field.bytes)
        );
    }
}

fn render_constructor(
    text: &mut String,
    side: MaturityContinuityReportSide,
    facts: &MaturityContinuityConstructorFacts,
) {
    let side = side.name();
    render_metadata(text, &format!("{side}_semantic"), facts.semantic);
    let _ = writeln!(text, "{side}_nonce {}", facts.nonce.get());
    for (key, bytes) in [
        ("metadata_bytes", facts.metadata_bytes.as_slice()),
        ("metadata_leaf_bytes", facts.metadata_leaf_bytes.as_slice()),
        ("metadata_leaf_hash", facts.metadata_leaf_hash.as_slice()),
    ] {
        let _ = writeln!(text, "{side}_{key} {}", hex(bytes));
    }
    let _ = writeln!(
        text,
        "{side}_metadata_leaf_agrees {}",
        facts.metadata_leaf_agrees
    );
    for (key, bytes) in [
        ("static_root", facts.static_root.as_slice()),
        ("outer_root", facts.outer_root.as_slice()),
        ("internal_key", facts.internal_key.as_slice()),
        ("tweak_digest", facts.tweak_digest.as_slice()),
        ("output_key", facts.output_key.as_slice()),
    ] {
        let _ = writeln!(text, "{side}_{key} {}", hex(bytes));
    }
    let _ = writeln!(text, "{side}_parity {}", facts.parity);
    let _ = writeln!(text, "{side}_output_program {}", hex(&facts.output_program));
    render_recipe(
        text,
        &format!("{side}_control_recipe"),
        &facts.control_recipe,
    );
    render_fields(
        text,
        &format!("{side}_field_commitment"),
        &facts.field_commitments,
    );
}

fn render_nonce_check(
    text: &mut String,
    key: &str,
    check: &crate::maturity_continuity::MaturityNonceCheck,
) {
    let _ = write!(
        text,
        "{key} {} {} {} {} {}",
        check.candidate().get(),
        hex(check.encoding()),
        hex(check.leaf_digest()),
        hex(check.retained_root()),
        check.branch_admissible()
    );
    match check.curve_outcome() {
        None => text.push_str(" curve-not-reached"),
        Some(tapscript::StateTweakOutcome::OutputKey { key, parity }) => {
            let _ = write!(text, " output-key {} {parity}", hex(&key));
        }
        Some(outcome) => {
            let _ = write!(text, " {outcome:?}");
        }
    }
    text.push('\n');
}

fn render_leastness(
    text: &mut String,
    side: MaturityContinuityReportSide,
    facts: &MaturityContinuityLeastnessFacts,
) {
    let side = side.name();
    let _ = writeln!(
        text,
        "{side}_witnessed_nonce {}",
        facts.witnessed_nonce.get()
    );
    let _ = writeln!(text, "{side}_selected_nonce {}", facts.selected_nonce.get());
    let _ = writeln!(text, "{side}_host_least {}", facts.host_least);
    let _ = writeln!(text, "{side}_rejected_count {}", facts.rejected_count);
    let _ = writeln!(
        text,
        "{side}_reconstructed_facts_match {:?}",
        facts.reconstructed_facts_match
    );
    let _ = writeln!(text, "{side}_leastness_residual {}", facts.residual.text());
    let checked = &facts.checked;
    let _ = writeln!(
        text,
        "{side}_checked_prefix {:?} {} {}",
        checked.side(),
        checked.is_host_least(),
        checked.residual().text()
    );
    for (candidate, cause) in checked.rejected() {
        render_nonce_check(text, &format!("{side}_rejected_candidate"), candidate);
        let _ = writeln!(text, "{side}_rejection {cause:?}");
    }
    render_nonce_check(
        text,
        &format!("{side}_selected_candidate"),
        checked.selected(),
    );
    render_nonce_check(
        text,
        &format!("{side}_witnessed_candidate"),
        checked.witnessed(),
    );
}

fn render_field_comparisons(text: &mut String, key: &str, comparisons: &[MaturityFieldComparison]) {
    for comparison in comparisons {
        let left = comparison.left();
        let right = comparison.right();
        let _ = writeln!(
            text,
            "{key} {} {:?} {} {} {} {:?} {} {} {} {}",
            comparison.field().name(),
            left.side,
            left.range.start,
            left.range.end,
            hex(&left.bytes),
            right.side,
            right.range.start,
            right.range.end,
            hex(&right.bytes),
            comparison.agrees()
        );
    }
}

fn render_static_semantic(text: &mut String, entry: &MaturityContinuityReportEntry) {
    let static_facts = &entry.static_facts;
    render_comparison(text, "static_root_binding", &static_facts.root_binding);
    render_comparison(
        text,
        "static_control_binding",
        &static_facts.control_binding,
    );
    let _ = writeln!(
        text,
        "descriptor_result {:?}",
        static_facts.descriptor_result
    );
    let _ = writeln!(
        text,
        "constructor_result {:?}",
        static_facts.constructor_result
    );
    let semantic = &entry.semantic_facts;
    let _ = writeln!(text, "transition_agreement {}", semantic.transition_agrees);
    let _ = writeln!(
        text,
        "reconstruction_agreement {:?}",
        semantic.reconstruction_agrees
    );
    render_metadata(text, "expected_successor", semantic.expected_successor);
    let _ = writeln!(text, "requested_cycle {}", semantic.requested_cycle.get());
    let _ = writeln!(
        text,
        "lead_bounds {} {}",
        semantic.bounds.minimum().get(),
        semantic.bounds.maximum().get()
    );
    render_field_comparisons(
        text,
        "predecessor_field_comparison",
        &semantic.predecessor_fields,
    );
    if let Some(fields) = &semantic.successor_fields {
        render_field_comparisons(text, "successor_field_comparison", fields);
    }
}

fn render_tweak(
    text: &mut String,
    side: MaturityContinuityReportSide,
    evidence: &MaturityTweakEvidence,
) {
    let side = side.name();
    let _ = writeln!(text, "{side}_tweak_side {:?}", evidence.side());
    for (key, comparison) in tweak_comparisons(evidence) {
        render_comparison(text, &format!("{side}_tweak_{key}"), comparison);
    }
    for premise in evidence.premises() {
        let _ = writeln!(
            text,
            "{side}_tweak_premise {premise:?} {}",
            premise.domain_or_residual()
        );
    }
}

const fn observation_name(observation: MaturityControlObservation) -> &'static str {
    match observation {
        MaturityControlObservation::ObservedPredecessorSpend => "submitted-predecessor-spend",
        MaturityControlObservation::DerivedSuccessorUnobserved => "derived-successor-no-spend",
    }
}

fn render_controls(text: &mut String, controls: &MaturityControlPaths) {
    render_recipe(text, "control_before", controls.before());
    render_recipe(text, "control_after", controls.after());
    render_comparison(text, "control_submitted_binding", controls.observed());
    let _ = writeln!(text, "control_derived_bytes {}", hex(controls.derived()));
    let kinds = controls.observations();
    let _ = writeln!(
        text,
        "control_kinds {} {}",
        observation_name(kinds[0]),
        observation_name(kinds[1])
    );
    let (before, after) = controls.lengths();
    let _ = writeln!(text, "control_lengths {before} {after}");
    render_comparison(text, "control_inner", controls.inner());
    for outer in controls.outer() {
        render_comparison(text, "control_outer", outer);
    }
    let (actual, predicted) = controls.first_byte_relation();
    let _ = writeln!(text, "control_first_byte_relation {actual} {predicted}");
}

fn render_entry_bindings(text: &mut String, entry: &MaturityContinuityReportEntry) {
    let _ = writeln!(text, "source_index {}", entry.position);
    let (class, address) = match &entry.source {
        MaturityByteSource::NodeFreeSubmitReady => ("node-free-submit-ready", "-".to_owned()),
        MaturityByteSource::ArchivedSubmission { run_address } => {
            ("archived-submission", hex(run_address.as_bytes()))
        }
    };
    let _ = writeln!(text, "source_class {class}");
    let _ = writeln!(text, "source_address {address}");
    let _ = writeln!(text, "byte_identity {}", hex(&entry.byte_identity));
    let funded = &entry.funded;
    let _ = writeln!(
        text,
        "funded_outpoint {} {}",
        funded.outpoint.txid(),
        funded.outpoint.index()
    );
    let _ = writeln!(text, "funded_asset {}", hex(funded.asset.internal()));
    let _ = writeln!(text, "funded_amount {}", funded.amount);
    let _ = writeln!(text, "funded_program {}", hex(&funded.program));
    let _ = writeln!(text, "branch_identifier {}", hex(entry.branch.identifier()));
    let _ = writeln!(text, "branch_checkpoint {}", entry.branch.checkpoint());
    let _ = writeln!(
        text,
        "deployment_identity {} {}",
        hex(entry.identity.network_id()),
        hex(entry.identity.genesis_id())
    );
    let bytes = &entry.bytes;
    let _ = writeln!(text, "submitted_bytes {}", bytes.submitted);
    let _ = writeln!(text, "stripped_bytes {}", bytes.stripped);
    let _ = writeln!(text, "weight {}", bytes.weight);
    let _ = writeln!(text, "witness_widths {:?}", bytes.witness_widths);
}

fn render_entry(text: &mut String, entry: &MaturityContinuityReportEntry) {
    use MaturityContinuityReportSide::{Predecessor, Successor};
    render_entry_bindings(text, entry);
    render_constructor(text, Predecessor, &entry.predecessor);
    render_constructor(text, Successor, &entry.successor);
    render_leastness(text, Predecessor, &entry.predecessor_leastness);
    render_leastness(text, Successor, &entry.successor_leastness);
    render_static_semantic(text, entry);
    render_tweak(text, Predecessor, &entry.predecessor_tweak);
    render_tweak(text, Successor, &entry.successor_tweak);
    render_controls(text, &entry.controls);
    let _ = writeln!(text, "observation_class {:?}", entry.observation_class);
    let _ = writeln!(text, "quantifier {}", entry.quantifier);
}

fn render_header(text: &mut String, report: &MaturityContinuityReport) {
    let target = report.binding.target().projection();
    let _ = writeln!(text, "schema {}", report.schema);
    let _ = writeln!(text, "role {}", report.role.name());
    let _ = writeln!(text, "operation announce-maturity");
    let _ = writeln!(text, "target_contract {:?}", target.version());
    let _ = writeln!(text, "execution_domain {:?}", target.execution_domain());
    let _ = writeln!(text, "deployment {}", report.binding.deployment().name());
    let provenance = match report.provenance {
        MaturityExecutorProvenanceExpectation::Stated(_) => "stated",
        MaturityExecutorProvenanceExpectation::NotStatedByTheOperator => {
            "not-stated-by-the-operator"
        }
    };
    let _ = writeln!(text, "executor_provenance_expectation {provenance}");
    let MaturityAcceptanceObligation::Outstanding { routes } = report.acceptance;
    let _ = writeln!(text, "acceptance outstanding");
    for route in routes {
        let _ = writeln!(text, "acceptance_route {route:?}");
    }
    for residual in &report.residuals {
        let _ = writeln!(text, "residual {}", residual.name());
    }
    let _ = writeln!(text, "sources {}", report.census.sources);
    let _ = writeln!(text, "comparisons {}", report.census.comparisons);
    let _ = writeln!(text, "agreements {}", report.census.agreements);
    let _ = writeln!(text, "completeness {}", report.completeness.name());
}

/// Render canonical bytes from a fully validated wrapper alone.
///
/// Binary values are lowercase hex, textual source addresses are encoded, and
/// source order and each nested inventory are preserved. The module documentation
/// names block order; the closed key-order test pins every nested key. Timing has
/// no field or argument here and is emitted independently.
#[must_use]
pub fn render_maturity_continuity_report(validated: &ValidatedMaturityContinuityReport) -> String {
    let mut text = String::new();
    render_header(&mut text, &validated.report);
    for item in &validated.recomputed_items {
        let _ = writeln!(text, "recomputed {}", item.name());
    }
    for entry in &validated.report.entries {
        render_entry(&mut text, entry);
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::live_owner_observation::{asset_of, decode_hex, outpoint_of};
    use crate::maturity_continuity::{MaturityProjectionInput, project_maturity_continuity};
    use crate::maturity_corpus::maturity_run_of_record;
    use crate::maturity_evidence::derive_maturity_evidence_plan_with;
    use crate::maturity_native::MaturityAnnouncementPlanner;
    use crate::maturity_report::{
        MaturityReportTiming, MaturitySafetyReportRole, MaturityVolatileField,
    };
    use std::io::Write as _;
    use std::sync::LazyLock;
    use std::time::Duration;
    use target_elements_conformance::executor::{OperationStep, TargetOperationPlanner};
    use target_elements_conformance::protocol::{
        FundedOutput, NATIVE_PROTOCOL_SCHEMA, NativeOperationResponse, NativeResourceObservation,
        ObservedOutcomeLayer, OperationSubject, WireOutpoint,
    };
    use target_elements_conformance::provenance::ExpectedExecutorProvenance;

    static SOURCES: LazyLock<[ValidatedMaturityContinuity; 2]> = LazyLock::new(derive_sources);
    static BINDING: LazyLock<MaturityTargetBinding> = LazyLock::new(|| {
        derive_maturity_evidence_plan_with(
            MaturityExecutorProvenanceExpectation::NotStatedByTheOperator,
        )
        .expect("independent evidence plan")
        .binding()
        .clone()
    });

    fn references() -> [&'static ValidatedMaturityContinuity; 2] {
        [&SOURCES[0], &SOURCES[1]]
    }
    fn provenance() -> MaturityExecutorProvenanceExpectation {
        MaturityExecutorProvenanceExpectation::NotStatedByTheOperator
    }
    fn assembled() -> MaturityContinuityReport {
        assemble_maturity_continuity_report(&references(), BINDING.clone(), provenance())
    }
    fn validate(
        report: &MaturityContinuityReport,
    ) -> Result<ValidatedMaturityContinuityReport, MaturityContinuityReportRefusal> {
        validate_maturity_continuity_report(report, &references(), &BINDING, &provenance())
    }
    fn rendered() -> String {
        render_maturity_continuity_report(&validate(&assembled()).expect("report validation"))
    }

    fn funding(output: &FundedOutput) -> MaturityFundedPredecessor {
        MaturityFundedPredecessor {
            outpoint: outpoint_of(&output.outpoint).expect("funding outpoint"),
            asset: asset_of(&output.asset).expect("funding asset"),
            amount: output.amount_satoshis,
            program: decode_hex(&output.script).expect("funding program"),
        }
    }

    fn archived() -> ValidatedMaturityContinuity {
        let corpus = maturity_run_of_record().expect("validated archive");
        let identity = corpus.evidence().identity().clone();
        let branch = corpus.evidence().branch();
        let mut planner = MaturityAnnouncementPlanner::new(
            identity.clone(),
            branch,
            crate::maturity_closure::MaturityWitnessSelection::retained_whole_metadata(),
        )
        .expect("planner");
        let mut next = planner.next_step(None).expect("issuance");
        for (step, response) in &corpus.exchanges()[..2] {
            assert_eq!(next.as_ref(), Some(step));
            next = planner
                .next_step(Some((step.case(), response)))
                .expect("replayed funding");
        }
        assert_eq!(next.as_ref(), Some(&corpus.exchanges()[2].0));
        let submission = match corpus.exchanges()[2].0.subject() {
            OperationSubject::Submission(value) => Some(value),
            _ => None,
        }
        .expect("submission subject");
        assert_eq!(
            planner.submission_bytes(),
            Some(submission.transaction_bytes.as_slice())
        );
        let outputs = &corpus.exchanges()[1].1.funded_outputs;
        assert_eq!(outputs.len(), 1);
        project_maturity_continuity(MaturityProjectionInput {
            source: MaturityByteSource::ArchivedSubmission {
                run_address: corpus.report().run_address().to_owned(),
            },
            submitted_bytes: &submission.transaction_bytes,
            funded: &funding(&outputs[0]),
            branch,
            bundle: planner.bundle(),
            identity: &identity,
        })
        .expect("archived projection")
    }

    fn scripted_response(step: &OperationStep) -> NativeOperationResponse {
        let subject = match step.subject() {
            OperationSubject::Funding(value) => Some(value),
            _ => None,
        }
        .expect("funding subject");
        let asset = format!("01{}fe", "55".repeat(30));
        NativeOperationResponse {
            schema: NATIVE_PROTOCOL_SCHEMA,
            case: step.case().clone(),
            observed_layer: ObservedOutcomeLayer::Accepted,
            observed_detail: None,
            issued_asset: subject.issue_asset.then(|| asset.clone()),
            funded_outputs: vec![FundedOutput {
                outpoint: WireOutpoint {
                    txid: format!("02{}fd", "66".repeat(30)),
                    vout: if subject.issue_asset { 3 } else { 7 },
                },
                asset,
                amount_satoshis: subject.amount_per_output,
                script: hex(&subject.output_program),
            }],
            confidential_funded_outputs: Vec::new(),
            mined_readback: None,
            accepted_txid: None,
            sponsor_witness: Vec::new(),
            script_path_witness: Vec::new(),
            signer_public_key: None,
            signed_profile: None,
            signing_genesis: None,
            signature_bound_to: None,
            resources: NativeResourceObservation::default(),
        }
    }

    fn node_free() -> ValidatedMaturityContinuity {
        let mut genesis = [0x22; 32];
        genesis[0] = 0x01;
        genesis[31] = 0xfe;
        let identity = CandidateDeploymentIdentity::new([0x17; 32], genesis).expect("identity");
        let branch = BranchContext::new([0x41; 32], 7).expect("branch");
        let mut planner = MaturityAnnouncementPlanner::new(
            identity.clone(),
            branch,
            crate::maturity_closure::MaturityWitnessSelection::retained_whole_metadata(),
        )
        .expect("planner");
        let issue = planner.next_step(None).expect("issuance").expect("step");
        let issued = scripted_response(&issue);
        issued.validate_shape().expect("issuance response shape");
        let fund = planner
            .next_step(Some((issue.case(), &issued)))
            .expect("funding")
            .expect("step");
        let response = scripted_response(&fund);
        response.validate_shape().expect("funding response shape");
        let step = planner
            .next_step(Some((fund.case(), &response)))
            .expect("submission")
            .expect("step");
        let submission = match step.subject() {
            OperationSubject::Submission(value) => Some(value),
            _ => None,
        }
        .expect("submission subject");
        assert_eq!(
            planner.submission_bytes(),
            Some(submission.transaction_bytes.as_slice())
        );
        project_maturity_continuity(MaturityProjectionInput {
            source: MaturityByteSource::NodeFreeSubmitReady,
            submitted_bytes: &submission.transaction_bytes,
            funded: &funding(&response.funded_outputs[0]),
            branch,
            bundle: planner.bundle(),
            identity: &identity,
        })
        .expect("node-free projection")
    }

    fn derive_sources() -> [ValidatedMaturityContinuity; 2] {
        [archived(), node_free()]
    }

    fn stated_provenance() -> MaturityExecutorProvenanceExpectation {
        MaturityExecutorProvenanceExpectation::Stated(
            ExpectedExecutorProvenance::new(&"1".repeat(40), &"2".repeat(40), ["continuity"])
                .expect("synthetic provenance premise"),
        )
    }

    fn mutate_0(
        item: MaturityContinuityRecomputedItem,
        report: &mut MaturityContinuityReport,
    ) -> bool {
        use MaturityContinuityRecomputedItem as Item;
        match item {
            Item::Schema => {
                report.schema += 1;
            }
            Item::SourceCount => {
                report.entries.pop();
            }
            Item::SourceOrder => {
                report.entries.swap(0, 1);
            }
            Item::ExecutorProvenanceExpectation => {
                report.provenance = stated_provenance();
            }
            Item::SourceClass => {
                report.entries[0].source = MaturityByteSource::NodeFreeSubmitReady;
            }
            Item::ByteIdentity => {
                report.entries[0].byte_identity[0] ^= 1;
            }
            Item::FundedPredecessor => {
                report.entries[0].funded.amount += 1;
            }
            Item::BranchContext => {
                report.entries[0].branch =
                    BranchContext::new([9; 32], 8).expect("alternate context");
            }
            Item::DeploymentIdentity => {
                report.entries[0].identity = CandidateDeploymentIdentity::new([7; 32], [8; 32])
                    .expect("alternate deployment");
            }
            Item::SubmittedLength => {
                report.entries[0].bytes.submitted += 1;
            }
            Item::StrippedLength => {
                report.entries[0].bytes.stripped += 1;
            }
            Item::TransactionWeight => {
                report.entries[0].bytes.weight += 1;
            }
            _ => return false,
        }
        true
    }

    fn mutate_1(
        item: MaturityContinuityRecomputedItem,
        report: &mut MaturityContinuityReport,
    ) -> bool {
        use MaturityContinuityRecomputedItem as Item;
        match item {
            Item::WitnessWidths => {
                report.entries[0].bytes.witness_widths[0] += 1;
            }
            Item::PredecessorSemanticMetadata => {
                report.entries[0].predecessor.semantic.cycle = Cycle::new(99);
            }
            Item::PredecessorNonce => {
                report.entries[0].predecessor.nonce =
                    StateRepresentationNonce::new(report.entries[0].predecessor.nonce.get() + 1);
            }
            Item::PredecessorMetadataBytes => {
                report.entries[0].predecessor.metadata_bytes.push(0);
            }
            Item::PredecessorMetadataLeafBytes => {
                report.entries[0].predecessor.metadata_leaf_bytes.push(0);
            }
            Item::PredecessorMetadataLeafHash => {
                report.entries[0].predecessor.metadata_leaf_hash[0] ^= 1;
            }
            Item::PredecessorMetadataLeafAgreement => {
                report.entries[0].predecessor.metadata_leaf_agrees =
                    !report.entries[0].predecessor.metadata_leaf_agrees;
            }
            Item::PredecessorStaticRoot => {
                report.entries[0].predecessor.static_root[0] ^= 1;
            }
            Item::PredecessorOuterRoot => {
                report.entries[0].predecessor.outer_root[0] ^= 1;
            }
            Item::PredecessorInternalKey => {
                report.entries[0].predecessor.internal_key[0] ^= 1;
            }
            Item::PredecessorTweakDigest => {
                report.entries[0].predecessor.tweak_digest[0] ^= 1;
            }
            Item::PredecessorOutputKey => {
                report.entries[0].predecessor.output_key[0] ^= 1;
            }
            _ => return false,
        }
        true
    }

    fn mutate_2(
        item: MaturityContinuityRecomputedItem,
        report: &mut MaturityContinuityReport,
    ) -> bool {
        use MaturityContinuityRecomputedItem as Item;
        match item {
            Item::PredecessorParity => {
                report.entries[0].predecessor.parity = !report.entries[0].predecessor.parity;
            }
            Item::PredecessorOutputProgram => {
                report.entries[0].predecessor.output_program.push(0);
            }
            Item::PredecessorControlRecipe => {
                report.entries[0].predecessor.control_recipe.internal_key[0] ^= 1;
            }
            Item::PredecessorFieldCommitments => {
                report.entries[0].predecessor.field_commitments.clear();
            }
            Item::PredecessorWitnessedNonce => {
                report.entries[0].predecessor_leastness.witnessed_nonce =
                    StateRepresentationNonce::new(
                        report.entries[0]
                            .predecessor_leastness
                            .witnessed_nonce
                            .get()
                            + 1,
                    );
            }
            Item::PredecessorSelectedNonce => {
                report.entries[0].predecessor_leastness.selected_nonce =
                    StateRepresentationNonce::new(
                        report.entries[0].predecessor_leastness.selected_nonce.get() + 1,
                    );
            }
            Item::PredecessorHostLeast => {
                report.entries[0].predecessor_leastness.host_least =
                    !report.entries[0].predecessor_leastness.host_least;
            }
            Item::PredecessorRejectedCount => {
                report.entries[0].predecessor_leastness.rejected_count += 1;
            }
            Item::PredecessorReconstructedFactsMatch => {
                report.entries[0]
                    .predecessor_leastness
                    .reconstructed_facts_match = None;
            }
            Item::PredecessorCheckedPrefix => {
                report.entries[0].predecessor_leastness.checked =
                    report.entries[1].predecessor_leastness.checked.clone();
            }
            Item::SuccessorSemanticMetadata => {
                report.entries[0].successor.semantic.cycle = Cycle::new(99);
            }
            Item::SuccessorNonce => {
                report.entries[0].successor.nonce =
                    StateRepresentationNonce::new(report.entries[0].successor.nonce.get() + 1);
            }
            _ => return false,
        }
        true
    }

    fn mutate_3(
        item: MaturityContinuityRecomputedItem,
        report: &mut MaturityContinuityReport,
    ) -> bool {
        use MaturityContinuityRecomputedItem as Item;
        match item {
            Item::SuccessorMetadataBytes => {
                report.entries[0].successor.metadata_bytes.push(0);
            }
            Item::SuccessorMetadataLeafBytes => {
                report.entries[0].successor.metadata_leaf_bytes.push(0);
            }
            Item::SuccessorMetadataLeafHash => {
                report.entries[0].successor.metadata_leaf_hash[0] ^= 1;
            }
            Item::SuccessorMetadataLeafAgreement => {
                report.entries[0].successor.metadata_leaf_agrees =
                    !report.entries[0].successor.metadata_leaf_agrees;
            }
            Item::SuccessorStaticRoot => {
                report.entries[0].successor.static_root[0] ^= 1;
            }
            Item::SuccessorOuterRoot => {
                report.entries[0].successor.outer_root[0] ^= 1;
            }
            Item::SuccessorInternalKey => {
                report.entries[0].successor.internal_key[0] ^= 1;
            }
            Item::SuccessorTweakDigest => {
                report.entries[0].successor.tweak_digest[0] ^= 1;
            }
            Item::SuccessorOutputKey => {
                report.entries[0].successor.output_key[0] ^= 1;
            }
            Item::SuccessorParity => {
                report.entries[0].successor.parity = !report.entries[0].successor.parity;
            }
            Item::SuccessorOutputProgram => {
                report.entries[0].successor.output_program.push(0);
            }
            Item::SuccessorControlRecipe => {
                report.entries[0].successor.control_recipe.internal_key[0] ^= 1;
            }
            _ => return false,
        }
        true
    }

    fn mutate_4(
        item: MaturityContinuityRecomputedItem,
        report: &mut MaturityContinuityReport,
    ) -> bool {
        use MaturityContinuityRecomputedItem as Item;
        match item {
            Item::SuccessorFieldCommitments => {
                report.entries[0].successor.field_commitments.clear();
            }
            Item::SuccessorWitnessedNonce => {
                report.entries[0].successor_leastness.witnessed_nonce =
                    StateRepresentationNonce::new(
                        report.entries[0].successor_leastness.witnessed_nonce.get() + 1,
                    );
            }
            Item::SuccessorSelectedNonce => {
                report.entries[0].successor_leastness.selected_nonce =
                    StateRepresentationNonce::new(
                        report.entries[0].successor_leastness.selected_nonce.get() + 1,
                    );
            }
            Item::SuccessorHostLeast => {
                report.entries[0].successor_leastness.host_least =
                    !report.entries[0].successor_leastness.host_least;
            }
            Item::SuccessorRejectedCount => {
                report.entries[0].successor_leastness.rejected_count += 1;
            }
            Item::SuccessorReconstructedFactsMatch => {
                report.entries[0]
                    .successor_leastness
                    .reconstructed_facts_match = None;
            }
            Item::SuccessorCheckedPrefix => {
                report.entries[0].successor_leastness.checked =
                    report.entries[1].successor_leastness.checked.clone();
            }
            Item::StaticRootBinding => {
                report.entries[0].static_facts.root_binding =
                    report.entries[1].static_facts.root_binding.clone();
            }
            Item::StaticControlBinding => {
                report.entries[0].static_facts.control_binding =
                    report.entries[1].static_facts.control_binding.clone();
            }
            Item::DescriptorResult => {
                report.entries[0].static_facts.descriptor_result =
                    Err(Box::new(StateLinkRefusal::SingletonDeclarationMismatch));
            }
            Item::ConstructorResult => {
                report.entries[0].static_facts.constructor_result =
                    Err(Box::new(StateConstructorRefusal::MetadataEncodingRefused));
            }
            Item::TransitionAgreement => {
                report.entries[0].semantic_facts.transition_agrees = false;
            }
            _ => return false,
        }
        true
    }

    fn mutate_5(
        item: MaturityContinuityRecomputedItem,
        report: &mut MaturityContinuityReport,
    ) -> bool {
        use MaturityContinuityRecomputedItem as Item;
        match item {
            Item::ReconstructionAgreement => {
                report.entries[0].semantic_facts.reconstruction_agrees = None;
            }
            Item::ExpectedSuccessor => {
                report.entries[0].semantic_facts.expected_successor.cycle = Cycle::new(99);
            }
            Item::RequestedCycle => {
                report.entries[0].semantic_facts.requested_cycle = Cycle::new(99);
            }
            Item::LeadBounds => {
                report.entries[0].semantic_facts.bounds =
                    AnnouncementLeadBounds::new(Cycle::new(1), Cycle::new(2))
                        .expect("alternate bounds");
            }
            Item::PredecessorFieldComparisons => {
                report.entries[0].semantic_facts.predecessor_fields.clear();
            }
            Item::SuccessorFieldComparisons => {
                report.entries[0].semantic_facts.successor_fields = None;
            }
            Item::ControlRecipes => {
                report.entries[0].controls = report.entries[1].controls.clone();
            }
            Item::ObservationClass => {
                report.entries[0].observation_class =
                    MaturityObservationClass::ModeledUnintendedEnvironmentTransition;
            }
            Item::AcceptanceDisposition => {
                report.acceptance = MaturityAcceptanceObligation::Outstanding {
                    routes: [
                        MaturityAcceptanceRoute::BlockLayerSubmissionSubject,
                        MaturityAcceptanceRoute::RelayWitnessRestructure,
                    ],
                };
            }
            Item::AttributionResidual => {
                report.residuals[0] = MaturityContinuityReportResidual::ALL[1];
            }
            Item::HostSearchResidual => {
                report.residuals[1] = MaturityContinuityReportResidual::ALL[2];
            }
            Item::RetainedTreeResidual => {
                report.residuals[2] = MaturityContinuityReportResidual::ALL[3];
            }
            _ => return false,
        }
        true
    }

    fn mutate_6(
        item: MaturityContinuityRecomputedItem,
        report: &mut MaturityContinuityReport,
    ) -> bool {
        use MaturityContinuityRecomputedItem as Item;
        match item {
            Item::BranchContextResidual => {
                report.residuals[3] = MaturityContinuityReportResidual::ALL[0];
            }
            Item::CensusSources => {
                report.census.sources += 1;
            }
            Item::CensusComparisons => {
                report.census.comparisons += 1;
            }
            Item::CensusAgreements => {
                report.census.agreements += 1;
            }
            Item::Completeness => {
                report.completeness = MaturitySafetyCompleteness::CompleteForTheRequiredMatrix;
            }
            Item::PredecessorTweakSide => {
                report.entries[0].predecessor_tweak = report.entries[0].successor_tweak.clone();
            }
            Item::SuccessorTweakSide => {
                report.entries[0].successor_tweak = report.entries[0].predecessor_tweak.clone();
            }
            _ => return false,
        }
        true
    }

    fn mutate(
        item: MaturityContinuityRecomputedItem,
        report: &mut MaturityContinuityReport,
    ) -> bool {
        mutate_0(item, report)
            || mutate_1(item, report)
            || mutate_2(item, report)
            || mutate_3(item, report)
            || mutate_4(item, report)
            || mutate_5(item, report)
            || mutate_6(item, report)
    }

    fn unreachable_reason(item: MaturityContinuityRecomputedItem) -> Option<&'static str> {
        use MaturityContinuityRecomputedItem as Item;
        match item {
            Item::Role => Some("unreachable: the continuity role has exactly one member"),
            Item::TargetBinding => Some(
                "unreachable: target binding has private fields, no public constructor, and the public evidence producer fixes one binding",
            ),
            Item::PredecessorLeastnessResidual
            | Item::SuccessorLeastnessResidual
            | Item::GuaranteeQuantifier => {
                Some("unreachable: the imported vocabulary has one value and no mutable fields")
            }
            Item::PredecessorTweakInternal
            | Item::PredecessorTweakMetadata
            | Item::PredecessorTweakBranch
            | Item::PredecessorTweakDigestComparison
            | Item::PredecessorTweakKey
            | Item::PredecessorTweakParity
            | Item::PredecessorTweakProgram
            | Item::PredecessorTweakPremises
            | Item::SuccessorTweakInternal
            | Item::SuccessorTweakMetadata
            | Item::SuccessorTweakBranch
            | Item::SuccessorTweakDigestComparison
            | Item::SuccessorTweakKey
            | Item::SuccessorTweakParity
            | Item::SuccessorTweakProgram
            | Item::SuccessorTweakPremises
            | Item::ControlSubmittedBinding
            | Item::ControlDerivedBytes
            | Item::ControlObservationKinds
            | Item::ControlLengths
            | Item::ControlInner
            | Item::ControlOuter
            | Item::ControlFirstByteRelation => Some(
                "unreachable isolated mutation: the imported evidence has private fields and no public constructor; whole-value substitutions are checked separately",
            ),
            _ => None,
        }
    }

    fn assert_refused(report: &MaturityContinuityReport, item: MaturityContinuityRecomputedItem) {
        assert_eq!(
            validate(report)
                .expect_err("altered report refuses")
                .failed_item(),
            item
        );
    }

    #[test]
    fn assembles_over_both_public_source_paths() {
        let report = assembled();
        assert_eq!(report.entries().len(), SOURCES.len());
        assert!(matches!(
            report.entries()[0].source(),
            MaturityByteSource::ArchivedSubmission { .. }
        ));
        assert_eq!(
            report.entries()[1].source(),
            &MaturityByteSource::NodeFreeSubmitReady
        );
        for (entry, source) in report.entries().iter().zip(references()) {
            assert_eq!(entry.byte_identity(), source.byte_identity());
            assert_eq!(entry.funded(), source.funded());
            assert_eq!(entry.identity(), source.identity());
        }
    }

    #[test]
    fn validation_records_the_complete_item_census() {
        let validated = validate(&assembled()).expect("validation");
        let expected: BTreeSet<_> = MaturityContinuityRecomputedItem::ALL
            .iter()
            .copied()
            .collect();
        assert_eq!(validated.recomputed_items(), &expected);
        assert_eq!(expected.len(), 107);
        let names: BTreeSet<_> = expected.iter().map(|item| item.name()).collect();
        assert_eq!(names.len(), expected.len());
        assert!(names.iter().all(|name| !name.is_empty()));
    }

    #[test]
    fn independent_derivations_render_identical_bytes() {
        let first = derive_sources();
        let second = derive_sources();
        let render = |sources: &[ValidatedMaturityContinuity; 2]| {
            let references = [&sources[0], &sources[1]];
            let report =
                assemble_maturity_continuity_report(&references, BINDING.clone(), provenance());
            let validated =
                validate_maturity_continuity_report(&report, &references, &BINDING, &provenance())
                    .expect("independent validation");
            render_maturity_continuity_report(&validated)
        };
        assert_eq!(render(&first), render(&second));
    }

    #[test]
    fn timing_has_a_separate_noncanonical_emission() {
        let report = validate(&assembled()).expect("validation");
        let before = render_maturity_continuity_report(&report);
        let quick = MaturityReportTiming::measured(Duration::from_nanos(1_234_567));
        let slow = MaturityReportTiming::measured(Duration::from_nanos(7_654_321));
        assert_ne!(quick.render(), slow.render());
        assert!(quick.render().contains("canonical false"));
        assert!(slow.render().contains("canonical false"));
        assert_eq!(before, render_maturity_continuity_report(&report));
        assert!(!before.contains("derivation_nanos"));
    }

    #[test]
    fn acceptance_stays_outstanding_with_both_routes() {
        let report = assembled();
        assert_eq!(report.acceptance(), SOURCES[0].acceptance_obligation());
        let bytes = rendered();
        assert!(bytes.contains("acceptance outstanding\n"));
        assert!(bytes.contains("acceptance_route RelayWitnessRestructure\n"));
        assert!(bytes.contains("acceptance_route BlockLayerSubmissionSubject\n"));
        assert!(bytes.contains("completeness partial-required-rows-outstanding\n"));
    }

    #[test]
    fn no_input_shape_can_produce_complete_readback() {
        let both = references();
        for sources in [
            Vec::new(),
            vec![both[0]],
            vec![both[1]],
            both.to_vec(),
            vec![both[0], both[0]],
            vec![both[1], both[0]],
        ] {
            let report =
                assemble_maturity_continuity_report(&sources, BINDING.clone(), provenance());
            let validated =
                validate_maturity_continuity_report(&report, &sources, &BINDING, &provenance())
                    .expect("all admitted lengths");
            assert_eq!(
                validated.report().completeness(),
                MaturitySafetyCompleteness::PartialRequiredRowsOutstanding
            );
            assert_ne!(
                validated.report().completeness(),
                MaturitySafetyCompleteness::CompleteForTheRequiredMatrix
            );
        }
        // The acceptance input has only Outstanding; none of these lengths can
        // supply a target-computed accepted identity or change the total rule.
    }

    #[test]
    fn every_item_is_mutated_or_has_an_explicit_type_barrier() {
        let mut altered = 0;
        let mut unreachable = 0;
        for &item in MaturityContinuityRecomputedItem::ALL {
            let mut report = assembled();
            if mutate(item, &mut report) {
                assert!(unreachable_reason(item).is_none(), "{item:?}");
                assert_refused(&report, item);
                altered += 1;
            } else {
                let reason = unreachable_reason(item)
                    .expect("every nonmutated item names its exact barrier");
                assert!(reason.starts_with("unreachable"));
                assert!(
                    validate(&report)
                        .expect("unchanged item is still checked")
                        .recomputed_items()
                        .contains(&item)
                );
                writeln!(
                    std::io::stdout().lock(),
                    "\nRUN-REPORT item={item:?} {reason}"
                )
                .expect("item evidence");
                unreachable += 1;
            }
        }
        assert_eq!((altered, unreachable), (79, 28));
    }

    fn compound_mutations() -> Vec<(MaturityContinuityRecomputedItem, MaturityContinuityReport)> {
        use MaturityContinuityRecomputedItem as Item;
        let original = assembled();
        let mut mutants = Vec::new();
        for change in [
            |funded: &mut MaturityFundedPredecessor| funded.outpoint = SOURCES[1].funded().outpoint,
            |funded: &mut MaturityFundedPredecessor| funded.asset = SOURCES[1].funded().asset,
            |funded: &mut MaturityFundedPredecessor| funded.amount += 1,
            |funded: &mut MaturityFundedPredecessor| funded.program.push(0),
        ] {
            let mut report = original.clone();
            change(&mut report.entries[0].funded);
            mutants.push((Item::FundedPredecessor, report));
        }
        for change in [
            |recipe: &mut StateControlRecipe| recipe.internal_key[0] ^= 1,
            |recipe: &mut StateControlRecipe| recipe.executing_leaf_hash[0] ^= 1,
            |recipe: &mut StateControlRecipe| recipe.parity = !recipe.parity,
            |recipe: &mut StateControlRecipe| recipe.siblings.push([1; 32]),
        ] {
            let mut report = original.clone();
            change(&mut report.entries[0].predecessor.control_recipe);
            mutants.push((Item::PredecessorControlRecipe, report));
        }
        for change in [
            |metadata: &mut StateMetadata| metadata.omega = realization::ProtocolAmount::ZERO,
            |metadata: &mut StateMetadata| metadata.y_l = realization::ProtocolAmount::ZERO,
            |metadata: &mut StateMetadata| metadata.y_t = realization::ProtocolAmount::ZERO,
            |metadata: &mut StateMetadata| metadata.q = realization::ProtocolAmount::ZERO,
            |metadata: &mut StateMetadata| metadata.cycle = Cycle::new(99),
            |metadata: &mut StateMetadata| {
                metadata.maturity = realization::Maturity::Announced {
                    cycle: Cycle::new(99),
                }
            },
        ] {
            let mut report = original.clone();
            change(&mut report.entries[0].predecessor.semantic);
            mutants.push((Item::PredecessorSemanticMetadata, report));
        }
        mutants
    }

    #[test]
    fn compound_claims_and_opaque_evidence_are_compared_in_full() {
        for (item, report) in compound_mutations() {
            assert_refused(&report, item);
        }
        let original = assembled();
        let mut changed = original.entries[0].clone();
        changed.predecessor_tweak = original.entries[1].predecessor_tweak.clone();
        for item in [
            MaturityContinuityRecomputedItem::PredecessorTweakMetadata,
            MaturityContinuityRecomputedItem::PredecessorTweakBranch,
            MaturityContinuityRecomputedItem::PredecessorTweakDigestComparison,
            MaturityContinuityRecomputedItem::PredecessorTweakKey,
            MaturityContinuityRecomputedItem::PredecessorTweakParity,
            MaturityContinuityRecomputedItem::PredecessorTweakProgram,
        ] {
            assert!(
                !entry_agrees(item, &changed, &original.entries[0]),
                "{item:?}"
            );
        }
        let mut report = original;
        report.entries[0].predecessor_tweak = changed.predecessor_tweak;
        assert_refused(
            &report,
            MaturityContinuityRecomputedItem::PredecessorTweakMetadata,
        );
    }

    #[test]
    fn removed_entry_refuses_before_payload_comparison() {
        let mut report = assembled();
        report.entries.pop();
        report.entries[0].bytes.submitted += 1;
        assert!(matches!(
            validate(&report),
            Err(MaturityContinuityReportRefusal::SourceCountDiffers {
                stated: 1,
                recomputed: 2
            })
        ));
    }

    #[test]
    fn duplicated_entry_refuses_before_payload_comparison() {
        let mut report = assembled();
        report.entries.push(report.entries[0].clone());
        assert!(matches!(
            validate(&report),
            Err(MaturityContinuityReportRefusal::SourceCountDiffers {
                stated: 3,
                recomputed: 2
            })
        ));
    }

    #[test]
    fn reordered_entries_refuse() {
        let mut report = assembled();
        report.entries.swap(0, 1);
        assert!(matches!(
            validate(&report),
            Err(MaturityContinuityReportRefusal::SourceOrderDiffers { position: 0 })
        ));
    }

    #[test]
    fn reordered_supplied_sources_refuse() {
        let report = assembled();
        let sources = [&SOURCES[1], &SOURCES[0]];
        assert!(matches!(
            validate_maturity_continuity_report(&report, &sources, &BINDING, &provenance()),
            Err(MaturityContinuityReportRefusal::SourceOrderDiffers { position: 0 })
        ));
    }

    #[test]
    fn unread_schema_has_first_precedence() {
        let mut report = assembled();
        report.schema += 1;
        report.entries.clear();
        assert!(matches!(
            validate(&report),
            Err(MaturityContinuityReportRefusal::UnsupportedSchema { .. })
        ));
    }

    #[test]
    fn structural_order_precedes_payload_disagreement() {
        let mut report = assembled();
        report.entries.swap(0, 1);
        report.entries[0].predecessor.parity = !report.entries[0].predecessor.parity;
        assert_refused(&report, MaturityContinuityRecomputedItem::SourceOrder);
    }

    #[test]
    fn exclusions_are_exactly_the_eleven_guide_categories() {
        let names: Vec<_> = MaturityVolatileField::ALL
            .iter()
            .map(|field| field.name())
            .collect();
        assert_eq!(
            names,
            [
                "wall-clock-time",
                "elapsed-time",
                "hostname",
                "username",
                "process-id",
                "temporary-path",
                "executor-path",
                "caller-selected-filesystem-path",
                "ambient-environment",
                "raw-child-standard-error",
                "production-or-test-private-scalar",
            ]
        );
        assert_eq!(names.iter().collect::<BTreeSet<_>>().len(), 11);
    }

    #[test]
    fn every_rendered_key_excludes_volatile_categories() {
        for line in rendered().lines() {
            let key = line.split_whitespace().next().expect("key");
            let normalized = key.replace('_', "-");
            for field in MaturityVolatileField::ALL {
                assert_ne!(normalized, field.name());
            }
            assert!(line.len() > key.len() + 1);
        }
    }

    #[test]
    fn continuity_and_safety_role_spellings_differ() {
        assert_ne!(
            MaturityContinuityReportRole::StateConstructorContinuity.name(),
            MaturitySafetyReportRole::MaturityAnnouncementSafety.name()
        );
        assert!(rendered().contains("role state-constructor-continuity\n"));
    }

    #[test]
    fn quantifier_admits_two_classes_and_excludes_poison() {
        for entry in assembled().entries() {
            assert_eq!(
                entry.quantifier().admitted(),
                [
                    MaturityObservationClass::IntendedTransition,
                    MaturityObservationClass::ModeledUnintendedEnvironmentTransition,
                ]
            );
            assert_eq!(
                entry.quantifier().excluded(),
                MaturityObservationClass::ModelFalsifyingWithNoPreimage
            );
            assert_eq!(
                entry.observation_class(),
                MaturityObservationClass::IntendedTransition
            );
        }
        let mut report = assembled();
        report.entries[0].observation_class =
            MaturityObservationClass::ModelFalsifyingWithNoPreimage;
        assert_refused(&report, MaturityContinuityRecomputedItem::ObservationClass);
    }

    #[test]
    fn branch_operands_carry_context_without_freshness() {
        let report = assembled();
        let bytes = rendered();
        for (entry, source) in report.entries().iter().zip(references()) {
            assert_eq!(entry.branch(), source.branch());
            assert!(bytes.contains(&format!(
                "branch_identifier {}\n",
                hex(source.branch().identifier())
            )));
            assert!(bytes.contains(&format!(
                "branch_checkpoint {}\n",
                source.branch().checkpoint()
            )));
        }
        assert!(!bytes.lines().any(|line| {
            line.split_whitespace()
                .next()
                .is_some_and(|key| key.contains("fresh"))
        }));
        assert!(bytes.contains("residual caller-stated-branch-without-freshness\n"));
    }

    #[test]
    fn canonical_bytes_have_no_accepted_or_observed_standing_spelling() {
        let bytes = rendered();
        // Neither route name needs an exception. Control evidence uses distinct
        // submitted-versus-derived wire labels without changing its meaning.
        for forbidden in ["Accepted", "Observed"] {
            assert!(!bytes.contains(forbidden));
        }
        assert!(
            bytes
                .contains("control_kinds submitted-predecessor-spend derived-successor-no-spend\n")
        );
    }

    fn check_rendered_measurements(index: usize, label: &str) {
        let source = &SOURCES[index];
        let report = assembled();
        let entry = &report.entries()[index];
        let mut bytes = String::new();
        render_entry(&mut bytes, entry);
        let widths: Vec<_> = source
            .transaction()
            .witnesses()
            .iter()
            .flat_map(|witness| witness.stack().iter().map(Vec::len))
            .collect();
        for (key, value) in [
            (
                "submitted_bytes",
                source.submitted_bytes().len().to_string(),
            ),
            (
                "stripped_bytes",
                source
                    .transaction()
                    .encode_without_witness()
                    .len()
                    .to_string(),
            ),
            ("weight", source.transaction().weight().to_string()),
            ("witness_widths", format!("{widths:?}")),
            (
                "predecessor_nonce",
                source.predecessor().nonce().get().to_string(),
            ),
            (
                "successor_nonce",
                source.successor().nonce().get().to_string(),
            ),
            (
                "predecessor_rejected_count",
                source.checked_predecessor().rejected().len().to_string(),
            ),
            (
                "successor_rejected_count",
                source.checked_successor().rejected().len().to_string(),
            ),
            (
                "requested_cycle",
                source.requested_cycle().get().to_string(),
            ),
        ] {
            assert!(bytes.contains(&format!("{key} {value}\n")), "{key}");
        }
        for (projection, facts, checked) in [
            (
                source.predecessor(),
                entry.predecessor(),
                source.checked_predecessor(),
            ),
            (
                source.successor(),
                entry.successor(),
                source.checked_successor(),
            ),
        ] {
            let target = closure_target().expect("target");
            let program = state_metadata_leaf_program(&target, projection.encoded_metadata())
                .expect("witnessed leaf");
            assert_eq!(facts.metadata_leaf_bytes(), program.encode(&target));
            assert_eq!(
                leaf_hash(
                    target.definition().leaf_version(),
                    facts.metadata_leaf_bytes()
                ),
                *checked.witnessed().leaf_digest()
            );
            assert!(facts.metadata_leaf_agrees());
        }
        writeln!(std::io::stdout().lock(),
            "\nRUN-REPORT continuity-report source={label} submitted={} stripped={} weight={} widths={widths:?} predecessor_nonce={} successor_nonce={} predecessor_rejected={} successor_rejected={} cycle={} bounds={}-{} request={} comparisons={} agreements={} completeness={} acceptance=outstanding",
            entry.bytes().submitted(), entry.bytes().stripped(), entry.bytes().weight(),
            entry.predecessor().nonce().get(), entry.successor().nonce().get(),
            entry.predecessor_leastness().rejected_count(), entry.successor_leastness().rejected_count(),
            entry.predecessor().semantic().cycle.get(), entry.semantic_facts().bounds().minimum().get(),
            entry.semantic_facts().bounds().maximum().get(), entry.semantic_facts().requested_cycle().get(),
            comparison_results(source).len(), comparison_results(source).iter().filter(|result| **result).count(),
            report.completeness().name()).expect("measured report evidence");
    }

    #[test]
    fn archived_rendering_uses_projection_measurements() {
        check_rendered_measurements(0, "archived");
    }

    #[test]
    fn node_free_rendering_uses_projection_measurements() {
        check_rendered_measurements(1, "node-free");
    }

    #[test]
    fn independent_static_and_semantic_results_keep_their_own_claims() {
        let report = assembled();
        for (entry, source) in report.entries().iter().zip(references()) {
            let facts = entry.static_facts();
            assert_eq!(
                facts.root_binding(),
                source.static_comparison().static_root()
            );
            assert_eq!(
                facts.control_binding(),
                source.static_comparison().control_block()
            );
            assert_eq!(
                facts.descriptor_result(),
                source.static_comparison().descriptor_result()
            );
            assert_eq!(
                facts.constructor_result(),
                source.static_comparison().constructor_result()
            );
            assert_eq!(
                entry.semantic_facts().transition_agrees(),
                source.semantic_comparison().transition_agrees()
            );
            assert_eq!(
                entry.semantic_facts().reconstruction_agrees(),
                source.semantic_comparison().reconstruction_agrees()
            );
            assert_eq!(entry.predecessor_tweak(), source.predecessor_tweak());
            assert_eq!(entry.successor_tweak(), source.successor_tweak());
            assert_eq!(entry.controls(), source.controls());
        }
        assert_eq!(
            report.census().comparisons(),
            references()
                .iter()
                .map(|source| comparison_results(source).len())
                .sum::<usize>()
        );
        assert_eq!(
            report.census().agreements(),
            references()
                .iter()
                .flat_map(|source| comparison_results(source))
                .filter(|result| *result)
                .count()
        );
    }

    fn constructor_keys(side: &str) -> Vec<String> {
        let mut keys: Vec<_> = [
            "semantic",
            "nonce",
            "metadata_bytes",
            "metadata_leaf_bytes",
            "metadata_leaf_hash",
            "metadata_leaf_agrees",
            "static_root",
            "outer_root",
            "internal_key",
            "tweak_digest",
            "output_key",
            "parity",
            "output_program",
            "control_recipe",
        ]
        .iter()
        .map(|key| format!("{side}_{key}"))
        .collect();
        keys.extend(std::iter::repeat_n(format!("{side}_field_commitment"), 6));
        keys
    }

    fn leastness_keys(side: &str, rejected: usize) -> Vec<String> {
        let mut keys: Vec<_> = [
            "witnessed_nonce",
            "selected_nonce",
            "host_least",
            "rejected_count",
            "reconstructed_facts_match",
            "leastness_residual",
            "checked_prefix",
        ]
        .iter()
        .map(|key| format!("{side}_{key}"))
        .collect();
        for _ in 0..rejected {
            keys.extend([
                format!("{side}_rejected_candidate"),
                format!("{side}_rejection"),
            ]);
        }
        keys.extend([
            format!("{side}_selected_candidate"),
            format!("{side}_witnessed_candidate"),
        ]);
        keys
    }

    fn entry_keys(source: &ValidatedMaturityContinuity) -> Vec<String> {
        let mut keys: Vec<_> = [
            "source_index",
            "source_class",
            "source_address",
            "byte_identity",
            "funded_outpoint",
            "funded_asset",
            "funded_amount",
            "funded_program",
            "branch_identifier",
            "branch_checkpoint",
            "deployment_identity",
            "submitted_bytes",
            "stripped_bytes",
            "weight",
            "witness_widths",
        ]
        .iter()
        .map(ToString::to_string)
        .collect();
        keys.extend(constructor_keys("predecessor"));
        keys.extend(constructor_keys("successor"));
        keys.extend(leastness_keys(
            "predecessor",
            source.checked_predecessor().rejected().len(),
        ));
        keys.extend(leastness_keys(
            "successor",
            source.checked_successor().rejected().len(),
        ));
        keys.extend(
            [
                "static_root_binding",
                "static_control_binding",
                "descriptor_result",
                "constructor_result",
                "transition_agreement",
                "reconstruction_agreement",
                "expected_successor",
                "requested_cycle",
                "lead_bounds",
            ]
            .iter()
            .map(ToString::to_string),
        );
        keys.extend(std::iter::repeat_n(
            "predecessor_field_comparison".to_owned(),
            6,
        ));
        keys.extend(std::iter::repeat_n(
            "successor_field_comparison".to_owned(),
            6,
        ));
        for side in ["predecessor", "successor"] {
            for key in [
                "side", "internal", "metadata", "branch", "digest", "key", "parity", "program",
            ] {
                keys.push(format!("{side}_tweak_{key}"));
            }
            keys.extend(std::iter::repeat_n(format!("{side}_tweak_premise"), 7));
        }
        keys.extend(
            [
                "control_before",
                "control_after",
                "control_submitted_binding",
                "control_derived_bytes",
                "control_kinds",
                "control_lengths",
                "control_inner",
                "control_outer",
                "control_outer",
                "control_first_byte_relation",
                "observation_class",
                "quantifier",
            ]
            .iter()
            .map(ToString::to_string),
        );
        keys
    }

    #[test]
    fn key_order_cardinality_and_residuals_are_canonical() {
        let mut expected: Vec<_> = [
            "schema",
            "role",
            "operation",
            "target_contract",
            "execution_domain",
            "deployment",
            "executor_provenance_expectation",
            "acceptance",
            "acceptance_route",
            "acceptance_route",
            "residual",
            "residual",
            "residual",
            "residual",
            "sources",
            "comparisons",
            "agreements",
            "completeness",
        ]
        .iter()
        .map(ToString::to_string)
        .collect();
        expected.extend(std::iter::repeat_n(
            "recomputed".to_owned(),
            MaturityContinuityRecomputedItem::ALL.len(),
        ));
        for source in references() {
            expected.extend(entry_keys(source));
        }
        let bytes = rendered();
        let actual: Vec<_> = bytes
            .lines()
            .map(|line| line.split_whitespace().next().expect("key"))
            .collect();
        assert_eq!(actual, expected);
        assert_eq!(
            assembled().residuals(),
            MaturityContinuityReportResidual::ALL
        );
        let mut report = assembled();
        report
            .residuals
            .push(MaturityContinuityReportResidual::HostSearchOnly);
        assert_refused(
            &report,
            MaturityContinuityRecomputedItem::AttributionResidual,
        );
        assert_eq!(bytes, rendered());
    }
}
