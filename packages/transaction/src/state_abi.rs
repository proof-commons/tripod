//! The candidate maturity-announcement ABI (§12.1, §12.2, §12.6, §12.9).
//!
//! # A candidate, and nothing that could be mistaken for more
//!
//! §12.9 names the result a candidate and says in as many words that it
//! is not final and carries no digest. All three are structural here
//! rather than asserted: [`MaturityAbiStatus`] has one member and is
//! returned by a method that reads no field, so there is nothing to set;
//! there is no digest field and no accessor that would return one; and
//! the derivation refuses a bundle claiming more than the prototype
//! status it was linked at. No type in this module is final, and none
//! becomes final later, because the operations that would carry a
//! candidate past this point are not written yet.
//!
//! # Derived over a validated view, which is what makes the first half
//!
//! The derivation takes a [`ValidatedMaturityStateView`] rather than a
//! bare bundle, and the type is the evidence. A validated view exists
//! only where the supplied predecessor metadata and representation nonce,
//! encoded canonically and committed under the accepted bundle's own
//! constructor policy over the linked static subtree, reproduced the
//! supplied predecessor program — which is the announcement leaf's own
//! authentication equation, computed first-party. That computation is
//! exactly the reconstruction half of the link's current-state
//! obligation, so the partition below can discharge that obligation
//! against a fact the argument's type proves happened rather than
//! against a sentence.
//!
//! The other half of it — that the outpoint is the *current* root — is
//! established by no layer here, and it reappears below as this ABI's own
//! narrower residual. The linker's obligation vocabulary is unchanged by
//! either half: what a linked bundle owes is a fact about that link, not
//! a record of what a later layer computed.
//!
//! # Two layouts, no shape index, and why the leaf does not state them
//!
//! The reduced announcement leaf pins one input position and one output
//! position: the STATE predecessor it spends and the STATE successor it
//! recreates. Every other clause of §12.1 and §12.2 — that there is
//! exactly one STATE input, that a request cannot select another
//! coordinator, that no other protocol output is admitted, and that
//! neither the sponsor-change nor the fee role can stand in for the STATE
//! successor or for each other — is enforced by no bytes the target runs.
//! They are statements of this layer, and this module is the only place
//! they are true, which is why they are types here and not comments.
//!
//! Neither layout carries a shape index. The STATE generation has no
//! linked shape quantity to index by: one STATE input and one STATE
//! successor is the whole family, and the optional sponsor suffix has no
//! count before a transaction is built. So the sponsor suffix is stated
//! as a rule about where it begins rather than as a range, and the
//! coordinator is a rule about which index it occupies rather than a
//! selection anything makes.
//!
//! # Seven roles, seven attributes, each read from its one source
//!
//! §12.6 asks each witness role to carry an encoding, a minimum and
//! maximum width, a public or held-key classification, a source, a
//! consumer, a target program and a canonical position. Four of the seven
//! are read rather than restated, and that is the point: the declared
//! type and its widths come from the composed record's own witness
//! schedule, the consumer comes from the carrier closure's attribution in
//! the linker, and the position is the index that schedule declares. A
//! second copy of any of them here would be a second place for the same
//! fact to drift, and the drift would be invisible — a witness schedule
//! and an ABI disagreeing about a width produce a spend nothing accepts
//! and no test notices.
//!
//! The three that are stated are stated because nothing publishes them:
//! the classification of a held key is a fact about who signs, the source
//! is a fact about which stage of construction supplies the item, and the
//! target program is the leaf the record composes.
//!
//! # The relay verdict is the target's, with its residual named
//!
//! The sponsorless announcement form review takes the composed record's declared
//! initial-item widths and the reviewed target's policy bound. Both
//! schedules have consensus admission and no fee output. The historical
//! eighty-six-byte metadata item exceeds the eighty-byte relay bound, so
//! its verdict is refusal with direct submission to a producer as its
//! route; the split residual stays outstanding. The variable schedule
//! carries fifty-three metadata bytes, so its verdict is conditional
//! admission with the fee-free topology and direct-submission conditions.
//! Its outstanding obligation names the absence of a measurement of
//! initial argument widths over the linked instructions. Both schedules
//! use the standard announcement transaction version; the verdict records
//! the different relay routes. The sponsored form's relay verdict remains
//! the C2 observation obligation; this review does not assign it a verdict.
//!
//! # The partition refuses rather than lists
//!
//! [`InheritedStateLinkObligations`] names what this derivation did about
//! each obligation the link left standing, and both halves are named
//! sets. The check is then made against the bundle's own set rather than
//! trusted: an obligation the link carries and this module does not name
//! is neither discharged nor carried, and the derivation
//! refuses instead of quietly dropping it. The ABI's own outstanding set
//! is structurally non-empty for the reason the link's is — the least
//! obligation is a field of its own — and its carried members are derived
//! from the partition rather than written down a second time.

use std::collections::{BTreeMap, BTreeSet};
use std::num::NonZeroUsize;

use linker::{
    LinkedArtifactStatus, OutstandingStateLinkObligations, StateLinkObligation,
    state_witness_component,
};
use tapscript::{
    StateAnnouncementProgram, StateLeafRole, StateProgramComponent, StateProgramWitness,
    StateWitnessSchedule,
};
use target_elements::{
    FormAdmission, LeafVersion, PayloadWidth, RelayCondition, ReviewedElementsTapscriptDefinition,
    StackValueType, TargetContractVersion, TransactionForm, TransactionFormReview,
    review_maturity_announcement_form,
};

use crate::abi::{SequenceConstraint, TargetTransactionVersion};
use crate::error::TransactionRefusal;
use crate::state_view::ValidatedMaturityStateView;

/// Bytes and positional claims offered to the classification boundary.
///
/// The spent census is supplied in input order. It is caller evidence,
/// not a claim that the outputs remain unspent on any chain.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaturityAbiOffer<'a> {
    pub bytes: &'a [u8],
    pub spent: &'a [crate::SpentOutputCensusEntry],
    pub reserve_asset: crate::bytes::AssetId,
    pub inputs: Vec<MaturityPositionClaim>,
    pub outputs: Vec<MaturityPositionClaim>,
    pub canonical: realization::ObservedCanonicalPartition,
}

/// An untrusted proposed classification of one concrete position.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MaturityPositionClaim {
    State,
    SponsorMember,
    SponsorChange,
    TargetFee,
    Family(architecture::ObjectId),
    Projection(architecture::ProjectionId),
    Destruction,
    DataOutput(Option<architecture::TagId>),
}

/// A refusal of offered maturity ABI classification, before construction admission.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MaturityAbiClassificationRefusal {
    Decode(Box<TransactionRefusal>),
    IssuanceOffered,
    ContractMismatch,
    SpentCensus {
        inputs: usize,
        entries: usize,
    },
    Unclassified {
        side: realization::ObservedSide,
        position: usize,
    },
    ClaimContradictsBytes {
        side: realization::ObservedSide,
        position: usize,
    },
    UnadmittedOutputRole {
        position: usize,
        claim: MaturityPositionClaim,
    },
    RelationVerdicts {
        failed: BTreeMap<realization::RelationId, realization::RelationFailure>,
    },
    Realization(Box<realization::RealizationError>),
    ClaimCensus {
        side: realization::ObservedSide,
        positions: usize,
        claims: usize,
    },
}

/// Classified bytes, their original claims and the whole realization report.
///
/// This is neither role validation nor admission to safe construction. The
/// sponsor-input bound supplied to evaluation is the observed count,
/// uncalibrated and never admission evidence. Sponsor objects are projected
/// without owners: sponsor-input recognition fails, blocking both its
/// cardinality and sponsor isolation. Cardinality never reads the bound. These
/// retained verdicts are artifacts of that ownerless projection, not
/// admission findings about the offered transaction. The role
/// boundary must carry the uncalibrated maximum as a residual.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidatedMaturityAbiClassification {
    transaction: crate::bytes::TargetTransaction,
    inputs: Vec<MaturityPositionClaim>,
    outputs: Vec<MaturityPositionClaim>,
    report: realization::ConformanceReport,
    sponsor_input_bound_uncalibrated: Option<realization::Count>,
}

impl ValidatedMaturityAbiClassification {
    /// The decoded transaction, without any witness-validity assertion.
    #[must_use]
    pub const fn transaction(&self) -> &crate::bytes::TargetTransaction {
        &self.transaction
    }

    /// The offered input claims, authenticated where the ABI fixes their meaning.
    #[must_use]
    pub fn input_claims(&self) -> &[MaturityPositionClaim] {
        &self.inputs
    }

    /// The offered output claims.
    #[must_use]
    pub fn output_claims(&self) -> &[MaturityPositionClaim] {
        &self.outputs
    }

    /// Every evaluated verdict, including findings this boundary does not own.
    #[must_use]
    pub const fn report(&self) -> &realization::ConformanceReport {
        &self.report
    }

    /// The evaluation-only sponsor-input bound; never calibrated policy.
    ///
    /// Absent when there are no sponsor inputs. Evaluation still receives
    /// zero in that case, because the evaluator requires the bound even
    /// for an empty sponsor family.
    #[must_use]
    pub const fn sponsor_input_bound_uncalibrated(&self) -> Option<realization::Count> {
        self.sponsor_input_bound_uncalibrated
    }
}

/// Authenticate and classify an offered announcement without admitting construction.
///
/// Only the two family closures, open-flow policy, canonical-delta policy
/// and projection policy can refuse through their failed verdicts. Other
/// verdicts are retained for their owning boundaries. Sponsor values are
/// opaque; no owner authorization or proof finalization is asserted.
/// Acceptance means no owned failure; the two control tests pin all five
/// owned identities as passed, rather than assuming absent or blocked passes.
///
/// `FeeSponsorInputMax` is supplied as the observed sponsor-input count
/// as an evaluation input. It is uncalibrated, never admission evidence,
/// and is exposed explicitly on the validated result. Sponsor objects have
/// no owners, so sponsor-input recognition fails, blocking sponsor-input
/// cardinality and sponsor isolation without reading the bound. These
/// retained verdicts are artifacts of the ownerless projection.
///
/// # Errors
///
/// Returns a typed decode, census, claim-authentication, unadmitted-role,
/// realization or owned-relation refusal. Issuance is refused by decoding
/// inside this entry, before a typed transaction can erase the offer.
/// [`MaturityAbiClassificationRefusal::ContractMismatch`] guards a future
/// second policy version. An end-to-end mismatch is unreachable today:
/// `PublicMaturityStateView::validate` rejects a policy differing from the
/// reviewed target, `derive_maturity_announcement_abi` repeats that check,
/// and the derivation copies the bundle policy into the ABI's contract.
pub fn validate_maturity_abi_classification(
    abi: &CandidateMaturityAnnouncementAbi,
    view: &ValidatedMaturityStateView,
    realization: &realization::ScopedRealizationSpec,
    offer: &MaturityAbiOffer<'_>,
) -> Result<ValidatedMaturityAbiClassification, MaturityAbiClassificationRefusal> {
    use MaturityAbiClassificationRefusal as Refusal;
    use architecture::OperationId;
    let transaction = crate::bytes::TargetTransaction::decode(offer.bytes).map_err(|error| {
        if error == TransactionRefusal::IssuanceInputRefused {
            Refusal::IssuanceOffered
        } else {
            Refusal::Decode(Box::new(error))
        }
    })?;
    if transaction.inputs().len() != offer.spent.len() {
        return Err(Refusal::SpentCensus {
            inputs: transaction.inputs().len(),
            entries: offer.spent.len(),
        });
    }
    check_maturity_claim_census(
        realization::ObservedSide::Input,
        transaction.inputs().len(),
        &offer.inputs,
    )?;
    check_maturity_claim_census(
        realization::ObservedSide::Output,
        transaction.outputs().len(),
        &offer.outputs,
    )?;
    check_maturity_policy_pair(
        abi.contract(),
        view.view()
            .accepted_linked_bundle()
            .policy()
            .target_policy(),
    )?;
    authenticate_maturity_claims(view, offer, &transaction)?;
    if realization
        .operation(OperationId::AnnounceMaturity)
        .is_none()
    {
        return Err(maturity_realization_refusal(
            realization::RealizationError::OperationOutsideScope(OperationId::AnnounceMaturity),
        ));
    }
    let observation = maturity_classification_observation(offer)?;
    let count = observation.bounds[&architecture::BoundId::FeeSponsorInputMax];
    let report = realization
        .evaluate_operation(&observation)
        .map_err(maturity_realization_refusal)?;
    let owned = maturity_classification_relations();
    let failed = report
        .verdicts
        .iter()
        .filter_map(|verdict| match &verdict.status {
            realization::RelationStatus::Failed { reason } if owned.contains(&verdict.relation) => {
                Some((verdict.relation.clone(), *reason))
            }
            _ => None,
        })
        .collect::<BTreeMap<_, _>>();
    if !failed.is_empty() {
        return Err(Refusal::RelationVerdicts { failed });
    }
    Ok(ValidatedMaturityAbiClassification {
        transaction,
        inputs: offer.inputs.clone(),
        outputs: offer.outputs.clone(),
        report,
        sponsor_input_bound_uncalibrated: (count.get() != 0).then_some(count),
    })
}

fn maturity_realization_refusal(
    error: realization::RealizationError,
) -> MaturityAbiClassificationRefusal {
    MaturityAbiClassificationRefusal::Realization(Box::new(error))
}

const fn check_maturity_claim_census(
    side: realization::ObservedSide,
    positions: usize,
    claims: &[MaturityPositionClaim],
) -> Result<(), MaturityAbiClassificationRefusal> {
    use MaturityAbiClassificationRefusal as Refusal;
    match claims.len() {
        count if count < positions => Err(Refusal::Unclassified {
            side,
            position: count,
        }),
        count if count > positions => Err(Refusal::ClaimCensus {
            side,
            positions,
            claims: count,
        }),
        _ => Ok(()),
    }
}

pub(super) fn check_maturity_policy_pair(
    abi: TargetContractVersion,
    view: TargetContractVersion,
) -> Result<(), MaturityAbiClassificationRefusal> {
    if abi == view {
        Ok(())
    } else {
        Err(MaturityAbiClassificationRefusal::ContractMismatch)
    }
}

fn authenticate_maturity_claims(
    view: &ValidatedMaturityStateView,
    offer: &MaturityAbiOffer<'_>,
    transaction: &crate::bytes::TargetTransaction,
) -> Result<(), MaturityAbiClassificationRefusal> {
    use realization::ObservedSide;
    let inputs =
        offer
            .inputs
            .iter()
            .zip(offer.spent)
            .enumerate()
            .map(|(position, (claim, spent))| {
                (
                    ObservedSide::Input,
                    position,
                    *claim,
                    spent.asset(),
                    spent.value(),
                    spent.program(),
                )
            });
    let outputs = offer
        .outputs
        .iter()
        .zip(transaction.outputs())
        .enumerate()
        .map(|(position, (claim, output))| {
            (
                ObservedSide::Output,
                position,
                *claim,
                output.asset(),
                output.value(),
                output.program(),
            )
        });
    for (side, position, claim, asset, value, program) in inputs.chain(outputs) {
        if !maturity_claim_matches(
            view,
            offer.reserve_asset,
            side,
            claim,
            (asset, value, program),
        ) {
            return Err(MaturityAbiClassificationRefusal::ClaimContradictsBytes { side, position });
        }
    }
    for (position, claim) in offer.outputs.iter().enumerate() {
        if matches!(
            claim,
            MaturityPositionClaim::Destruction | MaturityPositionClaim::DataOutput(_)
        ) {
            return Err(MaturityAbiClassificationRefusal::UnadmittedOutputRole {
                position,
                claim: *claim,
            });
        }
    }
    Ok(())
}

fn maturity_claim_matches(
    view: &ValidatedMaturityStateView,
    reserve: crate::bytes::AssetId,
    side: realization::ObservedSide,
    claim: MaturityPositionClaim,
    fields: (crate::bytes::AssetField, crate::bytes::ValueField, &[u8]),
) -> bool {
    use crate::bytes::{AssetField, ValueField};
    use MaturityPositionClaim as Claim;
    use architecture::{ObjectId, ProjectionId};
    use realization::ObservedSide;
    let (asset, value, program) = fields;
    let sponsor = asset == AssetField::Explicit(reserve)
        && asset != view.view().asset()
        && maturity_program_matches(
            match side {
                ObservedSide::Input => MaturityProgramClass::SponsorMember,
                ObservedSide::Output => MaturityProgramClass::SponsorChange,
            },
            program,
        );
    match claim {
        Claim::State | Claim::Family(ObjectId::State) => {
            maturity_state_fields_match(view, asset, value)
                && maturity_program_matches(MaturityProgramClass::State, program)
        }
        Claim::SponsorMember => side == ObservedSide::Input && sponsor,
        Claim::SponsorChange => side == ObservedSide::Output && sponsor,
        Claim::Family(ObjectId::PlainLbtc) => sponsor,
        Claim::TargetFee => {
            side == ObservedSide::Output
                && asset == AssetField::Explicit(reserve)
                && asset != view.view().asset()
                && program.is_empty()
                && matches!(value, ValueField::Explicit(_))
        }
        Claim::Projection(ProjectionId::TransitionCertificate) => false,
        Claim::Projection(_) | Claim::Destruction | Claim::DataOutput(_) => {
            side == ObservedSide::Output
        }
        Claim::Family(_) => true,
    }
}

// Member provenance: sponsor::WITNESS_V0_KEYHASH_PROGRAM_BYTES and its
// version-zero key-hash class. STATE uses the taproot program contract.
// Change is the image of state_construct::sponsor_change_output, which
// encodes the capability's version and payload through witness_program_script.
#[derive(Clone, Copy)]
enum MaturityProgramClass {
    SponsorMember,
    SponsorChange,
    State,
}

fn maturity_program_matches(class: MaturityProgramClass, program: &[u8]) -> bool {
    static TARGET: std::sync::LazyLock<
        Result<ReviewedElementsTapscriptDefinition, Vec<target_elements::TargetError>>,
    > = std::sync::LazyLock::new(target_elements::reviewed_elements_tapscript);
    let Ok(target) = TARGET.as_ref() else {
        return false;
    };
    // The reviewed encoder emits the version opcode, direct-push width,
    // then payload. Compare its output rather than transcribing opcodes.
    let Some(payload) = program.get(2..) else {
        return false;
    };
    let encoded_matches = |version| {
        crate::taproot::witness_program_script(target, version, payload)
            .is_ok_and(|encoded| encoded == program)
    };
    match class {
        MaturityProgramClass::SponsorMember => {
            payload.len() == crate::sponsor::WITNESS_V0_KEYHASH_PROGRAM_BYTES && encoded_matches(0)
        }
        MaturityProgramClass::State => {
            payload.len() == crate::taproot::TAPROOT_PROGRAM_BYTES
                && encoded_matches(crate::taproot::TAPROOT_WITNESS_VERSION)
        }
        MaturityProgramClass::SponsorChange => (0..=u8::MAX).any(encoded_matches),
    }
}

fn maturity_state_fields_match(
    view: &ValidatedMaturityStateView,
    asset: crate::bytes::AssetField,
    value: crate::bytes::ValueField,
) -> bool {
    use crate::bytes::{AssetField, ValueField};
    use linker::{StateLinkSymbol, StateSymbolValue};
    let public = view.view();
    let entries = public.deployment_symbols().entries();
    let Some(StateSymbolValue::Asset(defined_asset)) = entries
        .get(&StateLinkSymbol::StateAsset)
        .map(|entry| entry.definition().value())
    else {
        return false;
    };
    let Some(StateSymbolValue::ExplicitAmount(defined_amount)) = entries
        .get(&StateLinkSymbol::StateAmount)
        .map(|entry| entry.definition().value())
    else {
        return false;
    };
    let AssetField::Explicit(identifier) = asset else {
        return false;
    };
    let singleton = realization::ProtocolAmount::ONE.get();
    asset == public.asset()
        && value == public.value()
        && value == ValueField::Explicit(singleton)
        && identifier.internal().as_slice() == defined_asset.bytes()
        && defined_amount.bytes() == singleton.to_le_bytes()
}

fn maturity_classification_relations() -> BTreeSet<realization::RelationId> {
    use realization::{RelationId, RelationKind, RelationSubject, TransactionSide};
    let operation = architecture::OperationId::AnnounceMaturity;
    let mut ids = [
        RelationKind::OpenFlowPolicy,
        RelationKind::CanonicalDeltaPolicy,
        RelationKind::ProjectionPolicy,
    ]
    .into_iter()
    .map(|kind| RelationId::new(operation, kind, RelationSubject::Operation))
    .collect::<BTreeSet<_>>();
    for side in [TransactionSide::Input, TransactionSide::Output] {
        ids.insert(RelationId::new(
            operation,
            RelationKind::AllowedObjectFamilies,
            RelationSubject::TransactionSide { side },
        ));
    }
    ids
}

fn maturity_classification_observation(
    offer: &MaturityAbiOffer<'_>,
) -> Result<realization::OperationObservation, MaturityAbiClassificationRefusal> {
    use MaturityPositionClaim as Claim;
    use architecture::{BoundId, ObjectId, OpenFlowKind, OperationId, ProjectionId};
    use realization::{
        ObservedAsset, ObservedObject, ObservedObjectKind, ObservedObjectRef, ObservedSide,
        ObservedValue, ProtocolAmount, RepresentationMode,
    };
    let mut objects = Vec::new();
    let mut sources = Vec::new();
    let mut destinations = Vec::new();
    let mut projections = BTreeSet::from([ProjectionId::TransitionCertificate]);
    for (side, claims) in [
        (ObservedSide::Input, &offer.inputs),
        (ObservedSide::Output, &offer.outputs),
    ] {
        for (position, claim) in claims.iter().enumerate() {
            let ordinal = u32::try_from(position).map_err(|_| {
                maturity_realization_refusal(realization::RealizationError::CountOverflow)
            })?;
            let reference = ObservedObjectRef { side, ordinal };
            let family = match claim {
                Claim::State => ObjectId::State,
                Claim::SponsorMember | Claim::SponsorChange => ObjectId::PlainLbtc,
                Claim::Family(family) => *family,
                Claim::Projection(projection) => {
                    projections.insert(*projection);
                    continue;
                }
                Claim::TargetFee | Claim::Destruction | Claim::DataOutput(_) => continue,
            };
            let sponsor = family == ObjectId::PlainLbtc;
            if sponsor {
                match side {
                    ObservedSide::Input => sources.push(reference),
                    ObservedSide::Output => destinations.push(reference),
                }
            }
            objects.push(ObservedObject {
                reference,
                kind: ObservedObjectKind::Declared(family),
                asset: architecture::ARCHITECTURE
                    .object(family)
                    .map_or(ObservedAsset::Foreign(ordinal), |definition| {
                        ObservedAsset::Declared(definition.asset)
                    }),
                value: if sponsor {
                    ObservedValue::SponsorOpaque
                } else {
                    ObservedValue::Protocol(ProtocolAmount::ONE)
                },
                owner: None,
                representation: RepresentationMode::Explicit,
            });
        }
    }
    let count =
        realization::Count::new(u64::try_from(sources.len()).map_err(|_| {
            maturity_realization_refusal(realization::RealizationError::CountOverflow)
        })?);
    let open_flows = if sources.is_empty()
        && destinations.is_empty()
        && !offer.outputs.contains(&Claim::TargetFee)
    {
        Vec::new()
    } else {
        vec![realization::ObservedOpenFlow {
            kind: OpenFlowKind::FeeSponsor,
            sources,
            destinations,
            fee: ProtocolAmount::ZERO,
        }]
    };
    Ok(realization::OperationObservation {
        operation: OperationId::AnnounceMaturity,
        region: realization::ObservedOperationRegion {
            members: objects.iter().map(|object| object.reference).collect(),
            canonical_partition: offer.canonical.clone(),
            open_flows,
        },
        objects,
        projections,
        protocol_signers: BTreeSet::new(),
        sponsor_signers: BTreeSet::new(),
        other_regions: Vec::new(),
        root_effects: Vec::new(),
        bounds: BTreeMap::from([(BoundId::FeeSponsorInputMax, count)]),
    })
}

/// The status vocabulary a derived announcement ABI is distinguished by
/// (§12.9).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MaturityAbiStatus {
    /// Derived and internally checked; no operation evidence, and no
    /// digest.
    Candidate,
}

/// Which input the STATE predecessor and its coordinator occupy (§12.1).
///
/// A rule rather than a selection. §12.3 gives a request no field for the
/// coordinator, and this type is why it needs none: the index is a
/// constant of the ABI, and the leaf the spend executes pins that index of
/// itself, so a layout placing it elsewhere would emit bytes the leaf
/// rejects.
///
/// One index and not a run, which is how "there is exactly one STATE
/// input" is structural here: there is no second STATE position to
/// represent, so a layout carrying two has no representation to be
/// refused in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MaturityCoordinatorRule {
    index: u16,
}

impl MaturityCoordinatorRule {
    /// The input index the STATE predecessor and coordinator occupy.
    #[must_use]
    pub const fn index(self) -> u16 {
        self.index
    }
}

/// Where the optional sponsor suffix begins (§12.1).
///
/// A rule and not a range, because a range would need a count and no
/// count exists before a transaction is built: how many sponsor inputs a
/// request brings is a fact about that request, while where they start is
/// a fact about the layout. Stating the first index alone is the whole of
/// what this layer knows, and an end stated here would be an invention
/// read back later as an ABI constraint.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MaturitySponsorSuffixRule {
    first: u16,
}

impl MaturitySponsorSuffixRule {
    /// The first input index the sponsor suffix may occupy.
    #[must_use]
    pub const fn first(self) -> u16 {
        self.first
    }
}

/// The input layout of a maturity announcement (§12.1).
///
/// The canonical order a run of sponsor inputs is sorted in is the
/// inherited one and is deliberately not restated here: one ordering with
/// one name is what keeps a later change to it a change everywhere.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MaturityInputLayout {
    coordinator: MaturityCoordinatorRule,
    sponsor_suffix: MaturitySponsorSuffixRule,
}

impl MaturityInputLayout {
    /// The layout §12.1 states.
    #[must_use]
    pub const fn stated() -> Self {
        Self {
            coordinator: MaturityCoordinatorRule { index: 0 },
            sponsor_suffix: MaturitySponsorSuffixRule { first: 1 },
        }
    }

    /// Which input the STATE predecessor and coordinator occupy.
    #[must_use]
    pub const fn coordinator(self) -> MaturityCoordinatorRule {
        self.coordinator
    }

    /// Where the optional sponsor suffix begins.
    #[must_use]
    pub const fn sponsor_suffix(self) -> MaturitySponsorSuffixRule {
        self.sponsor_suffix
    }
}

/// One output role a maturity announcement admits (§12.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MaturityOutputRole {
    /// The canonical STATE successor.
    StateSuccessor,
    /// The sponsor's change, where the form carries a sponsor region.
    SponsorChange,
    /// The target's own fee role, where the form requires one.
    TargetFee,
}

impl MaturityOutputRole {
    /// Every admitted role, in §12.2's own order.
    ///
    /// Walkable so that "no other protocol output is admitted" is a
    /// census a test reads rather than a sentence it takes on trust.
    pub const ALL: &'static [Self] = &[Self::StateSuccessor, Self::SponsorChange, Self::TargetFee];

    /// Where §12.2 places this role.
    #[must_use]
    pub const fn placement(self) -> MaturityOutputPlacementRule {
        match self {
            Self::StateSuccessor => MaturityOutputPlacementRule::OutputZero,
            Self::SponsorChange => MaturityOutputPlacementRule::NextOptionalOutput,
            Self::TargetFee => MaturityOutputPlacementRule::FinalOutputWhereRequired,
        }
    }

    /// Whether an output standing as this role can also stand as
    /// `other`.
    ///
    /// True only where the two are the same role. §12.2 states the
    /// STATE successor against the other two in both directions; the
    /// remaining pair is refused here for the reason that makes the
    /// stated ones true, which is that each role is recognized by a
    /// different program — a sponsor's change pays a sponsor-change
    /// witness program and the target's fee role carries the empty one,
    /// so neither output is the other whatever its amount.
    #[must_use]
    pub const fn can_satisfy(self, other: Self) -> bool {
        matches!(
            (self, other),
            (Self::StateSuccessor, Self::StateSuccessor)
                | (Self::SponsorChange, Self::SponsorChange)
                | (Self::TargetFee, Self::TargetFee)
        )
    }
}

/// Where one output role sits (§12.2).
///
/// A rule per role rather than an index per role: only the successor has
/// a position this layer can state, and the other two are placed relative
/// to a census that does not exist until a transaction does.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MaturityOutputPlacementRule {
    /// Output zero, always present.
    OutputZero,
    /// The next output after the successor, present only where the form
    /// carries a sponsor region.
    NextOptionalOutput,
    /// The final output, present only where the form requires the
    /// target's fee role.
    FinalOutputWhereRequired,
}

/// The output layout of a maturity announcement (§12.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MaturityOutputLayout {
    successor: u16,
}

impl MaturityOutputLayout {
    /// The layout §12.2 states.
    #[must_use]
    pub const fn stated() -> Self {
        Self { successor: 0 }
    }

    /// Where the canonical STATE successor sits.
    ///
    /// The one output position the executing leaf pins of itself, which
    /// is why it is an index here while the other two roles are rules.
    #[must_use]
    pub const fn successor_position(self) -> u16 {
        self.successor
    }
}

/// The reviewed target's own verdict on the sponsorless announcement form.
///
/// The sponsored form's relay verdict remains the C2 observation obligation.
///
/// A reading of the reviewed form census rather than four fields copied
/// out of it. Copying would have given the figures a second definition,
/// and a policy fact with two definitions is a policy fact that can
/// disagree with itself; holding the review and delegating to it cannot.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaturityRelayVerdict {
    review: TransactionFormReview,
}

impl MaturityRelayVerdict {
    const fn from_review(review: TransactionFormReview) -> Self {
        Self { review }
    }

    /// The sponsorless announcement form's verdict, read from a reviewed census.
    ///
    /// # Errors
    ///
    /// [`TransactionRefusal::MissingReviewedTransactionForm`] when the
    /// census holds no review of the announcement form, whether because
    /// the entry is absent or because the entry found under that key
    /// reviews a different form. Both are the same finding — this ABI
    /// publishes a verdict it was unable to read — and a verdict invented
    /// in place of one is the failure the refusal exists to prevent.
    pub fn read(
        forms: &BTreeMap<TransactionForm, TransactionFormReview>,
    ) -> Result<Self, TransactionRefusal> {
        let review = forms
            .get(&TransactionForm::MaturityAnnouncement)
            .filter(|review| review.form() == TransactionForm::MaturityAnnouncement)
            .ok_or(TransactionRefusal::MissingReviewedTransactionForm {
                form: TransactionForm::MaturityAnnouncement,
            })?;
        Ok(Self {
            review: review.clone(),
        })
    }

    /// What consensus does with the sponsorless announcement form.
    #[must_use]
    pub const fn consensus(&self) -> FormAdmission {
        self.review.consensus()
    }

    /// What the default relay policy does with the sponsorless form.
    #[must_use]
    pub const fn relay(&self) -> FormAdmission {
        self.review.relay()
    }

    /// The route recorded beside a refused relay verdict.
    #[must_use]
    pub const fn relay_conditions(&self) -> &BTreeSet<RelayCondition> {
        self.review.relay_conditions()
    }

    /// Whether the sponsorless form itself fixes a fee output.
    #[must_use]
    pub const fn fee_output_present(&self) -> bool {
        self.review.fee_output_present()
    }
}

/// Who holds the material one witness item is built from (§12.6).
///
/// Two members, because the announcement's schedule divides exactly
/// twice. The guide's "test-secret" classification is a test's word for
/// the second one: what makes the operator's signature different from the
/// other six is not that it is secret but that producing it needs a key
/// somebody holds, which is true of a published test signer and of a
/// deployed operator alike.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WitnessClassification {
    /// Built from public facts, by anybody who has them.
    Public,
    /// Produced by the holder of a key, and by nobody else.
    SignerHeld,
}

/// Which stage of construction supplies one witness item (§12.6).
///
/// A statement about where the item is read, not a value read here: this
/// ABI builds no witness, and the source is the attribute that tells a
/// builder which stage owes it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MaturityWitnessSource {
    /// The constructor search's own result for the successor.
    ///
    /// Both the successor's nonce and the prefix byte of its output key
    /// come out of the one search, which has a single owner with a
    /// measured budget and its own leastness evidence.
    SuccessorConstructorSearch,
    /// The typed announcement request.
    TypedRequest,
    /// The linked bundle's committed static subtree.
    LinkedStaticSubtree,
    /// The validated view's predecessor metadata and representation
    /// nonce, encoded canonically.
    ValidatedViewPredecessorMetadata,
    /// The parity of the tweak that reconstructs the predecessor
    /// program.
    ///
    /// Not the view, which carries the predecessor program as the
    /// target does — an x-only key with no prefix byte in it — and no
    /// parity beside it. The byte is produced by the same commitment the
    /// view's own check runs, so a builder reads it from that
    /// computation rather than from the ten public facts.
    PredecessorCommitmentParity,
    /// The operator's signing over the finalized candidate.
    OperatorSigning,
}

/// One witness role with the seven attributes §12.6 asks of it.
///
/// The declared type, the two widths, the consumer and the position are
/// read from the composed record and the linker's own attribution; the
/// classification, the source and the target program are stated, and each
/// of the three says below why nothing publishes it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaturityWitnessRole {
    role: StateProgramWitness,
    declared: StackValueType,
    minimum_width: Option<usize>,
    maximum_width: Option<usize>,
    classification: WitnessClassification,
    source: MaturityWitnessSource,
    consumer: StateProgramComponent,
    program: StateLeafRole,
    position: usize,
}

impl MaturityWitnessRole {
    /// Which role of the record's schedule this is.
    #[must_use]
    pub const fn role(&self) -> StateProgramWitness {
        self.role
    }

    /// The encoding the record declares for it.
    #[must_use]
    pub const fn declared(&self) -> &StackValueType {
        &self.declared
    }

    /// The smallest width the declared encoding admits.
    ///
    /// Absent where the declared type fixes no width of its own. Every
    /// role of this schedule declares either an exact byte range or an
    /// encoded field whose reviewed payload width is exact, so all seven
    /// answer; a type that stated no width would have none to report, and
    /// a figure invented in its place would be a fact this layer made up.
    #[must_use]
    pub const fn minimum_width(&self) -> Option<usize> {
        self.minimum_width
    }

    /// The largest width the declared encoding admits.
    ///
    /// Absent under the same condition as the minimum, and for the same
    /// reason.
    #[must_use]
    pub const fn maximum_width(&self) -> Option<usize> {
        self.maximum_width
    }

    /// Whether the item is public or needs a held key.
    #[must_use]
    pub const fn classification(&self) -> WitnessClassification {
        self.classification
    }

    /// Which stage of construction supplies it.
    #[must_use]
    pub const fn source(&self) -> MaturityWitnessSource {
        self.source
    }

    /// The composed component that reads it.
    ///
    /// The carrier closure's own attribution, read from the linker rather
    /// than recomputed. The successor's nonce is where the two statements
    /// visibly differ: its declared type is the one the copy-through
    /// component declares in the schedule, while the component that reads
    /// it is the successor reconstruction. A record naming one and hiding
    /// the other would be wrong for one of the two readers.
    #[must_use]
    pub const fn consumer(&self) -> StateProgramComponent {
        self.consumer
    }

    /// The committed leaf whose program consumes it.
    ///
    /// The announcement leaf for all seven, because the seven are one
    /// leaf's witness and the record is that leaf's composed program. It
    /// is stated rather than read: the record publishes no leaf role of
    /// its own, and a lookup against the bundle's relocated program would
    /// compare across the substitution boundary and disagree for a reason
    /// that has nothing to do with the question.
    #[must_use]
    pub const fn program(&self) -> StateLeafRole {
        self.program
    }

    /// Where it sits in the record's deepest-first schedule.
    #[must_use]
    pub const fn position(&self) -> usize {
        self.position
    }
}

/// One obligation the announcement ABI creates or carries and does not
/// discharge.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum MaturityAbiObligation {
    /// Nothing establishes that the predecessor outpoint is the current
    /// root.
    ///
    /// The least obligation, and the narrower half of the link's
    /// current-state obligation. The reconstruction half is discharged by
    /// the computation a validated view's type proves happened; this half
    /// is a fact about a chain, no layer in this crate observes one, and a
    /// check here would need branch-indexed history this crate does not
    /// carry. Naming it narrowly is what keeps the discharge honest: an
    /// obligation carried whole would understate what was computed, and
    /// one dropped would overstate it.
    CurrentStateRootFreshnessUnestablished,
    /// The whole-metadata schedule still requires a relay-admissible split.
    ///
    /// The historical item's eighty-six bytes exceed the reviewed
    /// eighty-byte relay bound. This obligation is carried exactly when
    /// the record's declared witness widths make the form review refuse
    /// relay admission. The variable schedule instead carries only the
    /// changing region and reconstructs the canonical encoding in the leaf.
    RelayAdmissibleWitnessSplitUnopened,
    /// Initial argument widths have not been measured over the linked program.
    ///
    /// This ABI cites the composed record's declared widths. The link's
    /// resource projection measures execution bounds, while admission checks
    /// the initial arguments. Measuring the linked bytes is what prevents a
    /// changed declaration from claiming a width the program does not use.
    InitialArgumentWidthsUnmeasuredOverLinkedProgram,
    /// No representation nonce was searched for a successor of an
    /// observed predecessor.
    ///
    /// Carried from the link in the link's own words. This derivation
    /// runs no search: leastness is a property of a scan, and the scan has
    /// one owner with one budget.
    SuccessorNonceSearchUndischarged,
    /// Nothing is finalized and no witness is populated.
    ///
    /// Carried from the link in the link's own words. This ABI states
    /// what a witness carries and builds none, which is what keeps it a
    /// candidate.
    FinalizationAndWitnessPopulationUndischarged,
    /// The relations the realization evaluates over a whole observed
    /// transaction are enforced by no leaf.
    ///
    /// Carried from the link in the link's own words, and untouched by
    /// anything here: re-scoping those relations to the region an
    /// operation claims is the realization's refit.
    ModelScopeRelationsUnenforced,
}

/// The outstanding announcement-ABI obligations, which are never none.
///
/// Structurally non-empty for the reason the link's set is: the least
/// obligation is a field of its own, so an ABI owing nothing has no
/// representation.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct OutstandingMaturityAbiObligations {
    least: MaturityAbiObligation,
    rest: BTreeSet<MaturityAbiObligation>,
}

impl OutstandingMaturityAbiObligations {
    /// Every outstanding obligation, in canonical order.
    pub fn obligations(&self) -> impl Iterator<Item = &MaturityAbiObligation> {
        std::iter::once(&self.least).chain(self.rest.iter())
    }

    /// How many are outstanding, which is never zero.
    ///
    /// Saturation is unreachable: the remaining members are held in a set
    /// of a finite variant enumeration while the least is held beside it.
    #[must_use]
    pub fn count(&self) -> NonZeroUsize {
        NonZeroUsize::MIN.saturating_add(self.rest.len())
    }

    /// Whether one obligation is outstanding.
    #[must_use]
    pub fn holds(&self, obligation: MaturityAbiObligation) -> bool {
        self.least == obligation || self.rest.contains(&obligation)
    }
}

/// What this derivation did about each obligation the link left standing.
///
/// A partition rather than a list, and both halves are named sets checked
/// against the link's own rather than derived from it. An obligation
/// quietly dropped between two waves is the failure this type exists to
/// make impossible; an obligation recorded in both halves is the other
/// one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InheritedStateLinkObligations {
    discharged: BTreeSet<StateLinkObligation>,
    carried: BTreeSet<StateLinkObligation>,
}

impl InheritedStateLinkObligations {
    /// The link obligations this derivation discharged.
    #[must_use]
    pub const fn discharged(&self) -> &BTreeSet<StateLinkObligation> {
        &self.discharged
    }

    /// The link obligations this derivation carries forward.
    #[must_use]
    pub const fn carried(&self) -> &BTreeSet<StateLinkObligation> {
        &self.carried
    }
}

/// The candidate maturity-announcement ABI (§12.9).
///
/// There is no digest, and no accessor through which one could be read:
///
/// ```compile_fail,E0599
/// use transaction::CandidateMaturityAnnouncementAbi;
/// fn identity(abi: &CandidateMaturityAnnouncementAbi) -> Vec<u8> {
///     abi.digest()
/// }
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CandidateMaturityAnnouncementAbi {
    schedule: StateWitnessSchedule,
    contract: TargetContractVersion,
    leaf_version: LeafVersion,
    inputs: MaturityInputLayout,
    outputs: MaturityOutputLayout,
    sequence: SequenceConstraint,
    version: TargetTransactionVersion,
    relay: MaturityRelayVerdict,
    witness_roles: Vec<MaturityWitnessRole>,
    inherited: InheritedStateLinkObligations,
    obligations: OutstandingMaturityAbiObligations,
}

impl CandidateMaturityAnnouncementAbi {
    /// Transport schedule retained by the composed record.
    #[must_use]
    pub const fn schedule(&self) -> StateWitnessSchedule {
        self.schedule
    }

    /// The reviewed contract revision this ABI is bound to.
    #[must_use]
    pub const fn contract(&self) -> TargetContractVersion {
        self.contract
    }

    /// The leaf version every committed leaf carries.
    #[must_use]
    pub const fn leaf_version(&self) -> LeafVersion {
        self.leaf_version
    }

    /// The input layout.
    #[must_use]
    pub const fn inputs(&self) -> MaturityInputLayout {
        self.inputs
    }

    /// The output layout.
    #[must_use]
    pub const fn outputs(&self) -> MaturityOutputLayout {
        self.outputs
    }

    /// The sequence constraint every input is built under.
    ///
    /// The compact generation's convention, used as it stands rather than
    /// re-minted: no announcement program inspects a sequence field or
    /// executes a relative timelock, so the constraint is the same
    /// convention about the same field and a second name for it would be
    /// a second thing to keep in step.
    #[must_use]
    pub const fn sequence(&self) -> SequenceConstraint {
        self.sequence
    }

    /// The transaction version an announcement is built at.
    ///
    /// The standard announcement version is independent of whether the
    /// selected witness schedule's declared widths produce a relay refusal
    /// or conditional admission. The relay verdict records that distinction.
    #[must_use]
    pub const fn version(&self) -> TargetTransactionVersion {
        self.version
    }

    /// The reviewed target's verdict on the sponsorless announcement form.
    ///
    /// The sponsored form's relay verdict remains the C2 observation obligation.
    #[must_use]
    pub const fn relay_verdict(&self) -> &MaturityRelayVerdict {
        &self.relay
    }

    /// The seven witness roles, in the record's deepest-first order.
    #[must_use]
    pub fn witness_roles(&self) -> &[MaturityWitnessRole] {
        &self.witness_roles
    }

    /// What this derivation did about each link obligation.
    #[must_use]
    pub const fn inherited_link_obligations(&self) -> &InheritedStateLinkObligations {
        &self.inherited
    }

    /// The obligations this ABI does not discharge.
    #[must_use]
    pub const fn outstanding_obligations(&self) -> &OutstandingMaturityAbiObligations {
        &self.obligations
    }

    /// This artifact's status (§12.9).
    ///
    /// Always [`MaturityAbiStatus::Candidate`], and read-only.
    #[must_use]
    pub const fn status(&self) -> MaturityAbiStatus {
        MaturityAbiStatus::Candidate
    }
}

/// Derive the candidate maturity-announcement ABI (§12.9).
///
/// The view is the evidence rather than a convenience: taking a validated
/// one is what lets the partition below discharge the reconstruction half
/// of the link's current-state obligation against a computation that
/// demonstrably ran.
///
/// # Errors
///
/// [`TransactionRefusal::BundleIsNotACandidate`] for a bundle claiming
/// more than a prototype link and
/// [`TransactionRefusal::ContractRevisionMismatch`] when the bundle's
/// policy and the target disagree about the reviewed revision — both
/// checked at validation too, and checked again because this function's
/// contract stands on its own rather than on what a caller did first;
/// [`TransactionRefusal::MissingReviewedTransactionForm`] when the
/// reviewed census publishes no verdict for the announcement form; and
/// [`TransactionRefusal::InheritedObligationUnaccounted`] when the two
/// halves of the partition are not exactly the link's own set.
pub fn derive_maturity_announcement_abi(
    target: &ReviewedElementsTapscriptDefinition,
    validated: &ValidatedMaturityStateView,
) -> Result<CandidateMaturityAnnouncementAbi, TransactionRefusal> {
    let bundle = validated.view().accepted_linked_bundle();
    if bundle.status() != LinkedArtifactStatus::Prototype {
        return Err(TransactionRefusal::BundleIsNotACandidate);
    }
    if bundle.policy().target_policy() != target.definition().version() {
        return Err(TransactionRefusal::ContractRevisionMismatch);
    }

    let inherited = disposition(bundle.obligations())?;
    let roles = witness_roles(bundle.record());
    let review = review_maturity_announcement_form(
        roles
            .iter()
            .map(|role| role.maximum_width().unwrap_or(usize::MAX)),
        target.definition().resources(),
    );
    let relay = MaturityRelayVerdict::from_review(review);
    let obligations = outstanding(&inherited, &relay);

    Ok(CandidateMaturityAnnouncementAbi {
        schedule: bundle.record().schedule(),
        contract: bundle.policy().target_policy(),
        leaf_version: bundle.policy().leaf_version(),
        inputs: MaturityInputLayout::stated(),
        outputs: MaturityOutputLayout::stated(),
        sequence: SequenceConstraint::FinalOnEveryInput,
        version: TargetTransactionVersion::Standard,
        relay,
        witness_roles: roles,
        inherited,
        obligations,
    })
}

/// What this derivation does about each obligation the link owes.
///
/// Two are discharged and three are carried, and the two halves are named
/// rather than one being computed as the other's complement. The
/// difference matters: a complement absorbs an obligation a later wave
/// adds, while a named pair leaves it in neither half and the comparison
/// below refuses.
fn disposition(
    owed: &OutstandingStateLinkObligations,
) -> Result<InheritedStateLinkObligations, TransactionRefusal> {
    // Discharged here: this derivation is the ABI the link recorded as
    // unsupplied, and the reconstruction half of the current-state
    // obligation is the computation a validated view carries the evidence
    // of. The freshness half of that obligation is restated as this ABI's
    // own narrower residual rather than left on the link.
    let discharged = BTreeSet::from([
        StateLinkObligation::CurrentStateValidationUndischarged,
        StateLinkObligation::AbiUnsupplied,
    ]);
    let carried = BTreeSet::from([
        StateLinkObligation::SuccessorNonceSearchUndischarged,
        StateLinkObligation::FinalizationAndWitnessPopulationUndischarged,
        StateLinkObligation::ModelScopeRelationsUnenforced,
    ]);
    if !discharged.is_disjoint(&carried) {
        return Err(TransactionRefusal::InheritedObligationUnaccounted);
    }

    let accounted: BTreeSet<_> = discharged.union(&carried).copied().collect();
    let owed: BTreeSet<_> = owed.obligations().copied().collect();
    if accounted != owed {
        return Err(TransactionRefusal::InheritedObligationUnaccounted);
    }

    Ok(InheritedStateLinkObligations {
        discharged,
        carried,
    })
}

/// The obligations an ABI over this partition carries.
///
/// The carried half is read from the partition rather than listed a
/// second time, so the two censuses cannot disagree about which link
/// obligation survived.
fn outstanding(
    inherited: &InheritedStateLinkObligations,
    relay: &MaturityRelayVerdict,
) -> OutstandingMaturityAbiObligations {
    let mut rest: BTreeSet<_> = inherited
        .carried()
        .iter()
        .copied()
        .filter_map(carried_member)
        .collect();
    if relay.relay() == FormAdmission::Refused {
        rest.insert(MaturityAbiObligation::RelayAdmissibleWitnessSplitUnopened);
    } else {
        rest.insert(MaturityAbiObligation::InitialArgumentWidthsUnmeasuredOverLinkedProgram);
    }

    OutstandingMaturityAbiObligations {
        least: MaturityAbiObligation::CurrentStateRootFreshnessUnestablished,
        rest,
    }
}

/// This vocabulary's word for one carried link obligation.
///
/// The two discharged obligations have no member here, which is what
/// makes the filter meaningful rather than lossy. An obligation neither
/// named here nor in the partition never reaches this function: the
/// partition refuses it first, which is why a member absent from this
/// mapping cannot be silently dropped from the outstanding set.
const fn carried_member(obligation: StateLinkObligation) -> Option<MaturityAbiObligation> {
    match obligation {
        StateLinkObligation::SuccessorNonceSearchUndischarged => {
            Some(MaturityAbiObligation::SuccessorNonceSearchUndischarged)
        }
        StateLinkObligation::FinalizationAndWitnessPopulationUndischarged => {
            Some(MaturityAbiObligation::FinalizationAndWitnessPopulationUndischarged)
        }
        StateLinkObligation::ModelScopeRelationsUnenforced => {
            Some(MaturityAbiObligation::ModelScopeRelationsUnenforced)
        }
        _ => None,
    }
}

/// The seven records, read from the composed record's own schedule.
fn witness_roles(record: &StateAnnouncementProgram) -> Vec<MaturityWitnessRole> {
    record
        .witness()
        .iter()
        .enumerate()
        .map(|(position, (role, declared))| {
            let width = declared_width(declared);
            MaturityWitnessRole {
                role: *role,
                declared: declared.clone(),
                minimum_width: width.map(|(minimum, _)| minimum),
                maximum_width: width.map(|(_, maximum)| maximum),
                classification: classification(*role),
                source: source(*role),
                consumer: state_witness_component(*role),
                program: StateLeafRole::Announcement,
                position,
            }
        })
        .collect()
}

/// The inclusive width range one declared stack type fixes, where it
/// fixes one.
///
/// A byte range states its own bounds. An encoded field states a class,
/// and the class's reviewed shape states the payload width, which is how
/// the operator's signature reports a width without this module knowing
/// what a signature is. Anything else states no width, and the absence is
/// reported rather than filled in.
const fn declared_width(declared: &StackValueType) -> Option<(usize, usize)> {
    match declared {
        StackValueType::Bytes { minimum, maximum } => Some((*minimum, *maximum)),
        StackValueType::Encoded(class) => match class.v1_shape().payload() {
            PayloadWidth::Absent => Some((0, 0)),
            PayloadWidth::Exact(width) => Some((width.get(), width.get())),
            PayloadWidth::Bounded { minimum, maximum } => Some((minimum, maximum.get())),
        },
        _ => None,
    }
}

/// Whether one role's item is public or needs a held key.
const fn classification(role: StateProgramWitness) -> WitnessClassification {
    match role {
        StateProgramWitness::OperatorSignature => WitnessClassification::SignerHeld,
        StateProgramWitness::SuccessorOutputKeyPrefix
        | StateProgramWitness::SuccessorNonce
        | StateProgramWitness::RequestedCycle
        | StateProgramWitness::StaticSubtreeRoot
        | StateProgramWitness::PredecessorMetadata
        | StateProgramWitness::PredecessorOutputKeyPrefix => WitnessClassification::Public,
    }
}

/// Which stage of construction supplies one role's item.
const fn source(role: StateProgramWitness) -> MaturityWitnessSource {
    match role {
        StateProgramWitness::SuccessorOutputKeyPrefix | StateProgramWitness::SuccessorNonce => {
            MaturityWitnessSource::SuccessorConstructorSearch
        }
        StateProgramWitness::RequestedCycle => MaturityWitnessSource::TypedRequest,
        StateProgramWitness::StaticSubtreeRoot => MaturityWitnessSource::LinkedStaticSubtree,
        StateProgramWitness::PredecessorMetadata => {
            MaturityWitnessSource::ValidatedViewPredecessorMetadata
        }
        StateProgramWitness::PredecessorOutputKeyPrefix => {
            MaturityWitnessSource::PredecessorCommitmentParity
        }
        StateProgramWitness::OperatorSignature => MaturityWitnessSource::OperatorSigning,
    }
}
