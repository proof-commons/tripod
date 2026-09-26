//! Exact spend-time writes over an honest signed maturity announcement, or over a separately funded predecessor coin.
//!
//! A staged mutant gives one safety-matrix row concrete offerable bytes without adding a production constructor parameter. It establishes no node verdict and does not claim that a host projection clause is the clause a node reaches.
//! Written positions record the stage's acts; changed positions record bytes that differ from the honest submission and are computed at staging, because a derived write can coincide with an honest byte.
//! On the archived variable-schedule fixture, three derived writes coincide: predecessor wrong-internal-key item 5 carries the foreign parity, foreign-subtree successor item 1 carries the common nonce, and successor wrong-internal-key item 0 carries the foreign parity; each is written and pinned by value.

use linker::{CandidateDeploymentIdentity, CandidateLinkedMaturityBundle};
use realization::{EncodedStateMetadata, StateRepresentationNonce, TransactionSide};
use tapscript::{
    StateConstructorRefusal, StateCurveCapability, StateLeafRole, StateTweakOutcome,
    StateWitnessSchedule,
};
use target_elements::ReviewedElementsTapscriptDefinition;
use target_elements_conformance::test_material::TestSigningDefect;
use transaction::bytes::{
    AssetField, InputWitness, Outpoint, TargetInput, TargetOutput, TargetTransaction, ValueField,
};
use transaction::error::TransactionRefusal;
use transaction::operator_right::BranchContext;
use transaction::operator_signing::{
    OperatorSigningInput, OperatorSigningRefusal, OperatorSigningRequest,
};
use transaction::script_path_signing::{LiveDeployment, SpentOutputCensusEntry};
use transaction::state_finalize::FinalizedMaturityAnnouncement;

use crate::live_capability::OracleLiveCurve;
use crate::matrix::EvidenceBoundary;
use crate::maturity_closure::{
    MaturityClosureRefusal, MaturityDeployment, MaturityWitnessSelection, OracleStateCurve,
    maturity_sources,
};
use crate::maturity_continuity::{
    MaturityConstructorProjection, MaturityContinuityRefusal, MaturityFundedPredecessor,
    fixed_projection, prefix, witness,
};
use crate::maturity_operator::OPERATOR_HANDLE;
use crate::maturity_safety::{
    MaturityMutationLocator, MaturitySafetyRow, MaturitySafetySection, rows,
};

/// The seven offerable constructor-fault rows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MaturityMutantRow {
    /// A predecessor funded under another internal key.
    PredecessorWrongInternalKey,
    /// A predecessor parity witness that disagrees with its honest control path.
    PredecessorWrongControlBlock,
    /// The metadata commitment leaf selected as the executing leaf.
    MetadataLeafSelectedForExecution,
    /// A single-item key-path offer over the predecessor program.
    KeyPathSpendAttempt,
    /// A successor committed under another deployment's static subtree.
    SuccessorUnderAnotherStaticSubtree,
    /// A successor committed under another internal key.
    SuccessorWrongInternalKey,
    /// A successor parity witness that disagrees with its output key.
    SuccessorWrongParity,
}

/// The published generator point used only to derive a foreign-key test coin and output.
/// Its key path is never offered.
pub const FOREIGN_INTERNAL_KEY: [u8; 32] = tapscript::STATE_GENERATOR_X;

impl MaturityMutantRow {
    /// Every offered row, in the two table runs' order.
    pub const ALL: [Self; 7] = [
        Self::PredecessorWrongInternalKey,
        Self::PredecessorWrongControlBlock,
        Self::MetadataLeafSelectedForExecution,
        Self::KeyPathSpendAttempt,
        Self::SuccessorUnderAnotherStaticSubtree,
        Self::SuccessorWrongInternalKey,
        Self::SuccessorWrongParity,
    ];

    /// The §16 table containing this row.
    #[must_use]
    pub const fn section(self) -> MaturitySafetySection {
        match self {
            Self::PredecessorWrongInternalKey
            | Self::PredecessorWrongControlBlock
            | Self::MetadataLeafSelectedForExecution
            | Self::KeyPathSpendAttempt => MaturitySafetySection::PredecessorConstructorFault,
            Self::SuccessorUnderAnotherStaticSubtree
            | Self::SuccessorWrongInternalKey
            | Self::SuccessorWrongParity => MaturitySafetySection::SuccessorConstructorFault,
        }
    }

    /// The matrix row's stable name.
    #[must_use]
    pub const fn row(self) -> &'static str {
        match self {
            Self::PredecessorWrongInternalKey | Self::SuccessorWrongInternalKey => {
                "wrong-internal-key"
            }
            Self::PredecessorWrongControlBlock => "wrong-control-block",
            Self::MetadataLeafSelectedForExecution => "metadata-leaf-selected-for-execution",
            Self::KeyPathSpendAttempt => "key-path-spend-attempt",
            Self::SuccessorUnderAnotherStaticSubtree => "successor-under-another-static-subtree",
            Self::SuccessorWrongParity => "wrong-parity",
        }
    }

    /// The ceremony operation name for this row.
    #[must_use]
    pub const fn step(self) -> &'static str {
        match self {
            Self::PredecessorWrongInternalKey => "mutant-predecessor-wrong-internal-key",
            Self::PredecessorWrongControlBlock => "mutant-predecessor-wrong-control-block",
            Self::MetadataLeafSelectedForExecution => "mutant-metadata-leaf-selected-for-execution",
            Self::KeyPathSpendAttempt => "mutant-key-path-spend-attempt",
            Self::SuccessorUnderAnotherStaticSubtree => {
                "mutant-successor-under-another-static-subtree"
            }
            Self::SuccessorWrongInternalKey => "mutant-successor-wrong-internal-key",
            Self::SuccessorWrongParity => "mutant-successor-wrong-parity",
        }
    }

    /// Read the row by its table and name, rather than its current ordinal.
    #[must_use]
    pub fn matrix_row(self) -> &'static MaturitySafetyRow {
        let Some(entry) = rows()
            .iter()
            .find(|entry| (entry.section(), entry.name()) == (self.section(), self.row()))
        else {
            std::process::abort()
        };
        entry
    }

    /// The matrix's structural change site.
    ///
    /// The locator names the structural site; read written and changed positions from the staged mutant.
    #[must_use]
    pub fn site(self) -> MaturityMutationLocator {
        let Some(site) = self.matrix_row().locator() else {
            std::process::abort()
        };
        site
    }

    /// The matrix's refusing layer.
    #[must_use]
    pub fn declared_layer(self) -> EvidenceBoundary {
        let Some(layer) = self.matrix_row().refusing_layer() else {
            std::process::abort()
        };
        layer
    }

    /// Resolve an offerable row by its matrix key.
    ///
    /// # Errors
    /// The successor control recipe has no position in an announcement to write.
    pub fn from_matrix_name(
        section: MaturitySafetySection,
        name: &str,
    ) -> Result<Option<Self>, MaturityMutantStageRefusal> {
        if section == MaturitySafetySection::SuccessorConstructorFault
            && name == "wrong-control-recipe"
        {
            return Err(MaturityMutantStageRefusal::NoOfferablePosition {
                section,
                name: name.to_owned(),
            });
        }
        Ok(Self::ALL
            .into_iter()
            .find(|row| row.section() == section && row.row() == name))
    }
}

/// The two separately controlled constructor-fault tables.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MaturityMutantTable {
    /// The four predecessor-constructor offers.
    PredecessorConstructor,
    /// The three successor-constructor offers.
    SuccessorConstructor,
}

impl MaturityMutantTable {
    /// The ordered rows offered by this table.
    #[must_use]
    pub fn rows(self) -> &'static [MaturityMutantRow] {
        match self {
            Self::PredecessorConstructor => &MaturityMutantRow::ALL[..4],
            Self::SuccessorConstructor => &MaturityMutantRow::ALL[4..],
        }
    }
}

/// The complete context from which exact mutant bytes are staged.
pub struct MaturityMutantContext<'a> {
    /// The reviewed target used by public derivation and signing.
    pub target: &'a ReviewedElementsTapscriptDefinition,
    /// The honest linked constructor bundle.
    pub bundle: &'a CandidateLinkedMaturityBundle,
    /// The honest finalized announcement, including its successor constructor.
    pub finalized: &'a FinalizedMaturityAnnouncement,
    /// The honest signed transaction serialization.
    pub submitted_bytes: &'a [u8],
    /// The honest spent predecessor.
    pub funded: &'a MaturityFundedPredecessor,
    /// The retained deployment identity.
    pub identity: &'a CandidateDeploymentIdentity,
    /// The retained branch context.
    pub branch: BranchContext,
    /// The separately funded predecessor for the foreign-key row.
    pub foreign_funded: Option<&'a MaturityFundedPredecessor>,
}

/// One exact staged submission and its declared write positions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaturityStagedMutant {
    row: MaturityMutantRow,
    bytes: Vec<u8>,
    written_witness_items: Vec<usize>,
    changed_witness_items: Vec<usize>,
    changed_output: Option<usize>,
    spent_outpoint: Outpoint,
    re_signed: bool,
}

impl MaturityStagedMutant {
    /// The row this stage offers.
    #[must_use]
    pub const fn row(&self) -> MaturityMutantRow {
        self.row
    }
    /// Exact submitted transaction bytes.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    /// Witness positions the stage writes, whether or not their bytes change.
    #[must_use]
    pub fn written_witness_items(&self) -> &[usize] {
        &self.written_witness_items
    }
    /// Witness positions whose bytes differ from the honest submission, computed at staging.
    /// When the witness shape changes, every honest position is reported.
    #[must_use]
    pub fn changed_witness_items(&self) -> &[usize] {
        &self.changed_witness_items
    }
    /// Changed output position, if any.
    #[must_use]
    pub const fn changed_output(&self) -> Option<usize> {
        self.changed_output
    }
    /// The outpoint this submission spends.
    #[must_use]
    pub const fn spent_outpoint(&self) -> &Outpoint {
        &self.spent_outpoint
    }
    /// Whether the operator signed this mutant's own frozen bytes.
    #[must_use]
    pub const fn re_signed(&self) -> bool {
        self.re_signed
    }
}

/// A stage refusal that retains the lower layer's reason whole.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MaturityMutantStageRefusal {
    /// The maturity source or link refused.
    Closure(Box<MaturityClosureRefusal>),
    /// Transaction decoding or assembly refused.
    Transaction(Box<TransactionRefusal>),
    /// A tapscript constructor or control recipe refused.
    Constructor(Box<StateConstructorRefusal>),
    /// The honest witness did not have its declared shape.
    Continuity(Box<MaturityContinuityRefusal>),
    /// Freezing the changed candidate refused.
    OperatorSigning(Box<OperatorSigningRefusal>),
    /// The published signer could not sign.
    Signing(TestSigningDefect),
    /// The predecessor bundle retains no constructor.
    NoRetainedPredecessor,
    /// The foreign-key predecessor row needs its separately funded coin.
    ForeignCoinRequired,
    /// The separately funded coin does not carry the derived program.
    ForeignCoinProgramMismatch,
    /// A matrix row has no spend-time announcement position.
    NoOfferablePosition {
        /// The requested matrix table.
        section: MaturitySafetySection,
        /// The requested matrix name.
        name: String,
    },
    /// No nonce admitted the honest successor metadata under both trees.
    NoCommonAdmissibleNonce {
        /// Nonces admitted by the honest bundle within the checked range.
        honest: Vec<u32>,
        /// Nonces admitted by the other bundle within the checked range.
        foreign: Vec<u32>,
    },
}

fn foreign_output_key(
    merkle_root: &[u8; 32],
) -> Result<([u8; 32], bool), MaturityMutantStageRefusal> {
    match OracleStateCurve.output_key(&FOREIGN_INTERNAL_KEY, merkle_root) {
        StateTweakOutcome::OutputKey { key, parity } => Ok((key, parity)),
        StateTweakOutcome::InternalKeyNotAPoint => Err(MaturityMutantStageRefusal::Constructor(
            Box::new(StateConstructorRefusal::InternalKeyNotAPoint),
        )),
        StateTweakOutcome::TweakAboveGroupOrder => Err(MaturityMutantStageRefusal::Constructor(
            Box::new(StateConstructorRefusal::TweakAboveGroupOrder),
        )),
        StateTweakOutcome::TweakedPointIsIdentity => Err(MaturityMutantStageRefusal::Constructor(
            Box::new(StateConstructorRefusal::TweakedPointIsIdentity),
        )),
    }
}

fn version_one_program(key: &[u8; 32]) -> Vec<u8> {
    let mut program = vec![0x51, 0x20];
    program.extend_from_slice(key);
    program
}

/// Derive the second funded predecessor's program from public curve arithmetic.
///
/// # Errors
/// Refuses an absent retained constructor or a refused curve tweak.
pub fn foreign_key_predecessor_program(
    bundle: &CandidateLinkedMaturityBundle,
) -> Result<Vec<u8>, MaturityMutantStageRefusal> {
    let predecessor = bundle
        .instances()
        .first()
        .ok_or(MaturityMutantStageRefusal::NoRetainedPredecessor)?;
    let (key, _) = foreign_output_key(predecessor.constructor().merkle_root())?;
    Ok(version_one_program(&key))
}

fn common_successor_projection(
    context: &MaturityMutantContext<'_>,
    other: &CandidateLinkedMaturityBundle,
) -> Result<MaturityConstructorProjection, MaturityMutantStageRefusal> {
    let semantic = context
        .finalized
        .construction()
        .successor_constructor()
        .encoded_metadata()
        .semantic;
    let mut honest = Vec::new();
    let mut foreign = Vec::new();
    for value in 0..4096 {
        let encoded = EncodedStateMetadata {
            semantic,
            representation: StateRepresentationNonce::new(value),
        };
        let expected = fixed_projection(
            context.target,
            encoded,
            context.bundle,
            TransactionSide::Output,
        );
        let supplied = fixed_projection(context.target, encoded, other, TransactionSide::Output);
        if expected.is_ok() {
            honest.push(value);
        }
        if supplied.is_ok() {
            foreign.push(value);
        }
        if let (Ok(_), Ok(projection)) = (expected, supplied) {
            return Ok(projection);
        }
    }
    Err(MaturityMutantStageRefusal::NoCommonAdmissibleNonce { honest, foreign })
}

fn rebuild_transaction(
    original: &TargetTransaction,
    inputs: Vec<TargetInput>,
    outputs: Vec<TargetOutput>,
    stack: Vec<Vec<u8>>,
) -> Result<TargetTransaction, MaturityMutantStageRefusal> {
    TargetTransaction::with_output_witnesses(
        original.version(),
        inputs,
        outputs,
        original.lock_time(),
        vec![InputWitness::new(stack)],
        original.output_witnesses().to_vec(),
    )
    .map_err(|refusal| MaturityMutantStageRefusal::Transaction(Box::new(refusal)))
}

fn sign_mutant(
    context: &MaturityMutantContext<'_>,
    candidate: TargetTransaction,
    funded: &MaturityFundedPredecessor,
    control_block: Vec<u8>,
) -> Result<Vec<u8>, MaturityMutantStageRefusal> {
    let binding = context.bundle.deployment().operator();
    let mut genesis = *binding.deployment().genesis_id();
    genesis.reverse();
    let leaf = context.finalized.executing_leaf(context.target);
    let request = OperatorSigningRequest::freeze(
        context.target,
        binding,
        candidate,
        vec![SpentOutputCensusEntry::new(
            AssetField::Explicit(funded.asset),
            ValueField::Explicit(funded.amount),
            funded.program.clone(),
        )],
        LiveDeployment::new(genesis),
        OperatorSigningInput::new(
            0,
            *leaf.tapleaf_hash(),
            leaf.leaf_version(),
            leaf.leaf_script().to_vec(),
            control_block,
        ),
        &OracleLiveCurve::new(context.target.clone()),
    )
    .map_err(|refusal| MaturityMutantStageRefusal::OperatorSigning(Box::new(refusal)))?;
    let signature = OPERATOR_HANDLE
        .material()
        .map_err(MaturityMutantStageRefusal::Signing)?
        .sign(request.message().with_vector_grown(), &[0; 32])
        .map_err(MaturityMutantStageRefusal::Signing)?;
    Ok(signature.to_vec())
}

fn write_foreign_predecessor(
    context: &MaturityMutantContext<'_>,
    funded: &MaturityFundedPredecessor,
    stack: &mut [Vec<u8>],
    inputs: &mut [TargetInput],
) -> Result<(), MaturityMutantStageRefusal> {
    if funded.program != foreign_key_predecessor_program(context.bundle)? {
        return Err(MaturityMutantStageRefusal::ForeignCoinProgramMismatch);
    }
    let predecessor = context
        .bundle
        .instances()
        .first()
        .ok_or(MaturityMutantStageRefusal::NoRetainedPredecessor)?;
    let (_, parity) = foreign_output_key(predecessor.constructor().merkle_root())?;
    let mut recipe = predecessor
        .constructor()
        .control_recipe(StateLeafRole::Announcement)
        .map_err(|refusal| MaturityMutantStageRefusal::Constructor(Box::new(refusal)))?;
    recipe.internal_key = FOREIGN_INTERNAL_KEY;
    recipe.parity = parity;
    stack[5] = vec![prefix(parity)];
    stack[8] = recipe
        .control_bytes()
        .map_err(|refusal| MaturityMutantStageRefusal::Constructor(Box::new(refusal)))?;
    inputs[0] = TargetInput::new(funded.outpoint, inputs[0].sequence());
    Ok(())
}

fn metadata_reveal(
    context: &MaturityMutantContext<'_>,
) -> Result<Vec<Vec<u8>>, MaturityMutantStageRefusal> {
    let predecessor = context
        .bundle
        .instances()
        .first()
        .ok_or(MaturityMutantStageRefusal::NoRetainedPredecessor)?;
    Ok(vec![
        predecessor
            .constructor()
            .leaf_program()
            .encode(context.target),
        predecessor
            .constructor()
            .control_recipe(StateLeafRole::MetadataCommitment)
            .map_err(|refusal| MaturityMutantStageRefusal::Constructor(Box::new(refusal)))?
            .control_bytes()
            .map_err(|refusal| MaturityMutantStageRefusal::Constructor(Box::new(refusal)))?,
    ])
}

/// Stage one matrix fault as an exact transaction offer.
///
/// # Errors
/// Retains source, constructor, transaction and signing refusals; the foreign-key predecessor requires its funded coin.
pub fn stage_maturity_mutant(
    row: MaturityMutantRow,
    context: &MaturityMutantContext<'_>,
) -> Result<MaturityStagedMutant, MaturityMutantStageRefusal> {
    let original = TargetTransaction::decode(context.submitted_bytes)
        .map_err(|refusal| MaturityMutantStageRefusal::Transaction(Box::new(refusal)))?;
    let mut stack = witness(&original, context.bundle.record().schedule())
        .map_err(|refusal| MaturityMutantStageRefusal::Continuity(Box::new(refusal)))?
        .to_vec();
    let honest_stack = stack.clone();
    let mut inputs = original.inputs().to_vec();
    let mut outputs = original.outputs().to_vec();
    let mut funded = context.funded;
    let mut changed_output = None;
    let (written_witness_items, re_signed) = match row {
        MaturityMutantRow::PredecessorWrongInternalKey => {
            funded = context
                .foreign_funded
                .ok_or(MaturityMutantStageRefusal::ForeignCoinRequired)?;
            write_foreign_predecessor(context, funded, &mut stack, &mut inputs)?;
            (vec![5, 6, 8], true)
        }
        MaturityMutantRow::PredecessorWrongControlBlock => {
            stack[5][0] ^= 1;
            (vec![5], false)
        }
        MaturityMutantRow::MetadataLeafSelectedForExecution => {
            stack = metadata_reveal(context)?;
            ((0..9).collect(), false)
        }
        MaturityMutantRow::KeyPathSpendAttempt => {
            stack = vec![stack[6].clone()];
            ((0..9).collect(), false)
        }
        MaturityMutantRow::SuccessorUnderAnotherStaticSubtree => {
            let other = maturity_sources(
                MaturityDeployment::Second,
                MaturityWitnessSelection::Retained(StateWitnessSchedule::VariableMetadata),
            )
            .and_then(|sources| sources.link(&OracleStateCurve))
            .map_err(|refusal| MaturityMutantStageRefusal::Closure(Box::new(refusal)))?;
            let projection = common_successor_projection(context, &other)?;
            let output = &outputs[0];
            outputs[0] = TargetOutput::new(
                output.asset(),
                output.value(),
                output.nonce(),
                projection.output_program().to_vec(),
            );
            stack[0] = vec![prefix(projection.parity())];
            stack[1] = projection.nonce().get().to_be_bytes().to_vec();
            changed_output = Some(0);
            (vec![0, 1, 6], true)
        }
        MaturityMutantRow::SuccessorWrongInternalKey => {
            let successor = context.finalized.construction().successor_constructor();
            let (key, parity) = foreign_output_key(successor.merkle_root())?;
            let output = &outputs[0];
            outputs[0] = TargetOutput::new(
                output.asset(),
                output.value(),
                output.nonce(),
                version_one_program(&key),
            );
            stack[0] = vec![prefix(parity)];
            changed_output = Some(0);
            (vec![0, 6], true)
        }
        MaturityMutantRow::SuccessorWrongParity => {
            stack[0][0] ^= 1;
            (vec![0], false)
        }
    };
    let mut candidate =
        rebuild_transaction(&original, inputs.clone(), outputs.clone(), stack.clone())?;
    if re_signed {
        stack[6] = sign_mutant(context, candidate, funded, stack[8].clone())?;
        candidate = rebuild_transaction(&original, inputs, outputs, stack)?;
    }
    let changed_witness_items = changed_positions(&honest_stack, candidate.witnesses()[0].stack());
    Ok(MaturityStagedMutant {
        row,
        bytes: candidate.encode(),
        written_witness_items,
        changed_witness_items,
        changed_output,
        spent_outpoint: funded.outpoint,
        re_signed,
    })
}

fn changed_positions(honest: &[Vec<u8>], staged: &[Vec<u8>]) -> Vec<usize> {
    if honest.len() != staged.len() {
        return (0..honest.len()).collect();
    }
    honest
        .iter()
        .zip(staged)
        .enumerate()
        .filter_map(|(index, (before, after))| (before != after).then_some(index))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::matrix::MutationLayer;
    use crate::maturity_closure::closure_target;
    use crate::maturity_continuity::tests::{Source, variable_archived};
    use crate::maturity_continuity::{MaturityContinuityRefusal, project_maturity_continuity};
    use crate::maturity_corpus::maturity_variable_run_of_record;
    use crate::maturity_native::MaturityAnnouncementPlanner;
    use crate::maturity_operator::OperatorVerifier;
    use crate::maturity_safety::{
        MaturityCanonicalControl, MaturityIntendedCarrier, MaturityRowBoundary, boundary_admits,
    };
    use tapscript::{AbstractLimits, AbstractStackState, StateInternalKeyPolicy, validate_program};
    use target_elements_conformance::constructor::tagged::sha256;
    use target_elements_conformance::executor::TargetOperationPlanner;
    use transaction::operator_signing::ScriptPathSignatureVerifier;

    struct Fixture {
        source: &'static Source,
        planner: MaturityAnnouncementPlanner,
        target: ReviewedElementsTapscriptDefinition,
        foreign: MaturityFundedPredecessor,
    }

    impl Fixture {
        fn new() -> Self {
            let source = variable_archived();
            let input = source.input();
            let corpus = maturity_variable_run_of_record().expect("validated variable archive");
            let mut planner = MaturityAnnouncementPlanner::new(
                input.identity.clone(),
                input.branch,
                MaturityWitnessSelection::Retained(StateWitnessSchedule::VariableMetadata),
            )
            .expect("public variable planner");
            let mut next = planner.next_step(None).expect("issuance step");
            for (step, response) in &corpus.exchanges()[..2] {
                assert_eq!(next.as_ref(), Some(step));
                next = planner
                    .next_step(Some((step.case(), response)))
                    .expect("accepted funding replay");
            }
            assert_eq!(next.as_ref(), Some(&corpus.exchanges()[2].0));
            let rebuilt = planner
                .submission_bytes()
                .expect("rebuilt signed submission");
            assert!(
                rebuilt == input.submitted_bytes,
                "archived digest {:?}; rebuilt digest {:?}",
                sha256(input.submitted_bytes),
                sha256(rebuilt)
            );
            assert_eq!(planner.bundle(), input.bundle);
            let mut foreign = input.funded.clone();
            foreign.outpoint = Outpoint::new(
                input.funded.outpoint.txid(),
                input.funded.outpoint.index() + 1,
            )
            .expect("distinct foreign coin outpoint");
            foreign.program = foreign_key_predecessor_program(input.bundle)
                .expect("public foreign predecessor program");
            Self {
                source,
                planner,
                target: closure_target().expect("reviewed target"),
                foreign,
            }
        }

        fn context(&self) -> MaturityMutantContext<'_> {
            let input = self.source.input();
            MaturityMutantContext {
                target: &self.target,
                bundle: input.bundle,
                finalized: self
                    .planner
                    .announcement()
                    .expect("rebuilt finalized announcement"),
                submitted_bytes: input.submitted_bytes,
                funded: input.funded,
                identity: input.identity,
                branch: input.branch,
                foreign_funded: Some(&self.foreign),
            }
        }

        fn stage(&self, row: MaturityMutantRow) -> MaturityStagedMutant {
            stage_maturity_mutant(row, &self.context()).expect("stage exact mutant")
        }
    }

    fn expected_positions(row: MaturityMutantRow) -> (Vec<usize>, Vec<usize>, Option<usize>, bool) {
        match row {
            MaturityMutantRow::PredecessorWrongInternalKey => {
                (vec![5, 6, 8], vec![6, 8], None, true)
            }
            MaturityMutantRow::PredecessorWrongControlBlock => (vec![5], vec![5], None, false),
            MaturityMutantRow::MetadataLeafSelectedForExecution
            | MaturityMutantRow::KeyPathSpendAttempt => {
                ((0..9).collect(), (0..9).collect(), None, false)
            }
            MaturityMutantRow::SuccessorUnderAnotherStaticSubtree => {
                (vec![0, 1, 6], vec![0, 6], Some(0), true)
            }
            MaturityMutantRow::SuccessorWrongInternalKey => (vec![0, 6], vec![6], Some(0), true),
            MaturityMutantRow::SuccessorWrongParity => (vec![0], vec![0], None, false),
        }
    }

    fn assert_coinciding_writes(
        row: MaturityMutantRow,
        fixture: &Fixture,
        changed: &TargetTransaction,
        staged_stack: &[Vec<u8>],
        original_stack: &[Vec<u8>],
    ) {
        if row == MaturityMutantRow::PredecessorWrongInternalKey {
            let context = fixture.context();
            let predecessor = context.bundle.instances()[0].constructor();
            let (_, foreign_parity) = foreign_output_key(predecessor.merkle_root())
                .expect("public foreign predecessor tweak");
            assert_eq!(staged_stack[5], vec![prefix(foreign_parity)]);
            assert_eq!(staged_stack[5], original_stack[5]);
        }
        if row == MaturityMutantRow::SuccessorUnderAnotherStaticSubtree {
            let context = fixture.context();
            let selection =
                MaturityWitnessSelection::Retained(StateWitnessSchedule::VariableMetadata);
            let other = maturity_sources(MaturityDeployment::Second, selection)
                .expect("second deployment maturity sources")
                .link(&OracleStateCurve)
                .expect("second deployment linked bundle");
            let projection =
                common_successor_projection(&context, &other).expect("common successor projection");
            let nonce_bytes = projection.nonce().get().to_be_bytes().to_vec();
            assert_eq!(staged_stack[1], nonce_bytes);
            assert_eq!(staged_stack[1], original_stack[1]);
            assert_eq!(staged_stack[0], vec![prefix(projection.parity())]);
            assert_eq!(changed.outputs()[0].program(), projection.output_program());
        }
        if row == MaturityMutantRow::SuccessorWrongInternalKey {
            let context = fixture.context();
            let successor = context.finalized.construction().successor_constructor();
            let (_, parity) = foreign_output_key(successor.merkle_root())
                .expect("public foreign successor tweak");
            assert_eq!(staged_stack[0], vec![prefix(parity)]);
            assert_eq!(staged_stack[0], original_stack[0]);
        }
    }

    #[test]
    fn each_stage_changes_exactly_its_declared_positions() {
        let fixture = Fixture::new();
        let original = TargetTransaction::decode(fixture.source.input().submitted_bytes)
            .expect("honest transaction");
        for row in MaturityMutantRow::ALL {
            let staged = fixture.stage(row);
            let changed = TargetTransaction::decode(staged.bytes()).expect("staged transaction");
            let (expected_written, expected_items, expected_output, expected_signed) =
                expected_positions(row);
            assert_eq!(staged.row(), row);
            assert_eq!(staged.written_witness_items(), expected_written);
            assert_eq!(staged.changed_witness_items(), expected_items);
            assert_eq!(staged.changed_output(), expected_output);
            assert_eq!(staged.re_signed(), expected_signed);
            assert_eq!(changed.encode(), staged.bytes());
            assert_eq!(changed.version(), original.version());
            assert_eq!(changed.lock_time(), original.lock_time());
            assert_eq!(changed.output_witnesses(), original.output_witnesses());
            assert_eq!(changed.inputs().len(), 1);
            assert_eq!(changed.outputs().len(), 1);
            let expected_outpoint = if row == MaturityMutantRow::PredecessorWrongInternalKey {
                fixture.foreign.outpoint
            } else {
                original.inputs()[0].outpoint()
            };
            assert_eq!(changed.inputs()[0].outpoint(), expected_outpoint);
            assert_eq!(staged.spent_outpoint(), &expected_outpoint);
            assert_eq!(
                changed.inputs()[0].sequence(),
                original.inputs()[0].sequence()
            );
            assert_eq!(
                changed.inputs()[0].script_sig(),
                original.inputs()[0].script_sig()
            );
            assert_eq!(changed.outputs()[0].asset(), original.outputs()[0].asset());
            assert_eq!(changed.outputs()[0].value(), original.outputs()[0].value());
            assert_eq!(changed.outputs()[0].nonce(), original.outputs()[0].nonce());
            if expected_output.is_some() {
                assert_ne!(
                    changed.outputs()[0].program(),
                    original.outputs()[0].program()
                );
            } else {
                assert_eq!(
                    changed.outputs()[0].program(),
                    original.outputs()[0].program()
                );
            }
            let original_stack = original.witnesses()[0].stack();
            let staged_stack = changed.witnesses()[0].stack();
            assert_coinciding_writes(row, &fixture, &changed, staged_stack, original_stack);
            let replaced_witness = matches!(
                row,
                MaturityMutantRow::MetadataLeafSelectedForExecution
                    | MaturityMutantRow::KeyPathSpendAttempt
            );
            let actual_items: Vec<usize> = if replaced_witness {
                (0..original_stack.len()).collect()
            } else {
                assert_eq!(staged_stack.len(), original_stack.len());
                for (index, (before, after)) in original_stack.iter().zip(staged_stack).enumerate()
                {
                    if !expected_written.contains(&index) {
                        assert_eq!(before, after, "{row:?} item {index}");
                    }
                }
                original_stack
                    .iter()
                    .zip(staged_stack)
                    .enumerate()
                    .filter_map(|(index, (before, after))| (before != after).then_some(index))
                    .collect()
            };
            assert_eq!(actual_items, expected_items, "{row:?}");
        }
    }

    #[test]
    fn each_stage_reaches_its_rows_host_clause() {
        let fixture = Fixture::new();
        for row in [
            MaturityMutantRow::PredecessorWrongInternalKey,
            MaturityMutantRow::PredecessorWrongControlBlock,
            MaturityMutantRow::SuccessorUnderAnotherStaticSubtree,
            MaturityMutantRow::SuccessorWrongInternalKey,
            MaturityMutantRow::SuccessorWrongParity,
        ] {
            let staged = fixture.stage(row);
            let mut input = fixture.source.input();
            input.submitted_bytes = staged.bytes();
            if row == MaturityMutantRow::PredecessorWrongInternalKey {
                input.funded = &fixture.foreign;
            }
            let refusal = project_maturity_continuity(input).expect_err("host clause");
            let reached = matches!(
                (row, &refusal),
                (
                    MaturityMutantRow::PredecessorWrongInternalKey,
                    MaturityContinuityRefusal::RetainedContext { .. }
                ) | (
                    MaturityMutantRow::PredecessorWrongControlBlock,
                    MaturityContinuityRefusal::PredecessorPrefix(_)
                ) | (
                    MaturityMutantRow::SuccessorUnderAnotherStaticSubtree
                        | MaturityMutantRow::SuccessorWrongInternalKey,
                    MaturityContinuityRefusal::OutputProgram(_)
                ) | (
                    MaturityMutantRow::SuccessorWrongParity,
                    MaturityContinuityRefusal::SuccessorPrefix(_)
                )
            );
            assert!(reached, "{row:?}: {refusal:?}");
            if let MaturityContinuityRefusal::RetainedContext {
                funded_program,
                retained_program,
                ..
            } = refusal
            {
                assert_eq!(funded_program, fixture.foreign.program);
                assert_ne!(Some(funded_program), retained_program);
            }
        }
    }

    #[test]
    fn re_signed_mutants_verify_under_the_published_operator_key() {
        let fixture = Fixture::new();
        for row in [
            MaturityMutantRow::PredecessorWrongInternalKey,
            MaturityMutantRow::SuccessorUnderAnotherStaticSubtree,
            MaturityMutantRow::SuccessorWrongInternalKey,
        ] {
            let staged = fixture.stage(row);
            let candidate = TargetTransaction::decode(staged.bytes()).expect("mutant candidate");
            let stack = candidate.witnesses()[0].stack().to_vec();
            let context = fixture.context();
            let funded = if row == MaturityMutantRow::PredecessorWrongInternalKey {
                &fixture.foreign
            } else {
                context.funded
            };
            let binding = context.bundle.deployment().operator();
            let mut genesis = *binding.deployment().genesis_id();
            genesis.reverse();
            let leaf = context.finalized.executing_leaf(context.target);
            let request = OperatorSigningRequest::freeze(
                context.target,
                binding,
                candidate,
                vec![SpentOutputCensusEntry::new(
                    AssetField::Explicit(funded.asset),
                    ValueField::Explicit(funded.amount),
                    funded.program.clone(),
                )],
                LiveDeployment::new(genesis),
                OperatorSigningInput::new(
                    0,
                    *leaf.tapleaf_hash(),
                    leaf.leaf_version(),
                    leaf.leaf_script().to_vec(),
                    stack[8].clone(),
                ),
                &OracleLiveCurve::new(context.target.clone()),
            )
            .expect("mutant freeze");
            OperatorVerifier
                .verify(
                    binding.key().bytes(),
                    request.message().with_vector_grown(),
                    &stack[6],
                )
                .expect("published key verifies mutant signature");
        }
    }

    #[test]
    fn the_foreign_internal_key_is_refused_by_the_public_constructor_path() {
        assert_eq!(
            StateInternalKeyPolicy::new(FOREIGN_INTERNAL_KEY, &OracleStateCurve),
            Err(StateConstructorRefusal::WrongInternalKey)
        );
    }

    #[test]
    fn the_metadata_leaf_reveal_commits_to_the_funded_program() {
        let fixture = Fixture::new();
        let context = fixture.context();
        let predecessor = context.bundle.instances()[0].constructor();
        let staged = fixture.stage(MaturityMutantRow::MetadataLeafSelectedForExecution);
        let transaction = TargetTransaction::decode(staged.bytes()).expect("metadata reveal");
        let stack = transaction.witnesses()[0].stack();
        assert_eq!(stack.len(), 2);
        assert_eq!(stack[0], predecessor.leaf_program().encode(context.target));
        let (key, _) = match OracleStateCurve.output_key(
            context.bundle.policy().internal_key().key(),
            predecessor.merkle_root(),
        ) {
            StateTweakOutcome::OutputKey { key, parity } => (key, parity),
            other => panic!("honest tweak refused: {other:?}"),
        };
        assert_eq!(context.funded.program, version_one_program(&key));
        let execution = validate_program(
            context.target,
            predecessor.leaf_program(),
            &AbstractStackState::from_main(Vec::new()),
            AbstractLimits::for_target(context.target),
        )
        .expect("metadata abstract execution");
        assert!(execution.always_aborts());
    }

    #[test]
    fn the_key_path_mutant_is_one_item_over_equal_witnessless_bytes() {
        let fixture = Fixture::new();
        let honest = TargetTransaction::decode(fixture.source.input().submitted_bytes)
            .expect("honest transaction");
        let staged = fixture.stage(MaturityMutantRow::KeyPathSpendAttempt);
        let changed = TargetTransaction::decode(staged.bytes()).expect("key-path transaction");
        let [signature] = changed.witnesses()[0].stack() else {
            panic!("key-path witness has one item")
        };
        assert_eq!(signature.len(), 64);
        assert_eq!(signature, &honest.witnesses()[0].stack()[6]);
        assert_eq!(
            changed.encode_without_witness(),
            honest.encode_without_witness()
        );
    }

    #[test]
    fn every_stage_declares_its_rows_matrix_facts() {
        for row in MaturityMutantRow::ALL {
            let matrix = row.matrix_row();
            assert_eq!(matrix.section(), row.section());
            assert_eq!(matrix.name(), row.row());
            assert_eq!(matrix.locator(), Some(row.site()));
            assert_eq!(matrix.refusing_layer(), Some(row.declared_layer()));
            assert_eq!(
                matrix.boundary(),
                MaturityRowBoundary::Layer(row.declared_layer())
            );
            assert_eq!(matrix.carrier(), MaturityIntendedCarrier::LinkedConstructor);
            assert_eq!(
                matrix.control(),
                MaturityCanonicalControl::SponsorlessAnnouncement
            );
            let layer = matrix.mutation().expect("negative row mutation layer");
            assert!(matches!(
                layer,
                MutationLayer::LinkedConstructorProgram | MutationLayer::WitnessProof
            ));
            assert!(boundary_admits(row.declared_layer(), layer));
            assert_eq!(
                MaturityMutantRow::from_matrix_name(row.section(), row.row())
                    .expect("offerable row"),
                Some(row)
            );
        }
    }

    #[test]
    fn the_stages_are_deterministic() {
        let fixture = Fixture::new();
        for row in MaturityMutantRow::ALL {
            let first = fixture.stage(row);
            let second = fixture.stage(row);
            assert_eq!(first.bytes(), second.bytes(), "{row:?}");
        }
    }

    #[test]
    fn no_announcement_byte_carries_the_successor_recipe() {
        let fixture = Fixture::new();
        let context = fixture.context();
        let honest =
            TargetTransaction::decode(context.submitted_bytes).expect("honest transaction");
        assert_eq!(honest.outputs().len(), 1);
        let stack = honest.witnesses()[0].stack();
        assert_eq!(stack.len(), 9);
        let recipe = context
            .finalized
            .construction()
            .successor_constructor()
            .control_recipe(StateLeafRole::Announcement)
            .expect("successor control recipe")
            .control_bytes()
            .expect("successor control bytes");
        assert!(stack.iter().all(|item| item != &recipe));
        assert!(matches!(
            MaturityMutantRow::from_matrix_name(
                MaturitySafetySection::SuccessorConstructorFault,
                "wrong-control-recipe"
            ),
            Err(MaturityMutantStageRefusal::NoOfferablePosition { .. })
        ));
    }
}
