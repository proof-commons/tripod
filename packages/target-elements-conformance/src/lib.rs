#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

mod census;
pub mod claim;
pub mod commitment_oracle;
pub mod conservation;
pub mod conservation_report;
pub mod constructor;
pub mod declassification;
pub mod disposition;
pub mod emit;
pub mod error;
pub mod executor;
pub mod fixture;
pub mod lifecycle;
pub mod lifecycle_report;
pub mod normalization;
pub mod normalization_report;
pub mod owner_authorization;
pub mod protocol;
pub mod prototype;
pub mod prototype_program;
pub mod prototype_report;
pub mod prototype_validate;
pub mod provenance;
pub mod reference;
pub mod report;
pub mod test_material;
pub mod validate;
pub mod vocabulary;
pub mod wide_floor;

pub use error::NativeConformanceError;

#[cfg(test)]
mod tests;
