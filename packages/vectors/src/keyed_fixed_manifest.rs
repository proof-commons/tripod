//! Separate fixed-form selected-source provenance freeze, not native admission.
//!
//! Whole source bytes and path order are content. Build/runtime/toolchain facts
//! and the resolved dependency lock remain S8 capture provenance, not this freeze.

use crate::keyed_fixed_constructor::{KeyedFixedDescriptor, PublicKeyedCurve, construct};
use realization::keyed_fixed::KeyedFixedParameters;
use target_elements_conformance::constructor::tagged::sha256;

mod encoding;
mod pins;

/// The separate provenance recipe; it never names the semantic manifest.
pub const RECIPE: &str = "tripod-keyed-fixed-capture-freeze-v1";

/// Whole-file source pin in canonical path order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceIdentity {
    pub path: String,
    pub digest: [u8; 32],
}

/// Runtime values available through S5, with explicit stable encoding codes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComputedItem {
    Triple,
    Transcript,
    PairDigest,
    Leaf,
    Root,
    Tweak,
    Program,
    Control,
    EmissionAndClosure,
    MemberCoverage,
    DerivedResources,
}

/// No S6 transaction, S7 reading or S8 capture value is claimed frozen here.
pub const COMPUTED: [ComputedItem; 11] = [
    ComputedItem::Triple,
    ComputedItem::Transcript,
    ComputedItem::PairDigest,
    ComputedItem::Leaf,
    ComputedItem::Root,
    ComputedItem::Tweak,
    ComputedItem::Program,
    ComputedItem::Control,
    ComputedItem::EmissionAndClosure,
    ComputedItem::MemberCoverage,
    ComputedItem::DerivedResources,
];

/// Derived structural envelope. A missing ceiling means unadmitted, not infinity.
#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools)] // Four independently encoded constructor-envelope premises.
pub struct Envelope {
    pub maximum_members: Option<u64>,
    pub script_bytes: u64,
    pub script_witness_bytes: u64,
    pub input_witness_bytes: u64,
    pub control_depth: u64,
    pub argument_items: u64,
    pub annex: bool,
    pub issuance: bool,
    pub peg_in: bool,
    pub empty_nonce: bool,
}

/// Untrusted complete offer. Source pins check included build-time source bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FreezeManifest {
    pub sources: Vec<SourceIdentity>,
    pub computed: Vec<ComputedItem>,
    pub descriptor: KeyedFixedDescriptor,
    pub recipe_schema: u32,
    pub target_version: u32,
    pub envelope: Envelope,
}

/// Typed validation refusals; no field digest is an independent identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FreezeRefusal {
    SourceDrift(String),
    Inventory,
    Alphabet,
    Framing,
    Construction,
    Triple,
    Descriptor,
    Recipe,
    Target,
    Envelope,
}

/// Only validation creates this subject; only it publishes a provenance identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidatedFreeze {
    manifest: FreezeManifest,
    encoding: Vec<u8>,
}

impl ValidatedFreeze {
    /// Immutable complete validated subject.
    #[must_use]
    pub const fn manifest(&self) -> &FreezeManifest {
        &self.manifest
    }
    /// Canonical preimage, including the distinct recipe domain.
    #[must_use]
    pub fn encoding(&self) -> &[u8] {
        &self.encoding
    }
    /// SHA-256 provenance identity, conditional on the reviewed primitive.
    #[must_use]
    pub fn identity(&self) -> [u8; 32] {
        sha256(&self.encoding)
    }
}

impl FreezeManifest {
    /// Build an offer from current pins and public parameters; not yet validated.
    /// # Errors
    /// Refuses an unavailable reviewed target or unsuccessful commitment.
    pub fn offer(parameters: KeyedFixedParameters) -> Result<Self, FreezeRefusal> {
        let target = target_elements::reviewed_elements_tapscript()
            .map_err(|_| FreezeRefusal::Construction)?;
        let descriptor = construct(&target, parameters, &PublicKeyedCurve)
            .map_err(|_| FreezeRefusal::Construction)?;
        Ok(Self {
            sources: pins::SOURCES
                .iter()
                .map(|(path, _, digest)| SourceIdentity {
                    path: (*path).into(),
                    digest: *digest,
                })
                .collect(),
            computed: COMPUTED.to_vec(),
            descriptor,
            recipe_schema: 1,
            target_version: target.definition().version().get(),
            envelope: Envelope {
                maximum_members: None,
                script_bytes: 104,
                script_witness_bytes: 140,
                input_witness_bytes: 143,
                control_depth: 0,
                argument_items: 0,
                annex: false,
                issuance: false,
                peg_in: false,
                empty_nonce: true,
            },
        })
    }

    /// Validate included source bytes, inventories and the independently supplied triple.
    /// # Errors
    /// Returns source drift before inventory, binding, recipe or envelope defects.
    pub fn validate(
        self,
        parameters: KeyedFixedParameters,
    ) -> Result<ValidatedFreeze, FreezeRefusal> {
        let _span =
            tracing::debug_span!(target: "vectors::keyed_fixed_manifest", "validate").entered();
        let result = self.validate_core(parameters);
        tracing::debug!(target: "vectors::keyed_fixed_manifest", outcome = if result.is_ok() { "completed" } else { "refused" }, "fixed-form freeze validated");
        result
    }

    fn validate_core(
        self,
        parameters: KeyedFixedParameters,
    ) -> Result<ValidatedFreeze, FreezeRefusal> {
        for (path, bytes, digest) in pins::SOURCES {
            if sha256(bytes) != *digest {
                return Err(FreezeRefusal::SourceDrift((*path).into()));
            }
        }
        let encoding = encoding::encode(&self)?;
        let expected = Self::offer(parameters)?;
        if self.sources != expected.sources || self.computed != expected.computed {
            return Err(FreezeRefusal::Inventory);
        }
        if self.descriptor.parameters != parameters {
            return Err(FreezeRefusal::Triple);
        }
        if self.descriptor != expected.descriptor {
            return Err(FreezeRefusal::Descriptor);
        }
        if self.recipe_schema != expected.recipe_schema {
            return Err(FreezeRefusal::Recipe);
        }
        if self.target_version != expected.target_version {
            return Err(FreezeRefusal::Target);
        }
        if self.envelope != expected.envelope {
            return Err(FreezeRefusal::Envelope);
        }
        Ok(ValidatedFreeze {
            manifest: self,
            encoding,
        })
    }
}
