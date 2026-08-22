//! First-party negative evidence for the live-transfer safety matrix.
//!
//! §4.2 states what discharges a negative row whose boundary precedes the
//! target, and every clause of it removes a way of appearing to have
//! evidence: a canonical malformed typed input, the *exact owning*
//! validator, a typed refusal naming the intended class, a focused
//! positive control, a focused negative test, and a validated report
//! whose constructor is private. This module meets that policy for the
//! §15.3 rows §12.7 owns.
//!
//! # Why the control is half the evidence
//!
//! A validator that refuses everything refuses a malformed input too. The
//! argument only works because the malformed response set and the honest
//! one differ by exactly one thing and the *same call* accepts the
//! second: [`validate_live_first_party`] runs the owning entry point
//! twice per case and refuses to conclude anything if the control did not
//! pass.
//!
//! # What a discharge here is not
//!
//! Not a target verdict. §4.3 keeps three answers apart — the safe
//! constructor refused, an unsafe raw mutation exists, and the target was
//! not asked — and everything this module produces is the first of them.
//! A row discharged here still names the runtime relation it anticipates,
//! and the evidence plan records that the target was never asked about
//! it. A pre-target class is never submitted merely because an executor
//! is available.
//!
//! # Every secret here is published
//!
//! The scenarios sign with the BIP-340 appendix scalars through
//! [`crate::live_plan`], admitted under ADR-015's test-material rule
//! `(´[ADR015-rule:security:test-material]´)`. No interface here accepts
//! signing material from a caller, and the signatures are opaque bytes to
//! everything that reads them.

use std::collections::BTreeSet;

use linker::live_backend::LiveTransferRepresentationPlan;
use transaction::bytes::{AssetField, AssetId, Outpoint, TargetTransaction, Txid, ValueField};
use transaction::error::TransactionRefusal;
use transaction::live_abi::CandidateLiveTransferAbi;
use transaction::live_construct::finalize_live_transfer;
use transaction::live_finalize::FinalizedLiveTransfer;
use transaction::live_request::{
    LiveReceiptDestination, LiveTransferRequest, ProtocolValue, RequestedForm, SponsorChangeRequest,
};
use transaction::live_signing::{LiveOwnerResponse, authorize_live_transfer};
use transaction::view::{PublicConstructionView, PublicOutputView};

use crate::error::VectorError;
use crate::live_plan::{
    FIRST_SCALAR, PROTOCOL_ASSET, SECOND_SCALAR, demonstration_live_abi, published_owner,
    reviewed_target,
};
use crate::live_safety::{LiveSafetyRow, LiveSafetySection, rows_of};

/// Which first-party validator owns one §15 row.
///
/// A name for the exact entry point a discharge drove, carried into the
/// evidence so a reader of a coverage row can see *what* refused. There
/// is deliberately no arm for "the transaction crate", because that crate
/// refuses many things and a row answered by one of them is not answered
/// by the others.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum LiveFirstPartyValidator {
    /// `transaction::live_signing::authorize_live_transfer`, which
    /// collects owner responses and is the sole site constructing the
    /// seven collection-time refusals of §12.7.
    OwnerAuthorization,
    /// `FinalizedLiveTransfer::check_offered`, which compares an offered
    /// transaction against the finalized one and is the sole site
    /// constructing §12.7's three post-boundary refusals.
    OfferedTransactionCheck,
}

/// The scenario one case is staged in.
///
/// §1.6 keeps distinct semantic owners, concrete receipt inputs, and
/// concrete owner signatures apart, and three §15.3 rows differ only in
/// which of those three levels is short. Naming the scenario is what
/// keeps them three cases rather than one refusal counted three times.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum LiveOwnerScenario {
    /// One owner holding one receipt input.
    OneOwnerOneInput,
    /// Two distinct owners holding one receipt input each.
    TwoDistinctOwners,
    /// One owner holding two receipt inputs.
    ///
    /// The case a per-owner check passes and a per-input check fails.
    RepeatedOwner,
}

impl LiveOwnerScenario {
    /// Every scenario.
    pub const ALL: &'static [Self] = &[
        Self::OneOwnerOneInput,
        Self::TwoDistinctOwners,
        Self::RepeatedOwner,
    ];

    /// How many receipt inputs the scenario consumes.
    #[must_use]
    pub const fn receipt_inputs(self) -> usize {
        match self {
            Self::OneOwnerOneInput => 1,
            Self::TwoDistinctOwners | Self::RepeatedOwner => 2,
        }
    }

    /// How many distinct semantic owners it represents.
    #[must_use]
    pub const fn distinct_owners(self) -> usize {
        match self {
            Self::OneOwnerOneInput | Self::RepeatedOwner => 1,
            Self::TwoDistinctOwners => 2,
        }
    }
}

/// The one focused change a case makes to the honest offering.
///
/// Each arm is a single disagreement with the request the builder issued,
/// which is what makes the refusal attributable: an offering that
/// differed in two places would be refused for whichever the validator
/// reached first.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum LiveResponseMalformation {
    /// One required response is left out entirely.
    OmitOneResponse,
    /// One position is answered twice.
    AnswerOnePositionTwice,
    /// A response is offered at a position that is not a receipt input.
    AnswerAPositionThatIsNotAReceipt,
    /// A response claims an owner the input does not authenticate.
    ClaimAnotherOwner,
    /// A response answers a different request than its position.
    AnswerAnotherRequest,
    /// A response commits to dimensions the selected profile does not
    /// require.
    CommitUnderAnotherProfile,
    /// A response was taken over another transaction's exact bytes.
    BindToAnotherTransaction,
    /// An output of the offered transaction differs from the finalized
    /// one.
    ChangeAnOutputAfterSigning,
    /// The offered transaction carries an input the finalized one does
    /// not.
    AddAnInputAfterSigning,
    /// The offered transaction is missing one finalized output.
    RemoveAnOutputAfterSigning,
}

/// One first-party negative case: a §15 row, a scenario, and one change.
#[derive(Clone, Debug)]
pub struct LiveFirstPartyCase {
    row: &'static str,
    scenario: LiveOwnerScenario,
    malformation: LiveResponseMalformation,
    validator: LiveFirstPartyValidator,
    expected: fn(&TransactionRefusal) -> bool,
    expected_name: &'static str,
}

impl PartialEq for LiveFirstPartyCase {
    /// Compared by what a reader can check, never by function address.
    ///
    /// The expected-class predicate travels as a function pointer, whose
    /// address is not a meaningful identity; the class *name* beside it
    /// is, and it is the value a report renders anyway.
    fn eq(&self, other: &Self) -> bool {
        self.row == other.row
            && self.scenario == other.scenario
            && self.malformation == other.malformation
            && self.validator == other.validator
            && self.expected_name == other.expected_name
    }
}

impl Eq for LiveFirstPartyCase {}

impl LiveFirstPartyCase {
    /// The §15.3 row this case answers.
    #[must_use]
    pub const fn row(&self) -> &'static str {
        self.row
    }

    /// The scenario the case is staged in.
    #[must_use]
    pub const fn scenario(&self) -> LiveOwnerScenario {
        self.scenario
    }

    /// The one change the case makes.
    #[must_use]
    pub const fn malformation(&self) -> LiveResponseMalformation {
        self.malformation
    }

    /// Which entry point owns the refusal.
    #[must_use]
    pub const fn validator(&self) -> LiveFirstPartyValidator {
        self.validator
    }

    /// The refusal class the case expects, by name.
    #[must_use]
    pub const fn expected_class(&self) -> &'static str {
        self.expected_name
    }
}

/// Why one offered case does not discharge its row.
///
/// Every arm is a way the offered evidence falls short of §4.2, and none
/// of them is a defect in the validator: a validator that refused the
/// control has said something true about the control, and what fails is
/// the claim that the refusal was about the malformation.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum LiveFirstPartyRefusal {
    /// §15 names no such row.
    RowNotInTheMatrix(&'static str),
    /// The row's boundary is the target's, so a first-party refusal is
    /// not what answers it.
    RowIsNotFirstParty(&'static str),
    /// The live-transfer substrate could not be built.
    SubstrateUnavailable,
    /// The honest scenario could not be finalized.
    ///
    /// Then there is nothing to malform, and a refusal of the malformed
    /// offering would say nothing about the builder.
    ScenarioNotConstructible,
    /// The malformation left the offering unchanged.
    ///
    /// The whole argument is that the two offerings differ by exactly one
    /// thing; two identical offerings differ by nothing, and the
    /// validator would have to answer them the same way.
    MalformationChangedNothing,
    /// The validator accepted the malformed offering.
    MalformedOfferingWasAccepted,
    /// The validator refused, naming a class other than the row's.
    RefusalNamesAnotherClass(TransactionRefusal),
    /// The validator refused the control too.
    ///
    /// Then the refusal is not attributable to the malformation: the
    /// offering was unacceptable before it was malformed.
    ControlWasRefused(TransactionRefusal),
}

impl From<VectorError> for LiveFirstPartyRefusal {
    fn from(_: VectorError) -> Self {
        Self::SubstrateUnavailable
    }
}

/// One first-party negative row, discharged.
///
/// # What holding one of these establishes
///
/// That the exact entry point the row's boundary belongs to was run twice
/// — once on an offering malformed in one stated place and once on the
/// honest offering that malformation is a single change of — that it
/// refused the first naming the row's own class, and that it accepted the
/// second. All of that is recomputed by [`validate_live_first_party`],
/// which is the only constructor; every field here is a conclusion of
/// that run and none of them is a value a caller offered.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedLiveFirstPartyEvidence {
    row: &'static str,
    scenario: LiveOwnerScenario,
    malformation: LiveResponseMalformation,
    validator: LiveFirstPartyValidator,
    observed: TransactionRefusal,
}

impl ValidatedLiveFirstPartyEvidence {
    /// The §15 row this evidence answers.
    #[must_use]
    pub const fn row(&self) -> &'static str {
        self.row
    }

    /// The scenario the discharge was staged in.
    #[must_use]
    pub const fn scenario(&self) -> LiveOwnerScenario {
        self.scenario
    }

    /// The one change the discharge made.
    #[must_use]
    pub const fn malformation(&self) -> LiveResponseMalformation {
        self.malformation
    }

    /// Which entry point refused.
    #[must_use]
    pub const fn validator(&self) -> LiveFirstPartyValidator {
        self.validator
    }

    /// The refusal the validator actually returned.
    #[must_use]
    pub const fn observed(&self) -> &TransactionRefusal {
        &self.observed
    }
}

/// The outpoint of `index` of a transaction whose identifier is `byte`
/// repeated.
fn outpoint(byte: u8, index: u32) -> Result<Outpoint, VectorError> {
    Outpoint::new(Txid::from_internal([byte; 32]), index)
        .map_err(|_| VectorError::LiveSubstrateUnavailable)
}

/// A public view of one owner's explicit live receipt.
fn receipt_view(
    abi: &CandidateLiveTransferAbi,
    point: Outpoint,
    scalar: &[u8; 32],
    amount: u64,
) -> Result<PublicOutputView, VectorError> {
    Ok(PublicOutputView::new(
        point,
        AssetField::Explicit(AssetId::from_internal(PROTOCOL_ASSET)),
        ValueField::Explicit(amount),
        abi.destinations()
            .get(
                &linker::OwnerParameter::new(published_owner(scalar)?),
                LiveTransferRepresentationPlan::Explicit,
            )
            .ok_or(VectorError::LiveSubstrateUnavailable)?
            .instance()
            .program()
            .to_vec(),
    ))
}

/// One destination for a published owner at `amount`.
fn destination(scalar: &[u8; 32], amount: u64) -> Result<LiveReceiptDestination, VectorError> {
    Ok(LiveReceiptDestination::new(
        linker::OwnerParameter::new(published_owner(scalar)?),
        ProtocolValue::new(amount).map_err(|_| VectorError::LiveSubstrateUnavailable)?,
    ))
}

/// The finalized transfer one scenario determines.
///
/// The scenario fixes which owners hold which receipts; the destinations
/// are two receipts of one owner each, so every scenario finalizes to a
/// transfer with the same aggregate and a fixed destination census.
fn finalize(
    scenario: LiveOwnerScenario,
    identifier: u8,
) -> Result<FinalizedLiveTransfer, VectorError> {
    let target = reviewed_target()?;
    let abi = demonstration_live_abi()?;

    let owners: &[&[u8; 32]] = match scenario {
        LiveOwnerScenario::OneOwnerOneInput => &[&FIRST_SCALAR],
        LiveOwnerScenario::TwoDistinctOwners => &[&FIRST_SCALAR, &SECOND_SCALAR],
        LiveOwnerScenario::RepeatedOwner => &[&FIRST_SCALAR, &FIRST_SCALAR],
    };

    let mut points = Vec::with_capacity(owners.len());
    let mut views = Vec::with_capacity(owners.len());
    for (index, scalar) in owners.iter().enumerate() {
        let index = u32::try_from(index).map_err(|_| VectorError::LiveSubstrateUnavailable)?;
        let point = outpoint(identifier, index)?;
        points.push(point);
        views.push(receipt_view(&abi, point, scalar, 500)?);
    }

    let view =
        PublicConstructionView::new(views).map_err(|_| VectorError::LiveSubstrateUnavailable)?;
    let total =
        500 * u64::try_from(owners.len()).map_err(|_| VectorError::LiveSubstrateUnavailable)?;
    let request = LiveTransferRequest::new(
        points,
        [
            destination(&SECOND_SCALAR, total / 2)?,
            destination(&FIRST_SCALAR, total - total / 2)?,
        ],
        LiveTransferRepresentationPlan::Explicit,
        RequestedForm::Sponsorless,
        SponsorChangeRequest::NotRequested,
        None,
    )
    .map_err(|_| VectorError::LiveSubstrateUnavailable)?;

    Ok(
        finalize_live_transfer(&target, &abi, &request, &view, None, None)
            .map_err(|_| VectorError::LiveSubstrateUnavailable)?
            .into_finalized(),
    )
}

/// A fixed opaque signature.
///
/// Opaque bytes, and deliberately not a signature anybody took: §12.7's
/// refusals are about *binding* — which input, which owner, which
/// profile, which bytes — and none of them reads the signature. A real
/// signature here would suggest the refusals depended on one.
const OPAQUE_SIGNATURE: [u8; 64] = [0x5c; 64];

/// The honest response set for one finalized transfer.
fn honest_responses(finalized: &FinalizedLiveTransfer) -> Vec<(u16, LiveOwnerResponse)> {
    finalized
        .signing_requests()
        .iter()
        .map(|signing| {
            (
                signing.input(),
                LiveOwnerResponse::to(signing, OPAQUE_SIGNATURE.to_vec()),
            )
        })
        .collect()
}

/// Apply one malformation to an honest response set.
///
/// `None` for a malformation whose subject is the offered transaction
/// rather than the response set.
fn malform_responses(
    finalized: &FinalizedLiveTransfer,
    honest: &[(u16, LiveOwnerResponse)],
    malformation: LiveResponseMalformation,
    other_bytes: &[u8],
) -> Option<Vec<(u16, LiveOwnerResponse)>> {
    use LiveResponseMalformation as M;

    let mut offered = honest.to_vec();
    let (position, first) = offered.first()?.clone();
    match malformation {
        M::OmitOneResponse => {
            offered.pop()?;
        }
        M::AnswerOnePositionTwice => offered.push((position, first)),
        // A position past every receipt input. The finalized census fixes
        // how many there are, so the position is derived rather than
        // guessed at a number that might one day be a real input.
        M::AnswerAPositionThatIsNotAReceipt => {
            let beyond = u16::try_from(finalized.receipts().len()).ok()? + 100;
            offered.push((beyond, first));
        }
        M::ClaimAnotherOwner => {
            let stranger = crate::live_plan::published_owner(&SECOND_SCALAR).ok()?;
            let stranger = linker::OwnerParameter::new(stranger);
            // Only useful where the input authenticates somebody else,
            // which the scenario is chosen to guarantee.
            if first.owner() == &stranger {
                return None;
            }
            offered[0] = (
                position,
                LiveOwnerResponse::from_parts(
                    first.input(),
                    stranger,
                    first.representation(),
                    first.committed_dimensions().clone(),
                    first.bound_to().to_vec(),
                    first.signature().to_vec(),
                ),
            );
        }
        M::AnswerAnotherRequest => {
            offered[0] = (
                position,
                LiveOwnerResponse::from_parts(
                    first.input().checked_add(1)?,
                    first.owner().clone(),
                    first.representation(),
                    first.committed_dimensions().clone(),
                    first.bound_to().to_vec(),
                    first.signature().to_vec(),
                ),
            );
        }
        M::CommitUnderAnotherProfile => {
            offered[0] = (
                position,
                LiveOwnerResponse::from_parts(
                    first.input(),
                    first.owner().clone(),
                    first.representation(),
                    BTreeSet::new(),
                    first.bound_to().to_vec(),
                    first.signature().to_vec(),
                ),
            );
        }
        M::BindToAnotherTransaction => {
            if other_bytes == first.bound_to() {
                return None;
            }
            offered[0] = (
                position,
                LiveOwnerResponse::from_parts(
                    first.input(),
                    first.owner().clone(),
                    first.representation(),
                    first.committed_dimensions().clone(),
                    other_bytes.to_vec(),
                    first.signature().to_vec(),
                ),
            );
        }
        M::ChangeAnOutputAfterSigning
        | M::AddAnInputAfterSigning
        | M::RemoveAnOutputAfterSigning => return None,
    }
    Some(offered)
}

/// Apply one malformation to the offered transaction.
///
/// `None` for a malformation whose subject is the response set.
fn malform_offered(
    finalized: &FinalizedLiveTransfer,
    malformation: LiveResponseMalformation,
) -> Option<TargetTransaction> {
    use LiveResponseMalformation as M;

    let honest = TargetTransaction::decode(finalized.protected_bytes()).ok()?;
    let mut inputs = honest.inputs().to_vec();
    let mut outputs = honest.outputs().to_vec();
    // The witness census travels with the input census, because the
    // decoder requires one record per input. Carrying the honest one
    // keeps the offered transaction a single change of the finalized
    // one rather than two.
    let mut witnesses = honest.witnesses().to_vec();
    match malformation {
        M::ChangeAnOutputAfterSigning => {
            let first = outputs.first()?.clone();
            let ValueField::Explicit(amount) = first.value() else {
                return None;
            };
            outputs[0] = transaction::bytes::TargetOutput::new(
                first.asset(),
                ValueField::Explicit(amount.checked_add(1)?),
                first.nonce(),
                first.program().to_vec(),
            );
        }
        M::AddAnInputAfterSigning => {
            inputs.push(inputs.first()?.clone());
            witnesses.push(witnesses.first()?.clone());
        }
        M::RemoveAnOutputAfterSigning => {
            outputs.pop()?;
        }
        _ => return None,
    }
    TargetTransaction::new(
        honest.version(),
        inputs,
        outputs,
        honest.lock_time(),
        witnesses,
    )
    .ok()
}

/// The `OwnerResponseMissing` predicate, spelled once.
const fn response_missing(refusal: &TransactionRefusal) -> bool {
    matches!(refusal, TransactionRefusal::OwnerResponseMissing { .. })
}

/// One `case` constructor, shared by the two halves of the census.
const fn case(
    row: &'static str,
    scenario: LiveOwnerScenario,
    malformation: LiveResponseMalformation,
    validator: LiveFirstPartyValidator,
    expected: fn(&TransactionRefusal) -> bool,
    expected_name: &'static str,
) -> LiveFirstPartyCase {
    LiveFirstPartyCase {
        row,
        scenario,
        malformation,
        validator,
        expected,
        expected_name,
    }
}

/// The seven §12.7 rejections answered while responses are collected.
///
/// Nine cases over seven refusals, because three §15.3 rows — a missing
/// signature, one omitted owner, and a repeated owner with one witness
/// omitted — reach the same refusal from three different scenarios, and
/// §1.6 is the reason they are three rows rather than one counted three
/// times.
#[must_use]
pub fn collection_time_cases() -> Vec<LiveFirstPartyCase> {
    use LiveFirstPartyValidator as V;
    use LiveOwnerScenario as Scenario;
    use LiveResponseMalformation as M;

    vec![
        case(
            "missing-owner-signature",
            Scenario::OneOwnerOneInput,
            M::OmitOneResponse,
            V::OwnerAuthorization,
            response_missing,
            "OwnerResponseMissing",
        ),
        case(
            "one-omitted-owner",
            Scenario::TwoDistinctOwners,
            M::OmitOneResponse,
            V::OwnerAuthorization,
            response_missing,
            "OwnerResponseMissing",
        ),
        case(
            "repeated-owner-with-one-input-signature-omitted",
            Scenario::RepeatedOwner,
            M::OmitOneResponse,
            V::OwnerAuthorization,
            response_missing,
            "OwnerResponseMissing",
        ),
        case(
            "duplicated-signature-substituted-for-another-owner",
            Scenario::TwoDistinctOwners,
            M::AnswerOnePositionTwice,
            V::OwnerAuthorization,
            |refusal| matches!(refusal, TransactionRefusal::OwnerResponseDuplicated { .. }),
            "OwnerResponseDuplicated",
        ),
        case(
            "unrelated-signer",
            Scenario::TwoDistinctOwners,
            M::AnswerAPositionThatIsNotAReceipt,
            V::OwnerAuthorization,
            |refusal| matches!(refusal, TransactionRefusal::UnexpectedSigner { .. }),
            "UnexpectedSigner",
        ),
        case(
            "wrong-owner",
            Scenario::OneOwnerOneInput,
            M::ClaimAnotherOwner,
            V::OwnerAuthorization,
            |refusal| matches!(refusal, TransactionRefusal::ResponseFromWrongOwner { .. }),
            "ResponseFromWrongOwner",
        ),
        case(
            "wrong-sighash-profile",
            Scenario::OneOwnerOneInput,
            M::CommitUnderAnotherProfile,
            V::OwnerAuthorization,
            |refusal| {
                matches!(
                    refusal,
                    TransactionRefusal::ResponseUnderWrongSighashProfile { .. }
                )
            },
            "ResponseUnderWrongSighashProfile",
        ),
        case(
            "signature-over-another-transaction",
            Scenario::OneOwnerOneInput,
            M::BindToAnotherTransaction,
            V::OwnerAuthorization,
            |refusal| {
                matches!(
                    refusal,
                    TransactionRefusal::ResponseBoundToDifferentBytes { .. }
                )
            },
            "ResponseBoundToDifferentBytes",
        ),
    ]
}

/// The three §12.7 rejections answered against an offered transaction.
///
/// Separate from the collection-time half because they are things a
/// *builder* does to the bytes after signing rather than things a signer
/// does to a response, and a different entry point owns them.
#[must_use]
pub fn post_boundary_cases() -> Vec<LiveFirstPartyCase> {
    use LiveFirstPartyValidator as V;
    use LiveOwnerScenario as Scenario;
    use LiveResponseMalformation as M;

    vec![
        case(
            "output-changed-after-signing",
            Scenario::TwoDistinctOwners,
            M::ChangeAnOutputAfterSigning,
            V::OfferedTransactionCheck,
            |refusal| {
                matches!(
                    refusal,
                    TransactionRefusal::OutputMutatedAfterSigning { .. }
                )
            },
            "OutputMutatedAfterSigning",
        ),
        case(
            "input-added-after-signing",
            Scenario::TwoDistinctOwners,
            M::AddAnInputAfterSigning,
            V::OfferedTransactionCheck,
            |refusal| {
                matches!(
                    refusal,
                    TransactionRefusal::InputExtendedAfterSigning { .. }
                )
            },
            "InputExtendedAfterSigning",
        ),
        case(
            "output-removed-after-signing",
            Scenario::TwoDistinctOwners,
            M::RemoveAnOutputAfterSigning,
            V::OfferedTransactionCheck,
            |refusal| {
                matches!(
                    refusal,
                    TransactionRefusal::OutputOmittedAfterSigning { .. }
                )
            },
            "OutputOmittedAfterSigning",
        ),
    ]
}

/// The complete census of first-party cases for §15.3.
///
/// Twelve cases over §12.7's ten refusals, in the order the two halves
/// are declared.
#[must_use]
pub fn live_first_party_cases() -> Vec<LiveFirstPartyCase> {
    let mut cases = collection_time_cases();
    cases.extend(post_boundary_cases());
    cases
}

/// The §15 row one case names, if the matrix has it.
fn matrix_row(name: &str) -> Option<&'static LiveSafetyRow> {
    rows_of(LiveSafetySection::OwnerSignatureFault)
        .iter()
        .find(|row| row.name() == name)
}

/// Discharge one first-party negative row, per §4.2.
///
/// The owning entry point is run twice: once on the honest offering,
/// which must be accepted, and once on the offering malformed in the
/// case's one stated place, which must be refused naming the case's own
/// class.
///
/// # Errors
///
/// [`LiveFirstPartyRefusal`], naming which of §4.2's conditions the
/// offered case does not meet.
pub fn validate_live_first_party(
    case: &LiveFirstPartyCase,
) -> Result<ValidatedLiveFirstPartyEvidence, LiveFirstPartyRefusal> {
    let row = matrix_row(case.row).ok_or(LiveFirstPartyRefusal::RowNotInTheMatrix(case.row))?;
    if !row.is_first_party() {
        return Err(LiveFirstPartyRefusal::RowIsNotFirstParty(case.row));
    }

    let finalized = finalize(case.scenario, 0xa1)?;
    let honest = honest_responses(&finalized);

    let observed = match case.validator {
        LiveFirstPartyValidator::OwnerAuthorization => {
            // A second finalized transfer, for the one case that needs
            // another transaction's exact bytes to bind to. It differs
            // from the first only in which outpoints it consumes.
            let other = finalize(case.scenario, 0xa2)?;
            let malformed = malform_responses(
                &finalized,
                &honest,
                case.malformation,
                other.protected_bytes(),
            )
            .ok_or(LiveFirstPartyRefusal::MalformationChangedNothing)?;
            if malformed == honest {
                return Err(LiveFirstPartyRefusal::MalformationChangedNothing);
            }

            // The control first: a refusal of the honest offering would
            // make every later conclusion about the wrong thing.
            authorize_live_transfer(finalized.clone(), honest)
                .map_err(LiveFirstPartyRefusal::ControlWasRefused)?;
            authorize_live_transfer(finalized, malformed)
                .err()
                .ok_or(LiveFirstPartyRefusal::MalformedOfferingWasAccepted)?
        }
        LiveFirstPartyValidator::OfferedTransactionCheck => {
            let authorized = authorize_live_transfer(finalized.clone(), honest)
                .map_err(LiveFirstPartyRefusal::ControlWasRefused)?;
            let control = TargetTransaction::decode(finalized.protected_bytes())
                .map_err(|_| LiveFirstPartyRefusal::ScenarioNotConstructible)?;
            let offered = malform_offered(&finalized, case.malformation)
                .ok_or(LiveFirstPartyRefusal::MalformationChangedNothing)?;
            if offered == control {
                return Err(LiveFirstPartyRefusal::MalformationChangedNothing);
            }

            authorized
                .check_offered(&control)
                .map_err(LiveFirstPartyRefusal::ControlWasRefused)?;
            authorized
                .check_offered(&offered)
                .err()
                .ok_or(LiveFirstPartyRefusal::MalformedOfferingWasAccepted)?
        }
    };

    if !(case.expected)(&observed) {
        return Err(LiveFirstPartyRefusal::RefusalNamesAnotherClass(observed));
    }
    Ok(ValidatedLiveFirstPartyEvidence {
        row: case.row,
        scenario: case.scenario,
        malformation: case.malformation,
        validator: case.validator,
        observed,
    })
}

/// Every §15.3 row this census discharges.
///
/// # Errors
///
/// Whatever [`validate_live_first_party`] refuses, on the first case that
/// does not meet §4.2.
pub fn discharge_live_first_party()
-> Result<Vec<ValidatedLiveFirstPartyEvidence>, LiveFirstPartyRefusal> {
    live_first_party_cases()
        .iter()
        .map(validate_live_first_party)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{
        LiveFirstPartyValidator, LiveOwnerScenario, discharge_live_first_party, finalize,
        live_first_party_cases, matrix_row,
    };
    use std::collections::BTreeSet;

    #[test]
    fn every_case_names_a_first_party_row_of_the_matrix() {
        // A case naming a row §15 does not have, or naming one whose
        // boundary is the target's, would be evidence filed against
        // nothing. The check is here as well as inside the discharge so
        // that a mis-filed case fails without needing a node.
        for case in live_first_party_cases() {
            let row = matrix_row(case.row())
                .unwrap_or_else(|| panic!("{} is not a §15.3 row", case.row()));
            assert!(row.is_first_party(), "{row} is not answered first-party");
        }
    }

    #[test]
    fn the_three_scenarios_differ_in_the_levels_that_matter() {
        // §1.6's distinction, made a property of the scenarios rather
        // than of a comment: a repeated owner and two distinct owners
        // consume the same number of inputs and represent different
        // numbers of owners, which is exactly what separates two of the
        // three rows that reach the same refusal.
        let finalized: Vec<_> = LiveOwnerScenario::ALL
            .iter()
            .map(|scenario| {
                (
                    *scenario,
                    finalize(*scenario, 0xa1).expect("the scenario finalizes"),
                )
            })
            .collect();
        for (scenario, transfer) in &finalized {
            assert_eq!(transfer.receipts().len(), scenario.receipt_inputs());
            let owners: BTreeSet<_> = transfer
                .receipts()
                .iter()
                .map(|record| record.owner().clone())
                .collect();
            assert_eq!(owners.len(), scenario.distinct_owners());
        }
        assert_eq!(
            LiveOwnerScenario::RepeatedOwner.receipt_inputs(),
            LiveOwnerScenario::TwoDistinctOwners.receipt_inputs(),
        );
        assert_ne!(
            LiveOwnerScenario::RepeatedOwner.distinct_owners(),
            LiveOwnerScenario::TwoDistinctOwners.distinct_owners(),
        );
    }

    #[test]
    fn every_case_discharges_its_row_against_its_own_control() {
        // §4.2 performed, over the whole census. Each case runs the
        // owning entry point twice and must be refused naming its own
        // class while the honest offering is accepted; a case whose
        // refusal came from somewhere else fails here rather than being
        // filed as coverage.
        let discharged: Vec<_> = live_first_party_cases()
            .iter()
            .map(|case| {
                super::validate_live_first_party(case)
                    .unwrap_or_else(|refusal| panic!("{case:?} did not meet §4.2: {refusal:?}"))
            })
            .collect();
        assert_eq!(discharged.len(), live_first_party_cases().len());
        assert_eq!(
            discharge_live_first_party().expect("the census discharges"),
            discharged,
        );

        // Three rows reach the same refusal from three scenarios, and
        // they stay three rows. A census that had collapsed them would
        // report one discharge where §1.6 requires three.
        let missing: BTreeSet<_> = discharged
            .iter()
            .filter(|evidence| {
                matches!(
                    evidence.observed(),
                    transaction::error::TransactionRefusal::OwnerResponseMissing { .. }
                )
            })
            .map(super::ValidatedLiveFirstPartyEvidence::scenario)
            .collect();
        assert_eq!(missing.len(), 3, "the three levels collapsed into fewer");

        // And both owning entry points were really driven, so a census
        // that quietly lost the post-boundary half would fail rather
        // than report a smaller matrix.
        let validators: BTreeSet<_> = discharged
            .iter()
            .map(super::ValidatedLiveFirstPartyEvidence::validator)
            .collect();
        assert_eq!(
            validators,
            BTreeSet::from([
                LiveFirstPartyValidator::OwnerAuthorization,
                LiveFirstPartyValidator::OfferedTransactionCheck,
            ]),
        );
    }
}
