//! Authenticated deployment operators mapped into realization's vocabulary.
//!
//! The caller assigns one operation to a frozen request before authorization.
//! That assignment is explicit evidence, not an inference from transaction bytes;
//! binding it to finalized observations belongs to the later observation boundary.

use linker::OperatorDeploymentBinding;
use realization::{ObservedOperatorMembership, OperatorMembershipDisposition, RealizationScope};
use tapscript::EstablishedOperatorProfile;
use target_elements::{ReviewedElementsTapscriptDefinition, TargetContractVersion};

use crate::operator_right::{ConstructionRight, OperatorRightRegistry, RightFailure, RightScope};
use crate::operator_signing::{OperatorAuthorizedCandidate, OperatorSigningRequest};

/// A declared deployment-to-model operator association for one operation.
///
/// The binding supplies the key: an established profile contains coverage and
/// revision, not a key. This declaration supplies no signature or membership by
/// itself. Its profile and deployment are compared with verified authorization.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OperatorMembershipMapping {
    binding: OperatorDeploymentBinding,
    profile: EstablishedOperatorProfile,
    operation: RealizationScope,
}

impl OperatorMembershipMapping {
    /// Declare the deployment operator intended by one abstract operation.
    ///
    /// # Errors
    /// Refuses a scope containing more than one operation.
    pub fn new(
        binding: OperatorDeploymentBinding,
        profile: EstablishedOperatorProfile,
        operation: RealizationScope,
    ) -> Result<Self, OperatorMembershipRefusal> {
        check_operation(&operation)?;
        Ok(Self {
            binding,
            profile,
            operation,
        })
    }

    /// Compare the declared model association with a verified deployment.
    ///
    /// # Errors
    /// Refuses a different deployment, key, profile or capability revision.
    pub fn check(
        &self,
        current: TargetContractVersion,
        verified: &OperatorDeploymentBinding,
    ) -> Result<(), OperatorMappingRefusal> {
        if self.binding.capability_revision() != current
            || self.profile.check_revision(current).is_err()
        {
            return Err(OperatorMappingRefusal::Revision);
        }
        if self.binding.deployment() != verified.deployment() {
            return Err(OperatorMappingRefusal::Deployment);
        }
        if self.binding.key().encoding() != verified.key().encoding()
            || self.binding.key().bytes() != verified.key().bytes()
        {
            return Err(OperatorMappingRefusal::Key);
        }
        if self.binding.profile_matches(&self.profile).is_err()
            || verified.profile_matches(&self.profile).is_err()
        {
            return Err(OperatorMappingRefusal::Profile);
        }
        Ok(())
    }
}

/// A frozen operation assignment carrying its still-affine construction right.
#[derive(Debug)]
pub struct OperatorMembershipRequest<'binding> {
    request: OperatorSigningRequest<'binding>,
    right: ConstructionRight,
    operation: RealizationScope,
}

impl<'binding> OperatorMembershipRequest<'binding> {
    /// Associate a frozen request and right with exactly one operation.
    ///
    /// The operation is a caller assertion pending finalized observation binding.
    /// The right is moved, never cloned; the registry will check its live standing.
    ///
    /// # Errors
    /// Refuses a multi-operation scope or a right for another frozen scope.
    pub fn new(
        request: OperatorSigningRequest<'binding>,
        right: ConstructionRight,
        operation: RealizationScope,
    ) -> Result<Self, OperatorMembershipRefusal> {
        check_operation(&operation)?;
        let offered = RightScope::new(&request, *right.scope().branch())
            .map_err(|_| OperatorMembershipRefusal::RightScope)?;
        if &offered != right.scope() {
            return Err(OperatorMembershipRefusal::RightScope);
        }
        Ok(Self {
            request,
            right,
            operation,
        })
    }
}

/// Refusals of the explicit deployment-to-model association.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OperatorMappingRefusal {
    /// The model association names another deployment.
    Deployment,
    /// The model association names another operator key.
    Key,
    /// The established profile differs from the committed profile.
    Profile,
    /// The association or profile was established at another revision.
    Revision,
}

/// Producer failures that establish no membership decision.
#[derive(Debug)]
pub enum OperatorMembershipRefusal {
    /// A request or mapping must name exactly one operation.
    OperationScope,
    /// The frozen assignment and requested model association name different operations.
    OperationMismatch,
    /// The offered authorization belongs to another frozen request or deployment.
    AuthorizationMismatch,
    /// The verified deployment is stale at this consumption boundary.
    AuthorizationRevision,
    /// The right names another deployment, predecessor, key or revision.
    RightScope,
    /// The registry refused the token, candidate bytes or signing transition.
    Right(Box<RightFailure>),
    /// No non-whitespace run description was supplied.
    EmptyRun,
    /// Realization refused the producer's provenance text.
    Provenance,
}

fn check_operation(operation: &RealizationScope) -> Result<(), OperatorMembershipRefusal> {
    if operation.operations().len() == 1 {
        Ok(())
    } else {
        Err(OperatorMembershipRefusal::OperationScope)
    }
}

/// Produce one membership decision after consuming verified affine authority.
///
/// Mapping refusals `Deployment`, `Key`, `Profile` and `Revision` yield
/// `NonMember` only after authorization for the request's deployment has been
/// checked and its right consumed. Wrong authorization, stale authorization,
/// operation mismatch, invalid right and missing run are producer refusals and
/// emit no witness. The registry records the existing in-process verifier's
/// standing; this producer claims neither native acceptance nor observation binding.
///
/// # Errors
/// Returns a closed producer refusal before emitting any membership evidence.
pub fn produce_operator_membership<'binding>(
    target: &ReviewedElementsTapscriptDefinition,
    mapping: &OperatorMembershipMapping,
    request: OperatorMembershipRequest<'binding>,
    authorized: OperatorAuthorizedCandidate<'binding>,
    registry: &mut OperatorRightRegistry,
    run: &str,
) -> Result<ObservedOperatorMembership, OperatorMembershipRefusal> {
    if run.trim().is_empty() {
        return Err(OperatorMembershipRefusal::EmptyRun);
    }
    if request.operation != mapping.operation {
        return Err(OperatorMembershipRefusal::OperationMismatch);
    }
    let binding = request.request.binding();
    if binding.capability_revision() != target.definition().version()
        || binding
            .profile()
            .check_revision(target.definition().version())
            .is_err()
    {
        return Err(OperatorMembershipRefusal::AuthorizationRevision);
    }
    if authorized.request() != &request.request {
        return Err(OperatorMembershipRefusal::AuthorizationMismatch);
    }
    let disposition = if mapping
        .check(target.definition().version(), binding)
        .is_ok()
    {
        OperatorMembershipDisposition::Member
    } else {
        OperatorMembershipDisposition::NonMember
    };
    registry
        .consume(request.right, request.request, |_| Ok(authorized))
        .map_err(OperatorMembershipRefusal::Right)?;
    let operation = *request
        .operation
        .operations()
        .first()
        .ok_or(OperatorMembershipRefusal::OperationScope)?;
    ObservedOperatorMembership::new(
        operation,
        disposition,
        format!("transaction::produce_operator_membership run {run}"),
    )
    .map_err(|_| OperatorMembershipRefusal::Provenance)
}
