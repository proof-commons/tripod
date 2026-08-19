#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

mod census;
pub mod claim;
pub mod constructor;
pub mod error;
pub mod executor;
pub mod fixture;
pub mod protocol;
pub mod prototype;
pub mod prototype_program;
pub mod prototype_report;
pub mod prototype_validate;
pub mod provenance;
pub mod report;
pub mod validate;
pub mod vocabulary;
pub mod wide_floor;

pub use error::NativeConformanceError;

#[cfg(test)]
mod tests;
