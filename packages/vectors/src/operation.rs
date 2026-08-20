//! The one mapping from this package's own workload onto the executor's
//! step records (Guide-12 §16.2).
//!
//! [`RequiredTargetWork`] says what a target must do before any coverage
//! row can be discharged, in this package's vocabulary. The executor
//! boundary says what a target *can* be asked to do, in a target-generic
//! one: issue an asset, pay outputs to a witness program, submit a
//! transaction. This module is the single place the two meet, and
//! nothing else in the crate speaks the executor's vocabulary.
//!
//! # Nothing here tells the executor what to expect
//!
//! An [`OperationStep`] carries an identity and a subject and has no
//! member for an expected layer, an expected identity, or a class. The
//! planner therefore cannot hand the target the answer it is about to be
//! graded against, and the type is what says so rather than a convention
//! `(´[PLAN-rule:guide11-exec:request-subject]´)`.
//!
//! # Why the plan is consulted between steps
//!
//! A transaction cannot be built until the coins it spends exist, and
//! those coins are created by an earlier step of the same run against the
//! same chain. So the ceremony's answers are inputs to everything after
//! them: the asset the target issued decides which bundle is linked, the
//! bundle decides the constructor's output program, and the outpoints the
//! target reported decide what each vector spends.

use std::collections::BTreeMap;

use target_elements_conformance::executor::{OperationStep, PlanRefused, TargetOperationPlanner};
use target_elements_conformance::protocol::{
    NativeOperationResponse, ObservedOutcomeLayer, OperationCaseId, OperationStepKind,
    OperationSubject, TargetFundingSubject, TargetSubmissionSubject,
};
use transaction::{FundingCeremonyStep, Outpoint, Txid};

use crate::bundle::{FixtureBundle, ceremony_bundle};
use crate::error::VectorError;
use crate::fixture::{CompactAshSemanticCase, positive_semantic_census};
use crate::materialize::{
    AshFunding, MaterializedTargetVector, TargetVectorId, is_materializable, materialize, vector_id,
};

/// What each ceremony output carries while the asset is being issued.
///
/// The issuance step has to pay *somewhere*, and it pays before the
/// bundle that decides the real program can be linked at all. The
/// output it creates is deliberately not used by anything: it exists
/// because an issuance attaches to a transaction that has outputs, and
/// the asset identity is what the step is for.
const ISSUANCE_PROBE_AMOUNT: u64 = 1;

/// Why a plan could not state its next step.
///
/// The executor boundary's [`PlanRefused`] is a marker: a plan that
/// cannot continue knows why in its own vocabulary and that package
/// holds none for it. This is that vocabulary, read off the planner
/// after the run is refused.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum PlanRefusal {
    /// The issuance step reached no target verdict, so no asset exists.
    IssuanceDidNotHappen(ObservedOutcomeLayer),
    /// The target accepted the issuance but named no asset.
    IssuanceNamedNoAsset,
    /// The asset the target chose is not 32 bytes of hex.
    IssuedAssetUnreadable(String),
    /// A funding step reached no target verdict.
    FundingDidNotHappen(ObservedOutcomeLayer),
    /// A funding step created a different number of outputs than asked.
    FundingCardinalityWrong {
        /// How many outputs the step asked for.
        wanted: usize,
        /// How many it reported.
        reported: usize,
    },
    /// A funded output's outpoint is not one the target admits.
    FundedOutpointUnreadable(String),
    /// The target stored a script other than the program asked for.
    ///
    /// The whole content of the derive step: the program was computed
    /// here from this bundle's own tree, and this is the target saying
    /// what it actually created.
    ProgramMismatch {
        /// What the planner asked the target to pay to.
        asked: String,
        /// What the target reported storing.
        stored: String,
    },
    /// The ceremony-bound bundle could not be built or materialized.
    Bundle(Box<VectorError>),
}

/// What one submitted vector produced.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubmissionOutcome {
    vector: TargetVectorId,
    layer: ObservedOutcomeLayer,
    detail: Option<String>,
    accepted_txid: Option<String>,
    bytes: Vec<u8>,
}

impl SubmissionOutcome {
    /// The vector that was submitted.
    #[must_use]
    pub const fn vector(&self) -> TargetVectorId {
        self.vector
    }

    /// Where the target put it.
    ///
    /// §1.4 and §1.5 both live here: a consensus refusal, a script-path
    /// refusal, a relay refusal, and an acceptance are four different
    /// facts, and the two non-verdicts are not target facts at all.
    #[must_use]
    pub const fn layer(&self) -> ObservedOutcomeLayer {
        self.layer
    }

    /// What the target or the adapter said, verbatim and unmapped.
    #[must_use]
    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }

    /// The identity the target gave the transaction, where it took one.
    #[must_use]
    pub fn accepted_txid(&self) -> Option<&str> {
        self.accepted_txid.as_deref()
    }

    /// The exact bytes that were submitted.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

/// Everything one operation run established.
///
/// Built only by [`CompactAshOperationPlanner`], and only from answers
/// the executor recorded. Nothing here is an intention.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct OperationTranscript {
    issued_asset: Option<[u8; 32]>,
    constructor_program: Option<Vec<u8>>,
    funded: BTreeMap<TargetVectorId, Vec<Outpoint>>,
    submissions: Vec<SubmissionOutcome>,
    refusal: Option<PlanRefusal>,
}

impl OperationTranscript {
    /// The disposable asset the target issued, where it issued one.
    #[must_use]
    pub const fn issued_asset(&self) -> Option<[u8; 32]> {
        self.issued_asset
    }

    /// The constructor output program the ceremony funded.
    ///
    /// This is the real pin: the taproot output key of the
    /// ceremony-bound bundle's own committed tree, which the target
    /// created outputs at.
    #[must_use]
    pub fn constructor_program(&self) -> Option<&[u8]> {
        self.constructor_program.as_deref()
    }

    /// The coins the ceremony created for each vector.
    #[must_use]
    pub const fn funded(&self) -> &BTreeMap<TargetVectorId, Vec<Outpoint>> {
        &self.funded
    }

    /// What each submitted vector produced, in submission order.
    #[must_use]
    pub fn submissions(&self) -> &[SubmissionOutcome] {
        &self.submissions
    }

    /// Why the plan stopped, where it stopped early.
    #[must_use]
    pub const fn refusal(&self) -> Option<&PlanRefusal> {
        self.refusal.as_ref()
    }
}

/// One funding request the schedule holds.
#[derive(Clone, Debug)]
struct PlannedFunding {
    vector: TargetVectorId,
    member: usize,
    amount: u64,
}

/// Where the planner is.
#[derive(Clone, Debug)]
enum Stage {
    /// Ask the target to issue the disposable asset.
    Issue,
    /// Ask it to create one output at the derived constructor program.
    Derive,
    /// Work through the per-input funding schedule.
    Fund(usize),
    /// Work through the materialized vectors.
    Submit(usize),
    /// Nothing left to ask.
    Done,
}

/// The compact-ASH operation plan, as the executor consults it.
///
/// # How the thirteen work items map
///
/// The plan's [`RequiredTargetWork`] census is four ceremony steps and
/// nine vector submissions. The submissions map one-to-one onto
/// submission steps. The ceremony does not, and the mismatch is
/// deliberate rather than an approximation:
///
/// - `IssueDisposableTestAsset` is one funding step that issues.
/// - `DeriveConstructorOutputProgram` is one funding step at the program
///   this planner derived, whose answer is the target saying what it
///   stored there.
/// - `FundEachAshInput` is *many* funding steps, which is what the
///   census entry says: once per ASH input the fixture needs. A single
///   step could not express it, because a funding step states one amount
///   and the fixtures state different amounts per input.
/// - `RecordPublicView` is no step at all. Recording the outpoints and
///   their public fields is what this planner does with what the target
///   already reported; asking the target to do it would be asking it to
///   observe itself.
///
/// [`RequiredTargetWork`]: crate::plan::RequiredTargetWork
pub struct CompactAshOperationPlanner {
    stage: Stage,
    cases: Vec<CompactAshSemanticCase>,
    schedule: Vec<PlannedFunding>,
    bundle: Option<FixtureBundle>,
    probe_program: Vec<u8>,
    program: Vec<u8>,
    vectors: Vec<MaterializedTargetVector>,
    transcript: OperationTranscript,
}

impl CompactAshOperationPlanner {
    /// A planner for the sponsorless positive census.
    ///
    /// # Errors
    ///
    /// Any refusal from building the positive semantic census, which is
    /// a construction failure and never a target verdict.
    pub fn new() -> Result<Self, VectorError> {
        let cases: Vec<CompactAshSemanticCase> = positive_semantic_census()?
            .into_iter()
            .filter(is_materializable)
            .collect();
        let canonical = crate::bundle::fixture_bundle()?;
        let probe_program = canonical
            .pin()
            .output_script(canonical.target())
            .map_err(|cause| VectorError::TargetMaterializationFailed {
                vector: vector_id(&cases[0]),
                cause,
            })?;
        Ok(Self {
            stage: Stage::Issue,
            cases,
            schedule: Vec::new(),
            bundle: None,
            probe_program,
            program: Vec::new(),
            vectors: Vec::new(),
            transcript: OperationTranscript::default(),
        })
    }

    /// Everything the run established.
    #[must_use]
    pub const fn transcript(&self) -> &OperationTranscript {
        &self.transcript
    }

    /// The ceremony-bound bundle, once the asset is known.
    #[must_use]
    pub const fn bundle(&self) -> Option<&FixtureBundle> {
        self.bundle.as_ref()
    }

    /// The vectors materialized against the ceremony's own coins.
    #[must_use]
    pub fn vectors(&self) -> &[MaterializedTargetVector] {
        &self.vectors
    }

    fn refuse(&mut self, refusal: PlanRefusal) -> PlanRefused {
        self.transcript.refusal = Some(refusal);
        self.stage = Stage::Done;
        PlanRefused
    }

    /// The issuance step: one output, issuing the disposable asset.
    ///
    /// # Why it pays to the canonical program
    ///
    /// A funding step states a program, and the program the ceremony is
    /// actually for cannot be computed yet: it belongs to a bundle that
    /// is linked against the asset this very step is asking the target
    /// to choose. So the output goes to the canonical pinned program,
    /// which is a well-formed witness program of the right shape and
    /// which nothing in this run ever spends. The step's answer is what
    /// it is for — the identity the target chose — and the output it
    /// happens to create is not used by anything.
    fn issue_step(&self) -> OperationStep {
        OperationStep::new(
            "issue-disposable-test-asset",
            OperationSubject::Funding(Box::new(TargetFundingSubject {
                issue_asset: true,
                // Absent exactly when the step issues: the target has not
                // chosen the identity yet, so nothing here can name it.
                asset: None,
                output_program: self.probe_program.clone(),
                outputs: 1,
                amount_per_output: ISSUANCE_PROBE_AMOUNT,
            })),
        )
    }

    fn derive_step(&self) -> OperationStep {
        OperationStep::new(
            "derive-constructor-output-program",
            OperationSubject::Funding(Box::new(TargetFundingSubject {
                issue_asset: false,
                asset: self.transcript.issued_asset.map(hex_of),
                output_program: self.program.clone(),
                outputs: 1,
                amount_per_output: ISSUANCE_PROBE_AMOUNT,
            })),
        )
    }

    fn fund_step(&self, index: usize) -> Option<OperationStep> {
        let planned = self.schedule.get(index)?;
        Some(OperationStep::new(
            &format!(
                "fund-ash-input/{}/{}",
                planned.vector.fixture().ordinal(),
                planned.member
            ),
            OperationSubject::Funding(Box::new(TargetFundingSubject {
                issue_asset: false,
                asset: self.transcript.issued_asset.map(hex_of),
                output_program: self.program.clone(),
                outputs: 1,
                amount_per_output: planned.amount,
            })),
        ))
    }

    fn submit_step(&self, index: usize) -> Option<OperationStep> {
        let vector = self.vectors.get(index)?;
        Some(OperationStep::new(
            &format!("submit-vector/{}", vector.id().fixture().ordinal()),
            OperationSubject::Submission(Box::new(TargetSubmissionSubject {
                transaction_bytes: vector.bytes().to_vec(),
            })),
        ))
    }

    /// Read the issued asset, link the bundle, and build the schedule.
    fn settle_issuance(&mut self, response: &NativeOperationResponse) -> Result<(), PlanRefusal> {
        if response.observed_layer != ObservedOutcomeLayer::Accepted {
            return Err(PlanRefusal::IssuanceDidNotHappen(response.observed_layer));
        }
        let named = response
            .issued_asset
            .clone()
            .ok_or(PlanRefusal::IssuanceNamedNoAsset)?;
        let asset = asset_from_hex(&named)
            .ok_or_else(|| PlanRefusal::IssuedAssetUnreadable(named.clone()))?;
        self.transcript.issued_asset = Some(asset);

        // The bundle is linked against the asset the target chose, and
        // its pin is derived from its own tree. Neither could have been
        // stated before this answer arrived.
        let bundle =
            ceremony_bundle(asset).map_err(|cause| PlanRefusal::Bundle(Box::new(cause.into())))?;
        let program = bundle
            .pin()
            .output_script(bundle.target())
            .map_err(|cause| {
                PlanRefusal::Bundle(Box::new(VectorError::TargetMaterializationFailed {
                    vector: vector_id(&self.cases[0]),
                    cause,
                }))
            })?;
        self.program = program;
        self.bundle = Some(bundle);

        // One funding request per ASH input, in fixture order and then
        // member order, because materialization pairs the coins with the
        // amounts in exactly that order.
        self.schedule = self
            .cases
            .iter()
            .flat_map(|case| {
                let vector = vector_id(case);
                case.inputs()
                    .iter()
                    .enumerate()
                    .map(move |(member, amount)| PlannedFunding {
                        vector,
                        member,
                        amount: amount.get(),
                    })
                    .collect::<Vec<_>>()
            })
            .collect();
        Ok(())
    }

    /// Check the target stored the program the planner asked for.
    fn settle_derive(&mut self, response: &NativeOperationResponse) -> Result<(), PlanRefusal> {
        if response.observed_layer != ObservedOutcomeLayer::Accepted {
            return Err(PlanRefusal::FundingDidNotHappen(response.observed_layer));
        }
        let output =
            response
                .funded_outputs
                .first()
                .ok_or(PlanRefusal::FundingCardinalityWrong {
                    wanted: 1,
                    reported: response.funded_outputs.len(),
                })?;
        let asked = hex_of_slice(&self.program);
        if output.script != asked {
            return Err(PlanRefusal::ProgramMismatch {
                asked,
                stored: output.script.clone(),
            });
        }
        self.transcript.constructor_program = Some(self.program.clone());
        Ok(())
    }

    /// Record one funded coin against the vector it was cut for.
    fn settle_funding(
        &mut self,
        index: usize,
        response: &NativeOperationResponse,
    ) -> Result<(), PlanRefusal> {
        if response.observed_layer != ObservedOutcomeLayer::Accepted {
            return Err(PlanRefusal::FundingDidNotHappen(response.observed_layer));
        }
        if response.funded_outputs.len() != 1 {
            return Err(PlanRefusal::FundingCardinalityWrong {
                wanted: 1,
                reported: response.funded_outputs.len(),
            });
        }
        let output = &response.funded_outputs[0];
        let outpoint = outpoint_from_wire(&output.outpoint.txid, output.outpoint.vout)
            .ok_or_else(|| PlanRefusal::FundedOutpointUnreadable(output.outpoint.txid.clone()))?;
        let planned = &self.schedule[index];
        self.transcript
            .funded
            .entry(planned.vector)
            .or_default()
            .push(outpoint);
        Ok(())
    }

    /// Materialize every vector against the coins the ceremony created.
    fn materialize_all(&mut self) -> Result<(), PlanRefusal> {
        let bundle = self
            .bundle
            .clone()
            .ok_or(PlanRefusal::IssuanceNamedNoAsset)?;
        let mut vectors = Vec::with_capacity(self.cases.len());
        for case in &self.cases {
            let id = vector_id(case);
            let outpoints = self.transcript.funded.get(&id).cloned().unwrap_or_default();
            let funding = AshFunding::new(id, outpoints)
                .map_err(|cause| PlanRefusal::Bundle(Box::new(cause)))?;
            let vector = materialize(&bundle, case, &funding)
                .map_err(|cause| PlanRefusal::Bundle(Box::new(cause)))?;
            vectors.push(vector);
        }
        self.vectors = vectors;
        Ok(())
    }

    fn settle_submission(&mut self, index: usize, response: &NativeOperationResponse) {
        // A submission answer is never a refusal. Every layer the target
        // can reach is a fact about the target, and the two that are not
        // target facts are recorded as themselves rather than dropped
        // `(´[PLAN-rule:guide12-exec:failure-layers]´)`.
        if let Some(vector) = self.vectors.get(index) {
            self.transcript.submissions.push(SubmissionOutcome {
                vector: vector.id(),
                layer: response.observed_layer,
                detail: response.observed_detail.clone(),
                accepted_txid: response.accepted_txid.clone(),
                bytes: vector.bytes().to_vec(),
            });
        }
    }
}

impl TargetOperationPlanner for CompactAshOperationPlanner {
    fn next_step(
        &mut self,
        previous: Option<(&OperationCaseId, &NativeOperationResponse)>,
    ) -> Result<Option<OperationStep>, PlanRefused> {
        // Settle the previous answer before deciding what to ask next.
        // The stage is what says which answer this was; the case identity
        // is echoed back by the executor and is not re-parsed here,
        // because reading the stage out of a name the planner itself
        // chose would be this package agreeing with itself.
        if let Some((_case, response)) = previous {
            match self.stage {
                Stage::Issue => {
                    if let Err(refusal) = self.settle_issuance(response) {
                        return Err(self.refuse(refusal));
                    }
                    self.stage = Stage::Derive;
                }
                Stage::Derive => {
                    if let Err(refusal) = self.settle_derive(response) {
                        return Err(self.refuse(refusal));
                    }
                    self.stage = Stage::Fund(0);
                }
                Stage::Fund(index) => {
                    if let Err(refusal) = self.settle_funding(index, response) {
                        return Err(self.refuse(refusal));
                    }
                    let next = index + 1;
                    if next < self.schedule.len() {
                        self.stage = Stage::Fund(next);
                    } else {
                        if let Err(refusal) = self.materialize_all() {
                            return Err(self.refuse(refusal));
                        }
                        self.stage = Stage::Submit(0);
                    }
                }
                Stage::Submit(index) => {
                    self.settle_submission(index, response);
                    let next = index + 1;
                    self.stage = if next < self.vectors.len() {
                        Stage::Submit(next)
                    } else {
                        Stage::Done
                    };
                }
                Stage::Done => {}
            }
        }

        Ok(match self.stage {
            Stage::Issue => Some(self.issue_step()),
            Stage::Derive => Some(self.derive_step()),
            Stage::Fund(index) => self.fund_step(index),
            Stage::Submit(index) => self.submit_step(index),
            Stage::Done => None,
        })
    }
}

/// The target's own spelling of a 32-byte identity.
///
/// A target prints an asset identity in the reverse of the order it
/// commits to it in, exactly as it does for a transaction identity. The
/// transcript holds the committed order, because that is the order the
/// linked programs introspect and the order the constructor writes into
/// an explicit asset field; the reversal happens here and in
/// [`asset_from_hex`], at the two points where the target's spelling
/// crosses the boundary.
fn hex_of(bytes: [u8; 32]) -> String {
    let mut printed = bytes;
    printed.reverse();
    hex_of_slice(&printed)
}

fn hex_of_slice(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut text = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        // The width is fixed and the sink is a `String`, so the write
        // cannot fail; the result is consumed rather than unwrapped so
        // that no formatting path can panic.
        let _ = write!(text, "{byte:02x}");
    }
    text
}

fn bytes_from_hex(text: &str) -> Option<Vec<u8>> {
    let raw = text.as_bytes();
    let (pairs, remainder) = raw.as_chunks::<2>();
    if !remainder.is_empty() {
        return None;
    }
    let mut bytes = Vec::with_capacity(pairs.len());
    for pair in pairs {
        let digits = std::str::from_utf8(pair).ok()?;
        bytes.push(u8::from_str_radix(digits, 16).ok()?);
    }
    Some(bytes)
}

/// One asset identity, from the spelling the target printed.
///
/// Reversed into the committed order. See [`hex_of`].
fn asset_from_hex(text: &str) -> Option<[u8; 32]> {
    let mut bytes = bytes_from_hex(text)?;
    bytes.reverse();
    <[u8; 32]>::try_from(bytes.as_slice()).ok()
}

/// One outpoint, from the identity the target printed.
///
/// A target prints a transaction identity in the reverse of the order it
/// hashes it in, and this crate's `Txid` holds the hashed order. The
/// reversal is performed here, once, at the boundary where the target's
/// spelling arrives.
fn outpoint_from_wire(txid: &str, vout: u32) -> Option<Outpoint> {
    let mut bytes = bytes_from_hex(txid)?;
    bytes.reverse();
    let internal = <[u8; 32]>::try_from(bytes.as_slice()).ok()?;
    Outpoint::new(Txid::from_internal(internal), vout).ok()
}

/// The ceremony census, restated nowhere.
///
/// The planner's step names are its own, and the census entry each one
/// serves is stated here so a reader can check the mapping the module
/// documentation describes against something mechanical.
#[must_use]
pub const fn ceremony_census() -> &'static [FundingCeremonyStep] {
    FundingCeremonyStep::ALL
}

/// Whether a step identity names a funding step.
#[must_use]
pub fn is_funding(case: &OperationCaseId) -> bool {
    case.operation == OperationStepKind::Fund
}
