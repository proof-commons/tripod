#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

pub mod matrix;
pub mod subject;

pub use matrix::{
    EvidenceBoundary, MutationLayer, VectorClass, VectorFamily, VectorPolarity, all_classes,
    class_count, family_census,
};
pub use subject::{CanonicalSubject, ExperimentalSubject, SubjectStanding};
