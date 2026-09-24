//! Public recovery evidence over one published announcement handoff (Guide 14 §14.7).
//!
//! The report records the public inputs and recovered successor together.
//! Its residuals state the limits of pinned admission and public reconstruction.

use std::collections::BTreeSet;
use std::fmt::Write as _;

use realization::{
    AnnouncementLeadBounds, Cycle, EncodedStateMetadata, RealizationError, StateMetadata,
    StateRepresentationNonce, encode_state_metadata,
};
use tapscript::{
    STATE_NUMS_KEY, StateConstructorRefusal, StateInternalKeyPolicy, StateLeafRole,
    StateStaticLeaf, StateStaticNode, StateStaticSubtree, TapscriptError, TapscriptProgram,
};
use target_elements::TargetContractVersion;
use transaction::bytes::TargetTransaction;
use transaction::error::TransactionRefusal;

use crate::maturity_closure::{
    MaturityClosureRefusal, MaturityDeployment, OracleStateCurve, closure_target,
};
use crate::maturity_continuity::{
    MaturityByteComparison, MaturityByteSource, MaturityContinuityRefusal, witness,
};
use crate::maturity_corpus::ValidatedMaturityCorpus;
use crate::maturity_history_report::MaturitySyntheticOriginResidual;
use crate::maturity_native::{MaturityAcceptanceObligation, MaturityAcceptanceRoute};
use crate::maturity_recovery::{
    MaturityRecoveryRefusal, PublicAnnouncementHandoff, PublicAnnouncementLocator,
    PublicHandoffInput, recover_public_successor,
};
use crate::maturity_report::MaturitySafetyCompleteness;
use crate::maturity_safety::{
    MaturityIntendedCarrier, MaturitySafetyRow, MaturitySafetySection, rows_of,
};

/// Canonical schema for a public-recovery report.
pub const MATURITY_PUBLIC_RECOVERY_REPORT_SCHEMA: u32 = 1;

/// The distinct evidence role of public successor recovery.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MaturityPublicRecoveryReportRole {
    StatePublicRecovery,
}

impl MaturityPublicRecoveryReportRole {
    /// Canonical role name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::StatePublicRecovery => "state-public-recovery",
        }
    }
}

/// Limits retained beside a successful public reconstruction.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MaturityPublicRecoveryResidual {
    SyntheticOrigin(MaturitySyntheticOriginResidual),
    WitnessedNonceWithoutLeastness,
}

impl MaturityPublicRecoveryResidual {
    /// Canonical residual order.
    pub const ALL: &'static [Self; 4] = &[
        Self::SyntheticOrigin(MaturitySyntheticOriginResidual::NoChainObservation),
        Self::SyntheticOrigin(MaturitySyntheticOriginResidual::NoRootCursorFreshness),
        Self::SyntheticOrigin(MaturitySyntheticOriginResidual::NoArchiveOriginBeyondAdmission),
        Self::WitnessedNonceWithoutLeastness,
    ];

    /// Canonical residual name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::SyntheticOrigin(residual) => residual.name(),
            Self::WitnessedNonceWithoutLeastness => "witnessed-nonce-without-leastness",
        }
    }
}

/// A construction fact about the constrained handoff producer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PublicRecoveryConstructionProof {
    HandoffNamesNoRetainedInstance,
    RecoveryTakesTheHandoffAlone,
    HandoffNamesNoDescriptorOrOperatorKey,
    HandoffFieldsAreThePublicRoster,
}

impl PublicRecoveryConstructionProof {
    /// Canonical proof name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::HandoffNamesNoRetainedInstance => "handoff-names-no-retained-instance",
            Self::RecoveryTakesTheHandoffAlone => "recovery-takes-the-handoff-alone",
            Self::HandoffNamesNoDescriptorOrOperatorKey => {
                "handoff-names-no-descriptor-or-operator-key"
            }
            Self::HandoffFieldsAreThePublicRoster => "handoff-fields-are-the-public-roster",
        }
    }
}

/// How one public-recovery row is answered.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MaturityPublicRecoveryRowAnswer {
    RecoveredFromTheHandoff,
    RefusedOnItsOwnAxis,
    ConstructionProof(PublicRecoveryConstructionProof),
}

impl MaturityPublicRecoveryRowAnswer {
    /// Canonical answer name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::RecoveredFromTheHandoff => "recovered-from-the-handoff",
            Self::RefusedOnItsOwnAxis => "refused-on-its-own-axis",
            Self::ConstructionProof(proof) => proof.name(),
        }
    }
}

/// One matrix row and the mechanism that answers it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MaturityPublicRecoveryRow {
    section: MaturitySafetySection,
    row: &'static str,
    answer: MaturityPublicRecoveryRowAnswer,
}

impl MaturityPublicRecoveryRow {
    /// Matrix section.
    #[must_use]
    pub const fn section(&self) -> MaturitySafetySection {
        self.section
    }

    /// Matrix row name.
    #[must_use]
    pub const fn row(&self) -> &'static str {
        self.row
    }

    /// Answering mechanism.
    #[must_use]
    pub const fn answer(&self) -> MaturityPublicRecoveryRowAnswer {
        self.answer
    }
}

/// Counts recounted from witness, handoff roster and matrix rows.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MaturityPublicRecoveryReportCensus {
    witness_items: usize,
    public_inputs: usize,
    rows: usize,
}

impl MaturityPublicRecoveryReportCensus {
    #[must_use]
    pub const fn witness_items(&self) -> usize {
        self.witness_items
    }
    #[must_use]
    pub const fn public_inputs(&self) -> usize {
        self.public_inputs
    }
    #[must_use]
    pub const fn rows(&self) -> usize {
        self.rows
    }
}

/// A stated recovery over the public handoff alone.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaturityPublicRecoveryReport {
    schema: u32,
    role: MaturityPublicRecoveryReportRole,
    handoff: PublicAnnouncementHandoff,
    public_witness: Vec<Vec<u8>>,
    predecessor_metadata: EncodedStateMetadata,
    successor_semantics: StateMetadata,
    witnessed_representation: StateRepresentationNonce,
    requested_cycle: Cycle,
    program: MaturityByteComparison,
    contract: TargetContractVersion,
    public_inputs: Vec<PublicHandoffInput>,
    rows: Vec<MaturityPublicRecoveryRow>,
    residuals: Vec<MaturityPublicRecoveryResidual>,
    census: MaturityPublicRecoveryReportCensus,
    completeness: MaturitySafetyCompleteness,
}

/// Recovery claims checked against an independently received handoff.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidatedMaturityPublicRecoveryReport {
    report: MaturityPublicRecoveryReport,
    recomputed_items: BTreeSet<MaturityPublicRecoveryRecomputedItem>,
}

/// Every claim validated before canonical bytes are rendered.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MaturityPublicRecoveryRecomputedItem {
    Schema,
    Role,
    Recovery,
    TransactionLocator,
    ExactTransactionBytes,
    StateOutputIndex,
    PublishedParameters,
    PublicWitnessFields,
    DecodedPredecessorMetadata,
    DerivedSuccessorMetadata,
    WitnessedRepresentationNonce,
    RequestedCycle,
    ReconstructedSuccessorProgram,
    ActualSuccessorProgram,
    EqualityResult,
    ContractRevision,
    AbsenceOfCreatorPrivateDependencies,
    RowRegistry,
    ChainObservationResidual,
    CursorFreshnessResidual,
    ArchiveOriginResidual,
    LeastnessResidual,
    CensusWitnessItems,
    CensusPublicInputs,
    CensusRows,
    Completeness,
}

impl MaturityPublicRecoveryRecomputedItem {
    /// Validation order.
    pub const ALL: &'static [Self; 26] = &[
        Self::Schema,
        Self::Role,
        Self::Recovery,
        Self::TransactionLocator,
        Self::ExactTransactionBytes,
        Self::StateOutputIndex,
        Self::PublishedParameters,
        Self::PublicWitnessFields,
        Self::DecodedPredecessorMetadata,
        Self::DerivedSuccessorMetadata,
        Self::WitnessedRepresentationNonce,
        Self::RequestedCycle,
        Self::ReconstructedSuccessorProgram,
        Self::ActualSuccessorProgram,
        Self::EqualityResult,
        Self::ContractRevision,
        Self::AbsenceOfCreatorPrivateDependencies,
        Self::RowRegistry,
        Self::ChainObservationResidual,
        Self::CursorFreshnessResidual,
        Self::ArchiveOriginResidual,
        Self::LeastnessResidual,
        Self::CensusWitnessItems,
        Self::CensusPublicInputs,
        Self::CensusRows,
        Self::Completeness,
    ];

    /// Canonical item name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Schema => "schema",
            Self::Role => "role",
            Self::Recovery => "recovery",
            Self::TransactionLocator => "transaction-locator",
            Self::ExactTransactionBytes => "exact-transaction-bytes",
            Self::StateOutputIndex => "state-output-index",
            Self::PublishedParameters => "published-parameters",
            Self::PublicWitnessFields => "public-witness-fields",
            Self::DecodedPredecessorMetadata => "decoded-predecessor-metadata",
            Self::DerivedSuccessorMetadata => "derived-successor-metadata",
            Self::WitnessedRepresentationNonce => "witnessed-representation-nonce",
            Self::RequestedCycle => "requested-cycle",
            Self::ReconstructedSuccessorProgram => "reconstructed-successor-program",
            Self::ActualSuccessorProgram => "actual-successor-program",
            Self::EqualityResult => "equality-result",
            Self::ContractRevision => "contract-revision",
            Self::AbsenceOfCreatorPrivateDependencies => "absence-of-creator-private-dependencies",
            Self::RowRegistry => "row-registry",
            Self::ChainObservationResidual => "chain-observation-residual",
            Self::CursorFreshnessResidual => "cursor-freshness-residual",
            Self::ArchiveOriginResidual => "archive-origin-residual",
            Self::LeastnessResidual => "leastness-residual",
            Self::CensusWitnessItems => "census-witness-items",
            Self::CensusPublicInputs => "census-public-inputs",
            Self::CensusRows => "census-rows",
            Self::Completeness => "completeness",
        }
    }
}

/// First failed report comparison or public reading.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MaturityPublicRecoveryReportRefusal {
    UnsupportedSchema {
        stated: u32,
    },
    RecoveryRefused {
        refusal: Box<MaturityRecoveryRefusal>,
    },
    PublicWitnessUnreadable {
        refusal: Box<MaturityContinuityRefusal>,
    },
    RowWithoutAnAnswer {
        section: MaturitySafetySection,
        row: &'static str,
    },
    AcceptedHandoffUnavailable {
        stage: Box<MaturityAcceptedHandoffStage>,
    },
    ItemDiffers {
        item: MaturityPublicRecoveryRecomputedItem,
    },
}

impl MaturityPublicRecoveryReportRefusal {
    /// The report item named by this refusal.
    #[must_use]
    pub const fn failed_item(&self) -> MaturityPublicRecoveryRecomputedItem {
        match self {
            Self::UnsupportedSchema { .. } => MaturityPublicRecoveryRecomputedItem::Schema,
            Self::RecoveryRefused { .. } => MaturityPublicRecoveryRecomputedItem::Recovery,
            Self::PublicWitnessUnreadable { .. } => {
                MaturityPublicRecoveryRecomputedItem::PublicWitnessFields
            }
            Self::RowWithoutAnAnswer { .. } => MaturityPublicRecoveryRecomputedItem::RowRegistry,
            Self::AcceptedHandoffUnavailable { .. } => {
                MaturityPublicRecoveryRecomputedItem::Recovery
            }
            Self::ItemDiffers { item } => *item,
        }
    }
}

/// The first reading that blocked construction of the admitted handoff.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MaturityAcceptedHandoffStage {
    AcceptanceOutstanding {
        routes: [MaturityAcceptanceRoute; 2],
    },
    TransactionUndecodable(TransactionRefusal),
    ReviewedTargetUnavailable(MaturityClosureRefusal),
    LeafAbsent {
        items: usize,
    },
    LeafUndecodable(TapscriptError),
    SubtreeRefused(StateConstructorRefusal),
    DeploymentUnavailable(MaturityClosureRefusal),
    LeadBoundsRefused(RealizationError),
    InternalKeyRefused(StateConstructorRefusal),
}

impl MaturityPublicRecoveryReport {
    #[must_use]
    pub const fn schema(&self) -> u32 {
        self.schema
    }
    #[must_use]
    pub const fn role(&self) -> MaturityPublicRecoveryReportRole {
        self.role
    }
    #[must_use]
    pub const fn handoff(&self) -> &PublicAnnouncementHandoff {
        &self.handoff
    }
    #[must_use]
    pub fn public_witness(&self) -> &[Vec<u8>] {
        &self.public_witness
    }
    #[must_use]
    pub const fn predecessor_metadata(&self) -> &EncodedStateMetadata {
        &self.predecessor_metadata
    }
    #[must_use]
    pub const fn successor_semantics(&self) -> &StateMetadata {
        &self.successor_semantics
    }
    #[must_use]
    pub const fn witnessed_representation(&self) -> StateRepresentationNonce {
        self.witnessed_representation
    }
    #[must_use]
    pub const fn requested_cycle(&self) -> Cycle {
        self.requested_cycle
    }
    #[must_use]
    pub const fn program(&self) -> &MaturityByteComparison {
        &self.program
    }
    #[must_use]
    pub const fn contract(&self) -> TargetContractVersion {
        self.contract
    }
    #[must_use]
    pub fn public_inputs(&self) -> &[PublicHandoffInput] {
        &self.public_inputs
    }
    #[must_use]
    pub fn rows(&self) -> &[MaturityPublicRecoveryRow] {
        &self.rows
    }
    #[must_use]
    pub fn residuals(&self) -> &[MaturityPublicRecoveryResidual] {
        &self.residuals
    }
    #[must_use]
    pub const fn census(&self) -> MaturityPublicRecoveryReportCensus {
        self.census
    }
    #[must_use]
    pub const fn completeness(&self) -> MaturitySafetyCompleteness {
        self.completeness
    }
}

impl ValidatedMaturityPublicRecoveryReport {
    #[must_use]
    pub const fn report(&self) -> &MaturityPublicRecoveryReport {
        &self.report
    }
    #[must_use]
    pub const fn recomputed_items(&self) -> &BTreeSet<MaturityPublicRecoveryRecomputedItem> {
        &self.recomputed_items
    }
    /// Whether the validated registry carries this matrix row.
    #[must_use]
    pub fn observes(&self, section: MaturitySafetySection, row: &str) -> bool {
        self.report
            .rows()
            .iter()
            .any(|entry| entry.section() == section && entry.row() == row)
    }
}

fn read_public_witness(
    handoff: &PublicAnnouncementHandoff,
) -> Result<Vec<Vec<u8>>, MaturityPublicRecoveryReportRefusal> {
    let transaction = TargetTransaction::decode(handoff.bytes()).map_err(|refusal| {
        MaturityPublicRecoveryReportRefusal::PublicWitnessUnreadable {
            refusal: Box::new(MaturityContinuityRefusal::Decode(Box::new(refusal))),
        }
    })?;
    witness(&transaction, handoff.schedule())
        .map(<[Vec<u8>]>::to_vec)
        .map_err(
            |refusal| MaturityPublicRecoveryReportRefusal::PublicWitnessUnreadable {
                refusal: Box::new(refusal),
            },
        )
}

fn answer_for_row(
    row: &'static MaturitySafetyRow,
) -> Result<MaturityPublicRecoveryRowAnswer, MaturityPublicRecoveryReportRefusal> {
    use MaturityPublicRecoveryRowAnswer as Answer;
    use PublicRecoveryConstructionProof as Proof;
    let answer = match row.name() {
        "public-successor-recovery" => Answer::RecoveredFromTheHandoff,
        "missing-transaction"
        | "copied-transaction-bytes"
        | "wrong-output-index"
        | "stale-successor"
        | "malformed-public-witness"
        | "metadata-from-another-successor"
        | "missing-representation-nonce"
        | "wrong-static-subtree"
        | "reconstructed-program-differs-from-chain-output" => Answer::RefusedOnItsOwnAxis,
        "creator-process-memory-required" => {
            Answer::ConstructionProof(Proof::HandoffNamesNoRetainedInstance)
        }
        "temporary-file-required" => Answer::ConstructionProof(Proof::RecoveryTakesTheHandoffAlone),
        "wallet-descriptor-required" => {
            Answer::ConstructionProof(Proof::HandoffNamesNoDescriptorOrOperatorKey)
        }
        "unknown-publication-field" => {
            Answer::ConstructionProof(Proof::HandoffFieldsAreThePublicRoster)
        }
        _ => {
            return Err(MaturityPublicRecoveryReportRefusal::RowWithoutAnAnswer {
                section: row.section(),
                row: row.name(),
            });
        }
    };
    Ok(answer)
}

/// The report-fault rows have the report carrier too, but they are not Positive
/// and do not enter the public-recovery registry.
fn collect_rows() -> Result<Vec<MaturityPublicRecoveryRow>, MaturityPublicRecoveryReportRefusal> {
    rows_of(MaturitySafetySection::Positive)
        .filter(|row| row.carrier() == MaturityIntendedCarrier::Report)
        .chain(rows_of(MaturitySafetySection::RecoveryFault))
        .map(|row| {
            Ok(MaturityPublicRecoveryRow {
                section: row.section(),
                row: row.name(),
                answer: answer_for_row(row)?,
            })
        })
        .collect()
}

/// Assemble the eleven public-recovery readings from the sole handoff input.
///
/// # Errors
/// Returns the recovery refusal, unreadable public witness or unmapped matrix row.
#[must_use = "public recovery can refuse assembly"]
pub fn assemble_maturity_public_recovery_report(
    handoff: PublicAnnouncementHandoff,
) -> Result<MaturityPublicRecoveryReport, MaturityPublicRecoveryReportRefusal> {
    let recovered = recover_public_successor(handoff.clone()).map_err(|refusal| {
        MaturityPublicRecoveryReportRefusal::RecoveryRefused {
            refusal: Box::new(refusal),
        }
    })?;
    let public_witness = read_public_witness(&handoff)?;
    let rows = collect_rows()?;
    let public_inputs = PublicHandoffInput::ALL.to_vec();
    let program = MaturityByteComparison::new(
        recovered.actual_program(),
        recovered.reconstructed_program(),
    );
    Ok(MaturityPublicRecoveryReport {
        schema: MATURITY_PUBLIC_RECOVERY_REPORT_SCHEMA,
        role: MaturityPublicRecoveryReportRole::StatePublicRecovery,
        handoff,
        census: MaturityPublicRecoveryReportCensus {
            witness_items: public_witness.len(),
            public_inputs: public_inputs.len(),
            rows: rows.len(),
        },
        public_witness,
        predecessor_metadata: *recovered.predecessor_metadata(),
        successor_semantics: *recovered.successor_semantics(),
        witnessed_representation: recovered.representation(),
        requested_cycle: recovered.requested_cycle(),
        program,
        contract: recovered.contract(),
        public_inputs,
        rows,
        residuals: MaturityPublicRecoveryResidual::ALL.to_vec(),
        completeness: MaturitySafetyCompleteness::PartialRequiredRowsOutstanding,
    })
}

fn verify_item(
    agrees: bool,
    item: MaturityPublicRecoveryRecomputedItem,
    checked: &mut BTreeSet<MaturityPublicRecoveryRecomputedItem>,
) -> Result<(), MaturityPublicRecoveryReportRefusal> {
    if !agrees {
        return Err(MaturityPublicRecoveryReportRefusal::ItemDiffers { item });
    }
    checked.insert(item);
    Ok(())
}

/// Recompute every stated item from an independently received handoff.
///
/// # Errors
/// Returns the first failed item, public reading, or recovery refusal.
#[must_use = "report validation can refuse a stated claim"]
#[expect(
    clippy::too_many_lines,
    reason = "The report's ordered comparisons remain visible together."
)]
pub fn validate_maturity_public_recovery_report(
    report: &MaturityPublicRecoveryReport,
    handoff: &PublicAnnouncementHandoff,
) -> Result<ValidatedMaturityPublicRecoveryReport, MaturityPublicRecoveryReportRefusal> {
    use MaturityPublicRecoveryRecomputedItem as Item;
    if report.schema != MATURITY_PUBLIC_RECOVERY_REPORT_SCHEMA {
        return Err(MaturityPublicRecoveryReportRefusal::UnsupportedSchema {
            stated: report.schema,
        });
    }
    let mut checked = BTreeSet::new();
    checked.insert(Item::Schema);
    let recovered = recover_public_successor(handoff.clone()).map_err(|refusal| {
        MaturityPublicRecoveryReportRefusal::RecoveryRefused {
            refusal: Box::new(refusal),
        }
    })?;
    checked.insert(Item::Recovery);
    verify_item(
        report.role == MaturityPublicRecoveryReportRole::StatePublicRecovery,
        Item::Role,
        &mut checked,
    )?;
    verify_item(
        report.handoff.locator() == handoff.locator(),
        Item::TransactionLocator,
        &mut checked,
    )?;
    verify_item(
        report.handoff.bytes() == handoff.bytes(),
        Item::ExactTransactionBytes,
        &mut checked,
    )?;
    verify_item(
        report.handoff.state_output_index() == handoff.state_output_index(),
        Item::StateOutputIndex,
        &mut checked,
    )?;
    verify_item(
        report.handoff.schedule() == handoff.schedule()
            && report.handoff.bounds() == handoff.bounds()
            && report.handoff.static_subtree() == handoff.static_subtree()
            && report.handoff.internal_key() == handoff.internal_key()
            && report.handoff.contract() == handoff.contract(),
        Item::PublishedParameters,
        &mut checked,
    )?;
    let public_witness = read_public_witness(handoff)?;
    verify_item(
        report.public_witness == public_witness,
        Item::PublicWitnessFields,
        &mut checked,
    )?;
    verify_item(
        report.predecessor_metadata == *recovered.predecessor_metadata(),
        Item::DecodedPredecessorMetadata,
        &mut checked,
    )?;
    verify_item(
        report.successor_semantics == *recovered.successor_semantics(),
        Item::DerivedSuccessorMetadata,
        &mut checked,
    )?;
    verify_item(
        report.witnessed_representation == recovered.representation(),
        Item::WitnessedRepresentationNonce,
        &mut checked,
    )?;
    verify_item(
        report.requested_cycle == recovered.requested_cycle(),
        Item::RequestedCycle,
        &mut checked,
    )?;
    verify_item(
        report.program.reconstructed() == recovered.reconstructed_program(),
        Item::ReconstructedSuccessorProgram,
        &mut checked,
    )?;
    verify_item(
        report.program.witnessed() == recovered.actual_program(),
        Item::ActualSuccessorProgram,
        &mut checked,
    )?;
    verify_item(
        report.program.agrees() == recovered.programs_agree(),
        Item::EqualityResult,
        &mut checked,
    )?;
    verify_item(
        report.contract == recovered.contract(),
        Item::ContractRevision,
        &mut checked,
    )?;
    verify_item(
        report.public_inputs() == PublicHandoffInput::ALL.as_slice(),
        Item::AbsenceOfCreatorPrivateDependencies,
        &mut checked,
    )?;
    let rows = collect_rows()?;
    verify_item(report.rows == rows, Item::RowRegistry, &mut checked)?;
    verify_item(
        report.residuals.len() == MaturityPublicRecoveryResidual::ALL.len()
            && report.residuals.first() == MaturityPublicRecoveryResidual::ALL.first(),
        Item::ChainObservationResidual,
        &mut checked,
    )?;
    verify_item(
        report.residuals.get(1) == MaturityPublicRecoveryResidual::ALL.get(1),
        Item::CursorFreshnessResidual,
        &mut checked,
    )?;
    verify_item(
        report.residuals.get(2) == MaturityPublicRecoveryResidual::ALL.get(2),
        Item::ArchiveOriginResidual,
        &mut checked,
    )?;
    verify_item(
        report.residuals.get(3) == MaturityPublicRecoveryResidual::ALL.get(3),
        Item::LeastnessResidual,
        &mut checked,
    )?;
    verify_item(
        report.census.witness_items == public_witness.len(),
        Item::CensusWitnessItems,
        &mut checked,
    )?;
    verify_item(
        report.census.public_inputs == PublicHandoffInput::ALL.len(),
        Item::CensusPublicInputs,
        &mut checked,
    )?;
    verify_item(
        report.census.rows == rows.len(),
        Item::CensusRows,
        &mut checked,
    )?;
    verify_item(
        report.completeness == MaturitySafetyCompleteness::PartialRequiredRowsOutstanding,
        Item::Completeness,
        &mut checked,
    )?;
    Ok(ValidatedMaturityPublicRecoveryReport {
        report: report.clone(),
        recomputed_items: checked,
    })
}

fn hex(bytes: &[u8]) -> String {
    let mut text = String::new();
    for byte in bytes {
        let _ = write!(text, "{byte:02x}");
    }
    text
}

/// Render canonical lowercase bytes from a validated public-recovery report.
#[must_use]
pub fn render_maturity_public_recovery_report(
    validated: &ValidatedMaturityPublicRecoveryReport,
) -> String {
    let report = validated.report();
    let handoff = report.handoff();
    let locator = handoff.locator();
    let (source, address) = match locator.source() {
        MaturityByteSource::NodeFreeSubmitReady => ("node-free-submit-ready", "-".to_owned()),
        MaturityByteSource::ArchivedSubmission { run_address } => {
            ("archived-submission", hex(run_address.as_bytes()))
        }
    };
    let mut text = String::new();
    let _ = writeln!(text, "schema {}", report.schema());
    let _ = writeln!(text, "role {}", report.role().name());
    let _ = writeln!(text, "operation announce-maturity");
    let _ = writeln!(
        text,
        "transaction_locator {source} {address} {}",
        locator.identity().to_target_display()
    );
    let _ = writeln!(text, "transaction_bytes {}", hex(handoff.bytes()));
    let _ = writeln!(text, "state_output_index {}", handoff.state_output_index());
    let _ = writeln!(text, "witness_schedule {}", handoff.schedule().name());
    let _ = writeln!(
        text,
        "lead_bounds {} {}",
        handoff.bounds().minimum().get(),
        handoff.bounds().maximum().get()
    );
    let _ = writeln!(text, "static_root {}", hex(handoff.static_subtree().root()));
    let _ = writeln!(text, "internal_key {}", hex(handoff.internal_key().key()));
    let _ = writeln!(text, "contract_revision {}", report.contract().get());
    for (index, item) in report.public_witness().iter().enumerate() {
        let _ = writeln!(text, "witness_item {index} {}", hex(item));
    }
    let predecessor = report.predecessor_metadata();
    let _ = writeln!(
        text,
        "predecessor_metadata {}",
        hex(&encode_state_metadata(
            &predecessor.semantic,
            predecessor.representation
        ))
    );
    let _ = writeln!(
        text,
        "successor_metadata {}",
        hex(&encode_state_metadata(
            report.successor_semantics(),
            report.witnessed_representation()
        ))
    );
    let _ = writeln!(
        text,
        "witnessed_representation_nonce {}",
        report.witnessed_representation().get()
    );
    let _ = writeln!(text, "requested_cycle {}", report.requested_cycle().get());
    let _ = writeln!(
        text,
        "reconstructed_program {}",
        hex(report.program().reconstructed())
    );
    let _ = writeln!(text, "actual_program {}", hex(report.program().witnessed()));
    let _ = writeln!(text, "programs_agree {}", report.program().agrees());
    for input in report.public_inputs() {
        let _ = writeln!(text, "public_input {}", hex(input.item().as_bytes()));
    }
    for row in report.rows() {
        let _ = writeln!(
            text,
            "row {} {} {}",
            row.section().section(),
            row.row(),
            row.answer().name()
        );
    }
    for residual in report.residuals() {
        let _ = writeln!(text, "residual {}", residual.name());
    }
    let _ = writeln!(text, "witness_items {}", report.census().witness_items());
    let _ = writeln!(text, "public_inputs {}", report.census().public_inputs());
    let _ = writeln!(text, "rows {}", report.census().rows());
    let _ = writeln!(text, "completeness {}", report.completeness().name());
    for item in MaturityPublicRecoveryRecomputedItem::ALL {
        if validated.recomputed_items().contains(item) {
            let _ = writeln!(text, "recomputed {}", item.name());
        }
    }
    text.make_ascii_lowercase();
    text
}

/// Build the admitted archive's public handoff for report derivation.
///
/// This builder reads an admitted archive; the constrained producer receives
/// only the resulting handoff.
///
/// # Errors
/// Returns the first unavailable accepted reading with its stage.
#[must_use = "an admitted handoff can be unavailable"]
pub fn accepted_public_handoff(
    corpus: &ValidatedMaturityCorpus,
) -> Result<PublicAnnouncementHandoff, MaturityPublicRecoveryReportRefusal> {
    use MaturityAcceptedHandoffStage as Stage;
    let readback = match corpus.evidence().acceptance_obligation() {
        MaturityAcceptanceObligation::Established { readback, .. } => readback,
        MaturityAcceptanceObligation::Outstanding { routes } => {
            return Err(
                MaturityPublicRecoveryReportRefusal::AcceptedHandoffUnavailable {
                    stage: Box::new(Stage::AcceptanceOutstanding { routes: *routes }),
                },
            );
        }
    };
    let transaction = TargetTransaction::decode(readback.bytes()).map_err(|refusal| {
        MaturityPublicRecoveryReportRefusal::AcceptedHandoffUnavailable {
            stage: Box::new(Stage::TransactionUndecodable(refusal)),
        }
    })?;
    let target = closure_target().map_err(|refusal| {
        MaturityPublicRecoveryReportRefusal::AcceptedHandoffUnavailable {
            stage: Box::new(Stage::ReviewedTargetUnavailable(refusal)),
        }
    })?;
    let stack = transaction
        .witnesses()
        .first()
        .map_or(&[][..], |input| input.stack());
    let leaf = stack.get(7).ok_or_else(|| {
        MaturityPublicRecoveryReportRefusal::AcceptedHandoffUnavailable {
            stage: Box::new(Stage::LeafAbsent { items: stack.len() }),
        }
    })?;
    let program = TapscriptProgram::decode(&target, leaf).map_err(|refusal| {
        MaturityPublicRecoveryReportRefusal::AcceptedHandoffUnavailable {
            stage: Box::new(Stage::LeafUndecodable(refusal)),
        }
    })?;
    let subtree = StateStaticSubtree::new(
        &target,
        Some(StateStaticNode::Leaf {
            identity: 0,
            leaf: StateStaticLeaf {
                role: StateLeafRole::Announcement,
                program,
                version: target.definition().leaf_version().get(),
            },
        }),
    )
    .map_err(
        |refusal| MaturityPublicRecoveryReportRefusal::AcceptedHandoffUnavailable {
            stage: Box::new(Stage::SubtreeRefused(refusal)),
        },
    )?;
    let (minimum, maximum) = MaturityDeployment::PublishedSignerHeld
        .parameters()
        .map_err(
            |refusal| MaturityPublicRecoveryReportRefusal::AcceptedHandoffUnavailable {
                stage: Box::new(Stage::DeploymentUnavailable(refusal)),
            },
        )?
        .lead();
    let bounds = AnnouncementLeadBounds::new(Cycle::new(minimum), Cycle::new(maximum)).map_err(
        |refusal| MaturityPublicRecoveryReportRefusal::AcceptedHandoffUnavailable {
            stage: Box::new(Stage::LeadBoundsRefused(refusal)),
        },
    )?;
    let internal_key =
        StateInternalKeyPolicy::new(STATE_NUMS_KEY, &OracleStateCurve).map_err(|refusal| {
            MaturityPublicRecoveryReportRefusal::AcceptedHandoffUnavailable {
                stage: Box::new(Stage::InternalKeyRefused(refusal)),
            }
        })?;
    Ok(PublicAnnouncementHandoff::new(
        PublicAnnouncementLocator::new(
            MaturityByteSource::ArchivedSubmission {
                run_address: corpus.report().run_address().to_owned(),
            },
            readback.identity(),
        ),
        readback.bytes().to_vec(),
        0,
        corpus.schedule(),
        bounds,
        subtree,
        internal_key,
        TargetContractVersion::V2,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::maturity_continuity::project_maturity_continuity;
    use crate::maturity_continuity::tests::variable_archived;
    use crate::maturity_corpus::maturity_variable_run_of_record;
    use crate::maturity_evidence::{
        MaturityConstructorMaterial, MaturityConstructorMaterialAbsence,
        MaturityExecutorProvenanceExpectation, derive_maturity_evidence_plan_with,
    };
    use crate::maturity_history::{MaturityRootCheckpoint, StateRootEdge};
    use crate::maturity_history_report::{
        assemble_maturity_root_history_report, render_maturity_root_history_report,
        validate_maturity_root_history_report,
    };
    use crate::maturity_report::MaturityVolatileField;

    fn accepted() -> (PublicAnnouncementHandoff, MaturityPublicRecoveryReport) {
        let corpus = maturity_variable_run_of_record().expect("accepted archive");
        let handoff = accepted_public_handoff(corpus).expect("accepted public handoff");
        let report =
            assemble_maturity_public_recovery_report(handoff.clone()).expect("report assembles");
        (handoff, report)
    }

    fn validate(
        report: &MaturityPublicRecoveryReport,
        handoff: &PublicAnnouncementHandoff,
    ) -> Result<ValidatedMaturityPublicRecoveryReport, MaturityPublicRecoveryReportRefusal> {
        validate_maturity_public_recovery_report(report, handoff)
    }

    fn restated(
        handoff: &PublicAnnouncementHandoff,
        locator: PublicAnnouncementLocator,
        index: u32,
    ) -> PublicAnnouncementHandoff {
        PublicAnnouncementHandoff::new(
            locator,
            handoff.bytes().to_vec(),
            index,
            handoff.schedule(),
            handoff.bounds(),
            handoff.static_subtree().clone(),
            handoff.internal_key(),
            handoff.contract(),
        )
    }

    #[test]
    fn the_accepted_handoffs_recovery_report_validates_and_renders_four_residuals_in_order() {
        let (handoff, report) = accepted();
        let validated = validate(&report, &handoff).expect("accepted report validates");
        assert_eq!(validated.recomputed_items().len(), 26);
        let rendered = render_maturity_public_recovery_report(&validated);
        let residuals: Vec<_> = rendered
            .lines()
            .filter(|line| line.starts_with("residual "))
            .collect();
        assert_eq!(
            residuals,
            [
                "residual no-chain-observation",
                "residual no-root-cursor-freshness",
                "residual no-archive-origin-beyond-admission",
                "residual witnessed-nonce-without-leastness",
            ]
        );
    }

    #[test]
    fn the_report_states_all_eleven_items_as_the_recovery_recomputes_them() {
        let (handoff, report) = accepted();
        let recovered = recover_public_successor(handoff.clone()).expect("public recovery");
        assert_eq!(report.handoff().locator(), handoff.locator());
        assert_eq!(report.handoff().bytes(), handoff.bytes());
        assert_eq!(
            report.handoff().state_output_index(),
            handoff.state_output_index()
        );
        assert_eq!(report.public_witness().len(), 9);
        assert_eq!(
            report.public_witness(),
            read_public_witness(&handoff).expect("published witness")
        );
        assert_eq!(
            report.predecessor_metadata(),
            recovered.predecessor_metadata()
        );
        assert_eq!(
            report.successor_semantics(),
            recovered.successor_semantics()
        );
        let witnessed_nonce = u32::from_be_bytes(
            report.public_witness()[1]
                .as_slice()
                .try_into()
                .expect("nonce width"),
        );
        assert_eq!(report.witnessed_representation().get(), witnessed_nonce);
        assert_eq!(
            report.witnessed_representation(),
            recovered.representation()
        );
        assert_eq!(
            report.program().reconstructed(),
            recovered.reconstructed_program()
        );
        assert_eq!(report.program().witnessed(), recovered.actual_program());
        assert_eq!(report.program().agrees(), recovered.programs_agree());
        assert_eq!(report.public_inputs(), PublicHandoffInput::ALL);
    }

    #[test]
    fn a_stated_item_differing_from_its_recomputation_fails_at_that_item() {
        use MaturityPublicRecoveryRecomputedItem as Item;
        let (handoff, report) = accepted();
        let mut changed = report.clone();
        changed.witnessed_representation =
            StateRepresentationNonce::new(report.witnessed_representation().get() + 1);
        assert_eq!(
            validate(&changed, &handoff),
            Err(MaturityPublicRecoveryReportRefusal::ItemDiffers {
                item: Item::WitnessedRepresentationNonce,
            })
        );
        let mut changed = report.clone();
        changed.public_witness[0].push(0);
        assert_eq!(
            validate(&changed, &handoff),
            Err(MaturityPublicRecoveryReportRefusal::ItemDiffers {
                item: Item::PublicWitnessFields,
            })
        );
        let mut changed = report.clone();
        changed.public_inputs.pop();
        assert_eq!(
            validate(&changed, &handoff),
            Err(MaturityPublicRecoveryReportRefusal::ItemDiffers {
                item: Item::AbsenceOfCreatorPrivateDependencies,
            })
        );
        let mut changed = report;
        changed.census.rows += 1;
        assert_eq!(
            validate(&changed, &handoff),
            Err(MaturityPublicRecoveryReportRefusal::ItemDiffers {
                item: Item::CensusRows,
            })
        );
    }

    #[test]
    fn a_report_over_another_handoff_fails_at_its_item_and_a_refused_handoff_at_recovery() {
        let (handoff, report) = accepted();
        let mut changed = report;
        changed.handoff = restated(
            &handoff,
            PublicAnnouncementLocator::new(
                MaturityByteSource::NodeFreeSubmitReady,
                handoff.locator().identity(),
            ),
            handoff.state_output_index(),
        );
        assert_eq!(
            validate(&changed, &handoff),
            Err(MaturityPublicRecoveryReportRefusal::ItemDiffers {
                item: MaturityPublicRecoveryRecomputedItem::TransactionLocator,
            })
        );
        let wrong_index = restated(&handoff, handoff.locator().clone(), 1);
        assert!(matches!(
            validate(&changed, &wrong_index),
            Err(MaturityPublicRecoveryReportRefusal::RecoveryRefused { refusal })
                if matches!(*refusal, MaturityRecoveryRefusal::WrongOutputIndex { stated: 1, .. })
        ));
    }

    #[test]
    fn removed_and_duplicated_rows_refuse_at_the_row_registry_item() {
        let (handoff, report) = accepted();
        let mut removed = report.clone();
        removed.rows.pop();
        let expected = Err(MaturityPublicRecoveryReportRefusal::ItemDiffers {
            item: MaturityPublicRecoveryRecomputedItem::RowRegistry,
        });
        assert_eq!(validate(&removed, &handoff), expected);
        let mut duplicated = report;
        duplicated.rows.push(duplicated.rows[0]);
        assert_eq!(validate(&duplicated, &handoff), expected);
    }

    #[test]
    fn the_row_registry_answers_exactly_the_fourteen_rows_by_their_own_mechanism() {
        let (_, report) = accepted();
        let expected: BTreeSet<_> = rows_of(MaturitySafetySection::Positive)
            .filter(|row| row.carrier() == MaturityIntendedCarrier::Report)
            .chain(rows_of(MaturitySafetySection::RecoveryFault))
            .map(|row| (row.section(), row.name()))
            .collect();
        let actual: BTreeSet<_> = report
            .rows()
            .iter()
            .map(|row| (row.section(), row.row()))
            .collect();
        assert_eq!(report.rows().len(), 14);
        assert_eq!(expected.len(), 14);
        assert_eq!(actual, expected);
        assert!(
            report
                .rows()
                .iter()
                .all(|row| expected.contains(&(row.section(), row.row())))
        );
        assert!(expected.iter().all(|item| actual.contains(item)));
        let recovered = report
            .rows()
            .iter()
            .filter(|row| row.answer() == MaturityPublicRecoveryRowAnswer::RecoveredFromTheHandoff)
            .count();
        let refused = report
            .rows()
            .iter()
            .filter(|row| row.answer() == MaturityPublicRecoveryRowAnswer::RefusedOnItsOwnAxis)
            .count();
        let proofs: BTreeSet<_> = report
            .rows()
            .iter()
            .filter_map(|row| match row.answer() {
                MaturityPublicRecoveryRowAnswer::ConstructionProof(proof) => Some(proof.name()),
                _ => None,
            })
            .collect();
        assert_eq!(recovered, 1);
        assert_eq!(refused, 9);
        assert_eq!(proofs.len(), 4);
        assert_eq!(
            report
                .rows()
                .iter()
                .filter(|row| row.row() == "reconstructed-program-differs-from-chain-output")
                .count(),
            1
        );
    }

    #[test]
    fn an_unread_schema_refuses_first() {
        let (handoff, mut report) = accepted();
        report.schema += 1;
        let wrong_index = restated(&handoff, handoff.locator().clone(), 1);
        assert_eq!(
            validate(&report, &wrong_index),
            Err(MaturityPublicRecoveryReportRefusal::UnsupportedSchema { stated: 2 })
        );
    }

    #[test]
    fn the_canonical_bytes_carry_no_volatile_key_and_no_raw_text() {
        let (handoff, report) = accepted();
        let validated = validate(&report, &handoff).expect("accepted report validates");
        let rendered = render_maturity_public_recovery_report(&validated);
        for volatile in MaturityVolatileField::ALL {
            assert!(
                !rendered
                    .lines()
                    .any(|line| line.starts_with(volatile.name()))
            );
        }
        assert!(
            rendered
                .bytes()
                .all(|byte| byte.is_ascii() && !byte.is_ascii_uppercase())
        );
        if let MaturityByteSource::ArchivedSubmission { run_address } = handoff.locator().source() {
            assert!(!rendered.contains(run_address));
            assert!(rendered.contains(&hex(run_address.as_bytes())));
        }
        for input in PublicHandoffInput::ALL {
            assert!(!rendered.contains(input.item()));
            assert!(rendered.contains(&hex(input.item().as_bytes())));
        }
    }

    #[test]
    fn the_recomputation_inventory_is_complete_and_distinctly_named() {
        let names: BTreeSet<_> = MaturityPublicRecoveryRecomputedItem::ALL
            .iter()
            .map(|item| item.name())
            .collect();
        assert_eq!(MaturityPublicRecoveryRecomputedItem::ALL.len(), 26);
        assert_eq!(names.len(), 26);
        let (handoff, report) = accepted();
        let validated = validate(&report, &handoff).expect("accepted report validates");
        let expected: BTreeSet<_> = MaturityPublicRecoveryRecomputedItem::ALL
            .iter()
            .copied()
            .collect();
        assert_eq!(validated.recomputed_items(), &expected);
    }

    #[test]
    fn assembly_takes_the_handoff_alone() {
        let assembler: fn(
            PublicAnnouncementHandoff,
        ) -> Result<
            MaturityPublicRecoveryReport,
            MaturityPublicRecoveryReportRefusal,
        > = assemble_maturity_public_recovery_report;
        let (handoff, _) = accepted();
        assert!(assembler(handoff).is_ok());
    }

    #[test]
    fn no_role_spelling_crosses_between_the_reports() {
        let (handoff, report) = accepted();
        let validated = validate(&report, &handoff).expect("recovery report validates");
        let recovery = render_maturity_public_recovery_report(&validated);

        let plan = derive_maturity_evidence_plan_with(
            MaturityExecutorProvenanceExpectation::NotStatedByTheOperator,
            MaturityConstructorMaterial::Absent(
                MaturityConstructorMaterialAbsence::NotSuppliedToDerivation,
            ),
        )
        .expect("root-history evidence plan");
        let continuity =
            project_maturity_continuity(variable_archived().input()).expect("accepted projection");
        let edge = StateRootEdge::from_continuity(&continuity);
        let (linked, abi) = crate::maturity_evidence::checkpoint_premises_of(&continuity)
            .expect("accepted premises");
        let checkpoint = MaturityRootCheckpoint::bind(
            &edge,
            &handoff,
            maturity_variable_run_of_record().expect("accepted corpus"),
            linked,
            abi,
        )
        .expect("accepted checkpoint");
        let history_report = assemble_maturity_root_history_report(
            std::slice::from_ref(&edge),
            edge.predecessor(),
            checkpoint,
            plan.root_history_mutations().clone(),
            plan.binding().clone(),
            plan.executor_provenance().clone(),
        )
        .expect("history report assembles");
        let history_validated = validate_maturity_root_history_report(
            &history_report,
            std::slice::from_ref(&edge),
            edge.predecessor(),
            &handoff,
            plan.root_history_mutations(),
            plan.binding(),
            plan.executor_provenance(),
        )
        .expect("history report validates");
        let history = render_maturity_root_history_report(&history_validated);
        assert!(recovery.contains("role state-public-recovery\n"));
        assert!(history.contains("role state-root-history\n"));
        for other in [
            "state-root-history",
            "state-constructor-continuity",
            "maturity-announcement-safety",
        ] {
            assert!(!recovery.contains(other));
        }
        for other in [
            MaturityPublicRecoveryReportRole::StatePublicRecovery.name(),
            "state-constructor-continuity",
            "maturity-announcement-safety",
        ] {
            assert!(!history.contains(other));
        }
        for residual in MaturitySyntheticOriginResidual::ALL {
            let line = format!("residual {}\n", residual.name());
            assert!(recovery.contains(&line));
            assert!(history.contains(&line));
        }
        assert!(recovery.contains("residual witnessed-nonce-without-leastness\n"));
        assert!(!history.contains("residual witnessed-nonce-without-leastness\n"));
    }
}
