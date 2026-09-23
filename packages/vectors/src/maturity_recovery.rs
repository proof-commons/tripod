//! Public successor reconstruction under Guide 14 §§14.7 and 16.13 (`T11-116`).
//!
//! A handoff carries exact transaction bytes and published deployment facts.
//! Recovery consumes that handoff alone. Its comparison answers whether this
//! transaction's successor program is reconstructible from those values;
//! it makes no chain freshness, origin, relayability, or leastness claim.

use architecture::ids::{ObjectId, OperationId};
use architecture::spec::ARCHITECTURE;
use realization::{
    AnnouncementLeadBounds, Cycle, EncodedStateMetadata, MaturityTransitionRefusal, StateMetadata,
    StateRepresentationNonce, announce_maturity,
};
use tapscript::{
    StateConstructorRefusal, StateInternalKeyPolicy, StateStaticSubtree, StateWitnessSchedule,
    state_metadata_leaf_program, state_output_program_at_nonce,
};
use target_elements::TargetContractVersion;
use transaction::bytes::{TargetTransaction, Txid};
use transaction::taproot::{Digest32, leaf_hash};

use crate::maturity_closure::{MaturityClosureRefusal, OracleStateCurve, closure_target};
use crate::maturity_continuity::{
    MaturityByteComparison, MaturityByteSource, MaturityContinuityRefusal,
    decoded_witness_metadata, submitted_transaction_identities, witness,
};

/// A public source and the transaction identity claimed for its bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicAnnouncementLocator {
    source: MaturityByteSource,
    identity: Txid,
}

impl PublicAnnouncementLocator {
    /// Pair a public source with its claimed identity.
    #[must_use]
    pub const fn new(source: MaturityByteSource, identity: Txid) -> Self {
        Self { source, identity }
    }

    /// The source naming where the bytes were published.
    #[must_use]
    pub const fn source(&self) -> &MaturityByteSource {
        &self.source
    }

    /// The claimed witness-stripped transaction identity.
    #[must_use]
    pub const fn identity(&self) -> Txid {
        self.identity
    }
}

/// The public input to a process that recovers one announcement successor.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicAnnouncementHandoff {
    locator: PublicAnnouncementLocator,
    bytes: Vec<u8>,
    state_output_index: u32,
    schedule: StateWitnessSchedule,
    bounds: AnnouncementLeadBounds,
    /// The deployment's published taptree; its one-leaf announcement form is
    /// reproducible from the published transaction's leaf witness item.
    static_subtree: StateStaticSubtree,
    internal_key: StateInternalKeyPolicy,
    contract: TargetContractVersion,
}

impl PublicAnnouncementHandoff {
    /// Carry public bytes and the published deployment facts needed to check them.
    #[must_use]
    #[expect(
        clippy::too_many_arguments,
        reason = "The handoff carries eight public inputs in field order."
    )]
    #[expect(
        clippy::missing_const_for_fn,
        reason = "The handoff is assembled from runtime publication values and refuses nothing at construction."
    )]
    pub fn new(
        locator: PublicAnnouncementLocator,
        bytes: Vec<u8>,
        state_output_index: u32,
        schedule: StateWitnessSchedule,
        bounds: AnnouncementLeadBounds,
        static_subtree: StateStaticSubtree,
        internal_key: StateInternalKeyPolicy,
        contract: TargetContractVersion,
    ) -> Self {
        Self {
            locator,
            bytes,
            state_output_index,
            schedule,
            bounds,
            static_subtree,
            internal_key,
            contract,
        }
    }

    /// The source and claimed identity.
    #[must_use]
    pub const fn locator(&self) -> &PublicAnnouncementLocator {
        &self.locator
    }

    /// Exact published transaction bytes.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// The declared STATE output position.
    #[must_use]
    pub const fn state_output_index(&self) -> u32 {
        self.state_output_index
    }

    /// The published witness schedule.
    #[must_use]
    pub const fn schedule(&self) -> StateWitnessSchedule {
        self.schedule
    }

    /// The published announcement lead bounds.
    #[must_use]
    pub const fn bounds(&self) -> AnnouncementLeadBounds {
        self.bounds
    }

    /// The published whole static subtree.
    #[must_use]
    pub const fn static_subtree(&self) -> &StateStaticSubtree {
        &self.static_subtree
    }

    /// The published internal key policy.
    #[must_use]
    pub const fn internal_key(&self) -> StateInternalKeyPolicy {
        self.internal_key
    }

    /// The stated contract revision.
    #[must_use]
    pub const fn contract(&self) -> TargetContractVersion {
        self.contract
    }
}

/// Exhaustive roster of the handoff's public inputs, in field order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PublicHandoffInput {
    /// Public transaction source and claimed identity.
    TransactionLocator,
    /// Exact published transaction bytes.
    ExactTransactionBytes,
    /// Published STATE output position.
    StateOutputIndex,
    /// Published witness schedule.
    WitnessSchedule,
    /// Published announcement lead bounds.
    LeadBounds,
    /// Published whole static subtree.
    StaticSubtree,
    /// Published internal key policy.
    InternalKey,
    /// Stated target contract revision.
    ContractRevision,
}

impl PublicHandoffInput {
    /// The complete roster, in handoff field order.
    pub const ALL: &'static [Self; 8] = &[
        Self::TransactionLocator,
        Self::ExactTransactionBytes,
        Self::StateOutputIndex,
        Self::WitnessSchedule,
        Self::LeadBounds,
        Self::StaticSubtree,
        Self::InternalKey,
        Self::ContractRevision,
    ];

    /// The public item this member names.
    #[must_use]
    pub const fn item(self) -> &'static str {
        match self {
            Self::TransactionLocator => "transaction locator",
            Self::ExactTransactionBytes => "exact transaction bytes",
            Self::StateOutputIndex => "STATE output index",
            Self::WitnessSchedule => "published witness schedule",
            Self::LeadBounds => "published announcement lead bounds",
            Self::StaticSubtree => "published static subtree",
            Self::InternalKey => "published internal key policy",
            Self::ContractRevision => "published target contract revision",
        }
    }
}

/// A successor whose reconstructed program agrees with the stated output.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecoveredSuccessor {
    predecessor_metadata: EncodedStateMetadata,
    successor_semantics: StateMetadata,
    representation: StateRepresentationNonce,
    requested_cycle: Cycle,
    program: MaturityByteComparison,
    contract: TargetContractVersion,
}

impl RecoveredSuccessor {
    /// Decoded predecessor metadata.
    #[must_use]
    pub const fn predecessor_metadata(&self) -> &EncodedStateMetadata {
        &self.predecessor_metadata
    }

    /// Derived successor semantics.
    #[must_use]
    pub const fn successor_semantics(&self) -> &StateMetadata {
        &self.successor_semantics
    }

    /// The committed witnessed representation nonce.
    #[must_use]
    pub const fn representation(&self) -> StateRepresentationNonce {
        self.representation
    }

    /// The witnessed requested cycle.
    #[must_use]
    pub const fn requested_cycle(&self) -> Cycle {
        self.requested_cycle
    }

    /// Program reconstructed from public values.
    #[must_use]
    pub fn reconstructed_program(&self) -> &[u8] {
        self.program.reconstructed()
    }

    /// Program carried by the stated transaction output.
    #[must_use]
    pub fn actual_program(&self) -> &[u8] {
        self.program.witnessed()
    }

    /// Agreement of those exact program operands.
    #[must_use]
    pub fn programs_agree(&self) -> bool {
        self.program.agrees()
    }

    /// Stated contract revision carried through recovery.
    #[must_use]
    pub const fn contract(&self) -> TargetContractVersion {
        self.contract
    }
}

/// First refusal of the public recovery process.
///
/// A missing reviewed contract, a refused fixed commitment, and a differing
/// reconstructed program all answer the program row: none yields the program
/// that the stated chain output carries.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MaturityRecoveryRefusal {
    /// No transaction can be decoded from the offered bytes.
    MissingTransaction { locator: Txid, offered: usize },
    /// The bytes' identity differs from the locator's claim.
    CopiedTransactionBytes { locator: Txid, recomputed: Txid },
    /// The stated output position differs from the architecture's STATE output.
    WrongOutputIndex { stated: u32, declared: u32 },
    /// The requested cycle fails the published lead bounds.
    StaleSuccessor {
        requested: Cycle,
        bounds: AnnouncementLeadBounds,
        refusal: Box<MaturityTransitionRefusal>,
    },
    /// A public witness field is malformed.
    MalformedPublicWitness {
        refusal: Box<MaturityContinuityRefusal>,
    },
    /// The metadata leaf disagrees with the control path's final sibling.
    MetadataFromAnotherSuccessor {
        witnessed: Digest32,
        control: Digest32,
    },
    /// Witness item one has the wrong width for a representation nonce.
    MissingRepresentationNonce { expected: usize, actual: usize },
    /// Witness item three differs from the published static root.
    WrongStaticSubtree {
        witnessed: Digest32,
        published: Digest32,
    },
    /// The reviewed target could not be obtained.
    ReviewedContractUnavailable {
        refusal: Box<MaturityClosureRefusal>,
    },
    /// The fixed successor commitment refused the witnessed nonce.
    SuccessorCommitmentRefused {
        refusal: Box<StateConstructorRefusal>,
    },
    /// Reconstructed and actual output programs disagree.
    ReconstructedProgramDiffers { comparison: MaturityByteComparison },
}

impl MaturityRecoveryRefusal {
    /// The §16.13 row answered by this refusal.
    #[must_use]
    #[expect(
        clippy::match_same_arms,
        reason = "Each refusal explicitly names its row so the total mapping cannot inherit a verdict."
    )]
    pub const fn row(&self) -> &'static str {
        match self {
            Self::MissingTransaction { .. } => "missing-transaction",
            Self::CopiedTransactionBytes { .. } => "copied-transaction-bytes",
            Self::WrongOutputIndex { .. } => "wrong-output-index",
            Self::StaleSuccessor { .. } => "stale-successor",
            Self::MalformedPublicWitness { .. } => "malformed-public-witness",
            Self::MetadataFromAnotherSuccessor { .. } => "metadata-from-another-successor",
            Self::MissingRepresentationNonce { .. } => "missing-representation-nonce",
            Self::WrongStaticSubtree { .. } => "wrong-static-subtree",
            Self::ReviewedContractUnavailable { .. } => {
                "reconstructed-program-differs-from-chain-output"
            }
            Self::SuccessorCommitmentRefused { .. } => {
                "reconstructed-program-differs-from-chain-output"
            }
            Self::ReconstructedProgramDiffers { .. } => {
                "reconstructed-program-differs-from-chain-output"
            }
        }
    }
}

/// Copy a width-checked witness item into a fixed-size byte array.
fn fixed_bytes<const N: usize>(bytes: &[u8]) -> [u8; N] {
    let mut result = [0; N];
    result.copy_from_slice(bytes);
    result
}

/// Recover the successor program from one public announcement handoff.
///
/// The reviewed contract's leaf version is used for the metadata hash. The
/// control block is checked for its own minimum width before its final sibling
/// is read; the shared witness reader checks the other fixed item widths.
///
/// # Errors
/// Returns the first §16.13 refusal: transaction bytes, locator identity,
/// declared output index, public witness (including a short control block),
/// static root, metadata path, lead bounds, fixed commitment, or output program.
#[expect(
    clippy::too_many_lines,
    reason = "Nine ordered public comparisons retain their first refusal and its row."
)]
#[expect(
    clippy::needless_pass_by_value,
    reason = "The public recovery process takes ownership of its sole handoff input."
)]
pub fn recover_public_successor(
    handoff: PublicAnnouncementHandoff,
) -> Result<RecoveredSuccessor, MaturityRecoveryRefusal> {
    use MaturityRecoveryRefusal as Refusal;

    let transaction =
        TargetTransaction::decode(handoff.bytes()).map_err(|_| Refusal::MissingTransaction {
            locator: handoff.locator().identity(),
            offered: handoff.bytes().len(),
        })?;
    let recomputed = submitted_transaction_identities(&transaction, handoff.bytes()).identity();
    if recomputed != handoff.locator().identity() {
        return Err(Refusal::CopiedTransactionBytes {
            locator: handoff.locator().identity(),
            recomputed,
        });
    }

    let stated = handoff.state_output_index();
    let declared = ARCHITECTURE
        .operation(OperationId::AnnounceMaturity)
        .and_then(|spec| {
            spec.outputs
                .iter()
                .position(|output| output.object == ObjectId::State)
        })
        .and_then(|position| u32::try_from(position).ok())
        .unwrap_or(u32::MAX);
    if stated != declared {
        return Err(Refusal::WrongOutputIndex { stated, declared });
    }

    let stack = witness(&transaction, handoff.schedule()).map_err(|refusal| match refusal {
        MaturityContinuityRefusal::WitnessWidth {
            index: 1,
            expected,
            actual,
        } => Refusal::MissingRepresentationNonce { expected, actual },
        other => Refusal::MalformedPublicWitness {
            refusal: Box::new(other),
        },
    })?;
    let [
        _,
        nonce_bytes,
        cycle_bytes,
        root_bytes,
        _,
        _,
        _,
        _,
        control_bytes,
    ] = stack
    else {
        return Err(Refusal::MalformedPublicWitness {
            refusal: Box::new(MaturityContinuityRefusal::WitnessItemCount {
                actual: stack.len(),
            }),
        });
    };

    let witnessed = fixed_bytes::<32>(root_bytes);
    let published = *handoff.static_subtree().root();
    if witnessed != published {
        return Err(Refusal::WrongStaticSubtree {
            witnessed,
            published,
        });
    }

    let target = closure_target().map_err(|refusal| Refusal::ReviewedContractUnavailable {
        refusal: Box::new(refusal),
    })?;
    let metadata =
        decoded_witness_metadata(stack, handoff.schedule(), &target).map_err(|refusal| {
            Refusal::MalformedPublicWitness {
                refusal: Box::new(refusal),
            }
        })?;
    let minimum_control_width = 33;
    if control_bytes.len() < minimum_control_width {
        return Err(Refusal::MalformedPublicWitness {
            refusal: Box::new(MaturityContinuityRefusal::WitnessWidth {
                index: 8,
                expected: minimum_control_width,
                actual: control_bytes.len(),
            }),
        });
    }
    let leaf_program = state_metadata_leaf_program(&target, &metadata).map_err(|refusal| {
        Refusal::SuccessorCommitmentRefused {
            refusal: Box::new(refusal),
        }
    })?;
    let witnessed = leaf_hash(
        target.definition().leaf_version(),
        &leaf_program.encode(&target),
    );
    let tail = control_bytes
        .get(control_bytes.len() - 32..)
        .ok_or_else(|| Refusal::MalformedPublicWitness {
            refusal: Box::new(MaturityContinuityRefusal::WitnessWidth {
                index: 8,
                expected: minimum_control_width,
                actual: control_bytes.len(),
            }),
        })?;
    let control = fixed_bytes::<32>(tail);
    if witnessed != control {
        return Err(Refusal::MetadataFromAnotherSuccessor { witnessed, control });
    }

    let requested_cycle = Cycle::new(u64::from_be_bytes(fixed_bytes::<8>(cycle_bytes)));
    let successor_semantics =
        announce_maturity(&metadata.semantic, requested_cycle, handoff.bounds()).map_err(
            |refusal| Refusal::StaleSuccessor {
                requested: requested_cycle,
                bounds: handoff.bounds(),
                refusal: Box::new(refusal),
            },
        )?;
    let representation =
        StateRepresentationNonce::new(u32::from_be_bytes(fixed_bytes::<4>(nonce_bytes)));
    let successor_metadata = EncodedStateMetadata {
        semantic: successor_semantics,
        representation,
    };
    let reconstructed = state_output_program_at_nonce(
        &target,
        &successor_metadata,
        handoff.static_subtree(),
        handoff.internal_key(),
        &OracleStateCurve,
    )
    .map_err(|refusal| Refusal::SuccessorCommitmentRefused {
        refusal: Box::new(refusal),
    })?;

    let actual = usize::try_from(stated)
        .ok()
        .and_then(|index| transaction.outputs().get(index))
        .ok_or(Refusal::WrongOutputIndex { stated, declared })?;
    let comparison = MaturityByteComparison::new(actual.program(), &reconstructed);
    if !comparison.agrees() {
        return Err(Refusal::ReconstructedProgramDiffers { comparison });
    }
    Ok(RecoveredSuccessor {
        predecessor_metadata: metadata,
        successor_semantics,
        representation,
        requested_cycle,
        program: comparison,
        contract: handoff.contract(),
    })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;
    use crate::maturity_closure::MaturityDeployment;
    use crate::maturity_continuity::tests::archived;
    use crate::maturity_corpus::maturity_variable_run_of_record;
    use crate::maturity_native::MaturityAcceptanceObligation;
    use realization::{
        STATE_METADATA_BYTES, decode_state_metadata, encode_state_metadata,
        state_metadata_variable_region,
    };
    use tapscript::{StateLeafRole, StateStaticLeaf, StateStaticNode, TapscriptProgram};
    use target_elements::ReviewedElementsTapscriptDefinition;
    use transaction::bytes::{InputWitness, TargetOutput};

    /// The one published announcement leaf is the whole static subtree:
    /// the recipe carries every consumer within it and needs no support leaf.
    fn published_subtree(
        target: &ReviewedElementsTapscriptDefinition,
        leaf_bytes: &[u8],
    ) -> StateStaticSubtree {
        let program = TapscriptProgram::decode(target, leaf_bytes).expect("published leaf");
        StateStaticSubtree::new(
            target,
            Some(StateStaticNode::Leaf {
                identity: 0,
                leaf: StateStaticLeaf {
                    role: StateLeafRole::Announcement,
                    program,
                    version: target.definition().leaf_version().get(),
                },
            }),
        )
        .expect("published one-leaf subtree")
    }

    fn accepted_handoff() -> PublicAnnouncementHandoff {
        let corpus = maturity_variable_run_of_record().expect("accepted archive");
        let MaturityAcceptanceObligation::Established { readback, .. } =
            corpus.evidence().acceptance_obligation()
        else {
            panic!("accepted archive has an established readback")
        };
        let target = closure_target().expect("reviewed target");
        let transaction = TargetTransaction::decode(readback.bytes()).expect("accepted bytes");
        let subtree = published_subtree(&target, &transaction.witnesses()[0].stack()[7]);
        let (minimum, maximum) = MaturityDeployment::PublishedSignerHeld
            .parameters()
            .expect("deployment")
            .lead();
        PublicAnnouncementHandoff::new(
            PublicAnnouncementLocator::new(
                MaturityByteSource::ArchivedSubmission {
                    run_address: corpus.report().run_address().to_owned(),
                },
                readback.identity(),
            ),
            readback.bytes().to_vec(),
            0,
            corpus.schedule(),
            AnnouncementLeadBounds::new(Cycle::new(minimum), Cycle::new(maximum))
                .expect("lead bounds"),
            subtree,
            StateInternalKeyPolicy::new(tapscript::STATE_NUMS_KEY, &OracleStateCurve)
                .expect("published key"),
            TargetContractVersion::V2,
        )
    }

    fn changed_stack(bytes: &[u8], change: impl FnOnce(&mut Vec<Vec<u8>>)) -> Vec<u8> {
        let transaction = TargetTransaction::decode(bytes).expect("accepted transaction");
        let mut stack = transaction.witnesses()[0].stack().to_vec();
        change(&mut stack);
        TargetTransaction::with_output_witnesses(
            transaction.version(),
            transaction.inputs().to_vec(),
            transaction.outputs().to_vec(),
            transaction.lock_time(),
            vec![InputWitness::new(stack)],
            transaction.output_witnesses().to_vec(),
        )
        .expect("mutant transaction")
        .encode()
    }

    fn changed_program(bytes: &[u8], program: Vec<u8>) -> Vec<u8> {
        let transaction = TargetTransaction::decode(bytes).expect("accepted transaction");
        let output = &transaction.outputs()[0];
        let replaced = TargetOutput::new(output.asset(), output.value(), output.nonce(), program);
        TargetTransaction::with_output_witnesses(
            transaction.version(),
            transaction.inputs().to_vec(),
            vec![replaced],
            transaction.lock_time(),
            transaction.witnesses().to_vec(),
            transaction.output_witnesses().to_vec(),
        )
        .expect("mutant transaction")
        .encode()
    }

    #[test]
    fn missing_transaction_refuses_the_handoff_with_no_bytes() {
        for offered in [Vec::new(), vec![0]] {
            let mut handoff = accepted_handoff();
            let locator = handoff.locator.identity();
            handoff.bytes = offered;
            let error = recover_public_successor(handoff.clone()).expect_err("missing bytes");
            assert_eq!(
                error,
                MaturityRecoveryRefusal::MissingTransaction {
                    locator,
                    offered: handoff.bytes.len()
                }
            );
            assert_eq!(error.row(), "missing-transaction");
        }
    }

    #[test]
    fn copied_transaction_bytes_refuse_against_the_locators_identity() {
        let mut handoff = accepted_handoff();
        let historical = archived().input();
        let transaction = TargetTransaction::decode(historical.submitted_bytes).expect("history");
        let other =
            submitted_transaction_identities(&transaction, historical.submitted_bytes).identity();
        let accepted = handoff.locator.identity();
        assert_ne!(other, accepted);
        handoff.locator.identity = other;
        let error = recover_public_successor(handoff).expect_err("copied bytes");
        assert_eq!(
            error,
            MaturityRecoveryRefusal::CopiedTransactionBytes {
                locator: other,
                recomputed: accepted
            }
        );
        assert_eq!(error.row(), "copied-transaction-bytes");
    }

    #[test]
    fn wrong_output_index_refuses_against_the_declared_state_position() {
        let mut handoff = accepted_handoff();
        let declared = ARCHITECTURE
            .operation(OperationId::AnnounceMaturity)
            .expect("operation")
            .outputs
            .iter()
            .position(|output| output.object == ObjectId::State)
            .expect("STATE output");
        let declared = u32::try_from(declared).expect("declared position fits");
        handoff.state_output_index = declared + 1;
        let error = recover_public_successor(handoff).expect_err("wrong index");
        assert_eq!(
            error,
            MaturityRecoveryRefusal::WrongOutputIndex {
                stated: declared + 1,
                declared
            }
        );
        assert_eq!(error.row(), "wrong-output-index");
    }

    #[test]
    fn stale_successor_refuses_a_cycle_outside_the_published_lead_window() {
        let mut handoff = accepted_handoff();
        let requested = Cycle::new(100);
        handoff.bytes = changed_stack(&handoff.bytes, |stack| {
            stack[2] = requested.get().to_be_bytes().to_vec();
        });
        let error = recover_public_successor(handoff.clone()).expect_err("stale cycle");
        assert_eq!(error.row(), "stale-successor");
        assert!(matches!(
            error,
            MaturityRecoveryRefusal::StaleSuccessor { requested: seen, bounds, refusal }
                if seen == requested
                    && bounds == handoff.bounds
                    && *refusal == MaturityTransitionRefusal::AnnouncementAboveMaximum
        ));
    }

    #[test]
    fn malformed_public_witness_refuses_a_short_stack_and_a_wrong_width() {
        let handoff = accepted_handoff();
        let mut short = handoff.clone();
        short.bytes = changed_stack(&short.bytes, |stack| {
            stack.pop();
        });
        let error = recover_public_successor(short).expect_err("short stack");
        assert_eq!(error.row(), "malformed-public-witness");
        assert!(matches!(
            error,
            MaturityRecoveryRefusal::MalformedPublicWitness { refusal }
                if *refusal == MaturityContinuityRefusal::WitnessItemCount { actual: 8 }
        ));

        let mut width = handoff.clone();
        width.bytes = changed_stack(&width.bytes, |stack| {
            stack[3].pop();
        });
        let error = recover_public_successor(width).expect_err("wrong width");
        assert_eq!(error.row(), "malformed-public-witness");
        assert!(matches!(
            error,
            MaturityRecoveryRefusal::MalformedPublicWitness { refusal }
                if *refusal == MaturityContinuityRefusal::WitnessWidth {
                    index: 3, expected: 32, actual: 31
                }
        ));

        let mut control = handoff;
        control.bytes = changed_stack(&control.bytes, |stack| {
            stack[8] = vec![0; 32];
        });
        let error = recover_public_successor(control).expect_err("short control");
        assert_eq!(error.row(), "malformed-public-witness");
        assert!(matches!(
            error,
            MaturityRecoveryRefusal::MalformedPublicWitness { refusal }
                if *refusal == MaturityContinuityRefusal::WitnessWidth {
                    index: 8, expected: 33, actual: 32
                }
        ));
    }

    #[test]
    fn metadata_from_another_successor_refuses_against_the_control_path() {
        let historical = archived().input();
        let transaction = TargetTransaction::decode(historical.submitted_bytes).expect("history");
        let canonical: [u8; STATE_METADATA_BYTES] = transaction.witnesses()[0].stack()[4]
            .as_slice()
            .try_into()
            .expect("whole metadata");
        let mut handoff = accepted_handoff();
        let accepted = TargetTransaction::decode(&handoff.bytes).expect("accepted bytes");
        assert_eq!(
            state_metadata_variable_region(&canonical).as_slice(),
            accepted.witnesses()[0].stack()[4]
        );
        let historical_metadata = decode_state_metadata(&canonical).expect("historical metadata");
        let requested = Cycle::new(u64::from_be_bytes(
            transaction.witnesses()[0].stack()[2]
                .as_slice()
                .try_into()
                .expect("historical cycle width"),
        ));
        let other_successor =
            announce_maturity(&historical_metadata.semantic, requested, handoff.bounds)
                .expect("historical successor");
        let other_encoding =
            encode_state_metadata(&other_successor, StateRepresentationNonce::ZERO);
        let other_canonical: [u8; STATE_METADATA_BYTES] = other_encoding
            .as_slice()
            .try_into()
            .expect("canonical successor width");
        let variable = state_metadata_variable_region(&other_canonical);
        handoff.bytes = changed_stack(&handoff.bytes, |stack| {
            stack[4] = variable.to_vec();
        });
        let error = recover_public_successor(handoff).expect_err("other metadata");
        assert_eq!(error.row(), "metadata-from-another-successor");
        assert!(matches!(
            error,
            MaturityRecoveryRefusal::MetadataFromAnotherSuccessor { witnessed, control }
                if witnessed != control
        ));
    }

    #[test]
    fn missing_representation_nonce_refuses_the_emptied_witness_item() {
        let mut handoff = accepted_handoff();
        handoff.bytes = changed_stack(&handoff.bytes, |stack| {
            stack[1].clear();
        });
        let error = recover_public_successor(handoff).expect_err("empty nonce");
        assert_eq!(
            error,
            MaturityRecoveryRefusal::MissingRepresentationNonce {
                expected: 4,
                actual: 0
            }
        );
        assert_eq!(error.row(), "missing-representation-nonce");
    }

    #[test]
    fn wrong_static_subtree_refuses_against_the_published_root() {
        let mut handoff = accepted_handoff();
        let published = *handoff.static_subtree.root();
        handoff.bytes = changed_stack(&handoff.bytes, |stack| {
            stack[3][0] ^= 1;
        });
        let error = recover_public_successor(handoff).expect_err("wrong root");
        assert_eq!(error.row(), "wrong-static-subtree");
        assert!(matches!(
            error,
            MaturityRecoveryRefusal::WrongStaticSubtree { witnessed, published: actual }
                if witnessed != published && actual == published
        ));
    }

    #[test]
    fn a_reconstruction_that_differs_from_output_zero_refuses_with_both_programs() {
        let mut handoff = accepted_handoff();
        let transaction = TargetTransaction::decode(&handoff.bytes).expect("accepted bytes");
        let mut program = transaction.outputs()[0].program().to_vec();
        program[2] ^= 1;
        handoff.bytes = changed_program(&handoff.bytes, program.clone());
        let changed = TargetTransaction::decode(&handoff.bytes).expect("changed output");
        handoff.locator.identity =
            submitted_transaction_identities(&changed, &handoff.bytes).identity();
        let error = recover_public_successor(handoff).expect_err("different output");
        assert_eq!(
            error.row(),
            "reconstructed-program-differs-from-chain-output"
        );
        assert!(matches!(
            error,
            MaturityRecoveryRefusal::ReconstructedProgramDiffers { comparison }
                if comparison.witnessed() == program && comparison.reconstructed() != program
        ));
    }

    #[test]
    #[expect(
        clippy::type_complexity,
        reason = "The function pointer binds all eight handoff constructor operands as a construction proof."
    )]
    fn creator_process_memory_is_unnameable_by_the_handoffs_inputs() {
        let _: fn(
            PublicAnnouncementLocator,
            Vec<u8>,
            u32,
            StateWitnessSchedule,
            AnnouncementLeadBounds,
            StateStaticSubtree,
            StateInternalKeyPolicy,
            TargetContractVersion,
        ) -> PublicAnnouncementHandoff = PublicAnnouncementHandoff::new;
        assert_eq!(PublicHandoffInput::ALL.len(), 8);
        for input in PublicHandoffInput::ALL {
            assert!(!input.item().contains("retained instance"));
            assert!(!input.item().contains("deployment identity"));
        }
    }

    #[test]
    fn no_temporary_file_is_nameable_by_the_recovery_signature() {
        let _: fn(
            PublicAnnouncementHandoff,
        ) -> Result<RecoveredSuccessor, MaturityRecoveryRefusal> = recover_public_successor;
        for input in PublicHandoffInput::ALL {
            assert!(!input.item().contains("path"));
            assert!(!input.item().contains("file"));
        }
    }

    #[test]
    fn no_wallet_descriptor_or_operator_key_is_nameable_by_the_handoff() {
        assert_eq!(
            PublicHandoffInput::InternalKey.item(),
            "published internal key policy"
        );
        for input in PublicHandoffInput::ALL {
            assert!(!input.item().contains("descriptor"));
            assert!(!input.item().contains("operator key"));
        }
    }

    #[test]
    fn the_item_mapping_is_total_over_the_public_inputs() {
        let items: BTreeSet<_> = PublicHandoffInput::ALL
            .iter()
            .map(|item| item.item())
            .collect();
        assert_eq!(items.len(), PublicHandoffInput::ALL.len());
        assert_eq!(
            PublicHandoffInput::TransactionLocator.item(),
            "transaction locator"
        );
        assert_eq!(
            PublicHandoffInput::ExactTransactionBytes.item(),
            "exact transaction bytes"
        );
        assert_eq!(
            PublicHandoffInput::StateOutputIndex.item(),
            "STATE output index"
        );
    }
}
