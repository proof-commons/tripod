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
    TransactionShape,
    CaptureSubjectCensus,
    ReaderRecurrence,
    MemberMaximum,
}

/// Names the retained computed fields; the extended source inventory binds S6–S8 owners.
pub const COMPUTED: [ComputedItem; 15] = [
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
    ComputedItem::TransactionShape,
    ComputedItem::CaptureSubjectCensus,
    ComputedItem::ReaderRecurrence,
    ComputedItem::MemberMaximum,
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

/// One capture-time state reconstructed from its retained source snapshot.
/// The reviewed identity is independent of the packet that claims it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidatedCaptureHistory {
    identity: [u8; 32],
    parameters: KeyedFixedParameters,
}

impl ValidatedCaptureHistory {
    /// The reconstructed capture-time identity.
    #[must_use]
    pub const fn identity(&self) -> [u8; 32] {
        self.identity
    }

    /// The triple whose descriptor was reconstructed.
    #[must_use]
    pub const fn parameters(&self) -> KeyedFixedParameters {
        self.parameters
    }
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
    /// Decode and authenticate one of the four reviewed campaign freezes.
    /// The two changed source snapshots must come from the retained corpus.
    /// # Errors
    /// Refuses grammar, source, descriptor, envelope or reviewed-identity drift.
    pub fn reconstruct_campaign_freeze(
        encoded: &[u8],
        historical_sources: &[(&str, &[u8])],
    ) -> Result<ValidatedFreeze, FreezeRefusal> {
        let _span = tracing::info_span!(target: "vectors::keyed_fixed_manifest", "reconstruct_campaign_freeze").entered();
        let manifest = encoding::decode(encoded)?;
        let parameters = manifest.descriptor.parameters;
        if historical_sources.len() != 2
            || historical_sources[0].0 != "packages/vectors/src/keyed_fixed_manifest.rs"
            || historical_sources[1].0 != "packages/vectors/src/keyed_fixed_manifest/encoding.rs"
        {
            return Err(FreezeRefusal::Inventory);
        }
        if manifest.sources.len() != pins::SOURCES.len()
            || manifest
                .sources
                .iter()
                .zip(pins::SOURCES)
                .any(|(identity, (path, current, _))| {
                    let bytes = historical_sources
                        .iter()
                        .find_map(|(historical_path, bytes)| {
                            (*historical_path == *path).then_some(*bytes)
                        })
                        .unwrap_or(*current);
                    identity.path != *path || sha256(bytes) != identity.digest
                })
        {
            return Err(FreezeRefusal::Inventory);
        }
        let expected = Self::offer(parameters)?;
        if manifest.descriptor != expected.descriptor {
            return Err(FreezeRefusal::Descriptor);
        }
        if manifest.recipe_schema != expected.recipe_schema {
            return Err(FreezeRefusal::Recipe);
        }
        if manifest.target_version != expected.target_version {
            return Err(FreezeRefusal::Target);
        }
        if manifest.envelope != expected.envelope {
            return Err(FreezeRefusal::Envelope);
        }
        let reencoded = encoding::encode(&manifest)?;
        if reencoded != encoded {
            return Err(FreezeRefusal::Framing);
        }
        let identity = crate::maturity_observer::encode_hex(&sha256(encoded));
        if !pins::CAMPAIGN_IDENTITIES.contains(&identity.as_str()) {
            return Err(FreezeRefusal::Inventory);
        }
        tracing::info!(target: "vectors::keyed_fixed_manifest", sources = manifest.sources.len(), encoded_bytes = encoded.len(), outcome = "completed", "campaign freeze authenticated");
        Ok(ValidatedFreeze {
            manifest,
            encoding: reencoded,
        })
    }

    /// Reconstruct a historical freeze from the retained source bytes and a
    /// separately reviewed identity, without substituting today's inventory.
    /// # Errors
    /// Refuses source bytes, triple, recipe or independently reviewed identity drift.
    pub fn reconstruct_capture_history(
        &self,
        parameters: KeyedFixedParameters,
        sources: &[(String, Vec<u8>)],
        reviewed_identity: [u8; 32],
    ) -> Result<ValidatedCaptureHistory, FreezeRefusal> {
        if sources.len() != self.sources.len()
            || sources
                .iter()
                .zip(&self.sources)
                .any(|((path, bytes), identity)| {
                    path != &identity.path || sha256(bytes) != identity.digest
                })
        {
            return Err(FreezeRefusal::Inventory);
        }
        if self.descriptor.parameters != parameters {
            return Err(FreezeRefusal::Triple);
        }
        // A later producer may change the descriptor, code list or envelope.
        // The reviewed capture-time identity binds their exact historical bytes.
        if self.recipe_schema != 1 {
            return Err(FreezeRefusal::Recipe);
        }
        if self.target_version != 2 {
            return Err(FreezeRefusal::Target);
        }
        let identity = sha256(&encoding::encode(self)?);
        if identity != reviewed_identity {
            return Err(FreezeRefusal::Inventory);
        }
        Ok(ValidatedCaptureHistory {
            identity,
            parameters,
        })
    }

    /// Build an offer from current pins and public parameters; not yet validated.
    /// # Errors
    /// Refuses an unavailable reviewed target or unsuccessful commitment.
    pub fn offer(parameters: KeyedFixedParameters) -> Result<Self, FreezeRefusal> {
        let target = target_elements::reviewed_elements_tapscript()
            .map_err(|_| FreezeRefusal::Construction)?;
        let descriptor = construct(&target, parameters, &PublicKeyedCurve)
            .map_err(|_| FreezeRefusal::Construction)?;
        let maximum =
            transaction::keyed_fixed_construct::keyed_fixed_policy_member_maximum(&target)
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
                maximum_members: Some(
                    u64::try_from(maximum).map_err(|_| FreezeRefusal::Construction)?,
                ),
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
