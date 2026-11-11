//! Concrete member coverage relative to a supplied input/role census.
//!
//! This module neither authenticates predecessors nor binds this census to a
//! serialized transaction. Those are distinct boundaries. Offline authentication
//! is an explicit caller assertion from the vectors conformance oracle.

use crate::keyed_bundle::{KeyedBundleSelection, KeyedEmissionBundle, KeyedLeafRecipe};
use compiler::keyed_plan::EvidenceObligation;
use realization::relation::KeyedRelationId;
use std::collections::{BTreeMap, BTreeSet};

/// Transaction outpoint identity, without collateral input flags.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct InputOutpoint {
    pub txid: [u8; 32],
    pub output_index: u32,
}

/// Role supplied by the consumer, never inferred from input position.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum KeyedInputRole {
    ConsumedHandle,
    Sponsor,
}

/// One row of the caller's input census, checked for contiguity from zero.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyedInputContext {
    pub input_index: u32,
    pub outpoint: InputOutpoint,
    pub role: KeyedInputRole,
    /// Complete predecessor scriptPubKey for a consumed handle, including version.
    /// Ignored for sponsors; their caller-supplied field may be empty.
    pub predecessor_program: Vec<u8>,
}

/// An offline oracle's assertion, not a production authentication result.
///
/// Public fields deliberately confer no trust. The binder compares the asserted
/// subject with the supplied context; it does not verify a control block or hash.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OfflinePredecessorAssertion {
    pub outpoint: InputOutpoint,
    pub predecessor_program: Vec<u8>,
    pub recipe: KeyedLeafRecipe,
}

/// An offered binding of one consumed handle to the selected assignments.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyedMemberBinding {
    pub input_index: u32,
    pub outpoint: InputOutpoint,
    /// Recipe of this predecessor, not a claim that it equals the successor.
    /// Program equality is checked by the emitted continuation, not the binder.
    pub recipe: KeyedLeafRecipe,
    pub offline_assertion: OfflinePredecessorAssertion,
    /// Exactly the bundle's selected runtime relation census, once each.
    pub relations: Vec<KeyedRelationId>,
}

/// Retained identities in relation-to-member coverage.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct BoundMember {
    pub input_index: u32,
    pub outpoint: InputOutpoint,
}

/// Normalized retained assertion and relation set for a consumed input.
/// The containing map supplies its input index; the assertion owns its subject.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BoundKeyedMember {
    offline_assertion: OfflinePredecessorAssertion,
    relations: BTreeSet<KeyedRelationId>,
}
impl BoundKeyedMember {
    #[must_use]
    pub const fn offline_assertion(&self) -> &OfflinePredecessorAssertion {
        &self.offline_assertion
    }
    #[must_use]
    pub const fn relations(&self) -> &BTreeSet<KeyedRelationId> {
        &self.relations
    }
}

/// Immutable receipt of coverage of supplied roles, not execution evidence.
/// Predecessor standing remains caller-asserted and unverified.
///
/// The receipt cannot be assembled or mutated by a consumer.
/// ```compile_fail
/// use tapscript::keyed_members::KeyedMemberReceipt;
/// fn forge(receipt: KeyedMemberReceipt) -> KeyedMemberReceipt {
///     KeyedMemberReceipt { ..receipt }
/// }
/// ```
/// ```compile_fail
/// use tapscript::keyed_members::KeyedMemberReceipt;
/// fn clear(receipt: &mut KeyedMemberReceipt) {
///     receipt.members().clear();
/// }
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyedMemberReceipt {
    selection: KeyedBundleSelection,
    inputs: BTreeMap<u32, KeyedInputContext>,
    members: BTreeMap<u32, BoundKeyedMember>,
    successor_recipe: KeyedLeafRecipe,
    coverage: BTreeMap<KeyedRelationId, BTreeSet<BoundMember>>,
    evidence: BTreeSet<EvidenceObligation>,
}
impl KeyedMemberReceipt {
    #[must_use]
    pub const fn selection(&self) -> &KeyedBundleSelection {
        &self.selection
    }
    #[must_use]
    pub const fn inputs(&self) -> &BTreeMap<u32, KeyedInputContext> {
        &self.inputs
    }
    #[must_use]
    pub const fn members(&self) -> &BTreeMap<u32, BoundKeyedMember> {
        &self.members
    }
    #[must_use]
    pub const fn coverage(&self) -> &BTreeMap<KeyedRelationId, BTreeSet<BoundMember>> {
        &self.coverage
    }
    #[must_use]
    pub const fn successor_recipe(&self) -> &KeyedLeafRecipe {
        &self.successor_recipe
    }
    /// Inclusion, retrieval and finality premises remain unresolved.
    #[must_use]
    pub const fn evidence(&self) -> &BTreeSet<EvidenceObligation> {
        &self.evidence
    }
}

/// Binding/coverage refusals, never covenant execution results.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum KeyedMemberRefusal {
    DuplicateInput(u32),
    DuplicateOutpoint(InputOutpoint),
    MissingInputIndex(u32),
    NoRuntimeAssignments,
    NoConsumedMembers,
    MissingMember(u32),
    DuplicateMember(u32),
    UnknownInput(u32),
    NotConsumedHandle(u32),
    WrongMember(u32),
    OfflineSubjectMismatch(u32),
    MissingRelation {
        input_index: u32,
        relation: KeyedRelationId,
    },
    DuplicateRelation {
        input_index: u32,
        relation: KeyedRelationId,
    },
    UnexpectedRelation {
        input_index: u32,
        relation: KeyedRelationId,
    },
}

impl KeyedMemberRefusal {
    /// Stable payload-free tracing classification.
    #[must_use]
    pub const fn refusal_kind(&self) -> &'static str {
        match self {
            Self::DuplicateInput(_) => "duplicate_input",
            Self::DuplicateOutpoint(_) => "duplicate_outpoint",
            Self::MissingInputIndex(_) => "missing_input_index",
            Self::NoRuntimeAssignments => "no_runtime_assignments",
            Self::NoConsumedMembers => "no_consumed_members",
            Self::MissingMember(_) => "missing_member",
            Self::DuplicateMember(_) => "duplicate_member",
            Self::UnknownInput(_) => "unknown_input",
            Self::NotConsumedHandle(_) => "not_consumed_handle",
            Self::WrongMember(_) => "wrong_member",
            Self::OfflineSubjectMismatch(_) => "offline_subject_mismatch",
            Self::MissingRelation { .. } => "missing_relation",
            Self::DuplicateRelation { .. } => "duplicate_relation",
            Self::UnexpectedRelation { .. } => "unexpected_relation",
        }
    }
}

/// Bind both selected runtime clauses to every supplied consumed handle.
///
/// Input indices must be contiguous from zero; completeness relative to serialized
/// inputs is not checked here. Sponsors may occur anywhere. A runtime bundle must
/// bind at least one consumed handle; this is a binding condition, not a leaf law. No maximum member count, asset, amount,
/// flag, output-count or predecessor-index predicate is imposed here.
///
/// # Errors
/// Returns a typed census, association or coverage refusal. Offline assertions
/// are checked only for agreement with their subjects; they are not authenticated.
pub fn bind_keyed_members(
    bundle: &KeyedEmissionBundle,
    inputs: &[KeyedInputContext],
    members: &[KeyedMemberBinding],
) -> Result<KeyedMemberReceipt, KeyedMemberRefusal> {
    let _span = tracing::debug_span!(target: "tapscript::keyed_members", "bind_keyed_members",
        input_count = u64::try_from(inputs.len()).unwrap_or(u64::MAX), member_count = u64::try_from(members.len()).unwrap_or(u64::MAX))
    .entered();
    let result = bind(bundle, inputs, members);
    match &result {
        Ok(_) => {
            tracing::debug!(target: "tapscript::keyed_members", outcome = "completed", "keyed members bound");
        }
        Err(error) => {
            tracing::debug!(target: "tapscript::keyed_members", outcome = "refused", refusal_kind = error.refusal_kind(), "keyed members bound");
        }
    }
    result
}

fn bind(
    bundle: &KeyedEmissionBundle,
    inputs: &[KeyedInputContext],
    members: &[KeyedMemberBinding],
) -> Result<KeyedMemberReceipt, KeyedMemberRefusal> {
    use KeyedMemberRefusal as E;
    let census = input_census(inputs)?;
    let expected = bundle
        .project()
        .fragments
        .keys()
        .copied()
        .collect::<BTreeSet<_>>();
    if expected.is_empty()
        && (!members.is_empty()
            || inputs
                .iter()
                .any(|i| i.role == KeyedInputRole::ConsumedHandle))
    {
        return Err(E::NoRuntimeAssignments);
    }
    if !expected.is_empty()
        && !census
            .values()
            .any(|input| input.role == KeyedInputRole::ConsumedHandle)
    {
        return Err(E::NoConsumedMembers);
    }
    let mut bound = BTreeMap::new();
    let mut coverage = expected
        .iter()
        .map(|id| (*id, BTreeSet::new()))
        .collect::<BTreeMap<_, _>>();
    for member in members {
        if bound.contains_key(&member.input_index) {
            return Err(E::DuplicateMember(member.input_index));
        }
        let input = census
            .get(&member.input_index)
            .ok_or(E::UnknownInput(member.input_index))?;
        if input.role != KeyedInputRole::ConsumedHandle {
            return Err(E::NotConsumedHandle(member.input_index));
        }
        if input.outpoint != member.outpoint {
            return Err(E::WrongMember(member.input_index));
        }
        let offline = &member.offline_assertion;
        if offline.outpoint != member.outpoint
            || offline.predecessor_program != input.predecessor_program
            || offline.recipe != member.recipe
        {
            return Err(E::OfflineSubjectMismatch(member.input_index));
        }
        let mut seen = BTreeSet::new();
        for relation in &member.relations {
            if !seen.insert(*relation) {
                return Err(E::DuplicateRelation {
                    input_index: member.input_index,
                    relation: *relation,
                });
            }
            let row = coverage.get_mut(relation).ok_or(E::UnexpectedRelation {
                input_index: member.input_index,
                relation: *relation,
            })?;
            row.insert(BoundMember {
                input_index: member.input_index,
                outpoint: member.outpoint,
            });
        }
        if let Some(relation) = expected.difference(&seen).next() {
            return Err(E::MissingRelation {
                input_index: member.input_index,
                relation: *relation,
            });
        }
        bound.insert(
            member.input_index,
            BoundKeyedMember {
                offline_assertion: member.offline_assertion.clone(),
                relations: seen,
            },
        );
    }
    for input in census.values() {
        if input.role == KeyedInputRole::ConsumedHandle && !bound.contains_key(&input.input_index) {
            return Err(E::MissingMember(input.input_index));
        }
    }
    Ok(KeyedMemberReceipt {
        selection: bundle.project().selection.clone(),
        inputs: census,
        members: bound,
        successor_recipe: *bundle.project().successor.recipe(),
        coverage,
        evidence: bundle.project().evidence.clone(),
    })
}

fn input_census(
    inputs: &[KeyedInputContext],
) -> Result<BTreeMap<u32, KeyedInputContext>, KeyedMemberRefusal> {
    use KeyedMemberRefusal as E;
    let mut census = BTreeMap::new();
    let mut outpoints = BTreeSet::new();
    for input in inputs {
        if census.insert(input.input_index, input.clone()).is_some() {
            return Err(E::DuplicateInput(input.input_index));
        }
        if !outpoints.insert(input.outpoint) {
            return Err(E::DuplicateOutpoint(input.outpoint));
        }
    }
    for (expected, index) in (0u32..).zip(census.keys()) {
        if expected != *index {
            return Err(E::MissingInputIndex(expected));
        }
    }
    Ok(census)
}

#[cfg(test)]
pub(crate) mod tests;
