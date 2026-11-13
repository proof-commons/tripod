//! Conformance carriers from sealed keyed construction to the existing executor.
//!
//! These are offline capture values, not a production planner or descriptor.
//! A mock transcript establishes transport and reproduction, never native validity.
use target_elements_conformance::constructor::tagged::sha256;
use target_elements_conformance::executor::OperationStep;
use target_elements_conformance::protocol::{
    ExecutorHandshake, NativeOperationRequest, NativeOperationResponse, ObservedOutcomeLayer,
    OperationRevision, OperationStepKind, OperationSubject, ResponseShapeDefect,
    TargetSponsorSigningSubject, TargetSubmissionSubject, WireOutpoint, WireSighashProfile,
    WireSpentOutput,
};
use target_elements_conformance::sponsor_parent::{SponsorParentRefusal, bind_sponsor_parent};
use transaction::keyed_construct::{
    KeyedAdmittedTransaction, KeyedConstructionRequest, KeyedStructuralTransaction,
};
use transaction::keyed_inputs::{BoundKeyedInput, BoundParentOutput, KeyedInputRole};
use transaction::{
    SighashProfile, SignerRole, SponsorOffer, SponsorSigningRequest, TargetTransaction,
};

/// SHA-256 of the exact complete submitted transaction, not a transaction id.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SubjectDigest(pub [u8; 32]);
/// The checked request's operation, not a caller's classification of raw bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FinalKind {
    /// Checked SOURCE request, consuming no handle of its key.
    Source,
    /// Checked nonempty MERGE request.
    Merge,
}
/// Binding failure before capture. None is a covenant execution verdict.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BridgeRefusal {
    /// A different ordered request census was paired with the structure.
    Census,
    /// Output zero does not carry this request's commitment.
    OutputZero,
    /// A member's leaf or control differs from its checked commitment.
    MemberWitness,
    /// The ordinary signer position cannot be represented or does not exist.
    SigningIndex,
    /// The selected signer is not collateral with a retained capability offer.
    SigningRole,
    /// The real signing request names another role or sighash profile.
    SigningProfile,
    /// The real signing request differs from the frozen bytes or protected outputs.
    SigningBytes,
    /// Reconstructed request capability snapshots differ from the frozen ones.
    CapabilitySnapshot,
    /// An admitted transaction changes the frozen non-witness bytes.
    SignedBytes,
    /// An untrusted submission is not the exact sealed final subject.
    SubmissionBytes,
    /// An untrusted request names another digest or operation.
    SubmissionCase,
    /// A request names another recorded protocol revision.
    SubmissionSchema,
    /// A future input role is not implemented by this conformance adapter.
    UnsupportedRole,
    /// The signer output is outside the explicit admitted form.
    SignerForm,
}
impl BridgeRefusal {
    /// Stable condition name; this is a capture-binding boundary.
    #[must_use]
    pub const fn refusal_kind(&self) -> &'static str {
        match self {
            Self::Census => "census",
            Self::OutputZero => "output_zero",
            Self::MemberWitness => "member_witness",
            Self::SigningIndex => "signing_index",
            Self::SigningRole => "signing_role",
            Self::SigningProfile => "signing_profile",
            Self::SigningBytes => "signing_bytes",
            Self::CapabilitySnapshot => "capability_snapshot",
            Self::SignedBytes => "signed_bytes",
            Self::SubmissionBytes => "submission_bytes",
            Self::SubmissionCase => "submission_case",
            Self::SubmissionSchema => "submission_schema",
            Self::UnsupportedRole => "unsupported_role",
            Self::SignerForm => "signer_form",
        }
    }
}
/// A parent-bound input retained for the byte/evidence join; no chain fact is discharged.
/// ```compile_fail,E0451
/// use vectors::keyed_final_native::FinalInput;
/// fn forge() -> FinalInput { FinalInput { index: 0, sequence: 0, parent: panic!(), member: false, capability: None } }
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FinalInput {
    index: u32,
    sequence: u32,
    parent: BoundParentOutput,
    member: bool,
    capability: Option<SponsorOffer>,
}
impl FinalInput {
    /// Actual transaction input index.
    #[must_use]
    pub const fn index(&self) -> u32 {
        self.index
    }
    /// Exact retained sequence offer.
    #[must_use]
    pub const fn sequence(&self) -> u32 {
        self.sequence
    }
    /// Canonical parent bytes and selected-output facts.
    #[must_use]
    pub const fn parent(&self) -> &BoundParentOutput {
        &self.parent
    }
    /// Whether the checked request classified this input as a member.
    #[must_use]
    pub const fn member(&self) -> bool {
        self.member
    }
    /// Snapshot read once by the request, not renewed signing authority.
    #[must_use]
    pub const fn capability(&self) -> Option<&SponsorOffer> {
        self.capability.as_ref()
    }
}
/// Immutable pre-signing structure originating in sealed public construction.
/// ```compile_fail,E0451
/// use vectors::keyed_final_native::FrozenFinalSubject;
/// fn forge() -> FrozenFinalSubject { FrozenFinalSubject { key: [0;32], kind: panic!(), transaction: panic!(), inputs: vec![], members: panic!(), signing: panic!() } }
/// ```
#[derive(Clone, Debug)]
pub struct FrozenFinalSubject {
    key: [u8; 32],
    kind: FinalKind,
    transaction: TargetTransaction,
    inputs: Vec<FinalInput>,
    members: tapscript::keyed_members::KeyedMemberReceipt,
    signing: TargetSponsorSigningSubject,
}
impl FrozenFinalSubject {
    /// Public semantic key data.
    #[must_use]
    pub const fn key(&self) -> &[u8; 32] {
        &self.key
    }
    /// Operation of the sealed request.
    #[must_use]
    pub const fn kind(&self) -> FinalKind {
        self.kind
    }
    /// Ordered census, never reordered for sponsor signing.
    #[must_use]
    pub fn inputs(&self) -> &[FinalInput] {
        &self.inputs
    }
    /// Byte-bound member/relation receipt; native execution remains owed.
    #[must_use]
    pub const fn members(&self) -> &tapscript::keyed_members::KeyedMemberReceipt {
        &self.members
    }
    /// Non-witness bytes fixed before ordinary signing.
    #[must_use]
    pub fn frozen_bytes(&self) -> &[u8] {
        &self.signing.finalized_transaction
    }
    /// The protocol request contains no expected result or covenant signature.
    #[must_use]
    pub fn signing_step(&self) -> OperationStep {
        let digest = sha256(self.frozen_bytes());
        OperationStep::new(
            &format!(
                "final-sign-{}",
                crate::maturity_observer::encode_hex(&digest)
            ),
            OperationSubject::SponsorSigning(Box::new(self.signing.clone())),
        )
    }
    /// Compare the production-generated signing request with the retained freeze.
    /// # Errors
    /// Returns the exact signing-coordinate mismatch; accepts no secret.
    pub fn bind_signing_request(
        &self,
        request: &SponsorSigningRequest,
    ) -> Result<(), BridgeRefusal> {
        let _span =
            tracing::debug_span!(target: "vectors::keyed_final_native", "bind_signing_request")
                .entered();
        let result = if request.role() != SignerRole::KeyedCollateral
            || request.profile() != SighashProfile::AllInputsAllOutputs
        {
            Err(BridgeRefusal::SigningProfile)
        } else if request.input() != self.signing.sponsor_input_index {
            Err(BridgeRefusal::SigningIndex)
        } else if request.transaction() != self.frozen_bytes()
            || request.protected() != self.transaction.outputs()
        {
            Err(BridgeRefusal::SigningBytes)
        } else {
            Ok(())
        };
        terminal(&result, "bind_signing_request");
        result
    }
}
fn census(inputs: &[BoundKeyedInput<'_>]) -> Result<Vec<FinalInput>, BridgeRefusal> {
    inputs
        .iter()
        .map(|row| {
            Ok(FinalInput {
                index: row.input_index(),
                sequence: row.sequence(),
                parent: row.parent().clone(),
                member: match row.role() {
                    KeyedInputRole::Member => true,
                    KeyedInputRole::Collateral => false,
                    _ => return Err(BridgeRefusal::UnsupportedRole),
                },
                capability: row.capability_offer().cloned(),
            })
        })
        .collect()
}
/// Freeze an already materialized public request, with one ordinary signer index.
/// # Errors
/// Refuses a detached structure, census, member witness or signing coordinate.
pub fn freeze_final_subject(
    request: KeyedConstructionRequest<'_, '_>,
    structure: &KeyedStructuralTransaction<'_, '_>,
    signer: usize,
) -> Result<FrozenFinalSubject, BridgeRefusal> {
    let _span = tracing::debug_span!(target: "vectors::keyed_final_native", "freeze_final_subject")
        .entered();
    let result = freeze_core(request, structure, signer);
    terminal(&result, "freeze_final_subject");
    result
}
fn freeze_core(
    request: KeyedConstructionRequest<'_, '_>,
    structure: &KeyedStructuralTransaction<'_, '_>,
    signer: usize,
) -> Result<FrozenFinalSubject, BridgeRefusal> {
    let (kind, rows, commitment, members) = match request {
        KeyedConstructionRequest::Source(r) => {
            (FinalKind::Source, r.inputs(), r.commitment(), r.members())
        }
        KeyedConstructionRequest::Merge(r) => {
            (FinalKind::Merge, r.inputs(), r.commitment(), r.members())
        }
    };
    let transaction = structure.transaction();
    if transaction.inputs().len() != rows.len() {
        return Err(BridgeRefusal::Census);
    }
    if transaction
        .outputs()
        .first()
        .map(transaction::TargetOutput::program)
        != Some(commitment.output_program())
    {
        return Err(BridgeRefusal::OutputZero);
    }
    for ((built, witness), row) in transaction
        .inputs()
        .iter()
        .zip(transaction.witnesses())
        .zip(rows)
    {
        if built.outpoint() != row.parent().outpoint() || built.sequence() != row.sequence() {
            return Err(BridgeRefusal::Census);
        }
        if row.role() == KeyedInputRole::Member {
            check_member_witness(witness.stack(), commitment.leaf(), commitment.control())?;
        }
    }
    let index = u16::try_from(signer).map_err(|_| BridgeRefusal::SigningIndex)?;
    let row = rows.get(signer).ok_or(BridgeRefusal::SigningIndex)?;
    if row.role() != KeyedInputRole::Collateral || row.capability_offer().is_none() {
        return Err(BridgeRefusal::SigningRole);
    }
    let output = row.parent().output();
    let (transaction::AssetField::Explicit(asset), transaction::ValueField::Explicit(value)) =
        (output.asset(), output.value())
    else {
        return Err(BridgeRefusal::SignerForm);
    };
    let mut asset_field = vec![1];
    asset_field.extend_from_slice(asset.internal());
    let mut value_field = vec![1];
    value_field.extend_from_slice(&value.to_be_bytes());
    Ok(FrozenFinalSubject {
        key: *commitment.recipe().key(),
        kind,
        transaction: transaction.clone(),
        inputs: census(rows)?,
        members: members.clone(),
        signing: TargetSponsorSigningSubject {
            finalized_transaction: transaction.encode_without_witness(),
            sponsor_input_index: index,
            sponsor_outpoint: WireOutpoint {
                txid: row.parent().outpoint().txid().to_target_display(),
                vout: row.parent().outpoint().index(),
            },
            spent_output: Some(WireSpentOutput {
                asset_field,
                value_field,
                program: output.program().to_vec(),
            }),
            sighash_profile: WireSighashProfile::AllInputsAllOutputs,
        },
    })
}
// Defensive over sealed structures: public key mispairing refuses at OutputZero first.
fn check_member_witness(
    stack: &[Vec<u8>],
    leaf: &[u8],
    control: &[u8],
) -> Result<(), BridgeRefusal> {
    if stack == [leaf.to_vec(), control.to_vec()] {
        Ok(())
    } else {
        Err(BridgeRefusal::MemberWitness)
    }
}
/// Expected target observation, held outside the executor request.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaptureExpectation {
    /// Target submission must accept; only a native capture can establish it.
    AcceptedAtSubmission,
}
/// Signed and policy-admitted public construction, bound to its exact capture bytes.
/// ```compile_fail,E0451
/// use vectors::keyed_final_native::FinalSubject;
/// fn forge() -> FinalSubject { FinalSubject { frozen: panic!(), bytes: vec![], digest: panic!(), policy: String::new() } }
/// ```
#[derive(Clone, Debug)]
pub struct FinalSubject {
    frozen: FrozenFinalSubject,
    bytes: Vec<u8>,
    digest: SubjectDigest,
    policy: String,
}
impl FinalSubject {
    /// Positive subject expectation; no expected answer crosses the protocol.
    #[must_use]
    pub const fn expectation(&self) -> CaptureExpectation {
        CaptureExpectation::AcceptedAtSubmission
    }
    /// Pre-signing subject and retained census.
    #[must_use]
    pub const fn frozen(&self) -> &FrozenFinalSubject {
        &self.frozen
    }
    /// Exact complete bytes submitted by this subject.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    /// SHA-256 of the complete bytes, not the stripped transaction id.
    #[must_use]
    pub const fn digest(&self) -> SubjectDigest {
        self.digest
    }
    /// Caller policy-reading provenance retained by admission.
    #[must_use]
    pub fn policy_provenance(&self) -> &str {
        &self.policy
    }
    /// Protocol subject named by its complete-byte digest; no expected answer.
    #[must_use]
    pub fn submission_step(&self) -> OperationStep {
        OperationStep::new(
            &format!(
                "final-submit-{}",
                crate::maturity_observer::encode_hex(&self.digest.0)
            ),
            OperationSubject::Submission(Box::new(TargetSubmissionSubject {
                transaction_bytes: self.bytes.clone(),
            })),
        )
    }
    /// Refuse bytes or a case identity not obtained from this sealed capture value.
    /// # Errors
    /// Returns the schema, case or byte mismatch in that order.
    pub fn validate_request(
        &self,
        recorded_revision: OperationRevision,
        request: &NativeOperationRequest,
    ) -> Result<(), BridgeRefusal> {
        let _span =
            tracing::debug_span!(target: "vectors::keyed_final_native", "validate_final_request")
                .entered();
        let step = self.submission_step();
        let result =
            if OperationRevision::from_schema(request.schema).ok() != Some(recorded_revision) {
                Err(BridgeRefusal::SubmissionSchema)
            } else if request.case != *step.case() {
                Err(BridgeRefusal::SubmissionCase)
            } else if request.subject != *step.subject() {
                Err(BridgeRefusal::SubmissionBytes)
            } else {
                Ok(())
            };
        terminal(&result, "validate_final_request");
        result
    }
}
/// Bind the final admission to its pre-signing structure and retained snapshots.
/// # Errors
/// Refuses changed frozen bytes, ordered inputs or capability snapshots.
pub fn complete_final_subject(
    frozen: &FrozenFinalSubject,
    request: KeyedConstructionRequest<'_, '_>,
    admitted: &KeyedAdmittedTransaction,
) -> Result<FinalSubject, BridgeRefusal> {
    let _span =
        tracing::debug_span!(target: "vectors::keyed_final_native", "complete_final_subject")
            .entered();
    let result = complete_core(frozen, request, admitted);
    terminal(&result, "complete_final_subject");
    result
}
fn complete_core(
    frozen: &FrozenFinalSubject,
    request: KeyedConstructionRequest<'_, '_>,
    admitted: &KeyedAdmittedTransaction,
) -> Result<FinalSubject, BridgeRefusal> {
    if admitted.transaction().encode_without_witness() != frozen.frozen_bytes() {
        return Err(BridgeRefusal::SignedBytes);
    }
    let rows = match request {
        KeyedConstructionRequest::Source(r) => r.inputs(),
        KeyedConstructionRequest::Merge(r) => r.inputs(),
    };
    if census(rows)? != frozen.inputs {
        return Err(BridgeRefusal::CapabilitySnapshot);
    }
    let bytes = admitted.transaction().encode();
    Ok(FinalSubject {
        frozen: frozen.clone(),
        digest: SubjectDigest(sha256(&bytes)),
        bytes,
        policy: admitted.policy_provenance().into(),
    })
}
fn terminal<T>(result: &Result<T, BridgeRefusal>, boundary: &'static str) {
    match result {
        Ok(_) => {
            tracing::debug!(target: "vectors::keyed_final_native", boundary, "final bridge completed");
        }
        Err(error) => {
            tracing::debug!(target: "vectors::keyed_final_native", boundary, refusal_kind=error.refusal_kind(), "final bridge refused");
        }
    }
}

/// Explicitly present or absent observations of one capture binary and environment.
///
/// Presence is a measurement claim, not proof of its truth; matching empty text
/// is not a measurement. Public all-zero key data remains valid.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CaptureProvenance {
    /// Runner records git rev-parse HEAD from the capture binary build checkout.
    pub repository_commit: Option<String>,
    /// Runner hashes and retains the resolved Cargo.lock used to build the capture binary.
    pub dependency_lock: Option<[u8; 32]>,
    /// Runner records rustc -Vv and cargo -V in the native build image.
    pub toolchain: Option<String>,
    /// Adapter handshake reports its executable name/version; runner compares the pinned implementation identity.
    pub executable: Option<String>,
    /// Adapter handshake reports framework source identity; runner compares the retained framework tree.
    pub framework: Option<String>,
    /// Runner hashes the adapter script bytes; must equal the rendered revision-nine comparison digest.
    pub executor: Option<[u8; 32]>,
    /// Adapter reports the handshake revision, retained by the capture binary without upgrading it.
    pub protocol: Option<u32>,
    /// Runner records the compute host identity using hostname, not an Elements chain identity.
    pub node: Option<String>,
    /// Runner records the immutable native image digest from podman image inspect.
    pub image: Option<String>,
    /// Capture binary computes `capture_freeze_digest` from its compiled-in source inventory; runner retains that self-report.
    pub freeze: Option<[u8; 32]>,
    /// Capture binary records the public supplied key bytes from the subject input; no secret handling.
    pub key: Option<[u8; 32]>,
    /// Runner hashes the exact capture test executable after its build, before launching it.
    pub capture_binary: Option<[u8; 32]>,
    /// Runner records the adapter interpreter executable identity, version and digest in the retained image.
    pub interpreter: Option<String>,
    /// Runner records the Elements daemon executable digest and version; adapter handshake independently names its upstream provenance.
    pub daemon: Option<String>,
}
/// Coordinate whose measurement is absent, independently of mismatch conditions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProvenanceField {
    /// Runner records git rev-parse HEAD from the capture binary build checkout.
    Repository,
    /// Runner hashes and retains the resolved Cargo.lock used to build the capture binary.
    Lock,
    /// Runner records rustc -Vv and cargo -V in the native build image.
    Toolchain,
    /// Adapter handshake reports its executable name/version; runner compares the pinned implementation identity.
    Executable,
    /// Adapter handshake reports framework source identity; runner compares the retained framework tree.
    Framework,
    /// Runner hashes the adapter script bytes; must equal the rendered revision-nine comparison digest.
    Executor,
    /// Adapter reports the handshake revision, retained by the capture binary without upgrading it.
    Protocol,
    /// Runner records the compute host identity using hostname, not an Elements chain identity.
    Node,
    /// Runner records the immutable native image digest from podman image inspect.
    Image,
    /// Capture binary computes `capture_freeze_digest` from its compiled-in source inventory; runner retains that self-report.
    Freeze,
    /// Capture binary records the public supplied key bytes from the subject input; no secret handling.
    Key,
    /// Runner hashes the exact capture test executable after its build, before launching it.
    CaptureBinary,
    /// Runner records the adapter interpreter executable identity, version and digest in the retained image.
    Interpreter,
    /// Runner records the Elements daemon executable digest and version; adapter handshake independently names its upstream provenance.
    Daemon,
}
/// Coordinate that prevents a capture from counting; no native verdict.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProvenanceRefusal {
    /// A required coordinate has no measurement on either side.
    Missing(ProvenanceField),
    /// Both offers agree on an adapter other than the rendered revision-nine source.
    ExecutorPin,
    /// Capture binary digest differs.
    CaptureBinary,
    /// Adapter interpreter identity differs.
    Interpreter,
    /// Elements daemon identity differs.
    Daemon,
    /// Repository build identity differs.
    Repository,
    /// Resolved dependency lock differs.
    Lock,
    /// Toolchain differs.
    Toolchain,
    /// Pinned executable differs.
    Executable,
    /// Framework differs.
    Framework,
    /// Adapter digest differs.
    Executor,
    /// Live protocol differs.
    Protocol,
    /// Node differs.
    Node,
    /// Image differs.
    Image,
    /// Freeze inventory differs.
    Freeze,
    /// Supplied key differs.
    Key,
}
impl ProvenanceRefusal {
    /// Stable trace condition.
    #[must_use]
    pub const fn refusal_kind(self) -> &'static str {
        match self {
            Self::Missing(_) => "missing",
            Self::ExecutorPin => "executor_pin",
            Self::CaptureBinary => "capture_binary",
            Self::Interpreter => "interpreter",
            Self::Daemon => "daemon",

            Self::Repository => "repository",
            Self::Lock => "lock",
            Self::Toolchain => "toolchain",
            Self::Executable => "executable",
            Self::Framework => "framework",
            Self::Executor => "executor",
            Self::Protocol => "protocol",
            Self::Node => "node",
            Self::Image => "image",
            Self::Freeze => "freeze",
            Self::Key => "key",
        }
    }
}
/// Match retained build/environment observations before accepting any capture.
/// This checks agreement, not the truth of a caller's observation.
/// # Errors
/// Returns the first differing coordinate in the declared field order.
pub fn bind_capture_provenance(
    expected: &CaptureProvenance,
    observed: &CaptureProvenance,
) -> Result<(), ProvenanceRefusal> {
    let _span =
        tracing::debug_span!(target: "vectors::keyed_final_native", "bind_capture_provenance")
            .entered();
    let result = provenance_core(expected, observed);
    match &result {
        Ok(()) => {
            tracing::debug!(target: "vectors::keyed_final_native", "capture provenance completed");
        }
        Err(error) => {
            tracing::debug!(target: "vectors::keyed_final_native", refusal_kind=error.refusal_kind(), "capture provenance refused");
        }
    }
    result
}
fn provenance_core(e: &CaptureProvenance, o: &CaptureProvenance) -> Result<(), ProvenanceRefusal> {
    use ProvenanceRefusal as R;
    if e.repository_commit.as_ref().is_none_or(String::is_empty)
        || o.repository_commit.as_ref().is_none_or(String::is_empty)
    {
        return Err(R::Missing(ProvenanceField::Repository));
    }
    if e.dependency_lock.is_none() || o.dependency_lock.is_none() {
        return Err(R::Missing(ProvenanceField::Lock));
    }
    if e.toolchain.as_ref().is_none_or(String::is_empty)
        || o.toolchain.as_ref().is_none_or(String::is_empty)
    {
        return Err(R::Missing(ProvenanceField::Toolchain));
    }
    if e.executable.as_ref().is_none_or(String::is_empty)
        || o.executable.as_ref().is_none_or(String::is_empty)
    {
        return Err(R::Missing(ProvenanceField::Executable));
    }
    if e.framework.as_ref().is_none_or(String::is_empty)
        || o.framework.as_ref().is_none_or(String::is_empty)
    {
        return Err(R::Missing(ProvenanceField::Framework));
    }
    if e.executor.is_none() || o.executor.is_none() {
        return Err(R::Missing(ProvenanceField::Executor));
    }
    if e.protocol.is_none() || o.protocol.is_none() {
        return Err(R::Missing(ProvenanceField::Protocol));
    }
    if e.node.as_ref().is_none_or(String::is_empty) || o.node.as_ref().is_none_or(String::is_empty)
    {
        return Err(R::Missing(ProvenanceField::Node));
    }
    if e.image.as_ref().is_none_or(String::is_empty)
        || o.image.as_ref().is_none_or(String::is_empty)
    {
        return Err(R::Missing(ProvenanceField::Image));
    }
    if e.freeze.is_none() || o.freeze.is_none() {
        return Err(R::Missing(ProvenanceField::Freeze));
    }
    if e.key.is_none() || o.key.is_none() {
        return Err(R::Missing(ProvenanceField::Key));
    }
    if e.capture_binary.is_none() || o.capture_binary.is_none() {
        return Err(R::Missing(ProvenanceField::CaptureBinary));
    }
    if e.interpreter.as_ref().is_none_or(String::is_empty)
        || o.interpreter.as_ref().is_none_or(String::is_empty)
    {
        return Err(R::Missing(ProvenanceField::Interpreter));
    }
    if e.daemon.as_ref().is_none_or(String::is_empty)
        || o.daemon.as_ref().is_none_or(String::is_empty)
    {
        return Err(R::Missing(ProvenanceField::Daemon));
    }
    provenance_equality(e, o)
}

fn provenance_equality(
    e: &CaptureProvenance,
    o: &CaptureProvenance,
) -> Result<(), ProvenanceRefusal> {
    use ProvenanceRefusal as R;
    if e.repository_commit != o.repository_commit {
        return Err(R::Repository);
    }
    if e.dependency_lock != o.dependency_lock {
        return Err(R::Lock);
    }
    if e.toolchain != o.toolchain {
        return Err(R::Toolchain);
    }
    if e.executable != o.executable {
        return Err(R::Executable);
    }
    if e.framework != o.framework {
        return Err(R::Framework);
    }
    if e.executor != o.executor {
        return Err(R::Executor);
    }
    if e.protocol != o.protocol {
        return Err(R::Protocol);
    }
    if e.node != o.node {
        return Err(R::Node);
    }
    if e.image != o.image {
        return Err(R::Image);
    }
    if e.freeze != o.freeze {
        return Err(R::Freeze);
    }
    if e.key != o.key {
        return Err(R::Key);
    }
    if e.capture_binary != o.capture_binary {
        return Err(R::CaptureBinary);
    }
    if e.interpreter != o.interpreter {
        return Err(R::Interpreter);
    }
    if e.daemon != o.daemon {
        return Err(R::Daemon);
    }
    let row = include_str!("../fixtures/adapter-request-comparisons/CURRENT-REVISION-9")
        .lines()
        .next()
        .ok_or(R::ExecutorPin)?;
    let digest = row.split_whitespace().nth(19).ok_or(R::ExecutorPin)?;
    let expected: [u8; 32] = crate::maturity_observer::decode_hex(digest)
        .map_err(|_| R::ExecutorPin)?
        .try_into()
        .map_err(|_| R::ExecutorPin)?;
    if e.executor != Some(expected) {
        return Err(R::ExecutorPin);
    }

    Ok(())
}

/// Digest a checked freeze inventory using its diagnostic Rust rendering.
///
/// This join is explicitly test-local: a rendering change requires a new capture;
/// it is neither a protocol identity nor a maintained descriptor format.
/// # Errors
/// Preserves the manifest owner's refusal before producing any digest.
pub fn capture_freeze_digest(
    key: [u8; 32],
) -> Result<[u8; 32], crate::keyed_final_manifest::FreezeRefusal> {
    let _span =
        tracing::debug_span!(target: "vectors::keyed_final_native", "capture_freeze_digest")
            .entered();
    let manifest = crate::keyed_final_manifest::FreezeManifest::offer(key);
    let result = crate::keyed_final_manifest::validate_manifest(&manifest, key)
        .map(|()| sha256(format!("keyed-capture-freeze-v1:{manifest:?}").as_bytes()));
    match &result {
        Ok(_) => tracing::debug!(target: "vectors::keyed_final_native", "capture freeze completed"),
        Err(error) => {
            tracing::debug!(target: "vectors::keyed_final_native", refusal_kind = error.refusal_kind(), "capture freeze refused");
        }
    }
    result
}

/// A journal-binding refusal, not an execution verdict or a chain assertion.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JournalRefusal {
    /// The packet has no recorded revision.
    MissingRevision,
    /// The packet names no known recorded grammar.
    UnknownRevision(u32),
    /// The recorded handshake and packet disagree.
    HandshakeRevision,
    /// A request differs from the packet's revision.
    RequestRevision(usize),
    /// A response differs from the packet's revision.
    ResponseRevision(usize),
    /// A response answers another case.
    Case(usize),
    /// A request's subject and case name different operation kinds.
    Kind(usize),
    /// The recorded response violates its own revision's shape.
    Shape {
        /// Exchange index in recorded order.
        index: usize,
        /// Preserved protocol-shape condition.
        refusal: ResponseShapeDefect,
    },
    /// The canonical sponsor parent does not bind a reported output.
    SponsorParent {
        /// Exchange index in recorded order.
        index: usize,
        /// Reported output index within the funding response.
        output: usize,
        /// Preserved parent-binding condition.
        refusal: SponsorParentRefusal,
    },
}
impl JournalRefusal {
    /// Stable condition, retaining owner errors in the value.
    #[must_use]
    pub const fn refusal_kind(&self) -> &'static str {
        match self {
            Self::MissingRevision => "missing_revision",
            Self::UnknownRevision(_) => "unknown_revision",
            Self::HandshakeRevision => "handshake_revision",
            Self::RequestRevision(_) => "request_revision",
            Self::ResponseRevision(_) => "response_revision",
            Self::Case(_) => "case",
            Self::Kind(_) => "kind",
            Self::Shape { .. } => "shape",
            Self::SponsorParent { .. } => "sponsor_parent",
        }
    }
}
/// Recorded exchanges bound to one packet and handshake revision.
///
/// This is a prerequisite to archive admission, not native acceptance or full admission.
/// Revision-8 funding retains its historical shape; revision-9 funding also
/// authenticates every reported sponsor output against the canonical parent.
/// ```compile_fail,E0451
/// use vectors::keyed_final_native::BoundCaptureJournal;
/// fn forge() -> BoundCaptureJournal { BoundCaptureJournal { revision: 9, exchanges: vec![] } }
/// ```
#[derive(Clone, Debug)]
pub struct BoundCaptureJournal {
    revision: u32,
    exchanges: Vec<(NativeOperationRequest, NativeOperationResponse)>,
}
impl BoundCaptureJournal {
    /// Recorded grammar, not whatever revision a future live executor speaks.
    #[must_use]
    pub const fn revision(&self) -> u32 {
        self.revision
    }
    /// Unmodified records after binding. The caller cannot edit them in place.
    #[must_use]
    pub fn exchanges(&self) -> &[(NativeOperationRequest, NativeOperationResponse)] {
        &self.exchanges
    }
}
/// Bind a recorded operation journal before it can be used as revision-9 archive
/// evidence. Existing revision-8 archive-specific readers are unchanged.
///
/// This checks local bytes and agreement with recorded provenance, not the truth
/// of the handshake, inclusion, unspentness or any execution observation.
/// # Errors
/// Refuses unknown packet grammar, handshake disagreement, then each row's
/// request revision, response revision, case, kind, shape and sponsor binding.
pub fn bind_capture_journal(
    packet: &CaptureProvenance,
    handshake: &ExecutorHandshake,
    exchanges: &[(NativeOperationRequest, NativeOperationResponse)],
) -> Result<BoundCaptureJournal, JournalRefusal> {
    let _span = tracing::debug_span!(target: "vectors::keyed_final_native", "bind_capture_journal")
        .entered();
    let result = journal_core(packet, handshake, exchanges);
    match &result {
        Ok(_) => {
            tracing::debug!(target: "vectors::keyed_final_native", "capture journal completed");
        }
        Err(error) => {
            tracing::debug!(target: "vectors::keyed_final_native", refusal_kind=error.refusal_kind(), "capture journal refused");
        }
    }
    result
}
fn journal_core(
    packet: &CaptureProvenance,
    handshake: &ExecutorHandshake,
    exchanges: &[(NativeOperationRequest, NativeOperationResponse)],
) -> Result<BoundCaptureJournal, JournalRefusal> {
    use JournalRefusal as R;
    let schema = packet.protocol.ok_or(R::MissingRevision)?;
    let revision =
        OperationRevision::from_schema(schema).map_err(|_| R::UnknownRevision(schema))?;
    if handshake.protocol_schema != schema {
        return Err(R::HandshakeRevision);
    }
    for (index, (request, response)) in exchanges.iter().enumerate() {
        if request.schema != schema {
            return Err(R::RequestRevision(index));
        }
        if response.schema != schema {
            return Err(R::ResponseRevision(index));
        }
        if request.case != response.case {
            return Err(R::Case(index));
        }
        if request.case.operation != request.subject.kind() {
            return Err(R::Kind(index));
        }
        response
            .validate_shape()
            .map_err(|refusal| R::Shape { index, refusal })?;
        if revision == OperationRevision::Nine
            && response.case.operation == OperationStepKind::FundSponsor
            && response.observed_layer == ObservedOutcomeLayer::Accepted
        {
            // Shape validation above requires the readback. Keep the owner's
            // refusal if a future shape implementation loses that requirement.
            let readback = response.mined_readback.as_ref().ok_or(R::Shape {
                index,
                refusal: ResponseShapeDefect::SponsorReadbackMissing,
            })?;
            for (output, funded) in response.funded_outputs.iter().enumerate() {
                bind_sponsor_parent(readback, funded).map_err(|refusal| R::SponsorParent {
                    index,
                    output,
                    refusal,
                })?;
            }
        }
    }
    Ok(BoundCaptureJournal {
        revision: schema,
        exchanges: exchanges.to_vec(),
    })
}

#[cfg(test)]
mod defensive_member_control {
    #[test]
    fn internal_member_witness_comparison_refuses_one_changed_byte() {
        let control = vec![vec![1, 2], vec![3, 4]];
        assert_eq!(
            super::check_member_witness(&control, &[1, 2], &[3, 4]),
            Ok(())
        );
        let mut subject = control;
        subject[0][1] ^= 1;
        assert_eq!(
            super::check_member_witness(&subject, &[1, 2], &[3, 4]),
            Err(super::BridgeRefusal::MemberWitness)
        );
    }
}
