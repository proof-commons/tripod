//! The validated live-transfer minimality report (§13.3, §13.5, §13.6).
//!
//! §13.3 asks one question — does the private-committed plan perform the
//! same semantic transfer with less exact amount disclosure than the
//! explicit plan — and names nine things each row carries. §13.5 says the
//! gate reads a validated wrapper rather than a raw report. §13.6 forbids
//! this report and the safety report standing in for one another. This
//! module is all three, and its answer to §13.3's question is that
//! nobody knows.
//!
//! # The honest answer is a value, not an omission
//!
//! [`MinimalityStanding::Unanswered`] is what
//! [`validate_live_minimality_report`] recomputes today, and a report
//! claiming anything stronger is refused. Two of §16.2's ten conditions
//! name components that do not exist — no first-party component computes
//! the digest an owner must sign, and no funding step can create a
//! confidential predecessor — so no pair supports minimality and none
//! ever will until those exist. That is not a negative finding about the
//! private plan: §16.4's last sentence is that a failed minimality result
//! says nothing about explicit safety, and an *unanswered* one says even
//! less.
//!
//! # What is checkable today is checked today
//!
//! §16.2's last conditions are about this workspace's own bytes rather
//! than about a target, and they are answered here: the canonical
//! rendering publishes no sponsor value and no private opening, checked
//! by key against [`FORBIDDEN_KEYS`] with a test that has teeth. Seven of
//! the ten conditions hold first-party for every pair
//! ([`crate::live_pairs`]), and the report carries the count rather than
//! a summary of it.
//!
//! # This is not the safety report
//!
//! §13.6, and it is enforced in the bytes: the schema identity rendered
//! here is a *qualified* one, `live-transfer-minimality/1`, so a reader
//! holding only the first line of a document can tell which kind it has.
//! The safety report's schema line reads `1`, and no reader that checks
//! the schema can confuse the two. The role line says it again, and
//! [`validate_live_minimality_report`] refuses a document of the other
//! role rather than reading it as this one.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use compiler::live_transfer_plan::LiveTransferRepresentationPlan;
use target_elements::{DeploymentProjection, TargetProjection};

use crate::live_disclosure::{
    AdditionalDisclosureReason, DisclosedItem, DisclosureRow, DisclosureStanding,
    additional_exact_amount_disclosures, disclosure_difference, disclosure_table, shape_leakage,
};
use crate::live_evidence::LiveInfrastructureBlocker;
use crate::live_pairs::{
    BOTH_PLANS, MinimalityConditionStanding, MinimalityPair, MinimalityPairRefusal,
    MinimalityPairRow, PairAcceptanceCondition, build_minimality_pairs, condition_scoreboard,
};
use crate::live_report::{LiveLifecycleStatus, RecomputedItem, VolatileField};

/// The version of the canonical rendered minimality report.
pub const LIVE_MINIMALITY_REPORT_SCHEMA: u32 = 1;

/// The qualified schema identity the canonical bytes carry.
///
/// §13.6 forbids the safety and minimality reports substituting for one
/// another, and a version number alone cannot carry that: two documents
/// whose first line both read `schema 1` are told apart only by a reader
/// who goes on to the second. Qualifying the identity with the role makes
/// the first line decisive, and it does so without pretending this brand
/// new document is at version two of anything.
pub const LIVE_MINIMALITY_SCHEMA_ID: &str = "live-transfer-minimality/1";

/// What a minimality report is, said in the bytes.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum LiveMinimalityReportRole {
    /// The live-transfer minimality report of §13.3.
    LiveTransferMinimality,
}

impl LiveMinimalityReportRole {
    /// The role's wire spelling.
    ///
    /// Deliberately not the safety report's `live-transfer-safety`, and
    /// a test compares the two rather than trusting that they look
    /// different.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::LiveTransferMinimality => "live-transfer-minimality",
        }
    }
}

/// One of §16.4's nine minimality failure modes.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MinimalityFailureMode {
    /// The private materialization rejects.
    PrivateMaterializationRejects,
    /// Exact receipt values enter protocol predicates.
    ExactReceiptValuesEnterProtocolPredicates,
    /// A public subtotal is required.
    APublicSubtotalIsRequired,
    /// Fixture openings appear in the canonical report.
    FixtureOpeningsAppearInTheCanonicalReport,
    /// Undeclared owner-private data is required.
    UndeclaredOwnerPrivateDataIsRequired,
    /// The semantic output changes.
    SemanticOutputChanges,
    /// A required exit is lost.
    ARequiredExitIsLost,
    /// A hard target limit blocks the private plan while explicit fits.
    AHardTargetLimitBlocksThePrivatePlan,
    /// Confidential asset identity appears.
    ConfidentialAssetIdentityAppears,
}

impl MinimalityFailureMode {
    /// All nine, in §16.4's order.
    pub const ALL: &'static [Self] = &[
        Self::PrivateMaterializationRejects,
        Self::ExactReceiptValuesEnterProtocolPredicates,
        Self::APublicSubtotalIsRequired,
        Self::FixtureOpeningsAppearInTheCanonicalReport,
        Self::UndeclaredOwnerPrivateDataIsRequired,
        Self::SemanticOutputChanges,
        Self::ARequiredExitIsLost,
        Self::AHardTargetLimitBlocksThePrivatePlan,
        Self::ConfidentialAssetIdentityAppears,
    ];

    /// The mode's wire spelling.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::PrivateMaterializationRejects => "private-materialization-rejects",
            Self::ExactReceiptValuesEnterProtocolPredicates => {
                "exact-receipt-values-enter-protocol-predicates"
            }
            Self::APublicSubtotalIsRequired => "a-public-subtotal-is-required",
            Self::FixtureOpeningsAppearInTheCanonicalReport => {
                "fixture-openings-appear-in-the-canonical-report"
            }
            Self::UndeclaredOwnerPrivateDataIsRequired => {
                "undeclared-owner-private-data-is-required"
            }
            Self::SemanticOutputChanges => "semantic-output-changes",
            Self::ARequiredExitIsLost => "a-required-exit-is-lost",
            Self::AHardTargetLimitBlocksThePrivatePlan => {
                "a-hard-target-limit-blocks-the-private-plan"
            }
            Self::ConfidentialAssetIdentityAppears => "confidential-asset-identity-appears",
        }
    }
}

/// Where one §16.4 failure mode stands.
///
/// Three states, and the first is deliberately not "passed". A mode that
/// has not been observed has not been observed; whether it *could* occur
/// on a target is a different question, and the other two states are the
/// modes for which it is the only question there is.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum FailureModeStanding {
    /// Checked against the built pair registry, and it does not occur.
    NotObservedFirstParty,
    /// Every run of a pair member's shape that a target was asked about
    /// was accepted, and the shapes with no run are named in the
    /// registry rather than counted as rejections here.
    ///
    /// Distinct from [`Self::NotObservedFirstParty`], which is a check
    /// this crate ran, and from [`Self::AwaitsATargetRun`], which says
    /// no target was asked. A target WAS asked, repeatedly, and said
    /// yes every time — while some shapes remain unasked. Collapsing
    /// that into either neighbour would lose the half a reader needs.
    NotRejectedOnAnyObservedRun,
    /// The mode is about what a target does, and none was asked.
    AwaitsATargetRun(LiveInfrastructureBlocker),
    /// §18's study measured it, and the private plan stays inside the
    /// limits the reviewed contract declares.
    ///
    /// Candidate-scoped, and the name says which half of §1.4's pair it
    /// belongs to. The study completes both members of every pair it can
    /// complete and weighs them, and completion runs the reviewed
    /// contract's own weight bound — so a member that exists is a member
    /// inside that bound. What this does *not* say is that a target
    /// accepted anything: §1.4 keeps target acceptance and first-party
    /// agreement separate, and no candidate transfer has been accepted by
    /// anything (§1.7).
    MeasuredWithinTheDeclaredLimits,
}

impl FailureModeStanding {
    /// The standing's wire spelling.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::NotObservedFirstParty => "not-observed-first-party",
            Self::NotRejectedOnAnyObservedRun => "not-rejected-on-any-observed-run",
            Self::AwaitsATargetRun(_) => "awaits-a-target-run",
            Self::MeasuredWithinTheDeclaredLimits => "measured-within-the-declared-limits",
        }
    }
}

/// One thing a successful private transfer would still not establish
/// (§16.5).
///
/// Carried by the report rather than written in a doc comment, for the
/// reason [`transaction::live_private::PrivateConstructionNonClaim`]
/// gives: a sentence in prose is not something a summary can be checked
/// against, and a report that carried its non-claims cannot be
/// paraphrased into a stronger statement than it makes.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PrivacyNonClaim {
    /// Owner anonymity.
    OwnerAnonymity,
    /// Transaction-graph privacy.
    TransactionGraphPrivacy,
    /// Count privacy.
    CountPrivacy,
    /// Timing privacy.
    TimingPrivacy,
    /// Production wallet privacy.
    ProductionWalletPrivacy,
    /// Production side-channel resistance.
    ProductionSideChannelResistance,
    /// Production multi-party blinding.
    ProductionMultiPartyBlinding,
    /// Universal transaction confidentiality.
    UniversalTransactionConfidentiality,
}

impl PrivacyNonClaim {
    /// All eight, in §16.5's order.
    pub const ALL: &'static [Self] = &[
        Self::OwnerAnonymity,
        Self::TransactionGraphPrivacy,
        Self::CountPrivacy,
        Self::TimingPrivacy,
        Self::ProductionWalletPrivacy,
        Self::ProductionSideChannelResistance,
        Self::ProductionMultiPartyBlinding,
        Self::UniversalTransactionConfidentiality,
    ];

    /// The non-claim's wire spelling.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::OwnerAnonymity => "owner-anonymity",
            Self::TransactionGraphPrivacy => "transaction-graph-privacy",
            Self::CountPrivacy => "count-privacy",
            Self::TimingPrivacy => "timing-privacy",
            Self::ProductionWalletPrivacy => "production-wallet-privacy",
            Self::ProductionSideChannelResistance => "production-side-channel-resistance",
            Self::ProductionMultiPartyBlinding => "production-multi-party-blinding",
            Self::UniversalTransactionConfidentiality => "universal-transaction-confidentiality",
        }
    }
}

/// What §17.4 lets the minimality report conclude about the lifecycle.
///
/// §17.4 states the conclusion in as many words: equal for transfer
/// semantics, not equal for the complete live-receipt lifecycle, until
/// the required exits exist. The type has one arm because that is the one
/// conclusion available, and it exists so a later wave clearing §17.1's
/// two outstanding exits has to add the other rather than change a
/// string.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum LifecycleConclusion {
    /// Equal for transfer semantics, unequal for the whole lifecycle.
    EqualForTransferUnequalForTheLifecycle,
}

impl LifecycleConclusion {
    /// The conclusion's wire spelling.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::EqualForTransferUnequalForTheLifecycle => {
                "equal-for-transfer-unequal-for-the-lifecycle"
            }
        }
    }
}

/// Whether §13.3's question has an answer.
///
/// Three states and no inference. The report never implies an answer by
/// the absence of a failure, so the token is explicit and the first state
/// is what the report carries today.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MinimalityStanding {
    /// No pair satisfies §16.2 and the reason is a missing component.
    Unanswered,
    /// Some claimed pairs satisfy every §16.2 condition and some do not.
    ///
    /// # Why a third state had to exist
    ///
    /// The token was binary — every claimed pair or none — and that was
    /// adequate for exactly as long as the answer was none. It stopped
    /// being adequate the moment one pair's two shapes were both run
    /// and accepted, because both remaining tokens would then have been
    /// false: `Unanswered` says NO pair satisfies §16.2, and
    /// `SupportedForEveryClaimedPair` says every one does.
    ///
    /// A report forced to choose between two false tokens picks the
    /// conservative one and reads as a smaller claim, which sounds
    /// harmless and is not: `Unanswered`'s own words would have denied
    /// the observations, and a reader checking the sentence against the
    /// registry would find it contradicted by three pairs.
    ///
    /// # What it claims, and for which pairs
    ///
    /// That §16.2 is satisfied for the pairs the registry marks
    /// supporting and for NO others. §16.2 is a conjunction of ten per
    /// pair, so this token distributes over pairs and never over the
    /// matrix: it is not "minimality is partly established", which
    /// would be a claim about a whole nobody stated. The pairs are named
    /// in the registry and each unsupported one names its failing
    /// conjunct.
    SupportedForSomeClaimedPairsOnly,
    /// Every claimed pair satisfies every §16.2 condition.
    SupportedForEveryClaimedPair,
    /// Some pair exhibits a §16.4 failure mode.
    Failed,
}

impl MinimalityStanding {
    /// The token's wire spelling.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Unanswered => "unanswered",
            Self::SupportedForSomeClaimedPairsOnly => "supported-for-some-claimed-pairs-only",
            Self::SupportedForEveryClaimedPair => "supported-for-every-claimed-pair",
            Self::Failed => "failed",
        }
    }
}

/// The noncanonical diagnostic report of §13.6, for this document.
///
/// Its own value rather than the safety report's, because §13.6 keeps the
/// two documents apart and a shared diagnostic bag would be one place two
/// runs of two different reports wrote to. The canonical renderer takes
/// no argument of this type, which is what makes §13.5's exclusion
/// structural instead of a filter somebody could forget to apply.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LiveMinimalityDiagnostics {
    fields: BTreeMap<VolatileField, String>,
}

impl LiveMinimalityDiagnostics {
    /// A diagnostic report with one field recorded.
    #[must_use]
    pub fn with(mut self, field: VolatileField, value: impl Into<String>) -> Self {
        self.fields.insert(field, value.into());
        self
    }

    /// Every recorded field.
    #[must_use]
    pub const fn fields(&self) -> &BTreeMap<VolatileField, String> {
        &self.fields
    }

    /// The diagnostic report's own rendering.
    ///
    /// # Panics
    ///
    /// Never: the sink is a `String`, whose writes cannot fail.
    #[must_use]
    pub fn render(&self) -> String {
        let mut text = String::new();
        let _ = writeln!(text, "role live-transfer-minimality-diagnostics");
        let _ = writeln!(text, "canonical false");
        for (field, value) in &self.fields {
            let _ = writeln!(text, "{field:?} {value}");
        }
        text
    }
}

/// The census figures a reader of a minimality report wants first.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MinimalityPairCensus {
    pairs: usize,
    claimed: usize,
    unclaimed: usize,
    supporting: usize,
}

impl MinimalityPairCensus {
    /// How many §16.1 pairs the registry has.
    #[must_use]
    pub const fn pairs(&self) -> usize {
        self.pairs
    }

    /// How many the published shape set claims.
    #[must_use]
    pub const fn claimed(&self) -> usize {
        self.claimed
    }

    /// How many it does not.
    #[must_use]
    pub const fn unclaimed(&self) -> usize {
        self.unclaimed
    }

    /// How many satisfy every §16.2 condition.
    #[must_use]
    pub const fn supporting(&self) -> usize {
        self.supporting
    }

    /// The standing this census supports.
    ///
    /// Derived rather than declared, which is what makes the report's
    /// token something [`validate_live_minimality_report`] can disagree
    /// with.
    #[must_use]
    pub const fn standing(&self) -> MinimalityStanding {
        if self.claimed != 0 && self.supporting == self.claimed {
            MinimalityStanding::SupportedForEveryClaimedPair
        } else if self.supporting != 0 {
            // The partial state, and it is reached by counting rather
            // than by an author's judgement — which is the property that
            // lets `validate_live_minimality_report` disagree with a
            // token somebody wrote by hand.
            MinimalityStanding::SupportedForSomeClaimedPairsOnly
        } else {
            MinimalityStanding::Unanswered
        }
    }
}

/// The live-transfer minimality report of §13.3.
///
/// A value a caller can build, and one that establishes nothing on its
/// own. The conclusions are [`validate_live_minimality_report`]'s.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiveTransferMinimalityReport {
    schema: u32,
    role: LiveMinimalityReportRole,
    target: TargetProjection,
    deployment: Option<DeploymentProjection>,
    representations: BTreeSet<LiveTransferRepresentationPlan>,
    census: MinimalityPairCensus,
    rows: Vec<MinimalityPairRow>,
    failures: BTreeMap<MinimalityFailureMode, FailureModeStanding>,
    non_claims: BTreeSet<PrivacyNonClaim>,
    lifecycle: LiveLifecycleStatus,
    lifecycle_conclusion: LifecycleConclusion,
    standing: MinimalityStanding,
}

impl LiveTransferMinimalityReport {
    /// The report's schema version.
    #[must_use]
    pub const fn schema(&self) -> u32 {
        self.schema
    }

    /// What kind of report this is.
    #[must_use]
    pub const fn role(&self) -> LiveMinimalityReportRole {
        self.role
    }

    /// The reviewed target the pairs are stated against.
    #[must_use]
    pub const fn target(&self) -> &TargetProjection {
        &self.target
    }

    /// The deployment a run was bound to, where one happened.
    #[must_use]
    pub const fn deployment(&self) -> Option<&DeploymentProjection> {
        self.deployment.as_ref()
    }

    /// The representation plans compared.
    #[must_use]
    pub const fn representations(&self) -> &BTreeSet<LiveTransferRepresentationPlan> {
        &self.representations
    }

    /// The pair census.
    #[must_use]
    pub const fn census(&self) -> MinimalityPairCensus {
        self.census
    }

    /// Every §13.3 row.
    #[must_use]
    pub fn rows(&self) -> &[MinimalityPairRow] {
        &self.rows
    }

    /// Where each §16.4 failure mode stands.
    #[must_use]
    pub const fn failures(&self) -> &BTreeMap<MinimalityFailureMode, FailureModeStanding> {
        &self.failures
    }

    /// The §16.5 non-claims.
    #[must_use]
    pub const fn non_claims(&self) -> &BTreeSet<PrivacyNonClaim> {
        &self.non_claims
    }

    /// The lifecycle obligations (§17.1).
    #[must_use]
    pub const fn lifecycle(&self) -> &LiveLifecycleStatus {
        &self.lifecycle
    }

    /// What §17.4 lets the report conclude about them.
    #[must_use]
    pub const fn lifecycle_conclusion(&self) -> LifecycleConclusion {
        self.lifecycle_conclusion
    }

    /// Whether §13.3's question has an answer.
    #[must_use]
    pub const fn standing(&self) -> MinimalityStanding {
        self.standing
    }
}

/// Why one minimality report could not be validated.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum LiveMinimalityReportRefusal {
    /// The pair registry could not be built.
    RegistryUnavailable(MinimalityPairRefusal),
    /// The report states a schema version this validator does not read.
    UnsupportedSchema(u32),
    /// The report states a role that is not the minimality one.
    ///
    /// §13.6's separation, enforced: a safety report handed to the
    /// minimality gate is refused rather than read as a minimality one.
    WrongRole(LiveMinimalityReportRole),
    /// The report's target is not the one the pairs are stated against.
    TargetDiffers,
    /// The report's representation census is not both admitted plans.
    RepresentationCensusDiffers,
    /// The report's pair census is not the one recomputed.
    CensusDiffers {
        /// What the report said.
        reported: MinimalityPairCensus,
        /// What the registry recomputes to.
        recomputed: MinimalityPairCensus,
    },
    /// The report claims a standing its own census does not support.
    StandingDiffers {
        /// What the report said.
        reported: MinimalityStanding,
        /// What the census supports.
        recomputed: MinimalityStanding,
    },
    /// The report's rows are not the registry's.
    RowsDiffer,
    /// The report's §16.4 taxonomy is not the recomputed one.
    FailureTaxonomyDiffers,
    /// The report drops one of §16.5's non-claims.
    NonClaimCensusDiffers,
    /// The report names a deployment while recording that no run
    /// happened.
    DeploymentWithoutARun,
    /// The report's lifecycle status is not the candidate's.
    LifecycleDiffers,
    /// The canonical rendering publishes a sponsor value or a private
    /// opening.
    ///
    /// §16.2's last report-side conditions, and the two of them that are
    /// checkable today. A report that failed this is refused rather than
    /// published with a note.
    CanonicalBytesPublishAForbiddenKey(String),
}

/// One minimality report, validated: the only thing a minimality gate
/// reads.
///
/// # What holding one of these establishes
///
/// That the report's carriers agree with what this module recomputed from
/// the pair registry: the same target, the same representation census,
/// the same rows, the same §16.2 scoreboard, the same §16.4 taxonomy, the
/// same §16.5 non-claims, the same lifecycle status, and the same
/// standing token. It also establishes that the canonical bytes carry no
/// forbidden key, because the validator renders them and checks.
///
/// It establishes nothing about whether the private plan discloses less
/// on a chain, because no chain was asked.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedLiveTransferMinimalityReport {
    report: LiveTransferMinimalityReport,
    recomputed_items: BTreeSet<RecomputedItem>,
    conditions: BTreeMap<PairAcceptanceCondition, (usize, BTreeSet<MinimalityConditionStanding>)>,
    blockers: BTreeMap<LiveInfrastructureBlocker, usize>,
}

impl ValidatedLiveTransferMinimalityReport {
    /// The report every carrier of which has been recomputed.
    #[must_use]
    pub const fn report(&self) -> &LiveTransferMinimalityReport {
        &self.report
    }

    /// The §13.5 items this validation recomputed.
    #[must_use]
    pub const fn recomputed_items(&self) -> &BTreeSet<RecomputedItem> {
        &self.recomputed_items
    }

    /// The §16.2 scoreboard: how many pairs satisfy each condition, and
    /// what the deficit is typed as.
    #[must_use]
    pub const fn conditions(
        &self,
    ) -> &BTreeMap<PairAcceptanceCondition, (usize, BTreeSet<MinimalityConditionStanding>)> {
        &self.conditions
    }

    /// Every blocker the rows name, with how many rows name it.
    #[must_use]
    pub const fn blockers(&self) -> &BTreeMap<LiveInfrastructureBlocker, usize> {
        &self.blockers
    }
}

/// The §16.4 taxonomy, resolved against a built pair registry.
///
/// Seven modes are checked here against the two members of every pair;
/// one is about what a target decides and one about resource use, and
/// neither is answerable through this boundary. None of the seven is
/// reported as "passed": what a check establishes is that the mode was
/// not observed in what was built.
#[must_use]
pub fn resolve_failure_modes(
    rows: &[MinimalityPairRow],
) -> BTreeMap<MinimalityFailureMode, FailureModeStanding> {
    use FailureModeStanding as Standing;
    use MinimalityFailureMode as Mode;

    // Every one of the seven first-party modes is checked against every
    // pair, and a mode observed anywhere is a mode observed. The registry
    // refuses to build a pair whose halves disagree about the semantics
    // or whose private half publishes an amount, so these read `false`
    // today by construction rather than by luck — which is why the
    // conditions are recomputed here rather than trusted.
    let amounts_leak = rows.iter().any(|row| {
        row.private().publishes_exact_receipt_amounts()
            || row.private().report().consumed_total().is_some()
    });
    let model_undeclared = rows
        .iter()
        .any(|row| row.private().construction_model().is_none());
    let semantics_change = rows.iter().any(|row| {
        row.conditions()
            .get(&PairAcceptanceCondition::OwnerAndLiveClassAgree)
            .is_none_or(|standing| !standing.is_satisfied())
    });
    let exit_lost = rows
        .iter()
        .any(|row| row.lifecycle() != &LiveLifecycleStatus::candidate());
    let asset_confidential = rows.iter().any(|row| {
        row.conditions()
            .get(&PairAcceptanceCondition::ExplicitAssetAgrees)
            .is_none_or(|standing| !standing.is_satisfied())
    });

    // §16.4's eighth mode, measured rather than deferred. Its premise is
    // that the explicit half fits where the private half does not, so a
    // row with neither half completed does not exhibit it — the premise
    // fails on the explicit side too, and a pair nobody could weigh is
    // not a pair the private plan lost.
    let private_blocked_where_explicit_fits = rows.iter().any(|row| {
        row.explicit().complete_weight().is_some() && row.private().complete_weight().is_none()
    });

    // A mode this module observed would be a finding, and the taxonomy
    // has no arm for "observed" because §16.4 is a list of ways
    // minimality *fails* — an observation here is a refusal from
    // `validate_live_minimality_report`, not a standing.
    let observed = |seen: bool| {
        if seen {
            Standing::AwaitsATargetRun(LiveInfrastructureBlocker::NoAcceptingControlExists)
        } else {
            Standing::NotObservedFirstParty
        }
    };

    BTreeMap::from([
        // Whether a target rejects the private materialization is a
        // target's answer.
        //
        // Re-pointed. This named the digest blocker, which is cleared:
        // the digest is computed and two ceremonies carry observed
        // acceptances. What actually keeps the private member off a
        // target is that this pipeline does not build one, which is the
        // blocker the pair conditions carry.
        //
        // Re-pointed a second time, and this time onto an observation
        // rather than onto another blocker. The blocker named here was
        // `NoConfidentialPredecessorCanBeFunded`, which the guide's own
        // closeout carries in its CLEARED set — so the mode was
        // awaiting a run behind a component that already exists.
        //
        // What is known is stronger than "awaiting" and weaker than
        // "never rejects": every run of a pair member's shape that a
        // target has been asked about was ACCEPTED, and ONE pair's own
        // MEMBERS have since been submitted and accepted too. It stays
        // at this standing rather than moving to a stronger one because
        // the mode is about what a target does to a private
        // materialization in general, and four pairs' private
        // materializations have still never been offered to one — what
        // was offered for them is a run of the shape. The registry
        // spells which is which per pair, so the standing does not have
        // to carry it here.
        (
            Mode::PrivateMaterializationRejects,
            Standing::NotRejectedOnAnyObservedRun,
        ),
        (
            Mode::ExactReceiptValuesEnterProtocolPredicates,
            observed(amounts_leak),
        ),
        (Mode::APublicSubtotalIsRequired, observed(amounts_leak)),
        (
            Mode::FixtureOpeningsAppearInTheCanonicalReport,
            // Checked against the rendered bytes by
            // `canonical_bytes_publish_no_forbidden_key`, which the
            // validator runs before it returns.
            Standing::NotObservedFirstParty,
        ),
        (
            Mode::UndeclaredOwnerPrivateDataIsRequired,
            observed(model_undeclared),
        ),
        (Mode::SemanticOutputChanges, observed(semantics_change)),
        (Mode::ARequiredExitIsLost, observed(exit_lost)),
        // §18's study fills this one. The mode is a *conjunction* — a
        // hard target limit blocks the private plan **while explicit
        // fits** — so it is resolved by looking for a row whose explicit
        // member reached a complete transaction and whose private member
        // did not. Every row that can be completed at all completes both
        // halves, and completion runs the reviewed contract's own weight
        // bound, so the conjunction is measured false rather than
        // assumed false.
        (
            Mode::AHardTargetLimitBlocksThePrivatePlan,
            if private_blocked_where_explicit_fits {
                Standing::AwaitsATargetRun(LiveInfrastructureBlocker::NoAcceptingControlExists)
            } else {
                Standing::MeasuredWithinTheDeclaredLimits
            },
        ),
        (
            Mode::ConfidentialAssetIdentityAppears,
            observed(asset_confidential),
        ),
    ])
}

/// The pair census one registry determines.
#[must_use]
pub fn pair_census(rows: &[MinimalityPairRow]) -> MinimalityPairCensus {
    let mut census = MinimalityPairCensus {
        pairs: rows.len(),
        ..MinimalityPairCensus::default()
    };
    for row in rows {
        if row.claim().is_claimed() {
            census.claimed += 1;
        } else {
            census.unclaimed += 1;
        }
        if row.supports_minimality() {
            census.supporting += 1;
        }
    }
    census
}

/// Assemble the minimality report one pair registry determines (§13.3).
///
/// Every carrier is derived from the registry, which is why this builds
/// the registry rather than taking a report: a constructor accepting a
/// census would let a caller state a conclusion the rows do not support.
///
/// # Errors
///
/// [`LiveMinimalityReportRefusal::RegistryUnavailable`] when the pair
/// registry does not build.
pub fn assemble_live_minimality_report(
    target: TargetProjection,
) -> Result<LiveTransferMinimalityReport, LiveMinimalityReportRefusal> {
    let rows =
        build_minimality_pairs().map_err(LiveMinimalityReportRefusal::RegistryUnavailable)?;
    let census = pair_census(&rows);
    Ok(LiveTransferMinimalityReport {
        schema: LIVE_MINIMALITY_REPORT_SCHEMA,
        role: LiveMinimalityReportRole::LiveTransferMinimality,
        target,
        // No run, so no deployment binding (§19.5).
        deployment: None,
        representations: BOTH_PLANS.iter().copied().collect(),
        failures: resolve_failure_modes(&rows),
        non_claims: PrivacyNonClaim::ALL.iter().copied().collect(),
        lifecycle: LiveLifecycleStatus::candidate(),
        // §17.4's own conclusion, and the only one available while §17.1
        // leaves two exits outstanding.
        lifecycle_conclusion: LifecycleConclusion::EqualForTransferUnequalForTheLifecycle,
        standing: census.standing(),
        census,
        rows,
    })
}

/// Validate one minimality report against the registry it claims to be
/// about.
///
/// §13.5's items, recomputed here rather than read off the report,
/// including the disclosure comparison and the report-side half of
/// §16.2 — the canonical bytes are rendered and checked for a forbidden
/// key before this returns.
///
/// # Errors
///
/// [`LiveMinimalityReportRefusal`], naming the first disagreement found
/// in the order the checks are written.
pub fn validate_live_minimality_report(
    report: LiveTransferMinimalityReport,
    target: &TargetProjection,
) -> Result<ValidatedLiveTransferMinimalityReport, LiveMinimalityReportRefusal> {
    if report.schema != LIVE_MINIMALITY_REPORT_SCHEMA {
        return Err(LiveMinimalityReportRefusal::UnsupportedSchema(
            report.schema,
        ));
    }
    if report.role != LiveMinimalityReportRole::LiveTransferMinimality {
        return Err(LiveMinimalityReportRefusal::WrongRole(report.role));
    }
    if &report.target != target {
        return Err(LiveMinimalityReportRefusal::TargetDiffers);
    }
    if report.representations != BOTH_PLANS.iter().copied().collect() {
        return Err(LiveMinimalityReportRefusal::RepresentationCensusDiffers);
    }

    let rows =
        build_minimality_pairs().map_err(LiveMinimalityReportRefusal::RegistryUnavailable)?;
    if report.rows != rows {
        return Err(LiveMinimalityReportRefusal::RowsDiffer);
    }

    let recomputed = pair_census(&rows);
    if report.census != recomputed {
        return Err(LiveMinimalityReportRefusal::CensusDiffers {
            reported: report.census,
            recomputed,
        });
    }
    let standing = recomputed.standing();
    if report.standing != standing {
        return Err(LiveMinimalityReportRefusal::StandingDiffers {
            reported: report.standing,
            recomputed: standing,
        });
    }

    if report.failures != resolve_failure_modes(&rows) {
        return Err(LiveMinimalityReportRefusal::FailureTaxonomyDiffers);
    }
    if report.non_claims != PrivacyNonClaim::ALL.iter().copied().collect() {
        return Err(LiveMinimalityReportRefusal::NonClaimCensusDiffers);
    }
    if report.deployment.is_some() {
        return Err(LiveMinimalityReportRefusal::DeploymentWithoutARun);
    }
    if report.lifecycle != LiveLifecycleStatus::candidate() {
        return Err(LiveMinimalityReportRefusal::LifecycleDiffers);
    }

    let validated = ValidatedLiveTransferMinimalityReport {
        conditions: condition_scoreboard(&rows),
        blockers: row_blockers(&rows),
        recomputed_items: RecomputedItem::ALL.iter().copied().collect(),
        report,
    };

    // §16.2's last two report-side conditions, checked on the bytes this
    // validation is about to bless. A report that published a sponsor
    // value or a fixture opening is refused rather than returned with the
    // condition marked outstanding.
    if let Some(key) = forbidden_key_in(&render_live_minimality_report(&validated)) {
        return Err(
            LiveMinimalityReportRefusal::CanonicalBytesPublishAForbiddenKey(key.to_owned()),
        );
    }
    Ok(validated)
}

/// Every blocker the rows name, with how many rows name it.
fn row_blockers(rows: &[MinimalityPairRow]) -> BTreeMap<LiveInfrastructureBlocker, usize> {
    let mut census = BTreeMap::new();
    for row in rows {
        for blocker in row.blockers() {
            *census.entry(blocker).or_insert(0_usize) += 1;
        }
    }
    census
}

/// Every key the canonical minimality rendering is forbidden to emit.
///
/// §1.9's sponsor list and §6.4's opening list together, as keys rather
/// than as substrings, for the reason the safety report's own list gives:
/// what is forbidden is a value being *emitted*, which is a statement
/// about what a field carries and not about whether a word appears.
///
/// The opening half is this document's own, and §16.2 is why: a
/// minimality report is the one document whose subject is exactly the
/// material §6.4 keeps out of reports, so it needs the sharper list.
pub const FORBIDDEN_KEYS: &[&str] = &[
    "sponsor_amount",
    "sponsor_amounts",
    "sponsor_opening",
    "sponsor_openings",
    "sponsor_value",
    "sponsor_values",
    "sponsor_blinding",
    "sponsor_change_amount",
    "sponsor_total",
    "blinding_factor",
    "value_blinding",
    "fixture_opening",
    "fixture_openings",
    "private_opening",
    "private_openings",
    "receipt_amount",
    "receipt_amounts",
    "input_amount",
    "output_amount",
    "value_commitment",
    "commitment_opening",
];

/// The first forbidden key one rendering emits, if it emits one.
///
/// Compared against each line's first whitespace-delimited token, which
/// is that line's key. A value that happened to spell a forbidden key is
/// not a key, and §1.9 forbids the emission rather than the word.
#[must_use]
pub fn forbidden_key_in(rendered: &str) -> Option<&'static str> {
    rendered
        .lines()
        .filter_map(|line| line.split_whitespace().next())
        .find_map(|key| FORBIDDEN_KEYS.iter().copied().find(|entry| *entry == key))
}

/// Whether the canonical bytes publish no sponsor value and no private
/// opening (§16.2).
#[must_use]
pub fn canonical_bytes_publish_no_forbidden_key(
    validated: &ValidatedLiveTransferMinimalityReport,
) -> bool {
    forbidden_key_in(&render_live_minimality_report(validated)).is_none()
}

/// Render the canonical bytes of one validated minimality report.
///
/// Takes a validated report and nothing else. §13.5's ten volatile fields
/// are not filtered here — they are not reachable from this function's one
/// argument.
///
/// # What is deliberately absent
///
/// No amount, no commitment, no opening, and no target byte. The rows
/// carry finalized transactions and this rendering carries their
/// *shapes*: what a row publishes is how many receipts it consumed, how
/// many it created, and which standing each §16.2 condition has. A
/// canonical minimality report that printed the values would be the
/// §16.4 failure mode it exists to report on.
///
/// # Panics
///
/// Never: the sink is a `String`, whose writes cannot fail.
#[must_use]
pub fn render_live_minimality_report(validated: &ValidatedLiveTransferMinimalityReport) -> String {
    let report = &validated.report;
    let mut text = String::new();
    let _ = writeln!(text, "schema {LIVE_MINIMALITY_SCHEMA_ID}");
    let _ = writeln!(text, "schema_version {}", report.schema);
    let _ = writeln!(text, "role {}", report.role.name());
    let _ = writeln!(text, "operation transfer-live-receipts");
    let _ = writeln!(text, "candidate true");
    let _ = writeln!(text, "target_contract {:?}", report.target.version());
    let _ = writeln!(
        text,
        "execution_domain {:?}",
        report.target.execution_domain()
    );
    let _ = writeln!(text, "deployment none");
    for representation in &report.representations {
        let _ = writeln!(text, "representation {representation:?}");
    }

    let _ = writeln!(text, "pairs {}", report.census.pairs());
    let _ = writeln!(text, "claimed {}", report.census.claimed());
    let _ = writeln!(text, "unclaimed {}", report.census.unclaimed());
    let _ = writeln!(text, "supporting {}", report.census.supporting());

    render_rows(&mut text, report);
    render_conditions(&mut text, validated);
    render_disclosure(&mut text);

    for (mode, standing) in &report.failures {
        let _ = writeln!(text, "failure_mode {} {}", mode.name(), standing.name());
    }
    for non_claim in &report.non_claims {
        let _ = writeln!(text, "non_claim {}", non_claim.name());
    }
    for (blocker, pairs) in &validated.blockers {
        let _ = writeln!(text, "blocker {blocker:?} {pairs}");
    }
    for item in &validated.recomputed_items {
        let _ = writeln!(text, "recomputed {item:?}");
    }

    for exit in report.lifecycle.implemented() {
        let _ = writeln!(text, "lifecycle_implemented {exit}");
    }
    for exit in report.lifecycle.outstanding() {
        let _ = writeln!(text, "lifecycle_outstanding {exit}");
    }
    let _ = writeln!(
        text,
        "release_complete {}",
        report.lifecycle.release_complete()
    );
    let _ = writeln!(
        text,
        "lifecycle_conclusion {}",
        report.lifecycle_conclusion.name()
    );
    let _ = writeln!(text, "minimality {}", report.standing.name());
    text
}

/// The §13.3 rows, as canonical lines.
///
/// Shapes and standings, and no value: a canonical minimality report that
/// printed the amounts would be the §16.4 failure mode it exists to
/// report on.
fn render_rows(text: &mut String, report: &LiveTransferMinimalityReport) {
    for row in &report.rows {
        let _ = writeln!(
            text,
            "pair {} claim {} consumed {} created {} sponsor {}",
            row.pair().name(),
            if row.claim().is_claimed() {
                "claimed"
            } else {
                "not-claimed"
            },
            row.expected().consumed_receipts(),
            row.fixture().destinations().len(),
            row.fixture().sponsor().name(),
        );
        for member in row.members() {
            let _ = writeln!(
                text,
                "member {} {:?} verdict {} predecessor {:?}",
                row.pair().name(),
                member.representation(),
                member.verdict().name(),
                member.predecessor(),
            );
        }
        for blocker in row.blockers() {
            let _ = writeln!(text, "pair_blocker {} {blocker:?}", row.pair().name());
        }
        let _ = writeln!(
            text,
            "pair_resources {} {:?}",
            row.pair().name(),
            row.resources(),
        );
    }
}

/// The §16.2 scoreboard, as canonical lines.
fn render_conditions(text: &mut String, validated: &ValidatedLiveTransferMinimalityReport) {
    for (condition, (satisfied, standings)) in &validated.conditions {
        let _ = writeln!(text, "condition {} satisfied {satisfied}", condition.name());
        for standing in standings {
            let _ = writeln!(
                text,
                "condition_deficit {} {}",
                condition.name(),
                standing.name()
            );
        }
    }
}

/// The §16.3 comparison, as canonical lines.
fn render_disclosure(text: &mut String) {
    for plan in BOTH_PLANS {
        for entry in disclosure_table(*plan) {
            let _ = writeln!(
                text,
                "disclosure {plan:?} {} class {} standing {}",
                entry.item().name(),
                entry.class().name(),
                entry.standing().name(),
            );
        }
        for (item, reason) in additional_exact_amount_disclosures(*plan) {
            let _ = writeln!(
                text,
                "additional_disclosure {plan:?} {} reason {}",
                item.name(),
                reason.name(),
            );
        }
        for item in shape_leakage(*plan) {
            let _ = writeln!(text, "shape_leakage {plan:?} {}", item.name());
        }
    }
    for (item, (explicit, private)) in disclosure_difference() {
        let _ = writeln!(
            text,
            "disclosure_difference {} explicit {} private {}",
            item.name(),
            explicit.name(),
            private.name(),
        );
    }
}

/// One row of the §13.3 table, as a reader's summary.
///
/// The pair, whether it is claimed, and every blocker it names. Offered
/// beside the canonical rendering because a caller wanting the standing
/// of one pair should not have to parse bytes for it.
#[must_use]
pub fn pair_standings(
    validated: &ValidatedLiveTransferMinimalityReport,
) -> BTreeMap<MinimalityPair, (bool, BTreeSet<LiveInfrastructureBlocker>)> {
    validated
        .report
        .rows
        .iter()
        .map(|row| (row.pair(), (row.claim().is_claimed(), row.blockers())))
        .collect()
}

/// One representation plan's whole disclosure position (§16.3).
///
/// A named value rather than a tuple, because the three parts answer
/// three different questions and a reader of a tuple would have to
/// remember which slot was the leakage.
#[derive(Clone, Debug, Eq, PartialEq)]
#[expect(
    clippy::zero_sized_map_values,
    reason = "§16.3 requires each additional disclosure to name its own \
              reason, so the association is the value; the reason census \
              happens to have one member today and a set would drop the \
              association rather than represent it"
)]
pub struct PlanDisclosure {
    table: Vec<DisclosureRow>,
    additional: BTreeMap<DisclosedItem, AdditionalDisclosureReason>,
    leakage: BTreeSet<DisclosedItem>,
}

impl PlanDisclosure {
    /// Every §16.3 item's standing under this plan.
    #[must_use]
    pub fn table(&self) -> &[DisclosureRow] {
        &self.table
    }

    /// Every additional exact amount disclosure, with its typed reason.
    #[must_use]
    #[expect(
        clippy::zero_sized_map_values,
        reason = "the association is the value; see the type's own note"
    )]
    pub const fn additional(&self) -> &BTreeMap<DisclosedItem, AdditionalDisclosureReason> {
        &self.additional
    }

    /// Every item the transaction's shape leaks under this plan.
    #[must_use]
    pub const fn leakage(&self) -> &BTreeSet<DisclosedItem> {
        &self.leakage
    }
}

/// The disclosure comparison, as the report carries it.
#[must_use]
pub fn disclosure_comparison() -> BTreeMap<LiveTransferRepresentationPlan, PlanDisclosure> {
    BOTH_PLANS
        .iter()
        .map(|plan| {
            (
                *plan,
                PlanDisclosure {
                    table: disclosure_table(*plan).to_vec(),
                    additional: additional_exact_amount_disclosures(*plan),
                    leakage: shape_leakage(*plan),
                },
            )
        })
        .collect()
}

/// The standing of one disclosed item under each plan.
#[must_use]
pub fn item_standings()
-> BTreeMap<DisclosedItem, BTreeMap<LiveTransferRepresentationPlan, DisclosureStanding>> {
    let mut standings: BTreeMap<
        DisclosedItem,
        BTreeMap<LiveTransferRepresentationPlan, DisclosureStanding>,
    > = BTreeMap::new();
    for plan in BOTH_PLANS {
        for entry in disclosure_table(*plan) {
            standings
                .entry(entry.item())
                .or_default()
                .insert(*plan, entry.standing());
        }
    }
    standings
}

#[cfg(test)]
mod tests {
    use super::{
        FailureModeStanding, LIVE_MINIMALITY_SCHEMA_ID, LifecycleConclusion,
        LiveMinimalityDiagnostics, LiveMinimalityReportRefusal, LiveMinimalityReportRole,
        MinimalityFailureMode, MinimalityStanding, PrivacyNonClaim,
        assemble_live_minimality_report, canonical_bytes_publish_no_forbidden_key,
        disclosure_comparison, forbidden_key_in, item_standings, pair_standings,
        render_live_minimality_report, validate_live_minimality_report,
    };
    use crate::live_disclosure::{DisclosedItem, DisclosureStanding};
    use crate::live_pairs::MinimalityPair;
    use crate::live_plan::reviewed_target;
    use crate::live_report::{RecomputedItem, VolatileField};
    use compiler::live_transfer_plan::LiveTransferRepresentationPlan;
    use std::collections::BTreeSet;
    use target_elements::TargetProjection;

    fn projection() -> TargetProjection {
        reviewed_target()
            .expect("the reviewed contract validates")
            .projection()
    }

    #[test]
    fn a_minimality_report_validates_against_the_registry_it_is_about() {
        let target = projection();
        let report = assemble_live_minimality_report(target.clone()).expect("the report assembles");
        let validated =
            validate_live_minimality_report(report, &target).expect("the report validates");
        assert_eq!(
            validated.recomputed_items().len(),
            RecomputedItem::ALL.len()
        );
        assert_eq!(validated.recomputed_items().len(), 14);
        assert_eq!(pair_standings(&validated).len(), MinimalityPair::ALL.len());
    }

    #[test]
    fn the_report_says_which_pairs_minimality_is_supported_for() {
        // The wave's whole finding, in the document that carries it.
        // The token read `unanswered` for as long as no pair satisfied
        // the ten conjuncts, and a report claiming a minimality result
        // then would have been claiming one for pairs neither of whose
        // halves had ever reached a target.
        //
        // Both halves of ALL FIVE pairs have now reached one, so the
        // token moves to the stronger of the two. It moves because the
        // census supports it and for no other reason: the weaker token
        // stood while any pair was short, and no pair is short.
        //
        // What the stronger token does NOT say is that every pair
        // MEMBER was submitted. ONE pair's were: the pairs arc
        // materialized §16.1's one-to-one fixture twice and a node
        // accepted both members, and `PairTargetVerdict::Accepted` is
        // the variant that says so — the only spelling in that type
        // which does not begin `not-submitted`. The OTHER FOUR pairs
        // cite an acceptance of each member's own SHAPE, which is this
        // workspace's rule for moving a row and is a narrower fact than
        // §16.2's words. The registry spells the difference at every one
        // of these identities, and this token inherits the narrowness of
        // the four rather than escaping it on the strength of the one.
        let target = projection();
        let report = assemble_live_minimality_report(target.clone()).expect("the report assembles");
        assert_eq!(
            report.standing(),
            MinimalityStanding::SupportedForEveryClaimedPair,
        );
        assert_eq!(report.census().pairs(), MinimalityPair::ALL.len());
        assert_eq!(report.census().claimed(), MinimalityPair::ALL.len());
        assert_eq!(report.census().supporting(), 5);
        // And NOT the weaker token, which is the assertion that now
        // costs something: a census supporting every claimed pair and a
        // report still hedging would be a report refusing to say what
        // it recomputed.
        assert_ne!(
            report.standing(),
            MinimalityStanding::SupportedForSomeClaimedPairsOnly,
        );

        let validated =
            validate_live_minimality_report(report, &target).expect("the report validates");
        // NO named component, and it used to be one. Every blocker this
        // report ever carried named something that does not exist, and
        // the last of them is in the guide closeout's CLEARED set. What
        // the two unsupported pairs lack is a run of one shape each,
        // which the registry names on the acceptance conjunct — a run
        // not yet attempted is not a missing component, and a report
        // naming one would be counting a cleared blocker as a deficit.
        assert_eq!(
            validated
                .blockers()
                .keys()
                .copied()
                .collect::<BTreeSet<_>>(),
            BTreeSet::new(),
        );
        let standings = pair_standings(&validated);
        assert_eq!(standings.len(), MinimalityPair::ALL.len());
        for (pair, (claimed, blockers)) in &standings {
            assert!(*claimed, "{} is unclaimed", pair.name());
            assert_eq!(blockers.len(), 0, "{} names a blocker", pair.name());
        }
    }

    #[test]
    fn a_report_claiming_a_standing_its_census_does_not_support_is_refused() {
        // The recomputation made falsifiable: the validator derives the
        // token and compares rather than reading it.
        let target = projection();
        let mut report =
            assemble_live_minimality_report(target.clone()).expect("the report assembles");
        // The injected token is the WEAKER one now, because the
        // recomputation produces the stronger. Falsifying a derivation
        // means claiming something it did not derive, whichever
        // direction that happens to lie in.
        report.standing = MinimalityStanding::SupportedForSomeClaimedPairsOnly;
        assert_eq!(
            validate_live_minimality_report(report, &target),
            Err(LiveMinimalityReportRefusal::StandingDiffers {
                reported: MinimalityStanding::SupportedForSomeClaimedPairsOnly,
                recomputed: MinimalityStanding::SupportedForEveryClaimedPair,
            }),
        );
    }

    #[test]
    fn a_document_of_the_other_schema_is_refused_by_the_minimality_validator() {
        // §13.6's separation, enforced rather than asserted.
        let target = projection();
        let mut report =
            assemble_live_minimality_report(target.clone()).expect("the report assembles");
        report.schema = super::LIVE_MINIMALITY_REPORT_SCHEMA + 1;
        assert_eq!(
            validate_live_minimality_report(report, &target),
            Err(LiveMinimalityReportRefusal::UnsupportedSchema(
                super::LIVE_MINIMALITY_REPORT_SCHEMA + 1
            )),
        );
    }

    #[test]
    fn the_two_reports_are_different_documents_at_their_first_line() {
        // §13.6. The safety report and this one are rendered side by
        // side and compared where a reader would look: the schema line
        // and the role line. A reader holding one document must never
        // have to guess which kind it is.
        use crate::live_evidence::derive_live_evidence_plan;
        use crate::live_report::{
            LiveSafetyReportRole, assemble_live_safety_report, render_live_safety_report,
            validate_live_safety_report,
        };

        let target = projection();
        let plan = derive_live_evidence_plan().expect("the evidence plan derives");
        let safety = assemble_live_safety_report(&plan, target.clone())
            .expect("the safety report assembles");
        let safety = validate_live_safety_report(safety, &plan, &target).expect("validates");
        let safety = render_live_safety_report(&safety);

        let report = assemble_live_minimality_report(target.clone()).expect("the report assembles");
        let report = validate_live_minimality_report(report, &target).expect("validates");
        let minimality = render_live_minimality_report(&report);

        let first = |text: &str| text.lines().next().unwrap_or_default().to_owned();
        assert_ne!(first(&safety), first(&minimality));
        assert_eq!(
            first(&minimality),
            format!("schema {LIVE_MINIMALITY_SCHEMA_ID}")
        );
        assert_ne!(
            LiveMinimalityReportRole::LiveTransferMinimality.name(),
            LiveSafetyReportRole::LiveTransferSafety.name(),
        );
        assert!(minimality.contains("role live-transfer-minimality"));
        assert!(!minimality.contains("role live-transfer-safety"));
        assert!(!safety.contains("role live-transfer-minimality"));
    }

    #[test]
    fn the_canonical_bytes_carry_no_volatile_field() {
        // §13.5's exclusion, checked rather than argued. Each of the ten
        // fields is given a value nothing else in the workspace produces
        // and recorded in this document's own diagnostic report; the
        // canonical bytes are then searched for every one. The renderer
        // takes no diagnostic argument, so a hit would mean some other
        // carrier had smuggled a volatile value in.
        let target = projection();
        let report = assemble_live_minimality_report(target.clone()).expect("the report assembles");
        let validated = validate_live_minimality_report(report, &target).expect("validates");
        let canonical = render_live_minimality_report(&validated);

        let mut diagnostics = LiveMinimalityDiagnostics::default();
        let mut markers = Vec::with_capacity(VolatileField::ALL.len());
        for (index, field) in VolatileField::ALL.iter().enumerate() {
            let marker = format!("minimality-volatile-{index}-{field:?}");
            diagnostics = diagnostics.with(*field, marker.clone());
            markers.push(marker);
        }
        assert_eq!(diagnostics.fields().len(), 10);
        for marker in &markers {
            assert!(
                !canonical.contains(marker.as_str()),
                "the canonical bytes carry {marker}",
            );
            assert!(diagnostics.render().contains(marker.as_str()));
        }
        // And no line's *key* is a timing one. Checked by key rather than
        // by substring, which the safety report can afford and this
        // document cannot: §16.5's non-claim census contains
        // `production-wallet-privacy`, and a search for the letters of
        // "wall" anywhere in the bytes would flag the report for
        // publishing its own disclaimer. §13.5 excludes a wall-clock
        // *field*, which is a statement about what a line carries.
        let keys: BTreeSet<_> = canonical
            .lines()
            .filter_map(|line| line.split_whitespace().next())
            .collect();
        for timing in ["wall_clock", "wall_clock_time", "elapsed", "elapsed_time"] {
            assert!(!keys.contains(timing), "the canonical bytes carry {timing}");
        }
        for field in VolatileField::ALL {
            let key = format!("{field:?}");
            assert!(
                !keys.contains(key.as_str()),
                "the canonical bytes carry a {key} field",
            );
        }
        assert!(diagnostics.render().contains("canonical false"));
    }

    #[test]
    fn the_canonical_bytes_are_stable_across_renderings() {
        let target = projection();
        let report = assemble_live_minimality_report(target.clone()).expect("the report assembles");
        let validated = validate_live_minimality_report(report, &target).expect("validates");
        assert_eq!(
            render_live_minimality_report(&validated),
            render_live_minimality_report(&validated),
        );
    }

    #[test]
    fn the_canonical_bytes_publish_no_sponsor_value_and_no_private_opening() {
        // §16.2's last two conditions, and the two of them that are
        // checkable today. The validator already refuses a report that
        // fails this, so holding a validated one is itself the evidence;
        // the check is repeated here so the property is a test rather
        // than a side effect.
        let target = projection();
        let report = assemble_live_minimality_report(target.clone()).expect("the report assembles");
        let validated = validate_live_minimality_report(report, &target).expect("validates");
        assert!(canonical_bytes_publish_no_forbidden_key(&validated));

        // And the check has teeth. Every forbidden key is staged as a
        // real line appended to the real rendering, over the same key
        // vocabulary the production path reads — so a key removed from
        // the list stops being checked here too.
        let rendered = render_live_minimality_report(&validated);
        for key in super::FORBIDDEN_KEYS {
            let leaked = format!("{rendered}{key} 1000\n");
            assert_eq!(
                forbidden_key_in(&leaked),
                Some(*key),
                "{key} would not have been caught",
            );
        }
        // A forbidden word that is not a key is not a leak: §1.9 forbids
        // the emission, and a line whose *value* spells one carries
        // nothing.
        assert_eq!(
            forbidden_key_in(&format!("{rendered}note sponsor_amount\n")),
            None,
        );
    }

    #[test]
    fn the_failure_taxonomy_and_the_non_claims_are_complete() {
        // §16.4's nine and §16.5's eight, carried as types. Seven modes
        // are checked against the built registry and two are honestly
        // outside this boundary.
        let target = projection();
        let report = assemble_live_minimality_report(target).expect("the report assembles");
        assert_eq!(report.failures().len(), MinimalityFailureMode::ALL.len());
        assert_eq!(report.non_claims().len(), PrivacyNonClaim::ALL.len());
        assert_eq!(
            *report.non_claims(),
            PrivacyNonClaim::ALL
                .iter()
                .copied()
                .collect::<BTreeSet<_>>(),
        );

        assert_eq!(
            report.failures()[&MinimalityFailureMode::PrivateMaterializationRejects],
            // Re-pointed onto an observation. It awaited a run behind a
            // blocker the guide's own closeout carries as CLEARED, so
            // it was awaiting a run nothing was holding up; and what a
            // target has actually said about these shapes is stronger
            // than "awaiting" and weaker than "never rejects".
            FailureModeStanding::NotRejectedOnAnyObservedRun,
        );
        // §18's study filled this one. It is a conjunction, and no pair
        // has an explicit member that completes while its private member
        // does not — measured by completing and weighing both halves of
        // every pair that can be completed at all.
        assert_eq!(
            report.failures()[&MinimalityFailureMode::AHardTargetLimitBlocksThePrivatePlan],
            FailureModeStanding::MeasuredWithinTheDeclaredLimits,
        );
        let observed = report
            .failures()
            .values()
            .filter(|standing| **standing == FailureModeStanding::NotObservedFirstParty)
            .count();
        assert_eq!(observed, 7, "the first-party half of §16.4");
    }

    #[test]
    fn the_report_concludes_what_seventeen_four_permits_and_no_more() {
        // §17.4: equal for transfer semantics, not equal for the complete
        // live-receipt lifecycle. And §17.1's two outstanding exits are
        // what makes the second half true.
        let target = projection();
        let report = assemble_live_minimality_report(target).expect("the report assembles");
        assert_eq!(
            report.lifecycle_conclusion(),
            LifecycleConclusion::EqualForTransferUnequalForTheLifecycle,
        );
        assert!(!report.lifecycle().release_complete());
        assert_eq!(report.lifecycle().outstanding().len(), 2);
        assert_eq!(report.lifecycle().implemented().len(), 1);
    }

    #[test]
    fn the_disclosure_comparison_travels_in_the_canonical_bytes() {
        // §13.5 recomputes a disclosure comparison, and a report that
        // recomputed one without publishing it would leave a reader of
        // the bytes unable to check it.
        let target = projection();
        let report = assemble_live_minimality_report(target.clone()).expect("the report assembles");
        let validated = validate_live_minimality_report(report, &target).expect("validates");
        let rendered = render_live_minimality_report(&validated);

        let comparison = disclosure_comparison();
        assert_eq!(comparison.len(), 2);
        for (plan, position) in &comparison {
            let (table, additional, leakage) =
                (position.table(), position.additional(), position.leakage());
            assert_eq!(table.len(), DisclosedItem::ALL.len());
            for entry in table {
                assert!(
                    rendered.contains(&format!(
                        "disclosure {plan:?} {} class {} standing {}",
                        entry.item().name(),
                        entry.class().name(),
                        entry.standing().name(),
                    )),
                    "{plan:?} {} is not published",
                    entry.item().name(),
                );
            }
            for item in additional.keys() {
                assert!(
                    rendered.contains(&format!("additional_disclosure {plan:?} {}", item.name()))
                );
            }
            for item in leakage {
                assert!(rendered.contains(&format!("shape_leakage {plan:?} {}", item.name())));
            }
        }

        // The item table, from the other direction: every item has a
        // standing under both plans, and the amounts are the ones that
        // differ.
        let standings = item_standings();
        assert_eq!(standings.len(), DisclosedItem::ALL.len());
        for (item, per_plan) in &standings {
            assert_eq!(per_plan.len(), 2, "{} is stated for one plan", item.name());
        }
        assert_eq!(
            standings[&DisclosedItem::ExactOutputReceiptAmounts]
                [&LiveTransferRepresentationPlan::PrivateCommitted],
            DisclosureStanding::Private,
        );
        assert_eq!(
            standings[&DisclosedItem::ExactOutputReceiptAmounts]
                [&LiveTransferRepresentationPlan::Explicit],
            DisclosureStanding::Explicit,
        );
    }
}
