//! The adapter from compiler-owned abstract target requirements to
//! Elements target obligations.
//!
//! # Boundary
//!
//! This crate is the join, and only the join. The compiler owns what an
//! approved analysis requires of *some* target, abstractly; the target
//! package owns what one reviewed Elements tapscript contract offers,
//! concretely; neither may name the other. The join has to live
//! somewhere, and it lives here because this is the one package whose
//! contract admits both.
//!
//! It owns no target program, no instruction, no stack schedule, no
//! transaction layout, and no bundle. It emits nothing.
//!
//! # Dependencies
//!
//! `compiler` and `target-elements`, and nothing else — not first-party
//! and not third-party. The crate serializes nothing, hashes nothing,
//! parses nothing, and opens no file.
//!
//! # State
//!
//! Implemented: the package boundary.
//!
//! Not implemented: the capability adapter, and beyond it every backend
//! deliverable — target program type, instruction builder, stack
//! scheduler, backend proof patterns, constructors, and the relocatable
//! bundle.
//!
//! Not claimed: anything about a real node. No target program has been
//! emitted, no transaction has been built, and every evidence
//! requirement the target contract names remains unresolved.

#![forbid(unsafe_code)]
