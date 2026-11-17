//! Exact ordered keyed materialization and declared-envelope admission.
//!
//! No receipt offered by a caller is accepted here: inputs come only from sealed
//! SOURCE/MERGE requests. Ordinary authorization remains outstanding on structural
//! artifacts; the admitted entry verifies exactly one P2WPKH sponsor.
use crate::keyed_fixed_inputs::{BoundKeyedFixedInput, KeyedFixedInputRole};
use crate::keyed_fixed_request::{KeyedFixedMergeRequest, KeyedFixedSourceRequest};
use crate::keyed_fixed_sponsor::KeyedFixedSponsorRefusal;
use crate::keyed_fixed_taproot::KeyedFixedCommitment;
use crate::sponsor::{SighashProfile, SignerRole, SponsorSigningRequest};
use crate::trace_support::TraceRefusal;
use crate::{
    AssetField, AssetId, InputWitness, NonceField, TargetInput, TargetOutput, TargetTransaction,
    TransactionRefusal, ValueField,
};
use std::collections::BTreeSet;
use target_elements::ReviewedElementsTapscriptDefinition;

/// Derive the admitted member count from the target's policy weight bound.
///
/// The stripped shape is one sponsor plus `n` members, three outputs, and a
/// `CompactSize` input count. Its nonmember witness allowance is the original
/// P2WPKH sponsor's 72-byte signature and 33-byte compressed key (117 bytes
/// including framing and empty proof vectors). Each fixed-form member adds
/// 41 stripped bytes and the S5-owned 143-byte witness.
/// # Errors
/// Refuses an absent policy ceiling or arithmetic overflow.
pub fn keyed_fixed_policy_member_maximum(
    target: &ReviewedElementsTapscriptDefinition,
) -> Result<usize, KeyedFixedConstructionError> {
    let ceiling = target
        .definition()
        .resources()
        .policy()
        .bounds()
        .get(&target_elements::ResourceDimension::TransactionWeight)
        .and_then(|bound| bound.maximum())
        .ok_or(KeyedFixedConstructionError::ResourceContext)?;
    let mut count = 0_usize;
    loop {
        let next = count
            .checked_add(1)
            .ok_or(KeyedFixedConstructionError::Arithmetic)?;
        let stripped = 239_u64
            .checked_add(crate::compact_size((next + 1) as u64).len() as u64)
            .and_then(|size| size.checked_add(41_u64.checked_mul(next as u64)?))
            .ok_or(KeyedFixedConstructionError::Arithmetic)?;
        let weight = tapscript::keyed_fixed_resources::transaction_weight(
            target,
            next as u64,
            stripped,
            117,
        )
        .map_err(|_| KeyedFixedConstructionError::Arithmetic)?;
        if weight > ceiling {
            return Ok(count);
        }
        count = next;
    }
}

/// Only checked requests are construction authority.
#[derive(Clone, Copy, Debug)]
pub enum KeyedFixedConstructionRequest<'request, 'capability> {
    /// SOURCE with no consumed member of this key.
    Source(&'request KeyedFixedSourceRequest<'capability>),
    /// MERGE with an authenticated nonempty member census.
    Merge(&'request KeyedFixedMergeRequest<'capability>),
}
/// Construction choices, not new covenant laws. Additional outputs retain order.
///
/// No script, control, input list or stated input total can be supplied here.
/// ```compile_fail,E0560
/// use transaction::{AssetId, keyed_fixed_construct::KeyedFixedOutputOffer};
/// let _ = KeyedFixedOutputOffer { policy_asset: AssetId::from_internal([3;32]),
///     successor_value: 1, additional_outputs: vec![], fee: Some(1), input_total: 2 };
/// ```
/// ```compile_fail,E0560
/// use transaction::{AssetId, keyed_fixed_construct::KeyedFixedOutputOffer};
/// let _ = KeyedFixedOutputOffer { policy_asset: AssetId::from_internal([3;32]),
///     successor_value: 1, additional_outputs: vec![], fee: Some(1), covenant_script: vec![0x51] };
/// ```
#[derive(Clone, Debug)]
pub struct KeyedFixedOutputOffer {
    /// Construction asset; admission independently checks the policy reading.
    pub policy_asset: AssetId,
    /// Positive explicit amount at the sealed successor output zero.
    pub successor_value: u64,
    /// Additional outputs, retained in this exact order before the fee.
    pub additional_outputs: Vec<TargetOutput>,
    /// Positive explicit final fee; absent or zero refuses.
    pub fee: Option<u64>,
}
/// Layered construction or policy failure, never covenant execution refusal.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum KeyedFixedConstructionError {
    /// Existing target consensus/encoding refusal, preserved intact.
    Target(TransactionRefusal),
    /// Construction needs at least one funding input.
    EmptySource,
    /// Internal: materialized input order differs from the sealed census.
    InputCensus,
    /// Construction fields must be explicit, null-nonce and in the offered asset.
    ExplicitPolicyAsset,
    /// Offered triple asset differs from the construction asset, or that asset differs from the policy reading.
    PolicyAssetMismatch,
    /// Consensus stated amount or total exceeds the target range.
    AmountRange,
    /// Construction requires positive explicit outputs.
    NonPositiveOutput,
    /// Construction fee is absent or zero.
    MissingFee,
    /// An additional output attempts to supply another fee.
    ExtraFee,
    /// Decoded input total differs from the constructed output total.
    Balance,
    /// Checked size or amount arithmetic overflowed.
    Arithmetic,
    /// Output zero differs from the sealed commitment.
    OutputZero,
    /// Internal: a member witness differs from the sealed leaf/control.
    CovenantWitness,
    /// Capability group outpoints differ from its offered input set.
    CapabilityCoverage,
    /// A collateral capability names a keyed member.
    CapabilityOverlap,
    /// Rows sharing a capability have absent or unequal retained snapshots.
    CapabilitySnapshot,
    /// Sum of group fee offers differs from the construction fee.
    CapabilityFee,
    /// Input/output shape is outside the declared envelope.
    Envelope,
    /// Measured sponsor has no retained spending capability.
    SponsorCapabilityMissing,
    /// Retained change amount differs from output one.
    ChangeOfferMismatch,
    /// Sponsor supplies no change destination.
    ChangeDestinationMissing,
    /// Sponsor change destination differs from output one.
    ChangeDestinationMismatch,
    /// Authenticated handle census exceeds the declared policy maximum.
    HandleCountExceedsPolicy {
        /// Authenticated handle count offered to the declared policy envelope.
        offered: usize,
        /// Greatest admissible count.
        maximum: usize,
    },
    /// Fee is below the supplied policy rates for the witness-bounded size.
    FeeShortage {
        /// Offered explicit fee in policy-asset satoshis.
        offered: u64,
        /// Required fee from supplied rates.
        required: u64,
    },
    /// Output is below the supplied minimum for its exact form.
    OutputPolicy {
        /// Zero-based output index.
        index: usize,
    },
    /// Policy signature bound is outside 1 through 72 bytes.
    SignatureBoundOutOfRange,
    /// Signer returned a signature exceeding its declared byte bound.
    SignatureBoundExceeded {
        /// Measured or returned value.
        reached: usize,
        /// Inclusive admissible bound.
        bound: usize,
    },
    /// Internal: final non-witness bytes differ from the frozen bytes.
    FrozenBytesChanged,
    /// Internal: independently counted witness contributions disagree.
    WeightFormulaMismatch {
        /// Weight from complete serialization.
        serialized: u64,
        /// Weight from independent contribution counts.
        formula: u64,
    },
    /// Supplied transaction policy weight maximum exceeded.
    TransactionWeightExceeded {
        /// Measured or returned value.
        reached: u64,
        /// Inclusive admissible bound.
        bound: u64,
    },
    /// Supplied deployment weight maximum exceeded.
    DeploymentWeightExceeded {
        /// Measured or returned value.
        reached: u64,
        /// Inclusive admissible bound.
        bound: u64,
    },
    /// Supplied package weight maximum exceeded.
    PackageWeightExceeded {
        /// Measured or returned value.
        reached: u64,
        /// Inclusive admissible bound.
        bound: u64,
    },
    /// Internal: final serialized weight exceeds the verified witness bound.
    FinalWeightExceeded {
        /// Measured or returned value.
        reached: u64,
        /// Inclusive admissible bound.
        bound: u64,
    },
    /// Defensive: actual input index cannot fit the signing interface.
    SigningIndex,
    /// Sponsor declined the signing request.
    SignatureMissing,
    /// Signer echoed bytes other than the frozen transaction.
    SignatureBinding,
    /// Sponsor witness or digest refusal, preserved intact.
    Sponsor(KeyedFixedSponsorRefusal),
    /// Policy provenance, output-floor census or positive ceilings are missing.
    ResourceContext,
}
impl KeyedFixedConstructionError {
    /// Static tracing class, without amounts or keys.
    #[must_use]
    pub fn refusal_kind(&self) -> &'static str {
        self.tracing_kind()
    }
}
impl crate::trace_support::TraceRefusal for KeyedFixedConstructionError {
    fn tracing_kind(&self) -> &'static str {
        match self {
            Self::Target(_) => "target",
            Self::EmptySource => "empty_source",
            Self::InputCensus => "input_census",
            Self::ExplicitPolicyAsset => "explicit_policy_asset",
            Self::PolicyAssetMismatch => "policy_asset_mismatch",
            Self::AmountRange => "amount_range",
            Self::NonPositiveOutput => "non_positive_output",
            Self::MissingFee => "missing_fee",
            Self::ExtraFee => "extra_fee",
            Self::Balance => "balance",
            Self::Arithmetic => "arithmetic",
            Self::OutputZero => "output_zero",
            Self::CovenantWitness => "covenant_witness",
            Self::CapabilityCoverage => "capability_coverage",
            Self::CapabilityOverlap => "capability_overlap",
            Self::CapabilitySnapshot => "capability_snapshot",
            Self::CapabilityFee => "capability_fee",
            Self::Envelope => "envelope",
            Self::SponsorCapabilityMissing => "sponsor_capability_missing",
            Self::ChangeOfferMismatch => "change_offer_mismatch",
            Self::ChangeDestinationMissing => "change_destination_missing",
            Self::ChangeDestinationMismatch => "change_destination_mismatch",
            Self::HandleCountExceedsPolicy { .. } => "handle_count_exceeds_policy",
            Self::FeeShortage { .. } => "fee_shortage",
            Self::OutputPolicy { .. } => "output_policy",
            Self::SignatureBoundOutOfRange => "signature_bound_out_of_range",
            Self::SignatureBoundExceeded { .. } => "signature_bound_exceeded",
            Self::FrozenBytesChanged => "frozen_bytes_changed",
            Self::WeightFormulaMismatch { .. } => "weight_formula_mismatch",
            Self::TransactionWeightExceeded { .. } => "transaction_weight_exceeded",
            Self::DeploymentWeightExceeded { .. } => "deployment_weight_exceeded",
            Self::PackageWeightExceeded { .. } => "package_weight_exceeded",
            Self::FinalWeightExceeded { .. } => "final_weight_exceeded",
            Self::SigningIndex => "signing_index",
            Self::SignatureMissing => "signature_missing",
            Self::SignatureBinding => "signature_binding",
            Self::Sponsor(_) => "sponsor",
            Self::ResourceContext => "resource_context",
        }
    }
}
/// Structural material is not a policy-admitted or authorized transaction.
/// ```compile_fail,E0451
/// use transaction::keyed_fixed_construct::KeyedFixedStructuralTransaction;
/// fn forge() -> KeyedFixedStructuralTransaction<'static, 'static> {
///     KeyedFixedStructuralTransaction { transaction: panic!(), inputs: &[], commitment: panic!(), fee: 0, member_count: 0 }
/// }
/// ```
#[derive(Debug)]
pub struct KeyedFixedStructuralTransaction<'request, 'capability> {
    transaction: TargetTransaction,
    inputs: &'request [BoundKeyedFixedInput<'capability>],
    commitment: &'request KeyedFixedCommitment,
    fee: u64,
    member_count: usize,
}
impl KeyedFixedStructuralTransaction<'_, '_> {
    #[must_use]
    pub const fn transaction(&self) -> &TargetTransaction {
        &self.transaction
    }
    #[must_use]
    pub const fn member_count(&self) -> usize {
        self.member_count
    }
    /// External and local obligations remain outstanding on unsigned structure.
    #[must_use]
    pub const fn obligations(&self) -> &[crate::keyed_fixed_abi::KeyedFixedRequestObligation] {
        &crate::keyed_fixed_abi::KeyedFixedRequestObligation::ALL
    }
    /// Explicitly unadmitted; all ordinary signatures and policy checks remain.
    #[must_use]
    pub const fn policy_admitted(&self) -> bool {
        false
    }
}
/// Caller-supplied policy readings with their provenance; no historic universal floor.
///
/// Each output floor is an explicit reading for that exact output form. Package
/// and deployment bounds are supplied, never inferred as zero from missing data.
#[derive(Clone, Debug)]
pub struct KeyedFixedPolicy {
    provenance: String,
    policy_asset: AssetId,
    relay_sat_per_kvb: u64,
    mempool_sat_per_kvb: u64,
    block_sat_per_kvb: u64,
    dust_relay_sat_per_kvb: u64,
    require_standard: bool,
    output_minimums: Vec<u64>,
    maximum_transaction_weight: u64,
    package_weight_before: u64,
    maximum_package_weight: u64,
    maximum_deployment_weight: u64,
    sponsor_signature_bound: usize,
}
impl KeyedFixedPolicy {
    /// Check caller readings. Rates are relay/mempool/block/dust sat/kvB
    /// with the supplied standardness switch;
    /// `ceilings` are transaction/package/deployment weights, inclusive.
    /// Output minimums are ordered form-specific readings. Package weight
    /// already present is explicit; counts and ancestor topology are not modelled.
    /// # Errors
    /// Refuses empty provenance/minimums, zero ceilings or a signature bound
    /// outside 1..=72. No reading is authenticated by this constructor.
    pub fn new(
        provenance: String,
        policy_asset: AssetId,
        rates_and_standardness: ([u64; 4], bool),
        output_minimums: Vec<u64>,
        ceilings: [u64; 3],
        package_weight_before: u64,
        sponsor_signature_bound: usize,
    ) -> Result<Self, KeyedFixedConstructionError> {
        if provenance.is_empty() || output_minimums.is_empty() || ceilings.contains(&0) {
            return Err(KeyedFixedConstructionError::ResourceContext);
        }
        if !(1..=72).contains(&sponsor_signature_bound) {
            return Err(KeyedFixedConstructionError::SignatureBoundOutOfRange);
        }
        Ok(Self {
            provenance,
            policy_asset,
            relay_sat_per_kvb: rates_and_standardness.0[0],
            mempool_sat_per_kvb: rates_and_standardness.0[1],
            block_sat_per_kvb: rates_and_standardness.0[2],
            dust_relay_sat_per_kvb: rates_and_standardness.0[3],
            require_standard: rates_and_standardness.1,
            output_minimums,
            maximum_transaction_weight: ceilings[0],
            maximum_package_weight: ceilings[1],
            maximum_deployment_weight: ceilings[2],
            package_weight_before,
            sponsor_signature_bound,
        })
    }
    /// The network policy asset named by these readings.
    #[must_use]
    pub const fn policy_asset(&self) -> AssetId {
        self.policy_asset
    }
    /// Unauthenticated source of these readings.
    #[must_use]
    pub fn provenance(&self) -> &str {
        &self.provenance
    }
}
/// Derive the source-qualified dust floor for one exact output form.
///
/// The target policy's virtual spend allowance is `32+4+1+floor(107/4)+4=67`
/// bytes. A non-policy asset or disabled standardness has only positivity.
/// # Errors
/// Refuses arithmetic overflow in serialized size or rate multiplication.
pub fn keyed_fixed_policy_floor(
    output: &TargetOutput,
    policy_asset: AssetId,
    dust_relay_sat_per_kvb: u64,
    require_standard: bool,
) -> Result<u64, KeyedFixedConstructionError> {
    if !require_standard || output.is_fee() || output.asset() != AssetField::Explicit(policy_asset)
    {
        return Ok(1);
    }
    let mut bytes = Vec::new();
    output.encode(&mut bytes);
    let virtual_size = u64::try_from(bytes.len())
        .map_err(|_| KeyedFixedConstructionError::Arithmetic)?
        .checked_add(67)
        .ok_or(KeyedFixedConstructionError::Arithmetic)?;
    let fee = virtual_size
        .checked_mul(dust_relay_sat_per_kvb)
        .and_then(|value| value.checked_add(999))
        .ok_or(KeyedFixedConstructionError::Arithmetic)?
        / 1000;
    Ok(fee.max(1))
}
/// Signed, envelope-admitted bytes; chain and covenant execution remain obligations.
/// ```compile_fail,E0451
/// use transaction::keyed_fixed_construct::KeyedFixedAdmittedTransaction;
/// fn forge() -> KeyedFixedAdmittedTransaction { KeyedFixedAdmittedTransaction { transaction: panic!(), weight_bound: 0, policy_provenance: String::new() } }
/// ```
#[derive(Clone, Debug)]
pub struct KeyedFixedAdmittedTransaction {
    transaction: TargetTransaction,
    weight_bound: u64,
    policy_provenance: String,
}
impl KeyedFixedAdmittedTransaction {
    #[must_use]
    pub const fn transaction(&self) -> &TargetTransaction {
        &self.transaction
    }
    #[must_use]
    pub const fn weight_bound(&self) -> u64 {
        self.weight_bound
    }
    /// External parent facts remain unresolved; covenant execution is not claimed.
    #[must_use]
    pub const fn obligations(&self) -> &[crate::keyed_fixed_abi::KeyedFixedRequestObligation] {
        use crate::keyed_fixed_abi::KeyedFixedRequestObligation as O;
        &[
            O::Inclusion,
            O::Unspentness,
            O::Retrieval,
            O::Finality,
            O::FinalBytePolicyObservation,
            O::MemberExecutionAndRelationEvidence,
        ]
    }
    #[must_use]
    pub fn policy_provenance(&self) -> &str {
        &self.policy_provenance
    }
}
/// Materialize a checked census without signing or claiming policy admission.
/// # Errors
/// Refuses unsupported fields, imbalance, inconsistent capability snapshots or census.
pub fn construct_keyed_fixed<'r, 'c>(
    target: &ReviewedElementsTapscriptDefinition,
    request: KeyedFixedConstructionRequest<'r, 'c>,
    offer: KeyedFixedOutputOffer,
) -> Result<KeyedFixedStructuralTransaction<'r, 'c>, KeyedFixedConstructionError> {
    let _span = tracing::debug_span!(target: "transaction::keyed_fixed_construct", "construct_keyed", family = "keyed").entered();
    let result = construct_core(target, request, offer, false);
    terminal(&result, "construct");
    result
}
fn terminal<T>(result: &Result<T, KeyedFixedConstructionError>, boundary: &'static str) {
    match result {
        Ok(_) => {
            tracing::debug!(target: "transaction::keyed_fixed_construct", boundary, family = "keyed", outcome = "completed", "keyed construction completed");
        }
        Err(e) => {
            tracing::debug!(target: "transaction::keyed_fixed_construct", boundary, family = "keyed", outcome = "refused", refusal_kind = e.refusal_kind(), "keyed construction refused");
        }
    }
}
fn amount(o: &TargetOutput, asset: AssetId) -> Result<u64, KeyedFixedConstructionError> {
    match (o.asset(), o.value(), o.nonce()) {
        (AssetField::Explicit(a), ValueField::Explicit(n), NonceField::Null) if a == asset => {
            if !target_elements::transaction_form::reviewed_stated_amount_bound().admits(n) {
                return Err(KeyedFixedConstructionError::AmountRange);
            }
            Ok(n)
        }
        _ => Err(KeyedFixedConstructionError::ExplicitPolicyAsset),
    }
}
#[allow(clippy::too_many_lines)] // Keep ordered pre-byte input, amount, output and witness checks in one boundary.
fn construct_core<'r, 'c>(
    target: &ReviewedElementsTapscriptDefinition,
    request: KeyedFixedConstructionRequest<'r, 'c>,
    offer: KeyedFixedOutputOffer,
    enforce_envelope: bool,
) -> Result<KeyedFixedStructuralTransaction<'r, 'c>, KeyedFixedConstructionError> {
    let (inputs, commitment) = match request {
        KeyedFixedConstructionRequest::Source(r) => (r.inputs(), r.commitment()),
        KeyedFixedConstructionRequest::Merge(r) => (r.inputs(), r.commitment()),
    };
    if inputs.is_empty() {
        return Err(KeyedFixedConstructionError::EmptySource);
    }
    let fee = offer
        .fee
        .filter(|n| *n > 0)
        .ok_or(KeyedFixedConstructionError::MissingFee)?;
    let member_count = inputs
        .iter()
        .filter(|i| i.role() == KeyedFixedInputRole::Member)
        .count();
    // Counts are read only after the request authenticated/classified every input.
    // The declared envelope's bound is applied before building or serializing a tx.
    let declared_shape = inputs.len() == member_count + 1
        && offer.additional_outputs.len() == 1
        && inputs
            .iter()
            .filter(|i| i.role() == KeyedFixedInputRole::Collateral)
            .all(|i| is_p2wpkh(i.parent().output().program()))
        && offer
            .additional_outputs
            .iter()
            .all(|o| is_p2wpkh(o.program()));
    if enforce_envelope && !declared_shape {
        return Err(KeyedFixedConstructionError::Envelope);
    }
    let maximum = keyed_fixed_policy_member_maximum(target)?;
    if member_count > maximum {
        return Err(KeyedFixedConstructionError::HandleCountExceedsPolicy {
            offered: member_count,
            maximum,
        });
    }
    let parameters = commitment.recipe().parameters();
    if offer.policy_asset != AssetId::from_internal(parameters.asset) {
        return Err(KeyedFixedConstructionError::PolicyAssetMismatch);
    }
    if offer.successor_value != parameters.value.get() {
        return Err(KeyedFixedConstructionError::OutputZero);
    }
    reconcile(inputs, fee)?;
    let mut total = 0_u64;
    let mut built = Vec::new();
    let mut witnesses = Vec::new();
    for (index, input) in inputs.iter().enumerate() {
        if usize::try_from(input.input_index()).ok() != Some(index) {
            return Err(KeyedFixedConstructionError::InputCensus);
        }
        // Flags are zero by the Outpoint domain and TargetInput's field set.
        // Step-3 requests reject every nonzero offer; digest decoding pins this again.
        total = total
            .checked_add(amount(input.parent().output(), offer.policy_asset)?)
            .ok_or(KeyedFixedConstructionError::Arithmetic)?;
        built.push(TargetInput::new(
            input.parent().outpoint(),
            input.sequence(),
        ));
        witnesses.push(InputWitness::new(
            if input.role() == KeyedFixedInputRole::Member {
                vec![commitment.leaf().to_vec(), commitment.control().to_vec()]
            } else {
                vec![]
            },
        ));
    }
    if !target_elements::transaction_form::reviewed_stated_amount_bound().admits(total) {
        return Err(KeyedFixedConstructionError::AmountRange);
    }
    let mut outputs = vec![TargetOutput::new(
        AssetField::Explicit(offer.policy_asset),
        ValueField::Explicit(offer.successor_value),
        NonceField::Null,
        commitment.output_program().to_vec(),
    )];
    if offer.additional_outputs.iter().any(TargetOutput::is_fee) {
        return Err(KeyedFixedConstructionError::ExtraFee);
    }
    outputs.extend(offer.additional_outputs);
    outputs.push(TargetOutput::new(
        AssetField::Explicit(offer.policy_asset),
        ValueField::Explicit(fee),
        NonceField::Null,
        vec![],
    ));
    let sum = output_total(&outputs, offer.policy_asset)?;
    if sum != total {
        return Err(KeyedFixedConstructionError::Balance);
    }
    let transaction = TargetTransaction::new(2, built, outputs, 0, witnesses)
        .map_err(KeyedFixedConstructionError::Target)?;
    crate::construct::check_weight(target, &transaction)
        .map_err(KeyedFixedConstructionError::Target)?;
    let result = KeyedFixedStructuralTransaction {
        transaction,
        inputs,
        commitment,
        fee,
        member_count,
    };
    check_material(&result)?;
    Ok(result)
}
fn output_total(
    outputs: &[TargetOutput],
    asset: AssetId,
) -> Result<u64, KeyedFixedConstructionError> {
    let mut sum = 0_u64;
    for o in outputs {
        let n = amount(o, asset)?;
        if n == 0 {
            return Err(KeyedFixedConstructionError::NonPositiveOutput);
        }
        sum = sum
            .checked_add(n)
            .ok_or(KeyedFixedConstructionError::Arithmetic)?;
    }
    if !target_elements::transaction_form::reviewed_stated_amount_bound().admits(sum) {
        return Err(KeyedFixedConstructionError::AmountRange);
    }
    Ok(sum)
}

fn is_p2wpkh(p: &[u8]) -> bool {
    p.len() == 2 + crate::sponsor::WITNESS_V0_KEYHASH_PROGRAM_BYTES
        && p[0] == 0
        && usize::from(p[1]) == crate::sponsor::WITNESS_V0_KEYHASH_PROGRAM_BYTES
}
fn reconcile(
    inputs: &[BoundKeyedFixedInput<'_>],
    fee: u64,
) -> Result<(), KeyedFixedConstructionError> {
    let members: BTreeSet<_> = inputs
        .iter()
        .filter(|i| i.role() == KeyedFixedInputRole::Member)
        .map(|i| i.parent().outpoint())
        .collect();
    let mut visited = BTreeSet::new();
    let mut offered_fee = 0_u64;
    for row in inputs {
        let Some(cap) = row.capability() else {
            continue;
        };
        if visited.contains(&row.parent().outpoint()) {
            continue;
        }
        let offer = row
            .capability_offer()
            .ok_or(KeyedFixedConstructionError::CapabilitySnapshot)?;
        if !members.is_disjoint(offer.inputs()) {
            return Err(KeyedFixedConstructionError::CapabilityOverlap);
        }
        // Data-address identity only: duplicated vtables are not capability identity.
        // Distinct zero-sized capabilities may share an address and form one group.
        let group: Vec<_> = inputs
            .iter()
            .filter(|other| {
                other
                    .capability()
                    .is_some_and(|c| std::ptr::addr_eq(c, cap))
            })
            .collect();
        if group
            .iter()
            .any(|other| other.capability_offer() != Some(offer))
        {
            return Err(KeyedFixedConstructionError::CapabilitySnapshot);
        }
        let outpoints: BTreeSet<_> = group
            .iter()
            .map(|other| other.parent().outpoint())
            .collect();
        if &outpoints != offer.inputs() {
            return Err(KeyedFixedConstructionError::CapabilityCoverage);
        }
        offered_fee = offered_fee
            .checked_add(offer.fee())
            .ok_or(KeyedFixedConstructionError::Arithmetic)?;
        visited.extend(outpoints);
    }
    if !visited.is_empty() && offered_fee != fee {
        return Err(KeyedFixedConstructionError::CapabilityFee);
    }
    Ok(())
}
fn check_material(
    s: &KeyedFixedStructuralTransaction<'_, '_>,
) -> Result<(), KeyedFixedConstructionError> {
    if s.transaction.outputs().first().map(TargetOutput::program)
        != Some(s.commitment.output_program())
    {
        return Err(KeyedFixedConstructionError::OutputZero);
    }
    if s.transaction.inputs().len() != s.inputs.len() {
        return Err(KeyedFixedConstructionError::InputCensus);
    }
    for ((built, witness), bound) in s
        .transaction
        .inputs()
        .iter()
        .zip(s.transaction.witnesses())
        .zip(s.inputs)
    {
        if built.outpoint() != bound.parent().outpoint() || built.sequence() != bound.sequence() {
            return Err(KeyedFixedConstructionError::InputCensus);
        }
        if bound.role() == KeyedFixedInputRole::Member
            && witness.stack()
                != [
                    s.commitment.leaf().to_vec(),
                    s.commitment.control().to_vec(),
                ]
        {
            return Err(KeyedFixedConstructionError::CovenantWitness);
        }
    }
    Ok(())
}
/// Apply the declared one-sponsor/three-output envelope, sign frozen bytes and
/// verify the sponsor.
///
/// No generalized collateral authorization is claimed.
/// # Errors
/// Returns explicit policy/context, signing or nested cryptographic refusals.
pub fn admit_keyed_fixed(
    target: &ReviewedElementsTapscriptDefinition,
    request: KeyedFixedConstructionRequest<'_, '_>,
    offer: KeyedFixedOutputOffer,
    policy: &KeyedFixedPolicy,
) -> Result<KeyedFixedAdmittedTransaction, KeyedFixedConstructionError> {
    let _span =
        tracing::debug_span!(target: "transaction::keyed_fixed_construct", "admit_keyed", family="keyed")
            .entered();
    let result = if offer.policy_asset == policy.policy_asset {
        construct_core(target, request, offer, true).and_then(|s| admit_core(target, &s, policy))
    } else {
        Err(KeyedFixedConstructionError::PolicyAssetMismatch)
    };
    terminal(&result, "admit");
    result
}
fn admit_core(
    target: &ReviewedElementsTapscriptDefinition,
    s: &KeyedFixedStructuralTransaction<'_, '_>,
    policy: &KeyedFixedPolicy,
) -> Result<KeyedFixedAdmittedTransaction, KeyedFixedConstructionError> {
    check_material(s)?;
    let sponsors: Vec<_> = s
        .inputs
        .iter()
        .enumerate()
        .filter(|(_, i)| i.role() == KeyedFixedInputRole::Collateral)
        .collect();
    let [(index, sponsor)] = sponsors.as_slice() else {
        return Err(KeyedFixedConstructionError::Envelope);
    };
    let outputs = s.transaction.outputs();
    if outputs.len() != 3
        || !crate::construct::admitted_sponsor_program(target, sponsor.parent().output().program())
        || !is_p2wpkh(outputs[1].program())
    {
        return Err(KeyedFixedConstructionError::Envelope);
    }
    let cap = sponsor
        .capability()
        .ok_or(KeyedFixedConstructionError::SponsorCapabilityMissing)?;
    let offer = sponsor
        .capability_offer()
        .ok_or(KeyedFixedConstructionError::CapabilitySnapshot)?;
    if offer.change() != Some(outputs[1].value()) {
        return Err(KeyedFixedConstructionError::ChangeOfferMismatch);
    }
    let destination = cap
        .change_destination()
        .ok_or(KeyedFixedConstructionError::ChangeDestinationMissing)?;
    if crate::taproot::witness_program_script(target, destination.0, &destination.1)
        .map_err(KeyedFixedConstructionError::Target)?
        != outputs[1].program()
    {
        return Err(KeyedFixedConstructionError::ChangeDestinationMismatch);
    }
    let weight = policy_weight(target, s, policy, *index)?;
    let frozen = s.transaction.encode_without_witness();
    let signing = SponsorSigningRequest::new(
        frozen.clone(),
        u16::try_from(*index).map_err(|_| KeyedFixedConstructionError::SigningIndex)?,
        SignerRole::KeyedCollateral,
        SighashProfile::AllInputsAllOutputs,
        outputs.to_vec(),
    );
    let signature = cap
        .sign(&signing)
        .ok_or(KeyedFixedConstructionError::SignatureMissing)?;
    if signature.bound_to() != frozen {
        return Err(KeyedFixedConstructionError::SignatureBinding);
    }
    if signature
        .stack()
        .first()
        .is_some_and(|sig| sig.len() > policy.sponsor_signature_bound)
    {
        return Err(KeyedFixedConstructionError::SignatureBoundExceeded {
            reached: signature.stack()[0].len(),
            bound: policy.sponsor_signature_bound,
        });
    }
    crate::keyed_fixed_sponsor::verify_core(
        &frozen,
        *index,
        sponsor.parent().output(),
        signature.stack(),
    )
    .map_err(KeyedFixedConstructionError::Sponsor)?;
    let mut witnesses = s.transaction.witnesses().to_vec();
    witnesses[*index] = InputWitness::new(signature.stack().to_vec());
    let transaction = TargetTransaction::new(
        s.transaction.version(),
        s.transaction.inputs().to_vec(),
        outputs.to_vec(),
        s.transaction.lock_time(),
        witnesses,
    )
    .map_err(KeyedFixedConstructionError::Target)?;
    if transaction.encode_without_witness() != frozen {
        return Err(KeyedFixedConstructionError::FrozenBytesChanged);
    }
    if transaction.weight() > weight {
        return Err(KeyedFixedConstructionError::FinalWeightExceeded {
            reached: transaction.weight(),
            bound: weight,
        });
    }
    Ok(KeyedFixedAdmittedTransaction {
        transaction,
        weight_bound: weight,
        policy_provenance: policy.provenance.clone(),
    })
}

fn policy_weight(
    target: &ReviewedElementsTapscriptDefinition,
    s: &KeyedFixedStructuralTransaction<'_, '_>,
    policy: &KeyedFixedPolicy,
    index: usize,
) -> Result<u64, KeyedFixedConstructionError> {
    let outputs = s.transaction.outputs();
    if policy.sponsor_signature_bound == 0 || policy.sponsor_signature_bound > 72 {
        return Err(KeyedFixedConstructionError::SignatureBoundOutOfRange);
    }
    if policy.provenance.is_empty()
        || policy.output_minimums.len() != outputs.len()
        || policy.maximum_package_weight == 0
        || policy.maximum_deployment_weight == 0
        || policy.maximum_transaction_weight == 0
    {
        return Err(KeyedFixedConstructionError::ResourceContext);
    }
    let mut budget = s.transaction.witnesses().to_vec();
    budget[index] = InputWitness::new(vec![vec![0; policy.sponsor_signature_bound], vec![0; 33]]);
    let bounded = TargetTransaction::new(
        s.transaction.version(),
        s.transaction.inputs().to_vec(),
        outputs.to_vec(),
        s.transaction.lock_time(),
        budget,
    )
    .map_err(KeyedFixedConstructionError::Target)?;
    let weight = bounded.weight();
    check_weight_formula(target, s, &bounded)?;
    if weight > policy.maximum_transaction_weight {
        return Err(KeyedFixedConstructionError::TransactionWeightExceeded {
            reached: weight,
            bound: policy.maximum_transaction_weight,
        });
    }
    if weight > policy.maximum_deployment_weight {
        return Err(KeyedFixedConstructionError::DeploymentWeightExceeded {
            reached: weight,
            bound: policy.maximum_deployment_weight,
        });
    }
    let package = policy
        .package_weight_before
        .checked_add(weight)
        .ok_or(KeyedFixedConstructionError::Arithmetic)?;
    if package > policy.maximum_package_weight {
        return Err(KeyedFixedConstructionError::PackageWeightExceeded {
            reached: package,
            bound: policy.maximum_package_weight,
        });
    }
    let rate = policy
        .relay_sat_per_kvb
        .max(policy.mempool_sat_per_kvb)
        .max(policy.block_sat_per_kvb);
    let required = bounded
        .virtual_size()
        .checked_mul(rate)
        .and_then(|n| n.checked_add(999))
        .ok_or(KeyedFixedConstructionError::Arithmetic)?
        / 1000;
    if s.fee < required {
        return Err(KeyedFixedConstructionError::FeeShortage {
            offered: s.fee,
            required,
        });
    }
    for (index, (output, minimum)) in outputs.iter().zip(&policy.output_minimums).enumerate() {
        let derived = keyed_fixed_policy_floor(
            output,
            policy.policy_asset,
            policy.dust_relay_sat_per_kvb,
            policy.require_standard,
        )?;
        if let ValueField::Explicit(n) = output.value()
            && n < (*minimum).max(derived)
        {
            return Err(KeyedFixedConstructionError::OutputPolicy { index });
        }
    }
    Ok(weight)
}

fn check_weight_formula(
    target: &ReviewedElementsTapscriptDefinition,
    s: &KeyedFixedStructuralTransaction<'_, '_>,
    bounded: &TargetTransaction,
) -> Result<(), KeyedFixedConstructionError> {
    // Serialize only non-member witnesses independently of the complete weight.
    let mut other = bounded.output_witness_bytes();
    for (input, witness) in s.inputs.iter().zip(bounded.witnesses()) {
        if input.role() == KeyedFixedInputRole::Collateral {
            witness.encode(&mut other);
        }
    }
    // keyed_fixed_transaction_weight derives its member term from keyed_fixed_witness_bytes
    // plus the three empty proof/peg-in prefixes, not from actual member witnesses.
    let formula = tapscript::keyed_fixed_resources::transaction_weight(
        target,
        s.member_count as u64,
        bounded.encode_without_witness().len() as u64,
        other.len() as u64,
    )
    .map_err(|_| KeyedFixedConstructionError::Arithmetic)?;
    let serialized = bounded.weight();
    if serialized != formula {
        return Err(KeyedFixedConstructionError::WeightFormulaMismatch {
            serialized,
            formula,
        });
    }
    Ok(())
}
