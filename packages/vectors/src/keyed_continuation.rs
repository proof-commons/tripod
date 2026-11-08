//! Key-free continuation intent, before staging or native observation.
//!
//! These projections are not authenticated transactions or constructor evidence.
//! Staging must bind the parent readback, same-key alternative leaf recipe and
//! exact signed bytes. Wrong-key, mixed-key and coexistence rows wait for the
//! constructor. No amount floor, issuance, peg-in or native stack claim is made.
//! Capture must retain every complete executor response in the capture itself
//! in wire JSON form; admission must not reconstruct responses from Debug text.

use std::collections::BTreeSet;

use crate::keyed_birth::{BirthOutpoint, BirthOutput, Wave1TargetPacket};

/// Canonical key-free census, distinct from the later key-dependent campaign.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContinuationSubjectId {
    /// Same program and version at output zero.
    Control,
    /// Different leaf recipe under the same public key.
    WrongProgram,
    /// Same payload encoded as witness version zero.
    WrongVersion,
    /// Exchange successor zero with the existing output one.
    DisplacedSuccessor,
}

impl ContinuationSubjectId {
    /// Canonical manifest order, not submission order.
    pub const ALL: [Self; 4] = [
        Self::Control,
        Self::WrongProgram,
        Self::WrongVersion,
        Self::DisplacedSuccessor,
    ];
    /// Sign and submit negatives before their same-coin control.
    pub const SUBMISSION_ORDER: [Self; 4] = [
        Self::WrongProgram,
        Self::WrongVersion,
        Self::DisplacedSuccessor,
        Self::Control,
    ];

    /// The only semantic coordinate changed from the control.
    #[must_use]
    pub const fn coordinate(self) -> Option<ContinuationCoordinate> {
        match self {
            Self::Control => None,
            Self::WrongProgram => Some(ContinuationCoordinate::SuccessorProgram),
            Self::WrongVersion => Some(ContinuationCoordinate::SuccessorVersion),
            Self::DisplacedSuccessor => Some(ContinuationCoordinate::OutputPermutation),
        }
    }
}

/// One semantic mutation per negative.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContinuationCoordinate {
    /// Replace only the output-zero program payload with the alternative recipe.
    SuccessorProgram,
    /// Replace only the output-zero witness version.
    SuccessorVersion,
    /// One permutation exchanges the two existing complete outputs.
    OutputPermutation,
}

/// Encoding and authorization consequences, not additional semantic changes.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ContinuationCollateral {
    /// Serialize the changed output program or ordering.
    OutputEncoding,
    /// Recompute txid and wtxid from the frozen bytes.
    TransactionIdentities,
    /// Re-sign the unrelated funding input over the changed outputs.
    FundingSignature,
    /// Recheck balance and fee adequacy; this fixed-fee census changes no amount.
    FeeBalanceCheck,
}

impl ContinuationCollateral {
    /// Exact declared collateral for all three negatives.
    pub const ALL: [Self; 4] = [
        Self::OutputEncoding,
        Self::TransactionIdentities,
        Self::FundingSignature,
        Self::FeeBalanceCheck,
    ];
}

/// Intended boundary, never a native verdict.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContinuationBoundary {
    /// A mined successful control is required by later admission.
    ScriptAcceptance,
    /// Version equality, instruction five in the isolated fragment (zero-based).
    VersionEqualVerify,
    /// Payload equality, instruction six in the isolated fragment (zero-based).
    ProgramEqualVerify,
}

/// Attribution limitation retained even for the control row.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContinuationAttribution {
    /// `EqualVerify` text alone cannot distinguish birth, version, payload or
    /// sponsor sites. Wrong-program and later wrong-key rows share the payload
    /// site. Require independent site evidence and matching controls; an earlier
    /// form refusal stays open and must be repaired or stop the obligation.
    SharedEqualityRequiresIndependentEvidence,
}

/// Required representation for the next capture, not evidence of its existence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContinuationResponseRetention {
    /// Every complete response is retained in the capture in wire JSON form.
    CompleteWireJson,
}

/// Fixed declarations shared by the control and each negative.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContinuationPairContext {
    /// Exact parent bytes, to authenticate by mined readback during staging.
    pub parent_bytes: Vec<u8>,
    /// Covenant outpoint, always at birth position zero.
    pub spent_outpoint: BirthOutpoint,
    /// Position of this covenant input in the two-input spend.
    pub current_input_index: u16,
    /// Consumed witness-version-one covenant output.
    pub predecessor: BirthOutput,
    /// Same fixed public key in both declared leaf recipes; not a signing key.
    pub public_key: [u8; 32],
    /// Original leaf recipe bytes, authenticated during staging.
    pub leaf: Vec<u8>,
    /// Deliberately different leaf recipe under the same public key.
    pub alternative_leaf: Vec<u8>,
    /// Valid version-one output program for that alternative recipe; staging
    /// must reconstruct it, since this manifest checks projections only.
    pub alternative_program: Vec<u8>,
    /// Sponsor coin held fixed for all same-coin subjects.
    pub funding_outpoint: BirthOutpoint,
    /// Sponsor program, asset and value held fixed.
    pub funding_output: BirthOutput,
    /// Control outputs: successor zero and an existing different-program output.
    pub control_outputs: [BirthOutput; 2],
    /// Explicit fee held fixed, with adequacy checked during staging.
    pub fee: u64,
}

/// One declared projection; no capture or verdict is implied.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContinuationSubject {
    /// Canonical census member.
    pub id: ContinuationSubjectId,
    /// Matching control, absent for the control itself.
    pub control: Option<ContinuationSubjectId>,
    /// All fixed pair coordinates, including valid birth position.
    pub context: ContinuationPairContext,
    /// Ordered spendable outputs; fee is separate.
    pub outputs: [BirthOutput; 2],
    /// Exactly one coordinate for negatives and none for the control.
    pub changed_coordinate: Option<ContinuationCoordinate>,
    /// Exact collateral set, empty for the control.
    pub collateral: BTreeSet<ContinuationCollateral>,
    /// Intended script boundary, subject to earlier-layer refusal.
    pub boundary: ContinuationBoundary,
    /// Explicit shared-site limitation; no error-text-only attribution.
    pub attribution: ContinuationAttribution,
}

/// Preparation refusal, never a script or node refusal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContinuationManifestRefusal {
    /// Missing, duplicated or reordered subject.
    SubjectCensus,
    /// Empty or malformed projected context.
    IncompleteContext,
    /// Birth position, control, coordinate or boundary is wrong.
    SubjectCoordinate,
    /// A fixed pair coordinate or an undeclared output coordinate changed.
    PairInvariant,
    /// Missing or surplus collateral.
    Collateral,
}

/// Frozen key-free campaign, sharing the Wave-1 target/provenance/policy packet.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContinuationCampaignManifest {
    packet: Wave1TargetPacket,
    subjects: Vec<ContinuationSubject>,
}

impl ContinuationCampaignManifest {
    /// Check the complete census and all one-coordinate projections.
    ///
    /// # Errors
    /// Refuses an incomplete census/context, invalid birth position, changed
    /// fixed coordinate, wrong boundary or noncanonical collateral declaration.
    pub fn new(
        packet: Wave1TargetPacket,
        subjects: Vec<ContinuationSubject>,
    ) -> Result<Self, ContinuationManifestRefusal> {
        let span = tracing::debug_span!(target: "vectors::keyed_continuation", "bind_continuation_campaign", subject_count = subjects.len());
        let _entered = span.enter();
        let result = validate_subjects(&subjects).map(|()| Self { packet, subjects });
        match &result {
            Ok(_) => {
                tracing::debug!(target: "vectors::keyed_continuation", outcome = "completed", "continuation preparation checked");
            }
            Err(error) => {
                tracing::debug!(target: "vectors::keyed_continuation", outcome = "refused", refusal = ?error, "continuation preparation refused");
            }
        }
        result
    }

    /// Shared reviewed target and separately sourced policy/provenance facts.
    #[must_use]
    pub const fn packet(&self) -> &Wave1TargetPacket {
        &self.packet
    }

    /// Canonical four-subject census.
    #[must_use]
    pub fn subjects(&self) -> &[ContinuationSubject] {
        &self.subjects
    }

    /// Required format for the later campaign's complete executor responses.
    #[must_use]
    pub const fn response_retention(&self) -> ContinuationResponseRetention {
        ContinuationResponseRetention::CompleteWireJson
    }
}

fn version_one(program: &[u8]) -> bool {
    program.len() == 34 && program[..2] == [0x51, 0x20]
}

fn validate_context(context: &ContinuationPairContext) -> Result<(), ContinuationManifestRefusal> {
    use ContinuationManifestRefusal as R;
    if context.spent_outpoint.output_index != 0 || context.current_input_index > 1 {
        return Err(R::SubjectCoordinate);
    }
    if context.parent_bytes.is_empty()
        || context.spent_outpoint.transaction_id == [0; 32]
        || context.funding_outpoint.transaction_id == [0; 32]
        || context.funding_outpoint.transaction_id == context.spent_outpoint.transaction_id
        || context.leaf.is_empty()
        || context.alternative_leaf.is_empty()
        || context.leaf == context.alternative_leaf
        || !version_one(&context.predecessor.program)
        || !version_one(&context.alternative_program)
        || context.alternative_program == context.predecessor.program
    {
        return Err(R::IncompleteContext);
    }
    if context.control_outputs[0].program != context.predecessor.program
        || context.control_outputs[1].program == context.predecessor.program
    {
        return Err(R::PairInvariant);
    }
    for output in [&context.predecessor, &context.funding_output]
        .into_iter()
        .chain(&context.control_outputs)
    {
        if output.program.is_empty() || output.asset == [0; 32] || output.value == 0 {
            return Err(R::IncompleteContext);
        }
    }
    Ok(())
}

pub(crate) fn validate_subjects(
    subjects: &[ContinuationSubject],
) -> Result<(), ContinuationManifestRefusal> {
    use ContinuationManifestRefusal as R;
    use ContinuationSubjectId as I;
    if !subjects.iter().map(|s| s.id).eq(I::ALL) {
        return Err(R::SubjectCensus);
    }
    let control = &subjects[0];
    for subject in subjects {
        validate_context(&subject.context)?;
        if subject.context != control.context {
            return Err(R::PairInvariant);
        }
        let negative = subject.id != I::Control;
        let boundary = match subject.id {
            I::Control => ContinuationBoundary::ScriptAcceptance,
            I::WrongVersion => ContinuationBoundary::VersionEqualVerify,
            I::WrongProgram | I::DisplacedSuccessor => ContinuationBoundary::ProgramEqualVerify,
        };
        if subject.control != negative.then_some(I::Control)
            || subject.changed_coordinate != subject.id.coordinate()
            || subject.boundary != boundary
        {
            return Err(R::SubjectCoordinate);
        }
        let collateral = if negative {
            ContinuationCollateral::ALL.into_iter().collect()
        } else {
            BTreeSet::new()
        };
        if subject.collateral != collateral {
            return Err(R::Collateral);
        }
        let mut expected = control.context.control_outputs.clone();
        match subject.id {
            I::Control => {}
            I::WrongProgram => expected[0]
                .program
                .clone_from(&control.context.alternative_program),
            I::WrongVersion => expected[0].program[0] = 0,
            I::DisplacedSuccessor => expected.swap(0, 1),
        }
        if subject.outputs != expected {
            return Err(R::PairInvariant);
        }
    }
    // The displaced row is intended to reach payload equality; keep its version
    // unchanged as well, rather than silently testing version mismatch instead.
    if !version_one(&control.context.control_outputs[1].program) {
        return Err(R::PairInvariant);
    }
    Ok(())
}
