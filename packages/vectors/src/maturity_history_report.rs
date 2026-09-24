//! A validated report over an ordered STATE root-history sequence (Guide 14 §14.6).
//!
//! The report preserves the supplied edge order, the stated and advanced cursors,
//! and the bound checkpoint. Its residuals state the limits of synthetic origin:
//! sequence validation does not observe a chain, authenticate cursor freshness,
//! or establish the admitted archive's origin beyond its pinned bytes.

use std::collections::BTreeSet;
use std::fmt::Write as _;

use transaction::bytes::Outpoint;

use crate::maturity_evidence::{
    MaturityExecutorProvenanceExpectation, MaturityMutationRegistry, MaturityMutationRegistryKind,
    MaturityTargetBinding,
};
use crate::maturity_history::{
    MaturityRootCheckpoint, MaturityRootCheckpointRefusal, MaturityRootFact, MaturityRootFactKind,
    MaturityRootHistoryRefusal, StateRootEdge, validate_state_root_history,
};
use crate::maturity_recovery::PublicAnnouncementHandoff;
use crate::maturity_report::MaturitySafetyCompleteness;
use crate::maturity_safety::MaturitySafetySection;

/// Canonical schema for the root-history report.
pub const MATURITY_ROOT_HISTORY_REPORT_SCHEMA: u32 = 1;

/// The root-history evidence role.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MaturityRootHistoryReportRole {
    StateRootHistory,
}

impl MaturityRootHistoryReportRole {
    /// The role's canonical name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::StateRootHistory => "state-root-history",
        }
    }
}

/// Limits that structural history and pinned archive admission cannot discharge.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MaturitySyntheticOriginResidual {
    NoChainObservation,
    NoRootCursorFreshness,
    NoArchiveOriginBeyondAdmission,
}

impl MaturitySyntheticOriginResidual {
    /// Canonical residual order.
    pub const ALL: &'static [Self; 3] = &[
        Self::NoChainObservation,
        Self::NoRootCursorFreshness,
        Self::NoArchiveOriginBeyondAdmission,
    ];

    /// The residual's canonical name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::NoChainObservation => "no-chain-observation",
            Self::NoRootCursorFreshness => "no-root-cursor-freshness",
            Self::NoArchiveOriginBeyondAdmission => "no-archive-origin-beyond-admission",
        }
    }
}

/// Counts recounted from the carried edges and the plan's registry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MaturityRootHistoryReportCensus {
    edges: usize,
    registered_mutations: usize,
}

impl MaturityRootHistoryReportCensus {
    #[must_use]
    pub const fn edges(&self) -> usize {
        self.edges
    }

    #[must_use]
    pub const fn registered_mutations(&self) -> usize {
        self.registered_mutations
    }
}

/// A claim over an ordered root-history sequence and its bound checkpoint.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaturityRootHistoryReport {
    schema: u32,
    role: MaturityRootHistoryReportRole,
    binding: MaturityTargetBinding,
    provenance: MaturityExecutorProvenanceExpectation,
    starting_cursor: Outpoint,
    edges: Vec<StateRootEdge>,
    final_cursor: Outpoint,
    checkpoint: MaturityRootCheckpoint,
    registry: MaturityMutationRegistry,
    residuals: Vec<MaturitySyntheticOriginResidual>,
    census: MaturityRootHistoryReportCensus,
    completeness: MaturitySafetyCompleteness,
}

/// A report whose claims have been recomputed against independent operands.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidatedMaturityRootHistoryReport {
    report: MaturityRootHistoryReport,
    recomputed_items: BTreeSet<MaturityRootHistoryRecomputedItem>,
}

/// Every comparison required before canonical history bytes may be rendered.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MaturityRootHistoryRecomputedItem {
    Schema,
    Role,
    TargetBinding,
    ExecutorProvenanceExpectation,
    StartingCursor,
    EdgeCount,
    EdgeOrder,
    EdgeSequence,
    FinalCursor,
    CheckpointBinding,
    CheckpointSuccessor,
    CheckpointPremiseProvenance,
    RegistryKind,
    RegistryRows,
    ChainObservationResidual,
    CursorFreshnessResidual,
    ArchiveOriginResidual,
    CensusEdges,
    CensusRegisteredMutations,
    Completeness,
}

impl MaturityRootHistoryRecomputedItem {
    /// The order in which validation establishes each claim.
    pub const ALL: &'static [Self; 20] = &[
        Self::Schema,
        Self::Role,
        Self::TargetBinding,
        Self::ExecutorProvenanceExpectation,
        Self::StartingCursor,
        Self::EdgeCount,
        Self::EdgeOrder,
        Self::EdgeSequence,
        Self::FinalCursor,
        Self::CheckpointBinding,
        Self::CheckpointSuccessor,
        Self::CheckpointPremiseProvenance,
        Self::RegistryKind,
        Self::RegistryRows,
        Self::ChainObservationResidual,
        Self::CursorFreshnessResidual,
        Self::ArchiveOriginResidual,
        Self::CensusEdges,
        Self::CensusRegisteredMutations,
        Self::Completeness,
    ];

    /// The item's canonical name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Schema => "schema",
            Self::Role => "role",
            Self::TargetBinding => "target-binding",
            Self::ExecutorProvenanceExpectation => "executor-provenance-expectation",
            Self::StartingCursor => "starting-cursor",
            Self::EdgeCount => "edge-count",
            Self::EdgeOrder => "edge-order",
            Self::EdgeSequence => "edge-sequence",
            Self::FinalCursor => "final-cursor",
            Self::CheckpointBinding => "checkpoint-binding",
            Self::CheckpointSuccessor => "checkpoint-successor",
            Self::CheckpointPremiseProvenance => "checkpoint-premise-provenance",
            Self::RegistryKind => "registry-kind",
            Self::RegistryRows => "registry-rows",
            Self::ChainObservationResidual => "chain-observation-residual",
            Self::CursorFreshnessResidual => "cursor-freshness-residual",
            Self::ArchiveOriginResidual => "archive-origin-residual",
            Self::CensusEdges => "census-edges",
            Self::CensusRegisteredMutations => "census-registered-mutations",
            Self::Completeness => "completeness",
        }
    }
}

/// The first claim that failed recomputation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MaturityRootHistoryReportRefusal {
    UnsupportedSchema {
        stated: u32,
    },
    EdgeCountDiffers {
        stated: usize,
        recomputed: usize,
    },
    EdgeOrderDiffers {
        position: usize,
    },
    EdgeSequenceRefused {
        refusal: Box<MaturityRootHistoryRefusal>,
    },
    CheckpointRefused {
        refusal: Box<MaturityRootCheckpointRefusal>,
    },
    ItemDiffers {
        item: MaturityRootHistoryRecomputedItem,
    },
}

impl MaturityRootHistoryReportRefusal {
    /// The item named by this refusal.
    #[must_use]
    pub const fn failed_item(&self) -> MaturityRootHistoryRecomputedItem {
        match self {
            Self::UnsupportedSchema { .. } => MaturityRootHistoryRecomputedItem::Schema,
            Self::EdgeCountDiffers { .. } => MaturityRootHistoryRecomputedItem::EdgeCount,
            Self::EdgeOrderDiffers { .. } => MaturityRootHistoryRecomputedItem::EdgeOrder,
            Self::EdgeSequenceRefused { .. } => MaturityRootHistoryRecomputedItem::EdgeSequence,
            Self::CheckpointRefused { .. } => MaturityRootHistoryRecomputedItem::CheckpointBinding,
            Self::ItemDiffers { item } => *item,
        }
    }
}

impl MaturityRootHistoryReport {
    #[must_use]
    pub const fn schema(&self) -> u32 {
        self.schema
    }
    #[must_use]
    pub const fn role(&self) -> MaturityRootHistoryReportRole {
        self.role
    }
    #[must_use]
    pub const fn binding(&self) -> &MaturityTargetBinding {
        &self.binding
    }
    #[must_use]
    pub const fn provenance(&self) -> &MaturityExecutorProvenanceExpectation {
        &self.provenance
    }
    #[must_use]
    pub const fn starting_cursor(&self) -> Outpoint {
        self.starting_cursor
    }
    #[must_use]
    pub fn edges(&self) -> &[StateRootEdge] {
        &self.edges
    }
    #[must_use]
    pub const fn final_cursor(&self) -> Outpoint {
        self.final_cursor
    }
    #[must_use]
    pub const fn checkpoint(&self) -> &MaturityRootCheckpoint {
        &self.checkpoint
    }
    #[must_use]
    pub const fn registry(&self) -> &MaturityMutationRegistry {
        &self.registry
    }
    #[must_use]
    pub fn residuals(&self) -> &[MaturitySyntheticOriginResidual] {
        &self.residuals
    }
    #[must_use]
    pub const fn census(&self) -> MaturityRootHistoryReportCensus {
        self.census
    }
    #[must_use]
    pub const fn completeness(&self) -> MaturitySafetyCompleteness {
        self.completeness
    }
}

impl ValidatedMaturityRootHistoryReport {
    #[must_use]
    pub const fn report(&self) -> &MaturityRootHistoryReport {
        &self.report
    }

    #[must_use]
    pub const fn recomputed_items(&self) -> &BTreeSet<MaturityRootHistoryRecomputedItem> {
        &self.recomputed_items
    }

    /// Whether the carried registry names a section and row together.
    #[must_use]
    pub fn observes(&self, section: MaturitySafetySection, row: &str) -> bool {
        self.report
            .registry()
            .mutations()
            .iter()
            .any(|mutation| mutation.section() == section && mutation.row() == row)
    }
}

/// Assemble a report over a validated ordered edge sequence.
///
/// # Errors
/// Returns the first history refusal if the supplied sequence cannot advance its cursor.
#[must_use = "root-history sequence validation can refuse assembly"]
pub fn assemble_maturity_root_history_report(
    edges: &[StateRootEdge],
    starting_cursor: Outpoint,
    checkpoint: MaturityRootCheckpoint,
    registry: MaturityMutationRegistry,
    binding: MaturityTargetBinding,
    provenance: MaturityExecutorProvenanceExpectation,
) -> Result<MaturityRootHistoryReport, MaturityRootHistoryReportRefusal> {
    let final_cursor = validate_state_root_history(edges, starting_cursor).map_err(|refusal| {
        MaturityRootHistoryReportRefusal::EdgeSequenceRefused {
            refusal: Box::new(refusal),
        }
    })?;
    let registered_mutations = registry.census().mutations();
    Ok(MaturityRootHistoryReport {
        schema: MATURITY_ROOT_HISTORY_REPORT_SCHEMA,
        role: MaturityRootHistoryReportRole::StateRootHistory,
        binding,
        provenance,
        starting_cursor,
        edges: edges.to_vec(),
        final_cursor,
        checkpoint,
        registry,
        residuals: MaturitySyntheticOriginResidual::ALL.to_vec(),
        census: MaturityRootHistoryReportCensus {
            edges: edges.len(),
            registered_mutations,
        },
        completeness: MaturitySafetyCompleteness::PartialRequiredRowsOutstanding,
    })
}

fn verify_item(
    agrees: bool,
    item: MaturityRootHistoryRecomputedItem,
    recomputed_items: &mut BTreeSet<MaturityRootHistoryRecomputedItem>,
) -> Result<(), MaturityRootHistoryReportRefusal> {
    if !agrees {
        return Err(MaturityRootHistoryReportRefusal::ItemDiffers { item });
    }
    recomputed_items.insert(item);
    Ok(())
}

/// Recompute every report claim against independent history and plan operands.
///
/// Schema, count and order are preflight checks before any payload comparison.
/// The edge validator runs before the final cursor is compared.
///
/// # Errors
/// Returns the first named disagreement, history refusal or checkpoint refusal.
#[must_use = "report validation can refuse a stated claim"]
#[expect(
    clippy::too_many_lines,
    reason = "The ordered report-item checks remain visible together."
)]
pub fn validate_maturity_root_history_report(
    report: &MaturityRootHistoryReport,
    edges: &[StateRootEdge],
    starting_cursor: Outpoint,
    handoff: &PublicAnnouncementHandoff,
    registry: &MaturityMutationRegistry,
    binding: &MaturityTargetBinding,
    provenance: &MaturityExecutorProvenanceExpectation,
) -> Result<ValidatedMaturityRootHistoryReport, MaturityRootHistoryReportRefusal> {
    use MaturityRootHistoryRecomputedItem as Item;

    if report.schema != MATURITY_ROOT_HISTORY_REPORT_SCHEMA {
        return Err(MaturityRootHistoryReportRefusal::UnsupportedSchema {
            stated: report.schema,
        });
    }
    if report.edges.len() != edges.len() {
        return Err(MaturityRootHistoryReportRefusal::EdgeCountDiffers {
            stated: report.edges.len(),
            recomputed: edges.len(),
        });
    }
    for (position, (stated, supplied)) in report.edges.iter().zip(edges).enumerate() {
        if stated != supplied {
            return Err(MaturityRootHistoryReportRefusal::EdgeOrderDiffers { position });
        }
    }

    let mut items = BTreeSet::new();
    items.insert(Item::Schema);
    verify_item(
        report.role == MaturityRootHistoryReportRole::StateRootHistory,
        Item::Role,
        &mut items,
    )?;
    verify_item(&report.binding == binding, Item::TargetBinding, &mut items)?;
    verify_item(
        &report.provenance == provenance,
        Item::ExecutorProvenanceExpectation,
        &mut items,
    )?;
    verify_item(
        report.starting_cursor == starting_cursor,
        Item::StartingCursor,
        &mut items,
    )?;
    items.insert(Item::EdgeCount);
    items.insert(Item::EdgeOrder);

    let advanced =
        validate_state_root_history(&report.edges, report.starting_cursor).map_err(|refusal| {
            MaturityRootHistoryReportRefusal::EdgeSequenceRefused {
                refusal: Box::new(refusal),
            }
        })?;
    items.insert(Item::EdgeSequence);
    verify_item(
        report.final_cursor == advanced,
        Item::FinalCursor,
        &mut items,
    )?;

    let last_edge = report.edges.last().ok_or_else(|| {
        MaturityRootHistoryReportRefusal::EdgeSequenceRefused {
            refusal: Box::new(MaturityRootHistoryRefusal::EmptySequence),
        }
    })?;
    report
        .checkpoint
        .binds(last_edge, handoff)
        .map_err(
            |refusal| MaturityRootHistoryReportRefusal::CheckpointRefused {
                refusal: Box::new(refusal),
            },
        )?;
    items.insert(Item::CheckpointBinding);
    verify_item(
        report.checkpoint.successor() == advanced,
        Item::CheckpointSuccessor,
        &mut items,
    )?;
    verify_item(
        MaturityRootFactKind::DeclaredPremise(report.checkpoint.linked_candidate().provenance())
            == MaturityRootFact::LinkedCandidate.kind()
            && MaturityRootFactKind::DeclaredPremise(
                report.checkpoint.candidate_abi().provenance(),
            ) == MaturityRootFact::CandidateAbi.kind(),
        Item::CheckpointPremiseProvenance,
        &mut items,
    )?;
    verify_item(
        report.registry.kind() == MaturityMutationRegistryKind::RootHistory,
        Item::RegistryKind,
        &mut items,
    )?;
    verify_item(&report.registry == registry, Item::RegistryRows, &mut items)?;

    verify_item(
        report.residuals.len() == MaturitySyntheticOriginResidual::ALL.len()
            && report.residuals.first() == MaturitySyntheticOriginResidual::ALL.first(),
        Item::ChainObservationResidual,
        &mut items,
    )?;
    verify_item(
        report.residuals.get(1) == MaturitySyntheticOriginResidual::ALL.get(1),
        Item::CursorFreshnessResidual,
        &mut items,
    )?;
    verify_item(
        report.residuals.get(2) == MaturitySyntheticOriginResidual::ALL.get(2),
        Item::ArchiveOriginResidual,
        &mut items,
    )?;
    verify_item(
        report.census.edges == report.edges.len(),
        Item::CensusEdges,
        &mut items,
    )?;
    verify_item(
        report.census.registered_mutations == registry.census().mutations(),
        Item::CensusRegisteredMutations,
        &mut items,
    )?;
    verify_item(
        report.completeness == MaturitySafetyCompleteness::PartialRequiredRowsOutstanding,
        Item::Completeness,
        &mut items,
    )?;
    Ok(ValidatedMaturityRootHistoryReport {
        report: report.clone(),
        recomputed_items: items,
    })
}

fn hex(bytes: &[u8]) -> String {
    let mut text = String::new();
    for byte in bytes {
        let _ = write!(text, "{byte:02x}");
    }
    text
}

/// Render canonical lowercase report bytes from a validated wrapper.
#[must_use]
#[expect(
    clippy::too_many_lines,
    reason = "The canonical key order remains visible in one renderer."
)]
pub fn render_maturity_root_history_report(
    validated: &ValidatedMaturityRootHistoryReport,
) -> String {
    let report = validated.report();
    let checkpoint = report.checkpoint();
    let target = report.binding().target().projection();
    let mut text = String::new();
    let _ = writeln!(text, "schema {}", report.schema());
    let _ = writeln!(text, "role {}", report.role().name());
    let _ = writeln!(text, "operation announce-maturity");
    let _ = writeln!(text, "target_contract {:?}", target.version());
    let _ = writeln!(text, "execution_domain {:?}", target.execution_domain());
    let _ = writeln!(text, "deployment {}", report.binding().deployment().name());
    let expectation = match report.provenance() {
        MaturityExecutorProvenanceExpectation::Stated(_) => "stated",
        MaturityExecutorProvenanceExpectation::NotStatedByTheOperator => {
            "not-stated-by-the-operator"
        }
    };
    let _ = writeln!(text, "executor_provenance_expectation {expectation}");
    let _ = writeln!(
        text,
        "starting_cursor {} {}",
        report.starting_cursor().txid().to_target_display(),
        report.starting_cursor().index()
    );
    for (position, edge) in report.edges().iter().enumerate() {
        let _ = writeln!(
            text,
            "edge {position} {} {} {} {}",
            edge.predecessor().txid().to_target_display(),
            edge.predecessor().index(),
            edge.successor().txid().to_target_display(),
            edge.successor().index(),
        );
    }
    let _ = writeln!(
        text,
        "final_cursor {} {}",
        report.final_cursor().txid().to_target_display(),
        report.final_cursor().index()
    );
    let _ = writeln!(
        text,
        "checkpoint_network_identity {}",
        hex(checkpoint.network_identity().as_bytes())
    );
    let _ = writeln!(
        text,
        "checkpoint_genesis_identity {}",
        hex(checkpoint.genesis_identity().as_bytes())
    );
    let _ = writeln!(
        text,
        "checkpoint_block_hash {}",
        hex(checkpoint.block_hash())
    );
    let _ = writeln!(
        text,
        "checkpoint_block_height {}",
        checkpoint.block_height()
    );
    let _ = writeln!(
        text,
        "checkpoint_transaction_identity {}",
        checkpoint.transaction_identity().to_target_display()
    );
    let _ = writeln!(
        text,
        "checkpoint_predecessor_outpoint {} {}",
        checkpoint.predecessor().txid().to_target_display(),
        checkpoint.predecessor().index()
    );
    let _ = writeln!(
        text,
        "checkpoint_successor_outpoint {} {}",
        checkpoint.successor().txid().to_target_display(),
        checkpoint.successor().index()
    );
    let _ = writeln!(
        text,
        "checkpoint_target_contract {}",
        hex(checkpoint.target_contract().as_bytes())
    );
    let _ = writeln!(
        text,
        "checkpoint_linked_candidate declared {}",
        checkpoint.linked_candidate().provenance().name()
    );
    let _ = writeln!(
        text,
        "checkpoint_candidate_abi declared {}",
        checkpoint.candidate_abi().provenance().name()
    );
    let _ = writeln!(
        text,
        "branch {} {}",
        hex(checkpoint.branch().identifier()),
        checkpoint.branch().checkpoint()
    );
    for mutation in report.registry().mutations() {
        let _ = writeln!(text, "registered {}", mutation.row());
    }
    for residual in report.residuals() {
        let _ = writeln!(text, "residual {}", residual.name());
    }
    let _ = writeln!(text, "edges {}", report.census().edges());
    let _ = writeln!(
        text,
        "registered_mutations {}",
        report.census().registered_mutations()
    );
    let _ = writeln!(text, "completeness {}", report.completeness().name());
    for item in validated.recomputed_items() {
        let _ = writeln!(text, "recomputed {}", item.name());
    }
    text.make_ascii_lowercase();
    text
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::LazyLock;

    use crate::maturity_continuity::tests::{archived, variable_archived};
    use crate::maturity_continuity::{
        MaturityByteSource, ValidatedMaturityContinuity, project_maturity_continuity,
        submitted_transaction_identities,
    };
    use crate::maturity_corpus::{
        MaturityDeclaredPremise, MaturityPremiseProvenance, maturity_run_of_record,
        maturity_variable_run_of_record,
    };
    use crate::maturity_evidence::{
        MaturityAnnouncementEvidencePlan, MaturityConstructorMaterial,
        MaturityConstructorMaterialAbsence, checkpoint_premises_of,
        derive_maturity_evidence_plan_with,
    };
    use crate::maturity_recovery::PublicAnnouncementLocator;
    use crate::maturity_recovery_report::accepted_public_handoff;
    use crate::maturity_report::MaturityVolatileField;

    static PLAN: LazyLock<MaturityAnnouncementEvidencePlan> = LazyLock::new(|| {
        derive_maturity_evidence_plan_with(
            MaturityExecutorProvenanceExpectation::NotStatedByTheOperator,
            MaturityConstructorMaterial::Absent(
                MaturityConstructorMaterialAbsence::NotSuppliedToDerivation,
            ),
        )
        .expect("root-history evidence plan")
    });

    fn accepted_continuity() -> ValidatedMaturityContinuity {
        project_maturity_continuity(variable_archived().input()).expect("accepted archive projects")
    }

    fn historical_continuity() -> ValidatedMaturityContinuity {
        project_maturity_continuity(archived().input()).expect("historical archive projects")
    }

    fn checkpoint(
        edge: &StateRootEdge,
        continuity: &ValidatedMaturityContinuity,
        handoff: &PublicAnnouncementHandoff,
    ) -> MaturityRootCheckpoint {
        let (linked, abi) = checkpoint_premises_of(continuity).expect("accepted premises");
        MaturityRootCheckpoint::bind(
            edge,
            handoff,
            maturity_variable_run_of_record().expect("accepted corpus"),
            linked,
            abi,
        )
        .expect("accepted checkpoint binds")
    }

    fn historical_handoff(accepted: &PublicAnnouncementHandoff) -> PublicAnnouncementHandoff {
        let historical = historical_continuity();
        let corpus = maturity_run_of_record().expect("historical corpus");
        PublicAnnouncementHandoff::new(
            PublicAnnouncementLocator::new(
                MaturityByteSource::ArchivedSubmission {
                    run_address: corpus.report().run_address().to_owned(),
                },
                submitted_transaction_identities(
                    historical.transaction(),
                    historical.submitted_bytes(),
                )
                .identity(),
            ),
            archived().input().submitted_bytes.to_vec(),
            0,
            accepted.schedule(),
            accepted.bounds(),
            accepted.static_subtree().clone(),
            accepted.internal_key(),
            accepted.contract(),
        )
    }

    fn assembled() -> (
        MaturityRootHistoryReport,
        PublicAnnouncementHandoff,
        StateRootEdge,
    ) {
        let continuity = accepted_continuity();
        let edge = StateRootEdge::from_continuity(&continuity);
        let handoff =
            accepted_public_handoff(maturity_variable_run_of_record().expect("accepted corpus"))
                .expect("accepted handoff");
        let report = assemble_maturity_root_history_report(
            std::slice::from_ref(&edge),
            edge.predecessor(),
            checkpoint(&edge, &continuity, &handoff),
            PLAN.root_history_mutations().clone(),
            PLAN.binding().clone(),
            PLAN.executor_provenance().clone(),
        )
        .expect("accepted report assembles");
        (report, handoff, edge)
    }

    fn validate(
        report: &MaturityRootHistoryReport,
        edges: &[StateRootEdge],
        handoff: &PublicAnnouncementHandoff,
    ) -> Result<ValidatedMaturityRootHistoryReport, MaturityRootHistoryReportRefusal> {
        validate_maturity_root_history_report(
            report,
            edges,
            report.starting_cursor(),
            handoff,
            PLAN.root_history_mutations(),
            PLAN.binding(),
            PLAN.executor_provenance(),
        )
    }

    #[test]
    fn the_accepted_archives_history_report_validates_and_renders_three_residuals_in_order() {
        let (report, handoff, edge) = assembled();
        let validated = validate(&report, std::slice::from_ref(&edge), &handoff)
            .expect("accepted report validates");
        assert_eq!(report.final_cursor(), edge.successor());
        assert_eq!(report.final_cursor().index(), 0);
        assert_eq!(report.census().edges(), 1);
        assert_eq!(report.census().registered_mutations(), 16);
        assert_eq!(
            validated.recomputed_items(),
            &MaturityRootHistoryRecomputedItem::ALL
                .iter()
                .copied()
                .collect()
        );
        let rendered = render_maturity_root_history_report(&validated);
        assert!(rendered.contains("checkpoint_linked_candidate declared deployment-declaration\n"));
        assert!(rendered.contains("checkpoint_candidate_abi declared deployment-declaration\n"));
        assert!(rendered.contains("recomputed checkpoint-premise-provenance\n"));
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
            ]
        );
    }

    #[test]
    fn independent_assemblies_render_identical_bytes() {
        let (first, first_handoff, first_edge) = assembled();
        let (second, second_handoff, second_edge) = assembled();
        let first = validate(&first, std::slice::from_ref(&first_edge), &first_handoff)
            .expect("first report validates");
        let second = validate(&second, std::slice::from_ref(&second_edge), &second_handoff)
            .expect("second report validates");
        assert_eq!(
            render_maturity_root_history_report(&first),
            render_maturity_root_history_report(&second),
        );
    }

    #[test]
    fn a_final_cursor_equal_to_the_honest_one_does_not_hide_an_invalid_earlier_edge() {
        let (mut report, handoff, accepted_edge) = assembled();
        let historical_edge = StateRootEdge::from_continuity(&historical_continuity());
        let honest_final_cursor = accepted_edge.successor();
        report.edges = vec![historical_edge, accepted_edge];
        report.final_cursor = honest_final_cursor;
        assert_eq!(report.final_cursor(), honest_final_cursor);
        assert_eq!(
            report.edges.last().map(StateRootEdge::successor),
            Some(honest_final_cursor)
        );
        assert!(matches!(
            validate(&report, &report.edges, &handoff),
            Err(MaturityRootHistoryReportRefusal::EdgeSequenceRefused {
                refusal,
            }) if matches!(
                *refusal,
                MaturityRootHistoryRefusal::PredecessorIsNotTheCursor { position: 0, .. }
            )
        ));
    }

    #[test]
    fn a_stated_item_differing_from_its_recomputation_fails_at_that_item() {
        let (report, handoff, edge) = assembled();
        let mut changed = report.clone();
        changed.starting_cursor =
            StateRootEdge::from_continuity(&historical_continuity()).predecessor();
        assert!(matches!(
            validate_maturity_root_history_report(
                &changed,
                std::slice::from_ref(&edge),
                report.starting_cursor(),
                &handoff,
                PLAN.root_history_mutations(),
                PLAN.binding(),
                PLAN.executor_provenance(),
            ),
            Err(MaturityRootHistoryReportRefusal::ItemDiffers {
                item: MaturityRootHistoryRecomputedItem::StartingCursor
            })
        ));
        let mut changed = report.clone();
        changed.final_cursor = edge.predecessor();
        assert!(matches!(
            validate(&changed, std::slice::from_ref(&edge), &handoff),
            Err(MaturityRootHistoryReportRefusal::ItemDiffers {
                item: MaturityRootHistoryRecomputedItem::FinalCursor
            })
        ));
        let mut changed = report.clone();
        changed.census.edges = 2;
        assert!(matches!(
            validate(&changed, std::slice::from_ref(&edge), &handoff),
            Err(MaturityRootHistoryReportRefusal::ItemDiffers {
                item: MaturityRootHistoryRecomputedItem::CensusEdges
            })
        ));
        let mut changed = report;
        changed.completeness = MaturitySafetyCompleteness::Failed;
        assert!(matches!(
            validate(&changed, std::slice::from_ref(&edge), &handoff),
            Err(MaturityRootHistoryReportRefusal::ItemDiffers {
                item: MaturityRootHistoryRecomputedItem::Completeness
            })
        ));
    }

    #[test]
    fn removed_and_duplicated_edges_refuse_before_payload_comparison() {
        let (report, handoff, edge) = assembled();
        let removed = validate(&report, &[], &handoff).expect_err("removed edge refuses");
        assert_eq!(
            removed,
            MaturityRootHistoryReportRefusal::EdgeCountDiffers {
                stated: 1,
                recomputed: 0
            }
        );
        let duplicated = validate(&report, &[edge.clone(), edge.clone()], &handoff)
            .expect_err("duplicated edge refuses");
        assert_eq!(
            duplicated,
            MaturityRootHistoryReportRefusal::EdgeCountDiffers {
                stated: 1,
                recomputed: 2
            }
        );
        let historical_edge = StateRootEdge::from_continuity(&historical_continuity());
        let mut stated = report;
        stated.edges = vec![historical_edge.clone(), edge.clone()];
        let reordered = validate(&stated, &[edge, historical_edge], &handoff)
            .expect_err("reordered edges refuse");
        assert_eq!(
            reordered,
            MaturityRootHistoryReportRefusal::EdgeOrderDiffers { position: 0 }
        );
    }

    #[test]
    fn a_checkpoint_another_handoff_does_not_bind_refuses_at_the_checkpoint_item() {
        let (report, handoff, edge) = assembled();
        let historical = historical_handoff(&handoff);
        assert!(matches!(
            validate(&report, std::slice::from_ref(&edge), &historical),
            Err(MaturityRootHistoryReportRefusal::CheckpointRefused { refusal })
                if matches!(*refusal, MaturityRootCheckpointRefusal::TransactionIdentityDisagrees { .. })
        ));
    }

    #[test]
    fn a_premise_declared_from_executor_arguments_refuses_at_the_provenance_item() {
        let (report, handoff, edge) = assembled();
        let continuity = accepted_continuity();
        let corpus = maturity_variable_run_of_record().expect("accepted corpus");
        let (linked, abi) = checkpoint_premises_of(&continuity).expect("accepted premises");
        let executor_linked = MaturityDeclaredPremise::declared(
            linked.value().clone(),
            MaturityPremiseProvenance::ExecutorArguments,
        );
        let mut changed = report.clone();
        changed.checkpoint =
            MaturityRootCheckpoint::bind(&edge, &handoff, corpus, executor_linked, abi.clone())
                .expect("binding checks values, not provenance");
        assert!(matches!(
            validate(&changed, std::slice::from_ref(&edge), &handoff),
            Err(MaturityRootHistoryReportRefusal::ItemDiffers {
                item: MaturityRootHistoryRecomputedItem::CheckpointPremiseProvenance
            })
        ));

        let executor_abi = MaturityDeclaredPremise::declared(
            abi.value().clone(),
            MaturityPremiseProvenance::ExecutorArguments,
        );
        let mut changed = report;
        changed.checkpoint =
            MaturityRootCheckpoint::bind(&edge, &handoff, corpus, linked, executor_abi)
                .expect("binding checks values, not provenance");
        assert!(matches!(
            validate(&changed, std::slice::from_ref(&edge), &handoff),
            Err(MaturityRootHistoryReportRefusal::ItemDiffers {
                item: MaturityRootHistoryRecomputedItem::CheckpointPremiseProvenance
            })
        ));
    }

    #[test]
    fn an_unread_schema_refuses_first() {
        let (mut report, handoff, edge) = assembled();
        report.schema = MATURITY_ROOT_HISTORY_REPORT_SCHEMA + 1;
        report.edges.clear();
        assert_eq!(
            validate(&report, std::slice::from_ref(&edge), &handoff),
            Err(MaturityRootHistoryReportRefusal::UnsupportedSchema { stated: 2 }),
        );
    }

    #[test]
    fn the_canonical_bytes_carry_no_volatile_key_and_no_raw_text() {
        let (report, handoff, edge) = assembled();
        let validated = validate(&report, std::slice::from_ref(&edge), &handoff)
            .expect("accepted report validates");
        let rendered = render_maturity_root_history_report(&validated);
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
        for raw in [
            report.checkpoint().network_identity(),
            report.checkpoint().genesis_identity(),
            report.checkpoint().target_contract(),
        ] {
            assert!(!rendered.contains(raw));
        }
    }

    // §1.13 (`rule:guide14-exec:sponsor-opacity`): this checks keys, not values,
    // in sponsorless bytes; neither report-publishes-* row is answered.
    #[test]
    fn the_canonical_bytes_publish_no_sponsor_key() {
        let (report, handoff, edge) = assembled();
        let validated = validate(&report, std::slice::from_ref(&edge), &handoff)
            .expect("accepted report validates");
        let rendered = render_maturity_root_history_report(&validated);
        assert_eq!(
            crate::live_minimality_report::forbidden_key_in(&rendered),
            None,
        );

        for key in crate::live_minimality_report::FORBIDDEN_KEYS {
            let staged = format!("{rendered}{key} 1000\n");
            assert_eq!(
                crate::live_minimality_report::forbidden_key_in(&staged),
                Some(*key),
                "{key} would not have been caught",
            );
        }

        let value_word = format!("{rendered}note sponsor_amount\n");
        assert_eq!(
            crate::live_minimality_report::forbidden_key_in(&value_word),
            None,
        );
    }

    #[test]
    fn the_recomputation_inventory_is_complete_and_distinctly_named() {
        let names: BTreeSet<_> = MaturityRootHistoryRecomputedItem::ALL
            .iter()
            .map(|item| item.name())
            .collect();
        assert_eq!(MaturityRootHistoryRecomputedItem::ALL.len(), 20);
        assert_eq!(names.len(), 20);
        let (report, handoff, edge) = assembled();
        let validated = validate(&report, std::slice::from_ref(&edge), &handoff)
            .expect("accepted report validates");
        let expected: BTreeSet<_> = MaturityRootHistoryRecomputedItem::ALL
            .iter()
            .copied()
            .collect();
        assert_eq!(validated.recomputed_items(), &expected);
    }
}
