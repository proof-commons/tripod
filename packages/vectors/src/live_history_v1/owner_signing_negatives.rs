//! Historical owner-signing refusal observations.

use target_elements_conformance::protocol::ObservedOutcomeLayer;

/// The identity the target computed for the accepted control.
///
/// The unmutated vault-control-entitlement candidate, accepted after
/// the bare-u mutant had been refused — which is what makes the
/// mutant's refusal attributable rather than merely recorded.
pub const CONTROL_ACCEPTED_TXID: &str =
    "40cb6c4ee284ed38555a4840198c8130d1e2c3246b57b9d8b93842c3c6730029";

crate::recorded_acceptance::mint_recorded_acceptance!(control_accepted, CONTROL_ACCEPTED_TXID);

/// What the target said to the bare-u-output mutant, verbatim.
///
/// The coordinator leaf's `InspectOutputScriptPubKey` version clause,
/// reached because the mutant is re-signed over its own bytes and its
/// asset and value are unchanged, so the per-asset tally passes and the
/// leaf runs. The verdict is the leaf's own `EQUALVERIFY`, observed at
/// the script layer rather than at consensus.
pub const MUTANT_REJECT_DETAIL: &str =
    "mandatory-script-verify-flag-failed (Script failed an OP_EQUALVERIFY operation)";

/// The half-open witnessless byte range the mutant declared and stayed
/// within: the mutated destination's program.
pub const DECLARED_FIELD_RANGE: (usize, usize) = (132, 167);

/// How many bytes the bare-u mutant handed the node.
pub const MUTANT_SUBMITTED_BYTES: usize = 1152;

/// What the target said to every consensus-conservation mutant,
/// verbatim, at [`ObservedOutcomeLayer::ConsensusRejectionBeforeScript`].
///
/// The seven mutants all break the explicit per-asset sum, and the
/// target folds each break into the one balance verdict — the internal
/// tally and surjection codes never leave `VerifyAmounts` — so the
/// WORDS are identical and it is the declared field range, or the
/// transaction shape for the structural rows, that tells the rows
/// apart. The same discipline the conservation ceremony's
/// proof-negatives rest on, on the explicit successor.
pub const CONSENSUS_MUTANT_REJECT_DETAIL: &str = "bad-txns-in-ne-out";

/// The witnessless field range and 2-in-2-out shape the
/// `wrong-explicit-asset` mutant declared: the first receipt's asset
/// identifier, rewritten to a different explicit asset.
pub const WRONG_EXPLICIT_ASSET_FIELD_RANGE: (usize, usize) = (90, 122);

/// The `confidential-asset-commitment` mutant's field range: the second
/// receipt's asset explicitness prefix, flipped from explicit to a
/// blinded commitment with no surjection proof.
pub const CONFIDENTIAL_ASSET_COMMITMENT_FIELD_RANGE: (usize, usize) = (167, 168);

/// The `output-total-one-below-input` mutant's field range: the first
/// receipt's explicit value, lowered by one.
pub const OUTPUT_TOTAL_ONE_BELOW_FIELD_RANGE: (usize, usize) = (130, 131);

/// The `output-total-one-above-input` mutant's field range: the second
/// receipt's explicit value, raised by one.
pub const OUTPUT_TOTAL_ONE_ABOVE_FIELD_RANGE: (usize, usize) = (208, 209);

/// The structural range the two output-cardinality mutants share, and
/// the shapes that separate them.
///
/// `changed_range` cannot localize an insertion or a deletion past the
/// output-count varint, so `private-output-omitted` (a receipt removed)
/// and `hidden-private-u-output` (an output added) both declare this
/// range on the 2-in-2-out control. It is the SHAPE that separates
/// them — the removal leaves two inputs and ONE output, the addition
/// two inputs and THREE — which is the "distinct transaction structure"
/// the attributability rule admits beside a distinct field range.
pub const OUTPUT_CARDINALITY_FIELD_RANGE: (usize, usize) = (88, 245);
/// `private-output-omitted`'s shape: two inputs, one output.
pub const PRIVATE_OUTPUT_OMITTED_SHAPE: (usize, usize) = (2, 1);
/// `hidden-private-u-output`'s shape: two inputs, three outputs.
pub const HIDDEN_PRIVATE_U_OUTPUT_SHAPE: (usize, usize) = (2, 3);

/// The `omitted-source` mutant's field range and one-input shape: a
/// receipt input deleted, which drops the input sum.
pub const OMITTED_SOURCE_FIELD_RANGE: (usize, usize) = (5, 80);
/// `omitted-source`'s shape: one input, two outputs.
pub const OMITTED_SOURCE_SHAPE: (usize, usize) = (1, 2);

/// What the target said to the `two-coordinators` leaf-arrangement
/// mutant, verbatim.
///
/// The coordinator leaf revealed at input one runs the coordinator
/// index check `PushCurrentInputIndex; 0; EqualVerify` at index one and
/// fails it — the one input that fails, the other being the control's
/// valid coordinator. Reached because the mutant is re-signed over its
/// rearranged census and its shape and amounts are the control's, so
/// the per-asset tally passes and the leaf runs. The verdict READS the
/// same `OP_EQUALVERIFY` as the bare-u mutant, at a different clause, and
/// the rows separate by their distinct mutation — a second coordinator
/// leaf against a rewritten output program.
pub const TWO_COORDINATORS_REJECT_DETAIL: &str =
    "mandatory-script-verify-flag-failed (Script failed an OP_EQUALVERIFY operation)";

/// The `two-coordinators` mutant's revealed-leaf arrangement: the
/// coordinator leaf (receipt position zero) at BOTH inputs.
pub const TWO_COORDINATORS_ARRANGEMENT: [u16; 2] = [0, 0];

/// What the target said to the `no-coordinator` leaf-arrangement
/// mutant, verbatim.
///
/// The member leaf revealed at input zero runs the member bound's lower
/// check `... 1; GreaterThanOrEqual64; Verify` at index zero and fails
/// it — the one input that fails, the other being the control's valid
/// member. The verdict is an `OP_VERIFY`, which tells this row from the
/// two coordinator-index rows that draw `OP_EQUALVERIFY`.
pub const NO_COORDINATOR_REJECT_DETAIL: &str =
    "mandatory-script-verify-flag-failed (Script failed an OP_VERIFY operation)";

/// The `no-coordinator` mutant's revealed-leaf arrangement: the member
/// leaf (receipt position one) at BOTH inputs.
pub const NO_COORDINATOR_ARRANGEMENT: [u16; 2] = [1, 1];

/// The control's own revealed-leaf arrangement: the coordinator leaf at
/// input zero, the member leaf at input one. Each mutant's arrangement
/// is distinct from this and from the other's.
pub const CONTROL_ARRANGEMENT: [u16; 2] = [0, 1];

/// The run's wall time, in seconds.
///
/// Up from the pre-leaf-arrangement 8.3s: the ceremony now submits two
/// more mutants, the leaf-arrangement pair, before the control.
pub const WALL_SECONDS: f64 = 10.2;

/// The layer the target refused the bare-u mutant at, TYPED.
///
/// The recorded observation, as the vocabulary rather than as a
/// sentence about it. The words above already said "observed at the
/// script layer rather than at consensus" and nothing could read that
/// prose, so a classifier that needed the layer had to guess it from
/// the row's own declaration — which is how a consensus refusal came
/// to stand as a script-path answer. These constants are the run's
/// own record of WHERE, bound in the native test beside the words and
/// the identity, and they move no recorded value.
pub const MUTANT_OBSERVED_LAYER: ObservedOutcomeLayer = ObservedOutcomeLayer::ScriptPathRejection;

/// The layer the target refused every consensus-conservation mutant
/// at, TYPED: before any script ran, which is the whole content of
/// the seven rows' retype.
pub const CONSENSUS_MUTANT_OBSERVED_LAYER: ObservedOutcomeLayer =
    ObservedOutcomeLayer::ConsensusRejectionBeforeScript;

/// The layer the `two-coordinators` mutant was refused at, TYPED.
pub const TWO_COORDINATORS_OBSERVED_LAYER: ObservedOutcomeLayer =
    ObservedOutcomeLayer::ScriptPathRejection;

/// The layer the `no-coordinator` mutant was refused at, TYPED.
pub const NO_COORDINATOR_OBSERVED_LAYER: ObservedOutcomeLayer =
    ObservedOutcomeLayer::ScriptPathRejection;
