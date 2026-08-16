//! The independent host reference for constructor output construction.
//!
//! # What this is for
//!
//! A constructor fixture states an exact tree, an exact internal key,
//! and an exact successor output program. Something has to compute
//! those exact values, and it must not be the thing under test: an
//! expectation produced by the target program builder would agree with
//! the target program builder however wrong both were
//! (Guide-10 `rule:guide10:independent-oracles`,
//! `rule:guide10:constructor-host`).
//!
//! So this is a second implementation, written from the target's own
//! source with the provenance recorded beside each constant, using this
//! package's own hashing and curve arithmetic and nothing from
//! `tapscript` except the reviewed leaf version, which is a target fact
//! that package owns.
//!
//! # Three opinions, not one
//!
//! The oracle is checked against the published target vectors the
//! upstream tree carries — which were regenerated for this target's tag
//! strings and are not derived from anything here — and its algorithm
//! is stated so that it can be read against the upstream functional
//! framework's construction. A target-native spend verdict is the third
//! opinion, and it belongs to a run rather than to this module
//! (Guide-10 `rule:guide10:constructor-oracle`).
//!
//! # No secret material, at any point
//!
//! Nothing here generates, accepts, holds, or derives a private scalar.
//! The internal key is a published point with no known private scalar,
//! the tweak is a hash of public data, and the arithmetic is public
//! point arithmetic. There is no signing here and none may be added
//! (Guide-10 `rule:guide10:public-data`).

pub mod curve;
pub mod internal_key;
pub mod metadata;
pub mod metadata_leaf;
pub mod tagged;
pub mod totality;
pub mod tree;

pub use curve::{FIELD_ELEMENT_BYTES, PointDecodingDefect};
pub use internal_key::UNSPENDABLE_INTERNAL_KEY;
pub use metadata::{METADATA_BYTES, MetadataDefect, PrototypeMetadata, TransitionDefect};
pub use metadata_leaf::{metadata_leaf_program, metadata_leaf_script};
pub use tagged::{DIGEST_BYTES, Digest32, tagged_hash};
pub use totality::{TotalityDefect, TotalityOutcome, TweakTotalityPolicy, construct_under_policy};
pub use tree::{
    ConstructedOutput, ConstructionDefect, FixtureTapTree, TreeDefect, TweakDefect, branch_hash,
    construct, control_block, leaf_hash, output_program, tweak, tweaked_key,
};
