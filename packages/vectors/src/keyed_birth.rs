//! Frozen intent for the Wave-1 born-first experiment, before native capture.
//!
//! Packet completeness is not executable provenance verification or evidence
//! admission. The caller supplies provenance references; capture must check them
//! against the pinned executable and source. Policy facts retain their origin
//! separately from the reviewed contract and development declaration.
//!
//! Subject coordinates are declared projections, not decoded chain facts. The
//! later staging/capture step must bind them to exact signed submissions and
//! mined parent readbacks. No constructor here claims an accepted or refused
//! native outcome, a funding floor, issuance coverage or peg-in coverage.

use std::collections::BTreeSet;

use target_elements::{ReviewedDevelopmentBinding, ReviewedElementsTapscriptDefinition};
use target_elements_conformance::protocol::NATIVE_PROTOCOL_SCHEMA;

/// A missing or inconsistent preparation field; never a native refusal.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BirthPacketRefusal {
    /// A required field is absent or malformed.
    Incomplete(&'static str),
    /// The binding does not select the reviewed active development target.
    TargetBinding,
    /// The protocol or adapter differs from the inherited entry.
    ExecutorBinding,
    /// Executable and source provenance disagree.
    ProvenanceBinding,
    /// The four-subject census is missing, duplicated or out of order.
    SubjectCensus,
    /// A subject changes a coordinate other than its declared birth position.
    SubjectCoordinate,
    /// A pair does not share all fixed coordinates.
    PairInvariant,
    /// The two placement pairs reuse a parent or funding coin.
    ReusedPairFunding,
    /// The declared collateral differs from the exact allowed set.
    Collateral,
}

impl BirthPacketRefusal {
    const fn kind(&self) -> &'static str {
        match self {
            Self::Incomplete(_) => "incomplete",
            Self::TargetBinding => "target_binding",
            Self::ExecutorBinding => "executor_binding",
            Self::ProvenanceBinding => "provenance_binding",
            Self::SubjectCensus => "subject_census",
            Self::SubjectCoordinate => "subject_coordinate",
            Self::PairInvariant => "pair_invariant",
            Self::ReusedPairFunding => "reused_pair_funding",
            Self::Collateral => "collateral",
        }
    }
}

/// Executable and source provenance supplied for later independent verification.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Wave1ExecutableProvenance {
    /// Node implementation name, as in the handshake.
    pub node_name: String,
    /// Node version, as in the handshake.
    pub node_version: String,
    /// Binary-reported source revision, possibly a prefix of the intended tip.
    pub binary_revision: String,
    /// Full intended source revision, provenance rather than protocol identity.
    pub intended_tip: String,
    /// Functional framework source revision.
    pub framework_revision: String,
    /// Absolute path of the selected executable; no credential path belongs here.
    pub executable_path: String,
    /// SHA-256 of that executable, to compare with the provisioned record.
    pub executable_sha256: [u8; 32],
    /// Location of the selected toolchain's provenance record.
    pub provenance_record: String,
}

/// Native protocol and adapter identity, separate from the node's identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Wave1ExecutorBinding {
    /// Development chain name from the executor environment.
    pub chain: String,
    /// Native request/response schema, inherited as revision eight.
    pub protocol_schema: u32,
    /// Adapter name from the handshake.
    pub adapter_name: String,
    /// Adapter version from the handshake.
    pub adapter_version: String,
}

/// Whether a policy value was declared or actually reported by the node.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PolicyFactOrigin {
    /// A configuration or pinned-source declaration, not a node observation.
    Declared,
    /// A retained node response; its source must be checked during admission.
    Observed,
}

/// One fee floor with its evidence origin and source kept together.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BirthFeeFloor {
    /// Exact fee floor in satoshis per thousand virtual bytes.
    pub satoshis_per_kvb: u64,
    /// Declaration versus observation is never inferred from the value.
    pub origin: PolicyFactOrigin,
    /// Configuration, pinned-source location or retained response reference.
    pub source: String,
}

/// Policy selected for this campaign, not an additional covenant condition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BirthPolicy {
    /// Explicit statement of the value/asset representation used by the fixtures.
    pub representation: String,
    /// Adapter fee-policy selection, recorded rather than assumed.
    pub fee_policy: String,
    /// Exact node startup arguments, to be compared during capture.
    pub startup_arguments: Vec<String>,
    /// Relay floor and where it came from.
    pub relay: BirthFeeFloor,
    /// Mempool floor and where it came from.
    pub mempool: BirthFeeFloor,
    /// Block assembly floor and where it came from.
    pub block: BirthFeeFloor,
}

/// Complete preparation binding; no native evidence standing is conferred.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Wave1TargetPacket {
    target: ReviewedElementsTapscriptDefinition,
    deployment: ReviewedDevelopmentBinding,
    provenance: Wave1ExecutableProvenance,
    executor: Wave1ExecutorBinding,
    policy: BirthPolicy,
}

impl Wave1TargetPacket {
    /// Bind reviewed target, development declaration, provenance and policy.
    ///
    /// # Errors
    /// Refuses incomplete provenance/policy or an inconsistent target, source
    /// revision or executor binding. This does not read an executable or a node.
    pub fn bind(
        target: ReviewedElementsTapscriptDefinition,
        deployment: ReviewedDevelopmentBinding,
        provenance: Wave1ExecutableProvenance,
        executor: Wave1ExecutorBinding,
        policy: BirthPolicy,
    ) -> Result<Self, BirthPacketRefusal> {
        let span = tracing::debug_span!(target: "vectors::keyed_birth", "bind_target_packet");
        let _entered = span.enter();
        let packet = Self {
            target,
            deployment,
            provenance,
            executor,
            policy,
        };
        let result = packet.validate().map(|()| packet);
        record_outcome(&result);
        result
    }

    fn validate(&self) -> Result<(), BirthPacketRefusal> {
        if !self.deployment.welded_to(&self.target)
            || !self
                .deployment
                .binding()
                .activation()
                .tapscript_expected_active()
        {
            return Err(BirthPacketRefusal::TargetBinding);
        }
        if self.executor.chain != "elementsregtest"
            || self.executor.protocol_schema != NATIVE_PROTOCOL_SCHEMA
            || self.executor.adapter_name != "elements-native-executor"
            || self.executor.adapter_version != "2.1.0"
        {
            return Err(BirthPacketRefusal::ExecutorBinding);
        }
        self.provenance.validate()?;
        self.policy.validate()
    }

    /// Reviewed static target, without deployment observations.
    #[must_use]
    pub const fn target(&self) -> &ReviewedElementsTapscriptDefinition {
        &self.target
    }

    /// Development declaration checked against that target.
    #[must_use]
    pub const fn deployment(&self) -> &ReviewedDevelopmentBinding {
        &self.deployment
    }

    /// Provenance references that native capture must independently verify.
    #[must_use]
    pub const fn provenance(&self) -> &Wave1ExecutableProvenance {
        &self.provenance
    }

    /// Protocol and adapter selection.
    #[must_use]
    pub const fn executor(&self) -> &Wave1ExecutorBinding {
        &self.executor
    }

    /// Policy facts with their individual origins preserved.
    #[must_use]
    pub const fn policy(&self) -> &BirthPolicy {
        &self.policy
    }
}

impl Wave1ExecutableProvenance {
    fn validate(&self) -> Result<(), BirthPacketRefusal> {
        for (field, value) in [
            ("node name", self.node_name.as_str()),
            ("node version", self.node_version.as_str()),
            ("provenance record", self.provenance_record.as_str()),
        ] {
            require_text(field, value)?;
        }
        if !self.executable_path.starts_with('/') || self.executable_sha256 == [0; 32] {
            return Err(BirthPacketRefusal::Incomplete("executable"));
        }
        if !hex_revision(&self.intended_tip, 40)
            || !hex_revision(&self.framework_revision, 40)
            || !(12..=40).contains(&self.binary_revision.len())
            || !hex_revision(&self.binary_revision, self.binary_revision.len())
            || !self.intended_tip.starts_with(&self.binary_revision)
            || self.framework_revision != self.intended_tip
        {
            return Err(BirthPacketRefusal::ProvenanceBinding);
        }
        Ok(())
    }
}

impl BirthPolicy {
    fn validate(&self) -> Result<(), BirthPacketRefusal> {
        require_text("representation", &self.representation)?;
        if !matches!(
            self.fee_policy.as_str(),
            "node-defaults" | "historical-zero"
        ) {
            return Err(BirthPacketRefusal::Incomplete("fee policy"));
        }
        if self.startup_arguments.is_empty() {
            return Err(BirthPacketRefusal::Incomplete("startup arguments"));
        }
        for argument in &self.startup_arguments {
            require_text("startup argument", argument)?;
        }
        for floor in [&self.relay, &self.mempool, &self.block] {
            require_text("fee floor source", &floor.source)?;
        }
        Ok(())
    }
}

fn require_text(field: &'static str, value: &str) -> Result<(), BirthPacketRefusal> {
    if value.trim().is_empty() {
        Err(BirthPacketRefusal::Incomplete(field))
    } else {
        Ok(())
    }
}

fn hex_revision(value: &str, width: usize) -> bool {
    value.len() == width
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn record_outcome<T>(result: &Result<T, BirthPacketRefusal>) {
    match result {
        Ok(_) => tracing::debug!(
            target: "vectors::keyed_birth",
            outcome = "completed",
            "birth preparation checked"
        ),
        Err(error) => tracing::debug!(
            target: "vectors::keyed_birth",
            outcome = "refused",
            refusal_kind = error.kind(),
            "birth preparation refused"
        ),
    }
}

/// Stable experiment subjects; these names do not identify native transactions.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum BirthSubjectId {
    /// Birth zero at current input zero, the first control.
    A0,
    /// Birth one at current input zero, against A0.
    A1,
    /// Fresh birth zero at current input one, the second control.
    A2,
    /// Birth one at current input one, against A2.
    A3,
}

impl BirthSubjectId {
    /// Complete canonical subject order, independent of submission order.
    pub const ALL: [Self; 4] = [Self::A0, Self::A1, Self::A2, Self::A3];

    /// Negative-first submission order so a control cannot spend its pair's coin.
    pub const SUBMISSION_ORDER: [Self; 4] = [Self::A1, Self::A0, Self::A3, Self::A2];

    /// The subject's declared birth and current-input positions.
    #[must_use]
    pub const fn positions(self) -> (u32, u16) {
        match self {
            Self::A0 => (0, 0),
            Self::A1 => (1, 0),
            Self::A2 => (0, 1),
            Self::A3 => (1, 1),
        }
    }

    /// The matching control, absent for a control itself.
    #[must_use]
    pub const fn control(self) -> Option<Self> {
        match self {
            Self::A0 | Self::A2 => None,
            Self::A1 => Some(Self::A0),
            Self::A3 => Some(Self::A2),
        }
    }
}

/// The only semantic mutation in a birth negative.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BirthCoordinate {
    /// Spend the same parent's output one instead of output zero.
    SpentOutputIndex,
}

/// Necessary encoding consequences, not additional semantic mutations.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum BirthCollateral {
    /// Serialized covenant outpoint reflects the changed index.
    OutpointEncoding,
    /// Transaction and witness transaction identities are recomputed.
    TransactionIdentities,
    /// Funding authorization is recomputed over the exact changed transaction.
    FundingSignature,
}

impl BirthCollateral {
    /// Exact permitted collateral for either negative.
    pub const ALL: [Self; 3] = [
        Self::OutpointEncoding,
        Self::TransactionIdentities,
        Self::FundingSignature,
    ];
}

/// Intended script boundary, not a recorded observation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BirthBoundary {
    /// Successful standalone script spend; relay and consensus remain separate.
    ScriptAcceptance,
    /// Refusal at the birth fragment's verifying byte equality.
    BornFirstEqualVerify,
}

/// Public outpoint coordinates, with transaction bytes authenticated later.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BirthOutpoint {
    /// Transaction identity in internal byte order.
    pub transaction_id: [u8; 32],
    /// Output position, independent of current-input position.
    pub output_index: u32,
}

/// Explicit output coordinates frozen by the experiment.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BirthOutput {
    /// Complete scriptPubKey, including witness version where applicable.
    pub program: Vec<u8>,
    /// Asset identity in internal byte order.
    pub asset: [u8; 32],
    /// Positive fixture value, not a measured funding floor.
    pub value: u64,
}

/// All coordinates held fixed within a control/negative pair.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BirthPairContext {
    /// Exact proposed parent bytes; capture must authenticate them by readback.
    pub parent_bytes: Vec<u8>,
    /// Parent identity; capture must recompute it from the bytes.
    pub parent_id: [u8; 32],
    /// Equal same-program parent outputs at positions zero and one.
    pub parent_outputs: [BirthOutput; 2],
    /// Exact funding coin, including its own birth position.
    pub funding_outpoint: BirthOutpoint,
    /// Funding coin's program, asset and value.
    pub funding_output: BirthOutput,
    /// Spending outputs in order: successor at zero, then any change.
    /// The fee is represented separately, so every entry here is spendable.
    pub spending_outputs: Vec<BirthOutput>,
    /// Explicit policy-asset fee, fixed within a pair.
    pub fee: u64,
}

/// Declared subject projections; no signed submission or observation yet exists.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BirthSubject {
    /// Stable census identity.
    pub id: BirthSubjectId,
    /// Matching control identity, absent for controls.
    pub control: Option<BirthSubjectId>,
    /// Covenant input position in an exactly two-input spending form.
    pub current_input_index: u16,
    /// The covenant output being spent.
    pub spent_outpoint: BirthOutpoint,
    /// Fixed pair coordinates, compared independently of the changed outpoint.
    pub context: BirthPairContext,
    /// One semantic coordinate for negatives, none for controls.
    pub changed_coordinate: Option<BirthCoordinate>,
    /// Exact declared encoding collateral, empty for controls.
    pub collateral: BTreeSet<BirthCollateral>,
    /// Where this subject is intended to be answered.
    pub boundary: BirthBoundary,
}

/// Complete four-subject preparation, not an admitted native measurement.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BirthCampaignManifest {
    packet: Wave1TargetPacket,
    subjects: Vec<BirthSubject>,
}

impl BirthCampaignManifest {
    /// Freeze the complete census and check its declared one-coordinate pairs.
    ///
    /// # Errors
    /// Refuses an incomplete census, changed fixed coordinate, reused pair coin,
    /// incorrect boundary or collateral, or an incomplete parent/output record.
    pub fn new(
        packet: Wave1TargetPacket,
        subjects: Vec<BirthSubject>,
    ) -> Result<Self, BirthPacketRefusal> {
        let span = tracing::debug_span!(
            target: "vectors::keyed_birth",
            "bind_birth_campaign",
            subject_count = u64::try_from(subjects.len()).unwrap_or(u64::MAX)
        );
        let _entered = span.enter();
        let result = validate_subjects(&subjects).map(|()| Self { packet, subjects });
        record_outcome(&result);
        result
    }

    /// Exact target and policy preparation used by every subject.
    #[must_use]
    pub const fn packet(&self) -> &Wave1TargetPacket {
        &self.packet
    }

    /// Frozen subject census in canonical A0–A3 order.
    #[must_use]
    pub fn subjects(&self) -> &[BirthSubject] {
        &self.subjects
    }
}

fn validate_subjects(subjects: &[BirthSubject]) -> Result<(), BirthPacketRefusal> {
    if !subjects
        .iter()
        .map(|subject| subject.id)
        .eq(BirthSubjectId::ALL)
    {
        return Err(BirthPacketRefusal::SubjectCensus);
    }
    for subject in subjects {
        validate_subject(subject)?;
    }
    let [a0, a1, a2, a3] = subjects else {
        return Err(BirthPacketRefusal::SubjectCensus);
    };
    if a0.context != a1.context || a2.context != a3.context {
        return Err(BirthPacketRefusal::PairInvariant);
    }
    if a0.context.parent_id == a2.context.parent_id
        || a0.context.funding_outpoint == a2.context.funding_outpoint
    {
        return Err(BirthPacketRefusal::ReusedPairFunding);
    }
    Ok(())
}

fn validate_subject(subject: &BirthSubject) -> Result<(), BirthPacketRefusal> {
    let (birth, current) = subject.id.positions();
    let negative = subject.id.control().is_some();
    let context = &subject.context;
    if subject.control != subject.id.control()
        || subject.current_input_index != current
        || subject.spent_outpoint.output_index != birth
        || subject.spent_outpoint.transaction_id != context.parent_id
        || subject.changed_coordinate != negative.then_some(BirthCoordinate::SpentOutputIndex)
        || subject.boundary
            != if negative {
                BirthBoundary::BornFirstEqualVerify
            } else {
                BirthBoundary::ScriptAcceptance
            }
        || (current == 1 && context.funding_outpoint.output_index == 0)
    {
        return Err(BirthPacketRefusal::SubjectCoordinate);
    }
    let collateral = if negative {
        BirthCollateral::ALL.into_iter().collect()
    } else {
        BTreeSet::new()
    };
    if subject.collateral != collateral {
        return Err(BirthPacketRefusal::Collateral);
    }
    if context.parent_bytes.is_empty() || context.parent_id == [0; 32] {
        return Err(BirthPacketRefusal::Incomplete("parent"));
    }
    if context.funding_outpoint.transaction_id == [0; 32]
        || context.funding_outpoint.transaction_id == context.parent_id
    {
        return Err(BirthPacketRefusal::Incomplete("funding outpoint"));
    }
    if context.parent_outputs[0] != context.parent_outputs[1]
        || context.spending_outputs.first() != Some(&context.parent_outputs[0])
    {
        return Err(BirthPacketRefusal::PairInvariant);
    }
    for output in context
        .parent_outputs
        .iter()
        .chain([&context.funding_output])
        .chain(&context.spending_outputs)
    {
        if output.program.is_empty() || output.asset == [0; 32] || output.value == 0 {
            return Err(BirthPacketRefusal::Incomplete("explicit output"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
