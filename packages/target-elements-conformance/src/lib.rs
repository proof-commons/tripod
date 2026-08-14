//! The secretless target-native conformance harness.
//!
//! # Boundary
//!
//! This package owns the *mechanics* of asking an external Elements
//! executor what it did: the fixture language, the child-process
//! protocol, the executor driver, the typed report, and the first-party
//! evidence plan. It owns no target semantics whatever. Every reviewed
//! fact a fixture is stated against — the execution domain, the leaf
//! version, the primitive contracts, the encodings, the evidence
//! requirements — belongs to `target-elements`, and the exact script
//! bytes a fixture carries are produced by a typed `tapscript` program
//! rather than assembled here.
//!
//! # What the harness cannot establish
//!
//! An executor is caller-selected code. Selecting one grants it
//! execution authority (ADR-015), and the harness does not authenticate
//! it, sandbox it, or prove it independent of anything. Its handshake is
//! *provenance*: a dishonest executor can misdescribe itself, and the
//! report never claims otherwise. A mock executor is admitted for
//! protocol and failure-path tests and can never satisfy the
//! target-native gate — the gate requires an explicit nonmock selection,
//! recorded as ordinary provenance.
//!
//! # No secret material
//!
//! No interface here accepts an RPC username, a password, a bearer
//! token, a cookie path, a private key, a signing nonce, a production
//! blinding factor, a private opening, a wallet path, or a production
//! endpoint, and none may be added. If an executor needs to authenticate
//! to a node, it establishes that boundary itself, outside this process.
//!
//! # No identity
//!
//! Nothing here is hashed. There is no report digest, no fixture-set
//! digest, no executor digest, and no field reserved for one: a report
//! is compared by its typed content and its exact bytes, and no
//! persistent report identity is admitted until a cross-process release
//! consumer exists (Guide-9 §1.8, ADR-016).
//!
//! # State
//!
//! Implemented: the package boundary, the typed error root, the wire
//! vocabulary that names reviewed target identities, the generic fixture
//! language, the secretless executor protocol, and the external
//! executor driver.
//!
//! Not implemented: the canonical fixture census. No primitive fixture
//! has been authored yet, so no run of the checker can satisfy the
//! Guide-9 evidence plan, and the command reports that state rather than
//! passing on an empty census.

#![forbid(unsafe_code)]

pub mod error;
pub mod executor;
pub mod fixture;
pub mod protocol;
pub mod vocabulary;

pub use error::NativeConformanceError;

#[cfg(test)]
mod tests;
