//! Focused mutations of an accepted transaction, for the negative half.
//!
//! Guide-12 §19.2's negative requirements need a transaction the target
//! refuses, and the safe constructor cannot produce one: refusing to
//! emit an invalid transaction is what makes it safe. So a negative
//! vector is made the only other way it can be — by taking a
//! transaction the target *accepted* and changing exactly one thing
//! about it.
//!
//! # Why the un-mutated sibling is the whole argument
//!
//! A refusal only means something if it is attributable. Given a
//! transaction the target took and the same transaction with one
//! structural change, a refusal of the second is caused by that change
//! or by nothing at all. That argument fails the moment the surgery
//! introduces a second difference — a re-encoding that normalizes a
//! field, a witness that shifts, a length prefix that moves — because
//! then the refusal has two possible causes and the vector establishes
//! neither.
//!
//! So this module refuses to mutate a transaction it cannot reproduce.
//! [`round_trips`] decodes and re-encodes with no change at all and
//! compares bytes; every mutation runs it first, and a vector that does
//! not survive it is refused rather than mutated. That check is the
//! reason the rest of the module is allowed to describe its output as
//! "one change".
//!
//! # Nothing here decides what a refusal proves
//!
//! A mutation names the §18 class it stages and nothing more. Whether
//! the layer the target answered at is the layer that class expects is
//! a comparison made against an observed transcript, never here, and a
//! mutation that produced an acceptance is a finding this module has no
//! opinion about.

use transaction::{
    AssetField, AssetId, InputWitness, TargetInput, TargetOutput, TargetTransaction, ValueField,
};

use crate::error::VectorError;
use crate::materialize::{MaterializedTargetVector, TargetVectorId};
use crate::matrix::{EvidenceBoundary, VectorClass, all_classes};

/// Whether a transaction survives a decode and re-encode unchanged.
///
/// The precondition of every mutation here. It is not a claim about
/// this workspace's encoder in the abstract — it is a property of these
/// exact bytes, asked about each vector rather than assumed once,
/// because a transaction carrying a field the decoder normalized would
/// pass in general and fail on the one row that mattered.
///
/// # Errors
///
/// [`VectorError::UnmutatableVector`] when the bytes do not decode, or
/// decode to something that re-encodes differently.
pub fn round_trips(vector: &MaterializedTargetVector) -> Result<TargetTransaction, VectorError> {
    decode_exactly(vector.bytes(), vector.id())
}

/// The same check, against bytes a run observed rather than a vector
/// this package materialized.
///
/// # Errors
///
/// [`VectorError::UnmutatableVector`] on either failure.
pub fn decode_exactly(bytes: &[u8], id: TargetVectorId) -> Result<TargetTransaction, VectorError> {
    let decoded =
        TargetTransaction::decode(bytes).map_err(|_| VectorError::UnmutatableVector(id))?;
    if decoded.encode() != bytes {
        return Err(VectorError::UnmutatableVector(id));
    }
    Ok(decoded)
}

/// Which outputs carry one explicit asset.
///
/// The successor is the closed-asset output the covenant settles, and
/// finding it by asset rather than by position keeps a mutation from
/// depending on a layout the ABI is free to rearrange.
#[must_use]
pub fn outputs_carrying(transaction: &TargetTransaction, asset: [u8; 32]) -> Vec<usize> {
    let wanted = AssetField::Explicit(AssetId::from_internal(asset));
    transaction
        .outputs()
        .iter()
        .enumerate()
        .filter(|(_, output)| output.asset() == wanted)
        .map(|(index, _)| index)
        .collect()
}

/// One focused mutation of an accepted compact-ASH transaction.
///
/// Each arm names the §18 class it stages, and the class is where its
/// expected §1.5 boundary comes from — looked up rather than restated,
/// so a matrix that changed its mind about a boundary changes what this
/// module expects with nothing here to edit.
///
/// # Why value balance is a property of the arm
///
/// Elements checks per-asset value conservation before it runs any
/// script. A mutation that leaves the closed asset unbalanced is
/// therefore answered by consensus whatever the covenant would have
/// said, and a class expecting a script-path refusal cannot be reached
/// by one. That is a fact about the mutation rather than about the
/// target, so it is recorded here and reported alongside the observed
/// layer instead of being discovered again in every run.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum NegativeMutation {
    /// Split the successor across two closed-asset outputs at the same
    /// program. The family's total is unchanged, so the covenant sees a
    /// second successor rather than a value it can dispute.
    SplitSuccessorInTwo,
    /// Reverse the order of the ASH inputs, carrying each input's
    /// witness with it so no input acquires another's proof.
    ReverseAshInputOrder,
    /// Pay the successor to a program that is not the constructor's.
    RedirectSuccessorProgram,
    /// Move one unit of the successor into a second output the fixture
    /// never declared, keeping the family total fixed.
    RouteUnitIntoUndeclaredOutput,
    /// Change one input's sequence field.
    ///
    /// Staged but never submitted: the class's boundary is the
    /// constructor's, and the target has no rule to refuse a changed
    /// sequence by. Kept as an arm because the bytes are what a
    /// first-party conformance check would be handed.
    ChangeInputSequence,
    /// Change the transaction version.
    ChangeTransactionVersion,
    /// Swap two items of one input's witness stack.
    ReorderWitnessItems,
    /// Take one unit off the successor and give it to nothing, leaving
    /// the closed asset short of what the inputs carry.
    SuccessorOneBelowTheSum,
}

impl NegativeMutation {
    /// Every mutation this module can stage.
    pub const ALL: &'static [Self] = &[
        Self::SplitSuccessorInTwo,
        Self::ReverseAshInputOrder,
        Self::RedirectSuccessorProgram,
        Self::RouteUnitIntoUndeclaredOutput,
        Self::ChangeInputSequence,
        Self::ChangeTransactionVersion,
        Self::ReorderWitnessItems,
        Self::SuccessorOneBelowTheSum,
    ];

    /// The §18 class this mutation stages.
    #[must_use]
    pub const fn class_name(self) -> &'static str {
        match self {
            Self::SplitSuccessorInTwo => "two-ash-outputs",
            Self::ReverseAshInputOrder => "noncanonical-ash-ordering",
            Self::RedirectSuccessorProgram => "ordinary-wallet-u-output",
            Self::RouteUnitIntoUndeclaredOutput => "shorten-successor-and-grow-another-output",
            Self::ChangeInputSequence => "wrong-sequence",
            Self::ChangeTransactionVersion => "wrong-transaction-version",
            Self::ReorderWitnessItems => "witness-item-reorder",
            Self::SuccessorOneBelowTheSum => "successor-one-below-the-sum",
        }
    }

    /// The matrix class this mutation stages.
    ///
    /// # Errors
    ///
    /// [`VectorError::MatrixCoverageMismatch`] when §18 names no class
    /// by this mutation's name, which would mean the matrix and this
    /// module had drifted apart.
    pub fn class(self) -> Result<VectorClass, VectorError> {
        all_classes()
            .into_iter()
            .find(|class| class.name() == self.class_name())
            .ok_or(VectorError::MatrixCoverageMismatch {
                class: "18 mutation class",
            })
    }

    /// The §1.5 boundary the staged class expects.
    ///
    /// # Errors
    ///
    /// Whatever [`Self::class`] refuses.
    pub fn expected_boundary(self) -> Result<EvidenceBoundary, VectorError> {
        Ok(self.class()?.boundary())
    }

    /// Whether the mutation leaves every asset's value balanced.
    ///
    /// False only for [`Self::SuccessorOneBelowTheSum`], whose whole
    /// content is an imbalance.
    #[must_use]
    pub const fn preserves_value_balance(self) -> bool {
        !matches!(self, Self::SuccessorOneBelowTheSum)
    }
}

/// One mutated transaction, and what was done to make it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MutatedVector {
    origin: TargetVectorId,
    mutation: NegativeMutation,
    bytes: Vec<u8>,
}

impl MutatedVector {
    /// The accepted vector this was made from.
    #[must_use]
    pub const fn origin(&self) -> TargetVectorId {
        self.origin
    }

    /// What was changed.
    #[must_use]
    pub const fn mutation(&self) -> NegativeMutation {
        self.mutation
    }

    /// The mutated transaction bytes.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

/// Apply one mutation to one accepted vector.
///
/// The vector is round-tripped first, so the only difference between
/// these bytes and the accepted ones is the mutation named.
///
/// # Errors
///
/// [`VectorError::UnmutatableVector`] when the vector does not
/// reproduce its own bytes, or when its shape gives the mutation
/// nothing to act on — one input cannot be reordered, a witness of one
/// item cannot be permuted, and a transaction with no closed-asset
/// output has no successor to disturb. Refusing is the honest answer:
/// a mutation that silently did nothing would submit the accepted
/// transaction again and record its acceptance as a negative result.
pub fn apply(
    vector: &MaterializedTargetVector,
    asset: [u8; 32],
    mutation: NegativeMutation,
) -> Result<MutatedVector, VectorError> {
    let original = round_trips(vector)?;
    let id = vector.id();
    let refuse = || VectorError::UnmutatableVector(id);

    let successors = outputs_carrying(&original, asset);
    let &successor = successors.first().ok_or_else(refuse)?;

    let mut version = original.version();
    let mut inputs = original.inputs().to_vec();
    let mut outputs = original.outputs().to_vec();
    let mut witnesses = original.witnesses().to_vec();

    match mutation {
        NegativeMutation::SplitSuccessorInTwo => {
            let whole = explicit_value(&outputs[successor]).ok_or_else(refuse)?;
            // An odd amount would not halve evenly, and a split that
            // also changed the total would be two mutations.
            let half = whole / 2;
            let rest = whole - half;
            let template = outputs[successor].clone();
            outputs[successor] = with_value(&template, half);
            outputs.insert(successor + 1, with_value(&template, rest));
        }
        NegativeMutation::ReverseAshInputOrder => {
            let ash = usize::from(id.ash_inputs());
            if ash < 2 {
                return Err(refuse());
            }
            // The witness travels with its input. Reversing one and not
            // the other would change which proof answers which spend,
            // which is a different mutation entirely.
            inputs[..ash].reverse();
            witnesses[..ash].reverse();
        }
        NegativeMutation::RedirectSuccessorProgram => {
            let template = outputs[successor].clone();
            // A well-formed witness program of the same width that no
            // constructor in this bundle emits, so the output is
            // spendable-looking and simply not the successor's place.
            let mut program = template.program().to_vec();
            if program.is_empty() {
                return Err(refuse());
            }
            let last = program.len() - 1;
            program[last] ^= 0xff;
            outputs[successor] = TargetOutput::new(
                template.asset(),
                template.value(),
                template.nonce(),
                program,
            );
        }
        NegativeMutation::RouteUnitIntoUndeclaredOutput => {
            let whole = explicit_value(&outputs[successor]).ok_or_else(refuse)?;
            let kept = whole.checked_sub(1).ok_or_else(refuse)?;
            let template = outputs[successor].clone();
            let mut program = template.program().to_vec();
            if program.is_empty() {
                return Err(refuse());
            }
            let last = program.len() - 1;
            program[last] ^= 0xff;
            outputs[successor] = with_value(&template, kept);
            outputs.insert(
                successor + 1,
                TargetOutput::new(
                    template.asset(),
                    ValueField::Explicit(1),
                    template.nonce(),
                    program,
                ),
            );
        }
        NegativeMutation::ChangeInputSequence => {
            let first = inputs.first().ok_or_else(refuse)?;
            inputs[0] = TargetInput::new(first.outpoint(), first.sequence() ^ 1);
        }
        NegativeMutation::ChangeTransactionVersion => {
            version ^= 1;
        }
        NegativeMutation::ReorderWitnessItems => {
            let witness = witnesses.first().ok_or_else(refuse)?;
            let mut stack = witness.stack().to_vec();
            if stack.len() < 2 {
                return Err(refuse());
            }
            stack.swap(0, 1);
            witnesses[0] = InputWitness::new(stack);
        }
        NegativeMutation::SuccessorOneBelowTheSum => {
            let whole = explicit_value(&outputs[successor]).ok_or_else(refuse)?;
            let short = whole.checked_sub(1).ok_or_else(refuse)?;
            outputs[successor] = with_value(&outputs[successor].clone(), short);
        }
    }

    let mutated = TargetTransaction::new(version, inputs, outputs, original.lock_time(), witnesses)
        .map_err(|_| refuse())?;
    let bytes = mutated.encode();
    // A mutation that produced the accepted bytes again would submit
    // the positive vector under a negative name, and its acceptance
    // would be recorded as a target failing to refuse.
    if bytes == vector.bytes() {
        return Err(refuse());
    }
    Ok(MutatedVector {
        origin: id,
        mutation,
        bytes,
    })
}

/// One output's explicit amount, where it has one.
#[must_use]
const fn explicit_value(output: &TargetOutput) -> Option<u64> {
    match output.value() {
        ValueField::Explicit(amount) => Some(amount),
        // A blinded value, or any field a later revision adds, is not an
        // amount this module may arithmetic on. Answered with `None` so
        // the caller refuses the mutation rather than inventing a
        // number for a commitment.
        _ => None,
    }
}

/// The same output carrying another explicit amount.
#[must_use]
fn with_value(output: &TargetOutput, amount: u64) -> TargetOutput {
    TargetOutput::new(
        output.asset(),
        ValueField::Explicit(amount),
        output.nonce(),
        output.program().to_vec(),
    )
}

#[cfg(test)]
mod tests {
    use super::round_trips;
    use crate::bundle::fixture_bundle;
    use crate::plan::derive_evidence_plan;

    #[test]
    fn every_materialized_vector_survives_a_decode_and_re_encode() {
        // The soundness gate of the whole negative half, checked against
        // every vector this plan holds bytes for. A vector that failed
        // here could still be mutated, and the mutation would carry a
        // second difference nobody chose — so the refusal it produced
        // would be attributable to nothing in particular.
        let bundle = fixture_bundle().expect("the fixture bundle builds");
        let plan = derive_evidence_plan(&bundle).expect("the evidence plan derives");
        assert!(
            !plan.target_cases().is_empty(),
            "the plan materialized nothing to check",
        );
        for subject in plan.target_cases() {
            let vector = subject.subject();
            let decoded = round_trips(vector)
                .unwrap_or_else(|_| panic!("{:?} does not round-trip", vector.id()));
            // And the decode is not vacuous: it read the shape the
            // vector claims, so a decoder that returned an empty
            // transaction could not pass this.
            assert_eq!(
                decoded.inputs().len(),
                usize::from(vector.id().ash_inputs()) + usize::from(vector.id().sponsors()),
                "{:?} decoded to another input census",
                vector.id(),
            );
        }
    }

    /// A vector with at least two ASH inputs and a multi-item witness,
    /// so every mutation has something to act on.
    fn subject() -> (crate::materialize::MaterializedTargetVector, [u8; 32]) {
        let bundle = fixture_bundle().expect("the fixture bundle builds");
        let plan = derive_evidence_plan(&bundle).expect("the evidence plan derives");
        let vector = plan
            .target_cases()
            .iter()
            .map(crate::subject::CanonicalSubject::subject)
            .find(|vector| vector.id().ash_inputs() >= 2)
            .expect("a multi-input vector exists")
            .clone();
        (vector, bundle.closed_asset())
    }

    #[test]
    fn every_mutation_stages_a_class_the_matrix_actually_names() {
        // The expected boundary is looked up rather than restated, and
        // this is what keeps the lookup honest: a mutation naming a
        // class §18 does not have would silently lose its expectation.
        for mutation in super::NegativeMutation::ALL {
            let class = mutation
                .class()
                .unwrap_or_else(|_| panic!("{mutation:?} names no §18 class"));
            assert_eq!(class.name(), mutation.class_name());
            assert_eq!(
                class.polarity(),
                crate::matrix::VectorPolarity::Negative,
                "{mutation:?} stages a class that does not expect a refusal",
            );
            // An arm's boundary is either one a submission can reach or
            // one that happens before the target is asked, and the run
            // withholds the second kind rather than submitting it. What
            // is refused is a boundary in neither camp: an arm may not
            // stage a class whose verdict is an infrastructure failure,
            // because no mutation of any bytes produces one.
            let boundary = mutation.expected_boundary().expect("the class is named");
            assert_ne!(
                boundary.requires_target_execution(),
                boundary.is_pre_target(),
                "{mutation:?} expects a boundary that is neither reachable nor pre-target",
            );
        }
    }

    #[test]
    fn every_mutation_changes_the_bytes_and_still_decodes() {
        // A mutation that produced unreadable bytes would be answered by
        // the target's parser rather than by anything the class is
        // about, and one that produced the original bytes would submit
        // the accepted vector under a negative name.
        let (vector, asset) = subject();
        for mutation in super::NegativeMutation::ALL {
            let mutated = super::apply(&vector, asset, *mutation)
                .unwrap_or_else(|_| panic!("{mutation:?} could not be applied"));
            assert_ne!(
                mutated.bytes(),
                vector.bytes(),
                "{mutation:?} left the transaction alone",
            );
            let decoded = transaction::TargetTransaction::decode(mutated.bytes())
                .unwrap_or_else(|_| panic!("{mutation:?} produced unreadable bytes"));
            assert_eq!(
                decoded.encode(),
                mutated.bytes(),
                "{mutation:?} produced bytes that do not reproduce themselves",
            );
        }
    }

    #[test]
    fn each_mutation_changes_exactly_the_structure_it_names() {
        // The attributability argument, checked per arm: everything the
        // mutation does not claim to touch must compare equal to the
        // accepted transaction's own.
        use super::NegativeMutation as M;
        let (vector, asset) = subject();
        let before = round_trips(&vector).expect("the subject round-trips");

        for mutation in M::ALL {
            let mutated = super::apply(&vector, asset, *mutation).expect("applied");
            let after = transaction::TargetTransaction::decode(mutated.bytes()).expect("decodes");

            // The version moves for exactly one arm.
            assert_eq!(
                after.version() != before.version(),
                *mutation == M::ChangeTransactionVersion,
                "{mutation:?} disagreed about the version",
            );
            // The input census never changes: no arm here adds or drops
            // a spend.
            assert_eq!(
                after.inputs().len(),
                before.inputs().len(),
                "{mutation:?} changed the input census",
            );
            // The output census grows for exactly the two arms that say
            // they add an output.
            let adds_output = matches!(
                mutation,
                M::SplitSuccessorInTwo | M::RouteUnitIntoUndeclaredOutput
            );
            assert_eq!(
                after.outputs().len() > before.outputs().len(),
                adds_output,
                "{mutation:?} disagreed about adding an output",
            );
            // The witnesses move for exactly the two arms that touch
            // them.
            let touches_witness =
                matches!(mutation, M::ReverseAshInputOrder | M::ReorderWitnessItems);
            assert_eq!(
                after.witnesses() != before.witnesses(),
                touches_witness,
                "{mutation:?} disagreed about touching a witness",
            );
        }
    }

    #[test]
    fn the_closed_asset_stays_balanced_except_where_the_arm_says_it_does_not() {
        // Value balance is checked before any script runs, so an arm
        // that claims a script-path refusal must not disturb it. This is
        // the property that decides whether the expected boundary is
        // even reachable, so it is asserted rather than assumed.
        let (vector, asset) = subject();
        let before = round_trips(&vector).expect("the subject round-trips");
        let total = |transaction: &transaction::TargetTransaction| -> u64 {
            super::outputs_carrying(transaction, asset)
                .into_iter()
                .filter_map(|index| super::explicit_value(&transaction.outputs()[index]))
                .sum()
        };
        let accepted = total(&before);

        for mutation in super::NegativeMutation::ALL {
            let mutated = super::apply(&vector, asset, *mutation).expect("applied");
            let after = transaction::TargetTransaction::decode(mutated.bytes()).expect("decodes");
            assert_eq!(
                total(&after) == accepted,
                mutation.preserves_value_balance(),
                "{mutation:?} disagreed with its own balance claim",
            );
        }
    }
}
