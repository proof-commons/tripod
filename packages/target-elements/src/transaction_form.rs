//! Reviewed target facts about the compact-ASH transaction forms.
//!
//! The rest of this crate reviews what one *program* does when a node
//! executes it. This module reviews what one *transaction* has to look
//! like for a node to accept it at all: how the target names its fee
//! role, whether a sponsorless transaction is constructible, what a
//! sponsored one additionally needs, and which of those answers come
//! from consensus rather than from a relay policy a deployment can
//! change under us.
//!
//! # Why the distinction is load-bearing
//!
//! A backend that cannot tell the two apart will read "the node did not
//! relay it" as "the target rejects it" and make sponsorship mandatory
//! to make a red light go away. The candidate layout declares
//! sponsorship optional, so that confusion would silently change the
//! protocol. Every form below therefore carries two verdicts, and they
//! are separate fields rather than one summary.
//!
//! # This module states no acceptance
//!
//! Everything here is a reviewed reading of target source. Nothing here
//! executes, submits, or observes anything, so nothing here establishes
//! that a real node behaves as described — that is what the evidence
//! requirements at the bottom are for, and they are deliberately
//! unsatisfied by this crate. Nor does anything here claim production
//! activation: the reviewed forms are candidate forms for one Phase-4
//! pipeline whose lifecycle still has an outstanding exit.

use std::collections::{BTreeMap, BTreeSet};

use crate::evidence::TargetEvidenceRequirementId;

/// One structural term the target's fee-role test is made of.
///
/// The target recognizes its fee role by the *form* of the output, and
/// this enumerates the conjuncts of that form. None of them is an
/// amount comparison, which is the fact the protocol layer depends on:
/// a fee output is not "an ordinary output whose amount is zero", and an
/// ordinary output does not become a fee output by holding a small
/// number.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum FeeRecognitionTerm {
    /// The chain runs in the target's own transaction mode.
    ///
    /// Outside it the whole role does not exist, so this is a term of
    /// the test rather than an ambient assumption.
    ElementsTransactionMode,
    /// The output's program is the empty program.
    EmptyProgram,
    /// The output's value field is explicit rather than a commitment.
    ExplicitValue,
    /// The output's asset field is explicit rather than a commitment.
    ExplicitAsset,
}

impl FeeRecognitionTerm {
    /// The complete census of recognition terms.
    pub const ALL: &'static [Self] = &[
        Self::ElementsTransactionMode,
        Self::EmptyProgram,
        Self::ExplicitValue,
        Self::ExplicitAsset,
    ];
}

/// What a rule constrains, where the target constrains it.
///
/// A candidate ABI may fix a position or a count for its own reasons.
/// Recording that the *target* does not lets a later reader tell an ABI
/// convention from a target obligation, which is the difference between
/// a layout the project may revise and one it may not.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum FormConstraint {
    /// The target imposes no constraint of this kind.
    Unconstrained,
    /// Consensus imposes the constraint.
    ConsensusEnforced,
    /// A relay or deployment policy imposes it, and consensus does not.
    PolicyEnforced,
    /// Only a target interface convention imposes it, and neither
    /// consensus nor relay does.
    ///
    /// Weaker than either enforcement, and recorded rather than ignored
    /// because a construction path that goes through such an interface
    /// still has to satisfy it.
    InterfaceConvention,
}

/// How a transaction states that it pays no fee.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ZeroFeeRepresentation {
    /// By carrying no fee output at all.
    AbsentOutput,
    /// By carrying a fee output whose amount is zero.
    ZeroValuedOutput,
}

/// The reviewed target facts about the fee role.
///
/// Constructed once by [`reviewed_fee_output_contract`]; the accessors
/// are the only way to read it, so a consumer cannot quietly assemble a
/// different set of facts and call them reviewed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FeeOutputContract {
    recognition: BTreeSet<FeeRecognitionTerm>,
    cardinality: FormConstraint,
    position: FormConstraint,
    interface_cardinality: FormConstraint,
    interface_position: FormConstraint,
    zero_fee: ZeroFeeRepresentation,
    refused_zero_fee: ZeroFeeRepresentation,
    blindable: bool,
    protocol_object: bool,
}

impl FeeOutputContract {
    /// Every conjunct of the target's fee-role test.
    #[must_use]
    pub const fn recognition(&self) -> &BTreeSet<FeeRecognitionTerm> {
        &self.recognition
    }

    /// Where the count of fee outputs is constrained.
    #[must_use]
    pub const fn cardinality(&self) -> FormConstraint {
        self.cardinality
    }

    /// Where the position of a fee output is constrained.
    #[must_use]
    pub const fn position(&self) -> FormConstraint {
        self.position
    }

    /// Where a target construction interface constrains the count.
    #[must_use]
    pub const fn interface_cardinality(&self) -> FormConstraint {
        self.interface_cardinality
    }

    /// Where a target construction interface constrains the position.
    #[must_use]
    pub const fn interface_position(&self) -> FormConstraint {
        self.interface_position
    }

    /// How a transaction paying nothing states that.
    #[must_use]
    pub const fn zero_fee(&self) -> ZeroFeeRepresentation {
        self.zero_fee
    }

    /// The zero-fee spelling the target refuses.
    #[must_use]
    pub const fn refused_zero_fee(&self) -> ZeroFeeRepresentation {
        self.refused_zero_fee
    }

    /// Whether a fee output may carry blinded fields.
    #[must_use]
    pub const fn blindable(&self) -> bool {
        self.blindable
    }

    /// Whether the fee role is a protocol object.
    ///
    /// It is not, and the field exists so that the answer is stated
    /// rather than assumed: the role is target-structural, and a
    /// protocol relation that recognized it would be recognizing
    /// something the target owns.
    #[must_use]
    pub const fn protocol_object(&self) -> bool {
        self.protocol_object
    }
}

/// The reviewed fee-output contract.
///
/// Source, in the reviewed target checkout: the recognition terms are
/// the conjuncts of `CTxOut::IsFee` in `src/primitives/transaction.h`;
/// the refusal of a zero-amount fee output is `HasValidFee` in
/// `src/confidential_validation.cpp`, reached from
/// `Consensus::CheckTxInputs` in `src/consensus/tx_verify.cpp`; the
/// absence of a consensus count or position rule is the absence of any
/// such test in `CheckTransaction` (`src/consensus/tx_check.cpp`),
/// which constrains fee outputs only for coinbase transactions; the
/// interface convention is the "last output must be fee" and "only one
/// fee output" refusals in `src/rpc/rawtransaction.cpp`; the standardness
/// of the empty program is `Solver` in `src/script/solver.cpp`, which
/// maps it to the target's own fee output class rather than to the
/// nonstandard one; and the refusal to blind it is the fee test in
/// `src/blind.cpp`.
#[must_use]
pub fn reviewed_fee_output_contract() -> FeeOutputContract {
    FeeOutputContract {
        recognition: FeeRecognitionTerm::ALL.iter().copied().collect(),
        cardinality: FormConstraint::Unconstrained,
        position: FormConstraint::Unconstrained,
        interface_cardinality: FormConstraint::InterfaceConvention,
        interface_position: FormConstraint::InterfaceConvention,
        zero_fee: ZeroFeeRepresentation::AbsentOutput,
        refused_zero_fee: ZeroFeeRepresentation::ZeroValuedOutput,
        blindable: false,
        protocol_object: false,
    }
}

/// Which candidate transaction form is under review.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum TransactionForm {
    /// An ASH family alone, paying no fee.
    Sponsorless,
    /// An ASH family plus a sponsor suffix, paying a positive fee.
    Sponsored,
}

impl TransactionForm {
    /// The complete census of reviewed forms.
    pub const ALL: &'static [Self] = &[Self::Sponsorless, Self::Sponsored];
}

/// Whether one layer admits one form.
///
/// `Refused` is not the same as `AdmittedUnderCondition`: the first says
/// the form cannot exist at that layer, the second that it exists and
/// something further has to hold for it to travel. A backend that
/// collapsed the two would be the exact failure this module refuses.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum FormAdmission {
    /// The layer admits the form as it stands.
    Admitted,
    /// The layer admits the form only when a further condition holds.
    AdmittedUnderCondition,
    /// The layer refuses the form.
    Refused,
}

/// A condition a relay policy attaches to a form.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum RelayCondition {
    /// The transaction's own fee must reach the relay floor.
    ///
    /// The floor is a deployment setting, not a target constant, so a
    /// figure is deliberately absent here.
    OwnFeeReachesRelayFloor,
    /// The transaction must travel in a topology-restricted package
    /// whose descendant pays for it.
    TopologyRestrictedPackage,
    /// The transaction must be handed to a block producer directly,
    /// bypassing the relay path.
    DirectSubmissionToProducer,
}

/// The reviewed verdicts for one transaction form.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransactionFormReview {
    form: TransactionForm,
    consensus: FormAdmission,
    relay: FormAdmission,
    relay_conditions: BTreeSet<RelayCondition>,
    fee_output_present: bool,
}

impl TransactionFormReview {
    /// The form reviewed.
    #[must_use]
    pub const fn form(&self) -> TransactionForm {
        self.form
    }

    /// What consensus does with it.
    #[must_use]
    pub const fn consensus(&self) -> FormAdmission {
        self.consensus
    }

    /// What the default relay policy does with it.
    #[must_use]
    pub const fn relay(&self) -> FormAdmission {
        self.relay
    }

    /// The conditions under which the relay path admits it.
    #[must_use]
    pub const fn relay_conditions(&self) -> &BTreeSet<RelayCondition> {
        &self.relay_conditions
    }

    /// Whether the form carries a fee output.
    #[must_use]
    pub const fn fee_output_present(&self) -> bool {
        self.fee_output_present
    }
}

/// The reviewed verdicts for every candidate transaction form.
///
/// Source, in the reviewed target checkout: consensus admission of a
/// fee-free transaction is the absence of any fee requirement in
/// `CheckTransaction` (`src/consensus/tx_check.cpp`) together with the
/// exact balance test `VerifyAmounts` performs in
/// `src/confidential_validation.cpp`, which balances inputs against
/// outputs without a fee term of its own; the relay floor is the
/// non-bypassable minimum-relay test in `AcceptSingleTransaction`'s
/// workspace checks in `src/validation.cpp`, and the package escape and
/// its topology limits are the restricted-topology version carved out
/// of that same test and bounded in `src/policy/truc_policy.h`.
///
/// So a sponsorless compact-ASH transaction is constructible under
/// target consensus, and Phase 4 does not block on this. It is not
/// individually relayable under a default policy, which is a
/// deployment fact and not a target one, and which must not be
/// answered by making sponsorship mandatory.
#[must_use]
pub fn reviewed_transaction_forms() -> BTreeMap<TransactionForm, TransactionFormReview> {
    [
        TransactionFormReview {
            form: TransactionForm::Sponsorless,
            consensus: FormAdmission::Admitted,
            relay: FormAdmission::AdmittedUnderCondition,
            relay_conditions: [
                RelayCondition::TopologyRestrictedPackage,
                RelayCondition::DirectSubmissionToProducer,
            ]
            .into_iter()
            .collect(),
            fee_output_present: false,
        },
        TransactionFormReview {
            form: TransactionForm::Sponsored,
            consensus: FormAdmission::Admitted,
            relay: FormAdmission::AdmittedUnderCondition,
            relay_conditions: std::iter::once(RelayCondition::OwnFeeReachesRelayFloor).collect(),
            fee_output_present: true,
        },
    ]
    .into_iter()
    .map(|review| (review.form, review))
    .collect()
}

/// How the target treats an output whose explicit value is zero.
///
/// Recorded as its own reviewed fact because the candidate layout has
/// an optional role — sponsor change — whose amount may legitimately
/// come out zero, and because the answer turns out to be a consensus
/// rule rather than the construction policy it is easy to mistake it
/// for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ExplicitZeroValueRule {
    /// A zero-valued output whose program can be spent is refused.
    SpendableRefused,
    /// A zero-valued output whose program cannot be spent is admitted
    /// and contributes nothing to the balance.
    UnspendableAdmitted,
}

impl ExplicitZeroValueRule {
    /// The complete census of the reviewed rule's parts.
    pub const ALL: &'static [Self] = &[Self::SpendableRefused, Self::UnspendableAdmitted];
}

/// The reviewed explicit-zero-value rule.
///
/// Source, in the reviewed target checkout: the explicit-value branch
/// of the output loop in `VerifyAmounts`
/// (`src/confidential_validation.cpp`), which skips a zero-valued
/// output whose program is unspendable and refuses the whole
/// transaction otherwise, and `CScript::IsUnspendable`
/// (`src/script/script.h`), which counts the empty program as
/// unspendable in the target's transaction mode.
///
/// The consequence for the candidate layout: a known-zero sponsor
/// change output is not merely omitted as construction policy, it
/// cannot be carried at all while its program is spendable. Omitting it
/// is therefore forced, and a builder that emitted it would produce a
/// transaction consensus refuses.
#[must_use]
pub fn reviewed_explicit_zero_value_rule() -> BTreeSet<ExplicitZeroValueRule> {
    ExplicitZeroValueRule::ALL.iter().copied().collect()
}

/// Where the target checks a stated amount against its money bound.
///
/// Two separate refusals, because they are two separate comparisons and
/// a builder that only knew about the first would still produce a
/// transaction the target refuses.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum StatedAmountCheck {
    /// One output's own explicit value is compared against the bound.
    SingleOutputValue,
    /// The running total of explicit output values is compared against
    /// it as each output is added.
    RunningOutputTotal,
}

impl StatedAmountCheck {
    /// The complete census of the reviewed checks.
    pub const ALL: &'static [Self] = &[Self::SingleOutputValue, Self::RunningOutputTotal];
}

/// The greatest value the target admits in an explicit amount field.
///
/// # Why this is a target fact and not a protocol one
///
/// The protocol's own amount domain and this bound are two different
/// ceilings, and neither is derived from the other. A world the
/// protocol admits may state an amount this target refuses to encode at
/// all, which is a divergence between the two domains rather than a
/// defect in either. Recording the bound here — beside the other
/// reviewed facts about what a transaction must look like to be
/// accepted — is what lets a planner derive that divergence instead of
/// discovering it as an unexplained refusal.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StatedAmountBound {
    maximum: u64,
}

impl StatedAmountBound {
    /// The greatest admissible value of one explicit amount field.
    #[must_use]
    pub const fn maximum(self) -> u64 {
        self.maximum
    }

    /// Whether the target admits an amount of this size.
    #[must_use]
    pub const fn admits(self, amount: u64) -> bool {
        amount <= self.maximum
    }
}

/// The reviewed stated-amount bound.
///
/// Source, in the reviewed target checkout: `MAX_MONEY` in
/// `src/consensus/amount.h`, which is `21000000 * COIN` with `COIN` at
/// `100000000`, and the explicit-output loop of `CheckTransaction` in
/// `src/consensus/tx_check.cpp`, which refuses one output above the
/// bound as `bad-txns-vout-toolarge` and a running explicit total above
/// it as `bad-txns-txouttotal-toolarge`.
///
/// The bound is stated once, as a number, rather than as the two
/// diagnostics: a caller that matched on the spellings would be
/// asserting which of the target's several gates answers first, and on
/// a funding path an adapter's own reserve arithmetic can answer ahead
/// of all of them.
#[must_use]
pub const fn reviewed_stated_amount_bound() -> StatedAmountBound {
    StatedAmountBound {
        maximum: 21_000_000 * 100_000_000,
    }
}

/// The reviewed census of where that bound is checked.
#[must_use]
pub fn reviewed_stated_amount_checks() -> BTreeSet<StatedAmountCheck> {
    StatedAmountCheck::ALL.iter().copied().collect()
}

/// Which field of a sponsor input the coordinator reads.
///
/// The census is short on purpose. Each entry is a field the protocol
/// relation authenticates, and the sponsor's individual amount is
/// absent from it — not because reading it would be inconvenient, but
/// because the sponsor-erasure law forbids the protocol relation from
/// depending on it at all.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SponsorInspectedField {
    /// The input's position relative to the authenticated family range.
    RegionMembership,
    /// The declared start and length of the sponsor region.
    RegionExtent,
    /// The asset the input carries.
    Asset,
}

impl SponsorInspectedField {
    /// The complete census of inspected fields.
    pub const ALL: &'static [Self] = &[Self::RegionMembership, Self::RegionExtent, Self::Asset];
}

/// The form a sponsor input's field must take to be authenticable.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum FieldForm {
    /// The field must be explicit; a commitment cannot satisfy the test.
    ExplicitOnly,
    /// The field may be explicit or a commitment, because nothing tests
    /// it.
    EitherUninspected,
}

/// One class of sponsor spending condition the candidate admits.
///
/// Admission here is a statement about what the *builder* will issue a
/// signing request for and what the Phase-4 evidence will exercise. It
/// is not a claim that the coordinator authenticates the class, and it
/// is not a claim of arbitrary wallet support.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SponsorProgramClass {
    /// A single-key native witness program of the target's version-zero
    /// key-hash form.
    ///
    /// Chosen because it is what an unconfigured target wallet hands
    /// out, so the development sponsor adapter produces it without a
    /// setting the evidence would then depend on.
    WitnessV0KeyHash,
}

impl SponsorProgramClass {
    /// The complete census of admitted classes.
    pub const ALL: &'static [Self] = &[Self::WitnessV0KeyHash];
}

/// Where a sponsor input's authorization comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SponsorAuthorizationSource {
    /// From the input's own target spending condition, satisfied by the
    /// sponsor, and not from anything the coordinator program checks.
    OwnTargetSpendingCondition,
}

/// The admitted sponsor input authorization profile.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SponsorInputProfile {
    source: SponsorAuthorizationSource,
    inspected: BTreeSet<SponsorInspectedField>,
    asset_form: FieldForm,
    value_form: FieldForm,
    admitted_programs: BTreeSet<SponsorProgramClass>,
    arbitrary_program_support_claimed: bool,
}

impl SponsorInputProfile {
    /// Where authorization comes from.
    #[must_use]
    pub const fn source(&self) -> SponsorAuthorizationSource {
        self.source
    }

    /// The fields the protocol relation reads.
    #[must_use]
    pub const fn inspected(&self) -> &BTreeSet<SponsorInspectedField> {
        &self.inspected
    }

    /// The form the asset field must take.
    #[must_use]
    pub const fn asset_form(&self) -> FieldForm {
        self.asset_form
    }

    /// The form the value field may take.
    #[must_use]
    pub const fn value_form(&self) -> FieldForm {
        self.value_form
    }

    /// The spending-condition classes the candidate admits.
    #[must_use]
    pub const fn admitted_programs(&self) -> &BTreeSet<SponsorProgramClass> {
        &self.admitted_programs
    }

    /// Whether arbitrary wallet-program support is claimed.
    #[must_use]
    pub const fn arbitrary_program_support_claimed(&self) -> bool {
        self.arbitrary_program_support_claimed
    }
}

/// The reviewed sponsor input authorization profile.
///
/// Source, in the reviewed target checkout: the asset a sponsor input
/// carries reaches a program as a payload and a one-byte prefix, and
/// only the explicit prefix carries the asset itself, which is the
/// `pushasset` helper in `src/script/interpreter.cpp`; the value
/// reaches a program the same way through `pushvalue` in the same file,
/// which is why leaving the value uninspected is what preserves sponsor
/// opacity rather than a courtesy; the default wallet program class is
/// the target's own default address type in `src/wallet/wallet.h`.
///
/// The consequence worth stating plainly: requiring the reserve asset
/// forces a sponsor input's *asset* to be unblinded, while its *value*
/// may remain a commitment. Sponsor value opacity therefore survives
/// this profile, and sponsor asset opacity does not exist under it. A
/// deployment that needed the latter would need a different relation,
/// not a different builder.
#[must_use]
pub fn reviewed_sponsor_input_profile() -> SponsorInputProfile {
    SponsorInputProfile {
        source: SponsorAuthorizationSource::OwnTargetSpendingCondition,
        inspected: SponsorInspectedField::ALL.iter().copied().collect(),
        asset_form: FieldForm::ExplicitOnly,
        value_form: FieldForm::EitherUninspected,
        admitted_programs: SponsorProgramClass::ALL.iter().copied().collect(),
        arbitrary_program_support_claimed: false,
    }
}

/// Which transaction substrate the project selected.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SubstrateSelection {
    /// First-party structures, owned and reviewed in this workspace.
    FirstParty,
    /// A reviewed third-party library.
    ThirdParty,
}

/// What has and has not happened to a recorded decision.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum DecisionStatus {
    /// Reviewed and recorded, and not yet ratified.
    AwaitingRatification,
}

/// One ground on which the third-party candidate was not selected.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SubstrateGround {
    /// No package in this workspace consumes it yet.
    ///
    /// The consuming package is later work, and the standing rule
    /// admits a dependency only with a real consumer, so admitting it
    /// now would be admitting it on a plan.
    NoPresentConsumer,
    /// Its required graph reaches native code through a linked system
    /// library with a build script.
    NativeLinkedDependency,
    /// Its required graph reaches the same native library the target
    /// itself vendors, so agreement between the two would be one
    /// opinion counted twice.
    SharedImplementationWithTarget,
    /// Its licence is a different instrument from the workspace's, and
    /// adopting it is a licensing decision rather than a technical one.
    DistinctLicenceInstrument,
    /// The candidate's confidential and issuance surface is not on the
    /// candidate pipeline's path, so most of what it supplies would be
    /// carried unused.
    SurfaceExceedsCandidateNeed,
}

impl SubstrateGround {
    /// The complete census of recorded grounds.
    pub const ALL: &'static [Self] = &[
        Self::NoPresentConsumer,
        Self::NativeLinkedDependency,
        Self::SharedImplementationWithTarget,
        Self::DistinctLicenceInstrument,
        Self::SurfaceExceedsCandidateNeed,
    ];
}

/// A fact that would reopen the substrate decision.
///
/// Stated so that the decision is falsifiable rather than permanent. A
/// first-party substrate is the right answer for the reviewed candidate
/// need, and each trigger below names a need the review found to be
/// outside it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SubstrateRevisitTrigger {
    /// Blinded fields must be produced inside this workspace rather
    /// than by a sponsor adapter.
    InProcessBlinding,
    /// A transaction digest for signing must be computed inside this
    /// workspace rather than requested from a signer.
    InProcessSighash,
    /// Issuance or reissuance fields must be constructed inside this
    /// workspace rather than by a target funding interface.
    InProcessIssuance,
}

impl SubstrateRevisitTrigger {
    /// The complete census of triggers.
    pub const ALL: &'static [Self] = &[
        Self::InProcessBlinding,
        Self::InProcessSighash,
        Self::InProcessIssuance,
    ];
}

/// One capability the first-party substrate must own.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum FirstPartyCapability {
    /// Encoding a transaction whose asset and value fields are all
    /// explicit.
    ExplicitTransactionEncoding,
    /// Decoding target bytes back into validated typed values.
    ExplicitTransactionDecoding,
    /// Assembling a script-path witness from linked constructor data.
    ScriptPathWitnessAssembly,
    /// Stating the canonical role layout the candidate ABI fixes.
    CanonicalRoleLayout,
}

impl FirstPartyCapability {
    /// The complete census of first-party capabilities.
    pub const ALL: &'static [Self] = &[
        Self::ExplicitTransactionEncoding,
        Self::ExplicitTransactionDecoding,
        Self::ScriptPathWitnessAssembly,
        Self::CanonicalRoleLayout,
    ];
}

/// One capability the first-party substrate deliberately does not own.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum DelegatedCapability {
    /// Producing blinded fields and their proofs.
    Blinding,
    /// Computing a signing digest and producing a signature.
    Signing,
    /// Creating the disposable test asset the development fixtures
    /// stand on.
    TestAssetIssuance,
}

impl DelegatedCapability {
    /// The complete census of delegated capabilities.
    pub const ALL: &'static [Self] = &[Self::Blinding, Self::Signing, Self::TestAssetIssuance];
}

/// The reviewed third-party substrate candidate.
///
/// Every field is a pinned reading rather than a general impression of
/// the library. A review of "the latest version" would not be a review.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ThirdPartySubstrateReview {
    package: &'static str,
    version: &'static str,
    upstream: &'static str,
    licence: &'static str,
    minimum_rust: &'static str,
    default_features: &'static [&'static str],
    required_direct: &'static [&'static str],
    native_linked: &'static [&'static str],
    lockfile_entries_added: bool,
}

impl ThirdPartySubstrateReview {
    /// The package name on the registry.
    #[must_use]
    pub const fn package(&self) -> &'static str {
        self.package
    }

    /// The exact version reviewed.
    #[must_use]
    pub const fn version(&self) -> &'static str {
        self.version
    }

    /// The upstream repository.
    #[must_use]
    pub const fn upstream(&self) -> &'static str {
        self.upstream
    }

    /// The licence the package declares.
    #[must_use]
    pub const fn licence(&self) -> &'static str {
        self.licence
    }

    /// The minimum Rust version the package declares.
    #[must_use]
    pub const fn minimum_rust(&self) -> &'static str {
        self.minimum_rust
    }

    /// The features the package enables by default.
    #[must_use]
    pub const fn default_features(&self) -> &'static [&'static str] {
        self.default_features
    }

    /// The non-optional direct dependencies the package pulls.
    #[must_use]
    pub const fn required_direct(&self) -> &'static [&'static str] {
        self.required_direct
    }

    /// The packages in its required graph that link native code.
    #[must_use]
    pub const fn native_linked(&self) -> &'static [&'static str] {
        self.native_linked
    }

    /// Whether admitting it would add lock-file entries.
    #[must_use]
    pub const fn lockfile_entries_added(&self) -> bool {
        self.lockfile_entries_added
    }
}

/// The recorded transaction-substrate decision.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransactionSubstrateDecision {
    selection: SubstrateSelection,
    status: DecisionStatus,
    candidate: ThirdPartySubstrateReview,
    grounds: BTreeSet<SubstrateGround>,
    revisit: BTreeSet<SubstrateRevisitTrigger>,
    first_party: BTreeSet<FirstPartyCapability>,
    delegated: BTreeSet<DelegatedCapability>,
}

impl TransactionSubstrateDecision {
    /// What was selected.
    #[must_use]
    pub const fn selection(&self) -> SubstrateSelection {
        self.selection
    }

    /// What has happened to the decision so far.
    #[must_use]
    pub const fn status(&self) -> DecisionStatus {
        self.status
    }

    /// The third-party candidate that was reviewed.
    #[must_use]
    pub const fn candidate(&self) -> &ThirdPartySubstrateReview {
        &self.candidate
    }

    /// Why the candidate was not selected.
    #[must_use]
    pub const fn grounds(&self) -> &BTreeSet<SubstrateGround> {
        &self.grounds
    }

    /// What would reopen the decision.
    #[must_use]
    pub const fn revisit(&self) -> &BTreeSet<SubstrateRevisitTrigger> {
        &self.revisit
    }

    /// What the first-party substrate owns.
    #[must_use]
    pub const fn first_party(&self) -> &BTreeSet<FirstPartyCapability> {
        &self.first_party
    }

    /// What it delegates instead of owning.
    #[must_use]
    pub const fn delegated(&self) -> &BTreeSet<DelegatedCapability> {
        &self.delegated
    }
}

/// The reviewed transaction-substrate decision.
///
/// # What was decided
///
/// The candidate transaction package owns first-party explicit-only
/// transaction structures, and the reviewed third-party library is
/// recorded as not selected. The decision is recorded rather than
/// enacted: no manifest and no lock file changed, and none may until a
/// dependency review with a real consumer ratifies whatever is decided
/// then.
///
/// # Why
///
/// The library is competent and upstream-affiliated, and the review
/// found nothing wrong with it. It was not selected for reasons about
/// this workspace rather than about the library. Its required graph
/// reaches two packages that compile and link vendored native code
/// through build scripts, one of which is a binding to the very library
/// the target vendors in-tree — the same collapse an earlier review
/// already refused for the confidential-value oracle, where agreement
/// between a checker and the thing it checks would be a tautology. The
/// workspace has no native-linked dependency today and every package in
/// it forbids unsafe code, so admitting one is a change to the build
/// story and not only to a manifest. Its licence is a different
/// instrument from the workspace's own. And most of what it supplies —
/// confidential fields, issuance, peg-ins, signing digests — is off the
/// candidate pipeline's path, because the fixed Phase-4 representation
/// is explicit, blinding and signing belong to a sponsor adapter, and
/// the disposable test asset is created through a target funding
/// interface.
///
/// What remains for this workspace to own is therefore narrow: encode
/// and decode explicit-field transactions, assemble a script-path
/// witness from linked constructor data the workspace already computes
/// first-party, and state the canonical role layout.
///
/// # What that costs
///
/// The evidence burden moves rather than disappearing, and moves to a
/// place that already has a harness. A first-party encoder's errors are
/// byte errors, and a byte error fails closed against a real node: the
/// transaction is refused, which is a red result rather than a silent
/// one. The candidate pipeline already plans to put its transactions in
/// front of a real node, so the encoder is checked by the same run that
/// checks everything else. The evidence requirements below name what
/// that run has to establish.
///
/// # What is not claimed
///
/// That the first-party structures exist — they do not yet; that they
/// are correct — nothing has executed; that the decision is final — it
/// awaits ratification, and each recorded trigger names a need that
/// would reopen it.
#[must_use]
pub fn reviewed_substrate_decision() -> TransactionSubstrateDecision {
    TransactionSubstrateDecision {
        selection: SubstrateSelection::FirstParty,
        status: DecisionStatus::AwaitingRatification,
        candidate: ThirdPartySubstrateReview {
            package: "elements",
            version: "0.27.0",
            upstream: "https://github.com/ElementsProject/rust-elements/",
            licence: "CC0-1.0",
            minimum_rust: "1.74.0",
            default_features: &["json-contract"],
            required_direct: &[
                "bech32",
                "bitcoin",
                "bitcoin-consensus-encoding",
                "bitcoin-internals",
                "bitcoin_hashes",
                "hex-conservative",
                "secp256k1-zkp",
            ],
            native_linked: &["secp256k1-sys", "secp256k1-zkp-sys"],
            lockfile_entries_added: true,
        },
        grounds: SubstrateGround::ALL.iter().copied().collect(),
        revisit: SubstrateRevisitTrigger::ALL.iter().copied().collect(),
        first_party: FirstPartyCapability::ALL.iter().copied().collect(),
        delegated: DelegatedCapability::ALL.iter().copied().collect(),
    }
}

/// The evidence a real node must supply about the reviewed forms.
///
/// Each identity below names a claim this module states and does not
/// establish. They are separate from the primitive requirements because
/// answering one needs a complete transaction rather than a stack, and
/// the evidence plan classes them accordingly.
#[must_use]
pub fn transaction_form_evidence() -> BTreeSet<TargetEvidenceRequirementId> {
    [
        TargetEvidenceRequirementId::FeeOutputForm,
        TargetEvidenceRequirementId::ExplicitZeroValueOutputRule,
        TargetEvidenceRequirementId::FeelessTransactionAdmission,
        TargetEvidenceRequirementId::ConfidentialValueConservation,
    ]
    .into_iter()
    .collect()
}
