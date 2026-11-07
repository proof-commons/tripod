//! Payload-free compiler completion observations.

use crate::CompileError;

pub fn count(value: usize) -> u64 {
    u64::try_from(value).unwrap_or(u64::MAX)
}

pub fn plan_completion<T>(
    result: &Result<(T, u64), CompileError>,
    operation_code: u16,
    family: &'static str,
) {
    match result {
        Ok((_, proof_plan_count)) => tracing::info!(
            target: "compiler::plan",
            outcome = "completed",
            operation_code,
            family,
            proof_plan_count = *proof_plan_count,
            message = "target operation planned"
        ),
        Err(error) if error.tracing_failed() => tracing::info!(
            target: "compiler::plan",
            outcome = "failed",
            operation_code,
            family,
            error_kind = error.tracing_kind(),
            message = "target operation planned"
        ),
        Err(error) => tracing::debug!(
            target: "compiler::plan",
            outcome = "refused",
            operation_code,
            family,
            refusal_kind = error.tracing_kind(),
            message = "target operation planned"
        ),
    }
}
