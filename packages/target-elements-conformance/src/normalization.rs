//! Owner-authorized normalization, and the §10.4 threat matrix.
//!
//! Guide 11 §10 states a candidate: an owner, holding a private value,
//! authorizes its republication in a public representation. §10.1 says
//! what such a transition must preserve — the semantic amount, the
//! explicit asset, and the owner — and §10.4 asks for the mutations a
//! deployment would have to survive. This module states the claim, the
//! mutations, and where each mutation is expected to be refused.
//!
//! # The variant built here is the one that exists
//!
//! Guide 11 §10.2 discusses two shapes. Full consumption of private
//! inputs into an explicit-only output set is *not constructible on this
//! target*: Wave 7 asked the node for one and was told to "add another
//! output to blind", because residual blinding has nowhere to go without
//! a blinded output to absorb it (`G11-W7-03`). The constructible shape
//! is private → explicit **plus private change**, and it is the shape
//! this module claims. The other is not a weaker version of it; it is a
//! transaction the target refuses to help build, and that refusal is a
//! finding rather than a gap.
//!
//! # Why every expectation is written before a node is asked
//!
//! A mutation matrix whose expectations were written after a run would
//! record whatever the target did and call it the design. The rows here
//! carry [`MutationRow::expected`] as data, the executor is never sent
//! it `(´[PLAN-rule:guide11-exec:request-subject]´)`, and a disagreement
//! is reported rather than fitted.
//!
//! # The layer that refuses is the whole content of a row
//!
//! Wave 7's hardest lesson (`G11-W7-06`) was that a refusal attributed to
//! the wrong layer is worse than no evidence: it credits consensus with
//! policing something consensus never looked at. Two of these rows are
//! consensus-valid transactions — the target accepts them, and only the
//! report layer refuses. Recording those as consensus rejections would
//! repeat exactly that mistake, so [`RefusalLayer`] names the report
//! layer as its own value and [`RefusalLayer::is_target_verdict`]
//! separates it from anything the target said.
//!
//! # Everything here is public disposable test data
//!
//! The amounts are fixture constants on a disposable development chain
//! and authorize nothing
//! `(´[ADR015-rule:security:test-material]´)`.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::conservation::{
    CHAIN_POLICY_ASSET, TestAmount, TestAssetRepresentation, TestValueRepresentation,
};

/// The amount the owner's private coin carries.
pub const NORMALIZATION_INPUT: u64 = 10_000_000;

/// The amount republished in the clear.
pub const NORMALIZED_AMOUNT: u64 = 5_000_000;

/// The amount kept private as change.
pub const PRIVATE_CHANGE_AMOUNT: u64 = 5_000_000;

/// How much an amount-changing mutation moves.
///
/// Large enough that no reader mistakes it for a rounding artifact of the
/// node's decimal amount interface.
pub const AMOUNT_MUTATION_DELTA: u64 = 1_000_000;

/// How much a hidden output absorbs.
pub const HIDDEN_OUTPUT_AMOUNT: u64 = 1_000_000;

/// What one output in the claim is for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum OutputRole {
    /// The public representation the normalization creates.
    Normalized,
    /// The private output that absorbs the residual blinding.
    PrivateChange,
}

/// What Guide 11 §10.1 requires a normalization to preserve.
///
/// Stated as a census rather than as prose so that a report naming a
/// violated property is naming one of these rather than describing one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum PreservedProperty {
    /// The semantic amount is the amount the private coin carried.
    SemanticAmount,
    /// The explicit asset is the asset the private coin carried.
    ExplicitAsset,
    /// The owner of the public representation is the owner who
    /// authorized it.
    Owner,
}

impl PreservedProperty {
    /// The complete census of §10.1 properties.
    pub const ALL: &'static [Self] = &[Self::SemanticAmount, Self::ExplicitAsset, Self::Owner];
}

/// One mutation of Guide 11 §10.4.
///
/// # Two of these are the same act at different times
///
/// [`Self::OwnerChanged`] and [`Self::OutputMutatedAfterSigning`] both
/// pay the normalized value to a script the claim does not name. They are
/// separate rows because they are refused by different layers, and the
/// difference between them is the entire reason §10.3 requires a
/// signature over the finalized output set: before signing, paying the
/// wrong owner is a consensus-valid transaction that only a report can
/// refuse; after signing, the same edit is refused by the target itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum NormalizationMutation {
    /// No mutation: the prototype as claimed.
    None,
    /// The normalized output carries an amount the claim does not state,
    /// and the private change absorbs the difference.
    ///
    /// The compensation is deliberate. An uncompensated amount change is
    /// already established as a consensus rejection by conservation row
    /// 6, so repeating it here would spend a row confirming a known
    /// fact. Compensated, the transaction conserves value perfectly and
    /// the target has nothing to refuse — which is the case §10.4
    /// actually needs answered.
    AmountChanged,
    /// The normalized output pays a script the claim does not name.
    OwnerChanged,
    /// The normalized output names an asset the claim does not state.
    AssetChanged,
    /// An input blinding factor is declared that the chain does not
    /// agree with, so the blinding balance does not close.
    WrongBlindingBalance,
    /// An unstated private output absorbs value the claim does not
    /// account for.
    ///
    /// The value comes out of the private change, which is blinded — so
    /// no observer can see the change shrink. Counting outputs is the
    /// only check that catches this, which is why closure is stated over
    /// the output *set* rather than over any amount.
    HiddenPrivateOutput,
    /// An output is appended after the owner signed.
    ///
    /// Funded from the declared fee so that value still conserves: a row
    /// that also unbalanced the transaction would be refused by
    /// conservation, and would establish nothing about the signature.
    ExtraOutputAfterSigning,
    /// An existing output is edited after the owner signed.
    ///
    /// The edit changes the recipient and no amount, for the same reason
    /// the previous row is fee-funded: the refusal has to be the
    /// signature's.
    OutputMutatedAfterSigning,
    /// The representation is changed without the owner authorizing it.
    ///
    /// The owner's witness is removed, leaving a transaction that
    /// republishes their private value with nothing of theirs on it.
    UnauthorizedRepresentationChange,
}

impl NormalizationMutation {
    /// The complete census of §10.4 rows.
    pub const ALL: &'static [Self] = &[
        Self::None,
        Self::AmountChanged,
        Self::OwnerChanged,
        Self::AssetChanged,
        Self::WrongBlindingBalance,
        Self::HiddenPrivateOutput,
        Self::ExtraOutputAfterSigning,
        Self::OutputMutatedAfterSigning,
        Self::UnauthorizedRepresentationChange,
    ];

    /// Whether this mutation is applied after the owner's signature.
    #[must_use]
    pub const fn is_post_signing(&self) -> bool {
        matches!(
            self,
            Self::ExtraOutputAfterSigning
                | Self::OutputMutatedAfterSigning
                | Self::UnauthorizedRepresentationChange
        )
    }
}

/// Which layer refuses a mutated normalization.
///
/// # The report layer is not a target verdict, and says so
///
/// Three values here are things the target did. [`Self::ReportLayer`] is
/// not: it says the target accepted the transaction and the record built
/// from it does not stand. Wave 7 recorded the same distinction for the
/// hidden-output conservation row, and `G11-W7-06` is what happens when
/// it is lost.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum RefusalLayer {
    /// Nothing refuses it. The claim stands.
    NotRefused,
    /// The target refused it before any script ran.
    TargetConsensus,
    /// The target ran the signature check and it failed.
    TargetSignature,
    /// The target would not relay it, though consensus would have it.
    ///
    /// No row expects this, and that is exactly why it exists. A
    /// post-signing row is evidence about the signature only because the
    /// target named a script failure; the same row refused for its fee
    /// would be evidence about relay policy wearing a signature's name.
    /// Folding this into [`Self::TargetConsensus`] would make that
    /// substitution invisible, which is `G11-W7-06`'s mistake in a
    /// smaller costume.
    TargetRelayPolicy,
    /// The target accepted it, and the report layer refuses the claim.
    ReportLayer,
}

impl RefusalLayer {
    /// Whether this refusal is one the target itself made.
    #[must_use]
    pub const fn is_target_verdict(&self) -> bool {
        matches!(
            self,
            Self::TargetConsensus | Self::TargetSignature | Self::TargetRelayPolicy
        )
    }
}

impl std::fmt::Display for RefusalLayer {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::NotRefused => "not refused",
            Self::TargetConsensus => "target consensus",
            Self::TargetSignature => "target signature",
            Self::TargetRelayPolicy => "target relay policy",
            Self::ReportLayer => "report layer",
        };
        formatter.write_str(text)
    }
}

/// The sighash profile Guide 11 §10.3 requires of the authorization.
///
/// Wave 5 reviewed the target's own digest and recorded which modes
/// commit to what, in the reference's output-committing signature
/// profile table. Only the default and
/// all-outputs modes without anyone-can-pay commit every output, every
/// output value commitment, every output script, and every output
/// witness at once, which is what makes a post-signing edit detectable
/// by the target rather than only by a reader.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum AuthorizationProfile {
    /// A taproot key-path signature with no trailing sighash byte.
    ///
    /// The absence of the byte *is* the mode: the reviewed digest reads
    /// a missing byte as the default, all-outputs, non-anyone-can-pay
    /// profile, so a 64-byte signature is the profile rather than merely
    /// consistent with it.
    SighashDefault,
    /// An explicit all-outputs signature without anyone-can-pay.
    SighashAllNoAnyoneCanPay,
}

impl AuthorizationProfile {
    /// Whether this profile commits to every output and output witness.
    ///
    /// Both admitted profiles do; the method exists so that a report
    /// asserting the §10.3 prerequisite asserts it of a value rather
    /// than of a comment.
    #[must_use]
    pub const fn commits_all_outputs(&self) -> bool {
        matches!(self, Self::SighashDefault | Self::SighashAllNoAnyoneCanPay)
    }
}

/// One row of the Guide 11 §10.4 threat matrix.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MutationRow {
    /// The mutation this row applies.
    pub mutation: NormalizationMutation,
    /// Where the row's author expects the refusal to come from.
    pub expected: RefusalLayer,
    /// The §10.1 properties this mutation breaks, where it breaks any.
    pub violates: Vec<PreservedProperty>,
    /// Whether the stated output set stops being the whole output set.
    pub breaks_closure: bool,
    /// Why the expectation is what it is.
    pub reasoning: String,
}

/// The claim one normalization makes.
///
/// # A claim is what the report is checked against
///
/// The transaction says what it says; the claim says what the deployment
/// asserts it did. Every report-layer refusal in this matrix is a
/// disagreement between the two, so the claim has to exist as data
/// before a transaction is built — a claim reconstructed from the
/// transaction afterwards would agree with it by construction.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NormalizationClaim {
    /// The private coin the owner consumes.
    pub consumed: TestAmount,
    /// The public representation created.
    pub normalized: TestAmount,
    /// The private change that absorbs the residual blinding.
    pub change: TestAmount,
}

impl NormalizationClaim {
    /// The claim the canonical prototype makes.
    #[must_use]
    pub fn canonical() -> Self {
        Self {
            consumed: TestAmount {
                value: TestValueRepresentation::confidential(
                    NORMALIZATION_INPUT,
                    "normalization-owner-coin",
                ),
                asset: TestAssetRepresentation::Confidential {
                    asset_id: CHAIN_POLICY_ASSET,
                    asset_blinding_factor: crate::conservation::test_scalar(
                        "normalization-owner-coin",
                        "asset-blinder",
                    ),
                },
            },
            normalized: TestAmount {
                value: TestValueRepresentation::Explicit {
                    amount: NORMALIZED_AMOUNT,
                },
                asset: TestAssetRepresentation::Explicit {
                    asset_id: CHAIN_POLICY_ASSET,
                },
            },
            change: TestAmount {
                value: TestValueRepresentation::confidential(
                    PRIVATE_CHANGE_AMOUNT,
                    "normalization-private-change",
                ),
                asset: TestAssetRepresentation::Confidential {
                    asset_id: CHAIN_POLICY_ASSET,
                    asset_blinding_factor: crate::conservation::test_scalar(
                        "normalization-private-change",
                        "asset-blinder",
                    ),
                },
            },
        }
    }

    /// Whether the claim conserves the value it consumes.
    ///
    /// A claim whose own arithmetic did not close would make every row
    /// downstream of it meaningless, so it is checkable rather than
    /// asserted.
    #[must_use]
    pub const fn conserves(&self) -> bool {
        self.consumed.value.amount() == self.normalized.value.amount() + self.change.value.amount()
    }

    /// The asset the claim is denominated in.
    #[must_use]
    pub const fn asset_id(&self) -> &[u8; 32] {
        self.normalized.asset.asset_id()
    }
}

/// Exactly what an executor is asked to build and judge.
///
/// Carries the claim and the mutation and nothing about what should
/// happen `(´[PLAN-rule:guide11-exec:request-subject]´)`. The expected
/// layer stays with the harness, because a row's whole content is where
/// the refusal came from and an adapter that knew the answer could
/// report it for a transaction that never reached that layer.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NormalizationSubject {
    /// The claim to build.
    pub claim: NormalizationClaim,
    /// The mutation to apply.
    pub mutation: NormalizationMutation,
}

/// One output the claim names, as the adapter resolved it.
///
/// The scripts are not in the fixture: which script an owner holds is a
/// deployment's fact, and the adapter learns it when it creates the
/// coin. What matters is that the adapter states the claim's outputs
/// *from the claim*, before any mutation touches the transaction.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClaimedOutput {
    /// What this output is for.
    pub role: OutputRole,
    /// The script the claim pays.
    pub script_pubkey: Vec<u8>,
    /// The amount, where the claim publishes one.
    pub explicit_amount: Option<u64>,
    /// The asset, where the claim publishes one.
    pub explicit_asset: Option<[u8; 32]>,
}

/// One output the target's own decoder reported.
///
/// # Read back rather than remembered
///
/// The adapter could report the outputs it believes it wrote. It reports
/// what the node's decoder says instead, because the checks below exist
/// to catch a transaction that is not what the claim says — and an
/// observation the harness generated from its own intent could never
/// disagree with that intent.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObservedOutput {
    /// The script the output pays.
    pub script_pubkey: Vec<u8>,
    /// The amount, where the output publishes one.
    pub explicit_amount: Option<u64>,
    /// The asset, where the output publishes one.
    pub explicit_asset: Option<[u8; 32]>,
    /// Whether this output is the declared fee.
    pub is_fee: bool,
}

/// What a closure comparison matches on.
///
/// A blinded output publishes neither amount nor asset, so both are
/// `None` on each side and the script is what identifies it. That is not
/// a weakness of the check: an output whose value nobody can see is
/// exactly the one a claim must account for by *existing*, and counting
/// is what does that.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct OutputFingerprint {
    /// The script paid.
    pub script_pubkey: Vec<u8>,
    /// The published amount, where there is one.
    pub explicit_amount: Option<u64>,
    /// The published asset, where there is one.
    pub explicit_asset: Option<[u8; 32]>,
}

impl OutputFingerprint {
    fn of_claimed(output: &ClaimedOutput) -> Self {
        Self {
            script_pubkey: output.script_pubkey.clone(),
            explicit_amount: output.explicit_amount,
            explicit_asset: output.explicit_asset,
        }
    }

    fn of_observed(output: &ObservedOutput) -> Self {
        Self {
            script_pubkey: output.script_pubkey.clone(),
            explicit_amount: output.explicit_amount,
            explicit_asset: output.explicit_asset,
        }
    }
}

/// Whether the observed output set is exactly the claimed one.
///
/// # Exact multiset, and why nothing weaker will do
///
/// Wave 7 established that consensus accepts a transaction carrying an
/// unstated confidential output that absorbs value (`G11-W7-05`). No
/// target check will ever refuse it, so a candidate claiming that its
/// stated outputs are the whole story has to check that itself.
///
/// A subset test would pass the hidden-output case: every claimed output
/// really is present, and the extra one is simply not looked at. A count
/// test would pass a swap of one output for another. So the comparison
/// is multiset equality in both directions, and the two directions are
/// reported separately because they mean different things — an unclaimed
/// output is value going somewhere the claim does not admit, and a
/// missing one is the claim describing an output that was never created.
///
/// The declared fee is excluded. It is an explicit output the target's
/// own model treats separately, every party can read it, and a claim
/// that had to enumerate it would be restating a public total rather
/// than closing over the value it moved.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "outcome")]
#[non_exhaustive]
pub enum ClosureFinding {
    /// The observed set is exactly the claimed set.
    Holds,
    /// The two sets differ.
    Violated {
        /// Outputs the target carries that the claim does not name.
        unclaimed: Vec<OutputFingerprint>,
        /// Outputs the claim names that the target does not carry.
        missing: Vec<OutputFingerprint>,
    },
}

impl ClosureFinding {
    /// Whether closure holds.
    #[must_use]
    pub const fn holds(&self) -> bool {
        matches!(self, Self::Holds)
    }
}

/// Compares the claimed output set with the observed one, exactly.
///
/// The comparison is a multiset difference in both directions; see
/// [`ClosureFinding`] for why nothing weaker is sufficient.
#[must_use]
pub fn closure_finding(claimed: &[ClaimedOutput], observed: &[ObservedOutput]) -> ClosureFinding {
    let mut counts: BTreeMap<OutputFingerprint, i64> = BTreeMap::new();
    for output in claimed {
        *counts
            .entry(OutputFingerprint::of_claimed(output))
            .or_insert(0) += 1;
    }
    for output in observed.iter().filter(|output| !output.is_fee) {
        *counts
            .entry(OutputFingerprint::of_observed(output))
            .or_insert(0) -= 1;
    }

    let mut unclaimed = Vec::new();
    let mut missing = Vec::new();
    for (fingerprint, balance) in counts {
        match balance.cmp(&0) {
            // The target carries it more often than the claim names it.
            std::cmp::Ordering::Less => {
                for _ in 0..-balance {
                    unclaimed.push(fingerprint.clone());
                }
            }
            // The claim names it more often than the target carries it.
            std::cmp::Ordering::Greater => {
                for _ in 0..balance {
                    missing.push(fingerprint.clone());
                }
            }
            std::cmp::Ordering::Equal => {}
        }
    }

    if unclaimed.is_empty() && missing.is_empty() {
        ClosureFinding::Holds
    } else {
        ClosureFinding::Violated { unclaimed, missing }
    }
}

/// What §10.1 preservation found.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "outcome")]
#[non_exhaustive]
pub enum PreservationFinding {
    /// Every §10.1 property is preserved.
    Holds,
    /// The named properties are not.
    Violated {
        /// Which properties broke.
        properties: Vec<PreservedProperty>,
    },
}

impl PreservationFinding {
    /// Whether preservation holds.
    #[must_use]
    pub const fn holds(&self) -> bool {
        matches!(self, Self::Holds)
    }
}

/// Whether the normalized output preserves what §10.1 requires.
///
/// The claim's normalized output is looked for among the observed
/// outputs by the script the owner holds. Its absence is an owner
/// violation: the value was republished somewhere the owner does not
/// control, which is precisely the property being checked and not a
/// failure to find data.
#[must_use]
pub fn preservation_finding(
    claimed: &[ClaimedOutput],
    observed: &[ObservedOutput],
) -> PreservationFinding {
    let Some(target) = claimed
        .iter()
        .find(|output| output.role == OutputRole::Normalized)
    else {
        return PreservationFinding::Violated {
            properties: PreservedProperty::ALL.to_vec(),
        };
    };

    let paid = observed
        .iter()
        .find(|output| !output.is_fee && output.script_pubkey == target.script_pubkey);

    let Some(paid) = paid else {
        // Nothing pays the owner. The amount and asset the claim states
        // are not carried by any output the owner holds, so all three
        // properties fail together rather than the owner alone.
        return PreservationFinding::Violated {
            properties: PreservedProperty::ALL.to_vec(),
        };
    };

    let mut broken = Vec::new();
    if paid.explicit_amount != target.explicit_amount {
        broken.push(PreservedProperty::SemanticAmount);
    }
    if paid.explicit_asset != target.explicit_asset {
        broken.push(PreservedProperty::ExplicitAsset);
    }

    if broken.is_empty() {
        PreservationFinding::Holds
    } else {
        PreservationFinding::Violated { properties: broken }
    }
}

/// The canonical Guide 11 §10.4 threat matrix.
///
/// # Every expectation below was written before the target was asked
///
/// The rows are the record of a pre-analysis, and two of them predict
/// that the target will *accept* a transaction the claim must refuse.
/// Those two are the wave's substantive prediction: if the target
/// refused them, the report layer this matrix builds would be
/// unnecessary.
#[must_use]
// One literal per row, each carrying its own reasoning. Splitting this to
// satisfy a line count would scatter nine predictions a reader checks
// against one guide section across several functions.
#[allow(clippy::too_many_lines)]
pub fn canonical_mutation_matrix() -> Vec<MutationRow> {
    vec![
        MutationRow {
            mutation: NormalizationMutation::None,
            expected: RefusalLayer::NotRefused,
            violates: Vec::new(),
            breaks_closure: false,
            reasoning: "The prototype as claimed. A private coin is consumed, an \
                 explicit output republishes half of it, and a blinded change \
                 output absorbs the rest and the residual blinding. Conservation \
                 row 4 already established that this shape is accepted; what this \
                 row adds is that the owner authorized it under a profile \
                 committing to every output."
                .to_owned(),
        },
        MutationRow {
            mutation: NormalizationMutation::AmountChanged,
            expected: RefusalLayer::ReportLayer,
            violates: vec![PreservedProperty::SemanticAmount],
            breaks_closure: true,
            reasoning: "The normalized output publishes an amount the claim does \
                 not state, and the private change absorbs the difference so the \
                 transaction still conserves value exactly. Consensus checks that \
                 inputs equal outputs, and they do; there is nothing for the \
                 target to refuse. Only a reader holding the claim can see that \
                 the amount republished is not the amount claimed. The \
                 uncompensated form of this mutation is a consensus rejection and \
                 is already established by conservation row 6, so this row \
                 deliberately takes the case that row could not reach."
                .to_owned(),
        },
        MutationRow {
            mutation: NormalizationMutation::OwnerChanged,
            expected: RefusalLayer::ReportLayer,
            violates: vec![
                PreservedProperty::Owner,
                PreservedProperty::SemanticAmount,
                PreservedProperty::ExplicitAsset,
            ],
            breaks_closure: true,
            reasoning: "The normalized value is paid to a script the claim does \
                 not name. Every amount is right and every commitment balances, so \
                 the transaction is consensus-valid and the target accepts it. A \
                 transaction paying the wrong owner is not a target defect — the \
                 target has no notion of which owner was meant — so the refusal \
                 can only be the report's. All three §10.1 properties are recorded \
                 as broken together, because the owner's output is absent \
                 entirely: no output carries the claimed amount and asset for the \
                 claimed owner."
                .to_owned(),
        },
        MutationRow {
            mutation: NormalizationMutation::AssetChanged,
            expected: RefusalLayer::TargetConsensus,
            violates: vec![PreservedProperty::ExplicitAsset],
            breaks_closure: true,
            reasoning: "The normalized output names an asset the chain never \
                 issued. Unlike the owner and the amount, the asset cannot be \
                 changed while conserving value: the inputs are the policy asset \
                 and no input of the named asset exists, so the tally cannot \
                 close and the target refuses before any script runs. This row is \
                 expected to agree with conservation row 10, and it is stated \
                 anyway because §10.4 asks which layer answers each mutation and \
                 the answer being 'the same one as before' is a result rather \
                 than a duplicate."
                .to_owned(),
        },
        MutationRow {
            mutation: NormalizationMutation::WrongBlindingBalance,
            expected: RefusalLayer::TargetConsensus,
            violates: Vec::new(),
            breaks_closure: false,
            reasoning: "An input blinding factor is declared that the chain does \
                 not agree with. The amounts are all correct and the blinding \
                 scalars do not sum to zero, so the commitments do not balance and \
                 consensus refuses. No §10.1 property is listed as violated: the \
                 claim's amounts, asset, and owner are exactly as stated, and what \
                 fails is the target's own arithmetic rather than the claim."
                .to_owned(),
        },
        MutationRow {
            mutation: NormalizationMutation::HiddenPrivateOutput,
            expected: RefusalLayer::ReportLayer,
            violates: Vec::new(),
            breaks_closure: true,
            reasoning: "An unstated blinded output absorbs value taken from the \
                 private change. Wave 7 established that consensus admits exactly \
                 this (`G11-W7-05`): the commitments balance, so conservation is \
                 satisfied and the target accepts. Because the change is blinded, \
                 no observer can see it shrink — the only evidence is that an \
                 output exists which the claim does not name, which is why the \
                 check is exact multiset closure over the output set. No §10.1 \
                 property is violated: the normalized output is exactly as \
                 claimed, and the claim is still incomplete."
                .to_owned(),
        },
        MutationRow {
            mutation: NormalizationMutation::ExtraOutputAfterSigning,
            expected: RefusalLayer::TargetSignature,
            violates: Vec::new(),
            breaks_closure: true,
            reasoning: "An output is appended after the owner signed, funded from \
                 the declared fee so that value still conserves and the refusal \
                 cannot come from the tally. The reviewed digest commits to every \
                 output and every output witness under the profile §10.3 \
                 requires, so the signature no longer verifies and the target \
                 refuses on the script path. This row and the hidden-output row \
                 are the same act on opposite sides of the signature, and the \
                 layer that answers them is the difference authorization makes."
                .to_owned(),
        },
        MutationRow {
            mutation: NormalizationMutation::OutputMutatedAfterSigning,
            expected: RefusalLayer::TargetSignature,
            violates: Vec::new(),
            breaks_closure: true,
            reasoning: "The normalized output's recipient is edited after the \
                 owner signed, with no amount touched so that conservation still \
                 closes. This is the owner-changed mutation applied after \
                 authorization instead of before, and the expectation is the \
                 opposite one: the digest covers the output script, so the target \
                 refuses what it accepted when the same edit preceded the \
                 signature."
                .to_owned(),
        },
        MutationRow {
            mutation: NormalizationMutation::UnauthorizedRepresentationChange,
            expected: RefusalLayer::TargetSignature,
            violates: Vec::new(),
            breaks_closure: false,
            reasoning: "The owner's witness is removed, so a transaction \
                 republishing their private value carries nothing of theirs. The \
                 spent output is a key-path taproot program, and a witness that \
                 offers no signature fails script verification. This is the \
                 baseline §10 rests on: without it, 'owner-authorized' would be a \
                 description of intent rather than a property the target enforces."
                .to_owned(),
        },
    ]
}

/// Every mutation the canonical matrix states.
#[must_use]
pub fn matrix_mutations(rows: &[MutationRow]) -> Vec<NormalizationMutation> {
    rows.iter().map(|row| row.mutation).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conservation::FOREIGN_TEST_ASSET;

    #[test]
    fn the_matrix_states_every_mutation_exactly_once() {
        let rows = canonical_mutation_matrix();
        let stated = matrix_mutations(&rows);
        assert_eq!(stated.len(), NormalizationMutation::ALL.len());
        for mutation in NormalizationMutation::ALL {
            assert_eq!(
                stated.iter().filter(|entry| *entry == mutation).count(),
                1,
                "{mutation:?}"
            );
        }
    }

    #[test]
    fn the_unmutated_row_is_the_only_one_not_refused() {
        let refused: Vec<_> = canonical_mutation_matrix()
            .into_iter()
            .filter(|row| row.expected == RefusalLayer::NotRefused)
            .map(|row| row.mutation)
            .collect();
        assert_eq!(refused, vec![NormalizationMutation::None]);
    }

    #[test]
    fn the_report_layer_rows_are_the_waves_substantive_prediction() {
        // Three rows predict that the target ACCEPTS a transaction the
        // claim must refuse. If this set ever emptied, the report layer
        // this matrix builds would have nothing to do, and the change
        // would be a claim about the target rather than a tidy-up.
        let report_rows: Vec<_> = canonical_mutation_matrix()
            .into_iter()
            .filter(|row| row.expected == RefusalLayer::ReportLayer)
            .map(|row| row.mutation)
            .collect();
        assert_eq!(
            report_rows,
            vec![
                NormalizationMutation::AmountChanged,
                NormalizationMutation::OwnerChanged,
                NormalizationMutation::HiddenPrivateOutput,
            ]
        );
    }

    #[test]
    fn every_post_signing_mutation_expects_the_signature_to_refuse() {
        // The point of the §10.3 profile. A post-signing edit that some
        // other layer refused would leave the signature's coverage
        // unestablished, so the expectation is stated for all three.
        for row in canonical_mutation_matrix() {
            if row.mutation.is_post_signing() {
                assert_eq!(row.expected, RefusalLayer::TargetSignature, "{row:?}");
            }
        }
    }

    #[test]
    fn the_report_layer_is_never_a_target_verdict() {
        assert!(!RefusalLayer::ReportLayer.is_target_verdict());
        assert!(!RefusalLayer::NotRefused.is_target_verdict());
        assert!(RefusalLayer::TargetConsensus.is_target_verdict());
        assert!(RefusalLayer::TargetSignature.is_target_verdict());
        assert!(RefusalLayer::TargetRelayPolicy.is_target_verdict());
    }

    #[test]
    fn every_row_says_why() {
        for row in canonical_mutation_matrix() {
            assert!(!row.reasoning.is_empty(), "{:?}", row.mutation);
        }
    }

    #[test]
    fn the_canonical_claim_conserves_what_it_consumes() {
        let claim = NormalizationClaim::canonical();
        assert!(claim.conserves());
        assert!(claim.consumed.value.is_confidential());
        assert!(!claim.normalized.value.is_confidential());
        assert!(claim.change.value.is_confidential());
    }

    fn claimed(role: OutputRole, script: u8, amount: Option<u64>) -> ClaimedOutput {
        ClaimedOutput {
            role,
            script_pubkey: vec![script; 4],
            explicit_amount: amount,
            explicit_asset: amount.map(|_| CHAIN_POLICY_ASSET),
        }
    }

    fn observed(script: u8, amount: Option<u64>, is_fee: bool) -> ObservedOutput {
        ObservedOutput {
            script_pubkey: vec![script; 4],
            explicit_amount: amount,
            explicit_asset: amount.map(|_| CHAIN_POLICY_ASSET),
            is_fee,
        }
    }

    #[test]
    fn closure_holds_when_the_sets_match_and_ignores_the_fee() {
        let claim = vec![
            claimed(OutputRole::Normalized, 0xaa, Some(5)),
            claimed(OutputRole::PrivateChange, 0xbb, None),
        ];
        let seen = vec![
            observed(0xaa, Some(5), false),
            observed(0xbb, None, false),
            observed(0x00, Some(1), true),
        ];
        assert!(closure_finding(&claim, &seen).holds());
    }

    #[test]
    fn a_hidden_output_is_reported_as_unclaimed() {
        // The row consensus will never refuse. Nothing about the claimed
        // outputs changed, so only the extra one distinguishes this
        // transaction from an honest one.
        let claim = vec![
            claimed(OutputRole::Normalized, 0xaa, Some(5)),
            claimed(OutputRole::PrivateChange, 0xbb, None),
        ];
        let seen = vec![
            observed(0xaa, Some(5), false),
            observed(0xbb, None, false),
            observed(0xcc, None, false),
        ];
        let ClosureFinding::Violated { unclaimed, missing } = closure_finding(&claim, &seen) else {
            panic!("a hidden output violates closure");
        };
        assert_eq!(unclaimed.len(), 1);
        assert_eq!(unclaimed[0].script_pubkey, vec![0xcc; 4]);
        assert!(missing.is_empty());
    }

    #[test]
    fn a_subset_test_would_have_passed_the_hidden_output() {
        // Guards the reasoning rather than the code: every claimed output
        // really is present, so any check that only asked "are the
        // claimed outputs there?" would report closure holding.
        let claim = vec![claimed(OutputRole::Normalized, 0xaa, Some(5))];
        let seen = vec![observed(0xaa, Some(5), false), observed(0xcc, None, false)];
        let present = claim.iter().all(|wanted| {
            seen.iter()
                .any(|entry| entry.script_pubkey == wanted.script_pubkey)
        });
        assert!(present);
        assert!(!closure_finding(&claim, &seen).holds());
    }

    #[test]
    fn a_changed_amount_is_both_missing_and_unclaimed() {
        let claim = vec![claimed(OutputRole::Normalized, 0xaa, Some(5))];
        let seen = vec![observed(0xaa, Some(6), false)];
        let ClosureFinding::Violated { unclaimed, missing } = closure_finding(&claim, &seen) else {
            panic!("a changed amount violates closure");
        };
        assert_eq!(unclaimed[0].explicit_amount, Some(6));
        assert_eq!(missing[0].explicit_amount, Some(5));
    }

    #[test]
    fn preservation_holds_for_the_claimed_output() {
        let claim = vec![
            claimed(OutputRole::Normalized, 0xaa, Some(5)),
            claimed(OutputRole::PrivateChange, 0xbb, None),
        ];
        let seen = vec![observed(0xaa, Some(5), false), observed(0xbb, None, false)];
        assert!(preservation_finding(&claim, &seen).holds());
    }

    #[test]
    fn a_changed_amount_breaks_the_semantic_amount() {
        let claim = vec![claimed(OutputRole::Normalized, 0xaa, Some(5))];
        let seen = vec![observed(0xaa, Some(6), false)];
        let PreservationFinding::Violated { properties } = preservation_finding(&claim, &seen)
        else {
            panic!("a changed amount breaks preservation");
        };
        assert_eq!(properties, vec![PreservedProperty::SemanticAmount]);
    }

    #[test]
    fn a_changed_owner_breaks_all_three_properties() {
        // No output pays the owner at all, so the amount and asset the
        // claim states are carried by nothing the owner holds. Reporting
        // only the owner would understate what went wrong.
        let claim = vec![claimed(OutputRole::Normalized, 0xaa, Some(5))];
        let seen = vec![observed(0xcc, Some(5), false)];
        let PreservationFinding::Violated { properties } = preservation_finding(&claim, &seen)
        else {
            panic!("a changed owner breaks preservation");
        };
        assert_eq!(properties, PreservedProperty::ALL.to_vec());
    }

    #[test]
    fn both_admitted_profiles_commit_every_output() {
        assert!(AuthorizationProfile::SighashDefault.commits_all_outputs());
        assert!(AuthorizationProfile::SighashAllNoAnyoneCanPay.commits_all_outputs());
    }

    #[test]
    fn the_foreign_asset_is_not_the_policy_asset() {
        // The asset-changed row depends on it, and a constant that drifted
        // into equality would turn that row into a benign one.
        assert_ne!(FOREIGN_TEST_ASSET, CHAIN_POLICY_ASSET);
    }
}
