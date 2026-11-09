//! The immutable typed compiler input boundary (P2-004).
//!
//! [`bind_input`] is the only way to obtain a [`BoundCompilerInput`]:
//! it re-runs the realization owner's validation, requires the
//! realization's architecture binding to equal the requested
//! architecture, and requires every compiler-scope operation to be
//! declared by the realization. The bound value is immutable and
//! read-only; no partial analyzed program, plan, or digest is exposed
//! here.

use architecture::{Architecture, OperationId};
use realization::{ArchitectureBinding, RealizationError, ScopedRealizationSpec};

use crate::CompileError;
pub use crate::engine_attestation::AttestationOwner;
#[cfg(any(test, feature = "keyed-experimental"))]
pub use crate::engine_keyed::{KeyedBindingError, KeyedOwner};

/// Explicit, canonical compiler operation scope.
///
/// Distinct from the realization's own scope: the compiler may analyze
/// a subset of what the realization declares, but never more.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompilationScope {
    operations: Vec<OperationId>,
}

impl CompilationScope {
    /// Construct a nonempty scope in stable architecture-code order.
    ///
    /// # Errors
    ///
    /// [`CompileError::EmptyCompilationScope`] for an empty scope;
    /// [`CompileError::DuplicateScopeOperation`] for a repeated member
    /// (scope is a set — a duplicate is a caller defect, never
    /// silently normalized away).
    pub fn from_operations(
        operations: impl IntoIterator<Item = OperationId>,
    ) -> Result<Self, CompileError> {
        let mut operations = operations.into_iter().collect::<Vec<_>>();

        operations.sort_by_key(|operation| operation.code());

        if operations.is_empty() {
            return Err(CompileError::EmptyCompilationScope);
        }

        if let Some(duplicate) = operations
            .windows(2)
            .find_map(|pair| (pair[0] == pair[1]).then_some(pair[0]))
        {
            return Err(CompileError::DuplicateScopeOperation {
                operation: duplicate,
            });
        }

        Ok(Self { operations })
    }

    /// Operations in stable architecture-code order.
    #[must_use]
    pub fn operations(&self) -> &[OperationId] {
        &self.operations
    }
}

/// Explicit work limits for the exact proof-plan search.
///
/// Compiler configuration with an explicit constructor — no ambient
/// default exists for release-sensitive analysis, and no configuration
/// identity is minted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProofSearchLimits {
    pub maximum_states: std::num::NonZeroU64,
    pub maximum_candidates: std::num::NonZeroU64,
}

impl ProofSearchLimits {
    #[must_use]
    pub const fn new(
        maximum_states: std::num::NonZeroU64,
        maximum_candidates: std::num::NonZeroU64,
    ) -> Self {
        Self {
            maximum_states,
            maximum_candidates,
        }
    }
}

/// Typed analysis policy.
///
/// The strict semantics are the only reviewed policy: preserve every
/// in-scope realization relation, never weaken an unsupported
/// relation, retain explicit external evidence requirements, and fail
/// rather than emit a partial plan. The search limits are the policy's
/// one real configuration consumer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AnalysisPolicy {
    pub proof_search_limits: ProofSearchLimits,
}

impl AnalysisPolicy {
    /// The strict policy with explicit search limits.
    #[must_use]
    pub const fn strict(proof_search_limits: ProofSearchLimits) -> Self {
        Self {
            proof_search_limits,
        }
    }
}

/// One immutable validated compiler input.
///
/// Fields are private; there is no mutable accessor, no public
/// constructor besides [`bind_input`], no serializer, and no digest.
#[derive(Clone, Debug)]
pub struct BoundCompilerInput<O = AttestationOwner> {
    owner: O,
    policy: AnalysisPolicy,
}

impl<O> BoundCompilerInput<O> {
    pub(crate) const fn owner(&self) -> &O {
        &self.owner
    }

    /// The explicit analysis policy retained at binding.
    #[must_use]
    pub const fn policy(&self) -> AnalysisPolicy {
        self.policy
    }
}

impl BoundCompilerInput {
    /// The validated realization this compilation analyzes.
    #[must_use]
    pub const fn realization(&self) -> &ScopedRealizationSpec {
        self.owner.source()
    }

    pub(crate) fn analyze_relation_records(
        &self,
    ) -> Result<crate::relation::CompilerRelationAnalysis, CompileError> {
        let analyzed = crate::engine_records::analyze_bound_relations(self)?;
        analyzed.lowering.compatibility_view(analyzed.result)
    }

    /// The explicit compiler operation scope.
    #[must_use]
    pub const fn scope(&self) -> &CompilationScope {
        self.owner.scope()
    }

    /// The architecture identity both sides of this binding share.
    #[must_use]
    pub fn architecture_binding(&self) -> &ArchitectureBinding {
        self.realization().architecture()
    }

    /// The complete operation census of the validated architecture.
    ///
    /// Captured here, from the typed architecture the binder validated,
    /// because the architecture binding is an identity rather than a
    /// census: an analyzed program that must state which architecture
    /// operations lie outside its compiler scope needs the whole set,
    /// and deriving it from source files or planning prose instead
    /// would be a guess about the model rather than a reading of it.
    #[must_use]
    pub const fn architecture_operations(&self) -> &std::collections::BTreeSet<OperationId> {
        self.owner.architecture_operations()
    }
}

/// Validate and bind one compiler input.
///
/// # Errors
///
/// [`CompileError::ArchitectureBindingMismatch`] when the realization
/// binds a different architecture identity than `architecture`;
/// [`CompileError::InvalidRealization`] when the realization fails its
/// owner's re-run validation; [`CompileError::IncompleteRealizationScope`]
/// when the compiler scope names an operation the realization does not
/// declare.
fn bind_attestation_input(
    architecture: &Architecture,
    realization: ScopedRealizationSpec,
    scope: CompilationScope,
    policy: AnalysisPolicy,
) -> Result<BoundCompilerInput, CompileError> {
    let scope_operation_count = crate::instrumentation::count(scope.operations().len());
    let span = tracing::debug_span!(target: "compiler::input", "bind_input", scope_operation_count);
    let _entered = span.enter();
    let result = (|| {
        // The owner validates; the compiler never trusts a handed value.
        realization
            .validate_against(architecture)
            .map_err(|error| match error {
                RealizationError::ArchitectureBindingMismatch => {
                    CompileError::ArchitectureBindingMismatch
                }
                _ => CompileError::InvalidRealization,
            })?;

        // The realization's explicit partial scope is retained honestly:
        // compiler scope must be a subset, never a completion of it.
        for operation in scope.operations() {
            if realization.operation(*operation).is_none() {
                return Err(CompileError::IncompleteRealizationScope {
                    operation: *operation,
                });
            }
        }

        Ok(BoundCompilerInput {
            owner: AttestationOwner::new(
                realization,
                scope,
                architecture
                    .operations
                    .iter()
                    .map(|operation| operation.id)
                    .collect(),
            ),
            policy,
        })
    })();
    match &result {
        Ok(_) => tracing::debug!(
            target: "compiler::input",
            outcome = "completed",
            scope_operation_count,
            message = "input binding completed"
        ),
        Err(error) if error.tracing_failed() => tracing::debug!(
            target: "compiler::input",
            outcome = "failed",
            error_kind = error.tracing_kind(),
            scope_operation_count,
            message = "input binding completed"
        ),
        Err(error) => tracing::debug!(
            target: "compiler::input",
            outcome = "refused",
            refusal_kind = error.tracing_kind(),
            scope_operation_count,
            message = "input binding completed"
        ),
    }
    result
}

#[cfg(any(test, feature = "keyed-experimental"))]
mod owner_sealed {
    pub trait Sealed {}
    impl Sealed for architecture::Architecture {}
    impl Sealed for architecture::keyed::KeyedManifest {}
}

/// Closed manifest vocabulary of the binding seam. The associated owner is
/// fixed here and retained in the bound type; no later owner selection occurs.
#[cfg(any(test, feature = "keyed-experimental"))]
pub trait InputBinding: owner_sealed::Sealed {
    type Realization;
    type Scope;
    type Owner;
    type Error;
    /// Revalidate the original owner before creating a bound input.
    ///
    /// # Errors
    /// Returns the selected owner's realization, binding or scope refusal.
    fn bind_owned(
        &self,
        realization: Self::Realization,
        scope: Self::Scope,
        policy: AnalysisPolicy,
    ) -> Result<BoundCompilerInput<Self::Owner>, Self::Error>;
}

#[cfg(any(test, feature = "keyed-experimental"))]
impl InputBinding for Architecture {
    type Realization = ScopedRealizationSpec;
    type Scope = CompilationScope;
    type Owner = AttestationOwner;
    type Error = CompileError;
    fn bind_owned(
        &self,
        realization: Self::Realization,
        scope: Self::Scope,
        policy: AnalysisPolicy,
    ) -> Result<BoundCompilerInput<Self::Owner>, Self::Error> {
        bind_attestation_input(self, realization, scope, policy)
    }
}

#[cfg(any(test, feature = "keyed-experimental"))]
impl InputBinding for architecture::keyed::KeyedManifest {
    type Realization = realization::keyed::KeyedScopedRealization;
    type Scope = Vec<architecture::semantic::SemanticOperationId>;
    type Owner = KeyedOwner;
    type Error = KeyedBindingError;
    fn bind_owned(
        &self,
        realization: Self::Realization,
        mut scope: Self::Scope,
        policy: AnalysisPolicy,
    ) -> Result<BoundCompilerInput<Self::Owner>, Self::Error> {
        let span = tracing::debug_span!(target: "compiler::input", "bind_keyed_input", scope_operation_count = crate::instrumentation::count(scope.len()));
        let _entered = span.enter();
        let result = (|| {
            realization
                .validate_against(self)
                .map_err(KeyedBindingError::InvalidRealization)?;
            scope.sort();
            if scope.is_empty() {
                return Err(KeyedBindingError::EmptyScope);
            }
            if let Some(operation) = scope
                .windows(2)
                .find_map(|pair| (pair[0] == pair[1]).then_some(pair[0]))
            {
                return Err(KeyedBindingError::DuplicateOperation(operation));
            }
            for operation in &scope {
                if !self.operations.contains(operation) {
                    return Err(KeyedBindingError::ForeignOperation(*operation));
                }
            }
            Ok(BoundCompilerInput {
                owner: KeyedOwner::new(self.clone(), realization, scope),
                policy,
            })
        })();
        tracing::debug!(target: "compiler::input", outcome = if result.is_ok() { "completed" } else { "refused" }, refusal_kind = result.as_ref().err().map(KeyedBindingError::kind), "keyed input binding completed");
        result
    }
}

/// Bind the attestation realization and retain its owner for re-derivation.
///
/// # Errors
/// Returns the attestation owner's realization, binding or scope refusal.
#[cfg(not(any(test, feature = "keyed-experimental")))]
pub fn bind_input(
    architecture: &Architecture,
    realization: ScopedRealizationSpec,
    scope: CompilationScope,
    policy: AnalysisPolicy,
) -> Result<BoundCompilerInput, CompileError> {
    bind_attestation_input(architecture, realization, scope, policy)
}

/// Experimental typed binding; fixes the owner for all later re-derivation.
///
/// # Errors
/// Returns the selected owner's realization, binding or scope refusal.
#[cfg(any(test, feature = "keyed-experimental"))]
pub fn bind_input<A: InputBinding>(
    architecture: &A,
    realization: A::Realization,
    scope: A::Scope,
    policy: AnalysisPolicy,
) -> Result<BoundCompilerInput<A::Owner>, A::Error> {
    architecture.bind_owned(realization, scope, policy)
}
