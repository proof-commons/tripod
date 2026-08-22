#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

mod analyzed;
mod analyzed_operation;
mod analyzed_validate;
mod capability;
mod carrier;
mod case;
mod constructibility;
mod coverage;
mod coverage_graph;
mod disclosure;
pub mod error;
mod expression;
mod fold;
mod foundation;
pub mod input;
mod layout;
mod lifecycle;
pub mod operation_plan;
mod placement;
mod proof;
mod relation;
mod requirement;
mod search_counter;
mod source;
mod sponsor_region;
pub mod target;

pub use constructibility::validate_required_dependency;
pub use error::CompileError;
pub use expression::ExpressionCycleComponent;
pub use input::{
    AnalysisPolicy, BoundCompilerInput, CompilationScope, ProofSearchLimits, bind_input,
};
pub use relation::RelationCycleComponent;
pub use source::OperandId;

#[cfg(test)]
mod tests;
