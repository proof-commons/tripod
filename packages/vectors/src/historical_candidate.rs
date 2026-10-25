//! Candidate-bound archive standing and retained operator successor material.

use std::collections::{BTreeMap, BTreeSet};

use target_elements_conformance::constructor::tagged;
use transaction::live_taproot::LiveCurveCapability;

use crate::error::VectorError;

/// What the archive permits replay to establish.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HistoricalReplay {
    /// Exact subjects and settlement replay against retained producing programs.
    ProducingCandidate,
    /// Transcript checks stand; current-candidate planner replay is unavailable.
    NotReplayableForCurrentCandidate,
}

/// An executed subject's commitments, separate from source provenance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HistoricalCandidate {
    /// Optional source commit and tree, as recorded, never inferred.
    pub source_revision: Option<[String; 2]>,
    /// Deployment identities and target contract, in report order.
    pub deployment: [String; 3],
    /// SHA-256 commitments to the retained programs used by the subject.
    pub programs: BTreeSet<[u8; 32]>,
    /// The scope of the replay actually performed.
    pub replay: HistoricalReplay,
}

/// Historical acceptance cannot be consumed as current-candidate acceptance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CandidateStanding {
    /// Observations remain bound to this producing candidate.
    HistoricalCandidate(HistoricalCandidate),
    /// Current observations require replay against the selected compact pins.
    CurrentCandidate(CurrentCandidate),
}

/// The selected compact program, independent of source provenance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrentCandidate {
    /// Canonical merkle root, output key and constructor program.
    pub pins: [[u8; 32]; 3],
    /// Deployment identities and target contract in report order.
    pub deployment: [String; 3],
    /// Programs reconstructed with the observed issuance and reserve assets.
    pub programs: BTreeSet<[u8; 32]>,
    /// Source provenance does not establish program identity.
    pub source_revision: Option<[String; 2]>,
}

impl CurrentCandidate {
    /// Bind replay to the selected compact pins, independently of source identity.
    #[must_use]
    pub const fn selected(
        deployment: [String; 3],
        programs: BTreeSet<[u8; 32]>,
        source_revision: Option<[String; 2]>,
    ) -> Self {
        Self {
            pins: [
                target_elements_conformance::reference::FIXTURE_MERKLE_ROOT,
                target_elements_conformance::reference::FIXTURE_REFERENCE_OUTPUT_KEY,
                crate::bundle::PINNED_PROGRAM,
            ],
            deployment,
            programs,
            source_revision,
        }
    }
}

#[cfg(test)]
mod standing_tests {
    use super::*;

    #[test]
    fn current_standing_names_selected_pins_without_historical_access() {
        let current = CandidateStanding::CurrentCandidate(CurrentCandidate::selected(
            [String::new(), String::new(), String::new()],
            BTreeSet::new(),
            None,
        ));
        assert!(current.historical_candidate().is_none());
        let selected = current.current_candidate().expect("current");
        assert_eq!(
            selected.pins[0],
            target_elements_conformance::reference::FIXTURE_MERKLE_ROOT
        );
        assert_eq!(
            selected.pins[1],
            target_elements_conformance::reference::FIXTURE_REFERENCE_OUTPUT_KEY
        );
        assert_eq!(selected.pins[2], crate::bundle::PINNED_PROGRAM);
        let historical = CandidateStanding::HistoricalCandidate(HistoricalCandidate {
            source_revision: None,
            deployment: selected.deployment.clone(),
            programs: BTreeSet::new(),
            replay: HistoricalReplay::NotReplayableForCurrentCandidate,
        });
        assert!(historical.current_candidate().is_none());
        assert_eq!(
            historical
                .historical_candidate()
                .expect("historical")
                .replay,
            HistoricalReplay::NotReplayableForCurrentCandidate
        );
    }
}

// Independent pins for revision-seven observations, in PrivateShape::ALL order.
#[cfg(test)]
pub(crate) const SHAPE_FIXTURES: [&str; 10] = [
    "52f9d832e9ff93070ea9849cd8d5f817746fdb8bc544e6d94207810e594014ca",
    "67ec9e516460e453fcc0bf6cfacfdc58141afefa0b4e4c2d3ff3c2d7a2495b98",
    "6d86b884565d6d4bc1f81fb6915dc91987d97a62e5a3ca896403ef6d4963e30c",
    "7ec97a6e6cf6bf7e6c4308c007e55cf6ac799113abb3b93e3fb3d559c510ec14",
    "bba4ea6e919b8619d75d035b5f6ee9e8aac3a87c00336c8a86c5973a84936bad",
    "d4a4a2e4371a81d4d74ad2fbceef64303091496c374e11e2208c6d3bde1ea0d9",
    "013df551e2862b4018f2e9b2c2cc83174571234f1ad200605de66232514ea001",
    "7c3bda6049cb80635b0580148002c185eeee8ea48e67bfd1fd698e28a65b17c0",
    "2c8318413c8a726c6c65587d4c48d60af1ba82f1622152d216266c5858658671",
    "e8a824533192cd400e97343d3e5ca69d71fe1138d740723b24b87cc7bf2fd796",
];

// Revision-seven predecessor, primary successor and balancing successor.
#[cfg(test)]
pub(crate) const PRIVATE_RESTART_FIXTURES: [&str; 3] = [
    "7fb3e6666f7574af9d9a295b6d9412f38817faf02c97b020752dc57709f0ae8b",
    "414b7c3c514e5846398e5199e43789c07eda8865ef488e062b424d456997eeb1",
    "f56b971cffea21a748b509a9b8aa7ccb0764ba6ce208438405a4025d35342c55",
];

impl CandidateStanding {
    /// The producing candidate, including any explicit replay limitation.
    #[must_use]
    pub const fn historical_candidate(&self) -> Option<&HistoricalCandidate> {
        match self {
            Self::HistoricalCandidate(candidate) => Some(candidate),
            Self::CurrentCandidate(_) => None,
        }
    }

    /// Current standing never supplies a historical candidate.
    #[must_use]
    pub const fn current_candidate(&self) -> Option<&CurrentCandidate> {
        match self {
            Self::CurrentCandidate(candidate) => Some(candidate),
            Self::HistoricalCandidate(_) => None,
        }
    }
}

const OPERATOR_SUCCESSOR: &str =
    include_str!("../fixtures/historical-candidate-0.7.15-dev-operator-successor.leaves");
/// SHA-256 of the retained explicit successor's complete leaf set.
pub const OPERATOR_SUCCESSOR_DIGEST: &str =
    "6cbba0a002847113bf44eac3c3d18ee11b0e5ece865539e9d7dd5bba838ee794";
/// Merkle root of the retained leaves before deployment-asset relocation.
const OPERATOR_SUCCESSOR_MERKLE_ROOT: &str =
    "243029651a56fc5f9a78f8e61ba371dcad0daccd03ef1d923712d78e69903fcd";

const fn unavailable<T>() -> Result<T, VectorError> {
    Err(VectorError::LiveSubstrateUnavailable)
}

/// Decode retained public bytes without accepting an odd or non-hex string.
pub(crate) fn decode(text: &str) -> Option<Vec<u8>> {
    if !text.len().is_multiple_of(2) {
        return None;
    }
    text.as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| {
            let digits = std::str::from_utf8(pair).ok()?;
            u8::from_str_radix(digits, 16).ok()
        })
        .collect()
}

type Leaf = tapscript::LiveTransferLeafRole;

fn root(
    node: &BTreeSet<Leaf>,
    recipes: &BTreeMap<Leaf, linker::ControlPathRecipe<Leaf>>,
    leaves: &BTreeMap<Leaf, [u8; 32]>,
) -> Option<[u8; 32]> {
    let member = *node.first()?;
    if node.len() == 1 {
        return leaves.get(&member).copied();
    }
    let mut below = BTreeSet::from([member]);
    for sibling in recipes.get(&member)?.siblings() {
        let above = below.union(sibling).copied().collect::<BTreeSet<_>>();
        if &above == node {
            return Some(transaction::taproot::branch_hash(
                root(&below, recipes, leaves)?,
                root(sibling, recipes, leaves)?,
            ));
        }
        below = above;
    }
    None
}

// Only complete direct pushes of the retained deployment asset are relocated.
// The digest above binds the instruction stream before any relocation.
fn relocate_asset(script: &mut [u8], asset: &[u8; 32]) -> Option<()> {
    let mut cursor = 0;
    while cursor < script.len() {
        let opcode = script[cursor];
        cursor += 1;
        if (1..=75).contains(&opcode) {
            let end = cursor.checked_add(usize::from(opcode))?;
            let data = script.get_mut(cursor..end)?;
            if data == crate::live_plan::PROTOCOL_ASSET {
                data.copy_from_slice(asset);
            }
            cursor = end;
        } else if (76..=78).contains(&opcode) {
            return None;
        }
    }
    Some(())
}

/// Reconstruct the historical successor from its retained leaves, never from a capture.
pub(crate) fn operator_successor(asset: &[u8; 32]) -> Result<Vec<u8>, VectorError> {
    if decode(OPERATOR_SUCCESSOR_DIGEST).as_deref()
        != Some(tagged::sha256(OPERATOR_SUCCESSOR.as_bytes()).as_slice())
    {
        return unavailable();
    }
    let target = crate::live_plan::reviewed_target()?;
    let bundle = crate::live_plan::demonstration_live_bundle()?;
    let owner = linker::OwnerParameter::new(crate::live_plan::owner_key(
        &target_elements_conformance::test_material::PublicTestSignerHandle::First
            .x_only_public_key()
            .map_err(|_| VectorError::LiveSubstrateUnavailable)?,
    )?);
    let constructor = bundle
        .constructors()
        .get(&(
            owner,
            linker::live_backend::LiveTransferRepresentationPlan::Explicit,
        ))
        .ok_or(VectorError::LiveSubstrateUnavailable)?;
    let recipes = constructor.taptree().recipes();
    let retained = OPERATOR_SUCCESSOR.lines().collect::<Vec<_>>();
    if retained.len() != recipes.len() {
        return unavailable();
    }
    let mut leaves = BTreeMap::new();
    let mut baseline = BTreeMap::new();
    for (leaf, line) in recipes.keys().zip(retained) {
        let (name, bytes) = line
            .rsplit_once(' ')
            .ok_or(VectorError::LiveSubstrateUnavailable)?;
        if name != format!("{leaf:?}") {
            return unavailable();
        }
        let mut script = decode(bytes).ok_or(VectorError::LiveSubstrateUnavailable)?;
        baseline.insert(
            *leaf,
            transaction::taproot::leaf_hash(constructor.leaf_version(), &script),
        );
        relocate_asset(&mut script, asset).ok_or(VectorError::LiveSubstrateUnavailable)?;
        leaves.insert(
            *leaf,
            transaction::taproot::leaf_hash(constructor.leaf_version(), &script),
        );
    }
    let all = recipes.keys().copied().collect();
    let before = root(&all, recipes, &baseline).ok_or(VectorError::LiveSubstrateUnavailable)?;
    if decode(OPERATOR_SUCCESSOR_MERKLE_ROOT).as_deref() != Some(before.as_slice()) {
        return unavailable();
    }
    let committed = root(&all, recipes, &leaves).ok_or(VectorError::LiveSubstrateUnavailable)?;
    let key = crate::live_capability::OracleLiveCurve::new(target.clone())
        .output_key(constructor.internal_key().bytes(), &committed)
        .ok_or(VectorError::LiveSubstrateUnavailable)?;
    transaction::taproot::witness_program_script(&target, 1, key.key())
        .map_err(|_| VectorError::LiveSubstrateUnavailable)
}

#[cfg(test)]
mod tests {
    #[test]
    fn retained_successor_is_distinct_from_the_current_candidate() {
        let abi = crate::live_plan::demonstration_live_abi().expect("current ABI");
        let owner = linker::OwnerParameter::new(
            crate::live_plan::owner_key(
                &target_elements_conformance::test_material::PublicTestSignerHandle::First
                    .x_only_public_key()
                    .expect("public key"),
            )
            .expect("key encoding"),
        );
        let current = abi
            .destinations()
            .get(
                &owner,
                linker::live_backend::LiveTransferRepresentationPlan::Explicit,
            )
            .expect("current destination");
        assert_ne!(
            super::operator_successor(&crate::live_plan::PROTOCOL_ASSET)
                .expect("digest-bound historical tree"),
            current.instance().program()
        );
    }
}
