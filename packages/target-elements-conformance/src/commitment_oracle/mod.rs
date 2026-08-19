//! An independent oracle for confidential-value expectations.
//!
//! # What it is for
//!
//! The oracle predicts asset generators and Pedersen commitments from
//! their inputs, so that a later confidential-transaction fixture states
//! what it expects before anything runs. It computes those expectations
//! without the tapscript opening pattern, without the native executor's
//! answer, without a report row, and without reading a target
//! observation as an expected value
//! `(´[PLAN-rule:guide10:independent-oracles]´)`.
//!
//! # Why it is first-party
//!
//! The target materializes transactions with the node's own vendored C
//! curve library. An oracle bound to that same library — through any of
//! its bindings — would be one opinion wearing two hats, and the
//! agreement it reported would be a tautology. So the arithmetic here is
//! written from the published curve parameters and the reviewed recipe,
//! over the bignum the workspace already carries. No third-party curve
//! crate is added, and no entry enters the lock file, which keeps the
//! oracle independent by construction rather than by policy.
//!
//! # What it does not claim
//!
//! Nothing here is constant-time, side-channel resistant, or suitable
//! for a secret scalar. No blinding factor reaching this module is a
//! production secret; the vectors are public fixtures stated in the open
//! under ADR-015. Deriving a generator and constructing a commitment is
//! also not an opening proof: the reviewed target still has no on-script
//! form for one, and this module changes nothing about that.
//!
//! # Provenance
//!
//! Every rule is read from the vendored library at merged tip `78499c2`
//! under ADR-018, cited file and line at the step that uses it. The
//! oracle's own output is checked against the vendored library's
//! published fixed vectors, which cover the curve map, the generator
//! derivation, and both prefix conventions.

pub mod commitment;
pub mod curve;
pub mod generator;
pub mod vector;

pub use commitment::{
    CommitmentDefect, SEMANTIC_AMOUNT_BOUND, ScalarDefect, commitment, is_semantic_amount,
};
pub use curve::{PREFIXED_POINT_BYTES, PointEncodingDefect};
pub use generator::{GeneratorDefect, serialized_asset_generator};
pub use vector::{
    CommitmentSource, PointMismatch, PublicCommitmentVector, ThreeWayComparison, ThreeWayOutcome,
    VectorDefect, compare_points,
};
