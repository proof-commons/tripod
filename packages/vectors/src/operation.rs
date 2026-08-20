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

use std::collections::{BTreeMap, BTreeSet};

use target_elements_conformance::executor::{OperationStep, PlanRefused, TargetOperationPlanner};
use target_elements_conformance::protocol::{
    NativeOperationResponse, ObservedOutcomeLayer, OperationCaseId, OperationStepKind,
    OperationSubject, TargetFundingSubject, TargetSubmissionSubject,
};
use transaction::{FundingCeremonyStep, Outpoint, Txid};

use crate::bundle::{FixtureBundle, ceremony_bundle};
use crate::divergence::{AmountBeyondTargetBound, target_amount_standing};
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
    /// The target created a coin its own bound says it cannot state.
    ///
    /// The opposite surprise to a funding failure, and refused just as
    /// hard. A row is asked for only because the reviewed bound says
    /// the target cannot hold that amount; a target that holds it
    /// anyway has falsified the reviewed fact the classification was
    /// derived from, and continuing would build a run on a reading of
    /// the target that is known to be wrong.
    AmountBeyondBoundWasFunded {
        /// The vector the coin was cut for.
        vector: TargetVectorId,
        /// The amount the step asked the target to state.
        stated: u64,
        /// The bound the reviewed target facts publish.
        bound: u64,
    },
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

/// One target-bound divergence, as the target answered it.
///
/// # Why the answer is recorded and not just the classification
///
/// The classification is a comparison of two numbers this workspace
/// holds, and on its own it establishes nothing about a running node.
/// The divergence becomes a fact about the target when the target is
/// asked to create the coin and does not. Both halves are therefore
/// kept: what was derived, and what came back.
///
/// The layer is deliberately not constrained to one value. The reviewed
/// bound says the amount cannot be stated; which of the target's gates
/// says so first is the target's business, and on this path an
/// adapter's own reserve arithmetic can answer ahead of every one of
/// them. What is required is that the step did not succeed, and the
/// layer and detail record which way it failed
/// `(´[PLAN-rule:guide12-exec:failure-layers]´)`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObservedDivergence {
    vector: TargetVectorId,
    beyond: AmountBeyondTargetBound,
    layer: ObservedOutcomeLayer,
    detail: Option<String>,
}

impl ObservedDivergence {
    /// The vector whose funding the target's bound forbids.
    #[must_use]
    pub const fn vector(&self) -> TargetVectorId {
        self.vector
    }

    /// The amount, its place in the row, and the bound it overshoots.
    #[must_use]
    pub const fn beyond(&self) -> AmountBeyondTargetBound {
        self.beyond
    }

    /// Where the target put the step that was asked anyway.
    #[must_use]
    pub const fn layer(&self) -> ObservedOutcomeLayer {
        self.layer
    }

    /// What the target or the adapter said, verbatim and unmapped.
    #[must_use]
    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
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
    divergences: Vec<ObservedDivergence>,
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

    /// Every row the target's own bound refused to fund.
    ///
    /// Empty for a run whose census the target admits whole. A
    /// non-empty list is evidence of a divergence between the
    /// protocol's amount domain and this target's, and is never a
    /// submission result: nothing was built, so nothing was judged.
    #[must_use]
    pub fn divergences(&self) -> &[ObservedDivergence] {
        &self.divergences
    }

    /// Why the plan stopped, where it stopped early.
    #[must_use]
    pub const fn refusal(&self) -> Option<&PlanRefusal> {
        self.refusal.as_ref()
    }
}

/// What the ceremony is asking a funding step for.
///
/// # Why an expectation exists here and nowhere near the step
///
/// The [`OperationStep`] this produces carries an identity and a
/// subject, exactly as before: no expected layer crosses the boundary,
/// and the executor is told nothing about which answer would be the
/// interesting one. The expectation is the planner's own reading of
/// what it is about to ask, kept on this side so that an answer can be
/// compared against something rather than merely recorded.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FundingExpectation {
    /// A coin the vector will spend.
    Coin,
    /// A coin the reviewed target bound says the target cannot state.
    ///
    /// Asked anyway, and asked exactly once per divergent row. The
    /// classification is arithmetic over two numbers this workspace
    /// holds; without putting it to the target there is no observation
    /// behind the claim, only a calculation asserting one.
    BeyondBound(AmountBeyondTargetBound),
}

/// One funding request the schedule holds.
#[derive(Clone, Copy, Debug)]
struct PlannedFunding {
    vector: TargetVectorId,
    member: usize,
    amount: u64,
    expectation: FundingExpectation,
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
/// The plan's [`RequiredTargetWork`] census is four ceremony steps, one
/// per sponsorless vector after that, and — for a row the reviewed
/// target bound forbids — a divergence probe in the submission's place.
/// The submissions map one-to-one onto submission steps. The ceremony
/// does not, and the mismatch is deliberate rather than an
/// approximation:
///
/// - `IssueDisposableTestAsset` is one funding step that issues.
/// - `DeriveConstructorOutputProgram` is one funding step at the program
///   this planner derived, whose answer is the target saying what it
///   stored there.
/// - `FundEachAshInput` is *many* funding steps, which is what the
///   census entry says: once per ASH input the fixture needs. A single
///   step could not express it, because a funding step states one amount
///   and the fixtures state different amounts per input. A row the
///   reviewed target bound cannot state gets one step instead of one
///   per input, and that step's whole purpose is the refusal it
///   collects `(´[PLAN-rule:guide12-exec:failure-layers]´)`.
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
    unfundable: BTreeSet<TargetVectorId>,
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
            unfundable: BTreeSet::new(),
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
        //
        // Except for a row the reviewed target bound says cannot be
        // funded at all. That row gets exactly one step — the first
        // input the bound forbids — and none of the rest, because the
        // remaining coins would be cut for a transaction that is never
        // built. The one step is asked so that the divergence is
        // observed rather than assumed.
        self.schedule.clear();
        self.unfundable.clear();
        for case in &self.cases {
            let vector = vector_id(case);
            match target_amount_standing(case).unfundable_input() {
                Some((member, beyond)) => {
                    self.unfundable.insert(vector);
                    self.schedule.push(PlannedFunding {
                        vector,
                        member,
                        amount: beyond.stated(),
                        expectation: FundingExpectation::BeyondBound(beyond),
                    });
                }
                None => {
                    for (member, amount) in case.inputs().iter().enumerate() {
                        self.schedule.push(PlannedFunding {
                            vector,
                            member,
                            amount: amount.get(),
                            expectation: FundingExpectation::Coin,
                        });
                    }
                }
            }
        }
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
    ///
    /// # Why one row's refusal is not the run's refusal
    ///
    /// An unexpected funding failure still stops everything: the coins
    /// a later step depends on would not exist, and continuing would
    /// submit transactions against outputs no chain holds. The one
    /// exception is a step the reviewed target bound already said could
    /// not succeed. There the refusal is the answer the step was asked
    /// for, the row it belongs to is excluded from everything
    /// downstream, and the rest of the census carries on — which is the
    /// difference between a target that cannot state one amount and a
    /// ceremony that has broken.
    fn settle_funding(
        &mut self,
        index: usize,
        response: &NativeOperationResponse,
    ) -> Result<(), PlanRefusal> {
        let planned = self.schedule[index];
        if let FundingExpectation::BeyondBound(beyond) = planned.expectation {
            if response.observed_layer == ObservedOutcomeLayer::Accepted {
                return Err(PlanRefusal::AmountBeyondBoundWasFunded {
                    vector: planned.vector,
                    stated: beyond.stated(),
                    bound: beyond.bound(),
                });
            }
            self.transcript.divergences.push(ObservedDivergence {
                vector: planned.vector,
                beyond,
                layer: response.observed_layer,
                detail: response.observed_detail.clone(),
            });
            return Ok(());
        }
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

    /// Materialize every vector the ceremony actually created coins for.
    ///
    /// A row the target's bound refused is skipped rather than built
    /// against a short funding record. There is nothing to build: the
    /// coins do not exist, and a transaction naming outpoints no chain
    /// created is the very thing Wave 11 stopped producing.
    fn materialize_all(&mut self) -> Result<(), PlanRefusal> {
        let bundle = self
            .bundle
            .clone()
            .ok_or(PlanRefusal::IssuanceNamedNoAsset)?;
        let mut vectors = Vec::with_capacity(self.cases.len());
        for case in &self.cases {
            let id = vector_id(case);
            if self.unfundable.contains(&id) {
                continue;
            }
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

#[cfg(test)]
mod tests {
    use super::{CompactAshOperationPlanner, PlanRefusal, hex_of_slice, is_funding};
    use target_elements_conformance::executor::TargetOperationPlanner;
    use target_elements_conformance::protocol::{
        FundedOutput, NATIVE_PROTOCOL_SCHEMA, NativeOperationResponse, NativeResourceObservation,
        ObservedOutcomeLayer, OperationCaseId, OperationStepKind, OperationSubject, WireOutpoint,
    };

    /// A disposable asset identity, in the target's own spelling.
    ///
    /// A fixed value naming a coin on no chain, which authorizes
    /// nothing `(´[ADR015-rule:security:test-material]´)`.
    const ISSUED: &str = "11223344556677889900aabbccddeeff11223344556677889900aabbccddeeff";

    fn resources() -> NativeResourceObservation {
        NativeResourceObservation {
            script_bytes: 0,
            initial_stack_items: 0,
            peak_stack_items: None,
            peak_altstack_items: None,
            maximum_element_bytes: None,
            validation_budget_used: None,
            transaction_weight: None,
        }
    }

    /// One coin identity per funding answer, so no two collide.
    fn coin(sequence: u32) -> WireOutpoint {
        let mut bytes = [0_u8; 32];
        bytes[0..4].copy_from_slice(&sequence.to_be_bytes());
        WireOutpoint {
            txid: hex_of_slice(&bytes),
            vout: 0,
        }
    }

    /// The bound the reviewed target facts publish.
    fn bound() -> u64 {
        target_elements::reviewed_stated_amount_bound().maximum()
    }

    /// What the fake target should do with the step it was handed.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum Answer {
        /// Do what the step asked.
        Perform,
        /// Report that the step did not happen.
        Refuse(ObservedOutcomeLayer),
    }

    /// A fake target that answers exactly what a step asks for.
    ///
    /// It creates no chain and validates nothing. Its whole job is to
    /// produce well-shaped answers so that the *planner's* reaction to
    /// them can be tested, which is the half of the exchange this
    /// module owns.
    struct FakeTarget<F: FnMut(&OperationCaseId, u64) -> Answer> {
        sequence: u32,
        decide: F,
    }

    impl<F: FnMut(&OperationCaseId, u64) -> Answer> FakeTarget<F> {
        fn answer(
            &mut self,
            case: &OperationCaseId,
            subject: &OperationSubject,
        ) -> NativeOperationResponse {
            let mut response = NativeOperationResponse {
                schema: NATIVE_PROTOCOL_SCHEMA,
                case: case.clone(),
                observed_layer: ObservedOutcomeLayer::Accepted,
                observed_detail: None,
                issued_asset: None,
                funded_outputs: Vec::new(),
                accepted_txid: None,
                resources: resources(),
            };
            match subject {
                OperationSubject::Funding(funding) => {
                    if let Answer::Refuse(layer) = (self.decide)(case, funding.amount_per_output) {
                        response.observed_layer = layer;
                        response.observed_detail = Some(
                            "the step asks for more of the asset than this run holds".to_owned(),
                        );
                        return response;
                    }
                    self.sequence += 1;
                    if funding.issue_asset {
                        response.issued_asset = Some(ISSUED.to_owned());
                    }
                    response.funded_outputs = (0..u32::from(funding.outputs))
                        .map(|index| FundedOutput {
                            outpoint: coin(self.sequence * 16 + index),
                            asset: ISSUED.to_owned(),
                            amount_satoshis: funding.amount_per_output,
                            script: hex_of_slice(&funding.output_program),
                        })
                        .collect();
                }
                OperationSubject::Submission(_) => {
                    response.accepted_txid = Some(ISSUED.to_owned());
                }
            }
            response
        }
    }

    /// Drive a planner to completion against a fake target.
    ///
    /// Returns the planner and whether the plan ran out of steps rather
    /// than refusing.
    fn run<F: FnMut(&OperationCaseId, u64) -> Answer>(
        decide: F,
    ) -> (CompactAshOperationPlanner, bool) {
        let mut planner = CompactAshOperationPlanner::new().expect("the planner builds");
        let mut target = FakeTarget {
            sequence: 0,
            decide,
        };
        let mut previous: Option<(OperationCaseId, NativeOperationResponse)> = None;
        loop {
            let borrowed = previous.as_ref().map(|(case, response)| (case, response));
            let step = match planner.next_step(borrowed) {
                Ok(Some(step)) => step,
                Ok(None) => return (planner, true),
                Err(_) => return (planner, false),
            };
            let response = target.answer(step.case(), step.subject());
            response
                .validate_shape()
                .expect("the fake target answers in a shape the protocol defines");
            previous = Some((step.case().clone(), response));
        }
    }

    /// The answer a run gives to a step the reviewed bound forbids.
    fn refuse_beyond_bound(case: &OperationCaseId, amount: u64) -> Answer {
        if is_funding(case) && amount > bound() {
            Answer::Refuse(ObservedOutcomeLayer::ExecutorInfrastructureFailure)
        } else {
            Answer::Perform
        }
    }

    #[test]
    fn the_divergent_row_is_asked_for_once_and_the_run_continues() {
        // The whole behaviour in one run: the step the bound forbids is
        // asked, refused, recorded, and the remaining rows are funded
        // and submitted anyway.
        let (planner, completed) = run(refuse_beyond_bound);
        assert!(completed, "an expected refusal must not stop the run");

        let transcript = planner.transcript();
        assert_eq!(transcript.refusal(), None);
        assert_eq!(transcript.divergences().len(), 1);

        let divergence = &transcript.divergences()[0];
        assert_eq!(
            divergence.vector().fixture().name(),
            "values-summing-to-two-pow-51-minus-one"
        );
        assert_eq!(divergence.beyond().stated(), (1 << 51) - 2);
        assert_eq!(divergence.beyond().bound(), bound());
        assert_eq!(
            divergence.layer(),
            ObservedOutcomeLayer::ExecutorInfrastructureFailure
        );
        assert!(divergence.detail().is_some(), "a refusal states a reason");

        // Eight submissions, not nine, and the divergent row is not
        // among them: it was never built, so there was nothing to hand
        // over.
        assert_eq!(transcript.submissions().len(), 8);
        assert_eq!(planner.vectors().len(), 8);
        for submission in transcript.submissions() {
            assert_ne!(
                submission.vector().fixture().name(),
                "values-summing-to-two-pow-51-minus-one",
                "a row with no coins was submitted anyway"
            );
        }
    }

    #[test]
    fn the_divergent_row_costs_one_funding_step_and_not_its_whole_family() {
        // Its family has two members and only the forbidden one is
        // asked for: the second coin would be cut for a transaction
        // that is never built.
        let recorded = std::cell::RefCell::new(Vec::new());
        let (_, completed) = run(|case, amount| {
            if is_funding(case) {
                recorded.borrow_mut().push(amount);
            }
            refuse_beyond_bound(case, amount)
        });
        assert!(completed);
        let asked = recorded.into_inner();
        assert_eq!(asked.iter().filter(|amount| **amount > bound()).count(), 1);

        // The schedule's own length, recomputed from the census rather
        // than pinned as a bare number: every sponsorless row's inputs,
        // except the divergent row which contributes one, plus the two
        // ceremony steps that also fund.
        let census = crate::fixture::positive_semantic_census().expect("the census builds");
        let scheduled: usize = census
            .iter()
            .filter(|case| crate::materialize::is_materializable(case))
            .map(|case| {
                if crate::divergence::target_amount_standing(case).is_unfundable() {
                    1
                } else {
                    case.inputs().len()
                }
            })
            .sum();
        assert_eq!(asked.len(), scheduled + 2);
    }

    #[test]
    fn a_target_that_funds_the_forbidden_amount_refuses_the_plan() {
        // The other half of the expectation. The step is asked only
        // because the reviewed bound says it cannot succeed, so a
        // target that succeeds has falsified the fact the
        // classification was derived from, and the run must not
        // continue on a reading of the target known to be wrong.
        let (planner, completed) = run(|_, _| Answer::Perform);
        assert!(!completed, "the surprise must refuse the run");
        match planner.transcript().refusal() {
            Some(PlanRefusal::AmountBeyondBoundWasFunded {
                vector,
                stated,
                bound: reported,
            }) => {
                assert_eq!(
                    vector.fixture().name(),
                    "values-summing-to-two-pow-51-minus-one"
                );
                assert_eq!(*stated, (1 << 51) - 2);
                assert_eq!(*reported, bound());
            }
            other => panic!("the plan refused with {other:?}"),
        }
    }

    #[test]
    fn an_unexpected_funding_refusal_still_refuses_the_whole_plan() {
        // The conservative rule is not weakened generally: only the row
        // the bound names may fail. Any other failure leaves coins
        // missing that later steps depend on.
        let (planner, completed) = run(|case, amount| {
            if is_funding(case) && amount == 180 {
                Answer::Refuse(ObservedOutcomeLayer::ExecutorInfrastructureFailure)
            } else {
                Answer::Perform
            }
        });
        assert!(!completed);
        assert_eq!(
            planner.transcript().refusal(),
            Some(&PlanRefusal::FundingDidNotHappen(
                ObservedOutcomeLayer::ExecutorInfrastructureFailure
            ))
        );
        assert!(
            planner.transcript().divergences().is_empty(),
            "an ordinary failure is not a divergence"
        );
    }

    #[test]
    fn no_step_the_planner_states_carries_an_expectation() {
        // §16.2's forbidden direction, checked against the rendering of
        // every step this planner actually produces rather than against
        // the field list. The divergence machinery added an expectation
        // to the *plan*, and this is what says none of it leaked into
        // what crosses the boundary.
        let mut planner = CompactAshOperationPlanner::new().expect("the planner builds");
        let mut target = FakeTarget {
            sequence: 0,
            decide: refuse_beyond_bound,
        };
        let mut previous: Option<(OperationCaseId, NativeOperationResponse)> = None;
        let mut steps = 0_usize;
        while let Ok(Some(step)) =
            planner.next_step(previous.as_ref().map(|(case, response)| (case, response)))
        {
            let rendered = format!("{step:?}").to_lowercase();
            for forbidden in ["expect", "refus", "bound", "diverg", "beyond", "accept"] {
                assert!(!rendered.contains(forbidden), "a step rendered {forbidden}");
            }
            assert!(matches!(
                step.case().operation,
                OperationStepKind::Fund | OperationStepKind::Submit
            ));
            let response = target.answer(step.case(), step.subject());
            previous = Some((step.case().clone(), response));
            steps += 1;
        }
        assert!(steps > 0, "the planner stated no steps at all");
    }
}
