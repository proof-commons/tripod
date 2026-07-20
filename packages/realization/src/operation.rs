//! Typed operation declarations.

use architecture::OperationId;

use crate::{ExpressionDeclaration, RelationDeclaration};

/// One target-independent operation realization.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OperationRealization {
    pub operation: OperationId,
    pub expressions: Vec<ExpressionDeclaration>,
    pub relations: Vec<RelationDeclaration>,
}
