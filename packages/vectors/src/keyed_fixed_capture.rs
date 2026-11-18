//! Fixed-form capture packet admission before archive writing.
//! Native observations are separate from the packet's claimed identity.

use crate::keyed_fixed_manifest::ValidatedFreeze;
use crate::keyed_fixed_native::{ObservedFreeze, bind_structural_subject};
use serde::{Deserialize, Serialize};
use target_elements_conformance::constructor::tagged::sha256;
use target_elements_conformance::protocol::{
    NATIVE_PROTOCOL_SCHEMA, NativeOperationRequest, NativeOperationResponse, ObservedOutcomeLayer,
    OperationStepKind, OperationSubject,
};
use transaction::{AssetField, TargetTransaction, ValueField};

/// Operator family; the legacy control is retained outside fixed-form admission.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CaptureFamily {
    Pilot,
    Clauses,
    Policy,
    Scaling,
    Histories,
    LegacyCopy,
}

/// The reviewed finite subject census, before any node observation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CaptureSubject {
    PublicSource(u8),
    OneMemberMerge(u8),
    TwoMemberMerge(u8),
    SurplusDestination(u8),
    ValueMinus,
    ValuePlus,
    ConfidentialValue,
    MalformedPaymentValue,
    MalformedPaymentValueForm,
    LaterPosition,
    WrongProgram,
    WrongKey,
    DisplacedSuccessor,
    MalformedControl,
    KeyPathAttempt,
    Floor(u8),
    ZeroFeeFloor,
    PresentNonceFloor(u8),
    WeightSource,
    WeightMerge(u16),
    StructuralWeight1300,
    DirectBlock1300,
    HistoryLinear(u8),
    HistoryBranch(u8),
    HistoryMissingBranch,
    LegacyConfidentialValue,
    LegacyPublicCopy,
}

/// Expected observation site; a node diagnostic alone cannot select an opcode.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CaptureBoundary {
    Acceptance,
    OutputPair,
    InputPair,
    BirthFirst,
    Continuation,
    Commitment,
    KeyPath,
    TypedConstruction,
    RelayPolicy,
    FeePolicy,
    Consensus,
    IncompleteHistory,
}

impl CaptureSubject {
    /// A prediction from the fixed-form design, never a substitute for observation.
    #[must_use]
    pub fn boundary(self) -> Option<CaptureBoundary> {
        if ![
            CaptureFamily::Pilot,
            CaptureFamily::Clauses,
            CaptureFamily::Policy,
            CaptureFamily::Scaling,
            CaptureFamily::Histories,
            CaptureFamily::LegacyCopy,
        ]
        .into_iter()
        .any(|family| subjects(family).contains(&self))
        {
            return None;
        }
        Some(match self {
            Self::ValueMinus | Self::ValuePlus | Self::ConfidentialValue => {
                CaptureBoundary::OutputPair
            }
            Self::MalformedPaymentValue | Self::MalformedPaymentValueForm => {
                CaptureBoundary::InputPair
            }
            Self::LaterPosition => CaptureBoundary::BirthFirst,
            Self::WrongProgram | Self::WrongKey | Self::DisplacedSuccessor => {
                CaptureBoundary::Continuation
            }
            Self::MalformedControl => CaptureBoundary::Commitment,
            Self::KeyPathAttempt => CaptureBoundary::KeyPath,
            Self::WeightMerge(1300) => CaptureBoundary::TypedConstruction,
            Self::Floor(14) | Self::StructuralWeight1300 => CaptureBoundary::RelayPolicy,
            Self::ZeroFeeFloor => CaptureBoundary::FeePolicy,
            Self::DirectBlock1300 => CaptureBoundary::Consensus,
            Self::HistoryMissingBranch => CaptureBoundary::IncompleteHistory,
            Self::PublicSource(_)
            | Self::OneMemberMerge(_)
            | Self::TwoMemberMerge(_)
            | Self::SurplusDestination(_)
            | Self::Floor(_)
            | Self::PresentNonceFloor(_)
            | Self::WeightSource
            | Self::WeightMerge(_)
            | Self::HistoryLinear(_)
            | Self::HistoryBranch(_)
            | Self::LegacyConfidentialValue
            | Self::LegacyPublicCopy => CaptureBoundary::Acceptance,
        })
    }
}

/// Enumerate exactly the selected subjects for one operator family.
#[must_use]
pub fn subjects(family: CaptureFamily) -> Vec<CaptureSubject> {
    use CaptureSubject as S;
    match family {
        CaptureFamily::Pilot => vec![
            S::PublicSource(42),
            S::OneMemberMerge(42),
            S::Floor(15),
            S::Floor(14),
            S::WeightMerge(252),
        ],
        CaptureFamily::Clauses => vec![
            S::PublicSource(42),
            S::PublicSource(43),
            S::OneMemberMerge(42),
            S::OneMemberMerge(43),
            S::TwoMemberMerge(42),
            S::TwoMemberMerge(43),
            S::SurplusDestination(0),
            S::SurplusDestination(1),
            S::ValueMinus,
            S::ValuePlus,
            S::MalformedPaymentValue,
            S::LaterPosition,
            S::WrongProgram,
            S::WrongKey,
            S::DisplacedSuccessor,
            S::MalformedControl,
            S::KeyPathAttempt,
        ],
        CaptureFamily::Policy => {
            vec![S::Floor(15), S::Floor(14), S::ZeroFeeFloor]
        }
        CaptureFamily::Scaling => vec![
            // Each MERGE journals its separately funded member SOURCEs.
            S::PublicSource(42),
            S::WeightSource,
            S::WeightMerge(1),
            S::WeightMerge(2),
            S::WeightMerge(251),
            S::WeightMerge(252),
            S::WeightMerge(253),
            S::WeightMerge(650),
            S::WeightMerge(1299),
            S::WeightMerge(1300),
            S::StructuralWeight1300,
        ],
        CaptureFamily::Histories => vec![
            S::HistoryLinear(4),
            S::HistoryBranch(4),
            S::HistoryLinear(16),
            S::HistoryBranch(16),
            S::HistoryLinear(64),
            S::HistoryBranch(64),
            S::HistoryMissingBranch,
        ],
        CaptureFamily::LegacyCopy => Vec::new(),
    }
}

/// Capture-owned event graph; parent indices name earlier event positions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkloadShape {
    pub parents: Vec<Vec<usize>>,
    pub funding_edges: usize,
}

impl WorkloadShape {
    /// Number of accepted history events in the planned graph.
    #[must_use]
    pub const fn events(&self) -> usize {
        self.parents.len()
    }
    /// Input edges including independently funded SOURCE inputs.
    #[must_use]
    pub fn edges(&self) -> usize {
        self.funding_edges + self.parents.iter().map(Vec::len).sum::<usize>()
    }
}

/// Reproduce S7's linear or branch/join/extension recurrence at selected sizes.
/// Empty shape means an unselected size, never a claim of a measured run.
#[must_use]
pub fn workload_shape(events: usize, branching: bool) -> WorkloadShape {
    if ![4, 16, 64].contains(&events) {
        return WorkloadShape {
            parents: vec![],
            funding_edges: 0,
        };
    }
    if !branching {
        return WorkloadShape {
            parents: (0..events)
                .map(|index| if index == 0 { vec![] } else { vec![index - 1] })
                .collect(),
            funding_edges: 1,
        };
    }
    WorkloadShape {
        parents: (0..events)
            .map(|index| match index {
                0 | 1 => vec![],
                2 => vec![0, 1],
                _ => vec![index - 1],
            })
            .collect(),
        funding_edges: 2,
    }
}

/// Byte and reading samples for one funded native workload, kept outside the
/// graph recurrence so offline seed sizes cannot stand in for native bytes.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkloadSamples {
    pub event_bytes: Vec<usize>,
    pub parent_bytes: Vec<usize>,
    pub cold_read_micros: u64,
    pub warm_read_micros: [u64; 5],
}

/// Refusal to credit a workload measurement with absent funded bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkloadRefusal {
    IncompleteFundedBytes,
}

/// Refuse an incomplete or zero-length funded byte census.
/// # Errors
/// Refuses missing event/parent bytes or a selected shape with wrong counts.
pub fn bind_workload_samples(
    shape: &WorkloadShape,
    samples: &WorkloadSamples,
) -> Result<usize, WorkloadRefusal> {
    if samples.event_bytes.len() != shape.events()
        || samples.parent_bytes.len() != shape.funding_edges
        || samples
            .event_bytes
            .iter()
            .chain(&samples.parent_bytes)
            .any(|size| *size == 0)
    {
        return Err(WorkloadRefusal::IncompleteFundedBytes);
    }
    Ok(samples
        .event_bytes
        .iter()
        .chain(&samples.parent_bytes)
        .sum())
}

/// Refusal at the capture packet's identity boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaptureFreezeRefusal {
    /// No identity appeared in the packet.
    Missing,
    /// The packet identity differs from the expected validated triple.
    Unequal,
}

/// Check a capture packet's exact expected per-triple identity.
/// # Errors
/// Refuses missing or unequal identities before any observation is credited.
pub fn bind_capture_freeze(
    expected: &ValidatedFreeze,
    claimed: Option<[u8; 32]>,
) -> Result<[u8; 32], CaptureFreezeRefusal> {
    let _span = tracing::debug_span!(target: "vectors::keyed_fixed_capture", "bind_capture_freeze")
        .entered();
    let claimed = claimed.ok_or(CaptureFreezeRefusal::Missing)?;
    if claimed != expected.identity() {
        tracing::debug!(target: "vectors::keyed_fixed_capture", outcome = "refused", "capture freeze identity differs");
        return Err(CaptureFreezeRefusal::Unequal);
    }
    tracing::debug!(target: "vectors::keyed_fixed_capture", outcome = "completed", "capture freeze identity bound");
    Ok(claimed)
}

/// Exact request, answer and byte evidence for one proposed node submission.
#[derive(Clone, Debug)]
pub struct JournalRow {
    pub request: NativeOperationRequest,
    pub response: NativeOperationResponse,
    pub submitted: Vec<u8>,
    pub submitted_digest: [u8; 32],
    pub parent_bytes: Vec<Vec<u8>>,
    pub parent_digests: Vec<[u8; 32]>,
    pub readback: Option<Vec<u8>>,
    pub expected_provenance: CaptureProvenance,
    pub observed_provenance: CaptureProvenance,
}

/// Exact process and source provenance retained with each packet.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CaptureProvenance {
    pub freeze_identity: [u8; 32],
    pub executable_digest: [u8; 32],
    pub executor_digest: [u8; 32],
    pub dependency_lock_digest: [u8; 32],
    pub daemon_digest: [u8; 32],
    pub framework_revision: String,
    pub zk_library_digest: [u8; 32],
    pub image_identity: String,
}

/// The exact join that must precede any clause credit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JournalRefusal {
    Protocol,
    Case,
    Response,
    Submission,
    Parent,
    Readback,
    Provenance,
    Census,
    Outcome,
    Triple,
    Control,
    Sponsor,
}

/// One already observed and separately bound positive control.
#[derive(Clone, Copy)]
pub struct PairControl<'a> {
    pub subject: CaptureSubject,
    pub freeze: &'a ValidatedFreeze,
    pub row: &'a JournalRow,
    pub sponsor_index: usize,
}

/// Independent inputs to the credit decision, held outside the claimed row.
#[derive(Clone, Copy)]
pub struct JournalBinding<'a> {
    pub family: CaptureFamily,
    pub subject: CaptureSubject,
    pub sponsor_index: usize,
    pub prepared_provenance: &'a CaptureProvenance,
    pub control: Option<PairControl<'a>>,
}

/// The accepted subject that must precede a selected negative.
#[must_use]
pub const fn expected_control(subject: CaptureSubject) -> Option<CaptureSubject> {
    use CaptureSubject as S;
    match subject {
        S::ValueMinus | S::ValuePlus | S::ConfidentialValue | S::SurplusDestination(_) => {
            Some(S::OneMemberMerge(42))
        }
        S::MalformedPaymentValue
        | S::MalformedPaymentValueForm
        | S::LaterPosition
        | S::WrongProgram
        | S::WrongKey
        | S::DisplacedSuccessor
        | S::MalformedControl
        | S::KeyPathAttempt => Some(S::OneMemberMerge(42)),
        S::Floor(14) | S::ZeroFeeFloor => Some(S::Floor(15)),
        S::StructuralWeight1300 => Some(S::WeightMerge(1299)),
        S::PublicSource(_)
        | S::OneMemberMerge(_)
        | S::TwoMemberMerge(_)
        | S::Floor(_)
        | S::PresentNonceFloor(_)
        | S::WeightSource
        | S::WeightMerge(_)
        | S::DirectBlock1300
        | S::HistoryLinear(_)
        | S::HistoryBranch(_)
        | S::HistoryMissingBranch
        | S::LegacyConfidentialValue
        | S::LegacyPublicCopy => None,
    }
}

/// The target layer predicted for a submitted census subject; the script
/// instruction site still requires the bounded model and its paired facts.
///
/// A mixed ordinary sponsor input makes the adapter classify a member's
/// key-path attempt under its generic mandatory script-verification layer.
#[must_use]
pub fn expected_layer(subject: CaptureSubject) -> Option<ObservedOutcomeLayer> {
    use ObservedOutcomeLayer as O;
    Some(match subject.boundary()? {
        CaptureBoundary::Acceptance
        | CaptureBoundary::Consensus
        | CaptureBoundary::RelayPolicy
        | CaptureBoundary::FeePolicy => O::Accepted,
        CaptureBoundary::OutputPair
        | CaptureBoundary::InputPair
        | CaptureBoundary::BirthFirst
        | CaptureBoundary::Continuation
        | CaptureBoundary::Commitment
        | CaptureBoundary::KeyPath => O::ScriptPathRejection,
        CaptureBoundary::TypedConstruction | CaptureBoundary::IncompleteHistory => return None,
    })
}

fn matches_relay_prediction(subject: CaptureSubject, response: &NativeOperationResponse) -> bool {
    let expected_reason = match subject {
        CaptureSubject::Floor(14) => "dust",
        CaptureSubject::ZeroFeeFloor => "fee",
        CaptureSubject::StructuralWeight1300 => "tx-size",
        CaptureSubject::MalformedControl => "witness program hash mismatch",
        CaptureSubject::KeyPathAttempt => "invalid schnorr signature",
        _ => {
            if subject.boundary() != Some(CaptureBoundary::Acceptance) {
                return true;
            }
            let Some(detail) = response
                .observed_detail
                .as_deref()
                .and_then(|text| serde_json::from_str::<serde_json::Value>(text).ok())
            else {
                return false;
            };
            return detail["route"] == "relay-and-template" && detail["relay_allowed"] == true;
        }
    };
    let Some(detail) = response
        .observed_detail
        .as_deref()
        .and_then(|text| serde_json::from_str::<serde_json::Value>(text).ok())
    else {
        return false;
    };
    let route_matches = if matches!(
        subject,
        CaptureSubject::MalformedControl | CaptureSubject::KeyPathAttempt
    ) {
        detail["route"] == "refused"
    } else {
        detail["route"] == "direct-block"
    };
    route_matches
        && detail["relay_allowed"] == false
        && detail["relay_detail"]
            .as_str()
            .is_some_and(|reason| reason.to_ascii_lowercase().contains(expected_reason))
}

fn same_preparation(a: &CaptureProvenance, b: &CaptureProvenance) -> bool {
    a.executable_digest == b.executable_digest
        && a.executor_digest == b.executor_digest
        && a.dependency_lock_digest == b.dependency_lock_digest
        && a.daemon_digest == b.daemon_digest
        && a.framework_revision == b.framework_revision
        && a.zk_library_digest == b.zk_library_digest
        && a.image_identity == b.image_identity
}

fn bound_bytes(
    freeze: &ValidatedFreeze,
    row: &JournalRow,
    sponsor_index: usize,
    _subject: CaptureSubject,
) -> Result<TargetTransaction, JournalRefusal> {
    let transaction =
        TargetTransaction::decode(&row.submitted).map_err(|_| JournalRefusal::Triple)?;
    let parents: Vec<_> = row
        .parent_bytes
        .iter()
        .map(|bytes| TargetTransaction::decode(bytes).map(|parent| (parent, bytes.clone())))
        .collect::<Result<_, _>>()
        .map_err(|_| JournalRefusal::Parent)?;
    let program = &freeze.manifest().descriptor.output_program;
    let output_matches = transaction
        .outputs()
        .first()
        .is_some_and(|output| output.program() == program);
    let parent_matches = transaction
        .inputs()
        .iter()
        .zip(&parents)
        .any(|(input, (parent, _))| {
            parent
                .outputs()
                .get(input.outpoint().index() as usize)
                .is_some_and(|output| output.program() == program)
        });
    if !output_matches && !parent_matches {
        return Err(JournalRefusal::Triple);
    }
    if output_matches {
        let output = &transaction.outputs()[0];
        let expected = freeze.manifest().descriptor.parameters;
        let exact_pair = output.asset()
            == AssetField::Explicit(transaction::AssetId::from_internal(expected.asset))
            && output.value() == ValueField::Explicit(expected.value.get());
        if !exact_pair && !parent_matches {
            return Err(JournalRefusal::Triple);
        }
    }
    bind_structural_subject(
        freeze,
        Some(ObservedFreeze(freeze.identity())),
        transaction.clone(),
        &row.submitted,
        &parents,
        sponsor_index,
    )
    .map_err(|_| JournalRefusal::Triple)?
    .verify_sponsor()
    .map_err(|_| JournalRefusal::Sponsor)?;
    Ok(transaction)
}

/// Admit one capture row only after independent provenance, census, triple,
/// sponsor and paired-control joins; legacy controls have a separate owner.
/// # Errors
/// Refuses any absent or unequal join before giving fixed-form credit.
pub fn bind_journal(
    freeze: &ValidatedFreeze,
    row: &JournalRow,
    binding: &JournalBinding<'_>,
) -> Result<(), JournalRefusal> {
    validate_journal_transport(freeze, row)?;
    if binding.family == CaptureFamily::LegacyCopy
        || !subjects(binding.family).contains(&binding.subject)
    {
        return Err(JournalRefusal::Census);
    }
    let value = freeze.manifest().descriptor.parameters.value.get();
    if matches!(binding.subject, CaptureSubject::Floor(n) if value != u64::from(n))
        || (binding.subject == CaptureSubject::ZeroFeeFloor && value != 14)
    {
        return Err(JournalRefusal::Triple);
    }
    if Some(row.response.observed_layer) != expected_layer(binding.subject)
        || !matches_relay_prediction(binding.subject, &row.response)
    {
        return Err(JournalRefusal::Outcome);
    }
    if binding.prepared_provenance != &row.expected_provenance {
        return Err(JournalRefusal::Provenance);
    }
    let subject_transaction = bound_bytes(freeze, row, binding.sponsor_index, binding.subject)?;
    match (expected_control(binding.subject), &binding.control) {
        (None, None) => {}
        (Some(expected_subject), Some(control)) if control.subject == expected_subject => {
            let subject_q = freeze.manifest().descriptor.parameters;
            let control_q = control.freeze.manifest().descriptor.parameters;
            let floor_pair = matches!(
                binding.subject,
                CaptureSubject::Floor(14) | CaptureSubject::ZeroFeeFloor
            ) && subject_q.key == control_q.key
                && subject_q.asset == control_q.asset
                && subject_q.value.get().checked_add(1) == Some(control_q.value.get());
            if !floor_pair && control.freeze.identity() != freeze.identity() {
                return Err(JournalRefusal::Control);
            }
            validate_journal_transport(control.freeze, control.row)?;
            if !same_preparation(
                binding.prepared_provenance,
                &control.row.expected_provenance,
            ) {
                return Err(JournalRefusal::Control);
            }
            if control.row.response.observed_layer != ObservedOutcomeLayer::Accepted
                || !matches_relay_prediction(control.subject, &control.row.response)
            {
                return Err(JournalRefusal::Control);
            }
            let control_transaction = bound_bytes(
                control.freeze,
                control.row,
                control.sponsor_index,
                control.subject,
            )?;
            if subject_transaction.inputs().iter().any(|input| {
                control_transaction
                    .inputs()
                    .iter()
                    .any(|other| input.outpoint() == other.outpoint())
            }) {
                return Err(JournalRefusal::Control);
            }
        }
        (None, Some(_)) | (Some(_), None | Some(_)) => {
            return Err(JournalRefusal::Control);
        }
    }
    Ok(())
}

/// Bind the complete journal row without interpreting generic node error text.
/// # Errors
/// Refuses missing or substituted request, response, parent or accepted readback bytes.
pub fn validate_journal_transport(
    freeze: &ValidatedFreeze,
    row: &JournalRow,
) -> Result<(), JournalRefusal> {
    let _span = tracing::debug_span!(target: "vectors::keyed_fixed_capture", "bind_journal", parents = row.parent_bytes.len()).entered();
    if row.request.schema != NATIVE_PROTOCOL_SCHEMA || row.response.schema != NATIVE_PROTOCOL_SCHEMA
    {
        return Err(JournalRefusal::Protocol);
    }
    if row.expected_provenance.freeze_identity != freeze.identity()
        || row.expected_provenance != row.observed_provenance
    {
        return Err(JournalRefusal::Provenance);
    }
    if row.request.case != row.response.case
        || row.request.case.operation != OperationStepKind::Submit
    {
        return Err(JournalRefusal::Case);
    }
    row.response
        .validate_shape()
        .map_err(|_| JournalRefusal::Response)?;
    let OperationSubject::Submission(subject) = &row.request.subject else {
        return Err(JournalRefusal::Submission);
    };
    if row.submitted.is_empty()
        || subject.transaction_bytes != row.submitted
        || sha256(&row.submitted) != row.submitted_digest
    {
        return Err(JournalRefusal::Submission);
    }
    if row.parent_bytes.len() != row.parent_digests.len()
        || row
            .parent_bytes
            .iter()
            .zip(&row.parent_digests)
            .any(|(bytes, digest)| bytes.is_empty() || sha256(bytes) != *digest)
    {
        return Err(JournalRefusal::Parent);
    }
    let transaction: elements::Transaction =
        elements::encode::deserialize(&row.submitted).map_err(|_| JournalRefusal::Submission)?;
    if elements::encode::serialize(&transaction) != row.submitted
        || transaction.input.len() != row.parent_bytes.len()
    {
        return Err(JournalRefusal::Parent);
    }
    for (input, bytes) in transaction.input.iter().zip(&row.parent_bytes) {
        let parent: elements::Transaction =
            elements::encode::deserialize(bytes).map_err(|_| JournalRefusal::Parent)?;
        if elements::encode::serialize(&parent) != *bytes
            || input.previous_output.txid != parent.txid()
            || parent
                .output
                .get(input.previous_output.vout as usize)
                .is_none()
        {
            return Err(JournalRefusal::Parent);
        }
    }
    if let Some(txid) = &row.response.accepted_txid {
        let Some(mined) = &row.response.mined_readback else {
            return Err(JournalRefusal::Readback);
        };
        if txid != &mined.transaction_id
            || txid != &transaction.txid().to_string()
            || mined.raw_transaction != row.submitted
            || row
                .readback
                .as_ref()
                .is_none_or(|bytes| bytes != &row.submitted)
        {
            return Err(JournalRefusal::Readback);
        }
    } else if row.readback.is_some() {
        return Err(JournalRefusal::Readback);
    }
    Ok(())
}
