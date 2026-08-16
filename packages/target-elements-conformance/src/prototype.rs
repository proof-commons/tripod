//! The compound-prototype fixture language.
//!
//! # A separate kind of fixture, deliberately
//!
//! A primitive fixture asks what one reviewed primitive does. A compound
//! fixture asks whether a multi-step construction holds together across
//! a whole target output: a tree, an internal key, a control path, and a
//! successor output program. The two are not the same question and are
//! not one type (Guide-10 `rule:guide10:compound-fixture`).
//!
//! Keeping them apart is what stops a compound claim from being counted
//! as primitive coverage. A single fixture type with optional tree
//! fields would have made every primitive fixture look like a
//! constructor fixture that happened to omit them.
//!
//! # Target-generic, and it stays that way
//!
//! A compound fixture may name a metadata constructor continuity or a
//! wide-floor relation. It may not name a protocol object, a protocol
//! role, or an architecture relation identity: those are
//! attestation-contract semantics, and a fixture that named one would
//! put protocol meaning into a language whose entire subject is the
//! target.
//! The vocabulary below is closed, so the rule is enforced by what is
//! representable rather than by review.
//!
//! # Stated versus supplied
//!
//! A stated field is an exact requirement the executor must materialize
//! or refuse; an absent one is executor-supplied under a documented rule
//! (Guide-10 `rule:guide10:stated-fields`). Constructor fixtures state
//! the internal key, the tree, the leaf versions, the scripts, the
//! successor program, and the successor output role. Funding outpoints
//! remain executor-supplied: nothing here depends on which coin paid.

use std::collections::BTreeSet;
use std::fmt;

use serde::{Deserialize, Serialize};
use target_elements::{LeafVersion, ReviewedElementsTapscriptDefinition};

use crate::claim::ClaimRequirement;
use crate::constructor::curve::FIELD_ELEMENT_BYTES;
use crate::constructor::tree::{
    ConstructionDefect, FixtureTapTree, TreeDefect, construct, leaf_hash_of_version_byte,
};
use crate::fixture::ExpectedResourceObservation;

/// Which prototype relation a compound fixture bears on.
///
/// Exactly the two relations Guide 10 admits, and no room for a third
/// without a reviewed addition here.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum PrototypeRelation {
    /// One metadata object's constructor derives from its predecessor's
    /// by the stated transition, and every other field is carried
    /// through.
    MetadataConstructorContinuity,
    /// The exact wide-arithmetic floor relation.
    WideFloorRelation,
}

impl PrototypeRelation {
    /// The complete census.
    pub const ALL: &'static [Self] =
        &[Self::MetadataConstructorContinuity, Self::WideFloorRelation];

    /// The relation's wire spelling.
    #[must_use]
    pub const fn wire(self) -> &'static str {
        match self {
            Self::MetadataConstructorContinuity => "metadata_constructor_continuity",
            Self::WideFloorRelation => "wide_floor_relation",
        }
    }
}

/// Which prototype case a fixture answers for.
///
/// A relation and an ordinal name. The name is a label for a reader and
/// is not an identity anything persists: a report binds the complete
/// fixture, not the case name (Guide-10 `rule:guide10:fixture-binding`).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrototypeCaseId {
    /// The relation the case bears on.
    pub relation: PrototypeRelation,
    /// The case's name within that relation.
    pub name: String,
}

impl fmt::Display for PrototypeCaseId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}::{}", self.relation.wire(), self.name)
    }
}

/// What a passing compound case establishes.
///
/// # Constructor claims, stated before any case exists
///
/// The vocabulary is written now and every row is unresolved, because
/// the cases that would bear on these claims are a later wave's work.
/// That is the honest state and it is enumerable: a reader can see
/// exactly which corners of the constructor nothing yet establishes,
/// rather than inferring it from an absence
/// (Guide-10 `rule:guide10:claim-registry`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum PrototypeClaim {
    /// The consumed input's program was observed to be the one the
    /// predecessor constructor derives.
    PredecessorProgramObserved,
    /// The metadata leaf was observed to derive from exactly the stated
    /// metadata bytes.
    MetadataLeafDerivationObserved,
    /// The static subtree root was observed to participate in both
    /// constructors unchanged.
    StaticSubtreeContinuityObserved,
    /// The executing path was observed to belong to the admitted static
    /// operation subtree, and not to the metadata leaf.
    ExecutingPathAdmittedObserved,
    /// The metadata leaf was observed to be unspendable.
    MetadataLeafUnspendableObserved,
    /// Canonical branch ordering was observed to be enforced.
    CanonicalBranchOrderObserved,
    /// The successor output's program was observed to be the one the
    /// successor constructor derives.
    SuccessorProgramObserved,
    /// The counter transition was observed to be enforced, overflow
    /// included.
    CounterTransitionObserved,
    /// The output key's parity was observed to be checked rather than
    /// trusted.
    OutputKeyParityObserved,
}

impl PrototypeClaim {
    /// The complete census.
    pub const ALL: &'static [Self] = &[
        Self::PredecessorProgramObserved,
        Self::MetadataLeafDerivationObserved,
        Self::StaticSubtreeContinuityObserved,
        Self::ExecutingPathAdmittedObserved,
        Self::MetadataLeafUnspendableObserved,
        Self::CanonicalBranchOrderObserved,
        Self::SuccessorProgramObserved,
        Self::CounterTransitionObserved,
        Self::OutputKeyParityObserved,
    ];

    /// The relation the claim belongs to.
    ///
    /// Every constructor claim belongs to the constructor relation. The
    /// mapping is stated rather than assumed so that a claim admitted
    /// for the arithmetic relation later cannot be counted as
    /// constructor coverage.
    #[must_use]
    pub const fn relation(self) -> PrototypeRelation {
        PrototypeRelation::MetadataConstructorContinuity
    }

    /// Whether Guide 10 requires the claim, and why it is not yet met.
    #[must_use]
    pub const fn requirement(self) -> ClaimRequirement {
        ClaimRequirement::Unresolved(
            "the constructor prototype's target program and its cases are a later wave's work; \
             the fixture language and the reference oracle exist, and no executed case yet bears \
             on this claim",
        )
    }
}

/// What the target is required to do with a compound fixture.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ExpectedPrototypeOutcome {
    /// The spend stands.
    Accepted,
    /// The spend is rejected.
    Rejected,
}

/// Which output of the transaction carries the successor.
///
/// A role rather than a bare index, so that a fixture states *what* the
/// output is for and validation can require exactly one of it. An index
/// alone would be satisfied by any transaction long enough.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum OutputRole {
    /// The output carrying the successor constructor instance.
    Successor,
}

impl OutputRole {
    /// The complete census of roles a fixture may require.
    ///
    /// Validation walks this rather than the one role it knows today,
    /// so a role added later is required to be unique by construction
    /// rather than by somebody remembering to add a check.
    pub const ALL: &'static [Self] = &[Self::Successor];
}

/// One output a compound fixture requires the transaction to carry.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrototypeOutput {
    /// What the output is for.
    pub role: OutputRole,
    /// The exact program the output must carry.
    pub program: Vec<u8>,
}

/// The taproot construction a compound fixture states.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrototypeConstruction {
    /// The exact internal key, x-only.
    pub internal_key: [u8; FIELD_ELEMENT_BYTES],
    /// The complete tree.
    pub tree: FixtureTapTree,
    /// The leaf the spend executes.
    pub executing_leaf: FixtureTapTree,
    /// The control block, where the fixture states one independently.
    ///
    /// Stating it is how a fixture checks the executor's own
    /// construction against a value computed elsewhere. Leaving it
    /// absent lets the executor derive it from the tree it built, which
    /// is the ordinary case (Guide-10 `rule:guide10:stated-fields`).
    pub control: Option<Vec<u8>>,
    /// The program the consumed input must carry.
    pub predecessor_program: Vec<u8>,
    /// The outputs the transaction must carry, by role.
    pub outputs: Vec<PrototypeOutput>,
}

/// One compound-prototype execution, and what it must establish.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompoundPrototypeFixture {
    /// Which case this answers for.
    pub case: PrototypeCaseId,
    /// What a pass would establish.
    pub claims: BTreeSet<PrototypeClaim>,
    /// The target contract revision the fixture is stated against.
    pub target_contract_version: u32,
    /// The exact script the executing leaf runs.
    pub script: Vec<u8>,
    /// The exact initial witness stack, deepest item first.
    pub initial_stack: Vec<Vec<u8>>,
    /// The construction the executor must materialize exactly.
    pub construction: PrototypeConstruction,
    /// What the target must do.
    pub expected: ExpectedPrototypeOutcome,
    /// What the execution may cost.
    pub expected_resources: ExpectedResourceObservation,
}

/// Why a compound fixture does not state a coherent case.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum PrototypeFixtureDefect {
    /// The fixture is stated against a contract revision the reviewed
    /// contract is not.
    ContractRevisionMismatch {
        /// What the fixture claims.
        stated: u32,
        /// What the reviewed contract is.
        reviewed: u32,
    },
    /// The executing leaf is not a leaf.
    ExecutingLeafIsNotALeaf,
    /// The tree does not contain the executing leaf exactly once.
    Tree(TreeDefect),
    /// The executing leaf's script is not the script the fixture states.
    ExecutingLeafScriptMismatch,
    /// The executing leaf's version is not the reviewed one.
    ExecutingLeafVersionUnreviewed {
        /// The version byte stated.
        stated: u8,
    },
    /// The stated control block is not the one the stated construction
    /// determines.
    ControlBlockMismatch,
    /// The internal key and tree determine no output key at all.
    NotConstructible(ConstructionDefect),
    /// The consumed input's program is not the one the stated internal
    /// key and tree determine.
    ///
    /// # Why the successor's program has no such rule
    ///
    /// The predecessor's program is derivable from this fixture, because
    /// this fixture carries the tree that determines it. The successor's
    /// is not: it belongs to the *successor's* construction, over the
    /// successor's metadata, which a predecessor fixture does not carry
    /// and must not be made to carry. So the successor program stays a
    /// stated value that the harness's own oracle computes for a case,
    /// and requiring it to equal something derivable from this tree
    /// would be requiring it to be the predecessor's.
    PredecessorProgramMismatch,
    /// An output role appears other than exactly once.
    OutputRoleNotUnique {
        /// The role in question.
        role: OutputRole,
        /// How many outputs claimed it.
        found: usize,
    },
    /// A claim does not belong to the fixture's relation.
    ClaimOutsideRelation {
        /// The offending claim.
        claim: PrototypeClaim,
    },
    /// The fixture claims nothing, so a pass would establish nothing.
    NoClaims,
}

impl CompoundPrototypeFixture {
    /// The first way this fixture fails to state a coherent case.
    ///
    /// # Why validation is exhaustive rather than early-exit friendly
    ///
    /// Each check below is a way a fixture could be satisfied by a
    /// transaction other than the one it means. The tree check is the
    /// sharpest: a fixture whose tree does not contain its executing
    /// leaf exactly once has no determined control path, so an executor
    /// could authenticate a different leaf and the fixture would have no
    /// way to notice (Guide-10 `rule:guide10:fixture-validation`).
    #[must_use]
    pub fn defect(
        &self,
        target: &ReviewedElementsTapscriptDefinition,
    ) -> Option<PrototypeFixtureDefect> {
        let reviewed = target.definition().version().get();
        if self.target_contract_version != reviewed {
            return Some(PrototypeFixtureDefect::ContractRevisionMismatch {
                stated: self.target_contract_version,
                reviewed,
            });
        }

        if self.claims.is_empty() {
            return Some(PrototypeFixtureDefect::NoClaims);
        }
        for claim in &self.claims {
            if claim.relation() != self.case.relation {
                return Some(PrototypeFixtureDefect::ClaimOutsideRelation { claim: *claim });
            }
        }

        let FixtureTapTree::Leaf { version, script } = &self.construction.executing_leaf else {
            return Some(PrototypeFixtureDefect::ExecutingLeafIsNotALeaf);
        };
        if *script != self.script {
            return Some(PrototypeFixtureDefect::ExecutingLeafScriptMismatch);
        }
        if *version != target.definition().leaf_version().get() {
            return Some(PrototypeFixtureDefect::ExecutingLeafVersionUnreviewed {
                stated: *version,
            });
        }

        let executing = leaf_hash_of_version_byte(*version, script);
        if let Err(defect) = self.construction.tree.path_to(&executing) {
            return Some(PrototypeFixtureDefect::Tree(defect));
        }

        // Everything above is a property of the fixture's own text. The
        // check below is the one that asks whether the construction
        // exists at all: an internal key that is not a curve point, or a
        // tweak that is not a scalar, determines no output key, and a
        // fixture stating one would be coherent on its face and refused
        // by every honest executor.
        let built = match construct(
            &self.construction.internal_key,
            &self.construction.tree,
            &self.construction.executing_leaf,
        ) {
            Ok(built) => built,
            Err(defect) => return Some(PrototypeFixtureDefect::NotConstructible(defect)),
        };

        // The consumed input's program is not a free field: it is the
        // program this internal key and this tree determine, and no
        // other tree determines it. A fixture stating some other program
        // describes a spend of an output its own tree does not commit
        // to.
        if self.construction.predecessor_program != built.output_program() {
            return Some(PrototypeFixtureDefect::PredecessorProgramMismatch);
        }

        if let Some(stated) = &self.construction.control {
            // Compared byte for byte, parity bit included.
            //
            // An earlier version of this check masked the parity out, on
            // the reasoning that a fixture states a tree rather than an
            // output key. That was wrong: the parity is determined by
            // the internal key and the tree exactly as the rest of the
            // block is, so leaving it free admitted fixtures that were
            // coherent here and refused by every executor that derives
            // its own control block — which is every honest one.
            if stated != built.control_block() {
                return Some(PrototypeFixtureDefect::ControlBlockMismatch);
            }
        }

        for role in OutputRole::ALL.iter().copied() {
            let found = self
                .construction
                .outputs
                .iter()
                .filter(|output| output.role == role)
                .count();
            if found != 1 {
                return Some(PrototypeFixtureDefect::OutputRoleNotUnique { role, found });
            }
        }

        None
    }

    /// Whether the fixture states a coherent case.
    #[must_use]
    pub fn is_coherent(&self, target: &ReviewedElementsTapscriptDefinition) -> bool {
        self.defect(target).is_none()
    }

    /// The leaf version the fixture's executing leaf carries, where it
    /// carries the reviewed one.
    #[must_use]
    pub fn reviewed_leaf_version(&self) -> Option<LeafVersion> {
        match &self.construction.executing_leaf {
            FixtureTapTree::Leaf { version, .. } => LeafVersion::new(*version).ok(),
            FixtureTapTree::Branch { .. } => None,
        }
    }
}
