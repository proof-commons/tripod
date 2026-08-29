//! The sponsored owner-signing negative ceremony: a sponsored successor
//! re-signed over its own mutated bytes, carrying the three mutants that
//! confuse the sponsor range with the protocol range.
//!
//! # The rows this ceremony drives
//!
//! Three, all of them faults of REGION rather than of value:
//! `receipt-sponsor-range-exchange` swaps a receipt coin into a sponsor
//! position; `sponsor-change-in-protocol-range` moves the sponsor-change
//! output into the destination prefix; and `sponsor-protocol-overlap`
//! claims one position for both regions at once. Each keeps the
//! per-asset sums intact, so each reaches the covenant's sponsor
//! isolation fragment, which inspects the asset at a sponsor position
//! and requires it not to be the protocol asset.
//!
//! # Why a ceremony of its own
//!
//! The sponsorless owner-signing ceremony builds a successor with no
//! sponsor input and no sponsor change, so none of the three regions
//! these rows confuse exists in it. A sponsored successor is a different
//! candidate, not a mutation of that one, and building it here leaves
//! every digest that ceremony recorded where it is.
//!
//! # The separator is a distinct byte range each
//!
//! All three refuse at the same clause and may draw the same words, so
//! the separating fact is the witnessless byte RANGE each mutation
//! confines itself to — the sponsor input region, the change-output
//! region, and the overlapped position are three disjoint areas of one
//! control. This is the `private-ct-imbalance` discipline applied across
//! the sponsor half of a candidate, and it takes the range rather than
//! the verdict because the verdict is the fragment's and not the row's.
//!
//! # Every key here is published test material
//!
//! The signing scalars are the BIP-340 appendix secret keys, admitted
//! under ADR-015's test-material rule `(´[ADR015-rule:security:test-material]´)`.
//! They authorize nothing on any network anyone uses, and the chain this
//! ceremony runs against is created and destroyed by the run.
