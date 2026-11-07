//! Pinned four-file admission and planner replay of maturity announcement runs.
//!
//! Names, sizes and addresses bind each archive before its closed grammars are interpreted. The historical refused loader is one set of pins; the second immutable corpus binds a variable-metadata acceptance with its own schedule and sponsorless expectation. Each corpus also pins the cargo path, framework revision mode and funding advertisement of the host that ran it.
//!
//! An accepted payload carries mined readback beside the capture's request bytes, verdict, layer, target identity and detail. Replay checks the reconstructed response against the submitted transaction. This establishes transcript consistency under the schedule, not executor provenance, current-root freshness or a promoted standing.
//!
//! Two constructor-mutant archives use the same byte bindings and replay each recorded offer through the mutant ceremony planner. Their report phase walls are recorded facts, while the replayed offer records establish only the submitted requests and observed responses under the pinned target.

use crate::maturity_history::{MaturityRootCheckpointRefusal, StateRootEdge};
use crate::maturity_recovery::PublicAnnouncementHandoff;
use crate::maturity_recovery_report::{
    MaturityAcceptedHandoffStage, MaturityPublicRecoveryReportRefusal,
};
use linker::CandidateLinkedMaturityBundle;
use std::fmt::Write as _;
use std::sync::OnceLock;
use transaction::state_abi::CandidateMaturityAnnouncementAbi;

use linker::CandidateDeploymentIdentity;
use target_elements_conformance::constructor::tagged;
use target_elements_conformance::executor::OperationStep;
use target_elements_conformance::protocol::{
    MinedFundingReadback, NativeOperationResponse, ObservedOutcomeLayer, OperationSubject,
    TargetFundingSubject, TargetSubmissionSubject,
};
use transaction::bytes::Txid;
use transaction::operator_right::BranchContext;

use crate::maturity_closure::MaturityWitnessSelection;
use crate::maturity_mutant_ceremony::{self, MaturityMutantEvidence};
use crate::maturity_mutants::MaturityMutantTable;
use crate::maturity_native::{
    ANNOUNCEMENT_STEPS, MaturityAcceptanceObligation, MaturityAcceptanceRoute,
    MaturityNativeEvidence, MaturityNativePlanRefusal, MaturityNativeStanding,
};

/// The archive's witness schedule, established by exact replay of all three
/// recorded requests rather than selected as a deployment preference.
pub const MATURITY_RUN_SCHEDULE: tapscript::StateWitnessSchedule =
    tapscript::StateWitnessSchedule::WholeMetadata;

/// SHA-256 of the exact two-entry evidence manifest.
pub const MATURITY_MANIFEST_SHA256: &str =
    "66578a5b6e5aa150827c8e5c20a55003a0a21ddfbc1ba08b48943bd26d57b7e1";
/// SHA-256 of the report binding the complete maturity run.
pub const MATURITY_RUN_ADDRESS: &str =
    "fb2912ecb8e8b3a6ba8af7839167e644f6398fe7a296f89ff9fc757be83b31f3";

/// The witness schedule established by replay of the accepted archive.
pub const MATURITY_VARIABLE_RUN_SCHEDULE: tapscript::StateWitnessSchedule =
    tapscript::StateWitnessSchedule::VariableMetadata;
/// SHA-256 of the accepted archive's exact two-entry manifest.
pub const MATURITY_VARIABLE_MANIFEST_SHA256: &str =
    "a59b750c459c95d0b72fe2e77c45c6e959c1407c702ce13b9e4a3ba0c487dece";
/// SHA-256 of the accepted archive's report.
pub const MATURITY_VARIABLE_RUN_ADDRESS: &str =
    "b624e245fcdfe839ff9bb3e94285165ab0c9fbdd606748fd115ae8b02dc87b0c";

/// SHA-256 of the predecessor mutant archive's manifest.
pub const MATURITY_PREDECESSOR_MUTANT_MANIFEST_SHA256: &str =
    "46ba2aa5cc29720a9a8fced865cd46876dc51de985f47642f05e8571c50cc883";
/// SHA-256 of the predecessor mutant archive's report.
pub const MATURITY_PREDECESSOR_MUTANT_RUN_ADDRESS: &str =
    "26793148fc96436e2267ab5b39382a0bb38d23551c799e6c89571b71a1da63f0";
/// SHA-256 of the successor mutant archive's manifest.
pub const MATURITY_SUCCESSOR_MUTANT_MANIFEST_SHA256: &str =
    "a6567865310a1c1b6427e3c51f187caf383af05f4554f4d74bc53fe6b6bc7c98";
/// SHA-256 of the successor mutant archive's report.
pub const MATURITY_SUCCESSOR_MUTANT_RUN_ADDRESS: &str =
    "206c8e1262af5c1ea8ef541e9753961df15101884cc6ee451957019cdccca1a7";

const CEREMONY_SHA256: &str = "d48e105b85708cd8edadbe423ed020c48fdbe1c0225f67724ff6f68449d25515";
/// SHA-256 of the sponsored archive's two-entry manifest.
pub const MATURITY_SPONSORED_MANIFEST_SHA256: &str =
    "eac4f606524058264eb1920c274aeecfae7f2ebcce7e1c089789f2177d090a14";
/// SHA-256 of the sponsored report binding policy and per-subject observations.
pub const MATURITY_SPONSORED_RUN_ADDRESS: &str =
    "c78d141a6f236a7bd9006b6bbba77cff0055c7caf7eb672bb3341be112c0a962";
const MATURITY_SPONSORED_CEREMONY_SHA256: &str =
    "5299269816ae154ef0cd5b84edc23002953a4daa447294a4ebff6a3d3e0d6e94";
/// SHA-256 of the native observation manifest.
pub const MATURITY_NATIVE_MANIFEST_SHA256: &str =
    "75283f89a898d12559686ea03259c741e4e84b4fcdec3e489fe1c9ac760bcd52";
/// SHA-256 of the native observation report, including measured phase durations.
pub const MATURITY_NATIVE_RUN_ADDRESS: &str =
    "a0232585c887fcaf8ac4fd7050e77258bf7a6e0aebba7fce7d67c829374682f8";
const MATURITY_NATIVE_CEREMONY_SHA256: &str =
    "707adc52ee34f394fde4d95f9c5f5c5ec5fac48f051694a760b05afc4abbcae1";
/// SHA-256 of the diagnostic archive's exact manifest.
pub const MATURITY_DIAGNOSTIC_MANIFEST_SHA256: &str =
    "f041eb5d02b95f71829acbe966bf4eef4dbe9a738514f3cf09f10f545765faee";
/// SHA-256 of the original diagnostic report bytes.
pub const MATURITY_DIAGNOSTIC_RUN_ADDRESS: &str =
    "a7159fdc16c5a2a7bb8f9ce5011fe014377d481023e14e2acb714461a47a6cbc";
const MATURITY_DIAGNOSTIC_CEREMONY_SHA256: &str =
    "f27b3f710e495cf8e193ff062ca7b5304f3f6236c9bfadda268d91ace7caffe2";

/// SHA-256 of the independent observation manifest.
pub const MATURITY_OBSERVER_MANIFEST_SHA256: &str =
    "ef34c7d29b8bf6c54a9b23fbbd072ae97d11533fd47aafddf996c88edb0f57e8";
/// SHA-256 of the independent observation report and measured phase durations.
pub const MATURITY_OBSERVER_RUN_ADDRESS: &str =
    "e003e7d24e6edcad193561ce88a7da01ddf134269629930d9e6ba8bf862946dc";
/// SHA-256 of the independent acquisition ceremony.
pub const MATURITY_OBSERVER_CEREMONY_SHA256: &str =
    "bbf0f7cefdd412cde6e6705b8c73d3084949e694279df046550762251951582b";

const OBSERVER_FILES: [MaturityArchiveFile<'_>; 4] = [
    MaturityArchiveFile::new(
        "94b380dd04.report.capture",
        include_bytes!("../fixtures/maturity-observer-run-of-record/94b380dd04.report.capture"),
    ),
    MaturityArchiveFile::new(
        "94b380dd04.report.capture.timing",
        include_bytes!(
            "../fixtures/maturity-observer-run-of-record/94b380dd04.report.capture.timing"
        ),
    ),
    MaturityArchiveFile::new(
        "MANIFEST.sha256",
        include_bytes!("../fixtures/maturity-observer-run-of-record/MANIFEST.sha256"),
    ),
    MaturityArchiveFile::new(
        "RUN-REPORT",
        include_bytes!("../fixtures/maturity-observer-run-of-record/RUN-REPORT"),
    ),
];
const OBSERVER_PINS: [&str; 3] = [
    MATURITY_OBSERVER_MANIFEST_SHA256,
    MATURITY_OBSERVER_RUN_ADDRESS,
    MATURITY_OBSERVER_CEREMONY_SHA256,
];
const PREDECESSOR_MUTANT_CEREMONY_SHA256: &str =
    "730a0670d76a59e4640726ef54f0d40853987c35792a4329401abd485d42f319";
const SUCCESSOR_MUTANT_CEREMONY_SHA256: &str =
    "fd935b26cb1ea57943834473a83398249bd772aca8ab3e8840779624da3016e5";

/// One named file supplied to the four-file maturity corpus admission.
#[derive(Clone, Copy)]
pub struct MaturityArchiveFile<'a> {
    name: &'a str,
    bytes: &'a [u8],
}

impl<'a> MaturityArchiveFile<'a> {
    /// Binds a file name to its exact bytes for admission.
    #[must_use]
    pub const fn new(name: &'a str, bytes: &'a [u8]) -> Self {
        Self { name, bytes }
    }

    /// The name used by the ordered census and manifest.
    #[must_use]
    pub const fn name(&self) -> &'a str {
        self.name
    }

    /// The exact bytes to admit.
    #[must_use]
    pub const fn bytes(&self) -> &'a [u8] {
        self.bytes
    }
}

const FILES: [MaturityArchiveFile<'static>; 4] = [
    MaturityArchiveFile {
        name: "3a690139.report.capture",
        bytes: include_bytes!("../fixtures/maturity-run-of-record/3a690139.report.capture"),
    },
    MaturityArchiveFile {
        name: "3a690139.report.capture.timing",
        bytes: include_bytes!("../fixtures/maturity-run-of-record/3a690139.report.capture.timing"),
    },
    MaturityArchiveFile {
        name: "MANIFEST.sha256",
        bytes: include_bytes!("../fixtures/maturity-run-of-record/MANIFEST.sha256"),
    },
    MaturityArchiveFile {
        name: "RUN-REPORT",
        bytes: include_bytes!("../fixtures/maturity-run-of-record/RUN-REPORT"),
    },
];
const FILE_SIZES: [usize; 4] = [31_153, 49, 187, 1_927];

const VARIABLE_FILES: [MaturityArchiveFile<'static>; 4] = [
    MaturityArchiveFile {
        name: "cae3bd7c.report.capture",
        bytes: include_bytes!(
            "../fixtures/maturity-variable-run-of-record/cae3bd7c.report.capture"
        ),
    },
    MaturityArchiveFile {
        name: "cae3bd7c.report.capture.timing",
        bytes: include_bytes!(
            "../fixtures/maturity-variable-run-of-record/cae3bd7c.report.capture.timing"
        ),
    },
    MaturityArchiveFile {
        name: "MANIFEST.sha256",
        bytes: include_bytes!("../fixtures/maturity-variable-run-of-record/MANIFEST.sha256"),
    },
    MaturityArchiveFile {
        name: "RUN-REPORT",
        bytes: include_bytes!("../fixtures/maturity-variable-run-of-record/RUN-REPORT"),
    },
];

#[rustfmt::skip]
const PREDECESSOR_MUTANT_FILES: [MaturityArchiveFile<'static>; 4] = [
    MaturityArchiveFile {
        name: "f933b376.report.capture",
        bytes: include_bytes!("../fixtures/maturity-predecessor-mutant-run-of-record/f933b376.report.capture"),
    },
    MaturityArchiveFile {
        name: "f933b376.report.capture.timing",
        bytes: include_bytes!("../fixtures/maturity-predecessor-mutant-run-of-record/f933b376.report.capture.timing"),
    },
    MaturityArchiveFile {
        name: "MANIFEST.sha256",
        bytes: include_bytes!("../fixtures/maturity-predecessor-mutant-run-of-record/MANIFEST.sha256"),
    },
    MaturityArchiveFile {
        name: "RUN-REPORT",
        bytes: include_bytes!("../fixtures/maturity-predecessor-mutant-run-of-record/RUN-REPORT"),
    },
];
#[rustfmt::skip]
const SUCCESSOR_MUTANT_FILES: [MaturityArchiveFile<'static>; 4] = [
    MaturityArchiveFile {
        name: "f933b376.report.capture",
        bytes: include_bytes!("../fixtures/maturity-successor-mutant-run-of-record/f933b376.report.capture"),
    },
    MaturityArchiveFile {
        name: "f933b376.report.capture.timing",
        bytes: include_bytes!("../fixtures/maturity-successor-mutant-run-of-record/f933b376.report.capture.timing"),
    },
    MaturityArchiveFile {
        name: "MANIFEST.sha256",
        bytes: include_bytes!("../fixtures/maturity-successor-mutant-run-of-record/MANIFEST.sha256"),
    },
    MaturityArchiveFile {
        name: "RUN-REPORT",
        bytes: include_bytes!("../fixtures/maturity-successor-mutant-run-of-record/RUN-REPORT"),
    },
];

/// The Rust test target and test name bound by one capture.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MaturityCaptureTarget<'a> {
    target: &'a str,
    test: &'a str,
}

impl<'a> MaturityCaptureTarget<'a> {
    /// Declares both executable test identities.
    #[must_use]
    pub const fn new(target: &'a str, test: &'a str) -> Self {
        Self { target, test }
    }

    /// The Cargo test target.
    #[must_use]
    pub const fn target(&self) -> &'a str {
        self.target
    }

    /// The Rust test name carried by the capture.
    #[must_use]
    pub const fn test(&self) -> &'a str {
        self.test
    }
}

/// The expected outcome of the sponsorless operation in a pinned corpus.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MaturitySponsorlessExpectation {
    /// A relay refusal with its exact recorded response detail.
    RelayRefusal { detail: &'static str },
    /// An accepted submission with a mined readback carried in the payload.
    Acceptance,
}

/// How the capture binds the framework revision to its run report.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MaturityFrameworkRevision {
    /// The framework revision was recorded as empty text.
    Unrecorded,
    /// The framework revision equals the report's intended executed tip.
    IntendedTip,
}

pub use crate::maturity_history::{MaturityDeclaredPremise, MaturityPremiseProvenance};

/// Fee floors disclosed for the node that ran an admitted archive.
///
/// Historical reports carry no fee line and retain executor-argument premises.
/// The sponsored report binds a reported relay floor and a separately declared
/// block default; admission checks both values and their distinct provenances.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MaturityDisclosedFeeFloors {
    min_relay_tx_fee: &'static str,
    block_min_tx_fee: &'static str,
    provenance: MaturityPremiseProvenance,
    block_provenance: MaturityPremiseProvenance,
}

impl MaturityDisclosedFeeFloors {
    /// Declares the two executor node arguments without attributing them to archive bytes.
    #[must_use]
    pub const fn declared(min_relay_tx_fee: &'static str, block_min_tx_fee: &'static str) -> Self {
        Self {
            min_relay_tx_fee,
            block_min_tx_fee,
            provenance: MaturityPremiseProvenance::ExecutorArguments,
            block_provenance: MaturityPremiseProvenance::ExecutorArguments,
        }
    }

    /// Retains distinct sources for the default-policy observation.
    #[must_use]
    pub const fn node_defaults(
        min_relay_tx_fee: &'static str,
        block_min_tx_fee: &'static str,
    ) -> Self {
        Self {
            min_relay_tx_fee,
            block_min_tx_fee,
            provenance: MaturityPremiseProvenance::ReportedByNode,
            block_provenance: MaturityPremiseProvenance::PinnedSourceNoOverride,
        }
    }

    /// Source of the block assembly floor.
    #[must_use]
    pub const fn block_provenance(&self) -> MaturityPremiseProvenance {
        self.block_provenance
    }

    /// The declared minimum relay transaction fee argument.
    #[must_use]
    pub const fn min_relay_tx_fee(&self) -> &'static str {
        self.min_relay_tx_fee
    }

    /// The declared minimum block transaction fee argument.
    #[must_use]
    pub const fn block_min_tx_fee(&self) -> &'static str {
        self.block_min_tx_fee
    }

    /// Where the two declared values came from.
    #[must_use]
    pub const fn provenance(&self) -> MaturityPremiseProvenance {
        self.provenance
    }
}

/// The fee arguments disclosed for both admitted runs of the native executor.
pub const MATURITY_DISCLOSED_FEE_FLOORS: MaturityDisclosedFeeFloors =
    MaturityDisclosedFeeFloors::declared("0", "0");

/// The addresses and declaration that bind one immutable four-file corpus.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MaturityCorpusPins<'a> {
    schedule: tapscript::StateWitnessSchedule,
    names: [&'a str; 4],
    sizes: [usize; 4],
    manifest_sha256: &'a str,
    run_address: &'a str,
    ceremony_sha256: &'a str,
    sponsorless: MaturitySponsorlessExpectation,
    capture_target: MaturityCaptureTarget<'a>,
    cargo_path: &'a str,
    framework_revision: MaturityFrameworkRevision,
    funding_count: usize,
    capabilities: &'a [&'a str],
    funding: &'a [&'a str],
    fee_floors: MaturityDisclosedFeeFloors,
}

impl<'a> MaturityCorpusPins<'a> {
    /// Collects the ordered file pins, addresses, sponsorless declaration and host premises.
    #[must_use]
    #[expect(
        clippy::too_many_arguments,
        reason = "all nine pins are declared at every call site rather than defaulted"
    )]
    pub const fn new(
        schedule: tapscript::StateWitnessSchedule,
        files: ([&'a str; 4], [usize; 4]),
        addresses: (&'a str, &'a str, &'a str),
        sponsorless: MaturitySponsorlessExpectation,
        capture_target: MaturityCaptureTarget<'a>,
        cargo_path: &'a str,
        framework_revision: MaturityFrameworkRevision,
        funding_count: usize,
        fee_floors: MaturityDisclosedFeeFloors,
    ) -> Self {
        let (names, sizes) = files;
        let (manifest_sha256, run_address, ceremony_sha256) = addresses;
        Self {
            schedule,
            names,
            sizes,
            manifest_sha256,
            run_address,
            ceremony_sha256,
            sponsorless,
            capture_target,
            cargo_path,
            framework_revision,
            funding_count,
            capabilities: &CAPABILITIES,
            funding: &[],
            fee_floors,
        }
    }

    /// The schedule used for exact request replay.
    #[must_use]
    pub const fn schedule(&self) -> tapscript::StateWitnessSchedule {
        self.schedule
    }

    /// The ordered file names.
    #[must_use]
    pub const fn names(&self) -> &[&'a str; 4] {
        &self.names
    }

    /// The ordered file lengths.
    #[must_use]
    pub const fn sizes(&self) -> &[usize; 4] {
        &self.sizes
    }

    /// SHA-256 of the manifest bytes.
    #[must_use]
    pub const fn manifest_sha256(&self) -> &'a str {
        self.manifest_sha256
    }

    /// SHA-256 of the report bytes.
    #[must_use]
    pub const fn run_address(&self) -> &'a str {
        self.run_address
    }

    /// SHA-256 of the capture bytes.
    #[must_use]
    pub const fn ceremony_sha256(&self) -> &'a str {
        self.ceremony_sha256
    }

    /// The declared sponsorless outcome grammar.
    #[must_use]
    pub const fn sponsorless(&self) -> MaturitySponsorlessExpectation {
        self.sponsorless
    }

    /// The Rust target and test admitted by the archive.
    #[must_use]
    pub const fn capture_target(&self) -> MaturityCaptureTarget<'a> {
        self.capture_target
    }

    /// The executable path recorded at the start of the report's cargo argv.
    #[must_use]
    pub const fn cargo_path(&self) -> &'a str {
        self.cargo_path
    }

    /// The framework revision binding used by the capture handshake.
    #[must_use]
    pub const fn framework_revision(&self) -> MaturityFrameworkRevision {
        self.framework_revision
    }

    /// The number of funding advertisements in the capture environment.
    #[must_use]
    pub const fn funding_count(&self) -> usize {
        self.funding_count
    }

    /// The declared fee floors under which the executor ran this archive.
    #[must_use]
    pub const fn fee_floors(&self) -> MaturityDisclosedFeeFloors {
        self.fee_floors
    }
}

#[rustfmt::skip]
const HISTORICAL_PINS: MaturityCorpusPins<'static> = MaturityCorpusPins::new(
    MATURITY_RUN_SCHEDULE,
    (
        [
            "3a690139.report.capture",
            "3a690139.report.capture.timing",
            "MANIFEST.sha256",
            "RUN-REPORT",
        ],
        FILE_SIZES,
    ),
    (
        MATURITY_MANIFEST_SHA256,
        MATURITY_RUN_ADDRESS,
        CEREMONY_SHA256,
    ),
    MaturitySponsorlessExpectation::RelayRefusal {
        detail: "bad-witness-nonstandard",
    },
    MaturityCaptureTarget::new("maturity_native", "the_maturity_announcement_runs_against_a_real_target"),
    "/workspace/toolchains/cargo/bin/cargo",
    MaturityFrameworkRevision::Unrecorded,
    0,
    MATURITY_DISCLOSED_FEE_FLOORS,
);

#[rustfmt::skip]
const VARIABLE_PINS: MaturityCorpusPins<'static> = MaturityCorpusPins::new(
    MATURITY_VARIABLE_RUN_SCHEDULE,
    (
        [
            "cae3bd7c.report.capture",
            "cae3bd7c.report.capture.timing",
            "MANIFEST.sha256",
            "RUN-REPORT",
        ],
        [55_271, 49, 187, 1_927],
    ),
    (
        MATURITY_VARIABLE_MANIFEST_SHA256,
        MATURITY_VARIABLE_RUN_ADDRESS,
        "fa474763a165c60d4e48b7c0c06f01ce79204a1fc4f9020bd7612c82e22bb3ee",
    ),
    MaturitySponsorlessExpectation::Acceptance,
    MaturityCaptureTarget::new("maturity_native", "the_maturity_announcement_runs_against_a_real_target"),
    "/workspace/toolchains/cargo/bin/cargo",
    MaturityFrameworkRevision::IntendedTip,
    0,
    MATURITY_DISCLOSED_FEE_FLOORS,
);

/// The sponsored archive's four immutable files and independent pins.
const SPONSORED_FILES: [MaturityArchiveFile<'static>; 4] = [
    MaturityArchiveFile::new(
        "17280cedd2.report.capture",
        include_bytes!("../fixtures/maturity-sponsored-run-of-record/17280cedd2.report.capture"),
    ),
    MaturityArchiveFile::new(
        "17280cedd2.report.capture.timing",
        include_bytes!(
            "../fixtures/maturity-sponsored-run-of-record/17280cedd2.report.capture.timing"
        ),
    ),
    MaturityArchiveFile::new(
        "MANIFEST.sha256",
        include_bytes!("../fixtures/maturity-sponsored-run-of-record/MANIFEST.sha256"),
    ),
    MaturityArchiveFile::new(
        "RUN-REPORT",
        include_bytes!("../fixtures/maturity-sponsored-run-of-record/RUN-REPORT"),
    ),
];
const SPONSORED_PINS: MaturityCorpusPins<'static> = MaturityCorpusPins {
    names: [
        "17280cedd2.report.capture",
        "17280cedd2.report.capture.timing",
        "MANIFEST.sha256",
        "RUN-REPORT",
    ],
    sizes: [2_975_354, 49, 191, 204_336],
    manifest_sha256: MATURITY_SPONSORED_MANIFEST_SHA256,
    run_address: MATURITY_SPONSORED_RUN_ADDRESS,
    ceremony_sha256: MATURITY_SPONSORED_CEREMONY_SHA256,
    fee_floors: MaturityDisclosedFeeFloors::node_defaults("0.00000100", "0.00000100"),
    ..VARIABLE_PINS
};

/// SHA-256 of the retained archive's exact manifest.
pub const MATURITY_RETAINED_MANIFEST_SHA256: &str =
    "a94537093fae80f8053a67b60e00658fb0de0ccfb9fe8c286b3e8ea4737a690d";
/// SHA-256 of the retained report, including its campaign manifest.
pub const MATURITY_RETAINED_RUN_ADDRESS: &str =
    "18d9e05ed1bdc2e63a5fac65cdc4a1345db9b9f8ab2cf2c4abf074796ad76c6f";
const MATURITY_RETAINED_CEREMONY_SHA256: &str =
    "921a32c9201397b792133273c4e8a6c275271ac94a3a6f262c3ddd8010196923";
const RETAINED_NODE_TIP: &str = "b7fc5d080a7e9ccc0ef48c3ba11db243e794bdb0";
const RETAINED_FILES: [MaturityArchiveFile<'static>; 4] = [
    MaturityArchiveFile::new(
        "4558d26aba.report.capture",
        include_bytes!("../fixtures/maturity-retained-run-of-record/4558d26aba.report.capture"),
    ),
    MaturityArchiveFile::new(
        "4558d26aba.report.capture.timing",
        include_bytes!(
            "../fixtures/maturity-retained-run-of-record/4558d26aba.report.capture.timing"
        ),
    ),
    MaturityArchiveFile::new(
        "MANIFEST.sha256",
        include_bytes!("../fixtures/maturity-retained-run-of-record/MANIFEST.sha256"),
    ),
    MaturityArchiveFile::new(
        "RUN-REPORT",
        include_bytes!("../fixtures/maturity-retained-run-of-record/RUN-REPORT"),
    ),
];
const RETAINED_PINS: MaturityCorpusPins<'static> = MaturityCorpusPins {
    names: [
        "4558d26aba.report.capture",
        "4558d26aba.report.capture.timing",
        "MANIFEST.sha256",
        "RUN-REPORT",
    ],
    sizes: [382_918, 49, 191, 17_376],
    manifest_sha256: MATURITY_RETAINED_MANIFEST_SHA256,
    run_address: MATURITY_RETAINED_RUN_ADDRESS,
    ceremony_sha256: MATURITY_RETAINED_CEREMONY_SHA256,
    capture_target: MaturityCaptureTarget::new(
        "maturity_retained",
        "the_retained_maturity_routes_run_against_a_real_target",
    ),
    ..NATIVE_PINS
};

const DIAGNOSTIC_FILES: [MaturityArchiveFile<'static>; 4] = [
    MaturityArchiveFile::new(
        "ae053ac43a.report.capture",
        include_bytes!("../fixtures/maturity-diagnostic-run-of-record/ae053ac43a.report.capture"),
    ),
    MaturityArchiveFile::new(
        "ae053ac43a.report.capture.timing",
        include_bytes!(
            "../fixtures/maturity-diagnostic-run-of-record/ae053ac43a.report.capture.timing"
        ),
    ),
    MaturityArchiveFile::new(
        "MANIFEST.sha256",
        include_bytes!("../fixtures/maturity-diagnostic-run-of-record/MANIFEST.sha256"),
    ),
    MaturityArchiveFile::new(
        "RUN-REPORT",
        include_bytes!("../fixtures/maturity-diagnostic-run-of-record/RUN-REPORT"),
    ),
];
const DIAGNOSTIC_PINS: MaturityCorpusPins<'static> = MaturityCorpusPins {
    names: [
        "ae053ac43a.report.capture",
        "ae053ac43a.report.capture.timing",
        "MANIFEST.sha256",
        "RUN-REPORT",
    ],
    sizes: [2_321_183, 49, 191, 151_862],
    manifest_sha256: MATURITY_DIAGNOSTIC_MANIFEST_SHA256,
    run_address: MATURITY_DIAGNOSTIC_RUN_ADDRESS,
    ceremony_sha256: MATURITY_DIAGNOSTIC_CEREMONY_SHA256,
    capture_target: MaturityCaptureTarget::new(
        "maturity_diagnostic",
        "the_diagnostic_maturity_routes_run_against_a_real_target",
    ),
    funding_count: 4,
    capabilities: &CONFIDENTIAL_CAPABILITIES,
    funding: &CONFIDENTIAL_FUNDING,
    ..NATIVE_PINS
};

/// SHA-256 of the confidential archive's exact manifest.
pub const MATURITY_CONFIDENTIAL_MANIFEST_SHA256: &str =
    "f8eafa447f983af9fdae3258bd38f40eb37325d17654478fc03bfdacd03cd230";
/// SHA-256 of the confidential report, including its campaign manifest.
pub const MATURITY_CONFIDENTIAL_RUN_ADDRESS: &str =
    "5322aaa214c06593c431023f68775b1b6c946556877c172fb88589aeb0073905";
const MATURITY_CONFIDENTIAL_CEREMONY_SHA256: &str =
    "98684c49369aefc388e4d128aa53906b195676dbc31b18d30b1132fea61441d7";
const CONFIDENTIAL_FILES: [MaturityArchiveFile<'static>; 4] = [
    MaturityArchiveFile::new(
        "2b0d2f31c8.report.capture",
        include_bytes!("../fixtures/maturity-confidential-run-of-record/2b0d2f31c8.report.capture"),
    ),
    MaturityArchiveFile::new(
        "2b0d2f31c8.report.capture.timing",
        include_bytes!(
            "../fixtures/maturity-confidential-run-of-record/2b0d2f31c8.report.capture.timing"
        ),
    ),
    MaturityArchiveFile::new(
        "MANIFEST.sha256",
        include_bytes!("../fixtures/maturity-confidential-run-of-record/MANIFEST.sha256"),
    ),
    MaturityArchiveFile::new(
        "RUN-REPORT",
        include_bytes!("../fixtures/maturity-confidential-run-of-record/RUN-REPORT"),
    ),
];
const CONFIDENTIAL_PINS: MaturityCorpusPins<'static> = MaturityCorpusPins {
    names: [
        "2b0d2f31c8.report.capture",
        "2b0d2f31c8.report.capture.timing",
        "MANIFEST.sha256",
        "RUN-REPORT",
    ],
    sizes: [1_822_687, 49, 191, 193_213],
    manifest_sha256: MATURITY_CONFIDENTIAL_MANIFEST_SHA256,
    run_address: MATURITY_CONFIDENTIAL_RUN_ADDRESS,
    ceremony_sha256: MATURITY_CONFIDENTIAL_CEREMONY_SHA256,
    capture_target: MaturityCaptureTarget::new(
        "maturity_confidential",
        "the_confidential_maturity_route_runs_against_a_real_target",
    ),
    funding_count: 4,
    capabilities: &CONFIDENTIAL_CAPABILITIES,
    funding: &CONFIDENTIAL_FUNDING,
    ..RETAINED_PINS
};

const NATIVE_FILES: [MaturityArchiveFile<'static>; 4] = [
    MaturityArchiveFile::new(
        "745e4e01ad.report.capture",
        include_bytes!("../fixtures/maturity-native-run-of-record/745e4e01ad.report.capture"),
    ),
    MaturityArchiveFile::new(
        "745e4e01ad.report.capture.timing",
        include_bytes!(
            "../fixtures/maturity-native-run-of-record/745e4e01ad.report.capture.timing"
        ),
    ),
    MaturityArchiveFile::new(
        "MANIFEST.sha256",
        include_bytes!("../fixtures/maturity-native-run-of-record/MANIFEST.sha256"),
    ),
    MaturityArchiveFile::new(
        "RUN-REPORT",
        include_bytes!("../fixtures/maturity-native-run-of-record/RUN-REPORT"),
    ),
];
const NATIVE_PINS: MaturityCorpusPins<'static> = MaturityCorpusPins {
    names: [
        "745e4e01ad.report.capture",
        "745e4e01ad.report.capture.timing",
        "MANIFEST.sha256",
        "RUN-REPORT",
    ],
    sizes: [6_046_070, 49, 191, 446_426],
    manifest_sha256: MATURITY_NATIVE_MANIFEST_SHA256,
    run_address: MATURITY_NATIVE_RUN_ADDRESS,
    ceremony_sha256: MATURITY_NATIVE_CEREMONY_SHA256,
    fee_floors: MaturityDisclosedFeeFloors::node_defaults("0.00000100", "0.00000100"),
    ..VARIABLE_PINS
};

/// The ordinary archive pins and constructor table for one mutant run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MaturityMutantCorpusPins<'a> {
    pins: MaturityCorpusPins<'a>,
    table: MaturityMutantTable,
}

impl<'a> MaturityMutantCorpusPins<'a> {
    /// Pairs the four-file pins with the ceremony's table.
    #[must_use]
    pub const fn new(pins: MaturityCorpusPins<'a>, table: MaturityMutantTable) -> Self {
        Self { pins, table }
    }

    /// The archive's ordered files, addresses, and environment declarations.
    #[must_use]
    pub const fn pins(&self) -> &MaturityCorpusPins<'a> {
        &self.pins
    }

    /// The ceremony table reconstructed during replay.
    #[must_use]
    pub const fn table(&self) -> MaturityMutantTable {
        self.table
    }

    /// The report's constructor profile for this table.
    #[must_use]
    pub const fn profile(&self) -> &'static str {
        match self.table {
            MaturityMutantTable::PredecessorConstructor => "predecessor-constructor",
            MaturityMutantTable::SuccessorConstructor => "successor-constructor",
        }
    }
}

const PREDECESSOR_MUTANT_PINS: MaturityMutantCorpusPins<'static> = MaturityMutantCorpusPins::new(
    MaturityCorpusPins::new(
        MATURITY_VARIABLE_RUN_SCHEDULE,
        (
            [
                "f933b376.report.capture",
                "f933b376.report.capture.timing",
                "MANIFEST.sha256",
                "RUN-REPORT",
            ],
            [107_644, 49, 187, 2_116],
        ),
        (
            MATURITY_PREDECESSOR_MUTANT_MANIFEST_SHA256,
            MATURITY_PREDECESSOR_MUTANT_RUN_ADDRESS,
            PREDECESSOR_MUTANT_CEREMONY_SHA256,
        ),
        MaturitySponsorlessExpectation::Acceptance,
        MaturityCaptureTarget::new(
            "maturity_predecessor_mutants",
            "the_maturity_predecessor_mutants_are_refused_before_their_control_is_accepted",
        ),
        "/workspace/toolchains/cargo/bin/cargo",
        MaturityFrameworkRevision::IntendedTip,
        0,
        MATURITY_DISCLOSED_FEE_FLOORS,
    ),
    MaturityMutantTable::PredecessorConstructor,
);
const SUCCESSOR_MUTANT_PINS: MaturityMutantCorpusPins<'static> = MaturityMutantCorpusPins::new(
    MaturityCorpusPins::new(
        MATURITY_VARIABLE_RUN_SCHEDULE,
        (
            [
                "f933b376.report.capture",
                "f933b376.report.capture.timing",
                "MANIFEST.sha256",
                "RUN-REPORT",
            ],
            [109_614, 49, 187, 2_108],
        ),
        (
            MATURITY_SUCCESSOR_MUTANT_MANIFEST_SHA256,
            MATURITY_SUCCESSOR_MUTANT_RUN_ADDRESS,
            SUCCESSOR_MUTANT_CEREMONY_SHA256,
        ),
        MaturitySponsorlessExpectation::Acceptance,
        MaturityCaptureTarget::new(
            "maturity_successor_mutants",
            "the_maturity_successor_mutants_are_refused_before_their_control_is_accepted",
        ),
        "/workspace/toolchains/cargo/bin/cargo",
        MaturityFrameworkRevision::IntendedTip,
        0,
        MATURITY_DISCLOSED_FEE_FLOORS,
    ),
    MaturityMutantTable::SuccessorConstructor,
);

/// The layer at which maturity corpus admission stopped.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum MaturityCorpusImportRefusal {
    /// The four-file ordered census differs from the embedded archive.
    CorpusCensus,
    /// A fixed file has a different byte length.
    FileSize {
        name: String,
        expected: usize,
        actual: usize,
    },
    /// The manifest differs from its pinned SHA-256 address.
    ManifestHash,
    /// The manifest violates its closed two-line sorted grammar.
    ManifestGrammar,
    /// An embedded file differs from its manifest entry.
    ManifestDigest { name: String },
    /// The report violates its ordered grammar or fixed eligibility facts.
    RunReportGrammar,
    /// The report differs from its pinned run address.
    RunReportAddress,
    /// The capture prefix differs from its own recorded content hash.
    CaptureContentHash,
    /// A report, capture, deployment or response binding disagrees.
    CrossFileBinding { field: &'static str },
    /// The capture, payload or timing sidecar violates its closed grammar.
    TranscriptGrammar { name: String, line: usize },
    /// The payload contradicts evidence independently derived by planner replay.
    PayloadDisagreement { field: &'static str },
    /// The planner refused construction, an exact request, or a recorded response.
    Derivation(MaturityNativePlanRefusal),
    /// A retained family disagrees with its campaign or admission precondition.
    RetainedCampaign(crate::maturity_native::native::RetainedCampaignRefusal),
    /// A confidential family disagrees with its campaign or admission precondition.
    ConfidentialCampaign(crate::maturity_native::confidential::ConfidentialCampaignRefusal),
}

impl MaturityCorpusImportRefusal {
    const fn tracing_refusal_kind(&self) -> &'static str {
        match self {
            Self::CorpusCensus => "corpus_census",
            Self::FileSize { .. } => "file_size",
            Self::ManifestHash => "manifest_hash",
            Self::ManifestGrammar => "manifest_grammar",
            Self::ManifestDigest { .. } => "manifest_digest",
            Self::RunReportGrammar => "run_report_grammar",
            Self::RunReportAddress => "run_report_address",
            Self::CaptureContentHash => "capture_content_hash",
            Self::CrossFileBinding { .. } => "cross_file_binding",
            Self::TranscriptGrammar { .. } => "transcript_grammar",
            Self::PayloadDisagreement { .. } => "payload_disagreement",
            Self::Derivation(_) => "derivation",
            Self::RetainedCampaign(_) => "retained_campaign",
            Self::ConfidentialCampaign(_) => "confidential_campaign",
        }
    }
}

type ImportResult<T> = Result<T, MaturityCorpusImportRefusal>;
use MaturityCorpusImportRefusal as Refusal;

/// Report facts admitted together with the complete corpus.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MaturityReportFacts {
    suite_commit: String,
    suite_tree: String,
    expected_tip: String,
    binary_revision: String,
    intended_tip: String,
    adapter_name: String,
    adapter_version: String,
    node_name: String,
    node_version: String,
    environment: String,
    network_id: String,
    genesis_id: String,
    target_contract: String,
    ceremony_digest: String,
    manifest_digest: String,
    run_address: String,
}

impl MaturityReportFacts {
    /// Suite source commit read from the report.
    #[must_use]
    pub fn suite_commit(&self) -> &str {
        &self.suite_commit
    }

    /// Suite source tree read from the report.
    #[must_use]
    pub fn suite_tree(&self) -> &str {
        &self.suite_tree
    }

    /// Expected Elements source tip read from the report.
    #[must_use]
    pub fn expected_tip(&self) -> &str {
        &self.expected_tip
    }

    /// Revision reported by the executed binary.
    #[must_use]
    pub fn binary_revision(&self) -> &str {
        &self.binary_revision
    }

    /// Intended executed source tip.
    #[must_use]
    pub fn intended_tip(&self) -> &str {
        &self.intended_tip
    }

    /// Executed adapter name.
    #[must_use]
    pub fn adapter_name(&self) -> &str {
        &self.adapter_name
    }

    /// Executed adapter version.
    #[must_use]
    pub fn adapter_version(&self) -> &str {
        &self.adapter_version
    }

    /// Executed node name.
    #[must_use]
    pub fn node_name(&self) -> &str {
        &self.node_name
    }

    /// Complete binary version banner.
    #[must_use]
    pub fn node_version(&self) -> &str {
        &self.node_version
    }

    /// Deployment environment.
    #[must_use]
    pub fn environment(&self) -> &str {
        &self.environment
    }

    /// Printed-order deployment network identity.
    #[must_use]
    pub fn network_id(&self) -> &str {
        &self.network_id
    }

    /// Printed-order deployment genesis identity.
    #[must_use]
    pub fn genesis_id(&self) -> &str {
        &self.genesis_id
    }

    /// Deployment target contract.
    #[must_use]
    pub fn target_contract(&self) -> &str {
        &self.target_contract
    }

    /// SHA-256 of the complete capture file.
    #[must_use]
    pub fn ceremony_digest(&self) -> &str {
        &self.ceremony_digest
    }

    /// SHA-256 of the manifest bytes.
    #[must_use]
    pub fn manifest_digest(&self) -> &str {
        &self.manifest_digest
    }

    /// SHA-256 of the complete report bytes.
    #[must_use]
    pub fn run_address(&self) -> &str {
        &self.run_address
    }

    /// The admitted native protocol revision.
    #[must_use]
    pub const fn protocol_revision(&self) -> u32 {
        8
    }
}

fn maturity_historical_standing(
    report: &MaturityReportFacts,
    source: &[String; 2],
    exchanges: &[(OperationStep, NativeOperationResponse)],
) -> crate::historical_candidate::CandidateStanding {
    let programs = exchanges
        .iter()
        .filter_map(|(step, _)| match step.subject() {
            OperationSubject::Funding(subject) => Some(tagged::sha256(&subject.output_program)),
            OperationSubject::ScriptPathSigning(subject) => {
                Some(tagged::sha256(&subject.executing_leaf.script))
            }
            _ => None,
        })
        .collect();
    crate::historical_candidate::CandidateStanding::HistoricalCandidate(
        crate::historical_candidate::HistoricalCandidate {
            source_revision: Some(source.clone()),
            deployment: [
                report.network_id.clone(),
                report.genesis_id.clone(),
                report.target_contract.clone(),
            ],
            programs,
            replay: crate::historical_candidate::HistoricalReplay::ProducingCandidate,
        },
    )
}

/// Complete maturity evidence from admitted bytes with declared executor fee floors.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedMaturityCorpus {
    evidence: MaturityNativeEvidence,
    report: MaturityReportFacts,
    fee_floors: MaturityDisclosedFeeFloors,
    capture_suite: [String; 2],
    exchanges: Vec<(OperationStep, NativeOperationResponse)>,
    capabilities: Vec<String>,
    recorded_refusal_detail: String,
}

impl ValidatedMaturityCorpus {
    /// Historical standing retained by every transitive maturity report reader.
    #[must_use]
    pub fn historical_standing(&self) -> crate::historical_candidate::CandidateStanding {
        maturity_historical_standing(&self.report, &self.capture_suite, &self.exchanges)
    }

    /// The composed schedule established by replay of the admitted exchanges.
    #[must_use]
    pub const fn schedule(&self) -> tapscript::StateWitnessSchedule {
        self.evidence.schedule()
    }

    /// The evidence produced by the planner's reviewed non-mock settlement.
    #[must_use]
    pub const fn evidence(&self) -> &MaturityNativeEvidence {
        &self.evidence
    }

    /// The fee floors declared for this admitted archive's executor environment.
    #[must_use]
    pub const fn fee_floors(&self) -> MaturityDisclosedFeeFloors {
        self.fee_floors
    }

    /// The closed report's cross-checked facts.
    #[must_use]
    pub const fn report(&self) -> &MaturityReportFacts {
        &self.report
    }

    /// Source commit read independently from the capture.
    #[must_use]
    pub fn capture_suite_commit(&self) -> &str {
        &self.capture_suite[0]
    }

    /// Source tree read independently from the capture.
    #[must_use]
    pub fn capture_suite_tree(&self) -> &str {
        &self.capture_suite[1]
    }

    /// Recorded responses and their replay-checked subjects in native step order.
    #[must_use]
    pub fn exchanges(&self) -> &[(OperationStep, NativeOperationResponse)] {
        &self.exchanges
    }

    /// The exact ordered environment capability roster.
    #[must_use]
    pub fn capabilities(&self) -> &[String] {
        &self.capabilities
    }

    /// The sponsorless operation's recorded response detail: a refusal reason or empty on acceptance.
    #[must_use]
    pub fn recorded_refusal_detail(&self) -> &str {
        &self.recorded_refusal_detail
    }

    /// Stable content address of the report-bound corpus.
    #[must_use]
    pub fn content_address(&self) -> &str {
        self.report.run_address()
    }
}

/// Replay-validated mutant ceremony together with its pinned report facts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidatedMaturityMutantCorpus {
    evidence: MaturityMutantEvidence,
    report: MaturityReportFacts,
    fee_floors: MaturityDisclosedFeeFloors,
    capture_suite: [String; 2],
    exchanges: Vec<(OperationStep, NativeOperationResponse)>,
    capabilities: Vec<String>,
    phase_walls: [u64; 4],
}

impl ValidatedMaturityMutantCorpus {
    /// Historical standing of the exact replayed mutant subjects.
    #[must_use]
    pub fn historical_standing(&self) -> crate::historical_candidate::CandidateStanding {
        maturity_historical_standing(&self.report, &self.capture_suite, &self.exchanges)
    }

    /// The evidence reconstructed from the exact recorded requests and responses.
    #[must_use]
    pub const fn evidence(&self) -> &MaturityMutantEvidence {
        &self.evidence
    }
    /// The closed report's cross-checked facts.
    #[must_use]
    pub const fn report(&self) -> &MaturityReportFacts {
        &self.report
    }
    /// The declared fee floors of the executor environment.
    #[must_use]
    pub const fn fee_floors(&self) -> MaturityDisclosedFeeFloors {
        self.fee_floors
    }
    /// Source commit read independently from the capture.
    #[must_use]
    pub fn capture_suite_commit(&self) -> &str {
        &self.capture_suite[0]
    }
    /// Source tree read independently from the capture.
    #[must_use]
    pub fn capture_suite_tree(&self) -> &str {
        &self.capture_suite[1]
    }
    /// Recorded exchanges whose requests were rebuilt by the ceremony planner.
    #[must_use]
    pub fn exchanges(&self) -> &[(OperationStep, NativeOperationResponse)] {
        &self.exchanges
    }
    /// The exact ordered environment capability roster.
    #[must_use]
    pub fn capabilities(&self) -> &[String] {
        &self.capabilities
    }
    /// Reported preflight, cargo, census, and manifest wall times in milliseconds.
    #[must_use]
    pub const fn phase_walls(&self) -> &[u64; 4] {
        &self.phase_walls
    }
    /// The constructor table used for exact replay.
    #[must_use]
    pub const fn table(&self) -> MaturityMutantTable {
        self.evidence.table()
    }
}

struct Cursor<'a> {
    name: &'static str,
    lines: Vec<&'a str>,
    index: usize,
}

impl<'a> Cursor<'a> {
    fn new(name: &'static str, bytes: &'a [u8]) -> ImportResult<Self> {
        let refusal = || Refusal::TranscriptGrammar {
            name: name.to_owned(),
            line: 1,
        };
        if bytes.contains(&b'\r') || !bytes.ends_with(b"\n") {
            return Err(refusal());
        }
        let text = std::str::from_utf8(bytes).map_err(|_| refusal())?;
        let body = text.strip_suffix('\n').ok_or_else(refusal)?;
        if body.split('\n').any(str::is_empty) {
            return Err(refusal());
        }
        Ok(Self {
            name,
            lines: body.split('\n').collect(),
            index: 0,
        })
    }

    fn refusal(&self) -> Refusal {
        Refusal::TranscriptGrammar {
            name: self.name.to_owned(),
            line: self.index + 1,
        }
    }

    fn exact(&mut self, expected: &str) -> ImportResult<()> {
        if self.lines.get(self.index).copied() != Some(expected) {
            return Err(self.refusal());
        }
        self.index += 1;
        Ok(())
    }

    fn value(&mut self, field: &str) -> ImportResult<&'a str> {
        let line = self
            .lines
            .get(self.index)
            .copied()
            .ok_or_else(|| self.refusal())?;
        let value = line
            .strip_prefix(field)
            .and_then(|tail| tail.strip_prefix(' '))
            .ok_or_else(|| self.refusal())?;
        self.index += 1;
        Ok(value)
    }

    fn bytes(&mut self, field: &str) -> ImportResult<Vec<u8>> {
        let value = self.value(field)?;
        let (length, hex) = value.split_once(' ').ok_or_else(|| self.refusal())?;
        let bytes = decode_hex(hex).ok_or_else(|| self.refusal())?;
        if number(length) != Some(bytes.len()) {
            return Err(self.refusal());
        }
        Ok(bytes)
    }

    fn waiting(&mut self) -> ImportResult<Vec<u8>> {
        if self
            .lines
            .get(self.index)
            .is_some_and(|line| line.starts_with("sponsored-waiting "))
        {
            self.bytes("sponsored-waiting")
        } else {
            Ok(b"[]".to_vec())
        }
    }

    fn text(&mut self, field: &str) -> ImportResult<String> {
        String::from_utf8(self.bytes(field)?).map_err(|_| self.refusal())
    }

    fn done(&self) -> ImportResult<()> {
        if self.index != self.lines.len() {
            return Err(self.refusal());
        }
        Ok(())
    }
}

fn number(text: &str) -> Option<usize> {
    if text.is_empty()
        || text.len() > 1 && text.starts_with('0')
        || !text.bytes().all(|byte| byte.is_ascii_digit())
    {
        return None;
    }
    text.parse().ok()
}

fn decode_hex(text: &str) -> Option<Vec<u8>> {
    if !text.len().is_multiple_of(2)
        || !text
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return None;
    }
    let (pairs, _) = text.as_bytes().as_chunks::<2>();
    pairs
        .iter()
        .map(|pair| {
            let digit = |byte: u8| {
                if byte.is_ascii_digit() {
                    byte - b'0'
                } else {
                    byte - b'a' + 10
                }
            };
            Some((digit(pair[0]) << 4) | digit(pair[1]))
        })
        .collect()
}

fn hex_bytes(bytes: &[u8]) -> String {
    let mut text = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(text, "{byte:02x}");
    }
    text
}

fn digest(text: &str) -> Option<[u8; 32]> {
    decode_hex(text)?.try_into().ok()
}

fn source_identity(text: &str) -> bool {
    text.len() == 40 && decode_hex(text).is_some()
}

const fn binding(field: &'static str, agrees: bool) -> ImportResult<()> {
    if agrees {
        Ok(())
    } else {
        Err(Refusal::CrossFileBinding { field })
    }
}

fn validate_census(
    files: &[MaturityArchiveFile<'_>],
    pins: &MaturityCorpusPins<'_>,
) -> ImportResult<()> {
    if files.len() != pins.names.len()
        || files
            .iter()
            .zip(pins.names)
            .any(|(file, name)| file.name != name)
    {
        return Err(Refusal::CorpusCensus);
    }
    for (file, expected) in files.iter().zip(&pins.sizes) {
        if file.bytes.len() != *expected {
            return Err(Refusal::FileSize {
                name: file.name.to_owned(),
                expected: *expected,
                actual: file.bytes.len(),
            });
        }
    }
    Ok(())
}

fn validate_manifest(files: &[MaturityArchiveFile<'_>], expected_hash: &str) -> ImportResult<()> {
    if hex_bytes(&tagged::sha256(files[2].bytes)) != expected_hash {
        return Err(Refusal::ManifestHash);
    }
    let mut cursor =
        Cursor::new("MANIFEST.sha256", files[2].bytes).map_err(|_| Refusal::ManifestGrammar)?;
    for file in &files[..2] {
        let line = cursor
            .lines
            .get(cursor.index)
            .ok_or(Refusal::ManifestGrammar)?;
        let (hash, name) = line.split_once("  ").ok_or(Refusal::ManifestGrammar)?;
        if name != file.name || digest(hash).is_none() {
            return Err(Refusal::ManifestGrammar);
        }
        if hex_bytes(&tagged::sha256(file.bytes)) != hash {
            return Err(Refusal::ManifestDigest {
                name: name.to_owned(),
            });
        }
        cursor.index += 1;
    }
    cursor.done().map_err(|_| Refusal::ManifestGrammar)
}

fn parse_report_suite(
    cursor: &mut Cursor<'_>,
    pins: &MaturityCorpusPins<'_>,
    mutant_profile: Option<&str>,
    exact_test: Option<&str>,
) -> ImportResult<[String; 5]> {
    cursor.exact(if mutant_profile.is_some() {
        "run-report-schema native-maturity-mutant-run-report 1"
    } else {
        "run-report-schema native-maturity-run-report 1"
    })?;
    cursor.exact("capture-format-schema native-maturity-capture 1")?;
    let suite_commit = cursor.value("suite-commit")?.to_owned();
    let suite_tree = cursor.value("suite-tree")?.to_owned();
    if !source_identity(&suite_commit) || !source_identity(&suite_tree) {
        return Err(cursor.refusal());
    }
    cursor.exact("suite-clean yes")?;
    cursor.exact(&format!(
        "rust-test-target {}",
        pins.capture_target().target()
    ))?;
    if let Some(profile) = mutant_profile {
        cursor.exact(&format!("capture-profile {profile}"))?;
    }
    let selection = if let Some(test) = exact_test {
        if cursor.text("rust-test-name")? != test {
            return Err(cursor.refusal());
        }
        format!(" --exact {test}")
    } else {
        String::new()
    };
    if cursor.text("cargo-argv")?
        != format!(
            "{} test -p tripod-vectors --test {} -- --ignored{selection} --test-threads=1",
            pins.cargo_path(),
            pins.capture_target().target()
        )
    {
        return Err(cursor.refusal());
    }
    for line in [
        "expected-test-count 1",
        "observed-test-count 1",
        "expected-ceremony-count 1",
        "observed-ceremony-count 1",
        "expected-setup-count 0",
        "observed-setup-count 0",
        "diagnostics-present yes",
        "test-passed-count 1",
        "test-failed-count 0",
        "test-ignored-count 0",
        "cargo-exit-code 0",
    ] {
        cursor.exact(line)?;
    }
    let expected_tip = cursor.value("elementsd-expected-tip")?.to_owned();
    let binary_revision = cursor.text("elementsd-binary-reported-revision")?;
    let intended_tip = cursor.text("elementsd-intended-executed-tip")?;
    if !source_identity(&expected_tip)
        || binary_revision.len() != 12
        || !expected_tip.starts_with(&binary_revision)
        || intended_tip != expected_tip
    {
        return Err(cursor.refusal());
    }
    Ok([
        suite_commit,
        suite_tree,
        expected_tip,
        binary_revision,
        intended_tip,
    ])
}

fn parse_report_body(
    bytes: &[u8],
    pins: &MaturityCorpusPins<'_>,
    mutant_profile: Option<&str>,
) -> ImportResult<(MaturityReportFacts, [u64; 4])> {
    parse_report_body_selected(bytes, pins, mutant_profile, None)
}

fn parse_report_body_selected(
    bytes: &[u8],
    pins: &MaturityCorpusPins<'_>,
    mutant_profile: Option<&str>,
    exact_test: Option<&str>,
) -> ImportResult<(MaturityReportFacts, [u64; 4])> {
    let mut cursor = Cursor::new("RUN-REPORT", bytes)?;
    let [
        suite_commit,
        suite_tree,
        expected_tip,
        binary_revision,
        intended_tip,
    ] = parse_report_suite(&mut cursor, pins, mutant_profile, exact_test)?;
    let adapter_name = cursor.text("executor-adapter-name")?;
    let adapter_version = cursor.text("executor-adapter-version")?;
    let node_name = cursor.text("node-name")?;
    let node_version = cursor.text("node-version")?;
    if adapter_name != "elements-native-executor"
        || adapter_version != "2.1.0"
        || node_name != "Elements"
        || node_version != format!("Elements Core daemon version v28.99.0-{binary_revision}")
    {
        return Err(cursor.refusal());
    }
    cursor.exact("protocol-revision 8")?;
    cursor.exact("fixture-digest-algorithm forward-v2")?;
    let environment = cursor.value("deployment-environment")?.to_owned();
    let network_id = cursor.text("deployment-network-id")?;
    let genesis_id = cursor.text("deployment-genesis-id")?;
    let target_contract = cursor.text("deployment-target-contract")?;
    if environment != "development"
        || digest(&network_id).is_none()
        || digest(&genesis_id).is_none()
        || target_contract != "elements-tapscript-v2"
    {
        return Err(cursor.refusal());
    }
    let mut phase_walls = [0; 4];
    if mutant_profile.is_some() {
        for (index, phase) in ["preflight", "cargo", "census", "manifest"]
            .iter()
            .enumerate()
        {
            let wall = cursor.value("phase-wall-ms")?;
            let digits = wall
                .strip_prefix(phase)
                .and_then(|tail| tail.strip_prefix(' '))
                .ok_or_else(|| cursor.refusal())?;
            if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
                return Err(cursor.refusal());
            }
            phase_walls[index] = digits.parse().map_err(|_| cursor.refusal())?;
        }
    }
    cursor.exact("ceremony-roster begin")?;
    let ceremony_digest = cursor.value("ceremony report")?.to_owned();
    cursor.exact("ceremony-roster end")?;
    let manifest_digest = cursor.value("manifest-sha256")?.to_owned();
    if digest(&ceremony_digest).is_none() || digest(&manifest_digest).is_none() {
        return Err(cursor.refusal());
    }
    cursor.exact("eligible yes")?;
    cursor.done()?;
    Ok((
        MaturityReportFacts {
            suite_commit,
            suite_tree,
            expected_tip,
            binary_revision,
            intended_tip,
            adapter_name,
            adapter_version,
            node_name,
            node_version,
            environment,
            network_id,
            genesis_id,
            target_contract,
            ceremony_digest,
            manifest_digest,
            run_address: hex_bytes(&tagged::sha256(bytes)),
        },
        phase_walls,
    ))
}

fn parse_report(bytes: &[u8], pins: &MaturityCorpusPins<'_>) -> ImportResult<MaturityReportFacts> {
    let (report, _) =
        parse_report_body(bytes, pins, None).map_err(|_| Refusal::RunReportGrammar)?;
    check_report_address(&report, pins)?;
    Ok(report)
}

fn check_report_address(
    report: &MaturityReportFacts,
    pins: &MaturityCorpusPins<'_>,
) -> ImportResult<()> {
    if report.run_address != pins.run_address() {
        return Err(Refusal::RunReportAddress);
    }
    Ok(())
}

fn parse_mutant_report(
    bytes: &[u8],
    pins: &MaturityMutantCorpusPins<'_>,
) -> ImportResult<(MaturityReportFacts, [u64; 4])> {
    let (report, walls) = parse_report_body(bytes, pins.pins(), Some(pins.profile()))
        .map_err(|_| Refusal::RunReportGrammar)?;
    check_report_address(&report, pins.pins())?;
    Ok((report, walls))
}

const CAPABILITIES: [&str; 12] = [
    "failure-class-reporting",
    "transaction-context",
    "resource-observation",
    "tree-materialization",
    "confidential-conservation",
    "owner-authorized-normalization",
    "compound-prototype-fixtures",
    "fresh-process-lifecycle",
    "test-funding-ceremony",
    "target-transaction-submission",
    "test-sponsor-authorization",
    "test-script-path-authorization",
];

const CONFIDENTIAL_CAPABILITIES: [&str; 14] = [
    "failure-class-reporting",
    "transaction-context",
    "resource-observation",
    "tree-materialization",
    "confidential-conservation",
    "owner-authorized-normalization",
    "compound-prototype-fixtures",
    "fresh-process-lifecycle",
    "test-funding-ceremony",
    "target-transaction-submission",
    "test-sponsor-authorization",
    "test-script-path-authorization",
    "confidential-value-test-funding",
    "confidential-value-sponsor-authorization",
];

const CONFIDENTIAL_FUNDING: [&str; 4] = [
    "0 representation explicit-asset-confidential-value",
    "1 custody central-public-fixtures",
    "2 materializer guide-ctf-deterministic-v1",
    "3 reproducibility byte_identity",
];

fn parse_environment(
    cursor: &mut Cursor<'_>,
    report: &MaturityReportFacts,
    pins: &MaturityCorpusPins<'_>,
) -> ImportResult<Vec<String>> {
    cursor.exact("environment-schema 8")?;
    cursor.exact("environment-chain 15 656c656d656e747372656774657374")?;
    binding(
        "environment-network-id",
        cursor.text("environment-network-id")? == report.network_id,
    )?;
    binding(
        "environment-genesis-id",
        cursor.text("environment-genesis-id")? == report.genesis_id,
    )?;
    for line in [
        "environment-domain-count 1",
        "environment-domain 0 tapscript supported true active true",
        "environment-leaf-count 1",
        "environment-leaf 0 196 supported true active true",
    ] {
        cursor.exact(line)?;
    }
    cursor.exact(&format!(
        "environment-capability-count {}",
        pins.capabilities.len()
    ))?;
    let mut capabilities = Vec::new();
    for (index, expected) in pins.capabilities.iter().enumerate() {
        let value = cursor.value("environment-capability")?;
        if value != format!("{index} {expected}") {
            return Err(cursor.refusal());
        }
        capabilities.push((*expected).to_owned());
    }
    cursor.exact(&format!(
        "environment-funding-count {}",
        pins.funding_count()
    ))?;
    binding(
        "environment-funding-roster",
        pins.funding.len() == pins.funding_count(),
    )?;
    for expected in pins.funding {
        cursor.exact(&format!("environment-funding {expected}"))?;
    }
    Ok(capabilities)
}

fn parse_header(
    cursor: &mut Cursor<'_>,
    report: &MaturityReportFacts,
    pins: &MaturityCorpusPins<'_>,
) -> ImportResult<[String; 2]> {
    cursor.exact("native-capture-schema 2")?;
    cursor.exact("ceremony-id report")?;
    if cursor.text("rust-test-name")? != pins.capture_target().test() {
        return Err(cursor.refusal());
    }
    let suite_commit = cursor.value("suite-commit")?.to_owned();
    let suite_tree = cursor.value("suite-tree")?.to_owned();
    binding(
        "suite-commit",
        source_identity(&suite_commit) && suite_commit == report.suite_commit,
    )?;
    binding(
        "suite-tree",
        source_identity(&suite_tree) && suite_tree == report.suite_tree,
    )?;
    cursor.exact("fixture-digest-algorithm forward-v2")?;
    cursor.exact("run-id-input begin")?;
    binding(
        "deployment-environment",
        cursor.value("deployment-environment")? == report.environment,
    )?;
    for (field, expected) in [
        ("deployment-network-id", &report.network_id),
        ("deployment-genesis-id", &report.genesis_id),
        ("deployment-target-contract", &report.target_contract),
    ] {
        binding(field, cursor.text(field)? == *expected)?;
    }
    binding(
        "protocol-revision",
        cursor.value("handshake-protocol-schema")? == "8",
    )?;
    let framework_revision = match pins.framework_revision() {
        MaturityFrameworkRevision::Unrecorded => "",
        MaturityFrameworkRevision::IntendedTip => report.intended_tip.as_str(),
    };
    for (field, expected) in [
        ("handshake-adapter-name", report.adapter_name.as_str()),
        ("handshake-adapter-version", report.adapter_version.as_str()),
        ("handshake-framework-revision", framework_revision),
        ("handshake-node-name", report.node_name.as_str()),
        ("handshake-node-version", report.node_version.as_str()),
        (
            "handshake-binary-reported-revision",
            report.binary_revision.as_str(),
        ),
        (
            "handshake-intended-executed-tip",
            report.intended_tip.as_str(),
        ),
    ] {
        binding(field, cursor.text(field)? == expected)?;
    }
    cursor.exact("handshake-upstream-base 0 ")?;
    cursor.exact("handshake-topic-count 0")?;
    Ok([suite_commit, suite_tree])
}

struct RecordedOperation {
    step: OperationStep,
    layer: ObservedOutcomeLayer,
    detail: String,
    accepted_identity: Option<Txid>,
    request_bytes: Vec<u8>,
}

// These are the two exact JSON shapes emitted by this capture's serializer.
// Fixed punctuation and canonical numbers close the grammar without admitting
// duplicate, unknown, reordered or omitted members through a second JSON model.
fn json_bytes(text: &str) -> Option<Vec<u8>> {
    let body = text.strip_prefix('[')?.strip_suffix(']')?;
    if body.is_empty() {
        return Some(Vec::new());
    }
    body.split(',')
        .map(|value| number(value).and_then(|value| u8::try_from(value).ok()))
        .collect()
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SubjectKind {
    Funding,
    Submission,
    Sponsored,
    Confidential,
}

impl From<usize> for SubjectKind {
    fn from(index: usize) -> Self {
        if index == 2 {
            Self::Submission
        } else {
            Self::Funding
        }
    }
}

fn parse_subject(text: &str, kind: impl Into<SubjectKind>) -> Option<OperationSubject> {
    let kind = kind.into();
    if matches!(kind, SubjectKind::Sponsored | SubjectKind::Confidential) {
        let subject: OperationSubject = serde_json::from_str(text).ok()?;
        return (serde_json::to_string(&subject).ok()?.as_str() == text
            && (matches!(
                subject,
                OperationSubject::Funding(_)
                    | OperationSubject::SponsorFunding(_)
                    | OperationSubject::SponsorSigning(_)
                    | OperationSubject::Submission(_)
            ) || (kind == SubjectKind::Confidential
                && matches!(subject, OperationSubject::ConfidentialSponsorFunding(_)))))
        .then_some(subject);
    }
    if kind == SubjectKind::Submission {
        let array = text
            .strip_prefix("{\"transaction_bytes\":")?
            .strip_suffix('}')?;
        return Some(OperationSubject::Submission(Box::new(
            TargetSubmissionSubject {
                transaction_bytes: json_bytes(array)?,
            },
        )));
    }
    let rest = text.strip_prefix("{\"issue_asset\":")?;
    let (issue, rest) = rest.split_once(",\"asset\":")?;
    let issue_asset = match issue {
        "true" => true,
        "false" => false,
        _ => return None,
    };
    let (asset, rest) = rest.split_once(",\"output_program\":")?;
    let asset = if asset == "null" {
        None
    } else {
        let asset = asset.strip_prefix('"')?.strip_suffix('"')?;
        digest(asset)?;
        Some(asset.to_owned())
    };
    let (program, rest) = rest.split_once(",\"outputs\":")?;
    let (outputs, amount) = rest.split_once(",\"amount_per_output\":")?;
    let amount = amount.strip_suffix('}')?;
    number(amount)?;
    Some(OperationSubject::Funding(Box::new(TargetFundingSubject {
        issue_asset,
        asset,
        output_program: json_bytes(program)?,
        outputs: u8::try_from(number(outputs)?).ok()?,
        amount_per_output: amount.parse().ok()?,
    })))
}

fn parse_operation(
    cursor: &mut Cursor<'_>,
    index: usize,
    name: &'static str,
    expectation: MaturitySponsorlessExpectation,
) -> ImportResult<RecordedOperation> {
    let kind = SubjectKind::from(index);
    let operation = parse_operation_block(cursor, index, name, kind)?;
    let accepted = index < 2 || expectation == MaturitySponsorlessExpectation::Acceptance;
    let expected_layer = if accepted {
        ObservedOutcomeLayer::Accepted
    } else {
        ObservedOutcomeLayer::RelayPolicyRejection
    };
    let expected_detail = if accepted {
        ""
    } else {
        match expectation {
            MaturitySponsorlessExpectation::RelayRefusal { detail } => detail,
            MaturitySponsorlessExpectation::Acceptance => "",
        }
    };
    if operation.layer != expected_layer
        || operation.detail != expected_detail
        || (index < 2 && operation.accepted_identity.is_some())
        || (index == 2 && accepted != operation.accepted_identity.is_some())
    {
        return Err(cursor.refusal());
    }
    Ok(operation)
}

fn recorded_layer(text: &str) -> Option<ObservedOutcomeLayer> {
    Some(match text {
        "accepted" => ObservedOutcomeLayer::Accepted,
        "script-path-rejection" => ObservedOutcomeLayer::ScriptPathRejection,
        "key-path-rejection" => ObservedOutcomeLayer::KeyPathRejection,
        "relay-policy-rejection" => ObservedOutcomeLayer::RelayPolicyRejection,
        "consensus-rejection-before-script" => ObservedOutcomeLayer::ConsensusRejectionBeforeScript,
        "fixture-construction-failure" => ObservedOutcomeLayer::FixtureConstructionFailure,
        "executor-infrastructure-failure" => ObservedOutcomeLayer::ExecutorInfrastructureFailure,
        _ => return None,
    })
}

fn parse_operation_block(
    cursor: &mut Cursor<'_>,
    index: usize,
    name: &str,
    kind: SubjectKind,
) -> ImportResult<RecordedOperation> {
    cursor.exact(&format!("operation {index} begin"))?;
    for (field, expected) in [
        ("operation-id", format!("operation-{index}")),
        ("request-id", format!("request-{index}")),
    ] {
        binding(field, cursor.text(field)? == expected)?;
    }
    cursor.exact("request-role auxiliary")?;
    let request_bytes = cursor.bytes("request-bytes")?;
    let subject =
        parse_subject(&cursor.text("request-subject")?, kind).ok_or_else(|| cursor.refusal())?;
    binding(
        "request-bytes",
        match &subject {
            OperationSubject::Funding(_)
            | OperationSubject::SponsorFunding(_)
            | OperationSubject::SponsorSigning(_) => request_bytes.is_empty(),
            OperationSubject::ConfidentialSponsorFunding(_) => {
                kind == SubjectKind::Confidential && request_bytes.is_empty()
            }
            OperationSubject::Submission(submission) => {
                !request_bytes.is_empty() && request_bytes == submission.transaction_bytes
            }
            _ => false,
        },
    )?;
    for (field, expected) in [
        ("response-id", format!("response-{index}")),
        ("response-request-id", format!("request-{index}")),
        ("response-operation-id", format!("operation-{index}")),
    ] {
        binding(field, cursor.text(field)? == expected)?;
    }
    let accepted = match cursor.value("response-verdict")? {
        "accepted" => true,
        "refused" => false,
        _ => return Err(cursor.refusal()),
    };
    let layer = recorded_layer(cursor.value("response-layer")?).ok_or_else(|| cursor.refusal())?;
    if accepted != (layer == ObservedOutcomeLayer::Accepted) {
        return Err(cursor.refusal());
    }
    let identity = cursor.value("response-target-identity")?;
    let accepted_identity = if identity == "none" {
        None
    } else {
        Some(Txid::from_target_display(identity).map_err(|_| cursor.refusal())?)
    };
    if !accepted && accepted_identity.is_some() {
        return Err(cursor.refusal());
    }
    let detail = cursor.text("response-detail")?;
    for line in [
        "attribution-control-request-id none",
        "attribution-control-identity none",
        "mutation-kind none",
        "mutation-locator none",
        "projection-input none",
    ] {
        cursor.exact(line)?;
    }
    cursor.exact(&format!("operation {index} end"))?;
    Ok(RecordedOperation {
        step: OperationStep::new(name, subject),
        detail,
        accepted_identity,
        request_bytes,
        layer,
    })
}
fn parse_tail(cursor: &mut Cursor<'_>, bytes: &[u8]) -> ImportResult<Vec<u8>> {
    let payload = cursor.bytes("legacy-rendering")?;
    cursor.exact("terminal-state complete")?;
    cursor.exact("run-id-input end")?;
    let prefix_len = cursor.lines[..cursor.index]
        .iter()
        .map(|line| line.len() + 1)
        .sum::<usize>();
    let hash = cursor.value("capture-content-sha256")?;
    if digest(hash).is_none() {
        return Err(cursor.refusal());
    }
    if hex_bytes(&tagged::sha256(&bytes[..prefix_len])) != hash {
        return Err(Refusal::CaptureContentHash);
    }
    cursor.exact("native-capture-end report")?;
    cursor.done()?;
    Ok(payload)
}

enum PayloadAcceptance {
    Outstanding {
        claim: String,
        routes: [String; 2],
    },
    Established {
        schedule: tapscript::StateWitnessSchedule,
        identity: Txid,
        witness_identity: Txid,
        block_hash: [u8; 32],
        block_height: u32,
    },
}

struct Payload {
    identity: CandidateDeploymentIdentity,
    branch: BranchContext,
    exchanges: Vec<(OperationStep, NativeOperationResponse)>,
    rows: Vec<String>,
    standing: String,
    acceptance: PayloadAcceptance,
}

fn parse_acceptance(
    cursor: &mut Cursor<'_>,
    operations: &[RecordedOperation],
    exchanges: &mut [(OperationStep, NativeOperationResponse)],
    expectation: MaturitySponsorlessExpectation,
) -> ImportResult<PayloadAcceptance> {
    let claim = cursor.value("sponsorless-acceptance")?.to_owned();
    match expectation {
        MaturitySponsorlessExpectation::RelayRefusal { .. } => {
            if claim == "established" {
                return Err(cursor.refusal());
            }
            Ok(PayloadAcceptance::Outstanding {
                claim,
                routes: [
                    cursor.value("acceptance-route")?.to_owned(),
                    cursor.value("acceptance-route")?.to_owned(),
                ],
            })
        }
        MaturitySponsorlessExpectation::Acceptance => {
            if claim != "established" {
                return Err(cursor.refusal());
            }
            let schedule = match cursor.value("accepted-schedule")? {
                "whole-metadata" => tapscript::StateWitnessSchedule::WholeMetadata,
                "variable-metadata" => tapscript::StateWitnessSchedule::VariableMetadata,
                _ => return Err(cursor.refusal()),
            };
            let identity = Txid::from_target_display(cursor.value("accepted-identity")?)
                .map_err(|_| cursor.refusal())?;
            let witness_identity =
                Txid::from_target_display(cursor.value("accepted-witness-identity")?)
                    .map_err(|_| cursor.refusal())?;
            let block = cursor.value("accepted-block")?;
            let (block_hash, height) = block.split_once(' ').ok_or_else(|| cursor.refusal())?;
            let block_hash = digest(block_hash).ok_or_else(|| cursor.refusal())?;
            let block_height = number(height)
                .and_then(|height| u32::try_from(height).ok())
                .ok_or_else(|| cursor.refusal())?;
            let operation = operations.last().ok_or(Refusal::CorpusCensus)?;
            let capture_identity = operation
                .accepted_identity
                .ok_or_else(|| cursor.refusal())?;
            binding("accepted-identity", capture_identity == identity)?;
            let (_, response) = exchanges.last_mut().ok_or(Refusal::CorpusCensus)?;
            response.accepted_txid = Some(capture_identity.to_target_display());
            response.mined_readback = Some(MinedFundingReadback {
                transaction_id: capture_identity.to_target_display(),
                witness_transaction_id: witness_identity.to_target_display(),
                block_hash: hex_bytes(&block_hash),
                block_height,
                raw_transaction: operation.request_bytes.clone(),
            });
            Ok(PayloadAcceptance::Established {
                schedule,
                identity,
                witness_identity,
                block_hash,
                block_height,
            })
        }
    }
}

fn parse_payload(
    bytes: &[u8],
    operations: &[RecordedOperation],
    report: &MaturityReportFacts,
    expectation: MaturitySponsorlessExpectation,
) -> ImportResult<Payload> {
    let mut cursor = Cursor::new("maturity payload", bytes)?;
    cursor.exact("maturity-evidence-schema 1")?;
    let (identity, branch) = parse_payload_context(&mut cursor, report)?;
    cursor.exact("current-root-freshness unestablished")?;
    let mut exchanges = Vec::new();
    for (index, (name, operation)) in ANNOUNCEMENT_STEPS.iter().zip(operations).enumerate() {
        cursor.exact(&format!("maturity-step {name}"))?;
        let response =
            NativeOperationResponse::from_recorded_json(&cursor.bytes("validated-response")?)
                .map_err(|_| cursor.refusal())?;
        binding("response-case", response.case == *operation.step.case())?;
        binding(
            "response-summary",
            response.observed_layer == operation.layer
                && (index == 2 && expectation == MaturitySponsorlessExpectation::Acceptance
                    || response.accepted_txid.is_none())
                && response.observed_detail.as_deref().unwrap_or_default() == operation.detail,
        )?;
        exchanges.push((operation.step.clone(), response));
    }
    let rows = ANNOUNCEMENT_STEPS
        .iter()
        .map(|_| cursor.value("row").map(str::to_owned))
        .collect::<ImportResult<Vec<_>>>()?;
    let standing = cursor.value("sponsorless-standing")?.to_owned();
    let acceptance = parse_acceptance(&mut cursor, operations, &mut exchanges, expectation)?;
    cursor.done()?;
    Ok(Payload {
        identity,
        branch,
        exchanges,
        rows,
        standing,
        acceptance,
    })
}

fn parse_payload_context(
    cursor: &mut Cursor<'_>,
    report: &MaturityReportFacts,
) -> ImportResult<(CandidateDeploymentIdentity, BranchContext)> {
    let network = cursor.bytes("deployment-network-id")?;
    let genesis = cursor.bytes("deployment-genesis-id")?;
    binding(
        "payload-network-id",
        hex_bytes(&network) == report.network_id,
    )?;
    binding(
        "payload-genesis-id",
        hex_bytes(&genesis) == report.genesis_id,
    )?;
    let identity = CandidateDeploymentIdentity::new(
        network.try_into().map_err(|_| cursor.refusal())?,
        genesis.try_into().map_err(|_| cursor.refusal())?,
    )
    .map_err(|_| cursor.refusal())?;
    let identifier = cursor.bytes("branch-identifier")?;
    let checkpoint = cursor.value("branch-checkpoint")?;
    if identifier != [0x41; 32] || checkpoint != "7" {
        return Err(cursor.refusal());
    }
    let branch = BranchContext::new(
        identifier.try_into().map_err(|_| cursor.refusal())?,
        checkpoint.parse().map_err(|_| cursor.refusal())?,
    )
    .map_err(|_| cursor.refusal())?;
    Ok((identity, branch))
}

const fn payload_agrees(field: &'static str, agrees: bool) -> ImportResult<()> {
    if agrees {
        Ok(())
    } else {
        Err(Refusal::PayloadDisagreement { field })
    }
}

fn compare_payload(
    evidence: &MaturityNativeEvidence,
    payload: &Payload,
    pins: &MaturityCorpusPins<'_>,
) -> ImportResult<()> {
    payload_agrees("deployment", evidence.identity() == &payload.identity)?;
    payload_agrees("branch", evidence.branch() == payload.branch)?;
    payload_agrees(
        "row-count",
        evidence.observations().len() == payload.rows.len(),
    )?;
    for (observation, recorded) in evidence.observations().iter().zip(&payload.rows) {
        let expected = format!(
            "{} declared {:?} observed {:?}",
            observation.subject(),
            observation.declared_layer(),
            observation.observed_layer()
        );
        payload_agrees(observation.subject(), *recorded == expected)?;
    }
    payload_agrees(
        "sponsorless-standing",
        evidence.standing() == MaturityNativeStanding::AnsweredAtDeclaredBoundary
            && payload.standing == format!("{:?}", evidence.standing()),
    )?;
    match (evidence.acceptance_obligation(), &payload.acceptance) {
        (
            MaturityAcceptanceObligation::Outstanding { routes },
            PayloadAcceptance::Outstanding {
                claim,
                routes: recorded,
            },
        ) => {
            payload_agrees("sponsorless-acceptance", claim == "outstanding")?;
            payload_agrees(
                "acceptance-routes",
                *routes
                    == [
                        MaturityAcceptanceRoute::RelayWitnessRestructure,
                        MaturityAcceptanceRoute::BlockLayerSubmissionSubject,
                    ]
                    && *recorded == routes.map(|route| format!("{route:?}")),
            )
        }
        (
            MaturityAcceptanceObligation::Established {
                schedule,
                identity,
                readback,
            },
            PayloadAcceptance::Established {
                schedule: recorded_schedule,
                identity: recorded_identity,
                witness_identity,
                block_hash,
                block_height,
            },
        ) => {
            payload_agrees(
                "accepted-schedule",
                *schedule == pins.schedule() && schedule == recorded_schedule,
            )?;
            payload_agrees(
                "accepted-identity",
                identity == recorded_identity && *identity == readback.identity(),
            )?;
            payload_agrees(
                "accepted-witness-identity",
                *witness_identity == readback.witness_identity(),
            )?;
            payload_agrees("accepted-block", block_hash == readback.block_hash())?;
            payload_agrees(
                "accepted-block-height",
                *block_height == readback.block_height(),
            )
        }
        _ => payload_agrees("sponsorless-acceptance", false),
    }
}

fn derive(
    payload: &Payload,
    schedule: tapscript::StateWitnessSchedule,
) -> ImportResult<MaturityNativeEvidence> {
    MaturityNativeEvidence::from_transcript(
        payload.identity.clone(),
        payload.branch,
        MaturityWitnessSelection::Retained(schedule),
        &payload.exchanges,
    )
    .map_err(Refusal::Derivation)
}

fn admit_capture(
    bytes: &[u8],
    report: MaturityReportFacts,
    pins: &MaturityCorpusPins<'_>,
) -> ImportResult<ValidatedMaturityCorpus> {
    binding(
        "ceremony-digest",
        hex_bytes(&tagged::sha256(bytes)) == report.ceremony_digest,
    )?;
    let mut cursor = Cursor::new("maturity capture", bytes)?;
    let capture_suite = parse_header(&mut cursor, &report, pins)?;
    let capabilities = parse_environment(&mut cursor, &report, pins)?;
    cursor.exact("digest-count 0")?;
    cursor.exact("operation-count 3")?;
    let operations = ANNOUNCEMENT_STEPS
        .iter()
        .enumerate()
        .map(|(index, name)| parse_operation(&mut cursor, index, name, pins.sponsorless()))
        .collect::<ImportResult<Vec<_>>>()?;
    let payload = parse_payload(
        &parse_tail(&mut cursor, bytes)?,
        &operations,
        &report,
        pins.sponsorless(),
    )?;
    let evidence = derive(&payload, pins.schedule())?;
    compare_payload(&evidence, &payload, pins)?;
    let recorded_refusal_detail = operations
        .last()
        .ok_or(Refusal::CorpusCensus)?
        .detail
        .clone();
    Ok(ValidatedMaturityCorpus {
        evidence,
        report,
        fee_floors: pins.fee_floors(),
        capture_suite,
        capabilities,
        exchanges: payload.exchanges,
        recorded_refusal_detail,
    })
}

fn validate_inputs(
    files: &[MaturityArchiveFile<'_>],
    pins: &MaturityCorpusPins<'_>,
) -> ImportResult<ValidatedMaturityCorpus> {
    validate_census(files, pins)?;
    validate_manifest(files, pins.manifest_sha256())?;
    let report = parse_report(files[3].bytes, pins)?;
    validate_common_bindings(files, pins, &report)?;
    admit_capture(files[0].bytes, report, pins)
}

fn validate_common_bindings(
    files: &[MaturityArchiveFile<'_>],
    pins: &MaturityCorpusPins<'_>,
    report: &MaturityReportFacts,
) -> ImportResult<()> {
    binding(
        "manifest-sha256",
        report.manifest_digest == pins.manifest_sha256(),
    )?;
    binding(
        "pinned-ceremony-digest",
        report.ceremony_digest == pins.ceremony_sha256(),
    )?;
    // The earlier admitted run exposes these deployment facts through readers.
    // Comparing them avoids a second authored network or genesis identity.
    let operator = crate::live_corpus_native_operator::run_of_record().map_err(|_| {
        Refusal::CrossFileBinding {
            field: "operator-deployment",
        }
    })?;
    // Deployment agreement borrows historical facts and establishes no current acceptance.
    let _ = operator
        .standing()
        .historical_candidate()
        .expect("historical standing");
    binding(
        "development-network-id",
        report.network_id == operator.report().network_id(),
    )?;
    binding(
        "development-genesis-id",
        report.genesis_id == operator.report().genesis_id(),
    )?;
    let mut timing = Cursor::new("maturity timing", files[1].bytes)?;
    timing.exact("timing-schema 1")?;
    timing.exact("ceremony-id report")?;
    timing.exact("status passed")?;
    timing.done()?;
    Ok(())
}

struct MutantPayload {
    identity: CandidateDeploymentIdentity,
    branch: BranchContext,
    exchanges: Vec<(OperationStep, NativeOperationResponse)>,
    records: Vec<String>,
    control: String,
}

fn parse_mutant_operation(
    cursor: &mut Cursor<'_>,
    index: usize,
    name: &'static str,
    last: usize,
) -> ImportResult<RecordedOperation> {
    let kind = if name.starts_with("mutant-") || index == last {
        SubjectKind::Submission
    } else {
        SubjectKind::Funding
    };
    let operation = parse_operation_block(cursor, index, name, kind)?;
    if (kind == SubjectKind::Funding
        && (operation.layer != ObservedOutcomeLayer::Accepted
            || operation.accepted_identity.is_some()
            || !operation.detail.is_empty()))
        || (kind == SubjectKind::Submission
            && (operation.layer == ObservedOutcomeLayer::Accepted)
                != operation.accepted_identity.is_some())
        || (index == last
            && operation.layer == ObservedOutcomeLayer::Accepted
            && !operation.detail.is_empty())
    {
        return Err(cursor.refusal());
    }
    Ok(operation)
}

fn parse_mutant_payload(
    bytes: &[u8],
    operations: &[RecordedOperation],
    roster: &[&'static str],
    report: &MaturityReportFacts,
    offer_count: usize,
) -> ImportResult<MutantPayload> {
    let mut cursor = Cursor::new("maturity mutant payload", bytes)?;
    cursor.exact("maturity-mutant-evidence-schema 1")?;
    let (identity, branch) = parse_payload_context(&mut cursor, report)?;
    let mut exchanges = Vec::with_capacity(roster.len());
    for (name, operation) in roster.iter().zip(operations) {
        cursor.exact(&format!("maturity-step {name}"))?;
        let response =
            NativeOperationResponse::from_recorded_json(&cursor.bytes("validated-response")?)
                .map_err(|_| cursor.refusal())?;
        binding("response-case", response.case == *operation.step.case())?;
        binding(
            "response-summary",
            response.observed_layer == operation.layer
                && response.observed_detail.as_deref().unwrap_or_default() == operation.detail
                && match operation.accepted_identity {
                    Some(identity) => {
                        response.accepted_txid.as_deref()
                            == Some(identity.to_target_display().as_str())
                    }
                    None => response.accepted_txid.is_none(),
                },
        )?;
        exchanges.push((operation.step.clone(), response));
    }
    let records = (0..offer_count)
        .map(|_| cursor.value("mutant-record").map(str::to_owned))
        .collect::<ImportResult<Vec<_>>>()?;
    let control = cursor.value("mutant-control")?.to_owned();
    cursor.done()?;
    Ok(MutantPayload {
        identity,
        branch,
        exchanges,
        records,
        control,
    })
}

fn compare_mutant_payload(
    payload: &MutantPayload,
    evidence: &MaturityMutantEvidence,
    roster: &[&'static str],
) -> ImportResult<()> {
    payload_agrees(
        "mutant-record-count",
        payload.records.len() == evidence.offers().len(),
    )?;
    for ((offer, recorded), step) in evidence.offers().iter().zip(&payload.records).zip(
        roster
            .iter()
            .copied()
            .filter(|step| step.starts_with("mutant-")),
    ) {
        let expected = format!(
            "{} declared {:?} observed {:?} carrier {:?} binding {:?}",
            offer.step(),
            offer.declared_boundary(),
            offer.observed_layer(),
            offer.carrier().execution(),
            evidence.binding(offer)
        );
        payload_agrees(step, *recorded == expected)?;
    }
    let control = evidence.control().map_or_else(
        || "none".to_owned(),
        |control| control.identity().to_target_display(),
    );
    payload_agrees("mutant-control", payload.control == control)
}

fn admit_mutant_capture(
    bytes: &[u8],
    report: MaturityReportFacts,
    walls: [u64; 4],
    pins: &MaturityMutantCorpusPins<'_>,
) -> ImportResult<ValidatedMaturityMutantCorpus> {
    binding(
        "ceremony-digest",
        hex_bytes(&tagged::sha256(bytes)) == report.ceremony_digest,
    )?;
    let mut cursor = Cursor::new("maturity mutant capture", bytes)?;
    let capture_suite = parse_header(&mut cursor, &report, pins.pins())?;
    let capabilities = parse_environment(&mut cursor, &report, pins.pins())?;
    cursor.exact("digest-count 1")?;
    let submission_digest = cursor
        .value("digest")?
        .strip_prefix("0 maturity-submission forward-v2 ")
        .filter(|hash| digest(hash).is_some())
        .ok_or_else(|| cursor.refusal())?
        .to_owned();
    let roster = maturity_mutant_ceremony::roster(pins.table());
    cursor.exact(&format!("operation-count {}", roster.len()))?;
    let last = roster.len() - 1;
    let operations = roster
        .iter()
        .enumerate()
        .map(|(index, name)| parse_mutant_operation(&mut cursor, index, name, last))
        .collect::<ImportResult<Vec<_>>>()?;
    let payload = parse_mutant_payload(
        &parse_tail(&mut cursor, bytes)?,
        &operations,
        roster,
        &report,
        pins.table().rows().len(),
    )?;
    let evidence = MaturityMutantEvidence::from_transcript(
        payload.identity.clone(),
        payload.branch,
        pins.table(),
        &payload.exchanges,
    )
    .map_err(Refusal::Derivation)?;
    let control_bytes = &operations
        .last()
        .ok_or(Refusal::CorpusCensus)?
        .request_bytes;
    binding(
        "maturity-submission-digest",
        submission_digest == hex_bytes(&tagged::sha256(control_bytes)),
    )?;
    compare_mutant_payload(&payload, &evidence, roster)?;
    Ok(ValidatedMaturityMutantCorpus {
        evidence,
        report,
        fee_floors: pins.pins().fee_floors(),
        capture_suite,
        exchanges: payload.exchanges,
        capabilities,
        phase_walls: walls,
    })
}

/// Admits one four-file mutant archive through shared bindings and exact ceremony replay.
///
/// # Errors
/// Returns the first byte, grammar, binding, payload, or planner refusal.
///
/// # Panics
/// Panics only if the fixed constructor architecture omits its singleton or linker bundle,
/// which these published inputs cannot arrange.
#[must_use = "mutant corpus admission can refuse a file or replay binding"]
pub fn admit_maturity_mutant_corpus<'a>(
    files: &[MaturityArchiveFile<'a>; 4],
    pins: &MaturityMutantCorpusPins<'a>,
) -> Result<ValidatedMaturityMutantCorpus, MaturityCorpusImportRefusal> {
    validate_census(files, pins.pins())?;
    validate_manifest(files, pins.pins().manifest_sha256())?;
    let (report, walls) = parse_mutant_report(files[3].bytes, pins)?;
    validate_common_bindings(files, pins.pins(), &report)?;
    admit_mutant_capture(files[0].bytes, report, walls, pins)
}

/// Admits the pinned predecessor-constructor mutant archive once.
///
/// # Errors
/// Returns the first refused admission or replay layer.
///
/// # Panics
/// Panics only if the fixed constructor architecture omits its singleton or linker bundle,
/// which these published inputs cannot arrange.
pub fn maturity_predecessor_mutant_run_of_record()
-> Result<&'static ValidatedMaturityMutantCorpus, MaturityCorpusImportRefusal> {
    static CORPUS: OnceLock<ImportResult<ValidatedMaturityMutantCorpus>> = OnceLock::new();
    match CORPUS.get_or_init(|| {
        admit_maturity_mutant_corpus(&PREDECESSOR_MUTANT_FILES, &PREDECESSOR_MUTANT_PINS)
    }) {
        Ok(corpus) => Ok(corpus),
        Err(refusal) => Err(refusal.clone()),
    }
}

/// Admits the pinned successor-constructor mutant archive once.
///
/// # Errors
/// Returns the first refused admission or replay layer.
///
/// # Panics
/// Panics only if the fixed constructor architecture omits its singleton or linker bundle,
/// which these published inputs cannot arrange.
pub fn maturity_successor_mutant_run_of_record()
-> Result<&'static ValidatedMaturityMutantCorpus, MaturityCorpusImportRefusal> {
    static CORPUS: OnceLock<ImportResult<ValidatedMaturityMutantCorpus>> = OnceLock::new();
    match CORPUS.get_or_init(|| {
        admit_maturity_mutant_corpus(&SUCCESSOR_MUTANT_FILES, &SUCCESSOR_MUTANT_PINS)
    }) {
        Ok(corpus) => Ok(corpus),
        Err(refusal) => Err(refusal.clone()),
    }
}

/// Replays a selected admitted mutant archive through the ceremony planner.
///
/// # Errors
/// Preserves admission and exact replay refusals.
///
/// # Panics
/// Panics only if the fixed constructor architecture omits its singleton or linker bundle,
/// which these published inputs cannot arrange.
pub fn replay_maturity_mutant_run_of_record(
    table: MaturityMutantTable,
) -> Result<MaturityMutantEvidence, MaturityCorpusImportRefusal> {
    let corpus = match table {
        MaturityMutantTable::PredecessorConstructor => maturity_predecessor_mutant_run_of_record()?,
        MaturityMutantTable::SuccessorConstructor => maturity_successor_mutant_run_of_record()?,
    };
    let report = corpus.report();
    let identity = CandidateDeploymentIdentity::new(
        digest(report.network_id()).ok_or(Refusal::CrossFileBinding {
            field: "development-network-id",
        })?,
        digest(report.genesis_id()).ok_or(Refusal::CrossFileBinding {
            field: "development-genesis-id",
        })?,
    )
    .map_err(|_| Refusal::CrossFileBinding {
        field: "operator-deployment",
    })?;
    let branch = BranchContext::new([0x41; 32], 7)
        .map_err(|_| Refusal::CrossFileBinding { field: "branch" })?;
    MaturityMutantEvidence::from_transcript(identity, branch, table, corpus.exchanges())
        .map_err(Refusal::Derivation)
}

/// Admits four pinned maturity files through byte bindings, closed grammars and exact planner replay.
///
/// The payload supplies no standing by authority. A refusal and an acceptance use the same replay discipline with different pinned schedules and sponsorless grammars.
///
/// # Errors
/// Returns the refusal naming the first failed admission layer, preserving planner refusals whole.
///
/// # Panics
/// Panics only if the fixed architecture omits its singleton or the public linker loses a retained constructor, which neither published input can arrange.
#[must_use = "maturity corpus admission can refuse a file or replay binding"]
pub fn admit_maturity_corpus<'a>(
    files: &[MaturityArchiveFile<'a>; 4],
    pins: &MaturityCorpusPins<'a>,
) -> Result<ValidatedMaturityCorpus, MaturityCorpusImportRefusal> {
    let span = tracing::info_span!(
        target: "vectors::corpus",
        "admit_run_of_record",
        family = "state",
        archive_kind = "maturity"
    );
    let _entered = span.enter();
    let result = validate_inputs(files, pins);
    match &result {
        Ok(corpus) => tracing::info!(
            target: "vectors::corpus",
            family = "state",
            archive_kind = "maturity",
            outcome = "completed",
            row_count = u64::try_from(corpus.exchanges().len()).unwrap_or(u64::MAX),
            "run of record admitted"
        ),
        Err(refusal) => tracing::debug!(
            target: "vectors::corpus",
            family = "state",
            archive_kind = "maturity",
            outcome = "refused",
            refusal_kind = refusal.tracing_refusal_kind(),
            "run of record admitted"
        ),
    }
    result
}

/// Replay-derived native observations; no Safety standing is changed.
pub struct ValidatedNativeMaturityCorpus {
    /// Checked four-file report identity.
    pub report: MaturityReportFacts,
    /// Exact request replay and its derived census.
    pub evidence: crate::maturity_native::native::Planner,
    /// Floors with their distinct provenance markers.
    pub fee_floors: MaturityDisclosedFeeFloors,
}

/// Admits a native capture through its own file pins and exact request replay.
///
/// # Errors
/// Returns the first byte, grammar, policy, census, route or readback disagreement.
///
/// # Panics
/// Panics only if the fixed architecture omits its singleton, which its census cannot arrange.
pub fn admit_native_maturity_corpus(
    files: &[MaturityArchiveFile<'_>; 4],
    pins: &MaturityCorpusPins<'_>,
) -> ImportResult<ValidatedNativeMaturityCorpus> {
    admit_native_with_replay(files, pins, crate::maturity_native::native::replay)
}

#[expect(
    clippy::too_many_lines,
    reason = "The admission follows the four-file binding order before replaying the schedule."
)]
fn admit_native_with_replay(
    files: &[MaturityArchiveFile<'_>; 4],
    pins: &MaturityCorpusPins<'_>,
    replay: impl FnOnce(
        CandidateDeploymentIdentity,
        BranchContext,
        &[(OperationStep, NativeOperationResponse)],
    ) -> Result<
        crate::maturity_native::native::Planner,
        crate::maturity_native::MaturityNativePlanRefusal,
    >,
) -> ImportResult<ValidatedNativeMaturityCorpus> {
    use crate::maturity_native::sponsored::Policy;
    validate_census(files, pins)?;
    validate_manifest(files, pins.manifest_sha256())?;
    let original_report =
        std::str::from_utf8(files[3].bytes).map_err(|_| Refusal::RunReportGrammar)?;
    Cursor::new("native report", files[3].bytes)?;
    let mut report_lines = original_report.lines().peekable();
    let mut normalized = String::new();
    let mut extra = String::new();
    for line in report_lines.by_ref() {
        if line == "capture-profile native" {
            extra.push_str(line);
            extra.push('\n');
            break;
        }
        normalized.push_str(line);
        normalized.push('\n');
    }
    for phase in ["preflight", "cargo", "census", "manifest"] {
        let line = report_lines.next().ok_or(Refusal::RunReportGrammar)?;
        let prefix = format!("phase-wall-ms {phase} ");
        let duration = line
            .strip_prefix(&prefix)
            .ok_or(Refusal::RunReportGrammar)?;
        number(duration).ok_or(Refusal::RunReportGrammar)?;
    }
    let elapsed = report_lines.next().ok_or(Refusal::RunReportGrammar)?;
    number(
        elapsed
            .strip_prefix("elapsed-wall-ms ")
            .ok_or(Refusal::RunReportGrammar)?,
    )
    .ok_or(Refusal::RunReportGrammar)?;
    for _ in 0..3 {
        extra.push_str(report_lines.next().ok_or(Refusal::RunReportGrammar)?);
        extra.push('\n');
    }
    if report_lines
        .peek()
        .is_some_and(|line| line.starts_with("native-waiting "))
    {
        extra.push_str(report_lines.next().ok_or(Refusal::RunReportGrammar)?);
        extra.push('\n');
    }
    for line in report_lines {
        normalized.push_str(line);
        normalized.push('\n');
    }
    let mut fields = Cursor::new("native report", extra.as_bytes())?;
    fields.exact("capture-profile native")?;
    let policy_bytes = fields.bytes("native-policy")?;
    let census_text = fields.value("native-census")?;
    let probe_bytes = fields.bytes("native-probes")?;
    let waiting_bytes = fields.bytes("native-waiting")?;
    let census = census_text
        .split(' ')
        .map(number)
        .collect::<Option<Vec<_>>>()
        .ok_or(Refusal::RunReportGrammar)?;
    fields.done()?;
    let policy = Policy::parse(
        serde_json::from_slice(&policy_bytes).map_err(|_| Refusal::RunReportGrammar)?,
    )
    .map_err(Refusal::Derivation)?;
    binding(
        "native-policy-encoding",
        serde_json::to_vec(&policy.0).map_err(|_| Refusal::RunReportGrammar)? == policy_bytes,
    )?;
    let (mut report, _) = parse_report_body(normalized.as_bytes(), pins, None)?;
    report.run_address = hex_bytes(&tagged::sha256(files[3].bytes));
    check_report_address(&report, pins)?;
    validate_common_bindings(files, pins, &report)?;
    let floors = pins.fee_floors();
    binding(
        "native-fee-floors",
        floors.min_relay_tx_fee() == policy.0["relay"].as_str().unwrap_or_default()
            && floors.block_min_tx_fee() == policy.0["block"].as_str().unwrap_or_default()
            && floors.provenance() == MaturityPremiseProvenance::ReportedByNode
            && floors.block_provenance() == MaturityPremiseProvenance::PinnedSourceNoOverride,
    )?;
    let mut cursor = Cursor::new("native capture", files[0].bytes)?;
    parse_header(&mut cursor, &report, pins)?;
    parse_environment(&mut cursor, &report, pins)?;
    cursor.exact("digest-count 0")?;
    let count = number(cursor.value("operation-count")?).ok_or_else(|| cursor.refusal())?;
    let operations = (0..count)
        .map(|index| parse_operation_block(&mut cursor, index, "", SubjectKind::Sponsored))
        .collect::<ImportResult<Vec<_>>>()?;
    let payload = parse_tail(&mut cursor, files[0].bytes)?;
    let mut cursor = Cursor::new("native payload", &payload)?;
    cursor.exact("maturity-native-evidence-schema 1")?;
    let identity = CandidateDeploymentIdentity::new(
        cursor
            .bytes("deployment-id")?
            .try_into()
            .map_err(|_| cursor.refusal())?,
        cursor
            .bytes("genesis-id")?
            .try_into()
            .map_err(|_| cursor.refusal())?,
    )
    .map_err(|_| cursor.refusal())?;
    binding(
        "native-deployment",
        hex_bytes(identity.network_id()) == report.network_id
            && hex_bytes(identity.genesis_id()) == report.genesis_id,
    )?;
    let identifier = cursor
        .bytes("branch-identifier")?
        .try_into()
        .map_err(|_| cursor.refusal())?;
    let checkpoint = cursor
        .value("branch-checkpoint")?
        .parse::<u64>()
        .map_err(|_| cursor.refusal())?;
    let branch = BranchContext::new(identifier, checkpoint).map_err(|_| cursor.refusal())?;
    binding(
        "native-policy",
        cursor.bytes("native-policy")? == policy_bytes,
    )?;
    binding(
        "native-census",
        cursor.value("native-census")? == census_text,
    )?;
    binding(
        "native-probes",
        cursor.bytes("native-probes")? == probe_bytes,
    )?;
    binding(
        "native-waiting",
        cursor.bytes("native-waiting")? == waiting_bytes,
    )?;
    let mut exchanges = Vec::new();
    for operation in operations {
        let bytes = cursor.bytes("exchange")?;
        let (case, subject, response): (
            target_elements_conformance::protocol::OperationCaseId,
            OperationSubject,
            NativeOperationResponse,
        ) = serde_json::from_slice(&bytes).map_err(|_| cursor.refusal())?;
        binding(
            "native-exchange",
            operation.step.subject() == &subject
                && response.case == case
                && operation.layer == response.observed_layer
                && operation.detail == response.observed_detail.as_deref().unwrap_or_default()
                && operation.accepted_identity.map(|id| id.to_target_display())
                    == response.accepted_txid,
        )?;
        exchanges.push((OperationStep::new(&case.step, subject), response));
    }
    cursor.done()?;
    let evidence = replay(identity, branch, &exchanges).map_err(Refusal::Derivation)?;
    binding(
        "native-derived-waiting",
        serde_json::to_vec(&evidence.waiting_json()).map_err(|_| Refusal::RunReportGrammar)?
            == waiting_bytes,
    )?;
    binding(
        "native-probe-observations",
        serde_json::to_vec(&evidence.probes()).map_err(|_| Refusal::RunReportGrammar)?
            == probe_bytes,
    )?;
    binding(
        "native-derived-census",
        evidence.census().as_slice() == census && census.first() == Some(&count),
    )?;
    binding("native-derived-policy", evidence.policy() == Some(&policy))?;
    Ok(ValidatedNativeMaturityCorpus {
        report,
        evidence,
        fee_floors: floors,
    })
}

/// Admit the immutable native archive once, including exact live request replay.
/// # Errors
/// Preserves byte, provenance, schedule and readback disagreements.
/// # Panics
/// Panics only if the fixed architecture omits its singleton.
pub fn maturity_native_run_of_record() -> ImportResult<&'static ValidatedNativeMaturityCorpus> {
    static CORPUS: OnceLock<ImportResult<ValidatedNativeMaturityCorpus>> = OnceLock::new();
    match CORPUS.get_or_init(|| admit_native_maturity_corpus(&NATIVE_FILES, &NATIVE_PINS)) {
        Ok(corpus) => Ok(corpus),
        Err(refusal) => Err(refusal.clone()),
    }
}

/// Admitted retained observations, with both families validated before any fold.
pub struct ValidatedRetainedMaturityCorpus {
    /// Exact four-file report provenance.
    pub report: MaturityReportFacts,
    /// Complete exact replay of both retained families.
    pub evidence: crate::maturity_native::native::RetainedRoutesPlanner,
    /// Node-default policy premises with distinct provenance.
    pub fee_floors: MaturityDisclosedFeeFloors,
}

struct RetainedReportFields {
    policy: Vec<u8>,
    families: Vec<u8>,
    stopped: Vec<u8>,
    rows: [Vec<u8>; 2],
}

fn retained_json(bytes: &[u8]) -> ImportResult<serde_json::Value> {
    let value: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|_| Refusal::RunReportGrammar)?;
    binding(
        "retained-json-encoding",
        serde_json::to_vec(&value).map_err(|_| Refusal::RunReportGrammar)? == bytes,
    )?;
    Ok(value)
}

fn validate_retained_fields(fields: &RetainedReportFields) -> ImportResult<()> {
    let families = retained_json(&fields.families)?;
    let stopped = retained_json(&fields.stopped)?;
    for (index, name) in ["wrong-semantic-metadata", "two-state-successors"]
        .iter()
        .enumerate()
    {
        let fail = |field| {
            Refusal::RetainedCampaign(crate::maturity_native::native::RetainedCampaignRefusal {
                family: (*name).to_owned(),
                field,
            })
        };
        let family = families.get(index).ok_or_else(|| fail("absent family"))?;
        if family["family"] != *name {
            return Err(fail("family"));
        }
        if stopped
            .as_array()
            .is_some_and(|items| items.iter().any(|value| value.as_str() == Some(*name)))
            || family["stopped"] == true
        {
            return Err(fail("stopped"));
        }
        if family["subject"].is_null() {
            return Err(fail("absent subject"));
        }
        if family["expectation"] != "agreement" {
            return Err(fail("expectation"));
        }
        if family["control_accepted"] != true
            || family["control"]["layer"] != "accepted"
            || family["control"]["readback"] != "matched"
        {
            return Err(fail("accepted control"));
        }
    }
    binding(
        "retained-family-census",
        families.as_array().is_some_and(|values| values.len() == 2),
    )?;
    binding("retained-stopped-empty", stopped == serde_json::json!([]))
}

fn parse_retained_report(
    bytes: &[u8],
    pins: &MaturityCorpusPins<'_>,
) -> ImportResult<(MaturityReportFacts, RetainedReportFields)> {
    let mut cursor = Cursor::new("retained report", bytes)?;
    let profile = cursor
        .lines
        .iter()
        .position(|line| *line == "capture-profile retained")
        .ok_or(Refusal::RunReportGrammar)?;
    let mut normalized = cursor.lines[..profile].join("\n");
    normalized.push('\n');
    cursor.index = profile;
    cursor.exact("capture-profile retained")?;
    for phase in ["preflight", "cargo", "census", "manifest"] {
        let wall = cursor.value("phase-wall-ms")?;
        let digits = wall
            .strip_prefix(phase)
            .and_then(|tail| tail.strip_prefix(' '))
            .ok_or(Refusal::RunReportGrammar)?;
        number(digits).ok_or(Refusal::RunReportGrammar)?;
    }
    number(cursor.value("elapsed-wall-ms")?).ok_or(Refusal::RunReportGrammar)?;
    let policy = cursor.bytes("retained-policy")?;
    let families = cursor.bytes("retained-families")?;
    let stopped = cursor.bytes("retained-stopped")?;
    cursor.exact("campaign-manifest-schema 1")?;
    cursor.exact("campaign-manifest-count 2")?;
    let rows = [cursor.bytes("campaign-row")?, cursor.bytes("campaign-row")?];
    for line in &cursor.lines[cursor.index..] {
        normalized.push_str(line);
        normalized.push('\n');
    }
    let (mut report, _) = parse_report_body_selected(
        normalized.as_bytes(),
        pins,
        None,
        Some(pins.capture_target().test()),
    )?;
    binding(
        "retained-node-tip",
        report.expected_tip == RETAINED_NODE_TIP,
    )?;
    report.run_address = hex_bytes(&tagged::sha256(bytes));
    let fields = RetainedReportFields {
        policy,
        families,
        stopped,
        rows,
    };
    validate_retained_fields(&fields)?;
    Ok((report, fields))
}

/// Admit the retained archive before assigning either family's standing.
/// # Errors
/// Refuses byte, provenance, campaign, policy, route and readback disagreements.
/// # Panics
/// Panics only if the fixed architecture omits its singleton or retained row.
pub fn admit_retained_maturity_corpus(
    files: &[MaturityArchiveFile<'_>; 4],
    pins: &MaturityCorpusPins<'_>,
) -> ImportResult<ValidatedRetainedMaturityCorpus> {
    admit_retained_with_replay(
        files,
        pins,
        crate::maturity_native::native::replay_retained_routes,
    )
}

#[expect(
    clippy::too_many_lines,
    reason = "Admission follows the archive bindings before exact replay."
)]
fn admit_retained_with_replay(
    files: &[MaturityArchiveFile<'_>; 4],
    pins: &MaturityCorpusPins<'_>,
    replay: impl FnOnce(
        CandidateDeploymentIdentity,
        BranchContext,
        &[(OperationStep, NativeOperationResponse)],
    ) -> Result<
        crate::maturity_native::native::RetainedRoutesPlanner,
        crate::maturity_native::MaturityNativePlanRefusal,
    >,
) -> ImportResult<ValidatedRetainedMaturityCorpus> {
    use crate::maturity_native::sponsored::Policy;
    validate_census(files, pins)?;
    validate_manifest(files, pins.manifest_sha256())?;
    binding(
        "retained-schedule",
        pins.schedule() == tapscript::StateWitnessSchedule::VariableMetadata,
    )?;
    let (report, fields) = parse_retained_report(files[3].bytes, pins)?;
    check_report_address(&report, pins)?;
    let policy_bytes = &fields.policy;
    let policy = Policy::parse(retained_json(policy_bytes)?).map_err(Refusal::Derivation)?;
    validate_common_bindings(files, pins, &report)?;
    let floors = pins.fee_floors();
    binding(
        "retained-fee-floors",
        floors.min_relay_tx_fee() == policy.0["relay"].as_str().unwrap_or_default()
            && floors.block_min_tx_fee() == policy.0["block"].as_str().unwrap_or_default()
            && floors.provenance() == MaturityPremiseProvenance::ReportedByNode
            && floors.block_provenance() == MaturityPremiseProvenance::PinnedSourceNoOverride,
    )?;
    let mut cursor = Cursor::new("retained capture", files[0].bytes)?;
    parse_header(&mut cursor, &report, pins)?;
    parse_environment(&mut cursor, &report, pins)?;
    cursor.exact("digest-count 0")?;
    let count = number(cursor.value("operation-count")?).ok_or_else(|| cursor.refusal())?;
    let operations = (0..count)
        .map(|index| parse_operation_block(&mut cursor, index, "", SubjectKind::Sponsored))
        .collect::<ImportResult<Vec<_>>>()?;
    let payload = parse_tail(&mut cursor, files[0].bytes)?;
    let mut cursor = Cursor::new("retained payload", &payload)?;
    cursor.exact("maturity-retained-evidence-schema 1")?;
    let identity = CandidateDeploymentIdentity::new(
        cursor
            .bytes("deployment-id")?
            .try_into()
            .map_err(|_| cursor.refusal())?,
        cursor
            .bytes("genesis-id")?
            .try_into()
            .map_err(|_| cursor.refusal())?,
    )
    .map_err(|_| cursor.refusal())?;
    binding(
        "retained-deployment",
        hex_bytes(identity.network_id()) == report.network_id
            && hex_bytes(identity.genesis_id()) == report.genesis_id,
    )?;
    let identifier = cursor
        .bytes("branch-identifier")?
        .try_into()
        .map_err(|_| cursor.refusal())?;
    let checkpoint = cursor
        .value("branch-checkpoint")?
        .parse::<u64>()
        .map_err(|_| cursor.refusal())?;
    let branch = BranchContext::new(identifier, checkpoint).map_err(|_| cursor.refusal())?;
    binding(
        "retained-policy",
        cursor.bytes("retained-policy")? == fields.policy,
    )?;
    binding(
        "retained-families",
        cursor.bytes("retained-families")? == fields.families,
    )?;
    binding(
        "retained-stopped",
        cursor.bytes("retained-stopped")? == fields.stopped,
    )?;
    cursor.exact("campaign-manifest-schema 1")?;
    cursor.exact("campaign-manifest-count 2")?;
    for row in &fields.rows {
        binding(
            "retained-campaign-row",
            cursor.bytes("campaign-row")? == *row,
        )?;
    }
    let mut exchanges = Vec::new();
    for operation in operations {
        let bytes = cursor.bytes("exchange")?;
        let (case, subject, response): (
            target_elements_conformance::protocol::OperationCaseId,
            OperationSubject,
            NativeOperationResponse,
        ) = serde_json::from_slice(&bytes).map_err(|_| cursor.refusal())?;
        binding(
            "retained-exchange",
            operation.step.subject() == &subject
                && response.case == case
                && operation.layer == response.observed_layer
                && operation.detail == response.observed_detail.as_deref().unwrap_or_default()
                && operation.accepted_identity.map(|id| id.to_target_display())
                    == response.accepted_txid,
        )?;
        exchanges.push((OperationStep::new(&case.step, subject), response));
    }
    cursor.done()?;
    let evidence = replay(identity, branch, &exchanges).map_err(Refusal::Derivation)?;
    binding(
        "retained-derived-families",
        serde_json::to_vec(&evidence.families_json()).map_err(|_| Refusal::RunReportGrammar)?
            == fields.families,
    )?;
    binding(
        "retained-derived-stopped",
        serde_json::to_vec(&evidence.stopped()).map_err(|_| Refusal::RunReportGrammar)?
            == fields.stopped,
    )?;
    binding(
        "retained-derived-policy",
        evidence.policy() == Some(&policy),
    )?;
    binding(
        "retained-operation-census",
        evidence.exchanges().len() == count,
    )?;
    let rows = fields
        .rows
        .iter()
        .map(|row| retained_json(row))
        .collect::<ImportResult<Vec<_>>>()?;
    crate::maturity_native::native::RetainedCampaignManifest::validate(&evidence, &rows)
        .map_err(Refusal::RetainedCampaign)?;
    Ok(ValidatedRetainedMaturityCorpus {
        report,
        evidence,
        fee_floors: floors,
    })
}

/// Admit the immutable retained archive once per process.
/// # Errors
/// Preserves the first exact admission refusal.
/// # Panics
/// Panics only if the fixed architecture omits its singleton or retained row.
pub fn maturity_retained_run_of_record() -> ImportResult<&'static ValidatedRetainedMaturityCorpus> {
    static CORPUS: OnceLock<ImportResult<ValidatedRetainedMaturityCorpus>> = OnceLock::new();
    match CORPUS.get_or_init(|| admit_retained_maturity_corpus(&RETAINED_FILES, &RETAINED_PINS)) {
        Ok(corpus) => Ok(corpus),
        Err(error) => Err(error.clone()),
    }
}

/// Admitted diagnostic observations, with all seven families validated before any fold.
pub struct ValidatedDiagnosticMaturityCorpus {
    /// Exact four-file report provenance.
    pub report: MaturityReportFacts,
    /// Complete exact replay of all seven diagnostic families.
    pub evidence: crate::maturity_native::native::DiagnosticRoutesPlanner,
    /// Node-default policy premises with distinct provenance.
    pub fee_floors: MaturityDisclosedFeeFloors,
}

struct DiagnosticReportFields {
    policy: Vec<u8>,
    families: Vec<u8>,
    stopped: Vec<u8>,
    rows: Vec<Vec<u8>>,
}

fn validate_diagnostic_fields(fields: &DiagnosticReportFields) -> ImportResult<()> {
    let families = retained_json(&fields.families)?;
    let stopped = retained_json(&fields.stopped)?;
    let names = [
        "window",
        "metadata",
        "operator",
        "predecessor",
        "successor",
        "abi",
        "wrong-semantic-metadata",
    ];
    binding(
        "diagnostic-family-census",
        families
            .as_array()
            .is_some_and(|values| values.len() == names.len()),
    )?;
    binding("diagnostic-stopped-empty", stopped == serde_json::json!([]))?;
    for (index, name) in names.iter().enumerate() {
        let fail = |field| {
            Refusal::RetainedCampaign(crate::maturity_native::native::RetainedCampaignRefusal {
                family: (*name).to_owned(),
                field,
            })
        };
        let family = &families[index];
        if family["family"] != *name {
            return Err(fail("family"));
        }
        if family["stopped"] != false {
            return Err(fail("stopped"));
        }
        if family["expectation"] != "agreement" {
            return Err(fail("expectation"));
        }
        if family["control_accepted"] != true
            || family["control"]["layer"] != "accepted"
            || family["control"]["readback"] != "matched"
        {
            return Err(fail("accepted control"));
        }
        let subjects = family["subjects"]
            .as_array()
            .ok_or_else(|| fail("absent subject"))?;
        if subjects.len() != [2, 1, 6, 4, 6, 4, 1][index] {
            return Err(fail("subject census"));
        }
        for subject in subjects {
            let expected = if subject["subject"] == "wrong-sequence" {
                "consensus_rejection_before_script"
            } else {
                "script_path_rejection"
            };
            if subject["observation"].is_null() {
                return Err(fail("absent subject"));
            }
            if subject["expectation"] != "agreement" || subject["observation"]["layer"] != expected
            {
                return Err(fail("subject expectation"));
            }
        }
    }
    Ok(())
}

fn parse_diagnostic_report(
    bytes: &[u8],
    pins: &MaturityCorpusPins<'_>,
) -> ImportResult<(MaturityReportFacts, DiagnosticReportFields)> {
    let mut cursor = Cursor::new("diagnostic report", bytes)?;
    let profile = cursor
        .lines
        .iter()
        .position(|line| *line == "capture-profile diagnostic")
        .ok_or(Refusal::RunReportGrammar)?;
    let mut normalized = cursor.lines[..profile].join("\n");
    normalized.push('\n');
    cursor.index = profile;
    cursor.exact("capture-profile diagnostic")?;
    for phase in ["preflight", "cargo", "census", "manifest"] {
        let wall = cursor.value("phase-wall-ms")?;
        let digits = wall
            .strip_prefix(phase)
            .and_then(|tail| tail.strip_prefix(' '))
            .ok_or(Refusal::RunReportGrammar)?;
        number(digits).ok_or(Refusal::RunReportGrammar)?;
    }
    number(cursor.value("elapsed-wall-ms")?).ok_or(Refusal::RunReportGrammar)?;
    let policy = cursor.bytes("diagnostic-policy")?;
    let families = cursor.bytes("diagnostic-families")?;
    let stopped = cursor.bytes("diagnostic-stopped")?;
    cursor.exact("campaign-manifest-schema 1")?;
    cursor.exact("campaign-manifest-count 23")?;
    let rows = (0..23)
        .map(|_| cursor.bytes("campaign-row"))
        .collect::<ImportResult<Vec<_>>>()?;
    for line in &cursor.lines[cursor.index..] {
        normalized.push_str(line);
        normalized.push('\n');
    }
    let (mut report, _) = parse_report_body_selected(
        normalized.as_bytes(),
        pins,
        None,
        Some(pins.capture_target().test()),
    )?;
    binding(
        "diagnostic-node-tip",
        report.expected_tip == RETAINED_NODE_TIP,
    )?;
    report.run_address = hex_bytes(&tagged::sha256(bytes));
    let fields = DiagnosticReportFields {
        policy,
        families,
        stopped,
        rows,
    };
    validate_diagnostic_fields(&fields)?;
    Ok((report, fields))
}

/// Admit the complete diagnostic archive without assigning any standing.
/// # Errors
/// Refuses byte, provenance, campaign, policy, route and readback disagreements.
/// # Panics
/// Panics only if the fixed architecture omits its singleton or diagnostic row.
pub fn admit_diagnostic_maturity_corpus(
    files: &[MaturityArchiveFile<'_>; 4],
    pins: &MaturityCorpusPins<'_>,
) -> ImportResult<ValidatedDiagnosticMaturityCorpus> {
    admit_diagnostic_with_replay(
        files,
        pins,
        crate::maturity_native::native::replay_diagnostic_routes,
    )
}

#[expect(
    clippy::too_many_lines,
    reason = "Admission follows the archive bindings before exact replay."
)]
fn admit_diagnostic_with_replay(
    files: &[MaturityArchiveFile<'_>; 4],
    pins: &MaturityCorpusPins<'_>,
    replay: impl FnOnce(
        CandidateDeploymentIdentity,
        BranchContext,
        &[(OperationStep, NativeOperationResponse)],
    ) -> Result<
        crate::maturity_native::native::DiagnosticRoutesPlanner,
        crate::maturity_native::MaturityNativePlanRefusal,
    >,
) -> ImportResult<ValidatedDiagnosticMaturityCorpus> {
    use crate::maturity_native::sponsored::Policy;
    validate_census(files, pins)?;
    validate_manifest(files, pins.manifest_sha256())?;
    binding(
        "diagnostic-schedule",
        pins.schedule() == tapscript::StateWitnessSchedule::VariableMetadata,
    )?;
    let (report, fields) = parse_diagnostic_report(files[3].bytes, pins)?;
    check_report_address(&report, pins)?;
    let policy_bytes = &fields.policy;
    let policy = Policy::parse(retained_json(policy_bytes)?).map_err(Refusal::Derivation)?;
    validate_common_bindings(files, pins, &report)?;
    let floors = pins.fee_floors();
    binding(
        "diagnostic-fee-floors",
        floors.min_relay_tx_fee() == policy.0["relay"].as_str().unwrap_or_default()
            && floors.block_min_tx_fee() == policy.0["block"].as_str().unwrap_or_default()
            && floors.provenance() == MaturityPremiseProvenance::ReportedByNode
            && floors.block_provenance() == MaturityPremiseProvenance::PinnedSourceNoOverride,
    )?;
    let mut cursor = Cursor::new("diagnostic capture", files[0].bytes)?;
    parse_header(&mut cursor, &report, pins)?;
    parse_environment(&mut cursor, &report, pins)?;
    cursor.exact("digest-count 0")?;
    let count = number(cursor.value("operation-count")?).ok_or_else(|| cursor.refusal())?;
    let operations = (0..count)
        .map(|index| parse_operation_block(&mut cursor, index, "", SubjectKind::Sponsored))
        .collect::<ImportResult<Vec<_>>>()?;
    let payload = parse_tail(&mut cursor, files[0].bytes)?;
    let mut cursor = Cursor::new("diagnostic payload", &payload)?;
    cursor.exact("maturity-diagnostic-evidence-schema 1")?;
    let identity = CandidateDeploymentIdentity::new(
        cursor
            .bytes("deployment-id")?
            .try_into()
            .map_err(|_| cursor.refusal())?,
        cursor
            .bytes("genesis-id")?
            .try_into()
            .map_err(|_| cursor.refusal())?,
    )
    .map_err(|_| cursor.refusal())?;
    binding(
        "diagnostic-deployment",
        hex_bytes(identity.network_id()) == report.network_id
            && hex_bytes(identity.genesis_id()) == report.genesis_id,
    )?;
    let identifier = cursor
        .bytes("branch-identifier")?
        .try_into()
        .map_err(|_| cursor.refusal())?;
    let checkpoint = cursor
        .value("branch-checkpoint")?
        .parse::<u64>()
        .map_err(|_| cursor.refusal())?;
    let branch = BranchContext::new(identifier, checkpoint).map_err(|_| cursor.refusal())?;
    binding(
        "diagnostic-policy",
        cursor.bytes("diagnostic-policy")? == fields.policy,
    )?;
    binding(
        "diagnostic-families",
        cursor.bytes("diagnostic-families")? == fields.families,
    )?;
    binding(
        "diagnostic-stopped",
        cursor.bytes("diagnostic-stopped")? == fields.stopped,
    )?;
    cursor.exact("campaign-manifest-schema 1")?;
    cursor.exact("campaign-manifest-count 23")?;
    for row in &fields.rows {
        binding(
            "diagnostic-campaign-row",
            cursor.bytes("campaign-row")? == *row,
        )?;
    }
    let mut exchanges = Vec::new();
    for operation in operations {
        let bytes = cursor.bytes("exchange")?;
        let (case, subject, response): (
            target_elements_conformance::protocol::OperationCaseId,
            OperationSubject,
            NativeOperationResponse,
        ) = serde_json::from_slice(&bytes).map_err(|_| cursor.refusal())?;
        binding(
            "diagnostic-exchange",
            operation.step.subject() == &subject
                && response.case == case
                && operation.layer == response.observed_layer
                && operation.detail == response.observed_detail.as_deref().unwrap_or_default()
                && operation.accepted_identity.map(|id| id.to_target_display())
                    == response.accepted_txid,
        )?;
        exchanges.push((OperationStep::new(&case.step, subject), response));
    }
    cursor.done()?;
    let evidence = replay(identity, branch, &exchanges).map_err(Refusal::Derivation)?;
    binding(
        "diagnostic-derived-families",
        serde_json::to_vec(&evidence.families_json()).map_err(|_| Refusal::RunReportGrammar)?
            == fields.families,
    )?;
    binding(
        "diagnostic-derived-stopped",
        serde_json::to_vec(&evidence.stopped()).map_err(|_| Refusal::RunReportGrammar)?
            == fields.stopped,
    )?;
    binding(
        "diagnostic-derived-policy",
        evidence.policy() == Some(&policy),
    )?;
    binding(
        "diagnostic-operation-census",
        count == 48 && evidence.exchanges().len() == count,
    )?;
    let rows = fields
        .rows
        .iter()
        .map(|row| retained_json(row))
        .collect::<ImportResult<Vec<_>>>()?;
    crate::maturity_native::native::DiagnosticCampaignManifest::validate(&evidence, &rows)
        .map_err(Refusal::RetainedCampaign)?;
    Ok(ValidatedDiagnosticMaturityCorpus {
        report,
        evidence,
        fee_floors: floors,
    })
}

/// Admit the immutable diagnostic archive once per process.
/// # Errors
/// Preserves the first exact admission refusal.
/// # Panics
/// Panics only if the fixed architecture omits its singleton or diagnostic row.
pub fn maturity_diagnostic_run_of_record()
-> ImportResult<&'static ValidatedDiagnosticMaturityCorpus> {
    static CORPUS: OnceLock<ImportResult<ValidatedDiagnosticMaturityCorpus>> = OnceLock::new();
    match CORPUS
        .get_or_init(|| admit_diagnostic_maturity_corpus(&DIAGNOSTIC_FILES, &DIAGNOSTIC_PINS))
    {
        Ok(corpus) => Ok(corpus),
        Err(error) => Err(error.clone()),
    }
}

/// Admitted confidential observations, with the confidential family validated before any fold.
pub struct ValidatedConfidentialMaturityCorpus {
    /// Exact four-file report provenance.
    pub report: MaturityReportFacts,
    /// Complete exact replay of the confidential family.
    pub evidence: crate::maturity_native::confidential::Planner,
    /// Node-default policy premises with distinct provenance.
    pub fee_floors: MaturityDisclosedFeeFloors,
}

struct ConfidentialReportFields {
    policy: Vec<u8>,
    families: Vec<u8>,
    stopped: Vec<u8>,
    rows: [Vec<u8>; 1],
    preflight: Vec<u8>,
}

impl ConfidentialReportFields {
    fn decoded(
        &self,
    ) -> ImportResult<crate::maturity_native::confidential::ConfidentialCampaignFields> {
        Ok(
            crate::maturity_native::confidential::ConfidentialCampaignFields {
                policy: retained_json(&self.policy)?,
                families: retained_json(&self.families)?,
                stopped: retained_json(&self.stopped)?,
                preflight: retained_json(&self.preflight)?,
                rows: self
                    .rows
                    .iter()
                    .map(|row| retained_json(row))
                    .collect::<ImportResult<Vec<_>>>()?,
            },
        )
    }
}

fn parse_confidential_report(
    bytes: &[u8],
    pins: &MaturityCorpusPins<'_>,
) -> ImportResult<(MaturityReportFacts, ConfidentialReportFields)> {
    let mut cursor = Cursor::new("confidential report", bytes)?;
    let profile = cursor
        .lines
        .iter()
        .position(|line| *line == "capture-profile confidential")
        .ok_or(Refusal::RunReportGrammar)?;
    let mut normalized = cursor.lines[..profile].join("\n");
    normalized.push('\n');
    cursor.index = profile;
    cursor.exact("capture-profile confidential")?;
    for phase in ["preflight", "cargo", "census", "manifest"] {
        let wall = cursor.value("phase-wall-ms")?;
        let digits = wall
            .strip_prefix(phase)
            .and_then(|tail| tail.strip_prefix(' '))
            .ok_or(Refusal::RunReportGrammar)?;
        number(digits).ok_or(Refusal::RunReportGrammar)?;
    }
    number(cursor.value("elapsed-wall-ms")?).ok_or(Refusal::RunReportGrammar)?;
    let policy = cursor.bytes("confidential-policy")?;
    let families = cursor.bytes("confidential-families")?;
    let stopped = cursor.bytes("confidential-stopped")?;
    cursor.exact("campaign-manifest-schema 1")?;
    cursor.exact("campaign-manifest-count 1")?;
    let rows = [cursor.bytes("campaign-row")?];
    for _ in 0..4 {
        let line = cursor
            .lines
            .get(cursor.index)
            .ok_or_else(|| cursor.refusal())?;
        normalized.push_str(line);
        cursor.index += 1;
        normalized.push('\n');
    }
    let preflight = cursor.bytes("confidential-preflight")?;
    if cursor.lines.get(cursor.index) == Some(&"eligible no") {
        return Err(Refusal::ConfidentialCampaign(
            crate::maturity_native::confidential::ConfidentialCampaignRefusal {
                family: crate::maturity_native::confidential::FAMILY.to_owned(),
                field: "eligible",
            },
        ));
    }
    for line in &cursor.lines[cursor.index..] {
        normalized.push_str(line);
        normalized.push('\n');
    }
    let (mut report, _) = parse_report_body_selected(
        normalized.as_bytes(),
        pins,
        None,
        Some(pins.capture_target().test()),
    )?;
    binding(
        "confidential-node-tip",
        report.expected_tip == RETAINED_NODE_TIP,
    )?;
    report.run_address = hex_bytes(&tagged::sha256(bytes));
    let fields = ConfidentialReportFields {
        policy,
        families,
        stopped,
        rows,
        preflight,
    };
    fields
        .decoded()?
        .require_agreement()
        .map_err(Refusal::ConfidentialCampaign)?;
    Ok((report, fields))
}

/// Admit the confidential archive before assigning the family's standing.
/// # Errors
/// Refuses byte, provenance, campaign, policy, route and readback disagreements.
/// # Panics
/// Panics only if the fixed architecture omits its singleton or confidential row.
pub fn admit_confidential_maturity_corpus(
    files: &[MaturityArchiveFile<'_>; 4],
    pins: &MaturityCorpusPins<'_>,
) -> ImportResult<ValidatedConfidentialMaturityCorpus> {
    admit_confidential_with_replay(
        files,
        pins,
        crate::maturity_native::confidential::replay_confidential_route,
    )
}

#[expect(
    clippy::too_many_lines,
    reason = "Admission follows the archive bindings before exact replay."
)]
fn admit_confidential_with_replay(
    files: &[MaturityArchiveFile<'_>; 4],
    pins: &MaturityCorpusPins<'_>,
    replay: impl FnOnce(
        CandidateDeploymentIdentity,
        BranchContext,
        &[(OperationStep, NativeOperationResponse)],
    ) -> Result<
        crate::maturity_native::confidential::Planner,
        crate::maturity_native::confidential::RouteRefusal,
    >,
) -> ImportResult<ValidatedConfidentialMaturityCorpus> {
    use crate::maturity_native::sponsored::Policy;
    validate_census(files, pins)?;
    validate_manifest(files, pins.manifest_sha256())?;
    binding(
        "confidential-schedule",
        pins.schedule() == tapscript::StateWitnessSchedule::VariableMetadata,
    )?;
    let (report, fields) = parse_confidential_report(files[3].bytes, pins)?;
    check_report_address(&report, pins)?;
    let policy_bytes = &fields.policy;
    let policy = Policy::parse(retained_json(policy_bytes)?).map_err(Refusal::Derivation)?;
    validate_common_bindings(files, pins, &report)?;
    let floors = pins.fee_floors();
    binding(
        "confidential-fee-floors",
        floors.min_relay_tx_fee() == policy.0["relay"].as_str().unwrap_or_default()
            && floors.block_min_tx_fee() == policy.0["block"].as_str().unwrap_or_default()
            && floors.provenance() == MaturityPremiseProvenance::ReportedByNode
            && floors.block_provenance() == MaturityPremiseProvenance::PinnedSourceNoOverride,
    )?;
    let mut cursor = Cursor::new("confidential capture", files[0].bytes)?;
    parse_header(&mut cursor, &report, pins)?;
    parse_environment(&mut cursor, &report, pins)?;
    cursor.exact("digest-count 0")?;
    let count = number(cursor.value("operation-count")?).ok_or_else(|| cursor.refusal())?;
    let operations = (0..count)
        .map(|index| parse_operation_block(&mut cursor, index, "", SubjectKind::Confidential))
        .collect::<ImportResult<Vec<_>>>()?;
    let payload = parse_tail(&mut cursor, files[0].bytes)?;
    let mut cursor = Cursor::new("confidential payload", &payload)?;
    cursor.exact("maturity-confidential-evidence-schema 1")?;
    let identity = CandidateDeploymentIdentity::new(
        cursor
            .bytes("deployment-id")?
            .try_into()
            .map_err(|_| cursor.refusal())?,
        cursor
            .bytes("genesis-id")?
            .try_into()
            .map_err(|_| cursor.refusal())?,
    )
    .map_err(|_| cursor.refusal())?;
    binding(
        "confidential-deployment",
        hex_bytes(identity.network_id()) == report.network_id
            && hex_bytes(identity.genesis_id()) == report.genesis_id,
    )?;
    let identifier = cursor
        .bytes("branch-identifier")?
        .try_into()
        .map_err(|_| cursor.refusal())?;
    let checkpoint = cursor
        .value("branch-checkpoint")?
        .parse::<u64>()
        .map_err(|_| cursor.refusal())?;
    let branch = BranchContext::new(identifier, checkpoint).map_err(|_| cursor.refusal())?;
    binding(
        "confidential-policy",
        cursor.bytes("confidential-policy")? == fields.policy,
    )?;
    binding(
        "confidential-families",
        cursor.bytes("confidential-families")? == fields.families,
    )?;
    binding(
        "confidential-stopped",
        cursor.bytes("confidential-stopped")? == fields.stopped,
    )?;
    binding(
        "confidential-preflight",
        cursor.bytes("confidential-preflight")? == fields.preflight,
    )?;
    cursor.exact("campaign-manifest-schema 1")?;
    cursor.exact("campaign-manifest-count 1")?;
    for row in &fields.rows {
        binding(
            "confidential-campaign-row",
            cursor.bytes("campaign-row")? == *row,
        )?;
    }
    let mut exchanges = Vec::new();
    for operation in operations {
        let bytes = cursor.bytes("exchange")?;
        let (case, subject, response): (
            target_elements_conformance::protocol::OperationCaseId,
            OperationSubject,
            NativeOperationResponse,
        ) = serde_json::from_slice(&bytes).map_err(|_| cursor.refusal())?;
        binding(
            "confidential-exchange",
            operation.step.subject() == &subject
                && response.case == case
                && operation.layer == response.observed_layer
                && operation.detail == response.observed_detail.as_deref().unwrap_or_default()
                && operation.accepted_identity.map(|id| id.to_target_display())
                    == response.accepted_txid,
        )?;
        exchanges.push((OperationStep::new(&case.step, subject), response));
    }
    cursor.done()?;
    let evidence = replay(identity, branch, &exchanges).map_err(|error| {
        Refusal::ConfidentialCampaign(
            crate::maturity_native::confidential::ConfidentialCampaignRefusal {
                family: crate::maturity_native::confidential::FAMILY.to_owned(),
                field: error.field,
            },
        )
    })?;
    binding(
        "confidential-operation-census",
        count == 14 && evidence.exchanges().len() == count,
    )?;
    let decoded = fields.decoded()?;
    decoded
        .admit(&evidence)
        .map_err(Refusal::ConfidentialCampaign)?;
    Ok(ValidatedConfidentialMaturityCorpus {
        report,
        evidence,
        fee_floors: floors,
    })
}

/// Admit the immutable confidential archive once per process.
/// # Errors
/// Preserves the first exact admission refusal.
/// # Panics
/// Panics only if the fixed architecture omits its singleton or confidential row.
pub fn maturity_confidential_run_of_record()
-> ImportResult<&'static ValidatedConfidentialMaturityCorpus> {
    static CORPUS: OnceLock<ImportResult<ValidatedConfidentialMaturityCorpus>> = OnceLock::new();
    match CORPUS
        .get_or_init(|| admit_confidential_maturity_corpus(&CONFIDENTIAL_FILES, &CONFIDENTIAL_PINS))
    {
        Ok(corpus) => Ok(corpus),
        Err(error) => Err(error.clone()),
    }
}

/// Admitted independent acquisition and the two separately validated reports.
pub struct ValidatedObserverMaturityCorpus {
    observation: serde_json::Value,
}

impl ValidatedObserverMaturityCorpus {
    /// Exact checked experiment, including its declarations and node answers.
    #[must_use]
    pub const fn observation(&self) -> &serde_json::Value {
        &self.observation
    }
}

fn observer_check(agrees: bool, field: &str) -> Result<(), String> {
    agrees
        .then_some(())
        .ok_or_else(|| format!("observer-archive:{field}"))
}

fn observer_text<'a>(value: &'a serde_json::Value, field: &str) -> Result<&'a str, String> {
    value[field]
        .as_str()
        .ok_or_else(|| format!("observer-text:{field}"))
}

fn observer_subject(capture: &str) -> Result<Vec<u8>, String> {
    let lines: Vec<_> = capture
        .lines()
        .filter(|line| line.starts_with("request-bytes ") && !line.starts_with("request-bytes 0 "))
        .collect();
    observer_check(lines.len() == 1, "one-submission")?;
    let bytes = lines[0]
        .strip_prefix("request-bytes 1701 ")
        .ok_or("observer-subject-length")?;
    crate::maturity_observer::decode_hex(bytes)
}

fn observer_file_binding(
    files: &[MaturityArchiveFile<'_>; 4],
    pins: [&str; 3],
) -> Result<(), String> {
    observer_check(
        hex_bytes(&tagged::sha256(files[2].bytes)) == pins[0],
        "manifest-pin",
    )?;
    observer_check(
        hex_bytes(&tagged::sha256(files[3].bytes)) == pins[1],
        "report-pin",
    )?;
    observer_check(
        hex_bytes(&tagged::sha256(files[0].bytes)) == pins[2],
        "capture-pin",
    )?;
    observer_check(
        files[2].name == "MANIFEST.sha256"
            && files[3].name == "RUN-REPORT"
            && files[1].name == format!("{}.timing", files[0].name),
        "file-roster",
    )?;
    let manifest = format!(
        "{}  {}\n{}  {}\n",
        pins[2],
        files[0].name,
        hex_bytes(&tagged::sha256(files[1].bytes)),
        files[1].name
    );
    observer_check(files[2].bytes == manifest.as_bytes(), "manifest-content")?;
    observer_check(
        files[1].bytes == b"timing-schema 1\nceremony-id report\nstatus passed\n",
        "timing",
    )?;
    Ok(())
}

fn observer_report(
    files: &[MaturityArchiveFile<'_>; 4],
    pins: [&str; 3],
) -> Result<[String; 2], String> {
    observer_file_binding(files, pins)?;
    let report = std::str::from_utf8(files[3].bytes).map_err(|_| "observer-report-encoding")?;
    let lines: Vec<_> = report.lines().collect();
    observer_check(lines.len() == 22 && report.ends_with('\n'), "report-roster")?;
    for (index, expected) in [
        (0, "run-report-schema maturity-observer-run-report 1"),
        (3, "suite-clean yes"),
        (4, "protocol-revision 8"),
        (5, "capture-profile observer"),
        (15, "producer-exit 0"),
        (16, "acquisition-exit 0"),
        (17, "observer-exit 0"),
        (18, "experiment-count 7"),
        (21, "eligible yes"),
    ] {
        observer_check(lines[index] == expected, "report-field")?;
    }
    let mut phase_sum = 0_u64;
    for (index, prefix) in [
        (6, "phase-wall-ms preflight "),
        (7, "phase-wall-ms build "),
        (8, "phase-wall-ms producer "),
        (9, "phase-wall-ms acquisition "),
        (10, "phase-wall-ms observer "),
        (11, "phase-wall-ms census "),
        (12, "phase-wall-ms manifest "),
        (13, "phase-wall-ms coordinator "),
    ] {
        let duration = lines[index]
            .strip_prefix(prefix)
            .ok_or("observer-phase-order")?;
        phase_sum = phase_sum
            .checked_add(
                duration
                    .parse::<u64>()
                    .map_err(|_| "observer-phase-duration")?,
            )
            .ok_or("observer-phase-overflow")?;
    }
    let elapsed = lines[14]
        .strip_prefix("elapsed-wall-ms ")
        .ok_or("observer-phase-order")?
        .parse::<u64>()
        .map_err(|_| "observer-phase-duration")?;
    observer_check(phase_sum == elapsed, "phase-sum")?;
    observer_check(
        lines[19] == format!("ceremony report {}", pins[2])
            && lines[20] == format!("manifest-sha256 {}", pins[0]),
        "report-content-binding",
    )?;
    let commit = lines[1]
        .strip_prefix("suite-commit ")
        .ok_or("observer-suite-commit")?;
    let tree = lines[2]
        .strip_prefix("suite-tree ")
        .ok_or("observer-suite-tree")?;
    for revision in [commit, tree] {
        observer_check(
            crate::maturity_observer::decode_hex(revision)?.len() == 20,
            "source-identity",
        )?;
    }
    Ok([commit.to_owned(), tree.to_owned()])
}

fn observer_route(producer: &str, observation: &serde_json::Value) -> Result<(), String> {
    let mut direct = false;
    for line in producer
        .lines()
        .filter_map(|line| line.strip_prefix("response-detail "))
    {
        let (length, encoded) = line.split_once(' ').ok_or("observer-producer-detail")?;
        if length == "0" {
            continue;
        }
        let bytes = crate::maturity_observer::decode_hex(encoded)?;
        observer_check(
            length.parse::<usize>().ok() == Some(bytes.len()),
            "producer-detail-length",
        )?;
        let detail: serde_json::Value =
            serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
        if detail["route"] == "direct-block" {
            observer_check(
                detail["relay_allowed"] == false
                    && detail["detail"]["included_block"] == observation["reading"]["block_hash"],
                "direct-inclusion",
            )?;
            direct = true;
        }
    }
    observer_check(direct, "direct-route-present")?;
    Ok(())
}

fn observer_isolation(isolation: &serde_json::Value) -> Result<(), String> {
    let probe = &isolation["probe"];
    observer_check(
        probe["diagnostic"].is_string()
            && (probe["command"].is_string() || probe["command"].is_null())
            && (probe["exit"].is_i64() || probe["exit"].is_null()),
        "namespace-probe-shape",
    )?;
    observer_check(
        isolation["observer_arguments"] == serde_json::json!([])
            && isolation["observer_environment"] == serde_json::json!({})
            && isolation["producer_wallet_survived"] == false,
        "construction-boundary",
    )?;
    observer_check(
        isolation["child_exits"] == serde_json::json!({"producer":0,"acquisition":0,"observer":0}),
        "child-exits",
    )?;
    observer_check(
        match isolation["namespace"].as_str() {
            Some("available") => {
                isolation["layer"] == "construction+namespace"
                    && probe["available"] == true
                    && probe["exit"] == 0
                    && isolation["excluded_paths"] == "unopenable"
                    && isolation["exclusion_check_exit"] == 0
            }
            Some("unavailable") => {
                isolation["layer"] == "construction"
                    && isolation["excluded_paths"] == "unnamed"
                    && probe["available"] == false
                    && probe["diagnostic"]
                        .as_str()
                        .is_some_and(|text| !text.trim().is_empty())
            }
            _ => false,
        },
        "isolation-evidence",
    )?;
    Ok(())
}

fn admit_observer_maturity_corpus(
    files: &[MaturityArchiveFile<'_>; 4],
    pins: [&str; 3],
) -> Result<ValidatedObserverMaturityCorpus, String> {
    use crate::maturity_observer::{
        ObserverManifest, ObserverNodeReader, observe_maturity, render_maturity_observation,
    };
    let suite = observer_report(files, pins)?;
    let record: serde_json::Value =
        serde_json::from_slice(files[0].bytes).map_err(|e| e.to_string())?;
    observer_check(
        record.as_object().is_some_and(|o| o.len() == 7) && record["schema"] == 1,
        "capture-shape",
    )?;
    observer_check(
        record["suite_commit"] == suite[0] && record["suite_tree"] == suite[1],
        "suite-binding",
    )?;
    observer_check(
        record["route"] == "relay refusal then direct block",
        "route",
    )?;
    let observation = &record["observation"];
    let variable =
        maturity_variable_run_of_record().map_err(|e| format!("variable-admission:{e:?}"))?;
    observer_check(
        observation["manifest"]["chain_id"].as_str() == Some(variable.report().network_id()),
        "declared-chain-id-disagrees",
    )?;
    let manifest = ObserverManifest::from_value(observation["manifest"].clone())?;
    let reading = ObserverNodeReader::from_value(observation["reading"].clone())?;
    let validated = observe_maturity(&manifest, &reading)?;
    let reconstructed: serde_json::Value =
        serde_json::from_str(&render_maturity_observation(&validated))
            .map_err(|e| e.to_string())?;
    observer_check(
        &reconstructed == observation,
        "report-and-experiment-replay",
    )?;
    let producer = observer_text(&record, "producer_capture")?;
    let submitted = observer_subject(producer)?;
    let acquired = crate::maturity_observer::decode_hex(observer_text(
        &observation["reading"],
        "raw_transaction",
    )?)?;
    observer_check(
        submitted == acquired && acquired.len() == 1701,
        "submitted-acquired-bytes",
    )?;
    let MaturityAcceptanceObligation::Established { readback, .. } =
        variable.evidence().acceptance_obligation()
    else {
        return Err("variable-acceptance-absent".to_owned());
    };
    observer_check(readback.bytes() == acquired, "exact-variable-subject")?;
    observer_route(producer, observation)?;
    observer_isolation(&record["isolation"])?;
    Ok(ValidatedObserverMaturityCorpus {
        observation: reconstructed,
    })
}

/// Admit the independently acquired subject and its two reports once.
/// # Errors
/// Refuses changed archive bytes, process evidence, public inputs or reconstructed reports.
pub fn maturity_observer_run_of_record() -> Result<&'static ValidatedObserverMaturityCorpus, String>
{
    static CORPUS: OnceLock<Result<ValidatedObserverMaturityCorpus, String>> = OnceLock::new();
    match CORPUS.get_or_init(|| admit_observer_maturity_corpus(&OBSERVER_FILES, OBSERVER_PINS)) {
        Ok(corpus) => Ok(corpus),
        Err(refusal) => Err(refusal.clone()),
    }
}

/// Replay-derived sponsored observations; no Safety standing is changed.
pub struct ValidatedSponsoredMaturityCorpus {
    /// Checked four-file report identity.
    pub report: MaturityReportFacts,
    /// Exact request replay and its derived census.
    pub evidence: crate::maturity_native::sponsored::Planner,
    /// Floors with their distinct provenance markers.
    pub fee_floors: MaturityDisclosedFeeFloors,
}

/// The unique S2 control chosen for the combined projection, with no fallback.
pub(crate) const COMBINED_PROJECTION_CASE: &str =
    "sponsored-with-change-where-supported:control:52";

/// The selected admitted submission with its replayed bundle and public funding context.
#[derive(Clone, Debug)]
pub(crate) struct SponsoredProjectionSource {
    pub(crate) bundle: CandidateLinkedMaturityBundle,
    pub(crate) context: crate::maturity_recovery::MaturityPublicPositionContext,
    pub(crate) readback: crate::maturity_native::MaturityAcceptedReadback,
    pub(crate) response: NativeOperationResponse,
}

/// Select one admitted control and join every spent output by outpoint rather than archive order.
pub(crate) fn sponsored_projection_source(
    corpus: &ValidatedSponsoredMaturityCorpus,
) -> Result<SponsoredProjectionSource, crate::maturity_continuity::MaturityPositionalRefusal> {
    select_sponsored_projection_source(corpus, COMBINED_PROJECTION_CASE)
}

/// Select a different admitted case only for cross-transaction refusal tests.
#[cfg(test)]
pub(crate) fn sponsored_projection_source_for_case(
    corpus: &ValidatedSponsoredMaturityCorpus,
    case: &str,
) -> Result<SponsoredProjectionSource, crate::maturity_continuity::MaturityPositionalRefusal> {
    select_sponsored_projection_source(corpus, case)
}

/// Join the explicitly selected archive case with its unique public funding facts.
fn select_sponsored_projection_source(
    corpus: &ValidatedSponsoredMaturityCorpus,
    case: &str,
) -> Result<SponsoredProjectionSource, crate::maturity_continuity::MaturityPositionalRefusal> {
    use crate::live_owner_observation::{asset_of, decode_hex, outpoint_of};
    use crate::maturity_continuity::{
        MaturityFundedPredecessor, MaturityPositionalRefusal as Failure,
    };
    use crate::maturity_native::sponsored;
    if corpus.report.run_address() != MATURITY_SPONSORED_RUN_ADDRESS {
        return Err(Failure::PublicContext);
    }
    let credited = corpus.evidence.credited();
    let mut matches = credited.iter().filter(|(step, _)| step.case().step == case);
    let (step, response) = matches.next().ok_or(Failure::PublicContext)?;
    if matches.next().is_some()
        || response.observed_layer != ObservedOutcomeLayer::Accepted
        || !matches!(
            sponsored::detail(response),
            Ok((_, Some(MaturityAcceptanceRoute::RelayAndTemplate)))
        )
    {
        return Err(Failure::PublicContext);
    }
    let OperationSubject::Submission(subject) = step.subject() else {
        return Err(Failure::PublicContext);
    };
    let (bundle, branch, reserve_asset) = corpus
        .evidence
        .projection_context(step)
        .map_err(|_| Failure::Replay)?;
    let readback = sponsored::accepted_readback(&subject.transaction_bytes, response)
        .map_err(|_| Failure::Replay)?;
    let transaction = transaction::bytes::TargetTransaction::decode(readback.bytes())
        .map_err(|error| Failure::Decode(Box::new(error)))?;
    let spent = transaction
        .inputs()
        .iter()
        .map(|input| {
            let mut matches = corpus
                .evidence
                .exchanges()
                .iter()
                .flat_map(|(_, response)| &response.funded_outputs)
                .filter(|coin| outpoint_of(&coin.outpoint) == Some(input.outpoint()));
            let coin = matches.next().ok_or(Failure::PublicContext)?;
            if matches.next().is_some() {
                return Err(Failure::PublicContext);
            }
            Ok(MaturityFundedPredecessor {
                outpoint: input.outpoint(),
                asset: asset_of(&coin.asset).ok_or(Failure::PublicContext)?,
                amount: coin.amount_satoshis,
                program: decode_hex(&coin.script).ok_or(Failure::PublicContext)?,
            })
        })
        .collect::<Result<Vec<_>, Failure>>()?;
    let context = crate::maturity_recovery::MaturityPublicPositionContext {
        spent,
        reserve_asset,
        identity: bundle.deployment().identity().clone(),
        branch,
    };
    Ok(SponsoredProjectionSource {
        bundle,
        context,
        readback,
        response: response.clone(),
    })
}

/// Admits a sponsored capture through its own file pins and exact request replay.
///
/// # Errors
/// Returns the first byte, grammar, policy, census, route or readback disagreement.
///
/// # Panics
/// Panics only if the fixed architecture omits its singleton, which its census cannot arrange.
pub fn admit_sponsored_maturity_corpus(
    files: &[MaturityArchiveFile<'_>; 4],
    pins: &MaturityCorpusPins<'_>,
) -> ImportResult<ValidatedSponsoredMaturityCorpus> {
    admit_sponsored_with_replay(files, pins, crate::maturity_native::sponsored::replay)
}

#[expect(
    clippy::too_many_lines,
    reason = "The admission follows the four-file binding order before replaying the schedule."
)]
fn admit_sponsored_with_replay(
    files: &[MaturityArchiveFile<'_>; 4],
    pins: &MaturityCorpusPins<'_>,
    replay: impl FnOnce(
        CandidateDeploymentIdentity,
        BranchContext,
        &[(OperationStep, NativeOperationResponse)],
    ) -> Result<
        crate::maturity_native::sponsored::Planner,
        crate::maturity_native::MaturityNativePlanRefusal,
    >,
) -> ImportResult<ValidatedSponsoredMaturityCorpus> {
    use crate::maturity_native::sponsored::Policy;
    validate_census(files, pins)?;
    validate_manifest(files, pins.manifest_sha256())?;
    let original_report =
        std::str::from_utf8(files[3].bytes).map_err(|_| Refusal::RunReportGrammar)?;
    Cursor::new("sponsored report", files[3].bytes)?;
    let mut report_lines = original_report.lines().peekable();
    let mut normalized = String::new();
    let mut extra = String::new();
    for line in report_lines.by_ref() {
        if line == "capture-profile sponsored" {
            extra.push_str(line);
            extra.push('\n');
            break;
        }
        normalized.push_str(line);
        normalized.push('\n');
    }
    for _ in 0..3 {
        extra.push_str(report_lines.next().ok_or(Refusal::RunReportGrammar)?);
        extra.push('\n');
    }
    if report_lines
        .peek()
        .is_some_and(|line| line.starts_with("sponsored-waiting "))
    {
        extra.push_str(report_lines.next().ok_or(Refusal::RunReportGrammar)?);
        extra.push('\n');
    }
    for line in report_lines {
        normalized.push_str(line);
        normalized.push('\n');
    }
    let mut fields = Cursor::new("sponsored report", extra.as_bytes())?;
    fields.exact("capture-profile sponsored")?;
    let policy_bytes = fields.bytes("sponsored-policy")?;
    let census_text = fields.value("sponsored-census")?;
    let probe_bytes = fields.bytes("sponsored-probes")?;
    let waiting_bytes = fields.waiting()?;
    let census = census_text
        .split(' ')
        .map(number)
        .collect::<Option<Vec<_>>>()
        .ok_or(Refusal::RunReportGrammar)?;
    fields.done()?;
    let policy = Policy::parse(
        serde_json::from_slice(&policy_bytes).map_err(|_| Refusal::RunReportGrammar)?,
    )
    .map_err(Refusal::Derivation)?;
    binding(
        "sponsored-policy-encoding",
        serde_json::to_vec(&policy.0).map_err(|_| Refusal::RunReportGrammar)? == policy_bytes,
    )?;
    let (mut report, _) = parse_report_body(normalized.as_bytes(), pins, None)?;
    report.run_address = hex_bytes(&tagged::sha256(files[3].bytes));
    check_report_address(&report, pins)?;
    validate_common_bindings(files, pins, &report)?;
    let floors = pins.fee_floors();
    binding(
        "sponsored-fee-floors",
        floors.min_relay_tx_fee() == policy.0["relay"].as_str().unwrap_or_default()
            && floors.block_min_tx_fee() == policy.0["block"].as_str().unwrap_or_default()
            && floors.provenance() == MaturityPremiseProvenance::ReportedByNode
            && floors.block_provenance() == MaturityPremiseProvenance::PinnedSourceNoOverride,
    )?;
    let mut cursor = Cursor::new("sponsored capture", files[0].bytes)?;
    parse_header(&mut cursor, &report, pins)?;
    parse_environment(&mut cursor, &report, pins)?;
    cursor.exact("digest-count 0")?;
    let count = number(cursor.value("operation-count")?).ok_or_else(|| cursor.refusal())?;
    let operations = (0..count)
        .map(|index| parse_operation_block(&mut cursor, index, "", SubjectKind::Sponsored))
        .collect::<ImportResult<Vec<_>>>()?;
    let payload = parse_tail(&mut cursor, files[0].bytes)?;
    let mut cursor = Cursor::new("sponsored payload", &payload)?;
    cursor.exact("maturity-sponsored-evidence-schema 1")?;
    let identity = CandidateDeploymentIdentity::new(
        cursor
            .bytes("deployment-id")?
            .try_into()
            .map_err(|_| cursor.refusal())?,
        cursor
            .bytes("genesis-id")?
            .try_into()
            .map_err(|_| cursor.refusal())?,
    )
    .map_err(|_| cursor.refusal())?;
    binding(
        "sponsored-deployment",
        hex_bytes(identity.network_id()) == report.network_id
            && hex_bytes(identity.genesis_id()) == report.genesis_id,
    )?;
    let identifier = cursor
        .bytes("branch-identifier")?
        .try_into()
        .map_err(|_| cursor.refusal())?;
    let checkpoint = cursor
        .value("branch-checkpoint")?
        .parse::<u64>()
        .map_err(|_| cursor.refusal())?;
    let branch = BranchContext::new(identifier, checkpoint).map_err(|_| cursor.refusal())?;
    binding(
        "sponsored-policy",
        cursor.bytes("sponsored-policy")? == policy_bytes,
    )?;
    binding(
        "sponsored-census",
        cursor.value("sponsored-census")? == census_text,
    )?;
    binding(
        "sponsored-probes",
        cursor.bytes("sponsored-probes")? == probe_bytes,
    )?;
    binding("sponsored-waiting", cursor.waiting()? == waiting_bytes)?;
    let mut exchanges = Vec::new();
    for operation in operations {
        let bytes = cursor.bytes("exchange")?;
        let (case, subject, response): (
            target_elements_conformance::protocol::OperationCaseId,
            OperationSubject,
            NativeOperationResponse,
        ) = serde_json::from_slice(&bytes).map_err(|_| cursor.refusal())?;
        binding(
            "sponsored-exchange",
            operation.step.subject() == &subject
                && response.case == case
                && operation.layer == response.observed_layer
                && operation.detail == response.observed_detail.as_deref().unwrap_or_default()
                && operation.accepted_identity.map(|id| id.to_target_display())
                    == response.accepted_txid,
        )?;
        exchanges.push((OperationStep::new(&case.step, subject), response));
    }
    cursor.done()?;
    let evidence = replay(identity, branch, &exchanges).map_err(Refusal::Derivation)?;
    binding(
        "sponsored-derived-waiting",
        serde_json::to_vec(&evidence.waiting_json()).map_err(|_| Refusal::RunReportGrammar)?
            == waiting_bytes,
    )?;
    binding(
        "sponsored-probe-observations",
        serde_json::to_vec(&evidence.probes()).map_err(|_| Refusal::RunReportGrammar)?
            == probe_bytes,
    )?;
    binding(
        "sponsored-derived-census",
        evidence.census().as_slice() == census && census.first() == Some(&count),
    )?;
    binding(
        "sponsored-derived-policy",
        evidence.policy() == Some(&policy),
    )?;
    Ok(ValidatedSponsoredMaturityCorpus {
        report,
        evidence,
        fee_floors: floors,
    })
}

/// Admits the sponsored archive once through its independent pins and exact replay.
///
/// # Errors
/// Preserves the first binding or exact-replay refusal.
/// # Panics
/// Panics only if the fixed architecture omits its singleton, which archive bytes cannot arrange.
pub fn maturity_sponsored_run_of_record() -> ImportResult<&'static ValidatedSponsoredMaturityCorpus>
{
    static CORPUS: OnceLock<ImportResult<ValidatedSponsoredMaturityCorpus>> = OnceLock::new();
    match CORPUS.get_or_init(|| admit_sponsored_maturity_corpus(&SPONSORED_FILES, &SPONSORED_PINS))
    {
        Ok(corpus) => Ok(corpus),
        Err(refusal) => Err(refusal.clone()),
    }
}

/// Admits the immutable historical maturity archive through its own pins.
///
/// The payload is checked against the derived evidence; it supplies no standing by authority. The admitted relay refusal answers the declared boundary and leaves acceptance outstanding through both routes.
///
/// # Errors
/// Returns the refusal naming the first failed admission layer, preserving planner refusals whole.
///
/// # Panics
/// Panics only if the fixed architecture omits its singleton or the public linker loses a retained constructor, which neither published input can arrange.
pub fn maturity_run_of_record()
-> Result<&'static ValidatedMaturityCorpus, MaturityCorpusImportRefusal> {
    static CORPUS: OnceLock<ImportResult<ValidatedMaturityCorpus>> = OnceLock::new();
    match CORPUS.get_or_init(|| admit_maturity_corpus(&FILES, &HISTORICAL_PINS)) {
        Ok(corpus) => Ok(corpus),
        Err(refusal) => Err(refusal.clone()),
    }
}
/// Replays the once-admitted immutable archive under an explicit witness selection.
///
/// Every reconstructed request is compared with its recorded counterpart before
/// the responses establish evidence. The archive's byte bindings and admission
/// remain those of [`maturity_run_of_record`].
///
/// # Errors
/// Preserves archive admission refusals and the planner's schedule or exact-replay
/// refusals through [`MaturityCorpusImportRefusal::Derivation`].
///
/// # Panics
/// Panics only if the fixed architecture omits its singleton or the public linker
/// loses a retained constructor, which neither published input can arrange.
#[must_use = "archive replay can refuse the selected schedule or recorded exchanges"]
pub fn replay_maturity_run_of_record(
    selection: MaturityWitnessSelection,
) -> Result<MaturityNativeEvidence, MaturityCorpusImportRefusal> {
    let corpus = maturity_run_of_record()?;
    MaturityNativeEvidence::from_transcript(
        corpus.evidence().identity().clone(),
        corpus.evidence().branch(),
        selection,
        corpus.exchanges(),
    )
    .map_err(Refusal::Derivation)
}

/// Admits the immutable accepted variable-metadata archive through its own pins.
///
/// The executor's development node ran with `-minrelaytxfee=0` and `-blockmintxfee=0`. Admission establishes acceptance in that environment; it does not establish relayability under a positive fee floor, authenticate root freshness, or promote an artifact.
///
/// # Errors
/// Returns the first failed byte, grammar or planner replay binding.
///
/// # Panics
/// Panics only if the fixed architecture omits its singleton or the public linker loses a retained constructor, which neither published input can arrange.
#[must_use = "accepted corpus admission can refuse a file or replay binding"]
pub fn maturity_variable_run_of_record()
-> Result<&'static ValidatedMaturityCorpus, MaturityCorpusImportRefusal> {
    static CORPUS: OnceLock<ImportResult<ValidatedMaturityCorpus>> = OnceLock::new();
    match CORPUS.get_or_init(|| admit_maturity_corpus(&VARIABLE_FILES, &VARIABLE_PINS)) {
        Ok(corpus) => Ok(corpus),
        Err(refusal) => Err(refusal.clone()),
    }
}

/// Replays the once-admitted accepted archive under an explicit witness selection.
///
/// # Errors
/// Preserves archive admission refusals and planner schedule or exact-replay refusals.
///
/// # Panics
/// Panics only if the fixed architecture omits its singleton or the public linker loses a retained constructor, which neither published input can arrange.
#[must_use = "accepted archive replay can refuse the selection or exchanges"]
pub fn replay_maturity_variable_run_of_record(
    selection: MaturityWitnessSelection,
) -> Result<MaturityNativeEvidence, MaturityCorpusImportRefusal> {
    let corpus = maturity_variable_run_of_record()?;
    MaturityNativeEvidence::from_transcript(
        corpus.evidence().identity().clone(),
        corpus.evidence().branch(),
        selection,
        corpus.exchanges(),
    )
    .map_err(Refusal::Derivation)
}

impl crate::maturity_history::MaturityRootCheckpoint {
    /// Bind the selected sponsored readback through the same checkpoint checks.
    pub(crate) fn bind_sponsored(
        edge: &StateRootEdge,
        handoff: &PublicAnnouncementHandoff,
        corpus: &ValidatedSponsoredMaturityCorpus,
        source: &SponsoredProjectionSource,
        linked_candidate: MaturityDeclaredPremise<CandidateLinkedMaturityBundle>,
        candidate_abi: MaturityDeclaredPremise<CandidateMaturityAnnouncementAbi>,
    ) -> Result<Self, MaturityRootCheckpointRefusal> {
        let facts = crate::maturity_history::MaturityRootCheckpointFacts {
            network_identity: corpus.report.network_id().to_owned(),
            genesis_identity: corpus.report.genesis_id().to_owned(),
            predecessor: edge.predecessor(),
            successor: edge.successor(),
            target_contract: corpus.report.target_contract().to_owned(),
            branch: source.context.branch,
        };
        Self::from_readback(
            facts,
            &source.readback,
            linked_candidate,
            candidate_abi,
            edge,
            handoff,
        )
    }

    /// Bind the admitted readback and its ten checkpoint facts to a public handoff and edge.
    ///
    /// # Errors
    /// Refuses an outstanding acceptance or the first digest, identity, outpoint or premise
    /// disagreement.
    pub fn bind(
        edge: &StateRootEdge,
        handoff: &PublicAnnouncementHandoff,
        corpus: &ValidatedMaturityCorpus,
        linked_candidate: MaturityDeclaredPremise<CandidateLinkedMaturityBundle>,
        candidate_abi: MaturityDeclaredPremise<CandidateMaturityAnnouncementAbi>,
    ) -> Result<Self, MaturityRootCheckpointRefusal> {
        let readback = match corpus.evidence().acceptance_obligation() {
            MaturityAcceptanceObligation::Outstanding { routes } => {
                return Err(MaturityRootCheckpointRefusal::AcceptanceIsOutstanding {
                    routes: *routes,
                });
            }
            MaturityAcceptanceObligation::Established { readback, .. } => readback,
        };
        let facts = crate::maturity_history::MaturityRootCheckpointFacts {
            network_identity: corpus.report().network_id().to_owned(),
            genesis_identity: corpus.report().genesis_id().to_owned(),
            predecessor: edge.predecessor(),
            successor: edge.successor(),
            target_contract: corpus.report().target_contract().to_owned(),
            branch: corpus.evidence().branch(),
        };
        Self::from_readback(
            facts,
            readback,
            linked_candidate,
            candidate_abi,
            edge,
            handoff,
        )
    }
}

/// Build the admitted archive's public handoff for report derivation.
///
/// This builder reads an admitted archive; the constrained producer receives
/// only the resulting handoff.
///
/// # Errors
/// Returns the first unavailable accepted reading with its stage.
#[must_use = "an admitted handoff can be unavailable"]
pub fn accepted_public_handoff(
    corpus: &ValidatedMaturityCorpus,
) -> Result<PublicAnnouncementHandoff, MaturityPublicRecoveryReportRefusal> {
    use crate::maturity_closure::{MaturityDeployment, OracleStateCurve, closure_target};
    use crate::maturity_continuity::MaturityByteSource;
    use crate::maturity_recovery::PublicAnnouncementLocator;
    use realization::{AnnouncementLeadBounds, Cycle};
    use tapscript::{
        STATE_NUMS_KEY, StateInternalKeyPolicy, StateLeafRole, StateStaticLeaf, StateStaticNode,
        StateStaticSubtree, TapscriptProgram,
    };
    use target_elements::TargetContractVersion;
    use transaction::bytes::TargetTransaction;

    use MaturityAcceptedHandoffStage as Stage;
    let readback = match corpus.evidence().acceptance_obligation() {
        MaturityAcceptanceObligation::Established { readback, .. } => readback,
        MaturityAcceptanceObligation::Outstanding { routes } => {
            return Err(
                MaturityPublicRecoveryReportRefusal::AcceptedHandoffUnavailable {
                    stage: Box::new(Stage::AcceptanceOutstanding { routes: *routes }),
                },
            );
        }
    };
    let transaction = TargetTransaction::decode(readback.bytes()).map_err(|refusal| {
        MaturityPublicRecoveryReportRefusal::AcceptedHandoffUnavailable {
            stage: Box::new(Stage::TransactionUndecodable(refusal)),
        }
    })?;
    let target = closure_target().map_err(|refusal| {
        MaturityPublicRecoveryReportRefusal::AcceptedHandoffUnavailable {
            stage: Box::new(Stage::ReviewedTargetUnavailable(refusal)),
        }
    })?;
    let stack = transaction
        .witnesses()
        .first()
        .map_or(&[][..], |input| input.stack());
    let leaf = stack.get(7).ok_or_else(|| {
        MaturityPublicRecoveryReportRefusal::AcceptedHandoffUnavailable {
            stage: Box::new(Stage::LeafAbsent { items: stack.len() }),
        }
    })?;
    let program = TapscriptProgram::decode(&target, leaf).map_err(|refusal| {
        MaturityPublicRecoveryReportRefusal::AcceptedHandoffUnavailable {
            stage: Box::new(Stage::LeafUndecodable(refusal)),
        }
    })?;
    let subtree = StateStaticSubtree::new(
        &target,
        Some(StateStaticNode::Leaf {
            identity: 0,
            leaf: StateStaticLeaf {
                role: StateLeafRole::Announcement,
                program,
                version: target.definition().leaf_version().get(),
            },
        }),
    )
    .map_err(
        |refusal| MaturityPublicRecoveryReportRefusal::AcceptedHandoffUnavailable {
            stage: Box::new(Stage::SubtreeRefused(refusal)),
        },
    )?;
    let (minimum, maximum) = MaturityDeployment::PublishedSignerHeld
        .parameters()
        .map_err(
            |refusal| MaturityPublicRecoveryReportRefusal::AcceptedHandoffUnavailable {
                stage: Box::new(Stage::DeploymentUnavailable(refusal)),
            },
        )?
        .lead();
    let bounds = AnnouncementLeadBounds::new(Cycle::new(minimum), Cycle::new(maximum)).map_err(
        |refusal| MaturityPublicRecoveryReportRefusal::AcceptedHandoffUnavailable {
            stage: Box::new(Stage::LeadBoundsRefused(refusal)),
        },
    )?;
    let internal_key =
        StateInternalKeyPolicy::new(STATE_NUMS_KEY, &OracleStateCurve).map_err(|refusal| {
            MaturityPublicRecoveryReportRefusal::AcceptedHandoffUnavailable {
                stage: Box::new(Stage::InternalKeyRefused(refusal)),
            }
        })?;
    Ok(PublicAnnouncementHandoff::new(
        PublicAnnouncementLocator::new(
            MaturityByteSource::ArchivedSubmission {
                run_address: corpus.report().run_address().to_owned(),
            },
            readback.identity(),
        ),
        readback.bytes().to_vec(),
        0,
        corpus.schedule(),
        bounds,
        subtree,
        internal_key,
        TargetContractVersion::V2,
    ))
}

#[cfg(test)]
mod tests {

    fn rebound_diagnostic_payload(
        change: impl FnOnce(&str) -> String,
    ) -> ImportResult<ValidatedDiagnosticMaturityCorpus> {
        let mut bytes = DIAGNOSTIC_FILES.map(|file| file.bytes.to_vec());
        let capture = std::str::from_utf8(&bytes[0]).expect("capture");
        let line = capture
            .lines()
            .find(|line| line.starts_with("legacy-rendering "))
            .expect("payload");
        let payload =
            String::from_utf8(decode_hex(line.split(' ').nth(2).expect("hex")).expect("bytes"))
                .expect("text");
        let changed = change(&payload);
        let changed_line = format!(
            "legacy-rendering {} {}",
            changed.len(),
            hex_bytes(changed.as_bytes())
        );
        bytes[0] = capture.replace(line, &changed_line).into_bytes();
        let capture = std::str::from_utf8(&bytes[0]).expect("capture");
        let position = capture
            .find("capture-content-sha256 ")
            .expect("content hash");
        let hash = hex_bytes(&tagged::sha256(&bytes[0][..position]));
        let start = position + "capture-content-sha256 ".len();
        bytes[0][start..start + 64].copy_from_slice(hash.as_bytes());
        let ceremony = hex_bytes(&tagged::sha256(&bytes[0]));
        let mut manifest = String::new();
        for (file, value) in DIAGNOSTIC_FILES.iter().zip(&bytes).take(2) {
            writeln!(
                manifest,
                "{}  {}",
                hex_bytes(&tagged::sha256(value)),
                file.name
            )
            .expect("manifest");
        }
        bytes[2] = manifest.into_bytes();
        let manifest_hash = hex_bytes(&tagged::sha256(&bytes[2]));
        let mut report = String::new();
        for line in std::str::from_utf8(&bytes[3]).expect("report").lines() {
            if line.starts_with("ceremony report ") {
                writeln!(report, "ceremony report {ceremony}").expect("report");
            } else if line.starts_with("manifest-sha256 ") {
                writeln!(report, "manifest-sha256 {manifest_hash}").expect("report");
            } else {
                writeln!(report, "{line}").expect("report");
            }
        }
        bytes[3] = report.into_bytes();
        let address = hex_bytes(&tagged::sha256(&bytes[3]));
        let pins = MaturityCorpusPins {
            sizes: std::array::from_fn(|index| bytes[index].len()),
            ceremony_sha256: &ceremony,
            manifest_sha256: &manifest_hash,
            run_address: &address,
            ..DIAGNOSTIC_PINS
        };
        let files = std::array::from_fn(|index| {
            MaturityArchiveFile::new(DIAGNOSTIC_FILES[index].name, &bytes[index])
        });
        admit_diagnostic_maturity_corpus(&files, &pins)
    }

    #[test]
    fn diagnostic_payload_refuses_missing_duplicate_surplus_and_changed_records_after_rebinding() {
        assert!(rebound_diagnostic_payload(str::to_owned).is_ok());
        for prefix in ["diagnostic-policy ", "campaign-row ", "exchange "] {
            for duplicate in [false, true] {
                assert!(
                    rebound_diagnostic_payload(|payload| {
                        let line = payload
                            .lines()
                            .find(|line| line.starts_with(prefix))
                            .expect("record");
                        payload.replacen(
                            &format!("{line}\n"),
                            &if duplicate {
                                format!("{line}\n{line}\n")
                            } else {
                                String::new()
                            },
                            1,
                        )
                    })
                    .is_err(),
                    "{prefix} duplicate={duplicate}"
                );
            }
        }
        for suffix in [
            "refusal 4 6e756c6c\n",
            "manifest-refusal 4 6e756c6c\n",
            "surplus yes\n",
        ] {
            assert!(rebound_diagnostic_payload(|payload| format!("{payload}{suffix}")).is_err());
        }
        assert!(
            rebound_diagnostic_payload(|payload| payload.replacen(
                "branch-checkpoint 7",
                "branch-checkpoint 8",
                1
            ))
            .is_err()
        );
    }

    #[test]
    fn diagnostic_report_refuses_each_missing_or_duplicate_line() {
        let original = std::str::from_utf8(DIAGNOSTIC_FILES[3].bytes).expect("report");
        for line in original.lines() {
            for replacement in [String::new(), format!("{line}\n{line}\n")] {
                let changed = original.replacen(&format!("{line}\n"), &replacement, 1);
                assert!(
                    parse_diagnostic_report(changed.as_bytes(), &DIAGNOSTIC_PINS).is_err(),
                    "{}",
                    line.split(' ').next().expect("field")
                );
            }
        }
    }

    #[test]
    fn diagnostic_admission_binds_the_whole_archive_and_recordings() {
        use crate::maturity_native::{RecordedBlockScriptError, recorded_block_script_error};
        let corpus = admit_diagnostic_maturity_corpus(&DIAGNOSTIC_FILES, &DIAGNOSTIC_PINS)
            .expect("diagnostic archive");
        assert_eq!(corpus.report.run_address(), MATURITY_DIAGNOSTIC_RUN_ADDRESS);
        assert_eq!(corpus.evidence.complete(), Ok(()));
        assert_eq!(corpus.evidence.stopped(), [] as [&str; 0]);
        let mut counts = [0; 6];
        let mut recorded = 0;
        for (step, response) in corpus.evidence.exchanges() {
            counts[0] += 1;
            match step.subject() {
                OperationSubject::Funding(_) => counts[1] += 1,
                OperationSubject::Submission(_) => {
                    counts[2] += 1;
                    match response.observed_layer {
                        ObservedOutcomeLayer::Accepted => counts[3] += 1,
                        ObservedOutcomeLayer::ScriptPathRejection => counts[4] += 1,
                        ObservedOutcomeLayer::ConsensusRejectionBeforeScript => counts[5] += 1,
                        other => panic!("unexpected layer {other:?}"),
                    }
                }
                other => panic!("unexpected operation {other:?}"),
            }
            let reading = recorded_block_script_error(&response).expect("recording");
            if response.observed_layer == ObservedOutcomeLayer::Accepted {
                assert_eq!(reading, RecordedBlockScriptError::NotRecorded);
            } else {
                assert!(matches!(reading, RecordedBlockScriptError::Recorded(_)));
                recorded += 1;
            }
        }
        assert_eq!(counts, [48, 17, 31, 7, 23, 1]);
        assert_eq!(recorded, 24);
        let (_, fields) =
            parse_diagnostic_report(DIAGNOSTIC_FILES[3].bytes, &DIAGNOSTIC_PINS).expect("report");
        let rows: Vec<_> = fields
            .rows
            .iter()
            .map(|row| retained_json(row).expect("row"))
            .collect();
        assert_eq!(rows.len(), 23);
        assert_eq!(
            rows.iter()
                .map(|row| row["claims"].as_array().expect("claims").len())
                .sum::<usize>(),
            25
        );
        assert!(
            rows.iter()
                .all(|row| !row["control"].is_null() && !row["observation"].is_null())
        );
        assert_eq!(
            corpus.fee_floors.provenance(),
            MaturityPremiseProvenance::ReportedByNode
        );
        assert_eq!(
            corpus.fee_floors.block_provenance(),
            MaturityPremiseProvenance::PinnedSourceNoOverride
        );
        for family in corpus.evidence.families_json() {
            assert_eq!(family["control_accepted"], true);
            assert_eq!(family["control"]["readback"], "matched");
            assert_eq!(family["expectation"], "agreement");
        }
    }

    #[test]
    fn diagnostic_families_refuse_each_uncontrolled_or_wrong_boundary_subject() {
        for index in 0..7 {
            for (pointer, value) in [
                ("/stopped", serde_json::json!(true)),
                ("/expectation", serde_json::json!("contradiction")),
                ("/control_accepted", serde_json::json!(false)),
                ("/control/readback", serde_json::json!("mismatched")),
                ("/control/layer", serde_json::json!("script_path_rejection")),
                ("/subjects/0/observation", serde_json::Value::Null),
                ("/subjects/0/expectation", serde_json::json!("absent")),
                (
                    "/subjects/0/observation/layer",
                    serde_json::json!("key_path_rejection"),
                ),
            ] {
                let (_, mut fields) =
                    parse_diagnostic_report(DIAGNOSTIC_FILES[3].bytes, &DIAGNOSTIC_PINS)
                        .expect("report");
                let mut families = retained_json(&fields.families).expect("families");
                *families[index].pointer_mut(pointer).expect("coordinate") = value;
                fields.families = serde_json::to_vec(&families).expect("JSON");
                assert!(
                    validate_diagnostic_fields(&fields).is_err(),
                    "family {index} {pointer}"
                );
            }
        }
        let (_, mut fields) =
            parse_diagnostic_report(DIAGNOSTIC_FILES[3].bytes, &DIAGNOSTIC_PINS).expect("report");
        fields.stopped = br#"["window"]"#.to_vec();
        assert!(validate_diagnostic_fields(&fields).is_err());
        let mut families = retained_json(&fields.families).expect("families");
        families.as_array_mut().expect("array").pop();
        fields.families = serde_json::to_vec(&families).expect("JSON");
        assert!(validate_diagnostic_fields(&fields).is_err());
    }

    #[test]
    fn diagnostic_campaign_refuses_changed_bindings_and_claims() {
        use crate::maturity_native::native::DiagnosticCampaignManifest;
        let corpus = maturity_diagnostic_run_of_record().expect("archive");
        let (_, fields) =
            parse_diagnostic_report(DIAGNOSTIC_FILES[3].bytes, &DIAGNOSTIC_PINS).expect("report");
        let rows: Vec<_> = fields
            .rows
            .iter()
            .map(|row| retained_json(row).expect("row"))
            .collect();
        assert_eq!(
            DiagnosticCampaignManifest::validate(&corpus.evidence, &rows),
            Ok(())
        );
        for (index, _) in rows.iter().enumerate() {
            for pointer in [
                "/claims/0/name",
                "/claims/0/section",
                "/control/digest",
                "/control_id",
                "/observation/digest",
                "/observation/funded_outpoints/0/funding_step",
                "/candidate/subject_pin",
                "/expected_class",
                "/expectation_reason",
                "/expected_consumer",
                "/policy",
                "/family",
                "/declared_boundary",
                "/compatibility",
            ] {
                let mut changed = rows.clone();
                *changed[index].pointer_mut(pointer).expect("coordinate") =
                    serde_json::json!("changed");
                assert!(
                    DiagnosticCampaignManifest::validate(&corpus.evidence, &changed).is_err(),
                    "row {index} {pointer}"
                );
            }
        }
        let mut changed = rows.clone();
        changed[0]["surplus"] = serde_json::json!(true);
        assert!(DiagnosticCampaignManifest::validate(&corpus.evidence, &changed).is_err());
        assert!(DiagnosticCampaignManifest::validate(&corpus.evidence, &rows[..22]).is_err());
    }

    #[test]
    fn diagnostic_report_address_refuses_a_grammar_valid_changed_wall() {
        let mut bytes = DIAGNOSTIC_FILES[3].bytes.to_vec();
        let prefix = b"phase-wall-ms preflight ";
        let index = bytes
            .windows(prefix.len())
            .position(|part| part == prefix)
            .expect("phase wall")
            + prefix.len();
        bytes[index] = if bytes[index] == b'1' { b'2' } else { b'1' };
        assert!(parse_diagnostic_report(&bytes, &DIAGNOSTIC_PINS).is_ok());
        let mut files = DIAGNOSTIC_FILES;
        files[3] = MaturityArchiveFile::new(files[3].name, &bytes);
        assert_eq!(
            admit_diagnostic_maturity_corpus(&files, &DIAGNOSTIC_PINS).err(),
            Some(Refusal::RunReportAddress)
        );
    }

    #[test]
    fn diagnostic_node_tip_refuses_a_consistent_different_revision() {
        let tip = "1".repeat(40);
        let binary = &tip[..12];
        let version = format!("Elements Core daemon version v28.99.0-{binary}");
        let original = std::str::from_utf8(DIAGNOSTIC_FILES[3].bytes).expect("report");
        let lines: Vec<_> = original
            .lines()
            .map(|line| {
                if line.starts_with("elementsd-expected-tip ") {
                    return format!("elementsd-expected-tip {tip}");
                }
                for (field, value) in [
                    ("elementsd-binary-reported-revision", binary),
                    ("elementsd-intended-executed-tip", tip.as_str()),
                    ("node-version", version.as_str()),
                ] {
                    if line.starts_with(&format!("{field} ")) {
                        return format!("{field} {} {}", value.len(), hex_bytes(value.as_bytes()));
                    }
                }
                line.to_owned()
            })
            .collect();
        let bytes = format!("{}\n", lines.join("\n")).into_bytes();
        assert_eq!(
            parse_diagnostic_report(&bytes, &DIAGNOSTIC_PINS).err(),
            Some(Refusal::CrossFileBinding {
                field: "diagnostic-node-tip"
            })
        );
    }

    #[test]
    fn diagnostic_schedule_pin_refuses_another_schedule() {
        let pins = MaturityCorpusPins {
            schedule: tapscript::StateWitnessSchedule::WholeMetadata,
            ..DIAGNOSTIC_PINS
        };
        assert_eq!(
            admit_diagnostic_maturity_corpus(&DIAGNOSTIC_FILES, &pins).err(),
            Some(Refusal::CrossFileBinding {
                field: "diagnostic-schedule"
            })
        );
    }

    #[test]
    fn diagnostic_archive_pins_refuse_changed_files() {
        for index in 0..4 {
            let mut bytes = DIAGNOSTIC_FILES[index].bytes.to_vec();
            bytes[0] ^= 1;
            let mut files = DIAGNOSTIC_FILES;
            files[index] = MaturityArchiveFile::new(files[index].name, &bytes);
            assert!(admit_diagnostic_maturity_corpus(&files, &DIAGNOSTIC_PINS).is_err());
            let mut longer = bytes.clone();
            longer.push(b'\n');
            files[index] = MaturityArchiveFile::new(files[index].name, &longer);
            assert!(matches!(
                validate_census(&files, &DIAGNOSTIC_PINS),
                Err(Refusal::FileSize { .. })
            ));
        }
        let mut files = DIAGNOSTIC_FILES;
        files.swap(0, 1);
        assert_eq!(
            validate_census(&files, &DIAGNOSTIC_PINS),
            Err(Refusal::CorpusCensus)
        );
        for pins in [
            MaturityCorpusPins {
                manifest_sha256: &"0".repeat(64),
                ..DIAGNOSTIC_PINS
            },
            MaturityCorpusPins {
                run_address: &"0".repeat(64),
                ..DIAGNOSTIC_PINS
            },
            MaturityCorpusPins {
                ceremony_sha256: &"0".repeat(64),
                ..DIAGNOSTIC_PINS
            },
            MaturityCorpusPins {
                cargo_path: "/wrong/cargo",
                ..DIAGNOSTIC_PINS
            },
            MaturityCorpusPins {
                capture_target: MaturityCaptureTarget::new("wrong", "wrong"),
                ..DIAGNOSTIC_PINS
            },
            MaturityCorpusPins {
                funding_count: 1,
                ..DIAGNOSTIC_PINS
            },
            MaturityCorpusPins {
                framework_revision: MaturityFrameworkRevision::Unrecorded,
                ..DIAGNOSTIC_PINS
            },
            MaturityCorpusPins {
                fee_floors: MaturityDisclosedFeeFloors::node_defaults("0.00000200", "0.00000100"),
                ..DIAGNOSTIC_PINS
            },
        ] {
            assert!(admit_diagnostic_maturity_corpus(&DIAGNOSTIC_FILES, &pins).is_err());
        }
    }

    fn changed_diagnostic_report(field: &str, value: &str, encoded: bool) -> Vec<u8> {
        let report = std::str::from_utf8(DIAGNOSTIC_FILES[3].bytes).expect("UTF-8");
        let replacement = if encoded {
            format!("{field} {} {}", value.len(), hex_bytes(value.as_bytes()))
        } else {
            format!("{field} {value}")
        };
        let mut lines: Vec<_> = report.lines().map(str::to_owned).collect();
        let line = lines
            .iter_mut()
            .find(|line| line.starts_with(&format!("{field} ")))
            .expect("field");
        *line = replacement;
        format!("{}\n", lines.join("\n")).into_bytes()
    }

    macro_rules! diagnostic_grammar_refusal {
        ($name:ident, $field:literal, $value:expr, $encoded:literal) => {
            #[test]
            fn $name() {
                let bytes = changed_diagnostic_report($field, $value, $encoded);
                assert!(parse_diagnostic_report(&bytes, &DIAGNOSTIC_PINS).is_err());
            }
        };
    }
    diagnostic_grammar_refusal!(
        diagnostic_suite_refuses_wrong_test_name,
        "rust-test-name",
        "another_test",
        true
    );
    diagnostic_grammar_refusal!(
        diagnostic_suite_refuses_nonexact_argv,
        "cargo-argv",
        "/workspace/toolchains/cargo/bin/cargo test -p tripod-vectors --test maturity_diagnostic -- --ignored --test-threads=1",
        true
    );
    diagnostic_grammar_refusal!(
        diagnostic_suite_refuses_changed_cargo_path,
        "cargo-argv",
        "/wrong/cargo test -p tripod-vectors --test maturity_diagnostic -- --ignored --exact the_diagnostic_maturity_routes_run_against_a_real_target --test-threads=1",
        true
    );
    diagnostic_grammar_refusal!(
        diagnostic_suite_refuses_wrong_count,
        "observed-test-count",
        "2",
        false
    );
    diagnostic_grammar_refusal!(
        diagnostic_suite_refuses_dirty_provenance,
        "suite-clean",
        "no",
        false
    );
    diagnostic_grammar_refusal!(
        diagnostic_suite_refuses_bad_source_identity,
        "suite-tree",
        "not-source-provenance",
        false
    );
    diagnostic_grammar_refusal!(
        diagnostic_suite_refuses_changed_revision,
        "elementsd-binary-reported-revision",
        "000000000000",
        true
    );
    diagnostic_grammar_refusal!(
        diagnostic_suite_refuses_wrong_revision_length,
        "elementsd-binary-reported-revision",
        "b7fc5d08",
        true
    );
    diagnostic_grammar_refusal!(
        diagnostic_suite_refuses_changed_intended_tip,
        "elementsd-intended-executed-tip",
        &"0".repeat(40),
        true
    );
    diagnostic_grammar_refusal!(
        diagnostic_suite_refuses_wrong_adapter,
        "executor-adapter-version",
        "2.0.0",
        true
    );
    diagnostic_grammar_refusal!(
        diagnostic_suite_refuses_wrong_protocol,
        "protocol-revision",
        "7",
        false
    );
    diagnostic_grammar_refusal!(
        diagnostic_suite_refuses_wrong_target,
        "deployment-target-contract",
        "another-target",
        true
    );
    diagnostic_grammar_refusal!(
        diagnostic_tail_refuses_wrong_profile,
        "capture-profile",
        "native",
        false
    );
    diagnostic_grammar_refusal!(
        diagnostic_tail_refuses_phase_order,
        "phase-wall-ms",
        "cargo 42",
        false
    );
    diagnostic_grammar_refusal!(
        diagnostic_tail_refuses_phase_wall,
        "phase-wall-ms",
        "preflight -1",
        false
    );
    diagnostic_grammar_refusal!(
        diagnostic_tail_refuses_elapsed_wall,
        "elapsed-wall-ms",
        "01",
        false
    );
    diagnostic_grammar_refusal!(
        diagnostic_tail_refuses_campaign_schema,
        "campaign-manifest-schema",
        "2",
        false
    );
    diagnostic_grammar_refusal!(
        diagnostic_tail_refuses_campaign_count,
        "campaign-manifest-count",
        "1",
        false
    );
    diagnostic_grammar_refusal!(diagnostic_tail_refuses_ineligible, "eligible", "no", false);

    #[test]
    fn diagnostic_tail_refuses_missing_duplicate_and_surplus_lines() {
        let original = std::str::from_utf8(DIAGNOSTIC_FILES[3].bytes).expect("report");
        let policy = original
            .lines()
            .find(|line| line.starts_with("diagnostic-policy "))
            .expect("policy");
        for bytes in [
            original.replace(&format!("{policy}\n"), ""),
            original.replace(
                "campaign-manifest-count 23\n",
                "campaign-manifest-count 23\ncampaign-manifest-count 23\n",
            ),
            format!("{original}surplus yes\n"),
        ] {
            assert!(parse_diagnostic_report(bytes.as_bytes(), &DIAGNOSTIC_PINS).is_err());
        }
    }

    #[test]
    fn diagnostic_policy_requires_node_defaults_and_provenance() {
        let (_, fields) =
            parse_diagnostic_report(DIAGNOSTIC_FILES[3].bytes, &DIAGNOSTIC_PINS).expect("report");
        let policy = retained_json(&fields.policy).expect("policy");
        for (field, value) in [
            ("unit", serde_json::json!("units")),
            ("block_identifier", serde_json::json!("other")),
            ("block", serde_json::json!("0.00000200")),
            ("relay_provenance", serde_json::json!("assumed")),
            ("block_provenance", serde_json::json!("assumed")),
            (
                "start_arguments",
                serde_json::json!(["-daemon=0", "-minrelaytxfee=0"]),
            ),
            (
                "start_arguments",
                serde_json::json!(["-daemon=0", "-blockmintxfee=0"]),
            ),
        ] {
            let mut changed = policy.clone();
            changed[field] = value;
            assert_eq!(
                crate::maturity_native::sponsored::Policy::parse(changed),
                Err(MaturityNativePlanRefusal::SponsoredPolicy)
            );
        }
        assert!(crate::maturity_native::sponsored::Policy::parse(serde_json::Value::Null).is_err());
    }

    #[test]
    fn diagnostic_replay_refuses_changed_routes_and_readback() {
        for readback in [false, true] {
            let result = admit_diagnostic_with_replay(
                &DIAGNOSTIC_FILES,
                &DIAGNOSTIC_PINS,
                |identity, branch, exchanges| {
                    let mut changed = exchanges.to_vec();
                    let response = &mut changed[4].1;
                    if readback {
                        response
                            .mined_readback
                            .as_mut()
                            .expect("readback")
                            .raw_transaction[0] ^= 1;
                    } else {
                        let mut detail: serde_json::Value = serde_json::from_str(
                            response.observed_detail.as_deref().expect("detail"),
                        )
                        .expect("JSON");
                        detail["route"] = serde_json::json!("setup");
                        response.observed_detail =
                            Some(serde_json::to_string(&detail).expect("JSON"));
                    }
                    crate::maturity_native::native::replay_diagnostic_routes(
                        identity, branch, &changed,
                    )
                },
            );
            assert_eq!(
                result.err(),
                Some(Refusal::Derivation(if readback {
                    MaturityNativePlanRefusal::SubmissionReadbackMismatch
                } else {
                    MaturityNativePlanRefusal::SponsoredRoute
                }))
            );
        }
    }
    #[test]
    fn retained_admission_binds_bytes_provenance_census_routes_and_readback() {
        let corpus = admit_retained_maturity_corpus(&RETAINED_FILES, &RETAINED_PINS)
            .expect("retained archive");
        assert_eq!(corpus.report.run_address(), MATURITY_RETAINED_RUN_ADDRESS);
        assert_eq!(corpus.evidence.exchanges().len(), 9);
        let families = corpus.evidence.families();
        assert_eq!(families.len(), 2);
        assert_eq!(corpus.evidence.stopped(), [] as [&str; 0]);
        assert_eq!(corpus.evidence.complete(), Ok(()));
        for family in &families {
            assert_eq!(family.expectation(), "agreement");
            assert!(family.accepted_control_identity().is_some());
            assert_eq!(family.control_coins.len(), 1);
            let (_, response) = family.control.expect("accepted control");
            assert_eq!(
                crate::maturity_native::sponsored::detail(response)
                    .expect("route")
                    .1,
                Some(MaturityAcceptanceRoute::DirectBlockAfterRelayRefusal)
            );
        }
        assert_eq!(families[0].subject_coins.len(), 1);
        assert_eq!(families[1].subject_coins.len(), 2);
        assert_eq!(
            families[0].subject_coins[0].1,
            families[0].control_coins[0].1
        );
        assert_ne!(
            families[1].subject_coins[0].1.outpoint,
            families[1].control_coins[0].1.outpoint
        );
        assert_eq!(
            corpus.fee_floors.provenance(),
            MaturityPremiseProvenance::ReportedByNode
        );
        assert_eq!(
            corpus.fee_floors.block_provenance(),
            MaturityPremiseProvenance::PinnedSourceNoOverride
        );
    }

    #[test]
    fn retained_report_address_refuses_a_grammar_valid_changed_wall() {
        let mut bytes = RETAINED_FILES[3].bytes.to_vec();
        let prefix = b"phase-wall-ms preflight ";
        let index = bytes
            .windows(prefix.len())
            .position(|part| part == prefix)
            .expect("phase wall")
            + prefix.len();
        bytes[index] = if bytes[index] == b'1' { b'2' } else { b'1' };
        assert!(parse_retained_report(&bytes, &RETAINED_PINS).is_ok());
        let mut files = RETAINED_FILES;
        files[3] = MaturityArchiveFile::new(files[3].name, &bytes);
        assert_eq!(
            admit_retained_maturity_corpus(&files, &RETAINED_PINS).err(),
            Some(Refusal::RunReportAddress)
        );
    }

    #[test]
    fn retained_node_tip_refuses_a_consistent_different_revision() {
        let tip = "1".repeat(40);
        let binary = &tip[..12];
        let version = format!("Elements Core daemon version v28.99.0-{binary}");
        let original = std::str::from_utf8(RETAINED_FILES[3].bytes).expect("report");
        let lines: Vec<_> = original
            .lines()
            .map(|line| {
                if line.starts_with("elementsd-expected-tip ") {
                    return format!("elementsd-expected-tip {tip}");
                }
                for (field, value) in [
                    ("elementsd-binary-reported-revision", binary),
                    ("elementsd-intended-executed-tip", tip.as_str()),
                    ("node-version", version.as_str()),
                ] {
                    if line.starts_with(&format!("{field} ")) {
                        return format!("{field} {} {}", value.len(), hex_bytes(value.as_bytes()));
                    }
                }
                line.to_owned()
            })
            .collect();
        let bytes = format!("{}\n", lines.join("\n")).into_bytes();
        assert_eq!(
            parse_retained_report(&bytes, &RETAINED_PINS).err(),
            Some(Refusal::CrossFileBinding {
                field: "retained-node-tip"
            })
        );
    }

    #[test]
    fn retained_schedule_pin_refuses_another_schedule() {
        let pins = MaturityCorpusPins {
            schedule: tapscript::StateWitnessSchedule::WholeMetadata,
            ..RETAINED_PINS
        };
        assert_eq!(
            admit_retained_maturity_corpus(&RETAINED_FILES, &pins).err(),
            Some(Refusal::CrossFileBinding {
                field: "retained-schedule"
            })
        );
    }

    #[test]
    fn retained_one_family_record_names_the_absent_family() {
        let (_, mut fields) =
            parse_retained_report(RETAINED_FILES[3].bytes, &RETAINED_PINS).expect("report");
        let mut families = retained_json(&fields.families).expect("families");
        families.as_array_mut().expect("array").pop();
        fields.families = serde_json::to_vec(&families).expect("JSON");
        assert_eq!(
            validate_retained_fields(&fields),
            Err(Refusal::RetainedCampaign(
                crate::maturity_native::native::RetainedCampaignRefusal {
                    family: "two-state-successors".to_owned(),
                    field: "absent family",
                }
            ))
        );
    }

    #[test]
    fn retained_archive_pins_refuse_changed_files() {
        for index in 0..4 {
            let mut bytes = RETAINED_FILES[index].bytes.to_vec();
            bytes[0] ^= 1;
            let mut files = RETAINED_FILES;
            files[index] = MaturityArchiveFile::new(files[index].name, &bytes);
            assert!(admit_retained_maturity_corpus(&files, &RETAINED_PINS).is_err());
            let mut longer = bytes.clone();
            longer.push(b'\n');
            files[index] = MaturityArchiveFile::new(files[index].name, &longer);
            assert!(matches!(
                validate_census(&files, &RETAINED_PINS),
                Err(Refusal::FileSize { .. })
            ));
        }
        let mut files = RETAINED_FILES;
        files.swap(0, 1);
        assert_eq!(
            validate_census(&files, &RETAINED_PINS),
            Err(Refusal::CorpusCensus)
        );
        for pins in [
            MaturityCorpusPins {
                manifest_sha256: &"0".repeat(64),
                ..RETAINED_PINS
            },
            MaturityCorpusPins {
                run_address: &"0".repeat(64),
                ..RETAINED_PINS
            },
            MaturityCorpusPins {
                ceremony_sha256: &"0".repeat(64),
                ..RETAINED_PINS
            },
            MaturityCorpusPins {
                cargo_path: "/wrong/cargo",
                ..RETAINED_PINS
            },
            MaturityCorpusPins {
                capture_target: MaturityCaptureTarget::new("wrong", "wrong"),
                ..RETAINED_PINS
            },
            MaturityCorpusPins {
                funding_count: 1,
                ..RETAINED_PINS
            },
            MaturityCorpusPins {
                framework_revision: MaturityFrameworkRevision::Unrecorded,
                ..RETAINED_PINS
            },
            MaturityCorpusPins {
                fee_floors: MaturityDisclosedFeeFloors::node_defaults("0.00000200", "0.00000100"),
                ..RETAINED_PINS
            },
        ] {
            assert!(admit_retained_maturity_corpus(&RETAINED_FILES, &pins).is_err());
        }
    }

    #[test]
    fn confidential_report_address_refuses_a_grammar_valid_changed_wall() {
        let mut bytes = CONFIDENTIAL_FILES[3].bytes.to_vec();
        let prefix = b"phase-wall-ms preflight ";
        let index = bytes
            .windows(prefix.len())
            .position(|part| part == prefix)
            .expect("phase wall")
            + prefix.len();
        bytes[index] = if bytes[index] == b'1' { b'2' } else { b'1' };
        assert!(parse_confidential_report(&bytes, &CONFIDENTIAL_PINS).is_ok());
        let mut files = CONFIDENTIAL_FILES;
        files[3] = MaturityArchiveFile::new(files[3].name, &bytes);
        assert_eq!(
            admit_confidential_maturity_corpus(&files, &CONFIDENTIAL_PINS).err(),
            Some(Refusal::RunReportAddress)
        );
    }

    #[test]
    fn confidential_node_tip_refuses_a_consistent_different_revision() {
        let tip = "1".repeat(40);
        let binary = &tip[..12];
        let version = format!("Elements Core daemon version v28.99.0-{binary}");
        let original = std::str::from_utf8(CONFIDENTIAL_FILES[3].bytes).expect("report");
        let lines: Vec<_> = original
            .lines()
            .map(|line| {
                if line.starts_with("elementsd-expected-tip ") {
                    return format!("elementsd-expected-tip {tip}");
                }
                for (field, value) in [
                    ("elementsd-binary-reported-revision", binary),
                    ("elementsd-intended-executed-tip", tip.as_str()),
                    ("node-version", version.as_str()),
                ] {
                    if line.starts_with(&format!("{field} ")) {
                        return format!("{field} {} {}", value.len(), hex_bytes(value.as_bytes()));
                    }
                }
                line.to_owned()
            })
            .collect();
        let bytes = format!("{}\n", lines.join("\n")).into_bytes();
        assert_eq!(
            parse_confidential_report(&bytes, &CONFIDENTIAL_PINS).err(),
            Some(Refusal::CrossFileBinding {
                field: "confidential-node-tip"
            })
        );
    }

    #[test]
    fn confidential_schedule_pin_refuses_another_schedule() {
        let pins = MaturityCorpusPins {
            schedule: tapscript::StateWitnessSchedule::WholeMetadata,
            ..CONFIDENTIAL_PINS
        };
        assert_eq!(
            admit_confidential_maturity_corpus(&CONFIDENTIAL_FILES, &pins).err(),
            Some(Refusal::CrossFileBinding {
                field: "confidential-schedule"
            })
        );
    }

    #[test]
    fn confidential_archive_pins_refuse_changed_files() {
        for index in 0..4 {
            let mut bytes = CONFIDENTIAL_FILES[index].bytes.to_vec();
            bytes[0] ^= 1;
            let mut files = CONFIDENTIAL_FILES;
            files[index] = MaturityArchiveFile::new(files[index].name, &bytes);
            assert!(admit_confidential_maturity_corpus(&files, &CONFIDENTIAL_PINS).is_err());
            let mut longer = bytes.clone();
            longer.push(b'\n');
            files[index] = MaturityArchiveFile::new(files[index].name, &longer);
            assert!(matches!(
                validate_census(&files, &CONFIDENTIAL_PINS),
                Err(Refusal::FileSize { .. })
            ));
        }
        let mut files = CONFIDENTIAL_FILES;
        files.swap(0, 1);
        assert_eq!(
            validate_census(&files, &CONFIDENTIAL_PINS),
            Err(Refusal::CorpusCensus)
        );
        for pins in [
            MaturityCorpusPins {
                manifest_sha256: &"0".repeat(64),
                ..CONFIDENTIAL_PINS
            },
            MaturityCorpusPins {
                run_address: &"0".repeat(64),
                ..CONFIDENTIAL_PINS
            },
            MaturityCorpusPins {
                ceremony_sha256: &"0".repeat(64),
                ..CONFIDENTIAL_PINS
            },
            MaturityCorpusPins {
                cargo_path: "/wrong/cargo",
                ..CONFIDENTIAL_PINS
            },
            MaturityCorpusPins {
                capture_target: MaturityCaptureTarget::new("wrong", "wrong"),
                ..CONFIDENTIAL_PINS
            },
            MaturityCorpusPins {
                funding_count: 1,
                ..CONFIDENTIAL_PINS
            },
            MaturityCorpusPins {
                framework_revision: MaturityFrameworkRevision::Unrecorded,
                ..CONFIDENTIAL_PINS
            },
            MaturityCorpusPins {
                fee_floors: MaturityDisclosedFeeFloors::node_defaults("0.00000200", "0.00000100"),
                ..CONFIDENTIAL_PINS
            },
        ] {
            assert!(admit_confidential_maturity_corpus(&CONFIDENTIAL_FILES, &pins).is_err());
        }
    }

    #[test]
    fn confidential_admission_binds_bytes_provenance_census_route_and_readback() {
        let corpus = admit_confidential_maturity_corpus(&CONFIDENTIAL_FILES, &CONFIDENTIAL_PINS)
            .expect("confidential archive");
        assert_eq!(
            corpus.report.run_address(),
            MATURITY_CONFIDENTIAL_RUN_ADDRESS
        );
        assert_eq!(corpus.report.expected_tip(), RETAINED_NODE_TIP);
        assert_eq!(corpus.evidence.exchanges().len(), 14);
        assert_eq!(corpus.evidence.admit(), Ok(()));
        let families = corpus.evidence.families_json();
        assert_eq!(families.as_array().expect("families").len(), 1);
        assert_eq!(families[0]["expectation"], "agreement");
        assert_eq!(families[0]["control_accepted"], true);
        assert_eq!(families[0]["stopped"], false);
        assert_eq!(families[0]["eligible"], true);
        assert_eq!(families[0]["refusal"], serde_json::Value::Null);
        for (index, fee, change, height) in [(7, 141, 1_109, 7), (13, 409, 841, 10)] {
            let (step, response) = &corpus.evidence.exchanges()[index];
            assert_eq!(response.observed_layer, ObservedOutcomeLayer::Accepted);
            assert_eq!(
                crate::maturity_native::sponsored::detail(response)
                    .expect("route")
                    .1,
                Some(MaturityAcceptanceRoute::RelayAndTemplate)
            );
            let OperationSubject::Submission(subject) = step.subject() else {
                panic!("submission")
            };
            let readback = response.mined_readback.as_ref().expect("exact readback");
            assert_eq!(readback.raw_transaction, subject.transaction_bytes);
            let slot = if index == 7 { "control" } else { "subject" };
            assert_eq!(families[0][slot]["fee"], fee);
            assert_eq!(families[0][slot]["change"], change);
            assert_eq!(fee + change, 1_250);
            assert_eq!(readback.block_height, height);
        }
        assert_eq!(
            corpus.fee_floors.provenance(),
            MaturityPremiseProvenance::ReportedByNode
        );
        assert_eq!(
            corpus.fee_floors.block_provenance(),
            MaturityPremiseProvenance::PinnedSourceNoOverride
        );
    }

    fn confidential_fields() -> crate::maturity_native::confidential::ConfidentialCampaignFields {
        parse_confidential_report(CONFIDENTIAL_FILES[3].bytes, &CONFIDENTIAL_PINS)
            .expect("report")
            .1
            .decoded()
            .expect("fields")
    }

    #[test]
    fn confidential_campaign_refuses_each_changed_field() {
        use crate::maturity_native::confidential::{ConfidentialCampaignRefusal, FAMILY};
        let corpus = maturity_confidential_run_of_record().expect("admitted archive");
        let fail = |field| {
            Err(ConfidentialCampaignRefusal {
                family: FAMILY.to_owned(),
                field,
            })
        };
        for key in [
            "family",
            "claim",
            "control",
            "subject",
            "stopped",
            "expectation",
            "control_accepted",
            "eligible",
            "refusal",
        ] {
            let mut fields = confidential_fields();
            fields.families[0][key] = serde_json::json!("changed");
            assert_eq!(fields.validate(&corpus.evidence), fail(key), "{key}");
            assert_eq!(fields.admit(&corpus.evidence), fail(key), "{key}");
        }
        for (slot, key) in [
            ("control", "digest"),
            ("subject", "digest"),
            ("control", "funded_outpoints"),
            ("subject", "readback"),
        ] {
            let mut fields = confidential_fields();
            fields.families[0][slot][key] = serde_json::json!("changed");
            assert_eq!(fields.validate(&corpus.evidence), fail(slot));
        }
        let mut fields = confidential_fields();
        fields.families[0]["surplus"] = serde_json::json!(true);
        assert_eq!(fields.validate(&corpus.evidence), fail("family keys"));
        let mut fields = confidential_fields();
        fields.stopped = serde_json::json!([FAMILY]);
        assert_eq!(fields.validate(&corpus.evidence), fail("stopped set"));
        let mut fields = confidential_fields();
        fields.policy["relay"] = serde_json::json!("0");
        assert_eq!(fields.validate(&corpus.evidence), fail("policy"));
        let keys: Vec<_> = confidential_fields().rows[0]
            .as_object()
            .expect("row")
            .keys()
            .cloned()
            .collect();
        for key in keys {
            let mut fields = confidential_fields();
            fields.rows[0][&key] = serde_json::json!("changed");
            assert!(fields.validate(&corpus.evidence).is_err(), "{key}");
        }
    }

    #[test]
    fn confidential_preflight_is_a_reading_and_requires_exact_profiles() {
        let mut fields = confidential_fields();
        fields.preflight["library_sha256"] = serde_json::json!("a".repeat(64));
        assert_eq!(fields.validate_preflight(), Ok(()));
        for value in ["A".repeat(64), "a".repeat(63), "g".repeat(64)] {
            fields.preflight["library_sha256"] = serde_json::json!(value);
            assert!(fields.validate_preflight().is_err());
        }
        for key in ["library_configured", "library_readable", "probe", "capture"] {
            let mut fields = confidential_fields();
            fields.preflight[key] = serde_json::json!(false);
            assert!(fields.validate_preflight().is_err(), "{key}");
        }
        for reading in ["probe", "capture"] {
            let mut fields = confidential_fields();
            fields.preflight[reading]["profiles"]["materializer"] = serde_json::json!("changed");
            assert!(fields.validate_preflight().is_err());
        }
        let mut fields = confidential_fields();
        fields.preflight["surplus"] = serde_json::json!(true);
        assert!(fields.validate_preflight().is_err());
    }

    #[test]
    fn confidential_admission_refuses_contradicted_pending_and_stopped_families() {
        for (key, value) in [
            ("expectation", serde_json::json!("contradiction")),
            ("expectation", serde_json::json!("pending")),
            ("stopped", serde_json::json!(true)),
            ("eligible", serde_json::json!(false)),
            ("control_accepted", serde_json::json!(false)),
            ("refusal", serde_json::json!("readback")),
        ] {
            let mut fields = confidential_fields();
            fields.families[0][key] = value;
            assert_eq!(
                fields.require_agreement().expect_err("not admitted").field,
                key
            );
        }
        let mut fields = confidential_fields();
        fields.stopped = serde_json::json!(["confidential-sponsor"]);
        assert_eq!(
            fields.require_agreement().expect_err("stopped").field,
            "stopped set"
        );
    }

    #[test]
    fn confidential_environment_requires_its_exact_capabilities_and_funding() {
        let (report, _) =
            parse_confidential_report(CONFIDENTIAL_FILES[3].bytes, &CONFIDENTIAL_PINS)
                .expect("report");
        for pins in [
            MaturityCorpusPins {
                capabilities: &CAPABILITIES,
                ..CONFIDENTIAL_PINS
            },
            MaturityCorpusPins {
                funding: &[],
                ..CONFIDENTIAL_PINS
            },
            MaturityCorpusPins {
                funding_count: 0,
                ..CONFIDENTIAL_PINS
            },
        ] {
            let mut cursor = Cursor::new("capture", CONFIDENTIAL_FILES[0].bytes).expect("capture");
            parse_header(&mut cursor, &report, &pins).expect("header");
            assert!(parse_environment(&mut cursor, &report, &pins).is_err());
        }
    }

    #[test]
    fn confidential_funding_subject_is_scoped_to_the_confidential_parser() {
        let corpus = maturity_confidential_run_of_record().expect("archive");
        let text =
            serde_json::to_string(corpus.evidence.exchanges()[9].0.subject()).expect("subject");
        assert!(parse_subject(&text, SubjectKind::Confidential).is_some());
        for kind in [
            SubjectKind::Sponsored,
            SubjectKind::Funding,
            SubjectKind::Submission,
        ] {
            assert!(parse_subject(&text, kind).is_none());
        }
    }

    #[test]
    fn confidential_preflight_line_must_appear_once_at_its_declared_position() {
        let original = std::str::from_utf8(CONFIDENTIAL_FILES[3].bytes).expect("report");
        let line = original
            .lines()
            .find(|line| line.starts_with("confidential-preflight "))
            .expect("preflight");
        let missing = original.replace(&format!("{line}\n"), "");
        for bytes in [
            missing.clone(),
            original.replace(line, &format!("{line}\n{line}")),
            missing.replace(
                "ceremony-roster begin",
                &format!("{line}\nceremony-roster begin"),
            ),
            format!("{missing}{line}\n"),
        ] {
            assert!(parse_confidential_report(bytes.as_bytes(), &CONFIDENTIAL_PINS).is_err());
        }
    }

    fn changed_confidential_report(field: &str, value: &str, encoded: bool) -> Vec<u8> {
        let report = std::str::from_utf8(CONFIDENTIAL_FILES[3].bytes).expect("UTF-8");
        let replacement = if encoded {
            format!("{field} {} {}", value.len(), hex_bytes(value.as_bytes()))
        } else {
            format!("{field} {value}")
        };
        let mut lines: Vec<_> = report.lines().map(str::to_owned).collect();
        let line = lines
            .iter_mut()
            .find(|line| line.starts_with(&format!("{field} ")))
            .expect("field");
        *line = replacement;
        format!("{}\n", lines.join("\n")).into_bytes()
    }

    macro_rules! confidential_grammar_refusal {
        ($name:ident, $field:literal, $value:expr, $encoded:literal) => {
            #[test]
            fn $name() {
                let bytes = changed_confidential_report($field, $value, $encoded);
                assert!(parse_confidential_report(&bytes, &CONFIDENTIAL_PINS).is_err());
            }
        };
    }
    confidential_grammar_refusal!(
        confidential_suite_refuses_wrong_test_name,
        "rust-test-name",
        "another_test",
        true
    );
    confidential_grammar_refusal!(
        confidential_suite_refuses_nonexact_argv,
        "cargo-argv",
        "/workspace/toolchains/cargo/bin/cargo test -p tripod-vectors --test maturity_confidential -- --ignored --test-threads=1",
        true
    );
    confidential_grammar_refusal!(
        confidential_suite_refuses_changed_cargo_path,
        "cargo-argv",
        "/wrong/cargo test -p tripod-vectors --test maturity_confidential -- --ignored --exact the_confidential_maturity_route_runs_against_a_real_target --test-threads=1",
        true
    );
    confidential_grammar_refusal!(
        confidential_suite_refuses_wrong_count,
        "observed-test-count",
        "2",
        false
    );
    confidential_grammar_refusal!(
        confidential_suite_refuses_dirty_provenance,
        "suite-clean",
        "no",
        false
    );
    confidential_grammar_refusal!(
        confidential_suite_refuses_bad_source_identity,
        "suite-tree",
        "not-source-provenance",
        false
    );
    confidential_grammar_refusal!(
        confidential_suite_refuses_changed_revision,
        "elementsd-binary-reported-revision",
        "000000000000",
        true
    );
    confidential_grammar_refusal!(
        confidential_suite_refuses_wrong_revision_length,
        "elementsd-binary-reported-revision",
        "b7fc5d08",
        true
    );
    confidential_grammar_refusal!(
        confidential_suite_refuses_changed_intended_tip,
        "elementsd-intended-executed-tip",
        &"0".repeat(40),
        true
    );
    confidential_grammar_refusal!(
        confidential_suite_refuses_wrong_adapter,
        "executor-adapter-version",
        "2.0.0",
        true
    );
    confidential_grammar_refusal!(
        confidential_suite_refuses_wrong_protocol,
        "protocol-revision",
        "7",
        false
    );
    confidential_grammar_refusal!(
        confidential_suite_refuses_wrong_target,
        "deployment-target-contract",
        "another-target",
        true
    );
    confidential_grammar_refusal!(
        confidential_tail_refuses_wrong_profile,
        "capture-profile",
        "native",
        false
    );
    confidential_grammar_refusal!(
        confidential_tail_refuses_phase_order,
        "phase-wall-ms",
        "cargo 42",
        false
    );
    confidential_grammar_refusal!(
        confidential_tail_refuses_phase_wall,
        "phase-wall-ms",
        "preflight -1",
        false
    );
    confidential_grammar_refusal!(
        confidential_tail_refuses_elapsed_wall,
        "elapsed-wall-ms",
        "01",
        false
    );
    confidential_grammar_refusal!(
        confidential_tail_refuses_campaign_schema,
        "campaign-manifest-schema",
        "2",
        false
    );
    confidential_grammar_refusal!(
        confidential_tail_refuses_campaign_count,
        "campaign-manifest-count",
        "2",
        false
    );
    confidential_grammar_refusal!(
        confidential_tail_refuses_ineligible,
        "eligible",
        "no",
        false
    );

    #[test]
    fn confidential_tail_refuses_missing_duplicate_and_surplus_lines() {
        let original = std::str::from_utf8(CONFIDENTIAL_FILES[3].bytes).expect("report");
        let policy = original
            .lines()
            .find(|line| line.starts_with("confidential-policy "))
            .expect("policy");
        for bytes in [
            original.replace(&format!("{policy}\n"), ""),
            original.replace(
                "campaign-manifest-count 1\n",
                "campaign-manifest-count 1\ncampaign-manifest-count 1\n",
            ),
            format!("{original}surplus yes\n"),
        ] {
            assert!(parse_confidential_report(bytes.as_bytes(), &CONFIDENTIAL_PINS).is_err());
        }
    }

    fn changed_retained_report(field: &str, value: &str, encoded: bool) -> Vec<u8> {
        let report = std::str::from_utf8(RETAINED_FILES[3].bytes).expect("UTF-8");
        let replacement = if encoded {
            format!("{field} {} {}", value.len(), hex_bytes(value.as_bytes()))
        } else {
            format!("{field} {value}")
        };
        let mut lines: Vec<_> = report.lines().map(str::to_owned).collect();
        let line = lines
            .iter_mut()
            .find(|line| line.starts_with(&format!("{field} ")))
            .expect("field");
        *line = replacement;
        format!("{}\n", lines.join("\n")).into_bytes()
    }

    macro_rules! retained_grammar_refusal {
        ($name:ident, $field:literal, $value:expr, $encoded:literal) => {
            #[test]
            fn $name() {
                let bytes = changed_retained_report($field, $value, $encoded);
                assert!(parse_retained_report(&bytes, &RETAINED_PINS).is_err());
            }
        };
    }
    retained_grammar_refusal!(
        retained_suite_refuses_wrong_test_name,
        "rust-test-name",
        "another_test",
        true
    );
    retained_grammar_refusal!(
        retained_suite_refuses_nonexact_argv,
        "cargo-argv",
        "/workspace/toolchains/cargo/bin/cargo test -p tripod-vectors --test maturity_retained -- --ignored --test-threads=1",
        true
    );
    retained_grammar_refusal!(
        retained_suite_refuses_changed_cargo_path,
        "cargo-argv",
        "/wrong/cargo test -p tripod-vectors --test maturity_retained -- --ignored --exact the_retained_maturity_routes_run_against_a_real_target --test-threads=1",
        true
    );
    retained_grammar_refusal!(
        retained_suite_refuses_wrong_count,
        "observed-test-count",
        "2",
        false
    );
    retained_grammar_refusal!(
        retained_suite_refuses_dirty_provenance,
        "suite-clean",
        "no",
        false
    );
    retained_grammar_refusal!(
        retained_suite_refuses_bad_source_identity,
        "suite-tree",
        "not-source-provenance",
        false
    );
    retained_grammar_refusal!(
        retained_suite_refuses_changed_revision,
        "elementsd-binary-reported-revision",
        "000000000000",
        true
    );
    retained_grammar_refusal!(
        retained_suite_refuses_wrong_revision_length,
        "elementsd-binary-reported-revision",
        "b7fc5d08",
        true
    );
    retained_grammar_refusal!(
        retained_suite_refuses_changed_intended_tip,
        "elementsd-intended-executed-tip",
        &"0".repeat(40),
        true
    );
    retained_grammar_refusal!(
        retained_suite_refuses_wrong_adapter,
        "executor-adapter-version",
        "2.0.0",
        true
    );
    retained_grammar_refusal!(
        retained_suite_refuses_wrong_protocol,
        "protocol-revision",
        "7",
        false
    );
    retained_grammar_refusal!(
        retained_suite_refuses_wrong_target,
        "deployment-target-contract",
        "another-target",
        true
    );
    retained_grammar_refusal!(
        retained_tail_refuses_wrong_profile,
        "capture-profile",
        "native",
        false
    );
    retained_grammar_refusal!(
        retained_tail_refuses_phase_order,
        "phase-wall-ms",
        "cargo 42",
        false
    );
    retained_grammar_refusal!(
        retained_tail_refuses_phase_wall,
        "phase-wall-ms",
        "preflight -1",
        false
    );
    retained_grammar_refusal!(
        retained_tail_refuses_elapsed_wall,
        "elapsed-wall-ms",
        "01",
        false
    );
    retained_grammar_refusal!(
        retained_tail_refuses_campaign_schema,
        "campaign-manifest-schema",
        "2",
        false
    );
    retained_grammar_refusal!(
        retained_tail_refuses_campaign_count,
        "campaign-manifest-count",
        "1",
        false
    );
    retained_grammar_refusal!(retained_tail_refuses_ineligible, "eligible", "no", false);

    #[test]
    fn retained_tail_refuses_missing_duplicate_and_surplus_lines() {
        let original = std::str::from_utf8(RETAINED_FILES[3].bytes).expect("report");
        let policy = original
            .lines()
            .find(|line| line.starts_with("retained-policy "))
            .expect("policy");
        for bytes in [
            original.replace(&format!("{policy}\n"), ""),
            original.replace(
                "campaign-manifest-count 2\n",
                "campaign-manifest-count 2\ncampaign-manifest-count 2\n",
            ),
            format!("{original}surplus yes\n"),
        ] {
            assert!(parse_retained_report(bytes.as_bytes(), &RETAINED_PINS).is_err());
        }
    }

    #[test]
    fn retained_families_refuse_stopped_absent_and_contradicting_subjects() {
        for index in 0..2 {
            for (field, value, reason) in [
                ("stopped", serde_json::json!(true), "stopped"),
                ("subject", serde_json::Value::Null, "absent subject"),
                (
                    "expectation",
                    serde_json::json!("contradiction"),
                    "expectation",
                ),
                (
                    "control_accepted",
                    serde_json::json!(false),
                    "accepted control",
                ),
            ] {
                let (_, mut fields) =
                    parse_retained_report(RETAINED_FILES[3].bytes, &RETAINED_PINS).expect("report");
                let mut families = retained_json(&fields.families).expect("families");
                let family = families[index]["family"].as_str().expect("name").to_owned();
                families[index][field] = value;
                fields.families = serde_json::to_vec(&families).expect("JSON");
                assert_eq!(
                    validate_retained_fields(&fields),
                    Err(Refusal::RetainedCampaign(
                        crate::maturity_native::native::RetainedCampaignRefusal {
                            family,
                            field: reason
                        }
                    ))
                );
            }
        }
    }

    #[test]
    fn retained_campaign_refuses_each_changed_coordinate() {
        use crate::maturity_native::native::RetainedCampaignManifest;
        let corpus = maturity_retained_run_of_record().expect("archive");
        let (_, fields) =
            parse_retained_report(RETAINED_FILES[3].bytes, &RETAINED_PINS).expect("report");
        let rows: Vec<_> = fields
            .rows
            .iter()
            .map(|bytes| retained_json(bytes).expect("row"))
            .collect();
        assert_eq!(
            RetainedCampaignManifest::validate(&corpus.evidence, &rows),
            Ok(())
        );
        for index in 0..2 {
            for pointer in [
                "/claim/name",
                "/claim/section",
                "/control_id",
                "/control/digest",
                "/observation/digest",
                "/observation/funded_outpoints/0/output/outpoint/txid",
                "/observation/funded_outpoints/0/funding_step",
                "/compatibility",
                "/candidate/subject_pin",
                "/candidate/schedule",
                "/abi",
                "/deployment/network",
                "/branch/checkpoint",
                "/policy_requirement",
                "/policy",
                "/constructor",
                "/mutation",
                "/declared_boundary",
                "/expected_layer",
                "/expected_consumer",
                "/operation",
                "/target",
                "/family",
                "/control/claims/0/name",
            ] {
                let mut changed = rows.clone();
                *changed[index].pointer_mut(pointer).expect("coordinate") =
                    serde_json::json!("changed");
                let refusal = RetainedCampaignManifest::validate(&corpus.evidence, &changed)
                    .expect_err("changed coordinate");
                assert_eq!(
                    refusal.family,
                    rows[index]["family"].as_str().expect("family")
                );
                assert_eq!(refusal.field, pointer.split('/').nth(1).expect("field"));
            }
        }
        let mut changed = rows.clone();
        changed[0]["surplus"] = serde_json::json!(true);
        assert_eq!(
            RetainedCampaignManifest::validate(&corpus.evidence, &changed)
                .expect_err("surplus")
                .field,
            "field census"
        );
        assert!(RetainedCampaignManifest::validate(&corpus.evidence, &rows[..1]).is_err());
    }

    #[test]
    fn retained_policy_requires_node_defaults_and_provenance() {
        let (_, fields) =
            parse_retained_report(RETAINED_FILES[3].bytes, &RETAINED_PINS).expect("report");
        let policy = retained_json(&fields.policy).expect("policy");
        for (field, value) in [
            ("unit", serde_json::json!("units")),
            ("block_identifier", serde_json::json!("other")),
            ("block", serde_json::json!("0.00000200")),
            ("relay_provenance", serde_json::json!("assumed")),
            ("block_provenance", serde_json::json!("assumed")),
            (
                "start_arguments",
                serde_json::json!(["-daemon=0", "-minrelaytxfee=0"]),
            ),
            (
                "start_arguments",
                serde_json::json!(["-daemon=0", "-blockmintxfee=0"]),
            ),
        ] {
            let mut changed = policy.clone();
            changed[field] = value;
            assert_eq!(
                crate::maturity_native::sponsored::Policy::parse(changed),
                Err(MaturityNativePlanRefusal::SponsoredPolicy)
            );
        }
        assert!(crate::maturity_native::sponsored::Policy::parse(serde_json::Value::Null).is_err());
    }

    #[test]
    fn retained_replay_refuses_changed_routes_and_readback() {
        for readback in [false, true] {
            let result = admit_retained_with_replay(
                &RETAINED_FILES,
                &RETAINED_PINS,
                |identity, branch, exchanges| {
                    let mut changed = exchanges.to_vec();
                    let response = &mut changed[3].1;
                    if readback {
                        response
                            .mined_readback
                            .as_mut()
                            .expect("readback")
                            .raw_transaction[0] ^= 1;
                    } else {
                        let mut detail: serde_json::Value = serde_json::from_str(
                            response.observed_detail.as_deref().expect("detail"),
                        )
                        .expect("JSON");
                        detail["route"] = serde_json::json!("setup");
                        response.observed_detail =
                            Some(serde_json::to_string(&detail).expect("JSON"));
                    }
                    crate::maturity_native::native::replay_retained_routes(
                        identity, branch, &changed,
                    )
                },
            );
            assert_eq!(
                result.err(),
                Some(Refusal::Derivation(if readback {
                    MaturityNativePlanRefusal::SubmissionReadbackMismatch
                } else {
                    MaturityNativePlanRefusal::SponsoredRoute
                }))
            );
        }
    }
    #[test]
    fn native_observation_admission_binds_bytes_policy_census_and_replay() {
        let corpus = super::maturity_native_run_of_record().expect("native archive");
        assert_eq!(corpus.evidence.census(), [137, 63, 27, 47, 21, 26]);
        assert_eq!(corpus.evidence.waiting_json(), [] as [serde_json::Value; 0]);
        for index in 0..4 {
            let mut bytes = super::NATIVE_FILES[index].bytes().to_vec();
            bytes[0] ^= 1;
            let mut files = super::NATIVE_FILES;
            files[index] = super::MaturityArchiveFile::new(files[index].name(), &bytes);
            assert!(super::admit_native_maturity_corpus(&files, &super::NATIVE_PINS).is_err());
        }
        let pins = super::MaturityCorpusPins {
            fee_floors: super::MaturityDisclosedFeeFloors::node_defaults(
                "0.00000200",
                "0.00000100",
            ),
            ..super::NATIVE_PINS
        };
        assert!(super::admit_native_maturity_corpus(&super::NATIVE_FILES, &pins).is_err());
    }

    use super::*;
    use crate::maturity_evidence::{
        MaturityConstructorMaterial, MaturityConstructorMaterialAbsence,
        MaturityExecutorProvenanceExpectation, derive_maturity_evidence_plan_with,
    };
    use crate::maturity_native::MaturityAnnouncementPlanner;
    use target_elements_conformance::executor::TargetOperationPlanner;
    use target_elements_conformance::protocol::{
        FundedOutput, NativeResourceObservation, WireOutpoint,
    };
    use transaction::bytes::TargetTransaction;

    #[test]
    fn admitted_archives_retain_historical_zero_fee_declarations() {
        let historical = maturity_run_of_record().expect("historical archive");
        let accepted = maturity_variable_run_of_record().expect("accepted archive");
        for corpus in [historical, accepted] {
            let standing = corpus.historical_standing();
            assert!(
                standing
                    .historical_candidate()
                    .expect("historical standing")
                    .source_revision
                    .is_some()
            );
            assert!(
                !standing
                    .historical_candidate()
                    .expect("historical standing")
                    .programs
                    .is_empty()
            );
            assert_eq!(corpus.fee_floors(), MATURITY_DISCLOSED_FEE_FLOORS);
            assert_eq!(
                corpus.fee_floors().provenance(),
                MaturityPremiseProvenance::ExecutorArguments
            );
        }
        let floors = MATURITY_DISCLOSED_FEE_FLOORS;
        assert_eq!(floors.min_relay_tx_fee(), "0");
        assert_eq!(floors.block_min_tx_fee(), "0");
    }

    #[test]
    fn the_linked_candidate_and_the_candidate_abi_are_declared_premises() {
        let plan = derive_maturity_evidence_plan_with(
            MaturityExecutorProvenanceExpectation::NotStatedByTheOperator,
            MaturityConstructorMaterial::Absent(
                MaturityConstructorMaterialAbsence::NotSuppliedToDerivation,
            ),
        )
        .expect("evidence plan");
        let linked = MaturityDeclaredPremise::declared(
            plan.bundle().clone(),
            MaturityPremiseProvenance::DeploymentDeclaration,
        );
        let abi = MaturityDeclaredPremise::declared(
            plan.abi().clone(),
            MaturityPremiseProvenance::DeploymentDeclaration,
        );
        assert_eq!(linked.value(), plan.bundle());
        assert_eq!(abi.value(), plan.abi());
        assert_eq!(
            linked.provenance(),
            MaturityPremiseProvenance::DeploymentDeclaration
        );
        assert_eq!(
            abi.provenance(),
            MaturityPremiseProvenance::DeploymentDeclaration
        );
        assert_ne!(
            linked.provenance(),
            MaturityPremiseProvenance::ExecutorArguments
        );
        for archive in [&FILES, &VARIABLE_FILES] {
            let report = std::str::from_utf8(archive[3].bytes()).expect("RUN-REPORT text");
            assert_eq!(archive[3].name(), "RUN-REPORT");
            assert!(!report.contains("linked-candidate"));
            assert!(!report.contains("candidate-abi"));
        }
    }

    #[test]
    fn retained_whole_metadata_replays_all_three_archived_requests_exactly() {
        let corpus = maturity_run_of_record().expect("admitted archive");
        assert_eq!(
            MATURITY_RUN_SCHEDULE,
            tapscript::StateWitnessSchedule::WholeMetadata
        );
        assert_eq!(corpus.schedule(), MATURITY_RUN_SCHEDULE);
        assert_eq!(corpus.exchanges().len(), 3);
        let evidence = replay_maturity_run_of_record(MaturityWitnessSelection::Retained(
            MATURITY_RUN_SCHEDULE,
        ))
        .expect("exact replay of each recorded request");
        assert_eq!(&evidence, corpus.evidence());
        assert_eq!(evidence.schedule(), corpus.schedule());
    }

    #[test]
    fn archive_replay_refuses_a_different_leafs_transcript() {
        let schedule = tapscript::StateWitnessSchedule::VariableMetadata;
        assert_eq!(
            replay_maturity_run_of_record(MaturityWitnessSelection::Retained(schedule)),
            Err(Refusal::Derivation(
                MaturityNativePlanRefusal::TranscriptStepMismatch { position: 0 }
            ))
        );
    }

    #[test]
    fn archive_replay_refuses_whole_metadata_emission() {
        let schedule = MATURITY_RUN_SCHEDULE;
        assert_eq!(
            replay_maturity_run_of_record(MaturityWitnessSelection::Emission(schedule)),
            Err(Refusal::Derivation(MaturityNativePlanRefusal::Closure(
                Box::new(
                    crate::maturity_closure::MaturityClosureRefusal::ReplayOnlySchedule {
                        schedule
                    },
                )
            )))
        );
    }

    struct OwnedCorpus {
        bytes: [Vec<u8>; 4],
    }

    impl OwnedCorpus {
        fn new() -> Self {
            Self {
                bytes: FILES.map(|file| file.bytes.to_vec()),
            }
        }

        fn inputs(&self) -> [MaturityArchiveFile<'_>; 4] {
            std::array::from_fn(|index| MaturityArchiveFile {
                name: FILES[index].name,
                bytes: &self.bytes[index],
            })
        }

        fn pinned(&self) -> ImportResult<ValidatedMaturityCorpus> {
            admit_maturity_corpus(&self.inputs(), &HISTORICAL_PINS)
        }

        fn replace(&mut self, index: usize, from: &str, to: &str) {
            let text = std::str::from_utf8(&self.bytes[index]).expect("text fixture");
            assert!(text.contains(from), "missing mutation operand");
            self.bytes[index] = text.replacen(from, to, 1).into_bytes();
        }

        fn rebind(&mut self, content: bool) -> ImportResult<ValidatedMaturityCorpus> {
            self.rebind_for(
                content,
                MATURITY_RUN_SCHEDULE,
                MaturitySponsorlessExpectation::RelayRefusal {
                    detail: "bad-witness-nonstandard",
                },
            )
        }

        fn rebind_for(
            &mut self,
            content: bool,
            schedule: tapscript::StateWitnessSchedule,
            sponsorless: MaturitySponsorlessExpectation,
        ) -> ImportResult<ValidatedMaturityCorpus> {
            if content {
                let text = std::str::from_utf8(&self.bytes[0]).expect("capture");
                let position = text.find("capture-content-sha256 ").expect("content hash");
                let hash = hex_bytes(&tagged::sha256(&self.bytes[0][..position]));
                let start = position + "capture-content-sha256 ".len();
                self.bytes[0][start..start + 64].copy_from_slice(hash.as_bytes());
            }
            let capture_hash = hex_bytes(&tagged::sha256(&self.bytes[0]));
            let mut manifest = String::new();
            for (file, bytes) in FILES.iter().zip(&self.bytes).take(2) {
                let _ = writeln!(
                    manifest,
                    "{}  {}",
                    hex_bytes(&tagged::sha256(bytes)),
                    file.name
                );
            }
            self.bytes[2] = manifest.into_bytes();
            let manifest_hash = hex_bytes(&tagged::sha256(&self.bytes[2]));
            let report = std::str::from_utf8(&self.bytes[3]).expect("report");
            let mut rebound = String::new();
            for line in report.lines() {
                if line.starts_with("ceremony report ") {
                    let _ = writeln!(rebound, "ceremony report {capture_hash}");
                } else if line.starts_with("manifest-sha256 ") {
                    let _ = writeln!(rebound, "manifest-sha256 {manifest_hash}");
                } else {
                    let _ = writeln!(rebound, "{line}");
                }
            }
            self.bytes[3] = rebound.into_bytes();
            let address = hex_bytes(&tagged::sha256(&self.bytes[3]));
            let sizes = std::array::from_fn(|index| self.bytes[index].len());
            let pins = MaturityCorpusPins::new(
                schedule,
                (HISTORICAL_PINS.names, sizes),
                (&manifest_hash, &address, &capture_hash),
                sponsorless,
                HISTORICAL_PINS.capture_target,
                HISTORICAL_PINS.cargo_path,
                HISTORICAL_PINS.framework_revision,
                HISTORICAL_PINS.funding_count,
                MATURITY_DISCLOSED_FEE_FLOORS,
            );
            admit_maturity_corpus(&self.inputs(), &pins)
        }

        fn payload_replace(&mut self, from: &str, to: &str) {
            let text = std::str::from_utf8(&self.bytes[0]).expect("capture");
            let line = text
                .lines()
                .find(|line| line.starts_with("legacy-rendering "))
                .expect("payload")
                .to_owned();
            let encoded = line.split(' ').nth(2).expect("hex");
            let payload = String::from_utf8(decode_hex(encoded).expect("bytes")).expect("text");
            assert!(payload.contains(from));
            let changed = payload.replacen(from, to, 1);
            self.replace(
                0,
                &line,
                &format!(
                    "legacy-rendering {} {}",
                    changed.len(),
                    hex_bytes(changed.as_bytes())
                ),
            );
        }
    }

    #[test]
    fn corpus_trace_preserves_admission_and_one_byte_short_refusal() {
        if crate::tracing_test::isolate_capture_test(
            module_path!(),
            "corpus_trace_preserves_admission_and_one_byte_short_refusal",
        ) {
            return;
        }
        let control = OwnedCorpus::new();
        let plain_control = control.pinned();
        let (traced_control, control_records) = crate::tracing_test::capture(|| control.pinned());
        assert_eq!(traced_control, plain_control);
        let completion = control_records
            .iter()
            .find(|record| record["fields"]["message"] == "run of record admitted")
            .expect("corpus completion");
        assert_eq!(completion["target"], "vectors::corpus");
        assert_eq!(completion["level"], "INFO");
        assert_eq!(completion["span"]["name"], "admit_run_of_record");
        assert_eq!(completion["fields"]["outcome"], "completed");
        let mut short = OwnedCorpus::new();
        let _ = short.bytes[0].pop();
        let plain_refusal = short.pinned();
        let (traced_refusal, refusal_records) = crate::tracing_test::capture(|| short.pinned());
        assert_eq!(traced_refusal, plain_refusal);
        let completion = refusal_records
            .iter()
            .find(|record| record["fields"]["message"] == "run of record admitted")
            .expect("corpus refusal completion");
        assert_eq!(completion["target"], "vectors::corpus");
        assert_eq!(completion["level"], "DEBUG");
        assert_eq!(completion["span"]["name"], "admit_run_of_record");
        assert_eq!(completion["fields"]["outcome"], "refused");
        assert_eq!(completion["fields"]["refusal_kind"], "file_size");
        assert!(
            refusal_records
                .iter()
                .all(|record| record["level"] != "WARN" && record["level"] != "ERROR")
        );
    }

    fn scripted_response(step: &OperationStep) -> NativeOperationResponse {
        let mut response = NativeOperationResponse {
            schema: target_elements_conformance::protocol::NATIVE_PROTOCOL_SCHEMA,
            case: step.case().clone(),
            observed_layer: ObservedOutcomeLayer::Accepted,
            observed_detail: None,
            issued_asset: None,
            funded_outputs: Vec::new(),
            confidential_funded_outputs: Vec::new(),
            mined_readback: None,
            accepted_txid: None,
            sponsor_witness: Vec::new(),
            script_path_witness: Vec::new(),
            signer_public_key: None,
            signed_profile: None,
            signing_genesis: None,
            signature_bound_to: None,
            resources: NativeResourceObservation::default(),
        };
        match step.subject() {
            OperationSubject::Funding(subject) => {
                let asset = format!("01{}fe", "55".repeat(30));
                if subject.issue_asset {
                    response.issued_asset = Some(asset.clone());
                }
                response.funded_outputs.push(FundedOutput {
                    outpoint: WireOutpoint {
                        txid: format!("02{}fd", "66".repeat(30)),
                        vout: if subject.issue_asset { 3 } else { 7 },
                    },
                    asset,
                    amount_satoshis: subject.amount_per_output,
                    script: hex_bytes(&subject.output_program),
                });
            }
            OperationSubject::Submission(subject) => {
                let candidate =
                    TargetTransaction::decode(&subject.transaction_bytes).expect("decode");
                let identity = Txid::from_internal(tagged::sha256(&tagged::sha256(
                    &candidate.encode_without_witness(),
                )))
                .to_target_display();
                response.accepted_txid = Some(identity.clone());
                response.mined_readback = Some(MinedFundingReadback {
                    transaction_id: identity,
                    witness_transaction_id: Txid::from_internal(tagged::sha256(&tagged::sha256(
                        &subject.transaction_bytes,
                    )))
                    .to_target_display(),
                    block_hash: "77".repeat(32),
                    block_height: 11,
                    raw_transaction: subject.transaction_bytes.clone(),
                });
            }
            _ => panic!("scripted maturity exchange"),
        }
        response.validate_shape().expect("scripted shape");
        response
    }

    fn scripted_variable_run() -> (
        Vec<(OperationStep, NativeOperationResponse)>,
        MaturityNativeEvidence,
    ) {
        let historical = maturity_run_of_record().expect("historical corpus");
        let identity = historical.evidence().identity().clone();
        let branch = historical.evidence().branch();
        let selection =
            MaturityWitnessSelection::Retained(tapscript::StateWitnessSchedule::VariableMetadata);
        let mut planner = MaturityAnnouncementPlanner::new(identity.clone(), branch, selection)
            .expect("variable planner");
        let mut current = planner.next_step(None).expect("first step");
        while let Some(step) = current {
            let response = scripted_response(&step);
            current = planner
                .next_step(Some((step.case(), &response)))
                .expect("settled step");
        }
        let exchanges = planner.completed_transcript().expect("complete").to_vec();
        let evidence =
            MaturityNativeEvidence::from_transcript(identity, branch, selection, &exchanges)
                .expect("exact replay");
        (exchanges, evidence)
    }

    fn write_scripted_field(out: &mut String, name: &str, bytes: &[u8]) {
        let _ = writeln!(out, "{name} {} {}", bytes.len(), hex_bytes(bytes));
    }

    fn scripted_operation_lines(exchanges: &[(OperationStep, NativeOperationResponse)]) -> String {
        let mut out = String::new();
        let _ = writeln!(out, "operation-count {}", exchanges.len());
        for (index, (step, response)) in exchanges.iter().enumerate() {
            let _ = writeln!(out, "operation {index} begin");
            write_scripted_field(
                &mut out,
                "operation-id",
                format!("operation-{index}").as_bytes(),
            );
            write_scripted_field(
                &mut out,
                "request-id",
                format!("request-{index}").as_bytes(),
            );
            let _ = writeln!(out, "request-role auxiliary");
            let request_bytes = match step.subject() {
                OperationSubject::Funding(_)
                | OperationSubject::SponsorFunding(_)
                | OperationSubject::SponsorSigning(_) => &[][..],
                OperationSubject::Submission(subject) => subject.transaction_bytes.as_slice(),
                _ => panic!("maturity subject"),
            };
            write_scripted_field(&mut out, "request-bytes", request_bytes);
            write_scripted_field(
                &mut out,
                "request-subject",
                &serde_json::to_vec(step.subject()).expect("subject JSON"),
            );
            write_scripted_field(
                &mut out,
                "response-id",
                format!("response-{index}").as_bytes(),
            );
            write_scripted_field(
                &mut out,
                "response-request-id",
                format!("request-{index}").as_bytes(),
            );
            write_scripted_field(
                &mut out,
                "response-operation-id",
                format!("operation-{index}").as_bytes(),
            );
            let layer = match response.observed_layer {
                ObservedOutcomeLayer::Accepted => "accepted",
                ObservedOutcomeLayer::ScriptPathRejection => "script-path-rejection",
                ObservedOutcomeLayer::ConsensusRejectionBeforeScript => {
                    "consensus-rejection-before-script"
                }
                ObservedOutcomeLayer::ExecutorInfrastructureFailure => {
                    "executor-infrastructure-failure"
                }
                _ => panic!("unexpected scripted layer"),
            };
            let verdict = if layer == "accepted" {
                "accepted"
            } else {
                "refused"
            };
            let _ = writeln!(out, "response-verdict {verdict}\nresponse-layer {layer}");
            let _ = writeln!(
                out,
                "response-target-identity {}",
                response.accepted_txid.as_deref().unwrap_or("none")
            );
            write_scripted_field(
                &mut out,
                "response-detail",
                response
                    .observed_detail
                    .as_deref()
                    .unwrap_or_default()
                    .as_bytes(),
            );
            for line in [
                "attribution-control-request-id none",
                "attribution-control-identity none",
                "mutation-kind none",
                "mutation-locator none",
                "projection-input none",
            ] {
                let _ = writeln!(out, "{line}");
            }
            let _ = writeln!(out, "operation {index} end");
        }
        out
    }

    #[expect(
        clippy::too_many_lines,
        reason = "One synthetic archive binds each of its four files before testing admission."
    )]
    fn synthetic_sponsored(mutation: usize) -> ImportResult<ValidatedSponsoredMaturityCorpus> {
        let source = maturity_variable_run_of_record().expect("historical bindings");
        let identity = source.evidence().identity();
        let branch = source.evidence().branch();
        let plan = if mutation >= 6 {
            crate::maturity_native::tests::sponsored_script_with_retries(
                identity.clone(),
                branch,
                false,
                true,
            )
        } else {
            crate::maturity_native::tests::sponsored_script_for(identity.clone(), branch)
        };
        let mut exchanges = plan.exchanges().to_vec();
        let (_, response) = exchanges
            .iter_mut()
            .find(|(step, _)| {
                step.case()
                    .step
                    .starts_with("sponsored-without-change:control:")
            })
            .expect("control");
        if mutation == 3 {
            let mut detail: serde_json::Value =
                serde_json::from_str(response.observed_detail.as_deref().expect("detail"))
                    .expect("JSON");
            detail["route"] = serde_json::json!("direct-block");
            detail["relay_allowed"] = serde_json::json!(false);
            detail["relay_detail"] = serde_json::json!("refused");
            response.observed_detail = Some(detail.to_string());
        }
        if mutation == 4 {
            response
                .mined_readback
                .as_mut()
                .expect("readback")
                .raw_transaction[0] ^= 1;
        }
        let mut payload = String::from("maturity-sponsored-evidence-schema 1\n");
        write_scripted_field(&mut payload, "deployment-id", identity.network_id());
        write_scripted_field(&mut payload, "genesis-id", identity.genesis_id());
        write_scripted_field(&mut payload, "branch-identifier", branch.identifier());
        writeln!(payload, "branch-checkpoint {}", branch.checkpoint()).expect("String write");
        let mut extra = String::new();
        write_scripted_field(
            &mut extra,
            "sponsored-policy",
            &serde_json::to_vec(&plan.policy().expect("policy").0).expect("JSON"),
        );
        let mut census = plan.census();
        if mutation == 2 {
            census[1] += 1;
        }
        writeln!(
            extra,
            "sponsored-census {}",
            census.map(|n| n.to_string()).join(" ")
        )
        .expect("String write");
        write_scripted_field(
            &mut extra,
            "sponsored-probes",
            &serde_json::to_vec(
                &exchanges
                    .iter()
                    .filter_map(|(_, response)| response.observed_detail.as_deref())
                    .collect::<Vec<_>>(),
            )
            .expect("probes"),
        );
        if mutation != 7 {
            write_scripted_field(
                &mut extra,
                "sponsored-waiting",
                &if mutation == 8 {
                    b"[]".to_vec()
                } else {
                    serde_json::to_vec(&plan.waiting_json()).expect("waiting")
                },
            );
        }
        payload.push_str(&extra);
        for (step, response) in &exchanges {
            write_scripted_field(
                &mut payload,
                "exchange",
                &serde_json::to_vec(&(step.case(), step.subject(), response)).expect("JSON"),
            );
        }
        let original = std::str::from_utf8(VARIABLE_FILES[0].bytes()).expect("capture");
        let mut capture = original
            .split_once("operation-count ")
            .expect("operations")
            .0
            .to_owned();
        capture.push_str(&scripted_operation_lines(&exchanges));
        write_scripted_field(&mut capture, "legacy-rendering", payload.as_bytes());
        capture.push_str("terminal-state complete\nrun-id-input end\n");
        let content_hash = hex_bytes(&tagged::sha256(capture.as_bytes()));
        writeln!(
            capture,
            "capture-content-sha256 {content_hash}\nnative-capture-end report"
        )
        .expect("String write");
        let capture_hash = hex_bytes(&tagged::sha256(capture.as_bytes()));
        let timing = VARIABLE_FILES[1].bytes();
        let manifest = format!(
            "{}  {}\n{}  {}\n",
            capture_hash,
            VARIABLE_FILES[0].name(),
            hex_bytes(&tagged::sha256(timing)),
            VARIABLE_FILES[1].name()
        );
        let manifest_hash = hex_bytes(&tagged::sha256(manifest.as_bytes()));
        let mut report = String::new();
        for line in std::str::from_utf8(VARIABLE_FILES[3].bytes())
            .expect("report")
            .lines()
        {
            if line == "ceremony-roster begin" {
                writeln!(
                    report,
                    "capture-profile sponsored\n{extra}ceremony-roster begin"
                )
                .expect("String write");
            } else if line.starts_with("ceremony report ") {
                writeln!(report, "ceremony report {capture_hash}").expect("String write");
            } else if line.starts_with("manifest-sha256 ") {
                writeln!(report, "manifest-sha256 {manifest_hash}").expect("String write");
            } else {
                writeln!(report, "{line}").expect("String write");
            }
        }
        let address = hex_bytes(&tagged::sha256(report.as_bytes()));
        if mutation == 5 {
            capture.replace_range(0..1, "x");
        }
        let inputs = [
            capture.as_bytes(),
            timing,
            manifest.as_bytes(),
            report.as_bytes(),
        ];
        let files =
            std::array::from_fn(|i| MaturityArchiveFile::new(VARIABLE_FILES[i].name(), inputs[i]));
        let floors = if mutation == 1 {
            MaturityDisclosedFeeFloors::declared("0.00000100", "0.00000100")
        } else {
            MaturityDisclosedFeeFloors::node_defaults("0.00000100", "0.00000100")
        };
        let pins = MaturityCorpusPins {
            sizes: inputs.map(<[u8]>::len),
            manifest_sha256: &manifest_hash,
            run_address: &address,
            ceremony_sha256: &capture_hash,
            fee_floors: floors,
            ..VARIABLE_PINS
        };
        admit_sponsored_with_replay(
            &files,
            &pins,
            crate::maturity_native::sponsored::replay_through_checkpoints,
        )
    }

    fn assert_recorded_sponsored_fees_and_reasons() {
        let recorded = maturity_sponsored_run_of_record().expect("pinned sponsored archive");
        assert_eq!(recorded.evidence.census(), [69, 35, 17, 17, 12, 5]);
        let fees: Vec<_> = recorded
            .evidence
            .exchanges()
            .iter()
            .filter_map(|(step, response)| {
                if response.observed_layer != ObservedOutcomeLayer::Accepted
                    || !step.case().step.contains(":control:")
                {
                    return None;
                }
                let OperationSubject::Submission(subject) = step.subject() else {
                    return None;
                };
                let transaction =
                    transaction::bytes::TargetTransaction::decode(&subject.transaction_bytes)
                        .expect("submitted transaction");
                transaction.outputs().iter().find_map(|output| {
                    if !output.program().is_empty() {
                        return None;
                    }
                    let transaction::bytes::ValueField::Explicit(value) = output.value() else {
                        panic!("explicit fee");
                    };
                    Some(value)
                })
            })
            .collect();
        assert_eq!(fees, [128, 141, 142]);
        for (step, response) in recorded.evidence.exchanges() {
            use crate::maturity_native::sponsored::{
                CASES, NEGATIVE_REASONS, NEGATIVES, check_reason,
            };
            let name = &step.case().step;
            let expected = if let Some(index) = NEGATIVES.iter().position(|row| name.contains(row))
            {
                if !matches!(step.subject(), OperationSubject::Submission(_)) {
                    continue;
                }
                Some((NEGATIVE_REASONS[index], true))
            } else if CASES[..9].iter().any(|row| name.starts_with(row))
                && name.contains(":control:")
            {
                Some(("min relay fee not met", false))
            } else if name.starts_with(CASES[12])
                && response.observed_layer == ObservedOutcomeLayer::ConsensusRejectionBeforeScript
            {
                Some(("bad-txns-in-ne-out", true))
            } else {
                None
            };
            if let Some((reason, refused)) = expected {
                assert_eq!(check_reason(response, reason, refused), Ok(()), "{name}");
                assert_eq!(
                    check_reason(response, "another boundary", refused),
                    Err(MaturityNativePlanRefusal::SponsoredBoundary)
                );
            }
        }
    }

    #[test]
    fn sponsored_admission_binds_bytes_provenance_census_routes_and_readback() {
        assert_recorded_sponsored_fees_and_reasons();
        synthetic_sponsored(0).expect("unmutated sponsored archive");
        let retried = synthetic_sponsored(6)
            .expect("undecided exchanges and fresh attempts remain admissible");
        assert_eq!(retried.evidence.waiting().len(), 1);
        assert_fresh_reprobes(&retried.evidence);
        for mutation in [7, 8] {
            assert_eq!(
                synthetic_sponsored(mutation).err(),
                Some(Refusal::CrossFileBinding {
                    field: "sponsored-derived-waiting",
                })
            );
        }
        for (mutation, expected) in [
            (
                1,
                Refusal::CrossFileBinding {
                    field: "sponsored-fee-floors",
                },
            ),
            (
                2,
                Refusal::CrossFileBinding {
                    field: "sponsored-derived-census",
                },
            ),
            (
                3,
                Refusal::Derivation(MaturityNativePlanRefusal::SponsoredRoute),
            ),
            (
                4,
                Refusal::Derivation(MaturityNativePlanRefusal::SubmissionReadbackMismatch),
            ),
        ] {
            assert_eq!(synthetic_sponsored(mutation).err(), Some(expected));
        }
        assert!(matches!(
            synthetic_sponsored(5),
            Err(Refusal::ManifestDigest { .. })
        ));
    }

    fn assert_fresh_reprobes(evidence: &crate::maturity_native::sponsored::Planner) {
        let mut previous = std::collections::HashMap::new();
        let mut undecided = 0;
        for (step, response) in evidence.exchanges() {
            if !step.case().step.contains(":control:") {
                continue;
            }
            let OperationSubject::Submission(subject) = step.subject() else {
                panic!("submission");
            };
            let transaction =
                transaction::bytes::TargetTransaction::decode(&subject.transaction_bytes)
                    .expect("transaction");
            let row = step.case().step.split(':').next().expect("row");
            let coin = transaction.inputs()[0].outpoint();
            if let Some(old) = previous.insert(row, coin) {
                assert_ne!(old, coin, "fresh funded outpoint");
            }
            undecided += usize::from(
                response.observed_layer == ObservedOutcomeLayer::ExecutorInfrastructureFailure,
            );
        }
        assert_eq!(undecided, 7);
        assert_eq!(evidence.census()[4..], [11, 5]);
        let credited = evidence.credited();
        let family: Vec<_> = credited
            .iter()
            .filter(|(step, _)| {
                step.case()
                    .step
                    .starts_with("sponsored-with-change-where-supported:")
            })
            .collect();
        assert_eq!(family.len(), 6);
        let control = family
            .iter()
            .find(|(step, _)| step.case().step.contains(":control:"))
            .expect("fresh control");
        let OperationSubject::Submission(control_subject) = control.0.subject() else {
            panic!("control");
        };
        let control_tx =
            transaction::bytes::TargetTransaction::decode(&control_subject.transaction_bytes)
                .expect("control transaction");
        let identity = |subject: &OperationSubject| {
            let OperationSubject::Submission(subject) = subject else {
                panic!("submission");
            };
            let tx = transaction::bytes::TargetTransaction::decode(&subject.transaction_bytes)
                .expect("transaction");
            tagged::sha256(&tagged::sha256(&tx.encode_without_witness()))
        };
        let fresh_ids: Vec<_> = family
            .iter()
            .map(|(step, _)| identity(step.subject()))
            .collect();
        for (step, _) in evidence.exchanges().iter().filter(|(step, _)| {
            step.case()
                .step
                .starts_with("sponsored-with-change-where-supported:")
                && matches!(step.subject(), OperationSubject::Submission(_))
                && !family.iter().any(|(fresh, _)| fresh.case() == step.case())
        }) {
            assert!(
                !fresh_ids.contains(&identity(step.subject())),
                "abandoned transaction credited"
            );
        }
        for (step, response) in family {
            let OperationSubject::Submission(subject) = step.subject() else {
                panic!("subject");
            };
            let tx = transaction::bytes::TargetTransaction::decode(&subject.transaction_bytes)
                .expect("transaction");
            assert_eq!(tx.inputs()[0].outpoint(), control_tx.inputs()[0].outpoint());
            assert_eq!(
                evidence
                    .exchanges()
                    .iter()
                    .filter(|(candidate, answer)| candidate == step && answer == response)
                    .count(),
                1
            );
        }
    }

    fn scripted_payload(
        exchanges: &[(OperationStep, NativeOperationResponse)],
        evidence: &MaturityNativeEvidence,
    ) -> String {
        let mut out = String::new();
        let _ = writeln!(out, "maturity-evidence-schema 1");
        write_scripted_field(
            &mut out,
            "deployment-network-id",
            evidence.identity().network_id(),
        );
        write_scripted_field(
            &mut out,
            "deployment-genesis-id",
            evidence.identity().genesis_id(),
        );
        write_scripted_field(
            &mut out,
            "branch-identifier",
            evidence.branch().identifier(),
        );
        let _ = writeln!(out, "branch-checkpoint {}", evidence.branch().checkpoint());
        let _ = writeln!(out, "current-root-freshness unestablished");
        for (step, response) in exchanges {
            let _ = writeln!(out, "maturity-step {}", step.case().step);
            write_scripted_field(
                &mut out,
                "validated-response",
                &serde_json::to_vec(response).expect("response JSON"),
            );
        }
        for observation in evidence.observations() {
            let _ = writeln!(
                out,
                "row {} declared {:?} observed {:?}",
                observation.subject(),
                observation.declared_layer(),
                observation.observed_layer()
            );
        }
        let _ = writeln!(out, "sponsorless-standing {:?}", evidence.standing());
        let MaturityAcceptanceObligation::Established {
            schedule,
            identity,
            readback,
        } = evidence.acceptance_obligation()
        else {
            panic!("variable scripted acceptance")
        };
        let _ = writeln!(out, "sponsorless-acceptance established");
        let _ = writeln!(out, "accepted-schedule {}", schedule.name());
        let _ = writeln!(out, "accepted-identity {}", identity.to_target_display());
        let _ = writeln!(
            out,
            "accepted-witness-identity {}",
            readback.witness_identity().to_target_display()
        );
        let _ = writeln!(
            out,
            "accepted-block {} {}",
            hex_bytes(readback.block_hash()),
            readback.block_height()
        );
        out
    }

    fn accepted_corpus() -> OwnedCorpus {
        let (exchanges, evidence) = scripted_variable_run();
        let operations = scripted_operation_lines(&exchanges);
        let payload = scripted_payload(&exchanges, &evidence);
        let historical = std::str::from_utf8(FILES[0].bytes).expect("historical capture");
        let prefix = historical
            .split_once("operation-count 3\n")
            .expect("operations")
            .0;
        let after_payload = historical
            .split_once("legacy-rendering ")
            .expect("rendering")
            .1;
        let tail = after_payload.split_once('\n').expect("tail").1;
        let mut capture = String::new();
        capture.push_str(prefix);
        capture.push_str(&operations);
        let _ = writeln!(
            capture,
            "legacy-rendering {} {}",
            payload.len(),
            hex_bytes(payload.as_bytes())
        );
        capture.push_str(tail);
        let mut owned = OwnedCorpus::new();
        owned.bytes[0] = capture.into_bytes();
        owned
    }

    fn flip_recorded_submission_request(corpus: &mut OwnedCorpus) {
        let text = std::str::from_utf8(&corpus.bytes[0]).expect("capture");
        let (prefix, after_begin) = text.split_once("operation 2 begin\n").expect("submission");
        let (block, suffix) = after_begin
            .split_once("operation 2 end\n")
            .expect("submission end");
        let mut changed = String::new();
        changed.push_str(prefix);
        changed.push_str("operation 2 begin\n");
        let mut changed_byte = None;
        for line in block.lines() {
            if let Some(value) = line.strip_prefix("request-bytes ") {
                let (_, hex) = value.split_once(' ').expect("request bytes");
                let mut bytes = decode_hex(hex).expect("request encoding");
                bytes[0] ^= 1;
                changed_byte = Some(bytes[0]);
                write_scripted_field(&mut changed, "request-bytes", &bytes);
            } else if let Some(value) = line.strip_prefix("request-subject ") {
                let (_, hex) = value.split_once(' ').expect("subject bytes");
                let mut subject: serde_json::Value =
                    serde_json::from_slice(&decode_hex(hex).expect("subject encoding"))
                        .expect("subject JSON");
                subject["transaction_bytes"][0] =
                    serde_json::Value::from(changed_byte.expect("request byte"));
                write_scripted_field(
                    &mut changed,
                    "request-subject",
                    &serde_json::to_vec(&subject).expect("subject JSON"),
                );
            } else {
                let _ = writeln!(changed, "{line}");
            }
        }
        changed.push_str("operation 2 end\n");
        changed.push_str(suffix);
        corpus.bytes[0] = changed.into_bytes();
    }

    #[test]
    fn historical_pins_and_public_admission_equal_the_record_loader() {
        assert_eq!(HISTORICAL_PINS.schedule(), MATURITY_RUN_SCHEDULE);
        assert_eq!(HISTORICAL_PINS.names(), &FILES.map(|file| file.name));
        assert_eq!(HISTORICAL_PINS.sizes(), &FILE_SIZES);
        assert_eq!(HISTORICAL_PINS.manifest_sha256(), MATURITY_MANIFEST_SHA256);
        assert_eq!(HISTORICAL_PINS.run_address(), MATURITY_RUN_ADDRESS);
        assert_eq!(HISTORICAL_PINS.ceremony_sha256(), CEREMONY_SHA256);
        assert_eq!(
            HISTORICAL_PINS.cargo_path(),
            "/workspace/toolchains/cargo/bin/cargo"
        );
        assert_eq!(
            HISTORICAL_PINS.framework_revision(),
            MaturityFrameworkRevision::Unrecorded
        );
        assert_eq!(HISTORICAL_PINS.funding_count(), 0);
        assert_eq!(
            HISTORICAL_PINS.sponsorless(),
            MaturitySponsorlessExpectation::RelayRefusal {
                detail: "bad-witness-nonstandard"
            }
        );
        assert_eq!(
            admit_maturity_corpus(&FILES, &HISTORICAL_PINS),
            Ok(maturity_run_of_record().expect("historical loader").clone())
        );
    }

    #[test]
    fn variable_archive_admits_established_identity_and_exact_mined_readback() {
        let corpus = maturity_variable_run_of_record().expect("accepted archive");
        assert_eq!(corpus.schedule(), MATURITY_VARIABLE_RUN_SCHEDULE);
        assert_eq!(corpus.exchanges().len(), 3);
        assert_eq!(
            corpus.evidence().standing(),
            MaturityNativeStanding::AnsweredAtDeclaredBoundary
        );
        let MaturityAcceptanceObligation::Established {
            schedule,
            identity,
            readback,
        } = corpus.evidence().acceptance_obligation()
        else {
            panic!("accepted archive obligation")
        };
        assert_eq!(*schedule, MATURITY_VARIABLE_RUN_SCHEDULE);
        assert_eq!(*identity, readback.identity());
        assert!(readback.block_height() >= 1);
        let OperationSubject::Submission(subject) = corpus.exchanges()[2].0.subject() else {
            panic!("submission subject")
        };
        assert_eq!(readback.bytes(), subject.transaction_bytes.as_slice());
        let capture = std::str::from_utf8(VARIABLE_FILES[0].bytes()).expect("capture text");
        let response_identity = capture
            .lines()
            .rfind(|line| line.starts_with("response-target-identity "))
            .expect("submission response identity")
            .strip_prefix("response-target-identity ")
            .expect("response field");
        assert_eq!(identity.to_target_display(), response_identity);
        let rendered_hex = capture
            .lines()
            .find_map(|line| line.strip_prefix("legacy-rendering "))
            .expect("payload")
            .split_whitespace()
            .nth(1)
            .expect("payload hex");
        let rendered = decode_hex(rendered_hex).expect("payload bytes");
        let rendered = std::str::from_utf8(&rendered).expect("payload text");
        assert!(rendered.contains(&format!("accepted-identity {response_identity}\n")));
        assert_eq!(
            replay_maturity_variable_run_of_record(MaturityWitnessSelection::Retained(
                MATURITY_VARIABLE_RUN_SCHEDULE
            )),
            Ok(corpus.evidence().clone())
        );
    }

    #[test]
    fn variable_pins_equal_the_embedded_file_facts() {
        assert_eq!(
            VARIABLE_PINS.names(),
            &VARIABLE_FILES.map(|file| file.name())
        );
        assert_eq!(
            VARIABLE_PINS.sizes(),
            &VARIABLE_FILES.map(|file| file.bytes().len())
        );
        assert_eq!(
            VARIABLE_PINS.ceremony_sha256(),
            hex_bytes(&tagged::sha256(VARIABLE_FILES[0].bytes()))
        );
        assert_eq!(
            VARIABLE_PINS.manifest_sha256(),
            hex_bytes(&tagged::sha256(VARIABLE_FILES[2].bytes()))
        );
        assert_eq!(
            VARIABLE_PINS.run_address(),
            hex_bytes(&tagged::sha256(VARIABLE_FILES[3].bytes()))
        );
        assert_eq!(
            VARIABLE_PINS.sponsorless(),
            MaturitySponsorlessExpectation::Acceptance
        );
        let report = std::str::from_utf8(VARIABLE_FILES[3].bytes()).expect("report text");
        let argv_hex = report
            .lines()
            .find_map(|line| line.strip_prefix("cargo-argv "))
            .expect("cargo argv")
            .split_whitespace()
            .nth(1)
            .expect("cargo hex");
        let argv = decode_hex(argv_hex).expect("cargo bytes");
        assert_eq!(
            std::str::from_utf8(&argv).expect("cargo text"),
            format!(
                "{} test -p tripod-vectors --test {} -- --ignored --test-threads=1",
                VARIABLE_PINS.cargo_path(),
                VARIABLE_PINS.capture_target().target()
            )
        );
        let capture = std::str::from_utf8(VARIABLE_FILES[0].bytes()).expect("capture text");
        let framework_hex = capture
            .lines()
            .find_map(|line| line.strip_prefix("handshake-framework-revision 40 "))
            .expect("recorded framework tip");
        let framework = decode_hex(framework_hex).expect("framework bytes");
        assert_eq!(
            std::str::from_utf8(&framework).expect("framework text"),
            maturity_variable_run_of_record()
                .expect("accepted archive")
                .report()
                .intended_tip()
        );
        assert_eq!(
            VARIABLE_PINS.framework_revision(),
            MaturityFrameworkRevision::IntendedTip
        );
        let funding_count = capture
            .lines()
            .find_map(|line| line.strip_prefix("environment-funding-count "))
            .expect("funding count")
            .parse::<usize>()
            .expect("funding number");
        assert_eq!(VARIABLE_PINS.funding_count(), funding_count);
    }

    #[test]
    fn variable_files_under_historical_pins_refuse_corpus_census() {
        assert_eq!(
            admit_maturity_corpus(&VARIABLE_FILES, &HISTORICAL_PINS),
            Err(Refusal::CorpusCensus)
        );
    }

    #[test]
    fn historical_files_under_variable_pins_refuse_corpus_census() {
        assert_eq!(
            admit_maturity_corpus(&FILES, &VARIABLE_PINS),
            Err(Refusal::CorpusCensus)
        );
    }

    #[test]
    fn variable_replay_under_whole_schedule_refuses_transcript_step_mismatch_zero() {
        assert_eq!(
            replay_maturity_variable_run_of_record(MaturityWitnessSelection::Retained(
                tapscript::StateWitnessSchedule::WholeMetadata
            )),
            Err(Refusal::Derivation(
                MaturityNativePlanRefusal::TranscriptStepMismatch { position: 0 }
            ))
        );
    }

    #[test]
    fn wrong_cargo_path_pin_refuses_run_report_grammar() {
        let pins = MaturityCorpusPins {
            cargo_path: "/another/cargo",
            ..VARIABLE_PINS
        };
        assert_eq!(
            admit_maturity_corpus(&VARIABLE_FILES, &pins),
            Err(Refusal::RunReportGrammar)
        );
    }

    #[test]
    fn unrecorded_framework_pin_refuses_cross_file_binding() {
        let pins = MaturityCorpusPins {
            framework_revision: MaturityFrameworkRevision::Unrecorded,
            ..VARIABLE_PINS
        };
        assert_eq!(
            admit_maturity_corpus(&VARIABLE_FILES, &pins),
            Err(Refusal::CrossFileBinding {
                field: "handshake-framework-revision"
            })
        );
    }

    #[test]
    fn wrong_funding_count_pin_refuses_transcript_grammar() {
        let pins = MaturityCorpusPins {
            funding_count: VARIABLE_PINS.funding_count() + 1,
            ..VARIABLE_PINS
        };
        assert!(matches!(
            admit_maturity_corpus(&VARIABLE_FILES, &pins),
            Err(Refusal::TranscriptGrammar { .. })
        ));
    }

    #[test]
    fn scripted_variable_corpus_admits_established_identity_and_readback() {
        let mut files = accepted_corpus();
        let admitted = files
            .rebind_for(
                true,
                tapscript::StateWitnessSchedule::VariableMetadata,
                MaturitySponsorlessExpectation::Acceptance,
            )
            .expect("accepted corpus");
        let (_, scripted) = scripted_variable_run();
        assert_eq!(admitted.evidence(), &scripted);
        assert_eq!(
            admitted.schedule(),
            tapscript::StateWitnessSchedule::VariableMetadata
        );
        assert_eq!(admitted.recorded_refusal_detail(), "");
        let MaturityAcceptanceObligation::Established {
            identity, readback, ..
        } = admitted.evidence().acceptance_obligation()
        else {
            panic!("accepted corpus obligation")
        };
        assert_eq!(*identity, readback.identity());
        let OperationSubject::Submission(subject) = admitted.exchanges()[2].0.subject() else {
            panic!("submission bytes")
        };
        assert_eq!(readback.bytes(), subject.transaction_bytes.as_slice());
    }

    #[test]
    fn accepted_files_with_whole_schedule_refuse_transcript_step_mismatch_zero() {
        let mut files = accepted_corpus();
        assert_eq!(
            files.rebind_for(
                true,
                tapscript::StateWitnessSchedule::WholeMetadata,
                MaturitySponsorlessExpectation::Acceptance
            ),
            Err(Refusal::Derivation(
                MaturityNativePlanRefusal::TranscriptStepMismatch { position: 0 }
            ))
        );
    }

    #[test]
    fn historical_files_under_acceptance_refuse_transcript_grammar() {
        let mut files = OwnedCorpus::new();
        assert!(matches!(
            files.rebind_for(
                true,
                MATURITY_RUN_SCHEDULE,
                MaturitySponsorlessExpectation::Acceptance
            ),
            Err(Refusal::TranscriptGrammar { .. })
        ));
    }

    #[test]
    fn accepted_files_under_relay_refusal_pins_refuse_transcript_grammar() {
        let mut files = accepted_corpus();
        assert!(matches!(
            files.rebind_for(
                true,
                tapscript::StateWitnessSchedule::VariableMetadata,
                MaturitySponsorlessExpectation::RelayRefusal {
                    detail: "bad-witness-nonstandard"
                }
            ),
            Err(Refusal::TranscriptGrammar { .. })
        ));
    }

    #[test]
    fn wrong_capture_and_payload_identity_refuse_derivation_submission_readback_mismatch() {
        let mut files = accepted_corpus();
        let (_, evidence) = scripted_variable_run();
        let MaturityAcceptanceObligation::Established { identity, .. } =
            evidence.acceptance_obligation()
        else {
            panic!("identity")
        };
        let original = identity.to_target_display();
        let wrong = "99".repeat(32);
        files.replace(
            0,
            &format!("response-target-identity {original}"),
            &format!("response-target-identity {wrong}"),
        );
        files.payload_replace(
            &format!("accepted-identity {original}"),
            &format!("accepted-identity {wrong}"),
        );
        assert_eq!(
            files.rebind_for(
                true,
                tapscript::StateWitnessSchedule::VariableMetadata,
                MaturitySponsorlessExpectation::Acceptance
            ),
            Err(Refusal::Derivation(
                MaturityNativePlanRefusal::SubmissionReadbackMismatch
            ))
        );
    }

    #[test]
    fn wrong_payload_witness_identity_refuses_derivation_submission_readback_mismatch() {
        let mut files = accepted_corpus();
        let (_, evidence) = scripted_variable_run();
        let MaturityAcceptanceObligation::Established { readback, .. } =
            evidence.acceptance_obligation()
        else {
            panic!("readback")
        };
        let original = readback.witness_identity().to_target_display();
        files.payload_replace(
            &format!("accepted-witness-identity {original}"),
            &format!("accepted-witness-identity {}", "99".repeat(32)),
        );
        assert_eq!(
            files.rebind_for(
                true,
                tapscript::StateWitnessSchedule::VariableMetadata,
                MaturitySponsorlessExpectation::Acceptance
            ),
            Err(Refusal::Derivation(
                MaturityNativePlanRefusal::SubmissionReadbackMismatch
            ))
        );
    }

    #[test]
    fn payload_identity_different_from_capture_refuses_cross_file_binding() {
        let mut files = accepted_corpus();
        let (_, evidence) = scripted_variable_run();
        let MaturityAcceptanceObligation::Established { identity, .. } =
            evidence.acceptance_obligation()
        else {
            panic!("identity")
        };
        files.payload_replace(
            &format!("accepted-identity {}", identity.to_target_display()),
            &format!("accepted-identity {}", "99".repeat(32)),
        );
        assert_eq!(
            files.rebind_for(
                true,
                tapscript::StateWitnessSchedule::VariableMetadata,
                MaturitySponsorlessExpectation::Acceptance
            ),
            Err(Refusal::CrossFileBinding {
                field: "accepted-identity"
            })
        );
    }

    #[test]
    fn flipped_recorded_request_refuses_derivation_transcript_step_mismatch_two() {
        let mut files = accepted_corpus();
        flip_recorded_submission_request(&mut files);
        assert_eq!(
            files.rebind_for(
                true,
                tapscript::StateWitnessSchedule::VariableMetadata,
                MaturitySponsorlessExpectation::Acceptance
            ),
            Err(Refusal::Derivation(
                MaturityNativePlanRefusal::TranscriptStepMismatch { position: 2 }
            ))
        );
    }

    #[test]
    fn incomplete_and_refused_identity_or_established_payload_refuse_transcript_grammar() {
        let mut incomplete = accepted_corpus();
        incomplete.replace(0, "operation-count 3", "operation-count 2");
        assert!(matches!(
            incomplete.rebind_for(
                true,
                tapscript::StateWitnessSchedule::VariableMetadata,
                MaturitySponsorlessExpectation::Acceptance
            ),
            Err(Refusal::TranscriptGrammar { .. })
        ));
        let mut refused_identity = OwnedCorpus::new();
        refused_identity.replace(
            0,
            "response-layer relay-policy-rejection\nresponse-target-identity none",
            &format!(
                "response-layer relay-policy-rejection\nresponse-target-identity {}",
                "99".repeat(32)
            ),
        );
        assert!(matches!(
            refused_identity.rebind(true),
            Err(Refusal::TranscriptGrammar { .. })
        ));
        let mut refused_established = OwnedCorpus::new();
        refused_established.payload_replace(
            "sponsorless-acceptance outstanding",
            "sponsorless-acceptance established",
        );
        assert!(matches!(
            refused_established.rebind(true),
            Err(Refusal::TranscriptGrammar { .. })
        ));
    }

    #[test]
    fn the_admitted_bytes_derive_the_boundary_and_leave_acceptance_outstanding() {
        let corpus = maturity_run_of_record().expect("admitted corpus");
        let report = corpus.report();
        assert_eq!(corpus.content_address(), MATURITY_RUN_ADDRESS);
        assert_eq!(report.run_address(), MATURITY_RUN_ADDRESS);
        assert_eq!(report.manifest_digest(), MATURITY_MANIFEST_SHA256);
        assert_eq!(report.ceremony_digest(), CEREMONY_SHA256);
        assert_eq!(report.suite_commit(), corpus.capture_suite_commit());
        assert_eq!(report.suite_tree(), corpus.capture_suite_tree());
        assert_eq!(report.expected_tip(), report.intended_tip());
        assert!(report.expected_tip().starts_with(report.binary_revision()));
        assert_eq!(report.adapter_name(), "elements-native-executor");
        assert_eq!(report.adapter_version(), "2.1.0");
        assert_eq!(report.node_name(), "Elements");
        assert_eq!(
            report.node_version(),
            format!(
                "Elements Core daemon version v28.99.0-{}",
                report.binary_revision()
            )
        );
        assert_eq!(report.protocol_revision(), 8);
        assert_eq!(report.environment(), "development");
        assert_eq!(report.target_contract(), "elements-tapscript-v2");
        let operator =
            crate::live_corpus_native_operator::run_of_record().expect("operator corpus");
        assert_eq!(report.network_id(), operator.report().network_id());
        assert_eq!(report.genesis_id(), operator.report().genesis_id());
        assert_eq!(corpus.capabilities(), CAPABILITIES);
        assert_eq!(corpus.recorded_refusal_detail(), "bad-witness-nonstandard");
        let evidence = corpus.evidence();
        assert_eq!(
            evidence.identity().network_id(),
            &digest(report.network_id()).expect("network")
        );
        assert_eq!(
            evidence.identity().genesis_id(),
            &digest(report.genesis_id()).expect("genesis")
        );
        assert_eq!(evidence.branch().identifier(), &[0x41; 32]);
        assert_eq!(evidence.branch().checkpoint(), 7);
        assert_eq!(
            evidence.standing(),
            MaturityNativeStanding::AnsweredAtDeclaredBoundary
        );
        assert_eq!(
            evidence.acceptance_obligation(),
            &MaturityAcceptanceObligation::Outstanding {
                routes: [
                    MaturityAcceptanceRoute::RelayWitnessRestructure,
                    MaturityAcceptanceRoute::BlockLayerSubmissionSubject
                ],
            }
        );
        assert_eq!(corpus.exchanges().len(), 3);
        for ((step, response), observation) in
            corpus.exchanges().iter().zip(evidence.observations())
        {
            assert_eq!(&response.case, step.case());
            assert_eq!(step.case().step, observation.subject());
            assert_eq!(response.observed_layer, observation.observed_layer());
        }
    }

    #[test]
    fn every_file_byte_flip_stops_at_its_outer_layer() {
        for index in 0..4 {
            let mut corpus = OwnedCorpus::new();
            corpus.bytes[index][0] ^= 1;
            let expected = match index {
                0 | 1 => Refusal::ManifestDigest {
                    name: FILES[index].name.to_owned(),
                },
                2 => Refusal::ManifestHash,
                3 => Refusal::RunReportGrammar,
                _ => unreachable!(),
            };
            assert_eq!(corpus.pinned(), Err(expected));
        }
    }

    #[test]
    fn census_and_every_file_length_are_fixed() {
        assert_eq!(
            validate_census(&FILES[..3], &HISTORICAL_PINS),
            Err(Refusal::CorpusCensus)
        );
        let mut files = FILES;
        files.swap(0, 1);
        assert_eq!(
            validate_census(&files, &HISTORICAL_PINS),
            Err(Refusal::CorpusCensus)
        );
        for (index, expected) in FILE_SIZES.iter().enumerate() {
            let mut corpus = OwnedCorpus::new();
            corpus.bytes[index].pop();
            assert_eq!(
                corpus.pinned(),
                Err(Refusal::FileSize {
                    name: FILES[index].name.to_owned(),
                    expected: *expected,
                    actual: expected - 1,
                })
            );
        }
    }

    #[test]
    fn manifest_lines_are_closed_sorted_and_digest_checked() {
        let manifest = std::str::from_utf8(FILES[2].bytes).expect("manifest");
        let lines: Vec<_> = manifest.lines().collect();
        for malformed in [
            format!("{manifest}extra\n"),
            format!("{}\n{}\n", lines[1], lines[0]),
            format!("{}\n", lines[0]),
            manifest.replace("  ", " "),
            manifest.replace(".timing", ".other"),
        ] {
            let mut corpus = OwnedCorpus::new();
            corpus.bytes[2] = malformed.into_bytes();
            assert_eq!(
                validate_manifest(
                    &corpus.inputs(),
                    &hex_bytes(&tagged::sha256(&corpus.bytes[2]))
                ),
                Err(Refusal::ManifestGrammar)
            );
        }
        let mut corpus = OwnedCorpus::new();
        corpus.bytes[2][0] = b'0';
        assert_eq!(
            validate_manifest(
                &corpus.inputs(),
                &hex_bytes(&tagged::sha256(&corpus.bytes[2]))
            ),
            Err(Refusal::ManifestDigest {
                name: FILES[0].name.to_owned()
            })
        );
    }

    #[test]
    fn report_grammar_closes_counts_eligibility_order_and_provenance() {
        for (from, to) in [
            ("suite-clean yes", "suite-clean no"),
            ("observed-test-count 1", "observed-test-count 2"),
            ("observed-ceremony-count 1", "observed-ceremony-count 0"),
            ("observed-setup-count 0", "observed-setup-count 1"),
            ("diagnostics-present yes", "diagnostics-present no"),
            ("test-failed-count 0", "test-failed-count 1"),
            ("test-ignored-count 0", "test-ignored-count 1"),
            ("cargo-exit-code 0", "cargo-exit-code 1"),
            ("protocol-revision 8", "protocol-revision 7"),
            ("eligible yes", "eligible no"),
            (
                "expected-test-count 1\nobserved-test-count 1",
                "observed-test-count 1\nexpected-test-count 1",
            ),
        ] {
            let mut corpus = OwnedCorpus::new();
            corpus.replace(3, from, to);
            assert_eq!(
                parse_report(&corpus.bytes[3], &HISTORICAL_PINS),
                Err(Refusal::RunReportGrammar)
            );
        }
        let mut corpus = OwnedCorpus::new();
        corpus.bytes[3].extend_from_slice(b"extra field\n");
        assert_eq!(
            parse_report(&corpus.bytes[3], &HISTORICAL_PINS),
            Err(Refusal::RunReportGrammar)
        );
    }

    #[test]
    fn a_grammatical_report_still_needs_its_pinned_address() {
        let mut corpus = OwnedCorpus::new();
        let text = std::str::from_utf8(&corpus.bytes[3]).expect("report");
        let start = text.find("suite-tree ").expect("tree") + "suite-tree ".len();
        corpus.bytes[3][start] = b'2';
        assert_eq!(corpus.pinned(), Err(Refusal::RunReportAddress));
    }

    #[test]
    fn outer_rebinding_does_not_replace_the_capture_content_hash() {
        let mut corpus = OwnedCorpus::new();
        corpus.payload_replace(
            "sponsorless-acceptance outstanding",
            "sponsorless-acceptance accepted",
        );
        assert_eq!(corpus.rebind(false), Err(Refusal::CaptureContentHash));
    }

    #[test]
    fn report_and_capture_bind_each_source_identity() {
        for field in ["suite-commit", "suite-tree"] {
            let mut corpus = OwnedCorpus::new();
            let text = std::str::from_utf8(&corpus.bytes[0]).expect("capture");
            let start = text.find(&format!("{field} ")).expect("field") + field.len() + 1;
            corpus.bytes[0][start] = b'0';
            assert_eq!(
                corpus.rebind(true),
                Err(Refusal::CrossFileBinding { field })
            );
        }
    }

    #[test]
    fn handshake_and_environment_bind_the_report() {
        for (from, to, field) in [
            (
                "handshake-protocol-schema 8",
                "handshake-protocol-schema 7",
                "protocol-revision",
            ),
            (
                "deployment-environment development",
                "deployment-environment other",
                "deployment-environment",
            ),
            (
                "handshake-framework-revision 0 ",
                "handshake-framework-revision 1 61",
                "handshake-framework-revision",
            ),
            (
                "response-request-id 9 726571756573742d30",
                "response-request-id 9 726571756573742d31",
                "response-request-id",
            ),
        ] {
            let mut corpus = OwnedCorpus::new();
            corpus.replace(0, from, to);
            assert_eq!(
                corpus.rebind(true),
                Err(Refusal::CrossFileBinding { field })
            );
        }
    }

    #[test]
    fn the_capture_closes_schema_rosters_roles_mutations_and_terminal_state() {
        for (from, to) in [
            ("native-capture-schema 2", "native-capture-schema 1"),
            (
                "environment-capability-count 12",
                "environment-capability-count 11",
            ),
            (
                "environment-capability 0 failure-class-reporting",
                "environment-capability 0 unknown",
            ),
            ("environment-funding-count 0", "environment-funding-count 1"),
            ("operation-count 3", "operation-count 2"),
            ("request-role auxiliary", "request-role control"),
            ("response-verdict refused", "response-verdict accepted"),
            (
                "response-layer relay-policy-rejection",
                "response-layer script-path-rejection",
            ),
            ("mutation-kind none", "mutation-kind witness"),
            ("mutation-locator none", "mutation-locator 0"),
            ("terminal-state complete", "terminal-state incomplete"),
            ("native-capture-end report", "native-capture-end other"),
        ] {
            let mut corpus = OwnedCorpus::new();
            corpus.replace(0, from, to);
            assert!(
                matches!(corpus.rebind(true), Err(Refusal::TranscriptGrammar { .. })),
                "{from}"
            );
        }
    }

    #[test]
    fn byte_fields_and_line_endings_have_one_spelling() {
        for bytes in [
            b"field 01 00\n".as_slice(),
            b"field 1 FF\n",
            b"field 2 00\n",
            b"field 1 0g\n",
        ] {
            let mut cursor = Cursor::new("test", bytes).expect("lines");
            assert!(matches!(
                cursor.bytes("field"),
                Err(Refusal::TranscriptGrammar { .. })
            ));
        }
        for bytes in [
            b"field 0 \r\n".as_slice(),
            b"field 0 ",
            b"field 0 \n\n",
            b"\xff\n",
        ] {
            assert!(matches!(
                Cursor::new("test", bytes),
                Err(Refusal::TranscriptGrammar { .. })
            ));
        }
    }

    #[test]
    fn timing_is_a_closed_passed_record() {
        let mut corpus = OwnedCorpus::new();
        corpus.replace(1, "status passed", "status failed");
        assert!(
            matches!(corpus.rebind(true), Err(Refusal::TranscriptGrammar { name, .. }) if name == "maturity timing")
        );
    }

    #[test]
    fn recorded_request_subjects_match_the_protocol_serializer_exactly() {
        let corpus = maturity_run_of_record().expect("corpus");
        let text = std::str::from_utf8(FILES[0].bytes).expect("capture");
        for (line, (step, _)) in text
            .lines()
            .filter(|line| line.starts_with("request-subject "))
            .zip(corpus.exchanges())
        {
            let bytes = decode_hex(line.split(' ').nth(2).expect("hex")).expect("bytes");
            assert_eq!(
                bytes,
                serde_json::to_vec(step.subject()).expect("serialize")
            );
        }
        for malformed in [
            "{\"transaction_bytes\":[256]}",
            "{\"transaction_bytes\":[01]}",
            "{\"transaction_bytes\":[1],\"extra\":0}",
            "{\"transaction_bytes\":[1],\"transaction_bytes\":[2]}",
        ] {
            assert_eq!(parse_subject(malformed, 2), None);
        }
    }

    #[test]
    fn payload_context_and_step_census_are_closed() {
        for (from, to) in [
            ("branch-identifier 32 41", "branch-identifier 32 42"),
            ("branch-checkpoint 7", "branch-checkpoint 8"),
            (
                "current-root-freshness unestablished",
                "current-root-freshness established",
            ),
            (
                "maturity-step issue-maturity-singleton",
                "maturity-step fund-maturity-predecessor",
            ),
        ] {
            let mut corpus = OwnedCorpus::new();
            corpus.payload_replace(from, to);
            assert!(matches!(
                corpus.rebind(true),
                Err(Refusal::TranscriptGrammar { .. })
            ));
        }
        let mut corpus = OwnedCorpus::new();
        let network = maturity_run_of_record()
            .expect("corpus")
            .report()
            .network_id();
        corpus.payload_replace(
            &format!("deployment-network-id 32 {network}"),
            &format!("deployment-network-id 32 {}", "00".repeat(32)),
        );
        assert_eq!(
            corpus.rebind(true),
            Err(Refusal::CrossFileBinding {
                field: "payload-network-id"
            })
        );
    }

    #[test]
    fn every_payload_row_and_claim_is_compared_with_replay() {
        for (from, to, field) in [
            (
                "row issue-maturity-singleton declared None",
                "row issue-maturity-singleton declared Some(Accepted)",
                "issue-maturity-singleton",
            ),
            (
                "row fund-maturity-predecessor declared None",
                "row fund-maturity-predecessor declared Some(Accepted)",
                "fund-maturity-predecessor",
            ),
            (
                "row sponsorless declared Some(RelayPolicyRejection)",
                "row sponsorless declared Some(Accepted)",
                "sponsorless",
            ),
            (
                "sponsorless-standing AnsweredAtDeclaredBoundary",
                "sponsorless-standing ObservedElsewhere",
                "sponsorless-standing",
            ),
            (
                "sponsorless-acceptance outstanding",
                "sponsorless-acceptance accepted",
                "sponsorless-acceptance",
            ),
            (
                "acceptance-route RelayWitnessRestructure",
                "acceptance-route BlockLayerSubmissionSubject",
                "acceptance-routes",
            ),
            (
                "acceptance-route BlockLayerSubmissionSubject",
                "acceptance-route RelaxedPolicy",
                "acceptance-routes",
            ),
        ] {
            let mut corpus = OwnedCorpus::new();
            corpus.payload_replace(from, to);
            assert_eq!(
                corpus.rebind(true),
                Err(Refusal::PayloadDisagreement { field })
            );
        }
    }

    fn payload() -> Payload {
        let corpus = maturity_run_of_record().expect("corpus");
        Payload {
            identity: corpus.evidence().identity().clone(),
            branch: corpus.evidence().branch(),
            exchanges: corpus.exchanges().to_vec(),
            rows: Vec::new(),
            standing: String::new(),
            acceptance: PayloadAcceptance::Outstanding {
                claim: String::new(),
                routes: [String::new(), String::new()],
            },
        }
    }

    #[test]
    fn replay_preserves_each_exact_request_mismatch_and_incomplete_prefix() {
        for position in 0..3 {
            let mut payload = payload();
            let (step, _) = &mut payload.exchanges[position];
            let mut subject = step.subject().clone();
            match &mut subject {
                OperationSubject::Funding(funding) => funding.amount_per_output += 1,
                OperationSubject::Submission(submission) => submission.transaction_bytes[0] ^= 1,
                _ => panic!("only recorded shapes"),
            }
            *step = OperationStep::new(&step.case().step, subject);
            assert_eq!(
                derive(&payload, MATURITY_RUN_SCHEDULE),
                Err(Refusal::Derivation(
                    MaturityNativePlanRefusal::TranscriptStepMismatch { position }
                ))
            );
            let mut payload = self::payload();
            payload.exchanges.truncate(position);
            assert_eq!(
                derive(&payload, MATURITY_RUN_SCHEDULE),
                Err(Refusal::Derivation(
                    MaturityNativePlanRefusal::IncompleteTranscript
                ))
            );
        }
    }

    #[test]
    fn replay_preserves_response_refusals_whole() {
        let mut payload = payload();
        payload.exchanges[0].1.funded_outputs[0].amount_satoshis += 1;
        assert_eq!(
            derive(&payload, MATURITY_RUN_SCHEDULE),
            Err(Refusal::Derivation(
                MaturityNativePlanRefusal::FundingMismatch
            ))
        );
        let mut payload = self::payload();
        payload.exchanges[0].1.schema = 7;
        assert_eq!(
            derive(&payload, MATURITY_RUN_SCHEDULE),
            Err(Refusal::Derivation(
                MaturityNativePlanRefusal::ResponseSchema { offered: 7 }
            ))
        );
    }

    #[test]
    fn admission_passes_recorded_request_changes_to_exact_replay() {
        let mut corpus = OwnedCorpus::new();
        let text = std::str::from_utf8(&corpus.bytes[0]).expect("capture");
        let line = text
            .lines()
            .find(|line| line.starts_with("request-subject "))
            .expect("subject")
            .to_owned();
        let decoded = decode_hex(line.split(' ').nth(2).expect("hex")).expect("bytes");
        let subject = String::from_utf8(decoded)
            .expect("JSON")
            .replace("\"amount_per_output\":1", "\"amount_per_output\":2");
        corpus.replace(
            0,
            &line,
            &format!(
                "request-subject {} {}",
                subject.len(),
                hex_bytes(subject.as_bytes())
            ),
        );
        assert_eq!(
            corpus.rebind(true),
            Err(Refusal::Derivation(
                MaturityNativePlanRefusal::TranscriptStepMismatch { position: 0 }
            ))
        );
    }

    #[test]
    fn admission_preserves_the_protocol_revision_refusal_from_replay() {
        let mut corpus = OwnedCorpus::new();
        corpus.payload_replace("7b22736368656d61223a38", "7b22736368656d61223a37");
        assert_eq!(
            corpus.rebind(true),
            Err(Refusal::Derivation(
                MaturityNativePlanRefusal::ResponseSchema { offered: 7 }
            ))
        );
    }

    struct OwnedMutantCorpus {
        bytes: [Vec<u8>; 4],
        pins: MaturityMutantCorpusPins<'static>,
    }

    impl OwnedMutantCorpus {
        fn new(
            files: &[MaturityArchiveFile<'static>; 4],
            pins: &MaturityMutantCorpusPins<'static>,
        ) -> Self {
            Self {
                bytes: files.map(|file| file.bytes().to_vec()),
                pins: *pins,
            }
        }

        fn inputs(&self) -> [MaturityArchiveFile<'_>; 4] {
            std::array::from_fn(|index| {
                MaturityArchiveFile::new(self.pins.pins().names()[index], &self.bytes[index])
            })
        }

        fn pinned(&self) -> ImportResult<ValidatedMaturityMutantCorpus> {
            admit_maturity_mutant_corpus(&self.inputs(), &self.pins)
        }

        fn replace(&mut self, index: usize, from: &str, to: &str) {
            let text = std::str::from_utf8(&self.bytes[index]).expect("mutant fixture text");
            assert!(text.contains(from), "missing mutant mutation operand");
            self.bytes[index] = text.replacen(from, to, 1).into_bytes();
        }

        fn payload_replace(&mut self, from: &str, to: &str) {
            let text = std::str::from_utf8(&self.bytes[0]).expect("mutant capture text");
            let line = text
                .lines()
                .find(|line| line.starts_with("legacy-rendering "))
                .expect("mutant payload line")
                .to_owned();
            let hex = line.split(' ').nth(2).expect("mutant payload hex");
            let payload = String::from_utf8(decode_hex(hex).expect("mutant payload bytes"))
                .expect("mutant payload text");
            assert!(payload.contains(from), "missing mutant payload operand");
            let changed = payload.replacen(from, to, 1);
            self.replace(
                0,
                &line,
                &format!(
                    "legacy-rendering {} {}",
                    changed.len(),
                    hex_bytes(changed.as_bytes())
                ),
            );
        }

        fn rebind(&mut self, content: bool) -> ImportResult<ValidatedMaturityMutantCorpus> {
            if content {
                let text = std::str::from_utf8(&self.bytes[0]).expect("mutant capture");
                let position = text
                    .find("capture-content-sha256 ")
                    .expect("capture content hash");
                let hash = hex_bytes(&tagged::sha256(&self.bytes[0][..position]));
                let start = position + "capture-content-sha256 ".len();
                self.bytes[0][start..start + 64].copy_from_slice(hash.as_bytes());
            }
            let capture_hash = hex_bytes(&tagged::sha256(&self.bytes[0]));
            let mut manifest = String::new();
            for index in 0..2 {
                let _ = writeln!(
                    manifest,
                    "{}  {}",
                    hex_bytes(&tagged::sha256(&self.bytes[index])),
                    self.pins.pins().names()[index]
                );
            }
            self.bytes[2] = manifest.into_bytes();
            let manifest_hash = hex_bytes(&tagged::sha256(&self.bytes[2]));
            let report = std::str::from_utf8(&self.bytes[3]).expect("mutant report");
            let mut rebound = String::new();
            for line in report.lines() {
                if line.starts_with("ceremony report ") {
                    let _ = writeln!(rebound, "ceremony report {capture_hash}");
                } else if line.starts_with("manifest-sha256 ") {
                    let _ = writeln!(rebound, "manifest-sha256 {manifest_hash}");
                } else {
                    let _ = writeln!(rebound, "{line}");
                }
            }
            self.bytes[3] = rebound.into_bytes();
            let address = hex_bytes(&tagged::sha256(&self.bytes[3]));
            let sizes = std::array::from_fn(|index| self.bytes[index].len());
            let original = self.pins.pins();
            let pins = MaturityMutantCorpusPins::new(
                MaturityCorpusPins::new(
                    original.schedule(),
                    (*original.names(), sizes),
                    (&manifest_hash, &address, &capture_hash),
                    original.sponsorless(),
                    original.capture_target(),
                    original.cargo_path(),
                    original.framework_revision(),
                    original.funding_count(),
                    original.fee_floors(),
                ),
                self.pins.table(),
            );
            admit_maturity_mutant_corpus(&self.inputs(), &pins)
        }
    }

    fn flip_mutant_request(corpus: &mut OwnedMutantCorpus, position: usize) {
        let text = std::str::from_utf8(&corpus.bytes[0]).expect("mutant capture");
        let begin = format!("operation {position} begin\n");
        let end = format!("operation {position} end\n");
        let (prefix, after_begin) = text.split_once(&begin).expect("mutant operation");
        let (block, suffix) = after_begin.split_once(&end).expect("mutant operation end");
        let mut changed = String::new();
        changed.push_str(prefix);
        changed.push_str(&begin);
        let mut changed_byte = None;
        for line in block.lines() {
            if let Some(value) = line.strip_prefix("request-bytes ") {
                let (_, hex) = value.split_once(' ').expect("request bytes");
                let mut bytes = decode_hex(hex).expect("request encoding");
                bytes[0] ^= 1;
                changed_byte = Some(bytes[0]);
                let _ = writeln!(
                    changed,
                    "request-bytes {} {}",
                    bytes.len(),
                    hex_bytes(&bytes)
                );
            } else if let Some(value) = line.strip_prefix("request-subject ") {
                let (_, hex) = value.split_once(' ').expect("subject bytes");
                let mut subject: serde_json::Value =
                    serde_json::from_slice(&decode_hex(hex).expect("subject encoding"))
                        .expect("subject JSON");
                subject["transaction_bytes"][0] =
                    serde_json::Value::from(changed_byte.expect("request byte"));
                let bytes = serde_json::to_vec(&subject).expect("subject JSON");
                let _ = writeln!(
                    changed,
                    "request-subject {} {}",
                    bytes.len(),
                    hex_bytes(&bytes)
                );
            } else {
                let _ = writeln!(changed, "{line}");
            }
        }
        changed.push_str(&end);
        changed.push_str(suffix);
        corpus.bytes[0] = changed.into_bytes();
    }

    #[test]
    fn both_mutant_runs_admit_through_their_pins() {
        for (files, pins, count) in [
            (&PREDECESSOR_MUTANT_FILES, &PREDECESSOR_MUTANT_PINS, 4),
            (&SUCCESSOR_MUTANT_FILES, &SUCCESSOR_MUTANT_PINS, 3),
        ] {
            let base = pins.pins();
            assert_eq!(base.names(), &files.map(|file| file.name()));
            assert_eq!(base.sizes(), &files.map(|file| file.bytes().len()));
            assert_eq!(
                base.ceremony_sha256(),
                hex_bytes(&tagged::sha256(files[0].bytes()))
            );
            assert_eq!(
                base.manifest_sha256(),
                hex_bytes(&tagged::sha256(files[2].bytes()))
            );
            assert_eq!(
                base.run_address(),
                hex_bytes(&tagged::sha256(files[3].bytes()))
            );
            assert_eq!(
                base.sponsorless(),
                MaturitySponsorlessExpectation::Acceptance
            );
            assert_eq!(
                base.framework_revision(),
                MaturityFrameworkRevision::IntendedTip
            );
            assert_eq!(base.funding_count(), 0);
            assert_eq!(base.fee_floors(), MATURITY_DISCLOSED_FEE_FLOORS);
            let report = std::str::from_utf8(files[3].bytes()).expect("mutant report");
            assert!(report.contains(&format!(
                "rust-test-target {}\n",
                base.capture_target().target()
            )));
            assert!(report.contains(&format!("capture-profile {}\n", pins.profile())));
            let argv_hex = report
                .lines()
                .find_map(|line| line.strip_prefix("cargo-argv "))
                .expect("cargo argv")
                .split_whitespace()
                .nth(1)
                .expect("cargo hex");
            assert_eq!(
                String::from_utf8(decode_hex(argv_hex).expect("cargo bytes")).expect("cargo text"),
                format!(
                    "{} test -p tripod-vectors --test {} -- --ignored --test-threads=1",
                    base.cargo_path(),
                    base.capture_target().target()
                )
            );
            let corpus = admit_maturity_mutant_corpus(files, pins).expect("mutant admission");
            assert_eq!(corpus.evidence().offers().len(), count);
            assert!(
                corpus
                    .evidence()
                    .offers()
                    .iter()
                    .all(|offer| corpus.evidence().binding(offer)
                        == Some(crate::maturity_evidence::MaturityRowBinding::Bound))
            );
            assert_eq!(corpus.evidence().escapes().count(), 0);
            assert!(corpus.evidence().control().is_some());
            assert_eq!(
                replay_maturity_mutant_run_of_record(pins.table()),
                Ok(corpus.evidence().clone())
            );
        }
    }

    #[test]
    fn a_moved_byte_is_refused_at_its_layer_in_each_mutant_run() {
        for (files, pins) in [
            (&PREDECESSOR_MUTANT_FILES, PREDECESSOR_MUTANT_PINS),
            (&SUCCESSOR_MUTANT_FILES, SUCCESSOR_MUTANT_PINS),
        ] {
            let mut names = *files;
            names[0] = MaturityArchiveFile::new("other.capture", files[0].bytes());
            assert_eq!(
                admit_maturity_mutant_corpus(&names, &pins),
                Err(Refusal::CorpusCensus)
            );
            let mut corpus = OwnedMutantCorpus::new(files, &pins);
            corpus.bytes[0].pop();
            assert!(matches!(corpus.pinned(), Err(Refusal::FileSize { .. })));
            let mut corpus = OwnedMutantCorpus::new(files, &pins);
            corpus.bytes[2][0] ^= 1;
            assert_eq!(corpus.pinned(), Err(Refusal::ManifestHash));
            let mut corpus = OwnedMutantCorpus::new(files, &pins);
            corpus.bytes[3][0] ^= 1;
            assert_eq!(corpus.pinned(), Err(Refusal::RunReportGrammar));
            let mut corpus = OwnedMutantCorpus::new(files, &pins);
            corpus.bytes[0][0] ^= 1;
            assert!(matches!(
                corpus.pinned(),
                Err(Refusal::ManifestDigest { .. })
            ));
            let mut corpus = OwnedMutantCorpus::new(files, &pins);
            corpus.payload_replace("mutant-record mutant-", "mutant-record altered-mutant-");
            assert!(matches!(
                corpus.rebind(true),
                Err(Refusal::PayloadDisagreement { .. })
            ));
            let mut corpus = OwnedMutantCorpus::new(files, &pins);
            let position = if pins.table() == MaturityMutantTable::PredecessorConstructor {
                3
            } else {
                2
            };
            flip_mutant_request(&mut corpus, position);
            assert_eq!(
                corpus.rebind(true),
                Err(Refusal::Derivation(
                    MaturityNativePlanRefusal::TranscriptStepMismatch { position }
                ))
            );
        }
    }

    #[test]
    fn the_earlier_runs_admit_under_their_restated_targets() {
        for (files, pins) in [
            (&FILES, &HISTORICAL_PINS),
            (&VARIABLE_FILES, &VARIABLE_PINS),
        ] {
            assert_eq!(
                pins.capture_target(),
                MaturityCaptureTarget::new(
                    "maturity_native",
                    "the_maturity_announcement_runs_against_a_real_target"
                )
            );
            assert!(admit_maturity_corpus(files, pins).is_ok());
        }
        let report = std::str::from_utf8(FILES[3].bytes()).expect("historical report");
        assert!(report.contains(&format!(
            "rust-test-target {}\n",
            HISTORICAL_PINS.capture_target().target()
        )));
    }

    #[test]
    fn phase_walls_are_read_and_bind_nothing() {
        for (files, pins) in [
            (&PREDECESSOR_MUTANT_FILES, PREDECESSOR_MUTANT_PINS),
            (&SUCCESSOR_MUTANT_FILES, SUCCESSOR_MUTANT_PINS),
        ] {
            let corpus = admit_maturity_mutant_corpus(files, &pins).expect("mutant archive");
            let report = std::str::from_utf8(files[3].bytes()).expect("mutant report");
            let walls: Vec<u64> = report
                .lines()
                .filter_map(|line| line.strip_prefix("phase-wall-ms "))
                .map(|line| {
                    line.split_once(' ')
                        .expect("phase wall")
                        .1
                        .parse()
                        .expect("wall digits")
                })
                .collect();
            assert_eq!(corpus.phase_walls().as_slice(), walls);
            let mut reordered = OwnedMutantCorpus::new(files, &pins);
            let first = report
                .lines()
                .find(|line| line.starts_with("phase-wall-ms preflight "))
                .expect("preflight wall");
            let second = report
                .lines()
                .find(|line| line.starts_with("phase-wall-ms cargo "))
                .expect("cargo wall");
            reordered.replace(
                3,
                &format!("{first}\n{second}\n"),
                &format!("{second}\n{first}\n"),
            );
            assert_eq!(reordered.rebind(false), Err(Refusal::RunReportGrammar));
            let mut missing = OwnedMutantCorpus::new(files, &pins);
            missing.replace(3, &format!("{first}\n"), "");
            assert_eq!(missing.rebind(false), Err(Refusal::RunReportGrammar));
            let mut nondigit = OwnedMutantCorpus::new(files, &pins);
            nondigit.replace(3, first, "phase-wall-ms preflight absent");
            assert_eq!(nondigit.rebind(false), Err(Refusal::RunReportGrammar));
            let mut changed = OwnedMutantCorpus::new(files, &pins);
            for (phase, value) in ["preflight", "cargo", "census", "manifest"]
                .iter()
                .zip([1, 2, 3, 4])
            {
                let original = report
                    .lines()
                    .find(|line| line.starts_with(&format!("phase-wall-ms {phase} ")))
                    .expect("phase wall");
                changed.replace(3, original, &format!("phase-wall-ms {phase} {value}"));
            }
            assert!(changed.rebind(false).is_ok());
        }
    }
}

#[cfg(test)]
mod observer_archive_tests {
    use super::*;

    #[test]
    fn observer_archive_refuses_phase_sum_disagreement() {
        let report = std::str::from_utf8(OBSERVER_FILES[3].bytes).expect("report");
        let elapsed = report
            .lines()
            .find(|line| line.starts_with("elapsed-wall-ms "))
            .expect("elapsed wall");
        let duration = elapsed
            .strip_prefix("elapsed-wall-ms ")
            .expect("prefix")
            .parse::<u64>()
            .expect("duration");
        let changed = report.replace(elapsed, &format!("elapsed-wall-ms {}", duration + 1));
        let report_pin = hex_bytes(&tagged::sha256(changed.as_bytes()));
        let mut files = OBSERVER_FILES;
        files[3] = MaturityArchiveFile::new("RUN-REPORT", changed.as_bytes());
        let mut pins = OBSERVER_PINS;
        pins[1] = &report_pin;
        assert_eq!(
            observer_report(&files, pins).expect_err("inconsistent walls"),
            "observer-archive:phase-sum"
        );
    }

    #[test]
    fn observer_archive_replays_reports_and_preserves_each_boundary() {
        let admitted = maturity_observer_run_of_record().expect("independent observation archive");
        let observation = admitted.observation();
        let rows = observation["experiment"].as_array().expect("experiment");
        let keys: Vec<_> = rows
            .iter()
            .map(|row| row["key"].as_str().expect("row key"))
            .collect();
        assert_eq!(
            keys,
            [
                "16.13 / missing-transaction",
                "checkpoint-genesis",
                "16.13 / copied-transaction-bytes",
                "checkpoint-block",
                "checkpoint-height",
                "equal-final-cursor / altered-predecessor-edge-refused",
                "accepted-observation"
            ]
        );
        assert_eq!(rows[5]["equal_final_cursor"], true);
        assert_eq!(
            rows[5]["related"],
            "16.8 / successor-cursor-restored-after-invalid-intermediate-edge"
        );
        assert_eq!(rows[6]["programs_agree"], true);
        for field in ["history_report", "recovery_report"] {
            let report = observation[field].as_str().expect("report");
            if field == "recovery_report" {
                assert!(report.contains("residual no-chain-observation\n"));
                assert!(report.contains("acquisition_residual single-node-acquisition-only replaces no-chain-observation\n"));
            } else {
                assert!(report.contains("residual single-node-acquisition-only\n"));
                assert!(!report.contains("residual no-chain-observation\n"));
            }
            assert!(report.contains("residual no-root-cursor-freshness\n"));
            assert!(report.contains("residual no-archive-origin-beyond-admission\n"));
            assert!(report.contains("residual no-full-artifact-authentication\n"));
        }
        assert!(
            observation["history_report"]
                .as_str()
                .expect("history")
                .contains("edges 1\n")
        );
        assert!(
            observation["recovery_report"]
                .as_str()
                .expect("recovery")
                .contains("programs_agree true\n")
        );
    }

    #[test]
    fn observer_archive_refuses_each_changed_file() {
        for (position, file) in OBSERVER_FILES.iter().enumerate() {
            let mut changed = file.bytes.to_vec();
            changed[0] ^= 1;
            let mut files = OBSERVER_FILES;
            files[position] = MaturityArchiveFile::new(file.name, &changed);
            assert!(
                admit_observer_maturity_corpus(&files, OBSERVER_PINS).is_err(),
                "changed {}",
                file.name
            );
        }
    }

    #[test]
    fn observer_archive_refuses_a_different_declared_chain_id() {
        let mut record: serde_json::Value =
            serde_json::from_slice(OBSERVER_FILES[0].bytes).expect("capture");
        record["observation"]["manifest"]["chain_id"] = serde_json::json!("de".repeat(32));
        let capture = format!("{record}\n");
        let ceremony_pin = hex_bytes(&tagged::sha256(capture.as_bytes()));
        let manifest = format!(
            "{}  {}\n{}  {}\n",
            ceremony_pin,
            OBSERVER_FILES[0].name,
            hex_bytes(&tagged::sha256(OBSERVER_FILES[1].bytes)),
            OBSERVER_FILES[1].name
        );
        let manifest_pin = hex_bytes(&tagged::sha256(manifest.as_bytes()));
        let report = std::str::from_utf8(OBSERVER_FILES[3].bytes)
            .expect("report")
            .replace(OBSERVER_PINS[2], &ceremony_pin)
            .replace(OBSERVER_PINS[0], &manifest_pin);
        let report_pin = hex_bytes(&tagged::sha256(report.as_bytes()));
        let files = [
            MaturityArchiveFile::new(OBSERVER_FILES[0].name, capture.as_bytes()),
            OBSERVER_FILES[1],
            MaturityArchiveFile::new("MANIFEST.sha256", manifest.as_bytes()),
            MaturityArchiveFile::new("RUN-REPORT", report.as_bytes()),
        ];
        let refusal =
            admit_observer_maturity_corpus(&files, [&manifest_pin, &report_pin, &ceremony_pin])
                .err()
                .expect("different declaration refused");
        assert_eq!(refusal, "observer-archive:declared-chain-id-disagrees");
    }
}
