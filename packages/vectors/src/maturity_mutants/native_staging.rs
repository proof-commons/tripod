//! Typed negative staging subjects for the project's own STATE covenant
//! programs on the maturity candidate of record.
//!
//! The candidate of record is the accepted variable-schedule maturity
//! control's deployment: the sponsorless announcement the planner rebuilds
//! byte-equal to the archived submission. The historical inventory partitions
//! its thirty-five §16 rows into staged refusals, expected acceptances,
//! subjects awaiting the sponsored fee form and unavailable constructions.
//!
//! # What this module stages, and what it does not
//!
//! The native planner restages subjects over live funded coins and submits
//! them before their consuming controls. Exact replay checks those live
//! requests. Archived-context digests remain the staging baseline: each
//! deterministic one-coordinate departure is pinned by its spend-time
//! serialization, so a changed baseline byte fails a test. A staging pin
//! alone supplies no observation or refusal credit.
//!
//! # One coordinate, and the collateral that balances it
//!
//! A subject that changed two independent coordinates would refuse for
//! either, and a run could not say which. So each subject moves one
//! coordinate and makes only the balancing edits that coordinate forces,
//! each named by the row rather than a second fault:
//!
//! - a changed signed field (an output, a nonce, the version, the
//!   sequence) is re-signed under the published operator key, so a broken
//!   signature is not the confound;
//! - a spent coin of another asset, amount or program moves the matching
//!   output field and the spent outpoint with it, because a coin's asset,
//!   amount or program is one change read on both sides of the spend, and
//!   the predecessor control block is re-committed under a changed program;
//! - the one two-input subject funds a state-like output of another asset
//!   from a key-path coin, with a stand-in outpoint in the baseline and a
//!   funded outpoint in the live request, because the extra output must
//!   balance; two state-asset successors await a construction that preserves
//!   singleton conservation and the retained issuance reserves.

use linker::CandidateLinkedMaturityBundle;
use realization::{
    Cycle, EncodedStateMetadata, Maturity, StateMetadata, StateRepresentationNonce, TransactionSide,
};
use tapscript::{
    StateConstructorRefusal, StateCurveCapability, StateLeafRole, StateTweakOutcome,
    StateWitnessSchedule,
};
use target_elements::LeafVersion;
use target_elements_conformance::executor::TargetOperationPlanner;
use target_elements_conformance::test_material::{PublicTestSignerHandle, TestSigningDefect};
use transaction::bytes::{
    AssetField, AssetId, InputWitness, Outpoint, OutputWitness, TargetInput, TargetOutput,
    TargetTransaction, ValueField,
};
use transaction::error::TransactionRefusal;
use transaction::script_path_signing::SpentOutputCensusEntry;
use transaction::taproot::{TAP_LEAF_TAG, branch_hash, tagged_hash};

use crate::maturity_closure::{
    MaturityClosureRefusal, MaturityDeployment, MaturityWitnessSelection, OracleStateCurve,
    maturity_sources,
};
use crate::maturity_continuity::{
    MaturityConstructorProjection, MaturityContinuityRefusal, MaturityFundedPredecessor,
    fixed_projection, prefix, witness,
};
use crate::maturity_corpus::maturity_variable_run_of_record;
use crate::maturity_native::{
    MaturityAnnouncementPlanner, MaturityLead, MaturityNativePlanRefusal,
};
use crate::maturity_operator::OPERATOR_HANDLE;
use crate::maturity_safety::MaturitySafetySection;

use super::MaturityMutantContext;

/// The §1.5 boundary a native-run-required row declares on this candidate.
///
/// Only two of the matrix's boundaries are reachable from the sponsorless
/// announcement this candidate builds: a script-path rejection for every
/// covenant clause, and a consensus rejection before script for the two
/// §16.11 transaction-field rows the node reads before any leaf runs.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum NativeBoundary {
    /// The covenant clause refuses inside the executing leaf.
    ScriptPathRejection,
    /// The node refuses a transaction field before any script runs.
    ConsensusRejectionBeforeScript,
}

/// Why a native-run-required row cannot be staged on this candidate.
///
/// Each member names a fact about the candidate or the covenant, not a
/// gap a later stager could close here: a run against this candidate
/// would establish something other than the row's own fault, or no bytes
/// the public constructors emit carry the change at all.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum NativeUnstageable {
    /// No superseded key exists; any other key is the wrong-operator
    /// subject, and rewriting the key in the leaf script draws the
    /// program-generic commitment verdict rather than the row's fault.
    NoSupersededKeyExists,
    /// The successor control recipe appears in no announcement position,
    /// so there is nothing to write it over.
    NoOfferablePosition,
    /// Successor metadata is derived and never witnessed, so exchanging
    /// the two metadata witnesses changes a position the candidate does
    /// not carry.
    SuccessorMetadataIsNeverWitnessed,
    /// The deployments share one semantic census, so another object's
    /// semantic reconstructs byte-equal to the control and stages nothing.
    DeploymentsShareOneSemanticCensus,
    /// Singleton conservation precludes a second state-asset coin, so a
    /// second state successor has no coin to fund it.
    SingletonConservationPrecludesASecondCoin,
}

/// Why the covenant does not decide an acceptance-expected row's
/// coordinate on this candidate.
///
/// The subject is a formable departure the covenant reads nothing of, so
/// a run is expected to accept it; the member records which coordinate the
/// covenant leaves undecided, for the observation step to classify.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum NativeUndecided {
    /// The covenant introspects only output position zero, so an added
    /// state output is a position it never reads.
    OutputCountUnread,
    /// Nonce leastness is a stated leaf residual, not a clause the covenant
    /// reaches on this candidate.
    NonceLeastnessResidual,
    /// Consensus admits either transaction version and policy, not the
    /// covenant, decides the version; a native run states which.
    VersionAdmittedByConsensus,
}

/// Where one native-run-required row stands on this candidate.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum NativeStanding {
    /// Staged as the named subject, with a refusal of the row's fault
    /// expected and a short note on what the run is expected to observe.
    Staged {
        /// The one-coordinate subject staged for this row.
        subject: NativeSubject,
        /// What the later run is expected to observe.
        note: &'static str,
    },
    /// Staged as the named subject, with acceptance expected because the
    /// covenant does not decide the row's coordinate.
    StagedAcceptanceExpected {
        /// The one-coordinate subject staged for this row.
        subject: NativeSubject,
        /// Which coordinate the covenant leaves undecided.
        undecided: NativeUndecided,
    },
    /// Waiting on the sponsored form (C1): the fault lives only in a
    /// sponsored envelope this sponsorless candidate does not build.
    WaitingOnSponsoredForm,
    /// Unstageable on this candidate for the typed reason.
    Unstageable(NativeUnstageable),
}

impl NativeStanding {
    /// The subject staged for this standing, if it stages one.
    #[must_use]
    pub const fn subject(&self) -> Option<NativeSubject> {
        match self {
            Self::Staged { subject, .. } | Self::StagedAcceptanceExpected { subject, .. } => {
                Some(*subject)
            }
            Self::WaitingOnSponsoredForm | Self::Unstageable(_) => None,
        }
    }
}

/// One native-run-required row: its table, its name, the boundary it
/// declares, and where it stands on this candidate.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NativeRow {
    section: MaturitySafetySection,
    name: &'static str,
    boundary: NativeBoundary,
    standing: NativeStanding,
}

impl NativeRow {
    /// The §16 table this row is drawn from.
    #[must_use]
    pub const fn section(&self) -> MaturitySafetySection {
        self.section
    }

    /// The row's name within its table.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// The §1.5 boundary this row declares on this candidate.
    #[must_use]
    pub const fn boundary(&self) -> NativeBoundary {
        self.boundary
    }

    /// Where this row stands on this candidate.
    #[must_use]
    pub const fn standing(&self) -> NativeStanding {
        self.standing
    }
}

const fn staged(
    section: MaturitySafetySection,
    name: &'static str,
    boundary: NativeBoundary,
    subject: NativeSubject,
    note: &'static str,
) -> NativeRow {
    NativeRow {
        section,
        name,
        boundary,
        standing: NativeStanding::Staged { subject, note },
    }
}

const fn accepted(
    section: MaturitySafetySection,
    name: &'static str,
    boundary: NativeBoundary,
    subject: NativeSubject,
    undecided: NativeUndecided,
) -> NativeRow {
    NativeRow {
        section,
        name,
        boundary,
        standing: NativeStanding::StagedAcceptanceExpected { subject, undecided },
    }
}

const fn waiting(
    section: MaturitySafetySection,
    name: &'static str,
    boundary: NativeBoundary,
) -> NativeRow {
    NativeRow {
        section,
        name,
        boundary,
        standing: NativeStanding::WaitingOnSponsoredForm,
    }
}

const fn unstageable(
    section: MaturitySafetySection,
    name: &'static str,
    boundary: NativeBoundary,
    reason: NativeUnstageable,
) -> NativeRow {
    NativeRow {
        section,
        name,
        boundary,
        standing: NativeStanding::Unstageable(reason),
    }
}

use MaturitySafetySection as S;
use NativeBoundary::ConsensusRejectionBeforeScript as Consensus;
use NativeBoundary::ScriptPathRejection as ScriptPath;
use NativeSubject as N;
use NativeUndecided as D;
use NativeUnstageable as U;

/// A generic verdict is expected: the commitment rule answers before the
/// covenant's own clause, alike for every candidate that changes these
/// bytes.
const GENERIC: &str = "staged; generic verdict expected";
/// A specific script-path refusal of the row's own fault is expected.
const SPECIFIC: &str = "staged; script-path refusal of the row's fault";

/// Every historically native-run-required §16 row and its staging standing.
///
/// The set of keys here is exactly [`crate::maturity_negative_half::NATIVE_STAGING_HISTORY`]
/// with thirty-five retained keys. Observation discharges or types each key;
/// the current waiting register is checked independently.
pub const NATIVE_ROWS: &[NativeRow] = &[
    // §16.2 — the two window faults.
    staged(
        S::WindowFault,
        "successor-remains-unannounced",
        ScriptPath,
        N::SuccessorUnannounced,
        SPECIFIC,
    ),
    staged(
        S::WindowFault,
        "successor-becomes-complete",
        ScriptPath,
        N::SuccessorComplete,
        SPECIFIC,
    ),
    // §16.3 — the metadata fault.
    staged(
        S::MetadataFault,
        "metadata-from-another-state-object",
        ScriptPath,
        N::MetadataFromAnotherStateObject,
        SPECIFIC,
    ),
    // §16.4 — the operator faults.
    staged(
        S::OperatorFault,
        "empty-signature",
        ScriptPath,
        N::EmptySignature,
        SPECIFIC,
    ),
    staged(
        S::OperatorFault,
        "malformed-signature",
        ScriptPath,
        N::MalformedSignature,
        SPECIFIC,
    ),
    staged(
        S::OperatorFault,
        "wrong-operator",
        ScriptPath,
        N::WrongOperator,
        SPECIFIC,
    ),
    unstageable(
        S::OperatorFault,
        "stale-operator",
        ScriptPath,
        U::NoSupersededKeyExists,
    ),
    staged(
        S::OperatorFault,
        "valid-signature-under-another-key",
        ScriptPath,
        N::WrongOperator,
        SPECIFIC,
    ),
    staged(
        S::OperatorFault,
        "valid-signature-over-another-candidate",
        ScriptPath,
        N::ValidSignatureOverAnotherCandidate,
        SPECIFIC,
    ),
    staged(
        S::OperatorFault,
        "successor-metadata-changed-after-signing",
        ScriptPath,
        N::SuccessorMetadataChangedAfterSigning,
        SPECIFIC,
    ),
    staged(
        S::OperatorFault,
        "successor-program-changed-after-signing",
        ScriptPath,
        N::SuccessorProgramChangedAfterSigning,
        SPECIFIC,
    ),
    waiting(
        S::OperatorFault,
        "sponsor-input-added-after-signing",
        ScriptPath,
    ),
    waiting(
        S::OperatorFault,
        "fee-output-changed-after-signing",
        ScriptPath,
    ),
    // §16.5 — the predecessor-constructor faults.
    staged(
        S::PredecessorConstructorFault,
        "wrong-state-asset",
        ScriptPath,
        N::WrongStateAsset,
        SPECIFIC,
    ),
    staged(
        S::PredecessorConstructorFault,
        "wrong-singleton-amount",
        ScriptPath,
        N::WrongSingletonAmount,
        SPECIFIC,
    ),
    staged(
        S::PredecessorConstructorFault,
        "wrong-predecessor-program",
        ScriptPath,
        N::WrongPredecessorProgram,
        SPECIFIC,
    ),
    staged(
        S::PredecessorConstructorFault,
        "predecessor-metadata-reconstructs-another-program",
        ScriptPath,
        N::PredecessorMetadataReconstructsAnotherProgram,
        SPECIFIC,
    ),
    staged(
        S::PredecessorConstructorFault,
        "wrong-leaf-version",
        ScriptPath,
        N::LeafVersionControlByte,
        GENERIC,
    ),
    // §16.6 — the successor-constructor faults.
    unstageable(
        S::SuccessorConstructorFault,
        "successor-from-wrong-semantic-metadata",
        ScriptPath,
        U::DeploymentsShareOneSemanticCensus,
    ),
    staged(
        S::SuccessorConstructorFault,
        "successor-from-predecessor-metadata-unchanged",
        ScriptPath,
        N::SuccessorUnannounced,
        SPECIFIC,
    ),
    staged(
        S::SuccessorConstructorFault,
        "wrong-announcement-cycle",
        ScriptPath,
        N::WrongAnnouncementCycle,
        SPECIFIC,
    ),
    staged(
        S::SuccessorConstructorFault,
        "wrong-representation-nonce",
        ScriptPath,
        N::WrongRepresentationNonce,
        SPECIFIC,
    ),
    accepted(
        S::SuccessorConstructorFault,
        "later-admissible-nonce-instead-of-first",
        ScriptPath,
        N::LaterAdmissibleNonce,
        D::NonceLeastnessResidual,
    ),
    staged(
        S::SuccessorConstructorFault,
        "wrong-leaf-version",
        ScriptPath,
        N::SuccessorLeafVersion,
        SPECIFIC,
    ),
    unstageable(
        S::SuccessorConstructorFault,
        "wrong-control-recipe",
        ScriptPath,
        U::NoOfferablePosition,
    ),
    staged(
        S::SuccessorConstructorFault,
        "arbitrary-caller-supplied-output-program",
        ScriptPath,
        N::ArbitraryCallerSuppliedOutputProgram,
        SPECIFIC,
    ),
    staged(
        S::SuccessorConstructorFault,
        "no-state-successor",
        ScriptPath,
        N::NoStateSuccessor,
        SPECIFIC,
    ),
    unstageable(
        S::SuccessorConstructorFault,
        "two-state-successors",
        ScriptPath,
        U::SingletonConservationPrecludesASecondCoin,
    ),
    accepted(
        S::SuccessorConstructorFault,
        "extra-state-like-output",
        ScriptPath,
        N::ExtraStateLikeOutput,
        D::OutputCountUnread,
    ),
    // §16.11 — the ABI and linker faults.
    accepted(
        S::AbiLinkerFault,
        "wrong-transaction-version",
        Consensus,
        N::WrongTransactionVersion,
        D::VersionAdmittedByConsensus,
    ),
    staged(
        S::AbiLinkerFault,
        "wrong-sequence",
        Consensus,
        N::WrongSequence,
        SPECIFIC,
    ),
    staged(
        S::AbiLinkerFault,
        "witness-item-reorder",
        ScriptPath,
        N::WitnessItemReorder,
        SPECIFIC,
    ),
    unstageable(
        S::AbiLinkerFault,
        "predecessor-and-successor-metadata-witnesses-exchanged",
        ScriptPath,
        U::SuccessorMetadataIsNeverWitnessed,
    ),
    staged(
        S::AbiLinkerFault,
        "control-block-from-another-program",
        ScriptPath,
        N::ControlBlockFromAnotherProgram,
        GENERIC,
    ),
    staged(
        S::AbiLinkerFault,
        "target-bytes-changed-after-abi-validation",
        ScriptPath,
        N::TargetBytesChangedAfterAbiValidation,
        SPECIFIC,
    ),
];

/// One distinct staged subject.
///
/// Twenty-six members over twenty-eight staged keys: two pairs share a
/// subject because the two keys change the same bytes. Each member names a
/// digest constant, pinned by the test below.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum NativeSubject {
    /// Output 0 carries the predecessor semantic at its first admissible
    /// nonce; the successor stays unannounced. Shared with the §16.6
    /// metadata-unchanged key.
    SuccessorUnannounced,
    /// Output 0 carries the same semantic with the completed status.
    SuccessorComplete,
    /// Witness item 4 carries another state object's metadata record.
    MetadataFromAnotherStateObject,
    /// Witness item 6 is empty.
    EmptySignature,
    /// Witness item 6 is the honest signature truncated by one byte.
    MalformedSignature,
    /// Witness item 6 is the first published signer over the honest
    /// message. Shared with the valid-signature-under-another-key key.
    WrongOperator,
    /// Witness item 6 is an operator signature over a lock-time-one
    /// candidate.
    ValidSignatureOverAnotherCandidate,
    /// Witness items 0 through 2 and output 0 come from the latest-lead
    /// announcement; the honest signature is kept.
    SuccessorMetadataChangedAfterSigning,
    /// Output 0 is the successor at its next admissible nonce; the honest
    /// signature is kept.
    SuccessorProgramChangedAfterSigning,
    /// A coin of another asset is spent and output 0's asset follows.
    WrongStateAsset,
    /// A coin one unit over the singleton is spent and output 0's value
    /// follows.
    WrongSingletonAmount,
    /// A coin at the predecessor program at a later admissible nonce is
    /// spent and witness item 8 is re-committed under it.
    WrongPredecessorProgram,
    /// Witness item 4 carries the predecessor record at a later admissible
    /// nonce.
    PredecessorMetadataReconstructsAnotherProgram,
    /// Output 0 announces the next cycle; witness item 2 stays honest.
    WrongAnnouncementCycle,
    /// Witness item 1 is the honest nonce plus one.
    WrongRepresentationNonce,
    /// Output 0 is committed under another leaf version.
    SuccessorLeafVersion,
    /// Output 0 is a version-one program of the first signer's key.
    ArbitraryCallerSuppliedOutputProgram,
    /// Output 0 is a null-data burn of the singleton.
    NoStateSuccessor,
    /// The input sequence is a non-final BIP-68 value.
    WrongSequence,
    /// Witness items 1 and 2 are exchanged.
    WitnessItemReorder,
    /// The lock time is set to one, inert under final sequences.
    TargetBytesChangedAfterAbiValidation,
    /// Witness item 8 is a control block from another program.
    ControlBlockFromAnotherProgram,
    /// The leaf-version byte of witness item 8 is changed.
    LeafVersionControlByte,
    /// Output 0 is the successor at its next admissible nonce, re-signed;
    /// leastness is not a covenant clause.
    LaterAdmissibleNonce,
    /// The transaction version is the other admitted value, re-signed.
    WrongTransactionVersion,
    /// A stand-in key-path coin of another asset funds an extra
    /// state-like output of that asset.
    ExtraStateLikeOutput,
}

impl NativeSubject {
    /// Every distinct subject, in the matrix's order.
    pub const ALL: [Self; 26] = [
        Self::SuccessorUnannounced,
        Self::SuccessorComplete,
        Self::MetadataFromAnotherStateObject,
        Self::EmptySignature,
        Self::MalformedSignature,
        Self::WrongOperator,
        Self::ValidSignatureOverAnotherCandidate,
        Self::SuccessorMetadataChangedAfterSigning,
        Self::SuccessorProgramChangedAfterSigning,
        Self::WrongStateAsset,
        Self::WrongSingletonAmount,
        Self::WrongPredecessorProgram,
        Self::PredecessorMetadataReconstructsAnotherProgram,
        Self::WrongAnnouncementCycle,
        Self::WrongRepresentationNonce,
        Self::SuccessorLeafVersion,
        Self::ArbitraryCallerSuppliedOutputProgram,
        Self::NoStateSuccessor,
        Self::WrongSequence,
        Self::WitnessItemReorder,
        Self::TargetBytesChangedAfterAbiValidation,
        Self::ControlBlockFromAnotherProgram,
        Self::LeafVersionControlByte,
        Self::LaterAdmissibleNonce,
        Self::WrongTransactionVersion,
        Self::ExtraStateLikeOutput,
    ];

    /// The pinned digest of this subject's spend-time serialization, as
    /// lowercase hexadecimal.
    #[must_use]
    pub const fn pin(self) -> &'static str {
        match self {
            Self::SuccessorUnannounced => PIN_UNANNOUNCED,
            Self::SuccessorComplete => PIN_COMPLETE,
            Self::MetadataFromAnotherStateObject => PIN_META_OTHER,
            Self::EmptySignature => PIN_EMPTY_SIG,
            Self::MalformedSignature => PIN_MALFORMED_SIG,
            Self::WrongOperator => PIN_WRONG_OP,
            Self::ValidSignatureOverAnotherCandidate => PIN_SIG_OTHER_CAND,
            Self::SuccessorMetadataChangedAfterSigning => PIN_META_AFTER_SIG,
            Self::SuccessorProgramChangedAfterSigning => PIN_PROG_AFTER_SIG,
            Self::WrongStateAsset => PIN_WRONG_ASSET,
            Self::WrongSingletonAmount => PIN_WRONG_AMOUNT,
            Self::WrongPredecessorProgram => PIN_WRONG_PRED,
            Self::PredecessorMetadataReconstructsAnotherProgram => PIN_META_RECON,
            Self::WrongAnnouncementCycle => PIN_WRONG_CYCLE,
            Self::WrongRepresentationNonce => PIN_WRONG_NONCE,
            Self::SuccessorLeafVersion => PIN_LEAF_V_166,
            Self::ArbitraryCallerSuppliedOutputProgram => PIN_CALLER_PROG,
            Self::NoStateSuccessor => PIN_NO_SUCCESSOR,
            Self::WrongSequence => PIN_WRONG_SEQ,
            Self::WitnessItemReorder => PIN_REORDER,
            Self::TargetBytesChangedAfterAbiValidation => PIN_TARGET_BYTES,
            Self::ControlBlockFromAnotherProgram => PIN_CTRL_OTHER,
            Self::LeafVersionControlByte => PIN_LEAF_V_165,
            Self::LaterAdmissibleNonce => PIN_LATER_NONCE,
            Self::WrongTransactionVersion => PIN_WRONG_TX_VER,
            Self::ExtraStateLikeOutput => PIN_EXTRA_OUTPUT,
        }
    }

    #[cfg(test)]
    #[allow(clippy::too_many_lines)]
    fn expectation(self) -> Expectation {
        let witness = |allowed: &'static [usize], must: &'static [usize]| Expectation {
            witness_allowed: allowed,
            witness_must: must,
            output0: OutputField::Unchanged,
            input_count: 1,
            output_count: 1,
            version: false,
            lock_time: false,
            sequence: false,
            outpoint: false,
        };
        let successor_output = |allowed: &'static [usize]| Expectation {
            witness_allowed: allowed,
            witness_must: &[],
            output0: OutputField::Program,
            input_count: 1,
            output_count: 1,
            version: false,
            lock_time: false,
            sequence: false,
            outpoint: false,
        };
        let two_output = Expectation {
            witness_allowed: &[6],
            witness_must: &[],
            output0: OutputField::Unchanged,
            input_count: 2,
            output_count: 2,
            version: false,
            lock_time: false,
            sequence: false,
            outpoint: false,
        };
        match self {
            Self::EmptySignature
            | Self::MalformedSignature
            | Self::WrongOperator
            | Self::ValidSignatureOverAnotherCandidate => witness(&[6], &[6]),
            Self::WrongRepresentationNonce => witness(&[1], &[1]),
            Self::WitnessItemReorder => witness(&[1, 2], &[1, 2]),
            Self::MetadataFromAnotherStateObject
            | Self::PredecessorMetadataReconstructsAnotherProgram => witness(&[4], &[4]),
            Self::ControlBlockFromAnotherProgram | Self::LeafVersionControlByte => {
                witness(&[8], &[8])
            }
            Self::TargetBytesChangedAfterAbiValidation => Expectation {
                lock_time: true,
                ..witness(&[], &[])
            },
            Self::SuccessorUnannounced
            | Self::SuccessorComplete
            | Self::WrongAnnouncementCycle
            | Self::LaterAdmissibleNonce => successor_output(&[0, 1, 6]),
            Self::SuccessorMetadataChangedAfterSigning => successor_output(&[0, 1, 2]),
            Self::SuccessorProgramChangedAfterSigning => successor_output(&[0, 1]),
            Self::SuccessorLeafVersion => successor_output(&[0, 6]),
            Self::ArbitraryCallerSuppliedOutputProgram | Self::NoStateSuccessor => {
                successor_output(&[6])
            }
            Self::WrongSequence => Expectation {
                witness_allowed: &[6],
                sequence: true,
                ..witness(&[], &[])
            },
            Self::WrongTransactionVersion => Expectation {
                witness_allowed: &[6],
                version: true,
                ..witness(&[], &[])
            },
            Self::WrongStateAsset => Expectation {
                witness_allowed: &[6],
                output0: OutputField::Asset,
                outpoint: true,
                ..witness(&[], &[])
            },
            Self::WrongSingletonAmount => Expectation {
                witness_allowed: &[6],
                output0: OutputField::Value,
                outpoint: true,
                ..witness(&[], &[])
            },
            Self::WrongPredecessorProgram => Expectation {
                witness_allowed: &[6, 8],
                witness_must: &[8],
                outpoint: true,
                ..witness(&[], &[])
            },
            Self::ExtraStateLikeOutput => two_output,
        }
    }
}

/// Which field of output zero a subject moves.
#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum OutputField {
    Unchanged,
    Program,
    Asset,
    Value,
}

#[cfg(test)]
#[allow(clippy::struct_excessive_bools)]
struct Expectation {
    witness_allowed: &'static [usize],
    witness_must: &'static [usize],
    output0: OutputField,
    input_count: usize,
    output_count: usize,
    version: bool,
    lock_time: bool,
    sequence: bool,
    outpoint: bool,
}

/// The pinned spend-time digests, one per distinct subject.
///
/// Each value is the lowercase hexadecimal of the SHA-256 of the subject's
/// serialization, recomputed from the admitted variable control by the
/// staging test; a value the audit reads is bound to its subject by the
/// `PIN_` constant name on its own line.
const PIN_UNANNOUNCED: &str = "b4c5620c2da6809cb0d40065304b9297c44185d380d5c40fdfdb8d994c20eac3";
const PIN_COMPLETE: &str = "18b9dde1d23b68e933d88330937b2b8ea5e555b8365750fc042cf4f847773e71";
const PIN_META_OTHER: &str = "9b972f42e57ce71e6c24c58cbda58cacd0d9605e7c38f0c49322563122cef388";
const PIN_EMPTY_SIG: &str = "7c233504e0e2309b0ca3dde1ae950fc98e0d5591a244018de68dd8823af979c2";
const PIN_MALFORMED_SIG: &str = "3d3233a5fc6a98cf5b884ff99cf514b96820c50b76e5ce292c4b24503cb26a28";
const PIN_WRONG_OP: &str = "65132fff3142453ee5222a106f4b20d43ce44b2db127ef0ad1712c69e5a32fa9";
const PIN_SIG_OTHER_CAND: &str = "7ad7bf37e6b6520ce9390845b54b014964f4be6dbadac09523d6693ae84e3da9";
const PIN_META_AFTER_SIG: &str = "a04c1925c83fdde1a6d9a85c3dfe2df67f57ee286f0ab7b88b6c2bbb32e1b1f5";
const PIN_PROG_AFTER_SIG: &str = "2de96480542ff5c363129d4f43344ee7351fd0e7531215caa78badeceedbf290";
const PIN_WRONG_ASSET: &str = "56a443eba7ea0c0472056e25058e34fa1cd123f7ef321acd64829e025f7f24df";
const PIN_WRONG_AMOUNT: &str = "ce80e70b50a1c80422798ab2a36ac391ce3fe5ac3a472b644c38de55c0a3697f";
const PIN_WRONG_PRED: &str = "c4b2e06cace514a06d8dc84065b703e51c9e9388a53308753b10fab84341dbe2";
const PIN_META_RECON: &str = "241366c89fdfffc77992dfcba0d0a43c1dd1bf8e73019b8ce557f4b27f81f530";
const PIN_WRONG_CYCLE: &str = "cf1d639050bb590472a8983d73f8cb1002c0866a48f276e78d7be64aab126116";
const PIN_WRONG_NONCE: &str = "21ca7b80912c40f75b1f718c1cc0d2d1389346cca38fb6cc6d1ee04b1fe1fa11";
const PIN_LEAF_V_166: &str = "6c95471406faee8bf59b4bbf35e3bcd4a19a6beb923c0690fb76d8b8d9757a8e";
const PIN_CALLER_PROG: &str = "2f30384dd5c0c174a765279e65468d903fb5ddd6129d1eef9c82603177a826cb";
const PIN_NO_SUCCESSOR: &str = "43d8374e4ddf5bff007fe135ee7bd80e85b8eef96d6f2e419273766ae39ddd3f";
const PIN_WRONG_SEQ: &str = "6d3af63f92f39949c5773351adc2c9c0de6481818fe66eff07061111065cac0e";
const PIN_REORDER: &str = "16188568768a9db9eda2e8ab74fa8ebaf5fad6fba6fbf719075735f1b010dcf2";
const PIN_TARGET_BYTES: &str = "520305cea96d2193a43dfbecadd781bd9bc5693308ce508fa611f3ab4a421794";
const PIN_CTRL_OTHER: &str = "e199093b26e092a0e4da57dc67cc36239c1e9b0f97950ce310323540c7ff3c0f";
const PIN_LEAF_V_165: &str = "8ad10dd2267e1a004c46adcce3435cdbb9c83a02f82c7aa9bdf2693f2073dbf8";
const PIN_LATER_NONCE: &str = "a89446dd9631c42cd6e58657f45483d057e006b05ac3bd3286af5048b72d7184";
const PIN_WRONG_TX_VER: &str = "0c2949906902c5baa0018b78536b429a458a1d0e05919e44fe436dd43103a6fa";
const PIN_EXTRA_OUTPUT: &str = "27e529cb72f6f4de9ca02774c227af421dbacbb7f415de1de283473c7f8a8749";

/// A stage refusal for the native subjects.
///
/// The typed refusals the lower layers raise are retained as boxed values;
/// the planner's replayed-step reason is rendered to a string because its
/// type is the trait's and is not retained here, and [`Self::Corpus`] and
/// [`Self::Shape`] name a missing precondition of this module rather than
/// a lower layer's reason.
#[derive(Debug)]
pub enum NativeStageError {
    /// The shared mutant machinery refused.
    Stage(super::MaturityMutantStageRefusal),
    /// Transaction assembly refused.
    Transaction(Box<TransactionRefusal>),
    /// The honest witness did not have its declared shape.
    Continuity(Box<MaturityContinuityRefusal>),
    /// A constructor or control recipe refused.
    Constructor(Box<StateConstructorRefusal>),
    /// A maturity source or link refused.
    Closure(Box<MaturityClosureRefusal>),
    /// The published signer could not sign.
    Signing(TestSigningDefect),
    /// The latest-lead planner refused at construction.
    NativePlan(Box<MaturityNativePlanRefusal>),
    /// The latest-lead planner refused a replayed step, rendered because
    /// its type is not retained here.
    PlanStep(String),
    /// No admissible nonce was found in the checked range.
    NoAdmissibleNonce,
    /// The admitted variable control could not be rebuilt.
    Corpus,
    /// A retained position did not have its expected shape.
    Shape,
}

fn decode(bytes: &[u8]) -> Result<TargetTransaction, NativeStageError> {
    TargetTransaction::decode(bytes)
        .map_err(|refusal| NativeStageError::Transaction(Box::new(refusal)))
}

fn assemble_versioned(
    original: &TargetTransaction,
    version: u32,
    inputs: Vec<TargetInput>,
    outputs: Vec<TargetOutput>,
    stack: Vec<Vec<u8>>,
    lock_time: u32,
) -> Result<TargetTransaction, NativeStageError> {
    TargetTransaction::with_output_witnesses(
        version,
        inputs,
        outputs,
        lock_time,
        vec![InputWitness::new(stack)],
        original.output_witnesses().to_vec(),
    )
    .map_err(|refusal| NativeStageError::Transaction(Box::new(refusal)))
}

fn assemble(
    original: &TargetTransaction,
    inputs: Vec<TargetInput>,
    outputs: Vec<TargetOutput>,
    stack: Vec<Vec<u8>>,
    lock_time: u32,
) -> Result<TargetTransaction, NativeStageError> {
    assemble_versioned(
        original,
        original.version(),
        inputs,
        outputs,
        stack,
        lock_time,
    )
}

fn census_of(funded: &MaturityFundedPredecessor) -> SpentOutputCensusEntry {
    SpentOutputCensusEntry::new(
        AssetField::Explicit(funded.asset),
        ValueField::Explicit(funded.amount),
        funded.program.clone(),
    )
}

fn second_bundle() -> Result<CandidateLinkedMaturityBundle, NativeStageError> {
    maturity_sources(
        MaturityDeployment::Second,
        MaturityWitnessSelection::Retained(StateWitnessSchedule::VariableMetadata),
    )
    .and_then(|sources| sources.link(&OracleStateCurve))
    .map_err(|refusal| NativeStageError::Closure(Box::new(refusal)))
}

fn project(
    context: &MaturityMutantContext<'_>,
    semantic: StateMetadata,
    side: TransactionSide,
    from: u32,
) -> Result<MaturityConstructorProjection, NativeStageError> {
    for value in from..4096 {
        let encoded = EncodedStateMetadata {
            semantic,
            representation: StateRepresentationNonce::new(value),
        };
        if let Ok(projection) = fixed_projection(context.target, encoded, context.bundle, side) {
            return Ok(projection);
        }
    }
    Err(NativeStageError::NoAdmissibleNonce)
}

fn set_successor(
    outputs: &mut [TargetOutput],
    stack: &mut [Vec<u8>],
    projection: &MaturityConstructorProjection,
) {
    let output = &outputs[0];
    outputs[0] = TargetOutput::new(
        output.asset(),
        output.value(),
        output.nonce(),
        projection.output_program().to_vec(),
    );
    stack[0] = vec![prefix(projection.parity())];
    stack[1] = projection.nonce().get().to_be_bytes().to_vec();
}

fn lead_submission(
    context: &MaturityMutantContext<'_>,
    lead: MaturityLead,
) -> Result<Vec<u8>, NativeStageError> {
    let corpus = maturity_variable_run_of_record().map_err(|_| NativeStageError::Corpus)?;
    let mut planner = MaturityAnnouncementPlanner::at_lead(
        context.identity.clone(),
        context.branch,
        MaturityWitnessSelection::Retained(StateWitnessSchedule::VariableMetadata),
        lead,
    )
    .map_err(|refusal| NativeStageError::NativePlan(Box::new(refusal)))?;
    let mut next = planner
        .next_step(None)
        .map_err(|refusal| NativeStageError::PlanStep(format!("{refusal:?}")))?;
    for (step, response) in &corpus.exchanges()[..2] {
        next = planner
            .next_step(Some((step.case(), response)))
            .map_err(|refusal| NativeStageError::PlanStep(format!("{refusal:?}")))?;
    }
    drop(next);
    planner
        .submission_bytes()
        .map(<[u8]>::to_vec)
        .ok_or(NativeStageError::Shape)
}

fn alt_version_successor(
    context: &MaturityMutantContext<'_>,
) -> Result<(Vec<u8>, bool), NativeStageError> {
    let successor = context.finalized.construction().successor_constructor();
    let script = successor.leaf_program().encode(context.target);
    let length = u8::try_from(script.len()).map_err(|_| NativeStageError::Shape)?;
    let alternate = LeafVersion::TAPSCRIPT.get() ^ 0x02;
    let mut preimage = vec![alternate, length];
    preimage.extend_from_slice(&script);
    let leaf = tagged_hash(TAP_LEAF_TAG, &preimage);
    let root = branch_hash(leaf, *successor.static_subtree().root());
    match OracleStateCurve.output_key(context.bundle.policy().internal_key().key(), &root) {
        StateTweakOutcome::OutputKey { key, parity } => {
            Ok((super::version_one_program(&key), parity))
        }
        StateTweakOutcome::InternalKeyNotAPoint => Err(NativeStageError::Constructor(Box::new(
            StateConstructorRefusal::InternalKeyNotAPoint,
        ))),
        StateTweakOutcome::TweakAboveGroupOrder => Err(NativeStageError::Constructor(Box::new(
            StateConstructorRefusal::TweakAboveGroupOrder,
        ))),
        StateTweakOutcome::TweakedPointIsIdentity => Err(NativeStageError::Constructor(Box::new(
            StateConstructorRefusal::TweakedPointIsIdentity,
        ))),
    }
}

fn standin_outpoint(context: &MaturityMutantContext<'_>) -> Result<Outpoint, NativeStageError> {
    Outpoint::new(
        context.funded.outpoint.txid(),
        context.funded.outpoint.index() + 1,
    )
    .map_err(|_| NativeStageError::Shape)
}

fn wrong_asset_funded(
    context: &MaturityMutantContext<'_>,
) -> Result<MaturityFundedPredecessor, NativeStageError> {
    let asset = AssetId::from_internal(
        *MaturityDeployment::Second
            .parameters()
            .map_err(|refusal| NativeStageError::Closure(Box::new(refusal)))?
            .singleton(),
    );
    Ok(MaturityFundedPredecessor {
        outpoint: standin_outpoint(context)?,
        asset,
        amount: context.funded.amount,
        program: context.funded.program.clone(),
    })
}

fn wrong_amount_funded(
    context: &MaturityMutantContext<'_>,
) -> Result<MaturityFundedPredecessor, NativeStageError> {
    Ok(MaturityFundedPredecessor {
        outpoint: standin_outpoint(context)?,
        asset: context.funded.asset,
        amount: context.funded.amount.saturating_add(1),
        program: context.funded.program.clone(),
    })
}

fn wrong_predecessor_funded(
    context: &MaturityMutantContext<'_>,
) -> Result<(MaturityFundedPredecessor, Vec<u8>), NativeStageError> {
    let predecessor = context
        .bundle
        .instances()
        .first()
        .ok_or(NativeStageError::Shape)?
        .constructor();
    let from = predecessor.nonce().get().saturating_add(1);
    let projection = project(
        context,
        predecessor.encoded_metadata().semantic,
        TransactionSide::Input,
        from,
    )?;
    let control = projection
        .control_recipe()
        .control_bytes()
        .map_err(|refusal| NativeStageError::Constructor(Box::new(refusal)))?;
    let funded = MaturityFundedPredecessor {
        outpoint: standin_outpoint(context)?,
        asset: context.funded.asset,
        amount: context.funded.amount,
        program: projection.output_program().to_vec(),
    };
    Ok((funded, control))
}

fn resign_versioned(
    context: &MaturityMutantContext<'_>,
    original: &TargetTransaction,
    version: u32,
    inputs: Vec<TargetInput>,
    outputs: Vec<TargetOutput>,
    mut stack: Vec<Vec<u8>>,
    funded: &MaturityFundedPredecessor,
) -> Result<Vec<u8>, NativeStageError> {
    let control_block = stack[8].clone();
    let candidate = assemble_versioned(
        original,
        version,
        inputs.clone(),
        outputs.clone(),
        stack.clone(),
        original.lock_time(),
    )?;
    stack[6] = super::sign_mutant(context, candidate, funded, control_block)
        .map_err(NativeStageError::Stage)?;
    let candidate = assemble_versioned(
        original,
        version,
        inputs,
        outputs,
        stack,
        original.lock_time(),
    )?;
    Ok(candidate.encode())
}

fn resign(
    context: &MaturityMutantContext<'_>,
    original: &TargetTransaction,
    inputs: Vec<TargetInput>,
    outputs: Vec<TargetOutput>,
    stack: Vec<Vec<u8>>,
    funded: &MaturityFundedPredecessor,
) -> Result<Vec<u8>, NativeStageError> {
    resign_versioned(
        context,
        original,
        original.version(),
        inputs,
        outputs,
        stack,
        funded,
    )
}

fn assemble_two(
    original: &TargetTransaction,
    inputs: Vec<TargetInput>,
    outputs: Vec<TargetOutput>,
    state_stack: Vec<Vec<u8>>,
    coin_stack: Vec<Vec<u8>>,
) -> Result<TargetTransaction, NativeStageError> {
    let witnesses = vec![
        InputWitness::new(state_stack),
        InputWitness::new(coin_stack),
    ];
    let output_witnesses = vec![
        original
            .output_witnesses()
            .first()
            .ok_or(NativeStageError::Shape)?
            .clone(),
        OutputWitness::empty(),
    ];
    TargetTransaction::with_output_witnesses(
        original.version(),
        inputs,
        outputs,
        original.lock_time(),
        witnesses,
        output_witnesses,
    )
    .map_err(|refusal| NativeStageError::Transaction(Box::new(refusal)))
}

/// The stand-in key-path coin the extra-state-like-output subject spends
/// at its second input: another asset at a version-one program of a
/// published test signer, which a key path can spend.
fn key_path_coin(
    context: &MaturityMutantContext<'_>,
) -> Result<(AssetId, u64, Vec<u8>), NativeStageError> {
    let asset = AssetId::from_internal(
        *MaturityDeployment::Second
            .parameters()
            .map_err(|refusal| NativeStageError::Closure(Box::new(refusal)))?
            .singleton(),
    );
    let key = PublicTestSignerHandle::First
        .x_only_public_key()
        .map_err(NativeStageError::Signing)?;
    Ok((
        asset,
        context.funded.amount,
        super::version_one_program(&key),
    ))
}

fn key_path_coin_census(
    context: &MaturityMutantContext<'_>,
) -> Result<SpentOutputCensusEntry, NativeStageError> {
    let (asset, amount, program) = key_path_coin(context)?;
    Ok(SpentOutputCensusEntry::new(
        AssetField::Explicit(asset),
        ValueField::Explicit(amount),
        program,
    ))
}

fn stage_extra_output(
    context: &MaturityMutantContext<'_>,
    original: &TargetTransaction,
    live: Option<&MaturityFundedPredecessor>,
) -> Result<Vec<u8>, NativeStageError> {
    let state_stack = witness(original, context.bundle.record().schedule())
        .map_err(|refusal| NativeStageError::Continuity(Box::new(refusal)))?
        .to_vec();
    let honest_input = original
        .inputs()
        .first()
        .ok_or(NativeStageError::Shape)?
        .clone();
    let standin = match live {
        Some(coin) => coin.outpoint,
        None => standin_outpoint(context)?,
    };
    let inputs = vec![
        honest_input.clone(),
        TargetInput::new(standin, honest_input.sequence()),
    ];
    let output = original
        .outputs()
        .first()
        .ok_or(NativeStageError::Shape)?
        .clone();
    let (coin_asset, coin_amount, _coin_program) = match live {
        Some(coin) => (coin.asset, coin.amount, coin.program.clone()),
        None => key_path_coin(context)?,
    };
    let extra = TargetOutput::new(
        AssetField::Explicit(coin_asset),
        ValueField::Explicit(coin_amount),
        output.nonce(),
        output.program().to_vec(),
    );
    let outputs = vec![output, extra];
    let extra_census = match live {
        Some(coin) => census_of(coin),
        None => key_path_coin_census(context)?,
    };
    let census = vec![census_of(context.funded), extra_census];
    let control_block = state_stack[8].clone();
    let candidate = assemble_two(
        original,
        inputs.clone(),
        outputs.clone(),
        state_stack.clone(),
        Vec::new(),
    )?;
    let (operator_message, key_path) =
        super::operator_and_key_path_messages(context, candidate, census, control_block, 1)
            .map_err(NativeStageError::Stage)?;
    let operator_signature = OPERATOR_HANDLE
        .material()
        .map_err(NativeStageError::Signing)?
        .sign(&operator_message, &[0; 32])
        .map_err(NativeStageError::Signing)?;
    let key_path_signature = PublicTestSignerHandle::First
        .material()
        .map_err(NativeStageError::Signing)?
        .sign(&key_path, &[0; 32])
        .map_err(NativeStageError::Signing)?;
    let mut signed = state_stack;
    signed[6] = operator_signature.to_vec();
    let candidate = assemble_two(
        original,
        inputs,
        outputs,
        signed,
        vec![key_path_signature.to_vec()],
    )?;
    Ok(candidate.encode())
}

fn first_signer_over(
    context: &MaturityMutantContext<'_>,
    candidate: TargetTransaction,
    control_block: Vec<u8>,
) -> Result<Vec<u8>, NativeStageError> {
    let message = super::operator_message(
        context,
        candidate,
        vec![census_of(context.funded)],
        0,
        control_block,
    )
    .map_err(NativeStageError::Stage)?;
    let signature = PublicTestSignerHandle::First
        .material()
        .map_err(NativeStageError::Signing)?
        .sign(&message, &[0; 32])
        .map_err(NativeStageError::Signing)?;
    Ok(signature.to_vec())
}

fn operator_over(
    context: &MaturityMutantContext<'_>,
    candidate: TargetTransaction,
    control_block: Vec<u8>,
) -> Result<Vec<u8>, NativeStageError> {
    let message = super::operator_message(
        context,
        candidate,
        vec![census_of(context.funded)],
        0,
        control_block,
    )
    .map_err(NativeStageError::Stage)?;
    let signature = OPERATOR_HANDLE
        .material()
        .map_err(NativeStageError::Signing)?
        .sign(&message, &[0; 32])
        .map_err(NativeStageError::Signing)?;
    Ok(signature.to_vec())
}

/// Stage one native subject as an exact one-coordinate departure from the
/// accepted variable-schedule control.
///
/// # Errors
///
/// Retains the source, constructor, transaction and signing refusals each
/// derivation can produce, and refuses when no admissible nonce is found
/// or a retained position lacks its shape.
pub fn stage_native_subject(
    subject: NativeSubject,
    context: &MaturityMutantContext<'_>,
) -> Result<Vec<u8>, NativeStageError> {
    stage_native_subject_inner(subject, context, None, false)
}

/// Replay the same coordinate over the supplied funded coin and live context.
/// # Errors
/// Preserves construction and signing refusals without substituting another subject.
pub(crate) fn stage_live_subject(
    subject: NativeSubject,
    context: &MaturityMutantContext<'_>,
    funded: Option<&MaturityFundedPredecessor>,
) -> Result<Vec<u8>, NativeStageError> {
    stage_native_subject_inner(subject, context, funded, true)
}

/// The program and amount requested for a subject's additional funding.
/// # Errors
/// Preserves constructor and signer refusals.
pub(crate) fn live_funding_shape(
    subject: NativeSubject,
    context: &MaturityMutantContext<'_>,
) -> Result<(u64, Vec<u8>), NativeStageError> {
    match subject {
        NativeSubject::WrongStateAsset => {
            Ok((context.funded.amount, context.funded.program.clone()))
        }
        NativeSubject::WrongSingletonAmount => {
            Ok((context.funded.amount + 1, context.funded.program.clone()))
        }
        NativeSubject::WrongPredecessorProgram => {
            let (coin, _) = wrong_predecessor_funded(context)?;
            Ok((coin.amount, coin.program))
        }
        NativeSubject::ExtraStateLikeOutput => {
            let (_, amount, program) = key_path_coin(context)?;
            Ok((amount, program))
        }
        _ => Err(NativeStageError::Shape),
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "Each subject retains its coordinate and the balancing edits it requires."
)]
fn stage_native_subject_inner(
    subject: NativeSubject,
    context: &MaturityMutantContext<'_>,
    live: Option<&MaturityFundedPredecessor>,
    live_context: bool,
) -> Result<Vec<u8>, NativeStageError> {
    let original = decode(context.submitted_bytes)?;
    let mut stack = witness(&original, context.bundle.record().schedule())
        .map_err(|refusal| NativeStageError::Continuity(Box::new(refusal)))?
        .to_vec();
    let mut inputs = original.inputs().to_vec();
    let mut outputs = original.outputs().to_vec();
    let lock = original.lock_time();
    match subject {
        NativeSubject::EmptySignature => {
            stack[6] = Vec::new();
            Ok(assemble(&original, inputs, outputs, stack, lock)?.encode())
        }
        NativeSubject::MalformedSignature => {
            stack[6].truncate(63);
            Ok(assemble(&original, inputs, outputs, stack, lock)?.encode())
        }
        NativeSubject::WrongRepresentationNonce => {
            let bytes: [u8; 4] = stack[1]
                .clone()
                .try_into()
                .map_err(|_| NativeStageError::Shape)?;
            stack[1] = u32::from_be_bytes(bytes)
                .wrapping_add(1)
                .to_be_bytes()
                .to_vec();
            Ok(assemble(&original, inputs, outputs, stack, lock)?.encode())
        }
        NativeSubject::WitnessItemReorder => {
            stack.swap(1, 2);
            Ok(assemble(&original, inputs, outputs, stack, lock)?.encode())
        }
        NativeSubject::MetadataFromAnotherStateObject => {
            let other = second_bundle()?;
            let constructor = other
                .instances()
                .first()
                .ok_or(NativeStageError::Shape)?
                .constructor();
            stack[4] = constructor.metadata_bytes().to_vec();
            Ok(assemble(&original, inputs, outputs, stack, lock)?.encode())
        }
        NativeSubject::PredecessorMetadataReconstructsAnotherProgram => {
            let predecessor = context
                .bundle
                .instances()
                .first()
                .ok_or(NativeStageError::Shape)?
                .constructor();
            let from = predecessor.nonce().get().saturating_add(1);
            let projection = project(
                context,
                predecessor.encoded_metadata().semantic,
                TransactionSide::Input,
                from,
            )?;
            stack[4] = projection.metadata_bytes().to_vec();
            Ok(assemble(&original, inputs, outputs, stack, lock)?.encode())
        }
        NativeSubject::WrongOperator => {
            let candidate = assemble(
                &original,
                inputs.clone(),
                outputs.clone(),
                stack.clone(),
                lock,
            )?;
            stack[6] = first_signer_over(context, candidate, stack[8].clone())?;
            Ok(assemble(&original, inputs, outputs, stack, lock)?.encode())
        }
        NativeSubject::ValidSignatureOverAnotherCandidate => {
            let candidate = assemble(&original, inputs.clone(), outputs.clone(), stack.clone(), 1)?;
            stack[6] = operator_over(context, candidate, stack[8].clone())?;
            Ok(assemble(&original, inputs, outputs, stack, lock)?.encode())
        }
        NativeSubject::TargetBytesChangedAfterAbiValidation => {
            Ok(assemble(&original, inputs, outputs, stack, 1)?.encode())
        }
        NativeSubject::SuccessorUnannounced => {
            let predecessor = context
                .bundle
                .instances()
                .first()
                .ok_or(NativeStageError::Shape)?
                .constructor();
            let semantic = predecessor
                .encoded_metadata()
                .semantic
                .with_maturity(Maturity::Unannounced);
            let projection = project(context, semantic, TransactionSide::Output, 0)?;
            set_successor(&mut outputs, &mut stack, &projection);
            resign(context, &original, inputs, outputs, stack, context.funded)
        }
        NativeSubject::SuccessorComplete => {
            let predecessor = context
                .bundle
                .instances()
                .first()
                .ok_or(NativeStageError::Shape)?
                .constructor();
            let semantic = predecessor
                .encoded_metadata()
                .semantic
                .with_maturity(Maturity::Complete);
            let projection = project(context, semantic, TransactionSide::Output, 0)?;
            set_successor(&mut outputs, &mut stack, &projection);
            resign(context, &original, inputs, outputs, stack, context.funded)
        }
        NativeSubject::SuccessorProgramChangedAfterSigning => {
            let successor = context.finalized.construction().successor_constructor();
            let from = successor.nonce().get().saturating_add(1);
            let projection = project(
                context,
                successor.encoded_metadata().semantic,
                TransactionSide::Output,
                from,
            )?;
            set_successor(&mut outputs, &mut stack, &projection);
            Ok(assemble(&original, inputs, outputs, stack, lock)?.encode())
        }
        NativeSubject::LaterAdmissibleNonce => {
            let successor = context.finalized.construction().successor_constructor();
            let from = successor.nonce().get().saturating_add(1);
            let projection = project(
                context,
                successor.encoded_metadata().semantic,
                TransactionSide::Output,
                from,
            )?;
            set_successor(&mut outputs, &mut stack, &projection);
            resign(context, &original, inputs, outputs, stack, context.funded)
        }
        NativeSubject::WrongAnnouncementCycle => {
            let successor = context.finalized.construction().successor_constructor();
            let semantic = successor.encoded_metadata().semantic;
            let bumped = match semantic.maturity {
                Maturity::Announced { cycle } => semantic.with_maturity(Maturity::Announced {
                    cycle: Cycle::new(cycle.get().saturating_add(1)),
                }),
                _ => semantic.with_maturity(Maturity::Announced {
                    cycle: Cycle::new(1),
                }),
            };
            let projection = project(context, bumped, TransactionSide::Output, 0)?;
            set_successor(&mut outputs, &mut stack, &projection);
            resign(context, &original, inputs, outputs, stack, context.funded)
        }
        NativeSubject::SuccessorMetadataChangedAfterSigning => {
            let latest = if live_context {
                let finalized = crate::maturity_native::build_announcement(
                    context.bundle,
                    context.funded.asset,
                    context.funded.outpoint,
                    context.branch,
                    context.funded.amount,
                    MaturityLead::Maximum,
                )
                .map_err(|error| NativeStageError::NativePlan(Box::new(error)))?;
                crate::maturity_native::sign_announcement(&finalized)
                    .map_err(|error| NativeStageError::NativePlan(Box::new(error)))?
            } else {
                lead_submission(context, MaturityLead::Maximum)?
            };
            let latest = decode(&latest)?;
            let latest_stack = witness(&latest, context.bundle.record().schedule())
                .map_err(|refusal| NativeStageError::Continuity(Box::new(refusal)))?;
            stack[0].clone_from(&latest_stack[0]);
            stack[1].clone_from(&latest_stack[1]);
            stack[2].clone_from(&latest_stack[2]);
            outputs[0] = latest.outputs()[0].clone();
            Ok(assemble(&original, inputs, outputs, stack, lock)?.encode())
        }
        NativeSubject::SuccessorLeafVersion => {
            let (program, parity) = alt_version_successor(context)?;
            let output = &outputs[0];
            outputs[0] = TargetOutput::new(output.asset(), output.value(), output.nonce(), program);
            stack[0] = vec![prefix(parity)];
            resign(context, &original, inputs, outputs, stack, context.funded)
        }
        NativeSubject::ArbitraryCallerSuppliedOutputProgram => {
            let key = PublicTestSignerHandle::First
                .x_only_public_key()
                .map_err(NativeStageError::Signing)?;
            let output = &outputs[0];
            outputs[0] = TargetOutput::new(
                output.asset(),
                output.value(),
                output.nonce(),
                super::version_one_program(&key),
            );
            resign(context, &original, inputs, outputs, stack, context.funded)
        }
        NativeSubject::NoStateSuccessor => {
            let output = &outputs[0];
            outputs[0] =
                TargetOutput::new(output.asset(), output.value(), output.nonce(), vec![0x6a]);
            resign(context, &original, inputs, outputs, stack, context.funded)
        }
        NativeSubject::WrongSequence => {
            inputs[0] = TargetInput::new(inputs[0].outpoint(), 0x0000_ffff);
            resign(context, &original, inputs, outputs, stack, context.funded)
        }
        NativeSubject::WrongTransactionVersion => {
            let other = if original.version() == 3 { 2 } else { 3 };
            resign_versioned(
                context,
                &original,
                other,
                inputs,
                outputs,
                stack,
                context.funded,
            )
        }
        NativeSubject::WrongStateAsset => {
            let funded = match live {
                Some(coin) => coin.clone(),
                None => wrong_asset_funded(context)?,
            };
            inputs[0] = TargetInput::new(funded.outpoint, inputs[0].sequence());
            let output = &outputs[0];
            outputs[0] = TargetOutput::new(
                AssetField::Explicit(funded.asset),
                output.value(),
                output.nonce(),
                output.program().to_vec(),
            );
            resign(context, &original, inputs, outputs, stack, &funded)
        }
        NativeSubject::WrongSingletonAmount => {
            let funded = match live {
                Some(coin) => coin.clone(),
                None => wrong_amount_funded(context)?,
            };
            inputs[0] = TargetInput::new(funded.outpoint, inputs[0].sequence());
            let output = &outputs[0];
            outputs[0] = TargetOutput::new(
                output.asset(),
                ValueField::Explicit(funded.amount),
                output.nonce(),
                output.program().to_vec(),
            );
            resign(context, &original, inputs, outputs, stack, &funded)
        }
        NativeSubject::WrongPredecessorProgram => {
            let (funded, control) = wrong_predecessor_funded(context)?;
            let funded = live.unwrap_or(&funded);
            stack[8] = control;
            inputs[0] = TargetInput::new(funded.outpoint, inputs[0].sequence());
            resign(context, &original, inputs, outputs, stack, funded)
        }
        NativeSubject::ControlBlockFromAnotherProgram => {
            let other = second_bundle()?;
            let constructor = other
                .instances()
                .first()
                .ok_or(NativeStageError::Shape)?
                .constructor();
            stack[8] = constructor
                .control_recipe(StateLeafRole::Announcement)
                .map_err(|refusal| NativeStageError::Constructor(Box::new(refusal)))?
                .control_bytes()
                .map_err(|refusal| NativeStageError::Constructor(Box::new(refusal)))?;
            Ok(assemble(&original, inputs, outputs, stack, lock)?.encode())
        }
        NativeSubject::LeafVersionControlByte => {
            let block = stack.get_mut(8).ok_or(NativeStageError::Shape)?;
            let byte = block.first_mut().ok_or(NativeStageError::Shape)?;
            *byte ^= 0x02;
            Ok(assemble(&original, inputs, outputs, stack, lock)?.encode())
        }
        NativeSubject::ExtraStateLikeOutput => stage_extra_output(context, &original, live),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::matrix::EvidenceBoundary;
    use crate::maturity_continuity::tests::variable_archived;
    use crate::maturity_evidence::{
        MaturityConstructorMaterial, MaturityConstructorMaterialAbsence,
        MaturityExecutorProvenanceExpectation, derive_maturity_evidence_plan_with,
    };
    use crate::maturity_first_party::{
        MaturitySupplementaryDisposition, maturity_first_party_dispositions,
    };
    use crate::maturity_mutants::MaturityMutantRow;
    use crate::maturity_negative_half::NATIVE_STAGING_HISTORY;
    use crate::maturity_operator::OperatorVerifier;
    use crate::maturity_safety::row_count;
    use std::collections::{BTreeMap, BTreeSet};
    use std::fmt::Write as _;
    use target_elements::ReviewedElementsTapscriptDefinition;
    use target_elements_conformance::constructor::tagged::sha256;
    use transaction::operator_signing::ScriptPathSignatureVerifier;

    struct Fixture {
        target: ReviewedElementsTapscriptDefinition,
        planner: MaturityAnnouncementPlanner,
        foreign: MaturityFundedPredecessor,
        source: &'static crate::maturity_continuity::tests::Source,
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
            let rebuilt = planner
                .submission_bytes()
                .expect("rebuilt signed submission");
            assert!(
                rebuilt == input.submitted_bytes,
                "the rebuilt submission equals the archived control",
            );
            assert_eq!(planner.bundle(), input.bundle);
            let mut foreign = input.funded.clone();
            foreign.outpoint = Outpoint::new(
                input.funded.outpoint.txid(),
                input.funded.outpoint.index() + 1,
            )
            .expect("distinct foreign coin outpoint");
            foreign.program = super::super::foreign_key_predecessor_program(input.bundle)
                .expect("public foreign predecessor program");
            Self {
                target: crate::maturity_closure::closure_target().expect("reviewed target"),
                planner,
                foreign,
                source,
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
    }

    fn digest_hex(bytes: &[u8]) -> String {
        let digest = sha256(bytes);
        let mut rendered = String::with_capacity(64);
        for byte in digest {
            let _ = write!(rendered, "{byte:02x}");
        }
        rendered
    }

    fn render_undecided(undecided: NativeUndecided) -> &'static str {
        match undecided {
            NativeUndecided::OutputCountUnread => "the covenant reads no output count",
            NativeUndecided::NonceLeastnessResidual => "nonce leastness is a stated leaf residual",
            NativeUndecided::VersionAdmittedByConsensus => {
                "consensus admits either version and the covenant reads none"
            }
        }
    }

    fn render_unstageable(reason: NativeUnstageable) -> &'static str {
        match reason {
            NativeUnstageable::NoSupersededKeyExists => {
                "no superseded key exists; any other key is the wrong-operator subject"
            }
            NativeUnstageable::NoOfferablePosition => "no announcement position offers the recipe",
            NativeUnstageable::SuccessorMetadataIsNeverWitnessed => {
                "successor metadata is derived, never witnessed"
            }
            NativeUnstageable::DeploymentsShareOneSemanticCensus => {
                "the deployments share one semantic census"
            }
            NativeUnstageable::SingletonConservationPrecludesASecondCoin => {
                "singleton conservation precludes a second state-asset coin"
            }
        }
    }

    /// The Standing cell the record carries for one typed standing.
    ///
    /// A staged row's note is rendered, not dropped: the `GENERIC` note
    /// reads `staged; generic verdict expected` and every other staged note
    /// reads `staged; refusal expected`, so the generic/specific
    /// distinction is bound to the table cell.
    fn render_standing(standing: NativeStanding) -> String {
        match standing {
            NativeStanding::Staged { note, .. } => {
                if note == GENERIC {
                    "staged; generic verdict expected".to_owned()
                } else {
                    "staged; refusal expected".to_owned()
                }
            }
            NativeStanding::StagedAcceptanceExpected { undecided, .. } => {
                format!(
                    "staged; acceptance expected: {}",
                    render_undecided(undecided)
                )
            }
            NativeStanding::WaitingOnSponsoredForm => "waiting on C1".to_owned(),
            NativeStanding::Unstageable(reason) => {
                format!("unstageable: {}", render_unstageable(reason))
            }
        }
    }

    fn render_native_boundary(boundary: NativeBoundary) -> &'static str {
        match boundary {
            NativeBoundary::ScriptPathRejection => "ScriptPathRejection",
            NativeBoundary::ConsensusRejectionBeforeScript => "ConsensusRejectionBeforeScript",
        }
    }

    /// Each `EvidenceBoundary` by its own name, with no catch-all arm, so a
    /// boundary minted upstream forces this match to be revisited rather
    /// than rendering as a script-path rejection by default.
    fn render_evidence_boundary(boundary: EvidenceBoundary) -> &'static str {
        match boundary {
            EvidenceBoundary::SemanticRequestRejection => "SemanticRequestRejection",
            EvidenceBoundary::CompilerPlanRejection => "CompilerPlanRejection",
            EvidenceBoundary::ConstructorDerivationRejection => "ConstructorDerivationRejection",
            EvidenceBoundary::BackendEmissionRejection => "BackendEmissionRejection",
            EvidenceBoundary::LinkerRejection => "LinkerRejection",
            EvidenceBoundary::AbiConstructionRejection => "AbiConstructionRejection",
            EvidenceBoundary::ExecutorInfrastructureFailure => "ExecutorInfrastructureFailure",
            EvidenceBoundary::ConsensusRejectionBeforeScript => "ConsensusRejectionBeforeScript",
            EvidenceBoundary::KeyPathRejection => "KeyPathRejection",
            EvidenceBoundary::ScriptPathRejection => "ScriptPathRejection",
            EvidenceBoundary::RelayPolicyRejection => "RelayPolicyRejection",
            EvidenceBoundary::AcceptedTransaction => "AcceptedTransaction",
            EvidenceBoundary::ReportSemanticProjectionRejection => {
                "ReportSemanticProjectionRejection"
            }
            EvidenceBoundary::RootHistoryReportRejection => "RootHistoryReportRejection",
            EvidenceBoundary::PublicRecoveryReportRejection => "PublicRecoveryReportRejection",
        }
    }

    /// The operator-re-signed subjects whose signature covers their own
    /// bytes, with the spent-output census each was signed under.
    fn operator_census(
        subject: NativeSubject,
        context: &MaturityMutantContext<'_>,
    ) -> Option<Vec<SpentOutputCensusEntry>> {
        let one = || vec![census_of(context.funded)];
        match subject {
            NativeSubject::SuccessorUnannounced
            | NativeSubject::SuccessorComplete
            | NativeSubject::WrongAnnouncementCycle
            | NativeSubject::SuccessorLeafVersion
            | NativeSubject::ArbitraryCallerSuppliedOutputProgram
            | NativeSubject::NoStateSuccessor
            | NativeSubject::WrongSequence
            | NativeSubject::LaterAdmissibleNonce
            | NativeSubject::WrongTransactionVersion => Some(one()),
            NativeSubject::WrongStateAsset => {
                Some(vec![census_of(&wrong_asset_funded(context).ok()?)])
            }
            NativeSubject::WrongSingletonAmount => {
                Some(vec![census_of(&wrong_amount_funded(context).ok()?)])
            }
            NativeSubject::WrongPredecessorProgram => {
                Some(vec![census_of(&wrong_predecessor_funded(context).ok()?.0)])
            }
            NativeSubject::ExtraStateLikeOutput => Some(vec![
                census_of(context.funded),
                key_path_coin_census(context).ok()?,
            ]),
            _ => None,
        }
    }

    /// Every distinct subject stages to its pinned digest.
    ///
    /// Collected rather than asserted one at a time so a first run prints
    /// every subject's recomputed digest at once, and a later run holds
    /// them all to their pins.
    #[test]
    fn every_staged_subject_matches_its_pin() {
        let fixture = Fixture::new();
        let context = fixture.context();
        let mut mismatches = String::new();
        for subject in NativeSubject::ALL {
            match stage_native_subject(subject, &context) {
                Ok(bytes) => {
                    let got = digest_hex(&bytes);
                    if got != subject.pin() {
                        let _ = writeln!(mismatches, "{subject:?} => {got}");
                    }
                }
                Err(error) => {
                    let _ = writeln!(mismatches, "{subject:?} ERR {error:?}");
                }
            }
        }
        assert!(mismatches.is_empty(), "\n{mismatches}");
    }

    /// Each subject moves exactly its declared coordinate and its named
    /// collateral, and nothing else.
    #[test]
    fn each_subject_moves_only_its_coordinate() {
        let fixture = Fixture::new();
        let context = fixture.context();
        let honest = TargetTransaction::decode(context.submitted_bytes).expect("honest control");
        let honest_stack = honest.witnesses()[0].stack();
        for subject in NativeSubject::ALL {
            let bytes = stage_native_subject(subject, &context).expect("staged subject");
            assert_ne!(
                bytes, context.submitted_bytes,
                "{subject:?} copied the control"
            );
            let staged = TargetTransaction::decode(&bytes).expect("staged subject decodes");
            let expectation = subject.expectation();
            assert_eq!(
                staged.inputs().len(),
                expectation.input_count,
                "{subject:?} input count",
            );
            assert_eq!(
                staged.outputs().len(),
                expectation.output_count,
                "{subject:?} output count",
            );
            assert_eq!(
                staged.version() != honest.version(),
                expectation.version,
                "{subject:?} version",
            );
            assert_eq!(
                staged.lock_time() != honest.lock_time(),
                expectation.lock_time,
                "{subject:?} lock time",
            );
            assert_eq!(
                staged.inputs()[0].outpoint() != honest.inputs()[0].outpoint(),
                expectation.outpoint,
                "{subject:?} outpoint",
            );
            assert_eq!(
                staged.inputs()[0].sequence() != honest.inputs()[0].sequence(),
                expectation.sequence,
                "{subject:?} sequence",
            );
            let staged_output = &staged.outputs()[0];
            let honest_output = &honest.outputs()[0];
            let program_moved = staged_output.program() != honest_output.program();
            let asset_moved = staged_output.asset() != honest_output.asset();
            let value_moved = staged_output.value() != honest_output.value();
            match expectation.output0 {
                OutputField::Unchanged => {
                    assert!(!program_moved, "{subject:?} output program moved");
                    assert!(!asset_moved, "{subject:?} output asset moved");
                    assert!(!value_moved, "{subject:?} output value moved");
                }
                OutputField::Program => {
                    assert!(program_moved, "{subject:?} output program unchanged");
                    assert!(!asset_moved, "{subject:?} output asset moved");
                    assert!(!value_moved, "{subject:?} output value moved");
                }
                OutputField::Asset => {
                    assert!(asset_moved, "{subject:?} output asset unchanged");
                    assert!(!program_moved, "{subject:?} output program moved");
                    assert!(!value_moved, "{subject:?} output value moved");
                }
                OutputField::Value => {
                    assert!(value_moved, "{subject:?} output value unchanged");
                    assert!(!program_moved, "{subject:?} output program moved");
                    assert!(!asset_moved, "{subject:?} output asset moved");
                }
            }
            let staged_stack = staged.witnesses()[0].stack();
            assert_eq!(
                staged_stack.len(),
                honest_stack.len(),
                "{subject:?} witness item count",
            );
            for (index, (before, after)) in honest_stack.iter().zip(staged_stack).enumerate() {
                if before != after {
                    assert!(
                        expectation.witness_allowed.contains(&index),
                        "{subject:?} moved witness item {index} outside its collateral",
                    );
                }
            }
            for index in expectation.witness_must {
                assert_ne!(
                    honest_stack[*index], staged_stack[*index],
                    "{subject:?} left its own witness coordinate {index} unchanged",
                );
            }
        }
    }

    /// The staging is deterministic.
    #[test]
    fn staging_is_deterministic() {
        let fixture = Fixture::new();
        let context = fixture.context();
        for subject in NativeSubject::ALL {
            let first = stage_native_subject(subject, &context).expect("first stage");
            let second = stage_native_subject(subject, &context).expect("second stage");
            assert_eq!(first, second, "{subject:?}");
        }
    }

    /// No two staged subjects produce the same bytes.
    #[test]
    fn staged_subjects_are_pairwise_distinct() {
        let fixture = Fixture::new();
        let context = fixture.context();
        let mut seen: BTreeSet<Vec<u8>> = BTreeSet::new();
        for subject in NativeSubject::ALL {
            let bytes = stage_native_subject(subject, &context).expect("staged subject");
            assert!(
                seen.insert(bytes),
                "{subject:?} shares bytes with another subject"
            );
        }
        assert_eq!(seen.len(), NativeSubject::ALL.len());
    }

    /// Each re-signed subject's operator signature verifies under the
    /// published operator key against its own spent-output census, and the
    /// one key-path second input verifies under the first published
    /// signer's key against the same frozen census at input index 1.
    #[test]
    fn re_signed_subjects_verify_under_the_published_operator_key() {
        let fixture = Fixture::new();
        let context = fixture.context();
        let binding = context.bundle.deployment().operator();
        let mut operator_verified = 0;
        let mut key_path_verified = 0;
        for subject in NativeSubject::ALL {
            let Some(census) = operator_census(subject, &context) else {
                continue;
            };
            let bytes = stage_native_subject(subject, &context).expect("staged subject");
            let staged = TargetTransaction::decode(&bytes).expect("staged subject decodes");
            let input0 = staged.witnesses()[0].stack().to_vec();
            let control_block = input0[8].clone();
            if subject == NativeSubject::ExtraStateLikeOutput {
                let input1 = staged.witnesses()[1].stack().to_vec();
                let (operator, key_path) = super::super::operator_and_key_path_messages(
                    &context,
                    staged,
                    census,
                    control_block,
                    1,
                )
                .expect("re-freeze the two-input subject");
                OperatorVerifier
                    .verify(binding.key().bytes(), &operator, &input0[6])
                    .expect("published key verifies input 0");
                let first_key = PublicTestSignerHandle::First
                    .x_only_public_key()
                    .expect("first signer key");
                OperatorVerifier
                    .verify(&first_key, &key_path, &input1[0])
                    .expect("first signer verifies the key-path input 1");
                operator_verified += 1;
                key_path_verified += 1;
            } else {
                let message =
                    super::super::operator_message(&context, staged, census, 0, control_block)
                        .expect("re-freeze the staged subject");
                OperatorVerifier
                    .verify(binding.key().bytes(), &message, &input0[6])
                    .expect("published key verifies the re-signed subject");
                operator_verified += 1;
            }
        }
        assert_eq!(
            operator_verified, 13,
            "every operator-re-signed subject is checked"
        );
        assert_eq!(key_path_verified, 1, "the key-path second input is checked");
    }

    /// The two shared keys each map to a single subject.
    #[test]
    fn shared_pair_keys_map_to_one_subject() {
        let subject_of = |section: MaturitySafetySection, name: &str| {
            NATIVE_ROWS
                .iter()
                .find(|row| row.section() == section && row.name() == name)
                .and_then(|row| row.standing().subject())
                .expect("staged row")
        };
        assert_eq!(
            subject_of(S::WindowFault, "successor-remains-unannounced"),
            subject_of(
                S::SuccessorConstructorFault,
                "successor-from-predecessor-metadata-unchanged",
            ),
        );
        assert_eq!(
            subject_of(S::OperatorFault, "wrong-operator"),
            subject_of(S::OperatorFault, "valid-signature-under-another-key"),
        );
    }

    /// The thirty-five native rows partition into the four real standings,
    /// with the key set of the register.
    #[test]
    fn native_staging_ledger_partitions_the_thirty_five_native_rows() {
        assert_eq!(NATIVE_ROWS.len(), 35);
        let mut refusal = 0;
        let mut acceptance = 0;
        let mut waiting = 0;
        let mut unstageable = 0;
        for row in NATIVE_ROWS {
            match row.standing() {
                NativeStanding::Staged { .. } => refusal += 1,
                NativeStanding::StagedAcceptanceExpected { .. } => acceptance += 1,
                NativeStanding::WaitingOnSponsoredForm => waiting += 1,
                NativeStanding::Unstageable(_) => unstageable += 1,
            }
        }
        assert_eq!([refusal, acceptance, waiting, unstageable], [25, 3, 2, 5]);

        let classified: BTreeSet<(MaturitySafetySection, &str)> = NATIVE_ROWS
            .iter()
            .map(|row| (row.section(), row.name()))
            .collect();
        let registered: BTreeSet<(MaturitySafetySection, &str)> = NATIVE_STAGING_HISTORY
            .iter()
            .map(|entry| (entry.section(), entry.row()))
            .collect();
        assert_eq!(classified, registered);

        let recomputed: BTreeSet<(MaturitySafetySection, &str)> =
            crate::maturity_negative_half::outstanding_rows()
                .expect("the evidence plan derives")
                .into_iter()
                .collect();
        assert!(recomputed.is_empty());
    }

    /// Every row's full standing is pinned by key against an independent
    /// table.
    ///
    /// The expected table is spelled here rather than read from
    /// `NATIVE_ROWS`, so a swap of any two rows' subjects, a changed note,
    /// a changed undecided reason or a changed unstageable reason fails
    /// here — a per-row check over fewer rows would miss a swap between two
    /// refusal-expected rows.
    #[test]
    #[allow(clippy::too_many_lines)]
    fn each_row_standing_is_pinned_by_key() {
        let staged = |subject, note| NativeStanding::Staged { subject, note };
        let accepted =
            |subject, undecided| NativeStanding::StagedAcceptanceExpected { subject, undecided };
        let expected: [(MaturitySafetySection, &str, NativeStanding); 35] = [
            (
                S::WindowFault,
                "successor-remains-unannounced",
                staged(N::SuccessorUnannounced, SPECIFIC),
            ),
            (
                S::WindowFault,
                "successor-becomes-complete",
                staged(N::SuccessorComplete, SPECIFIC),
            ),
            (
                S::MetadataFault,
                "metadata-from-another-state-object",
                staged(N::MetadataFromAnotherStateObject, SPECIFIC),
            ),
            (
                S::OperatorFault,
                "empty-signature",
                staged(N::EmptySignature, SPECIFIC),
            ),
            (
                S::OperatorFault,
                "malformed-signature",
                staged(N::MalformedSignature, SPECIFIC),
            ),
            (
                S::OperatorFault,
                "wrong-operator",
                staged(N::WrongOperator, SPECIFIC),
            ),
            (
                S::OperatorFault,
                "stale-operator",
                NativeStanding::Unstageable(U::NoSupersededKeyExists),
            ),
            (
                S::OperatorFault,
                "valid-signature-under-another-key",
                staged(N::WrongOperator, SPECIFIC),
            ),
            (
                S::OperatorFault,
                "valid-signature-over-another-candidate",
                staged(N::ValidSignatureOverAnotherCandidate, SPECIFIC),
            ),
            (
                S::OperatorFault,
                "successor-metadata-changed-after-signing",
                staged(N::SuccessorMetadataChangedAfterSigning, SPECIFIC),
            ),
            (
                S::OperatorFault,
                "successor-program-changed-after-signing",
                staged(N::SuccessorProgramChangedAfterSigning, SPECIFIC),
            ),
            (
                S::OperatorFault,
                "sponsor-input-added-after-signing",
                NativeStanding::WaitingOnSponsoredForm,
            ),
            (
                S::OperatorFault,
                "fee-output-changed-after-signing",
                NativeStanding::WaitingOnSponsoredForm,
            ),
            (
                S::PredecessorConstructorFault,
                "wrong-state-asset",
                staged(N::WrongStateAsset, SPECIFIC),
            ),
            (
                S::PredecessorConstructorFault,
                "wrong-singleton-amount",
                staged(N::WrongSingletonAmount, SPECIFIC),
            ),
            (
                S::PredecessorConstructorFault,
                "wrong-predecessor-program",
                staged(N::WrongPredecessorProgram, SPECIFIC),
            ),
            (
                S::PredecessorConstructorFault,
                "predecessor-metadata-reconstructs-another-program",
                staged(N::PredecessorMetadataReconstructsAnotherProgram, SPECIFIC),
            ),
            (
                S::PredecessorConstructorFault,
                "wrong-leaf-version",
                staged(N::LeafVersionControlByte, GENERIC),
            ),
            (
                S::SuccessorConstructorFault,
                "successor-from-wrong-semantic-metadata",
                NativeStanding::Unstageable(U::DeploymentsShareOneSemanticCensus),
            ),
            (
                S::SuccessorConstructorFault,
                "successor-from-predecessor-metadata-unchanged",
                staged(N::SuccessorUnannounced, SPECIFIC),
            ),
            (
                S::SuccessorConstructorFault,
                "wrong-announcement-cycle",
                staged(N::WrongAnnouncementCycle, SPECIFIC),
            ),
            (
                S::SuccessorConstructorFault,
                "wrong-representation-nonce",
                staged(N::WrongRepresentationNonce, SPECIFIC),
            ),
            (
                S::SuccessorConstructorFault,
                "later-admissible-nonce-instead-of-first",
                accepted(N::LaterAdmissibleNonce, D::NonceLeastnessResidual),
            ),
            (
                S::SuccessorConstructorFault,
                "wrong-leaf-version",
                staged(N::SuccessorLeafVersion, SPECIFIC),
            ),
            (
                S::SuccessorConstructorFault,
                "wrong-control-recipe",
                NativeStanding::Unstageable(U::NoOfferablePosition),
            ),
            (
                S::SuccessorConstructorFault,
                "arbitrary-caller-supplied-output-program",
                staged(N::ArbitraryCallerSuppliedOutputProgram, SPECIFIC),
            ),
            (
                S::SuccessorConstructorFault,
                "no-state-successor",
                staged(N::NoStateSuccessor, SPECIFIC),
            ),
            (
                S::SuccessorConstructorFault,
                "two-state-successors",
                NativeStanding::Unstageable(U::SingletonConservationPrecludesASecondCoin),
            ),
            (
                S::SuccessorConstructorFault,
                "extra-state-like-output",
                accepted(N::ExtraStateLikeOutput, D::OutputCountUnread),
            ),
            (
                S::AbiLinkerFault,
                "wrong-transaction-version",
                accepted(N::WrongTransactionVersion, D::VersionAdmittedByConsensus),
            ),
            (
                S::AbiLinkerFault,
                "wrong-sequence",
                staged(N::WrongSequence, SPECIFIC),
            ),
            (
                S::AbiLinkerFault,
                "witness-item-reorder",
                staged(N::WitnessItemReorder, SPECIFIC),
            ),
            (
                S::AbiLinkerFault,
                "predecessor-and-successor-metadata-witnesses-exchanged",
                NativeStanding::Unstageable(U::SuccessorMetadataIsNeverWitnessed),
            ),
            (
                S::AbiLinkerFault,
                "control-block-from-another-program",
                staged(N::ControlBlockFromAnotherProgram, GENERIC),
            ),
            (
                S::AbiLinkerFault,
                "target-bytes-changed-after-abi-validation",
                staged(N::TargetBytesChangedAfterAbiValidation, SPECIFIC),
            ),
        ];
        assert_eq!(expected.len(), NATIVE_ROWS.len());
        for (row, (section, name, standing)) in NATIVE_ROWS.iter().zip(expected) {
            assert_eq!(row.section(), section, "{name} section");
            assert_eq!(row.name(), name, "{name} name");
            assert_eq!(row.standing(), standing, "{name} standing");
        }
    }

    /// The seven inherited admissions keep their mutation, control and
    /// boundary.
    #[test]
    fn inherited_admissions_keep_their_mutation_control_and_boundary() {
        assert_eq!(MaturityMutantRow::ALL.len(), 7);
        let key_path = MaturityMutantRow::ALL
            .iter()
            .filter(|row| row.declared_layer() == EvidenceBoundary::KeyPathRejection)
            .count();
        let script_path = MaturityMutantRow::ALL
            .iter()
            .filter(|row| row.declared_layer() == EvidenceBoundary::ScriptPathRejection)
            .count();
        assert_eq!(key_path, 1);
        assert_eq!(script_path, 6);
        for row in MaturityMutantRow::ALL {
            assert_eq!(row.matrix_row().section(), row.section());
            assert_eq!(row.matrix_row().name(), row.row());
        }
    }

    /// The whole Standing and Declared-boundary cells of the record's
    /// forty-two-row table are rendered from each typed row and compared
    /// cell for cell.
    ///
    /// The Standing and Declared boundary columns are bound here; the
    /// Mutation, Control route and Pinning test columns are prose and are
    /// not. A changed cell in either bound column, or a key that moved,
    /// fails here.
    #[test]
    fn safety_native_record_matches_the_typed_register() {
        let backlog = include_str!("../../../../plans/backlog.md");
        let table = backlog
            .split("· `tab:backlog:safety-native-subjects`")
            .nth(1)
            .expect("record table")
            .split("**Result (Native Safety staging)**")
            .next()
            .expect("table end");
        let recorded: BTreeMap<String, (String, String)> = table
            .lines()
            .filter(|line| line.starts_with("| `16."))
            .map(|line| {
                let columns: Vec<_> = line.split('|').map(str::trim).collect();
                assert_eq!(columns.len(), 8, "six-column row: {line}");
                (
                    columns[1].trim_matches('`').to_owned(),
                    (columns[4].to_owned(), columns[5].to_owned()),
                )
            })
            .collect();
        assert_eq!(recorded.len(), 42);

        for row in NATIVE_ROWS {
            let key = format!("{} / {}", row.section().section(), row.name());
            let (boundary, standing) = recorded.get(&key).expect("native key in the table");
            assert_eq!(
                *boundary,
                render_native_boundary(row.boundary()),
                "{key} boundary"
            );
            assert_eq!(*standing, render_standing(row.standing()), "{key} standing");
        }
        for row in MaturityMutantRow::ALL {
            let key = format!("{} / {}", row.section().section(), row.row());
            let (boundary, standing) = recorded.get(&key).expect("inherited key in the table");
            assert_eq!(
                *boundary,
                render_evidence_boundary(row.declared_layer()),
                "{key} boundary"
            );
            assert_eq!(standing, "inherited admission", "{key} standing");
        }
    }

    /// The observation fold preserves the denominator and historical staging keys.
    #[test]
    fn native_observation_preserves_the_denominator_and_staging_history() {
        let plan = derive_maturity_evidence_plan_with(
            MaturityExecutorProvenanceExpectation::NotStatedByTheOperator,
            MaturityConstructorMaterial::Absent(
                MaturityConstructorMaterialAbsence::NotSuppliedToDerivation,
            ),
        )
        .expect("the evidence plan derives");
        assert_eq!(plan.census().rows(), 206);
        // Thirteen supplementary comparisons add answers after the report campaign.
        assert_eq!(plan.census().answered(), 147);
        // Two ordering claims retire without becoming answered; thirteen comparisons answer.
        assert_eq!(plan.census().outstanding(), 59);
        assert_eq!(plan.census().native_run_required(), 0);
        assert_eq!(row_count(), 206);

        let ledger = maturity_first_party_dispositions().expect("valid retained subjects");
        let mut partition = [0usize; 3];
        for row in &ledger {
            match row.disposition() {
                MaturitySupplementaryDisposition::Discharged(_) => partition[0] += 1,
                MaturitySupplementaryDisposition::OutOfContract { .. } => partition[1] += 1,
                MaturitySupplementaryDisposition::Retained { .. } => partition[2] += 1,
            }
        }
        assert_eq!(partition, [13, 2, 28]);
        assert_eq!(partition.iter().sum::<usize>(), 43);
    }
}
