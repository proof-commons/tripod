//! The constructor prototype's case matrix.
//!
//! # What a row is
//!
//! One compound-prototype fixture: a complete taproot construction, the
//! exact script the spend executes, the exact witness it carries, the
//! successor output the transaction must produce, and the outcome the
//! reviewed target is required to reach. Every expected outcome is
//! authored here from this package's own constructor oracle and from the
//! emitted program's stated properties — never from an executor's answer
//! `(´[PLAN-rule:guide10:independent-oracles]´)`.
//!
//! # Where a mutation lives
//!
//! A fixture states a construction that must be internally coherent: the
//! consumed input's program is the one its own internal key and tree
//! determine, and no fixture may say otherwise. So a mutation of the
//! *predecessor* cannot be written into the construction — it is written
//! into the witness, which is where a caller's freedom actually is. A
//! mutation of the *successor* has nowhere to live in the witness at
//! all, because the successor object is derived rather than witnessed,
//! so it is written into the successor output's program: the transaction
//! creates an instance the program's derivation does not reach.
//!
//! That asymmetry is the composed program's central property rendered as
//! fixtures. A witnessed successor could be mutated directly; a derived
//! one can only be contradicted by the output.
//!
//! # What this matrix cannot state, and why
//!
//! The successor output read at the wrong index. The fixture language
//! carries one output role and requires exactly one output of every role
//! in the census, so a transaction with a decoy output ahead of the
//! successor is not expressible without either a second role or a
//! relaxation of that rule — both of which are reviewed changes to the
//! fixture language rather than matrix work. The nearest expressible
//! row is stated instead: the stated successor output carrying a program
//! that is not the successor's, which is the same comparison failing on
//! the same instruction.
//!
//! # A mock cannot satisfy any of this
//!
//! Every row's outcome is a target verdict. The mock executor recomputes
//! constructions and echoes fixtures' own expectations, which is enough
//! to check that a fixture is answerable and nothing else.
//!
//! These rows are answered by a native run: `check-target-elements-
//! prototypes` drives this matrix through the reviewed executor and the
//! prototype gate reads the result, which is why every claim here is now
//! required rather than unresolved.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use target_elements::ReviewedElementsTapscriptDefinition;

use crate::constructor::canonical::{
    CanonicalConstruction, CanonicalSide, construct_canonically_ordered,
};
use crate::constructor::curve::FIELD_ELEMENT_BYTES;
use crate::constructor::internal_key::UNSPENDABLE_INTERNAL_KEY;
use crate::constructor::metadata::{METADATA_BYTES, PrototypeMetadata};
use crate::constructor::metadata_leaf::metadata_leaf_script;
use crate::constructor::tagged::Digest32;
use crate::constructor::tree::{ConstructedOutput, FixtureTapTree, construct};
use crate::fixture::{ExpectedResourceObservation, ResourceExpectation};
use crate::prototype::{
    CompoundPrototypeFixture, ExpectedPrototypeOutcome, OutputRole, PrototypeCaseId,
    PrototypeClaim, PrototypeConstruction, PrototypeOutput, PrototypeRelation,
};
use crate::prototype_program::{
    COUNTER_AT, FLAGS_AT, MAXIMUM_PREDECESSOR_COUNTER, NONCE_AT, PROTOTYPE_SCHEMA,
    PrototypeProgram, RESERVED_AT,
};

/// How many nonces the matrix grinds before it gives up on an instance.
///
/// Two conditions have to hold at once, so a handful of attempts is the
/// expected cost and a bound this size is a diagnostic rather than a
/// limit anything reaches.
const GRIND_ATTEMPTS: u32 = 4_096;

/// The object kind every row uses.
///
/// One kind, because the kind is carried through rather than pinned and
/// a second one would vary a field no row is about.
const OBJECT_KIND: u32 = 3;

/// The flags every row starts from.
const BASE_FLAGS: u32 = 0x0000_0011;

/// Why the matrix could not be authored.
///
/// Every variant is a defect in this package, not in a target: the
/// matrix is built from the reviewed contract and this package's own
/// oracle, and either both agree or something here is wrong.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ConstructorMatrixDefect {
    /// The prototype program is not admitted against the reviewed
    /// contract, so there is no operation leaf to build a tree from.
    ProgramNotAdmitted,
    /// A metadata object has no canonically ordered instance.
    InstanceNotConstructible,
    /// A metadata object has no successor.
    TransitionNotAvailable,
    /// The metadata leaf script is not expressible.
    LeafNotExpressible,
    /// No predecessor of either output-key parity was found within the
    /// counters the matrix searches.
    ParityNotFound,
    /// No metadata leaf landing on the side the canonical search
    /// rejects was found within the nonces the matrix grinds.
    ///
    /// The branch-order row needs one, because what it states is an
    /// instance the program's own unsorted derivation does not reach.
    CanonicalSideNotFound,
}

impl fmt::Display for ConstructorMatrixDefect {
    /// The defect, spelled for a command's typed diagnostic.
    ///
    /// Every spelling says the matrix could not be authored and names
    /// which of this package's own parts did not determine a row. None
    /// of them describes a target: a command reporting one is reporting
    /// that this repository's contract and its oracle disagree, which is
    /// a first-party defect to fix rather than a finding about anything
    /// executed `(´[ADR010-rule:output:data-classification]´)`.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::ProgramNotAdmitted => {
                "the constructor prototype's program is not admitted against the reviewed \
                 contract, so there is no operation leaf to build a tree from"
            }
            Self::InstanceNotConstructible => {
                "a metadata object has no canonically ordered instance"
            }
            Self::TransitionNotAvailable => "a metadata object has no successor",
            Self::LeafNotExpressible => "the metadata leaf script is not expressible",
            Self::ParityNotFound => {
                "no predecessor of either output-key parity was found within the counters the \
                 matrix searches"
            }
            Self::CanonicalSideNotFound => {
                "no metadata leaf landing on the side the canonical search rejects was found \
                 within the nonces the matrix grinds"
            }
        };
        formatter.write_str(text)
    }
}

/// One predecessor instance and the successor it advances to.
#[derive(Clone, Debug)]
struct Step {
    predecessor: CanonicalConstruction,
    successor: CanonicalConstruction,
}

impl Step {
    /// The witness the composed program consumes, deepest item first.
    ///
    /// The successor output key, the predecessor output key, the one
    /// static root, the predecessor object, and the successor's ground
    /// representation nonce.
    fn witness(&self, static_root: &Digest32) -> Vec<Vec<u8>> {
        vec![
            compressed(self.successor.output()),
            compressed(self.predecessor.output()),
            static_root.to_vec(),
            self.predecessor.metadata().encode().to_vec(),
            self.successor.metadata().nonce.to_le_bytes().to_vec(),
        ]
    }
}

/// One output key, as the compressed point a curve check consumes.
///
/// The parity rides in the leading byte, which is what makes it a
/// witness the program has to check rather than a fact it may assume
/// `(´[PLAN-rule:guide10:parity]´)`.
fn compressed(output: &ConstructedOutput) -> Vec<u8> {
    let mut key = Vec::with_capacity(1 + FIELD_ELEMENT_BYTES);
    key.push(2 + output.parity());
    key.extend_from_slice(output.output_key());
    key
}

/// One metadata object of this recipe.
const fn object(counter: u64, flags: u32) -> PrototypeMetadata {
    PrototypeMetadata {
        schema: PROTOTYPE_SCHEMA,
        object_kind: OBJECT_KIND,
        counter,
        flags,
        nonce: 0,
    }
}

/// A resource expectation that compares nothing.
///
/// What one of these costs is a measurement the native run takes. An
/// invented figure would fail an honest executor over a number no
/// contract states.
const fn recorded_only() -> ExpectedResourceObservation {
    ExpectedResourceObservation {
        script_bytes: ResourceExpectation::RecordedOnly,
        initial_stack_items: ResourceExpectation::RecordedOnly,
        peak_stack_items: ResourceExpectation::RecordedOnly,
        peak_altstack_items: ResourceExpectation::RecordedOnly,
        maximum_element_bytes: ResourceExpectation::RecordedOnly,
        validation_budget_used: ResourceExpectation::RecordedOnly,
        transaction_weight: ResourceExpectation::RecordedOnly,
    }
}

/// Everything one matrix shares: the program, the tree it sits in, and
/// the one static root both constructors are built over.
struct Recipe {
    operation_leaf: FixtureTapTree,
    static_root: Digest32,
    program: Vec<u8>,
}

impl Recipe {
    /// The recipe the reviewed contract determines.
    fn resolve(
        target: &ReviewedElementsTapscriptDefinition,
    ) -> Result<Self, ConstructorMatrixDefect> {
        let emitted = PrototypeProgram::continuity(target)
            .map_err(|_| ConstructorMatrixDefect::ProgramNotAdmitted)?;
        let program = emitted.encode(target);
        let operation_leaf = FixtureTapTree::leaf(program.clone());
        Ok(Self {
            static_root: operation_leaf.node_hash(),
            operation_leaf,
            program,
        })
    }

    /// One canonically ordered instance over this recipe's subtree.
    fn instance(
        &self,
        target: &ReviewedElementsTapscriptDefinition,
        metadata: &PrototypeMetadata,
    ) -> Result<CanonicalConstruction, ConstructorMatrixDefect> {
        construct_canonically_ordered(
            target,
            &UNSPENDABLE_INTERNAL_KEY,
            metadata,
            &self.operation_leaf,
            &self.operation_leaf,
            GRIND_ATTEMPTS,
        )
        .map_err(|_| ConstructorMatrixDefect::InstanceNotConstructible)
    }

    /// One predecessor and the successor it advances to.
    fn step(
        &self,
        target: &ReviewedElementsTapscriptDefinition,
        metadata: &PrototypeMetadata,
    ) -> Result<Step, ConstructorMatrixDefect> {
        let predecessor = self.instance(target, metadata)?;
        let advanced = predecessor
            .metadata()
            .successor()
            .map_err(|_| ConstructorMatrixDefect::TransitionNotAvailable)?;
        let successor = self.instance(target, &advanced)?;
        Ok(Step {
            predecessor,
            successor,
        })
    }

    /// One row, with the predecessor's own construction and a stated
    /// successor program.
    fn row(
        &self,
        name: &str,
        claims: &[PrototypeClaim],
        step: &Step,
        witness: Vec<Vec<u8>>,
        successor_program: Vec<u8>,
        expected: ExpectedPrototypeOutcome,
    ) -> CompoundPrototypeFixture {
        Self::row_over(
            name,
            claims,
            step.predecessor.tree().clone(),
            self.operation_leaf.clone(),
            self.program.clone(),
            UNSPENDABLE_INTERNAL_KEY,
            witness,
            successor_program,
            expected,
        )
    }

    /// One row over a stated tree, executing leaf, and internal key.
    ///
    /// The general form. The predecessor program is not a parameter: it
    /// is what the stated key and tree determine, and a fixture saying
    /// anything else describes a spend of an output its own tree does not
    /// commit to.
    #[expect(
        clippy::too_many_arguments,
        reason = "each argument is one stated field of the fixture, and grouping them would build the same struct this returns"
    )]
    fn row_over(
        name: &str,
        claims: &[PrototypeClaim],
        tree: FixtureTapTree,
        executing_leaf: FixtureTapTree,
        script: Vec<u8>,
        internal_key: [u8; FIELD_ELEMENT_BYTES],
        witness: Vec<Vec<u8>>,
        successor_program: Vec<u8>,
        expected: ExpectedPrototypeOutcome,
    ) -> CompoundPrototypeFixture {
        let built = construct(&internal_key, &tree, &executing_leaf);
        let predecessor_program = built
            .as_ref()
            .map(|output| output.output_program().to_vec())
            .unwrap_or_default();
        let control = built
            .as_ref()
            .ok()
            .map(|output| output.control_block().to_vec());

        CompoundPrototypeFixture {
            case: PrototypeCaseId {
                relation: PrototypeRelation::MetadataConstructorContinuity,
                name: name.to_owned(),
            },
            claims: claims.iter().copied().collect(),
            target_contract_version: 0,
            script,
            initial_stack: witness,
            construction: PrototypeConstruction {
                internal_key,
                tree,
                executing_leaf,
                control,
                predecessor_program,
                outputs: vec![PrototypeOutput {
                    role: OutputRole::Successor,
                    program: successor_program,
                }],
            },
            expected,
            expected_resources: recorded_only(),
        }
    }
}

/// One byte string with one byte flipped at a stated offset.
fn mutated(bytes: &[u8], at: usize) -> Vec<u8> {
    let mut copy = bytes.to_vec();
    if let Some(byte) = copy.get_mut(at) {
        *byte ^= 0x01;
    }
    copy
}

/// The complete §22.5 constructor matrix.
///
/// # Errors
///
/// [`ConstructorMatrixDefect`] when the reviewed contract and this
/// package's oracle do not between them determine every row.
#[expect(
    clippy::too_many_lines,
    reason = "the matrix is one enumeration and splitting it would hide which rows exist"
)]
pub fn constructor_matrix(
    target: &ReviewedElementsTapscriptDefinition,
) -> Result<Vec<CompoundPrototypeFixture>, ConstructorMatrixDefect> {
    use ExpectedPrototypeOutcome::{Accepted, Rejected};
    use PrototypeClaim as K;

    let recipe = Recipe::resolve(target)?;
    let version = target.definition().version().get();
    let root = recipe.static_root;

    // The representative step every mutation row is stated against.
    let step = recipe.step(target, &object(7, BASE_FLAGS))?;
    let witness = step.witness(&root);
    let successor_program = step.successor.output().output_program().to_vec();
    let metadata = step.predecessor.metadata().encode();

    // The two output-key parities, which are a property of the tweaked
    // point rather than anything a fixture chooses. The counters below
    // are searched until one instance of each parity is found, so the
    // pair is a measurement rather than an assumption.
    let mut even: Option<Step> = None;
    let mut odd: Option<Step> = None;
    for counter in 0..64_u64 {
        let candidate = recipe.step(target, &object(counter, BASE_FLAGS))?;
        let slot = if candidate.predecessor.output().parity() == 0 {
            &mut even
        } else {
            &mut odd
        };
        if slot.is_none() {
            *slot = Some(candidate);
        }
        if even.is_some() && odd.is_some() {
            break;
        }
    }
    let (Some(even), Some(odd)) = (even, odd) else {
        return Err(ConstructorMatrixDefect::ParityNotFound);
    };

    // A second recipe subtree, for the rows that need a static root this
    // program was not built over.
    let other_subtree = FixtureTapTree::branch(
        recipe.operation_leaf.clone(),
        FixtureTapTree::leaf(b"an escape leaf this recipe does not admit".to_vec()),
    );
    let bare_subtree = FixtureTapTree::leaf(b"an operation leaf that is not the program".to_vec());

    let mut rows = Vec::new();

    // -- The accepting rows -------------------------------------------

    for (name, step) in [
        ("valid_continuity_even_parity", &even),
        ("valid_continuity_odd_parity", &odd),
    ] {
        rows.push(recipe.row(
            name,
            &[
                K::PredecessorProgramObserved,
                K::MetadataLeafDerivationObserved,
                K::StaticSubtreeContinuityObserved,
                K::ExecutingPathAdmittedObserved,
                K::CanonicalBranchOrderObserved,
                K::SuccessorProgramObserved,
                K::CounterTransitionObserved,
                K::OutputKeyParityObserved,
            ],
            step,
            step.witness(&root),
            step.successor.output().output_program().to_vec(),
            Accepted,
        ));
    }

    // The counter's two ends. The lower one is the object a construction
    // starts at; the upper one is the last counter the program's signed
    // increment admits.
    for (name, counter) in [
        ("valid_continuity_counter_zero", 0_u64),
        (
            "valid_continuity_counter_at_program_maximum",
            MAXIMUM_PREDECESSOR_COUNTER,
        ),
    ] {
        let step = recipe.step(target, &object(counter, BASE_FLAGS))?;
        rows.push(recipe.row(
            name,
            &[K::CounterTransitionObserved, K::SuccessorProgramObserved],
            &step,
            step.witness(&root),
            step.successor.output().output_program().to_vec(),
            Accepted,
        ));
    }

    // -- Predecessor mutations, which live in the witness --------------
    //
    // Each changes the object the program reads while the consumed input
    // still carries the program the unmutated object determines, so the
    // predecessor comparison is what fails.

    for (name, at) in [
        ("predecessor_domain_changed", 0_usize),
        ("predecessor_schema_changed", 16),
        ("predecessor_object_kind_changed", 20),
        ("predecessor_counter_changed", COUNTER_AT),
        ("predecessor_flags_changed", FLAGS_AT),
        ("predecessor_nonce_changed", NONCE_AT),
        ("predecessor_reserved_field_nonzero", RESERVED_AT),
    ] {
        let mut mutated_witness = witness.clone();
        mutated_witness[3] = mutated(&metadata, at);
        rows.push(recipe.row(
            name,
            &[
                K::PredecessorProgramObserved,
                K::MetadataLeafDerivationObserved,
            ],
            &step,
            mutated_witness,
            successor_program.clone(),
            Rejected,
        ));
    }

    // The object's width, either side of the schema's.
    for (name, object_bytes) in [
        ("predecessor_trailing_metadata_bytes", {
            let mut wide = metadata.to_vec();
            wide.push(0);
            wide
        }),
        ("predecessor_truncated_metadata", {
            let mut narrow = metadata.to_vec();
            narrow.truncate(METADATA_BYTES - 1);
            narrow
        }),
        ("predecessor_alternate_field_order", {
            let mut swapped = metadata.to_vec();
            swapped.swap(COUNTER_AT, FLAGS_AT);
            swapped
        }),
    ] {
        let mut mutated_witness = witness.clone();
        mutated_witness[3] = object_bytes;
        rows.push(recipe.row(
            name,
            &[K::MetadataLeafDerivationObserved],
            &step,
            mutated_witness,
            successor_program.clone(),
            Rejected,
        ));
    }

    // An object of another schema, whole: the instance is internally
    // consistent and belongs to a family this recipe is not written for.
    {
        let foreign = PrototypeMetadata {
            schema: PROTOTYPE_SCHEMA + 1,
            ..object(7, BASE_FLAGS)
        };
        let foreign_step = recipe.step(target, &foreign)?;
        rows.push(recipe.row(
            "constructor_from_another_schema",
            &[
                K::PredecessorProgramObserved,
                K::MetadataLeafDerivationObserved,
            ],
            &foreign_step,
            foreign_step.witness(&root),
            foreign_step.successor.output().output_program().to_vec(),
            Rejected,
        ));
    }

    // The counter's domain, either side of what the program admits.
    for (name, counter) in [
        (
            "counter_above_program_maximum",
            MAXIMUM_PREDECESSOR_COUNTER + 1,
        ),
        ("counter_with_the_high_bit_set", u64::MAX - 4),
    ] {
        let domain_step = recipe.step(target, &object(counter, BASE_FLAGS))?;
        rows.push(recipe.row(
            name,
            &[K::CounterTransitionObserved],
            &domain_step,
            domain_step.witness(&root),
            domain_step.successor.output().output_program().to_vec(),
            Rejected,
        ));
    }

    // -- Successor mutations, which live in the output -----------------
    //
    // The successor object is derived, so there is no witness to mutate.
    // Each row states an output the derivation does not reach.

    for (name, object_after) in [
        ("successor_counter_unchanged", *step.predecessor.metadata()),
        ("successor_counter_incremented_by_two", {
            let mut twice = *step.successor.metadata();
            twice.counter = step.predecessor.metadata().counter + 2;
            twice
        }),
        ("successor_flags_changed", {
            let mut changed = *step.successor.metadata();
            changed.flags = BASE_FLAGS ^ 0x0000_0100;
            changed
        }),
        ("successor_object_kind_changed", {
            let mut changed = *step.successor.metadata();
            changed.object_kind = OBJECT_KIND + 1;
            changed
        }),
    ] {
        let created = recipe.instance(target, &object_after)?;
        rows.push(recipe.row(
            name,
            &[K::SuccessorProgramObserved, K::CounterTransitionObserved],
            &step,
            witness.clone(),
            created.output().output_program().to_vec(),
            Rejected,
        ));
    }

    // A successor output whose nonce is not the one the witness carries.
    //
    // # Why this row states its leaf directly
    //
    // It used to state the mutation as an object and hand it to the
    // oracle's own instance search, exactly like the four mutations
    // above. That was not a mutation at all: the search grinds the nonce
    // itself, writing `with_nonce(attempt)` from zero upward, so the
    // offered nonce is discarded before the first leaf is built and the
    // instance it returned was byte-identical to the unmutated
    // successor's. The row stated the correct successor program, the
    // target accepted the spend it was handed, and the row recorded a
    // failure of a program that had done nothing wrong.
    //
    // The nonce is a representation choice rather than a field of the
    // state, which is exactly why the search owns it and why a mutation
    // of it has to be written down directly. So the leaf is built here
    // from the successor's own metadata at the next nonce, and the tree
    // the program never derives is what the fixture states.
    {
        let shifted = step
            .successor
            .metadata()
            .with_nonce(step.successor.metadata().nonce.wrapping_add(1));
        let leaf = FixtureTapTree::leaf(
            metadata_leaf_script(target, &shifted.encode())
                .map_err(|_| ConstructorMatrixDefect::LeafNotExpressible)?,
        );
        let tree = FixtureTapTree::branch(leaf, recipe.operation_leaf.clone());
        let created = construct(&UNSPENDABLE_INTERNAL_KEY, &tree, &recipe.operation_leaf)
            .map_err(|_| ConstructorMatrixDefect::InstanceNotConstructible)?;
        rows.push(recipe.row(
            "successor_nonce_not_the_witnessed_one",
            &[K::SuccessorProgramObserved, K::CounterTransitionObserved],
            &step,
            witness.clone(),
            created.output_program().to_vec(),
            Rejected,
        ));
    }

    // A successor whose reserved field is not zero has no canonical
    // instance at all, so the row states the leaf directly rather than
    // through the oracle's own object.
    {
        let mut nonzero = step.successor.metadata().encode().to_vec();
        nonzero[RESERVED_AT] = 1;
        let leaf = FixtureTapTree::leaf(
            metadata_leaf_script(target, &nonzero)
                .map_err(|_| ConstructorMatrixDefect::LeafNotExpressible)?,
        );
        let tree = FixtureTapTree::branch(leaf, recipe.operation_leaf.clone());
        let created = construct(&UNSPENDABLE_INTERNAL_KEY, &tree, &recipe.operation_leaf)
            .map_err(|_| ConstructorMatrixDefect::InstanceNotConstructible)?;
        rows.push(recipe.row(
            "successor_reserved_field_nonzero",
            &[K::SuccessorProgramObserved],
            &step,
            witness.clone(),
            created.output_program().to_vec(),
            Rejected,
        ));
    }

    // The stated successor output carrying a program that is not the
    // successor's at all. The nearest this fixture language comes to a
    // successor read at the wrong role, and the same comparison fails.
    rows.push(recipe.row(
        "successor_output_carries_another_program",
        &[K::SuccessorProgramObserved],
        &step,
        witness.clone(),
        step.predecessor.output().output_program().to_vec(),
        Rejected,
    ));

    // -- The static root ----------------------------------------------

    {
        // A witnessed root that is not the one the consumed input's tree
        // commits to.
        let mut wrong = witness.clone();
        wrong[2] = mutated(&root, 0);
        rows.push(recipe.row(
            "wrong_static_root",
            &[K::StaticSubtreeContinuityObserved],
            &step,
            wrong,
            successor_program.clone(),
            Rejected,
        ));
    }

    // A successor built over a different static subtree. The program
    // witnesses one root and uses it twice, so the split has nowhere to
    // live in the witness: it lives in the created output, which is
    // where a real attempt would put it.
    for (name, subtree) in [
        ("split_predecessor_and_successor_roots", &other_subtree),
        ("stale_constructor_successor_subtree", &bare_subtree),
    ] {
        let created = construct_canonically_ordered(
            target,
            &UNSPENDABLE_INTERNAL_KEY,
            step.successor.metadata(),
            subtree,
            &recipe.operation_leaf,
            GRIND_ATTEMPTS,
        );
        let program = if let Ok(created) = created {
            created.output().output_program().to_vec()
        } else {
            // A subtree the operation leaf does not sit in determines no
            // control path, so the instance is built for its own root
            // without an executing leaf inside it.
            {
                let leaf = FixtureTapTree::leaf(
                    metadata_leaf_script(target, &step.successor.metadata().encode())
                        .map_err(|_| ConstructorMatrixDefect::LeafNotExpressible)?,
                );
                let tree = FixtureTapTree::branch(leaf, subtree.clone());
                construct(&UNSPENDABLE_INTERNAL_KEY, &tree, subtree)
                    .map_err(|_| ConstructorMatrixDefect::InstanceNotConstructible)?
                    .output_program()
                    .to_vec()
            }
        };
        rows.push(recipe.row(
            name,
            &[K::StaticSubtreeContinuityObserved],
            &step,
            witness.clone(),
            program,
            Rejected,
        ));
    }

    // -- The key, the parity, and the framing --------------------------

    {
        // An instance built over an internal key that is not the
        // published one. The construction is coherent and the program
        // still pushes the published point.
        let leaf = FixtureTapTree::leaf(
            metadata_leaf_script(target, &metadata)
                .map_err(|_| ConstructorMatrixDefect::LeafNotExpressible)?,
        );
        let tree = FixtureTapTree::branch(leaf, recipe.operation_leaf.clone());

        // Not every thirty-two byte string is a point, and not every
        // point tweaks to one, so the alternative is searched for rather
        // than assumed. It is still a key with no known scalar: it is a
        // published constant with one byte moved, and nothing here
        // claims otherwise — the row is a refusal either way.
        let mut found = None;
        for offset in 0..FIELD_ELEMENT_BYTES {
            let mut candidate = UNSPENDABLE_INTERNAL_KEY;
            candidate[offset] ^= 0x01;
            if let Ok(built) = construct(&candidate, &tree, &recipe.operation_leaf) {
                found = Some((candidate, built));
                break;
            }
        }
        let (other_key, built) = found.ok_or(ConstructorMatrixDefect::InstanceNotConstructible)?;

        let mut keyed = witness.clone();
        keyed[1] = compressed(&built);
        rows.push(Recipe::row_over(
            "wrong_internal_key",
            &[K::PredecessorProgramObserved],
            tree,
            recipe.operation_leaf.clone(),
            recipe.program.clone(),
            other_key,
            keyed,
            successor_program.clone(),
            Rejected,
        ));
    }

    for (name, index) in [
        ("wrong_predecessor_output_key_parity", 1_usize),
        ("wrong_successor_output_key_parity", 0),
    ] {
        let mut flipped = witness.clone();
        flipped[index][0] ^= 0x01;
        rows.push(recipe.row(
            name,
            &[K::OutputKeyParityObserved],
            &step,
            flipped,
            successor_program.clone(),
            Rejected,
        ));
    }

    {
        // A key whose coordinate is not the one the program the input
        // carries commits to.
        let mut foreign = witness.clone();
        foreign[1] = compressed(step.successor.output());
        rows.push(recipe.row(
            "predecessor_key_from_another_output",
            &[K::OutputKeyParityObserved, K::PredecessorProgramObserved],
            &step,
            foreign,
            successor_program.clone(),
            Rejected,
        ));
    }

    {
        // The metadata leaf framed under a leaf version the program does
        // not hash with.
        let leaf = FixtureTapTree::leaf_of_version(
            0xc2,
            metadata_leaf_script(target, &metadata)
                .map_err(|_| ConstructorMatrixDefect::LeafNotExpressible)?,
        );
        let tree = FixtureTapTree::branch(leaf, recipe.operation_leaf.clone());
        let built = construct(&UNSPENDABLE_INTERNAL_KEY, &tree, &recipe.operation_leaf)
            .map_err(|_| ConstructorMatrixDefect::InstanceNotConstructible)?;
        let mut framed = witness.clone();
        framed[1] = compressed(&built);
        rows.push(Recipe::row_over(
            "wrong_metadata_leaf_framing",
            &[K::MetadataLeafDerivationObserved],
            tree,
            recipe.operation_leaf.clone(),
            recipe.program.clone(),
            UNSPENDABLE_INTERNAL_KEY,
            framed,
            successor_program.clone(),
            Rejected,
        ));
    }

    {
        // An instance whose metadata leaf lands on the side the program
        // does not hash.
        //
        // # What this row used to state, and why it could not fail
        //
        // It used to state the same two children written the other way
        // round, on the reasoning that the root would then not be the
        // one the program derives. That reasoning was wrong, and this
        // package says so three files away: the target sorts a branch's
        // two child hashes before hashing them, so a tree written with
        // its children swapped is the *same tree* with the same root,
        // the same output key, and the same program. The fixture stated
        // the spend it meant to reject as the spend it meant to accept,
        // and the target accepted it.
        //
        // The threat is real; what it is has to be stated exactly. The
        // program cannot sort — the reviewed domain gives it no
        // comparison to sort with — so it hashes the metadata leaf and
        // the static root in one fixed order, and the search that
        // authors an instance grinds the nonce until the true ordering
        // is that one. An instance ground the other way is therefore a
        // well-formed taproot output that the program's own branch
        // derivation does not reach: it computes the unsorted hash, the
        // target committed to the sorted one, and the derived output key
        // is not the witnessed one
        // (´[PLAN-rule:guide10:tapbranch-order]´).
        //
        // That is what is stated below, by grinding for the side the
        // canonical search rejects.
        let static_root = recipe.operation_leaf.node_hash();
        let mut wrong_side = None;
        for attempt in 0..GRIND_ATTEMPTS {
            let written = step.predecessor.metadata().with_nonce(attempt);
            let leaf = FixtureTapTree::leaf(
                metadata_leaf_script(target, &written.encode())
                    .map_err(|_| ConstructorMatrixDefect::LeafNotExpressible)?,
            );
            if CanonicalSide::MetadataFirst.holds(&leaf.node_hash(), &static_root) {
                continue;
            }
            let tree = FixtureTapTree::branch(leaf, recipe.operation_leaf.clone());
            if let Ok(built) = construct(&UNSPENDABLE_INTERNAL_KEY, &tree, &recipe.operation_leaf) {
                wrong_side = Some((written, tree, built));
                break;
            }
        }
        let (written, tree, built) =
            wrong_side.ok_or(ConstructorMatrixDefect::CanonicalSideNotFound)?;

        // The witness is the step's own, with the two items the shifted
        // nonce moves. A successor is derived at nonce zero from fields
        // the predecessor's nonce does not touch, so the successor half
        // of the witness is unchanged and this row mutates exactly one
        // thing.
        let mut ordered = witness;
        ordered[1] = compressed(&built);
        ordered[3] = written.encode().to_vec();
        rows.push(Recipe::row_over(
            "noncanonical_branch_order",
            &[K::CanonicalBranchOrderObserved],
            tree,
            recipe.operation_leaf.clone(),
            recipe.program.clone(),
            UNSPENDABLE_INTERNAL_KEY,
            ordered,
            successor_program.clone(),
            Rejected,
        ));
    }

    // -- The metadata leaf, spent -------------------------------------

    for (name, stack) in [
        ("metadata_leaf_spend_empty_witness", Vec::new()),
        ("metadata_leaf_spend_one_true_item", vec![vec![1_u8]]),
        (
            "metadata_leaf_spend_arbitrary_witness",
            vec![vec![0xff_u8; 32], vec![1_u8]],
        ),
    ] {
        let script = metadata_leaf_script(target, &metadata)
            .map_err(|_| ConstructorMatrixDefect::LeafNotExpressible)?;
        let leaf = FixtureTapTree::leaf(script.clone());
        let tree = FixtureTapTree::branch(leaf.clone(), recipe.operation_leaf.clone());
        rows.push(Recipe::row_over(
            name,
            &[
                K::MetadataLeafUnspendableObserved,
                K::ExecutingPathAdmittedObserved,
            ],
            tree,
            leaf,
            script,
            UNSPENDABLE_INTERNAL_KEY,
            stack,
            successor_program.clone(),
            Rejected,
        ));
    }

    // Every row is stated against the reviewed revision, set once here
    // rather than repeated at each row where one could drift.
    for row in &mut rows {
        row.target_contract_version = version;
    }

    Ok(rows)
}

/// Which cases bear on each claim.
///
/// The inverse of the fixtures' own claim sets, computed rather than
/// restated: a claim's bearing cases are exactly the rows that say they
/// bear on it, and a mapping written by hand could disagree with them
/// `(´[PLAN-rule:guide10:claim-coverage]´)`.
#[must_use]
pub fn bearing_cases(
    matrix: &[CompoundPrototypeFixture],
) -> BTreeMap<PrototypeClaim, BTreeSet<PrototypeCaseId>> {
    let mut bearing: BTreeMap<PrototypeClaim, BTreeSet<PrototypeCaseId>> = BTreeMap::new();
    for fixture in matrix {
        for claim in &fixture.claims {
            bearing
                .entry(*claim)
                .or_default()
                .insert(fixture.case.clone());
        }
    }
    bearing
}
