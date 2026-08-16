//! The authoring surface the census groups are written against.
//!
//! # Why an author rather than a list of literals
//!
//! Every case needs the same five things stated correctly: a typed
//! identity nothing else in the census claims, a typed program encoded
//! through the reviewed contract, an exact initial stack, the context the
//! primitive reads, and the outcome the contract requires. Writing those
//! out per case would put the identity arithmetic in three hundred
//! places, and a transposed ordinal is a silent duplicate rather than a
//! visible mistake.
//!
//! # Four verdicts, because the target has four answers
//!
//! The reviewed execution domain requires evaluation to finish with
//! exactly one item, and that item to be true. So a case ends in one of
//! four ways, and each has its own constructor here:
//!
//! ```text
//! accept          one item, true          the spend is valid
//! evaluated_false one item, false         the primitives all succeeded
//! left_multiple   any other final depth   ditto, and the depth says so
//! reject          a primitive aborted     with the classes the contract admits
//! ```
//!
//! The middle two matter: a comparison answering false and an overflow
//! retaining its operands are *successful primitives*, and calling either
//! a failure of the primitive would misdescribe the contract. What
//! distinguishes them natively is the final depth, which is why the
//! expected stack is stated even though a node reports none.
//!
//! # Failures are collected, never guessed
//!
//! A literal the target does not admit, a script number outside the
//! admissible range, and a program past the work limit are all defects in
//! this repository's own source. The author records the first of them and
//! [`CensusAuthor::finish`] returns it, so a broken census fails the
//! command instead of producing a slightly different census that still
//! looks complete.

use std::collections::BTreeMap;

use tapscript::{StackItem, TapscriptInstruction, TapscriptProgram};
use target_elements::{
    EncodingClass, OpcodeId, ReviewedDevelopmentBinding, ReviewedElementsTapscriptDefinition,
};

use crate::error::NativeConformanceError;
use crate::fixture::{
    EnforcementLayer, ExpectedPrimitiveOutcome, FixtureScript, FixtureStatement, LeafVersionStatus,
    NativeCaseGroup, NativeCaseId, PrimitiveExecutionContext, PrimitiveFixture,
};
use crate::protocol::ObservedFailureClass;

/// The truth value the target pushes.
///
/// Stated here once, as the target's canonical form, because every
/// successful arithmetic and comparison case ends with one and a census
/// that spelled it differently in two places would be testing its own
/// inconsistency.
pub const TRUE_ITEM: &[u8] = &[0x01];

/// The false value the target pushes, and its absent-field marker.
pub const FALSE_ITEM: &[u8] = &[];

/// Collects the canonical census, one case at a time.
pub struct CensusAuthor<'a> {
    target: &'a ReviewedElementsTapscriptDefinition,
    binding: &'a ReviewedDevelopmentBinding,
    ordinals: BTreeMap<(NativeCaseGroup, Option<OpcodeId>), u32>,
    fixtures: Vec<PrimitiveFixture>,
    failure: Option<NativeConformanceError>,
}

/// One case, as a group states it.
pub struct Case<'a> {
    /// Which dimension the case exercises.
    pub group: NativeCaseGroup,
    /// The primitive it is about, where it is about one.
    pub opcode: Option<OpcodeId>,
    /// The typed program.
    pub script: &'a [TapscriptInstruction],
    /// The exact initial stack, deepest item first.
    pub stack: &'a [StackItem],
    /// The transaction context, where the case needs one.
    pub context: Option<PrimitiveExecutionContext>,
}

impl<'a> CensusAuthor<'a> {
    /// Starts an empty census against one contract and one binding.
    pub const fn new(
        target: &'a ReviewedElementsTapscriptDefinition,
        binding: &'a ReviewedDevelopmentBinding,
    ) -> Self {
        Self {
            target,
            binding,
            ordinals: BTreeMap::new(),
            fixtures: Vec::new(),
            failure: None,
        }
    }

    /// The census, or the first defect that stopped it being stated.
    pub fn finish(self) -> Result<Vec<PrimitiveFixture>, NativeConformanceError> {
        match self.failure {
            Some(error) => Err(error),
            None => Ok(self.fixtures),
        }
    }

    // -- Stack items -------------------------------------------------

    /// The widest literal the reviewed contract admits.
    ///
    /// Read from the contract rather than restated, so a case about the
    /// bound cannot drift away from the bound it is about.
    #[must_use]
    pub const fn maximum_literal_bytes(&self) -> usize {
        self.target.definition().pushes().maximum_payload_bytes()
    }

    /// One literal of arbitrary bytes.
    pub fn item(&mut self, bytes: Vec<u8>) -> StackItem {
        self.admit(StackItem::new(self.target, bytes))
    }

    /// One script number, in the target's canonical minimal form.
    pub fn number(&mut self, value: i64) -> StackItem {
        self.admit(StackItem::script_number(self.target, value))
    }

    /// One signed fixed-width integer.
    pub fn le64(&self, value: i64) -> StackItem {
        StackItem::signed_le64(self.target, value)
    }

    /// One unsigned 32-bit integer.
    pub fn le32(&self, value: u32) -> StackItem {
        StackItem::unsigned_le32(self.target, value)
    }

    /// One payload of a reviewed encoding class, checked against it.
    pub fn encoded(&mut self, class: EncodingClass, bytes: Vec<u8>) -> StackItem {
        self.admit(StackItem::encoded(self.target, class, bytes))
    }

    /// The bytes of one script number, as they reach the stack.
    pub fn number_bytes(&mut self, value: i64) -> Vec<u8> {
        self.number(value).bytes().to_vec()
    }

    /// The bytes of one signed fixed-width integer.
    pub fn le64_bytes(&self, value: i64) -> Vec<u8> {
        self.le64(value).bytes().to_vec()
    }

    /// The bytes of one unsigned 32-bit integer.
    pub fn le32_bytes(&self, value: u32) -> Vec<u8> {
        self.le32(value).bytes().to_vec()
    }

    // -- Cases -------------------------------------------------------

    /// One case the target accepts, leaving exactly one true item.
    pub fn accept(&mut self, case: Case<'_>, final_stack: Vec<Vec<u8>>) {
        let expected = ExpectedPrimitiveOutcome::accept(Some(final_stack));
        self.state(case, expected, LeafVersionStatus::Reviewed, None);
    }

    /// One case the target accepts, whose one final item is a value the
    /// executor's own materialization fixes rather than the fixture.
    pub fn accept_unstated_stack(&mut self, case: Case<'_>) {
        let expected = ExpectedPrimitiveOutcome::Accept {
            static_final_stack: None,
            static_final_altstack: Some(Vec::new()),
        };
        self.state(case, expected, LeafVersionStatus::Reviewed, None);
    }

    /// One case whose primitives all succeed and whose one final item is
    /// false.
    pub fn evaluated_false(&mut self, case: Case<'_>, final_stack: Vec<Vec<u8>>) {
        let expected = ExpectedPrimitiveOutcome::reject(
            [ObservedFailureClass::EvaluatedFalse],
            Some(final_stack),
        );
        self.state(case, expected, LeafVersionStatus::Reviewed, None);
    }

    /// One case whose primitives all succeed and which leaves other than
    /// one item.
    ///
    /// The reviewed domain requires exactly one, so the spend is invalid
    /// — and the depth that made it invalid is the observation that
    /// separates a retained-operand failure from an abort.
    pub fn left_multiple(&mut self, case: Case<'_>, final_stack: Vec<Vec<u8>>) {
        let expected = ExpectedPrimitiveOutcome::reject(
            [ObservedFailureClass::NonSingletonFinalStack],
            Some(final_stack),
        );
        self.state(case, expected, LeafVersionStatus::Reviewed, None);
    }

    /// One case whose primitives all succeed, which leaves other than
    /// one item, and whose items are the network's rather than the
    /// fixture's.
    pub fn left_multiple_unstated(&mut self, case: Case<'_>) {
        let expected = ExpectedPrimitiveOutcome::Reject {
            classes: std::iter::once(ObservedFailureClass::NonSingletonFinalStack).collect(),
            static_final_stack: None,
            static_final_altstack: None,
        };
        self.state(case, expected, LeafVersionStatus::Reviewed, None);
    }

    /// One case a primitive aborts, in one of these classes.
    pub fn reject(&mut self, case: Case<'_>, classes: &[ObservedFailureClass]) {
        let expected = ExpectedPrimitiveOutcome::reject(classes.iter().copied(), None);
        self.state(case, expected, LeafVersionStatus::Reviewed, None);
    }

    /// One case a *relay* rule refuses, leaving the spend valid.
    ///
    /// Minimal encoding is the rule this exists for. The reviewed
    /// contract says a script-number operand is refused "where minimality
    /// is enforced", and at consensus it is not enforced at all: a
    /// nonminimal operand is read, and the primitive succeeds. A case
    /// stating that refusal at the consensus layer is therefore stating
    /// something the target does not do — while the same case stated at
    /// the relay layer is exactly true, and is the only place the rule
    /// can be observed.
    pub fn reject_at_relay(&mut self, case: Case<'_>, classes: &[ObservedFailureClass]) {
        let expected = ExpectedPrimitiveOutcome::reject(classes.iter().copied(), None);
        self.state_at(
            case,
            expected,
            LeafVersionStatus::Reviewed,
            None,
            EnforcementLayer::RelayPolicy,
        );
    }

    /// One case stated at a leaf version the contract has not reviewed.
    ///
    /// The reviewed semantics do not apply under an unreviewed leaf, so
    /// the outcome is not the script's: the case states what the target
    /// does with a leaf it does not interpret, and the script beneath it
    /// is one the reviewed semantics would have refused.
    pub fn unreviewed_leaf(
        &mut self,
        case: Case<'_>,
        leaf_version: u8,
        expected: ExpectedPrimitiveOutcome,
    ) {
        self.state(
            case,
            expected,
            LeafVersionStatus::Unreviewed,
            Some(leaf_version),
        );
    }

    /// One case whose script bytes no typed program expresses.
    pub fn malformed(
        &mut self,
        group: NativeCaseGroup,
        opcode: Option<OpcodeId>,
        script: Vec<u8>,
        expected: ExpectedPrimitiveOutcome,
        layer: EnforcementLayer,
    ) {
        let case = self.identity(group, opcode);
        let statement = FixtureStatement {
            case,
            script: FixtureScript::DeliberatelyMalformed(script),
            initial_stack: &[],
            context: None,
            expected,
            leaf_version: LeafVersionStatus::Reviewed,
            unreviewed_leaf_version: None,
            enforcement_layer: layer,
        };
        self.add(statement);
    }

    // -- Internals ---------------------------------------------------

    /// States one case at the reviewed consensus layer.
    fn state(
        &mut self,
        case: Case<'_>,
        expected: ExpectedPrimitiveOutcome,
        leaf_version: LeafVersionStatus,
        unreviewed_leaf_version: Option<u8>,
    ) {
        self.state_at(
            case,
            expected,
            leaf_version,
            unreviewed_leaf_version,
            EnforcementLayer::Consensus,
        );
    }

    /// States one case at one enforcement layer.
    fn state_at(
        &mut self,
        case: Case<'_>,
        expected: ExpectedPrimitiveOutcome,
        leaf_version: LeafVersionStatus,
        unreviewed_leaf_version: Option<u8>,
        enforcement_layer: EnforcementLayer,
    ) {
        let Some(program) = self.compile(case.script) else {
            return;
        };
        let identity = self.identity(case.group, case.opcode);
        let statement = FixtureStatement {
            case: identity,
            script: FixtureScript::Typed(&program),
            initial_stack: case.stack,
            context: case.context,
            expected,
            leaf_version,
            unreviewed_leaf_version,
            enforcement_layer,
        };
        self.add(statement);
    }

    /// Adds one stated fixture, or records why it could not be stated.
    fn add(&mut self, statement: FixtureStatement<'_>) {
        match PrimitiveFixture::state(self.target, self.binding, statement) {
            Ok(fixture) => self.fixtures.push(fixture),
            Err(error) => self.record(error),
        }
    }

    /// The next unused identity for one group and primitive.
    fn identity(&mut self, group: NativeCaseGroup, opcode: Option<OpcodeId>) -> NativeCaseId {
        let ordinal = self.ordinals.entry((group, opcode)).or_insert(0);
        let assigned = *ordinal;
        *ordinal += 1;
        NativeCaseId::new(group, opcode, assigned)
    }

    /// One typed program, or nothing once a defect has been recorded.
    fn compile(&mut self, script: &[TapscriptInstruction]) -> Option<TapscriptProgram> {
        let Ok(program) = TapscriptProgram::new(script.to_vec()) else {
            self.record(NativeConformanceError::FixtureNotExpressible);
            return None;
        };
        Some(program)
    }

    /// One admitted literal, or the empty one once a defect is recorded.
    fn admit<E>(&mut self, result: Result<StackItem, E>) -> StackItem {
        let Ok(item) = result else {
            self.record(NativeConformanceError::FixtureNotExpressible);
            return StackItem::empty();
        };
        item
    }

    /// Keeps the first defect, which is the one that explains the rest.
    fn record(&mut self, error: NativeConformanceError) {
        if self.failure.is_none() {
            self.failure = Some(error);
        }
    }
}

/// One reviewed primitive, as a program instruction.
pub const fn op(id: OpcodeId) -> TapscriptInstruction {
    TapscriptInstruction::Opcode(id)
}

/// One literal, as a program instruction.
pub const fn push(item: StackItem) -> TapscriptInstruction {
    TapscriptInstruction::Push(item)
}

/// The true item, as a stack expectation.
pub fn truth() -> Vec<u8> {
    TRUE_ITEM.to_vec()
}

/// The false item, as a stack expectation.
pub fn falsity() -> Vec<u8> {
    FALSE_ITEM.to_vec()
}
