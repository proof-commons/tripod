//! Offline final-evidence admission through the existing public owners.
use vectors::{
    keyed_constructor, keyed_evidence, keyed_final_capture, keyed_final_manifest,
    keyed_final_native, keyed_history_reader, maturity_observer,
};

#[path = "../src/keyed_evidence/final_native.rs"]
mod final_native;
#[path = "../src/keyed_final_corpus.rs"]
mod keyed_final_corpus;
