//! Why an ABI was not derived, or a transaction not constructed.
//!
//! Construction failure, never target rejection (§1.5). Every variant
//! below names something wrong with what this crate was asked to build
//! or asked to read, and none of them is a statement about what a
//! target node would do with a well-formed result. A refusal returns no
//! partial value (§1.11): there is no variant carrying a half-built
//! transaction, and no entry point returns one alongside a diagnostic.
//!
//! The decoder's refusals sit here too, and they are the same kind of
//! thing. Bytes this crate cannot turn into validated typed values are
//! refused rather than partially interpreted, because §1.12 admits
//! external target bytes only through an explicit parser that converts
//! them immediately into validated typed values — and a parser that
//! ignored a field it did not model would be converting them into
//! something else.

use std::collections::BTreeSet;

use linker::backend::{CompactAshShape, InputRole, LeafRole, OutputRole};
use target_elements::ResourceDimension;

use crate::bytes::Outpoint;

/// Why the transaction layer refused.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum TransactionRefusal {
    // --- ABI derivation ----------------------------------------------
    /// The bundle handed in claims more than a candidate link.
    ///
    /// The transaction ABI is a candidate ABI, and §1.9 keeps the
    /// states distinct in both directions: a candidate ABI derived from
    /// something claiming to be final would blur the boundary just as
    /// surely as a final ABI derived from a candidate bundle.
    BundleIsNotACandidate,
    /// The bundle's linked constructor is bound to a different reviewed
    /// contract revision than the target handed in.
    ContractRevisionMismatch,
    /// The bundle carries no layout for a shape it admits.
    MissingLayout(CompactAshShape),
    /// The bundle carries no linked program for a leaf its tree commits
    /// to, or no control-path recipe for a leaf it linked.
    MissingLeaf(LeafRole),
    /// A shape's layout does not place a role the ABI must name.
    MissingInputRole {
        /// The shape whose layout is short.
        shape: CompactAshShape,
        /// The role no run holds.
        role: InputRole,
    },
    /// A shape's layout does not place an output role the ABI must
    /// name.
    MissingOutputRole {
        /// The shape whose layout is short.
        shape: CompactAshShape,
        /// The role no position holds.
        role: OutputRole,
    },
    /// A deployment symbol the ABI needs has no linked definition.
    MissingDeploymentSymbol {
        /// A rendering of the symbol, for the report.
        symbol: &'static str,
    },
    /// A deployment symbol resolved to a value of the wrong kind.
    MalformedDeploymentSymbol {
        /// A rendering of the symbol, for the report.
        symbol: &'static str,
    },
    /// The linked coordinator leaf requires the anchor at an index the
    /// canonical layout does not put it at.
    CoordinatorNotAtAnchor {
        /// Where the layout puts the coordinator.
        placed: u16,
    },

    // --- The pinned instance -----------------------------------------
    /// The pinned ASH program is not the reviewed witness-v1 form.
    ///
    /// A pin is the one place a caller states which deployed object
    /// this ABI builds against, so its shape is checked exactly rather
    /// than trusted: a pin of the wrong form would produce a
    /// transaction that spends nothing.
    MalformedPinnedProgram {
        /// How many bytes the offered program occupies.
        offered: usize,
    },
    /// The pinned instance names an internal key other than the one the
    /// bundle linked.
    PinnedInternalKeyMismatch,
    /// The pinned instance names a leaf version other than the one the
    /// committed tree carries.
    PinnedLeafVersionMismatch {
        /// The version the pin states.
        pinned: u8,
        /// The version the tree commits under.
        linked: u8,
    },

    // --- Request validation -------------------------------------------
    /// The request selects no ASH input.
    EmptyAshSelection,
    /// The request selects one outpoint more than once.
    DuplicateOutpoint(Outpoint),
    /// An outpoint appears in both the ASH selection and the sponsor
    /// suffix, which would put one input in two regions.
    OverlappingOutpoint(Outpoint),
    /// The selected counts match no shape the candidate admits.
    UnsupportedShape {
        /// How many ASH inputs the request selects.
        ash_inputs: usize,
        /// How many sponsor inputs it selects.
        sponsor_inputs: usize,
        /// Whether it selects a sponsor change destination.
        sponsor_change: bool,
    },
    /// A selected ASH input's public view states an asset other than
    /// the closed protocol asset.
    AshInputCarriesForeignAsset(Outpoint),
    /// A selected ASH input's public view states a program other than
    /// the pinned one.
    AshInputCarriesForeignProgram(Outpoint),
    /// A selected ASH input's public view states no explicit amount.
    ///
    /// The fixed Phase-4 representation is explicit `U`, so an ASH
    /// whose amount is a commitment is not an ASH this candidate can
    /// consolidate — and refusing it is not a claim that the target
    /// would reject it.
    AshInputAmountNotExplicit(Outpoint),
    /// A selected sponsor input's public view states a confidential
    /// asset.
    ///
    /// The reviewed profile forces a sponsor asset explicit, because
    /// the coordinator authenticates it. Sponsor *value* opacity
    /// survives; sponsor asset opacity does not exist under this
    /// relation.
    SponsorInputAssetNotExplicit(Outpoint),
    /// A selected sponsor input carries an asset other than the reserve
    /// asset.
    SponsorInputCarriesForeignAsset(Outpoint),
    /// A selected sponsor input's program is not an admitted class.
    SponsorProgramClassNotAdmitted(Outpoint),
    /// The request selects a sponsor change destination but no sponsor
    /// input.
    SponsorChangeWithoutSponsor,
    /// The consolidated successor amount overflows the target's
    /// checked range.
    SuccessorAmountOutOfRange,
    /// The declared fee is not covered by the sponsor inputs the
    /// request selects.
    ///
    /// Raised only where every selected sponsor value is explicit. A
    /// sponsor whose value stays committed is not compared with
    /// anything, and this refusal is unreachable for it — which is the
    /// erasure law holding rather than a gap in the check.
    SponsorValueDoesNotCoverFee,

    // --- Signing ------------------------------------------------------
    /// A signing request was issued before every protected output was
    /// final.
    ///
    /// Structurally unreachable through the public pipeline, and kept
    /// as a typed outcome anyway: §15.8's ordering is a rule about what
    /// a signer's signature can be relied on to cover, so the layer
    /// that would violate it names the violation.
    SigningRequestedBeforeOutputsFinal,
    /// A sponsor capability returned no signature for an input it was
    /// asked to sign.
    SponsorSignatureMissing(Outpoint),
    /// A sponsor capability returned a signature against a transaction
    /// other than the one it was asked to sign.
    SponsorSignatureBindingMismatch(Outpoint),
    /// A sponsor capability returned a witness stack the admitted
    /// program class does not take.
    SponsorWitnessShapeRefused {
        /// The input.
        outpoint: Outpoint,
        /// How many items the capability returned.
        offered: usize,
        /// How many the admitted class takes.
        expected: usize,
    },

    // --- Preflight ----------------------------------------------------
    /// The assembled transaction does not conserve one asset.
    ///
    /// Only assets whose every input and output field is explicit are
    /// checked, and the check is a construction check rather than a
    /// consensus verdict.
    ConservationFailed {
        /// A rendering of the asset, for the report.
        asset: String,
    },
    /// The assembled transaction exceeds a bound the deployment states.
    ResourceBoundExceeded {
        /// The dimension.
        dimension: ResourceDimension,
        /// The figure the transaction reaches.
        reached: u64,
        /// The bound.
        bound: u64,
    },
    /// The assembled transaction's role census does not match the
    /// layout the ABI states for its shape.
    LayoutCensusMismatch {
        /// The shape.
        shape: CompactAshShape,
        /// The roles the layout places and the assembly does not.
        missing: BTreeSet<OutputRole>,
    },

    // --- Decoding target bytes ----------------------------------------
    /// The byte string ended inside a field.
    TruncatedTargetBytes {
        /// The offset the read began at.
        at: usize,
    },
    /// The byte string held more than one transaction's bytes.
    TrailingTargetBytes {
        /// Where the transaction ended.
        at: usize,
    },
    /// A count was encoded in a wider form than it needs.
    NonMinimalCompactSize {
        /// Where the count began.
        at: usize,
        /// The value it encoded.
        value: u64,
    },
    /// A field carried a prefix byte the reviewed encoding does not
    /// define.
    UnrecognizedFieldPrefix {
        /// The offered byte.
        prefix: u8,
    },
    /// The flag byte was neither of the two the target writes.
    UnrecognizedWitnessFlag {
        /// The offered byte.
        offered: u8,
    },
    /// An asset identifier was not the reviewed width.
    MalformedAssetIdentifier {
        /// How many bytes were offered.
        offered: usize,
    },
    /// An outpoint index occupies a bit the target reserves for a
    /// marker.
    OutpointIndexOutOfRange {
        /// The offered index.
        offered: u32,
    },
    /// The bytes carry an issuance, which is outside the candidate.
    IssuanceInputRefused,
    /// The bytes carry a peg-in, which is outside the candidate.
    PeginInputRefused,
    /// The bytes carry an issuance proof, which is outside the
    /// candidate.
    IssuanceProofRefused,
    /// The bytes carry a surjection proof, which is outside the
    /// candidate.
    SurjectionProofRefused,
    /// The bytes carry a range proof, which is outside the candidate.
    RangeProofRefused,
    /// A transaction was assembled with no input.
    EmptyInputCensus,
    /// A transaction was assembled with no output.
    EmptyOutputCensus,
    /// A transaction was assembled with a witness census that is not
    /// one entry per input.
    WitnessCensusMismatch {
        /// How many inputs.
        inputs: usize,
        /// How many witnesses.
        witnesses: usize,
    },
}
