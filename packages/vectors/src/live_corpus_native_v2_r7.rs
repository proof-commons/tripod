//! Strict importer for the reviewed native-v2, protocol-revision-7 corpus.
//!
//! The archive is admitted in layers. Its fixed file census and manifest
//! are checked before either the run report or a semantic transcript is
//! parsed. Only the resulting validated wrapper exposes report bindings or
//! fresh observations; callers cannot construct one from partial material.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::sync::OnceLock;

use target_elements_conformance::constructor::tagged;
use target_elements_conformance::protocol::ObservedOutcomeLayer;
use transaction::bytes::{SerializedFieldLocator, SerializedOutputField};
use transaction::{AssetField, TargetTransaction, Txid, ValueField};

use crate::live_report::{
    LiveCompositeAcceptanceMember, LiveCompositeTwoAcceptance, LiveMultiRowSemanticWitness,
    LiveMutantKind, LiveMutationLocator, LivePairMember, LivePairProjectionInput,
    LiveReportObservation, LiveRequestFact, LiveRowSemanticPredicate, LiveRunBinding,
    LiveSupportLink, LiveTargetResponse, LiveWitnessPathRole,
};
use crate::live_safety::required_safety_matrix;
use crate::matrix::EvidenceBoundary;

/// The run's own address: the digest of the label-derived input set the
/// capture driver publishes, under a separator that spells the product's
/// name.
///
/// It names the corpus in its filenames and in its grammar line, and it
/// is not the report digest below. That one is the address of the bytes
/// this corpus IS; this one is the address of the inputs the run was
/// taken OVER, and a reader recomputes it from the driver rather than
/// from anything here.
pub const NATIVE_V2_R7_INPUT_SET_ADDRESS: &str =
    "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517";

/// SHA-256 of the exact 79-entry manifest bytes.
pub const NATIVE_V2_R7_MANIFEST_SHA256: &str =
    "189704e06355609653eeaa4a8d7660ce79c8fd469ba23f76c050beda8f9a1412";

/// SHA-256 of the report that binds the manifest and complete suite census.
pub const NATIVE_V2_R7_RUN_ADDRESS: &str =
    "e1f3fee707c9762abb08ad631bfe2f6fd874f1e13d2c4e336e42ff021c23354c";

const MANIFEST_BYTES: &[u8] = include_bytes!(concat!(
    "../fixtures/native-v2-r7-run-of-record/",
    "MANIFEST.sha256"
));
const RUN_REPORT_BYTES: &[u8] = include_bytes!(concat!(
    "../fixtures/native-v2-r7-run-of-record/",
    "RUN-REPORT"
));

/// The driver-owned semantic ceremony roster, in canonical sort order.
pub const NATIVE_V2_R7_CEREMONY_ROSTER: [&str; 39] = [
    "conservation-negatives",
    "explicit-boundary-values",
    "explicit-maximum-inputs",
    "explicit-maximum-outputs",
    "explicit-merge",
    "explicit-normalization",
    "explicit-one-destination-owner",
    "explicit-one-to-one",
    "explicit-repeated-owner",
    "explicit-self-paid-fee",
    "explicit-several-destination-owners",
    "explicit-several-owners",
    "explicit-several-to-several",
    "explicit-split",
    "explicit-sponsorless",
    "explicit-witness-negatives",
    "keypath-probe",
    "multi-entry-crossing",
    "multi-exit-crossing",
    "multi-many-to-many",
    "multi-one-to-one-with-fee",
    "multi-private-merge",
    "multi-pure-split",
    "multi-several-owners",
    "multi-split",
    "multi-strict-one-to-one",
    "owner-observation",
    "owner-signing-negatives",
    "pairs-arc",
    "private-restart-control",
    "private-restart-parity",
    "proof-bearing-observation",
    "report",
    "sponsored-change-absent",
    "sponsored-change-present",
    "sponsored-committed-value",
    "sponsored-missing-authorization",
    "sponsored-private-explicit-no-change",
    "sponsored-private-with-change",
];

const ROW_ROSTER: [&str; 42] = [
    "both-commitment-parity-forms",
    "candidate-maximum-inputs",
    "candidate-maximum-outputs",
    "canonical-input-normalization",
    "one-destination-owner",
    "one-input-split-into-two",
    "one-input-to-one-output",
    "private-many-to-many-representative",
    "private-sponsor-values",
    "private-merge",
    "private-one-to-one",
    "private-several-distinct-owners",
    "private-split",
    "repeated-owner",
    "semantic-boundary-values",
    "several-destination-owners",
    "several-distinct-owners",
    "several-inputs-merged-into-one",
    "several-inputs-to-several-outputs",
    "sponsor-change-absent",
    "sponsor-change-present",
    "sponsored",
    "sponsorless",
    "target-ct-conservation",
    "confidential-asset-commitment",
    "empty-signature",
    "hidden-private-u-output",
    "key-path-escape",
    "malformed-rangeproof",
    "malformed-signature",
    "missing-sponsor-authorization",
    "no-coordinator",
    "omitted-source",
    "output-total-one-above-input",
    "output-total-one-below-input",
    "private-ct-imbalance",
    "private-output-omitted",
    "two-coordinators",
    "vault-control-entitlement-or-bare-u-output",
    "wrong-explicit-asset",
    "wrong-private-blinding-balance",
    "projection-equality-with-paired-explicit",
];

#[derive(Clone, Copy)]
struct ArchiveFile {
    name: &'static str,
    bytes: &'static [u8],
    expected_size: usize,
}

const ARCHIVE_FILES: [ArchiveFile; 79] = [
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.confidential-predecessor.setup",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.confidential-predecessor.setup"
        ),
        expected_size: 2957,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.conservation-negatives.capture",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.conservation-negatives.capture"
        ),
        expected_size: 93899,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.conservation-negatives.capture.timing",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.conservation-negatives.capture.timing"
        ),
        expected_size: 65,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.explicit-boundary-values.capture",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.explicit-boundary-values.capture"
        ),
        expected_size: 9453,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.explicit-boundary-values.capture.timing",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.explicit-boundary-values.capture.timing"
        ),
        expected_size: 67,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.explicit-maximum-inputs.capture",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.explicit-maximum-inputs.capture"
        ),
        expected_size: 11326,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.explicit-maximum-inputs.capture.timing",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.explicit-maximum-inputs.capture.timing"
        ),
        expected_size: 66,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.explicit-maximum-outputs.capture",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.explicit-maximum-outputs.capture"
        ),
        expected_size: 9845,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.explicit-maximum-outputs.capture.timing",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.explicit-maximum-outputs.capture.timing"
        ),
        expected_size: 67,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.explicit-merge.capture",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.explicit-merge.capture"
        ),
        expected_size: 10228,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.explicit-merge.capture.timing",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.explicit-merge.capture.timing"
        ),
        expected_size: 57,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.explicit-normalization.capture",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.explicit-normalization.capture"
        ),
        expected_size: 10272,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.explicit-normalization.capture.timing",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.explicit-normalization.capture.timing"
        ),
        expected_size: 65,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.explicit-one-destination-owner.capture",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.explicit-one-destination-owner.capture"
        ),
        expected_size: 10546,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.explicit-one-destination-owner.capture.timing",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.explicit-one-destination-owner.capture.timing"
        ),
        expected_size: 73,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.explicit-one-to-one.capture",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.explicit-one-to-one.capture"
        ),
        expected_size: 9039,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.explicit-one-to-one.capture.timing",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.explicit-one-to-one.capture.timing"
        ),
        expected_size: 62,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.explicit-repeated-owner.capture",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.explicit-repeated-owner.capture"
        ),
        expected_size: 10872,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.explicit-repeated-owner.capture.timing",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.explicit-repeated-owner.capture.timing"
        ),
        expected_size: 66,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.explicit-self-paid-fee.capture",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.explicit-self-paid-fee.capture"
        ),
        expected_size: 9469,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.explicit-self-paid-fee.capture.timing",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.explicit-self-paid-fee.capture.timing"
        ),
        expected_size: 65,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.explicit-several-destination-owners.capture",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.explicit-several-destination-owners.capture"
        ),
        expected_size: 9529,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.explicit-several-destination-owners.capture.timing",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.explicit-several-destination-owners.capture.timing"
        ),
        expected_size: 78,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.explicit-several-owners.capture",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.explicit-several-owners.capture"
        ),
        expected_size: 11158,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.explicit-several-owners.capture.timing",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.explicit-several-owners.capture.timing"
        ),
        expected_size: 66,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.explicit-several-to-several.capture",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.explicit-several-to-several.capture"
        ),
        expected_size: 10550,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.explicit-several-to-several.capture.timing",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.explicit-several-to-several.capture.timing"
        ),
        expected_size: 70,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.explicit-split.capture",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.explicit-split.capture"
        ),
        expected_size: 9399,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.explicit-split.capture.timing",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.explicit-split.capture.timing"
        ),
        expected_size: 57,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.explicit-sponsorless.capture",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.explicit-sponsorless.capture"
        ),
        expected_size: 9021,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.explicit-sponsorless.capture.timing",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.explicit-sponsorless.capture.timing"
        ),
        expected_size: 63,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.explicit-witness-negatives.capture",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.explicit-witness-negatives.capture"
        ),
        expected_size: 14078,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.explicit-witness-negatives.capture.timing",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.explicit-witness-negatives.capture.timing"
        ),
        expected_size: 69,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.keypath-probe.capture",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.keypath-probe.capture"
        ),
        expected_size: 17167,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.keypath-probe.capture.timing",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.keypath-probe.capture.timing"
        ),
        expected_size: 56,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.multi-entry-crossing.capture",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.multi-entry-crossing.capture"
        ),
        expected_size: 24829,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.multi-entry-crossing.capture.timing",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.multi-entry-crossing.capture.timing"
        ),
        expected_size: 63,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.multi-exit-crossing.capture",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.multi-exit-crossing.capture"
        ),
        expected_size: 17693,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.multi-exit-crossing.capture.timing",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.multi-exit-crossing.capture.timing"
        ),
        expected_size: 62,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.multi-many-to-many.capture",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.multi-many-to-many.capture"
        ),
        expected_size: 34490,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.multi-many-to-many.capture.timing",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.multi-many-to-many.capture.timing"
        ),
        expected_size: 61,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.multi-one-to-one-with-fee.capture",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.multi-one-to-one-with-fee.capture"
        ),
        expected_size: 16605,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.multi-one-to-one-with-fee.capture.timing",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.multi-one-to-one-with-fee.capture.timing"
        ),
        expected_size: 68,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.multi-private-merge.capture",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.multi-private-merge.capture"
        ),
        expected_size: 17410,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.multi-private-merge.capture.timing",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.multi-private-merge.capture.timing"
        ),
        expected_size: 62,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.multi-pure-split.capture",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.multi-pure-split.capture"
        ),
        expected_size: 24971,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.multi-pure-split.capture.timing",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.multi-pure-split.capture.timing"
        ),
        expected_size: 59,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.multi-several-owners.capture",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.multi-several-owners.capture"
        ),
        expected_size: 25797,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.multi-several-owners.capture.timing",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.multi-several-owners.capture.timing"
        ),
        expected_size: 63,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.multi-split.capture",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.multi-split.capture"
        ),
        expected_size: 33682,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.multi-split.capture.timing",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.multi-split.capture.timing"
        ),
        expected_size: 54,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.multi-strict-one-to-one.capture",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.multi-strict-one-to-one.capture"
        ),
        expected_size: 16529,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.multi-strict-one-to-one.capture.timing",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.multi-strict-one-to-one.capture.timing"
        ),
        expected_size: 66,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.owner-observation.capture",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.owner-observation.capture"
        ),
        expected_size: 33984,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.owner-observation.capture.timing",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.owner-observation.capture.timing"
        ),
        expected_size: 60,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.owner-signing-negatives.capture",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.owner-signing-negatives.capture"
        ),
        expected_size: 48046,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.owner-signing-negatives.capture.timing",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.owner-signing-negatives.capture.timing"
        ),
        expected_size: 66,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.pairs-arc.capture",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.pairs-arc.capture"
        ),
        expected_size: 20798,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.pairs-arc.capture.timing",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.pairs-arc.capture.timing"
        ),
        expected_size: 52,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.private-restart-control.capture",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.private-restart-control.capture"
        ),
        expected_size: 25169,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.private-restart-control.capture.timing",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.private-restart-control.capture.timing"
        ),
        expected_size: 66,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.private-restart-parity.capture",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.private-restart-parity.capture"
        ),
        expected_size: 25179,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.private-restart-parity.capture.timing",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.private-restart-parity.capture.timing"
        ),
        expected_size: 65,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.proof-bearing-observation.capture",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.proof-bearing-observation.capture"
        ),
        expected_size: 90105,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.proof-bearing-observation.capture.timing",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.proof-bearing-observation.capture.timing"
        ),
        expected_size: 68,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.report.capture",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.report.capture"
        ),
        expected_size: 9177,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.report.capture.timing",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.report.capture.timing"
        ),
        expected_size: 49,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.sponsored-change-absent.capture",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.sponsored-change-absent.capture"
        ),
        expected_size: 12022,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.sponsored-change-absent.capture.timing",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.sponsored-change-absent.capture.timing"
        ),
        expected_size: 66,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.sponsored-change-present.capture",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.sponsored-change-present.capture"
        ),
        expected_size: 12478,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.sponsored-change-present.capture.timing",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.sponsored-change-present.capture.timing"
        ),
        expected_size: 67,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.sponsored-committed-value.capture",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.sponsored-committed-value.capture"
        ),
        expected_size: 13017,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.sponsored-committed-value.capture.timing",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.sponsored-committed-value.capture.timing"
        ),
        expected_size: 68,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.sponsored-missing-authorization.capture",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.sponsored-missing-authorization.capture"
        ),
        expected_size: 16108,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.sponsored-missing-authorization.capture.timing",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.sponsored-missing-authorization.capture.timing"
        ),
        expected_size: 74,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.sponsored-private-explicit-no-change.capture",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.sponsored-private-explicit-no-change.capture"
        ),
        expected_size: 17163,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.sponsored-private-explicit-no-change.capture.timing",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.sponsored-private-explicit-no-change.capture.timing"
        ),
        expected_size: 79,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.sponsored-private-with-change.capture",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.sponsored-private-with-change.capture"
        ),
        expected_size: 35739,
    },
    ArchiveFile {
        name: "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.sponsored-private-with-change.capture.timing",
        bytes: include_bytes!(
            "../fixtures/native-v2-r7-run-of-record/e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.sponsored-private-with-change.capture.timing"
        ),
        expected_size: 72,
    },
];

/// Whether a proven archive link is row-bearing or reusable control support.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum NativeV2LinkClass {
    /// A row-bearing acceptance, mutant, or pair member.
    Primary,
    /// A same-ceremony accepted control supporting a refusal.
    Support,
}

/// One resolved request link backed by a validated run binding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvenNativeV2Link {
    row: &'static str,
    class: NativeV2LinkClass,
    ceremony: String,
    run_id: String,
    request_id: String,
}

impl ProvenNativeV2Link {
    /// The exact charter row this link contributes to.
    #[must_use]
    pub const fn row(&self) -> &'static str {
        self.row
    }

    /// Whether this is primary evidence or reusable support.
    #[must_use]
    pub const fn class(&self) -> NativeV2LinkClass {
        self.class
    }

    /// The named ceremony carrying the request.
    #[must_use]
    pub fn ceremony(&self) -> &str {
        &self.ceremony
    }

    /// The content address of the validated run.
    #[must_use]
    pub fn run_id(&self) -> &str {
        &self.run_id
    }

    /// The globally scoped request identity in that run.
    #[must_use]
    pub fn request_id(&self) -> &str {
        &self.request_id
    }
}

/// Strictly parsed summary of one complete semantic transcript.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeV2Transcript {
    ceremony: String,
    operation_count: usize,
    content_sha256: [u8; 32],
}

impl NativeV2Transcript {
    /// Stable driver ceremony identity.
    #[must_use]
    pub fn ceremony(&self) -> &str {
        &self.ceremony
    }

    /// Exact number of progressively captured executor outcomes.
    #[must_use]
    pub const fn operation_count(&self) -> usize {
        self.operation_count
    }

    /// Transcript content address over every byte through the run boundary.
    #[must_use]
    pub const fn content_sha256(&self) -> [u8; 32] {
        self.content_sha256
    }
}

/// One exact charter row and every archive ceremony attributed to it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeV2RowAttribution {
    row: &'static str,
    ceremonies: Vec<String>,
}

impl NativeV2RowAttribution {
    /// The fixed charter row.
    #[must_use]
    pub const fn row(&self) -> &'static str {
        self.row
    }

    /// One or two exact ceremony names from the reviewed row map.
    #[must_use]
    pub fn ceremonies(&self) -> &[String] {
        &self.ceremonies
    }
}

/// Fully admitted native-v2/revision-7 run of record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedNativeV2R7Corpus {
    transcripts: Vec<NativeV2Transcript>,
    runs: Vec<LiveRunBinding>,
    observations: Vec<LiveReportObservation>,
    links: Vec<ProvenNativeV2Link>,
    attributions: Vec<NativeV2RowAttribution>,
    outcome_count: usize,
    content_address: String,
}

impl ValidatedNativeV2R7Corpus {
    /// All 39 transcripts in ceremony order.
    #[must_use]
    pub fn transcripts(&self) -> &[NativeV2Transcript] {
        &self.transcripts
    }

    /// Row-bearing run bindings, sorted by their content address.
    #[must_use]
    pub fn runs(&self) -> &[LiveRunBinding] {
        &self.runs
    }

    /// Typed fresh observations derived only from validated archive material.
    #[must_use]
    pub fn observations(&self) -> &[LiveReportObservation] {
        &self.observations
    }

    /// Proven primary and reusable support links in row order.
    #[must_use]
    pub fn links(&self) -> &[ProvenNativeV2Link] {
        &self.links
    }

    /// Exact R01 through R42 attribution map.
    #[must_use]
    pub fn attributions(&self) -> &[NativeV2RowAttribution] {
        &self.attributions
    }

    /// The report's complete 40-outcome suite census.
    #[must_use]
    pub const fn outcome_count(&self) -> usize {
        self.outcome_count
    }

    /// Global content address of the report-bound corpus.
    #[must_use]
    pub fn content_address(&self) -> &str {
        &self.content_address
    }
}

/// Why archive admission stopped.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum NativeV2ImportRefusal {
    /// The fixed top-level or manifest-owned file census differs.
    CorpusCensus { expected: usize, actual: usize },
    /// One fixed archive member has a different byte length.
    FileSize {
        name: String,
        expected: usize,
        actual: usize,
    },
    /// The manifest is not the pinned run-of-record manifest.
    ManifestHash,
    /// The manifest violates its closed, sorted 79-line grammar.
    ManifestGrammar,
    /// A manifest entry does not hash its exact embedded file.
    ManifestDigest { name: String },
    /// The report violates its closed grammar or fixed eligibility census.
    RunReportGrammar,
    /// Report and manifest content addresses differ.
    ReportManifestBinding,
    /// A transcript violates the closed ordered grammar.
    TranscriptGrammar { name: String, line: usize },
    /// A transcript's own content address differs.
    CaptureContentHash { ceremony: String },
    /// Suite, deployment, executor, protocol, or digest facts disagree.
    CrossFileBinding {
        ceremony: String,
        field: &'static str,
    },
    /// Operation, request, or response identity linkage is not exact.
    IdentifierLink { ceremony: String, request: String },
    /// A transaction did not decode and re-encode byte-for-byte.
    TransactionDecode { ceremony: String, request: String },
    /// An accepted target identity differs from the recomputed txid.
    TransactionIdentity { ceremony: String, request: String },
    /// A decoded mutant and control contradict the declared locator.
    MutationLocator { ceremony: String, request: String },
    /// The paired members do not independently prove all 13 terms.
    PairProjection,
    /// The fixed R01 through R42 attribution did not cross-foot.
    RowAttribution,
    /// Schema-6 run material could not be minted from validated facts.
    RunBinding,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ReportFacts {
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
    manifest_hash: [u8; 32],
    ceremony_digests: BTreeMap<String, [u8; 32]>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum OperationRole {
    Auxiliary,
    Acceptance,
    Control,
    Refusal,
    Paired(LivePairMember),
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ParsedOperation {
    operation_id: String,
    request_id: String,
    response_id: String,
    role: OperationRole,
    request_bytes: Vec<u8>,
    layer: ObservedOutcomeLayer,
    accepted_identity: Option<Txid>,
    detail: String,
    control_request_id: Option<String>,
    control_identity: Option<Txid>,
    mutant: Option<LiveMutantKind>,
    locator: Option<LiveMutationLocator>,
    projection: Option<ParsedProjection>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ParsedProjection {
    input_owners: Vec<String>,
    semantic_input_amounts: Vec<u64>,
    destination_programs: BTreeMap<String, Vec<u8>>,
    semantic_destination_amounts: BTreeMap<String, u64>,
}

impl ParsedProjection {
    fn live(&self) -> LivePairProjectionInput {
        LivePairProjectionInput::new(
            self.input_owners.clone(),
            self.semantic_input_amounts.clone(),
            self.destination_programs.clone(),
            self.semantic_destination_amounts.clone(),
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ParsedTranscript {
    summary: NativeV2Transcript,
    run_archive: Vec<u8>,
    operations: Vec<ParsedOperation>,
}

#[derive(Clone, Copy)]
struct InputFile<'a> {
    name: &'a str,
    bytes: &'a [u8],
    expected_size: usize,
}

#[derive(Clone)]
struct CorpusInputs<'a> {
    files: Vec<InputFile<'a>>,
    manifest: &'a [u8],
    report: &'a [u8],
}

impl CorpusInputs<'static> {
    fn embedded() -> Self {
        Self {
            files: ARCHIVE_FILES
                .iter()
                .map(|file| InputFile {
                    name: file.name,
                    bytes: file.bytes,
                    expected_size: file.expected_size,
                })
                .collect(),
            manifest: MANIFEST_BYTES,
            report: RUN_REPORT_BYTES,
        }
    }
}

struct LineCursor<'a> {
    name: &'a str,
    lines: Vec<&'a str>,
    index: usize,
}

impl<'a> LineCursor<'a> {
    fn new(name: &'a str, bytes: &'a [u8]) -> Result<Self, NativeV2ImportRefusal> {
        if bytes.contains(&b'\r') || !bytes.ends_with(b"\n") {
            return Err(NativeV2ImportRefusal::TranscriptGrammar {
                name: name.to_owned(),
                line: 1,
            });
        }
        let text =
            std::str::from_utf8(bytes).map_err(|_| NativeV2ImportRefusal::TranscriptGrammar {
                name: name.to_owned(),
                line: 1,
            })?;
        let body =
            text.strip_suffix('\n')
                .ok_or_else(|| NativeV2ImportRefusal::TranscriptGrammar {
                    name: name.to_owned(),
                    line: 1,
                })?;
        if body.split('\n').any(str::is_empty) {
            return Err(NativeV2ImportRefusal::TranscriptGrammar {
                name: name.to_owned(),
                line: 1,
            });
        }
        Ok(Self {
            name,
            lines: body.split('\n').collect(),
            index: 0,
        })
    }

    fn refusal(&self) -> NativeV2ImportRefusal {
        NativeV2ImportRefusal::TranscriptGrammar {
            name: self.name.to_owned(),
            line: self.index + 1,
        }
    }

    fn exact(&mut self, expected: &str) -> Result<(), NativeV2ImportRefusal> {
        if self.lines.get(self.index).copied() != Some(expected) {
            return Err(self.refusal());
        }
        self.index += 1;
        Ok(())
    }

    fn value(&mut self, field: &str) -> Result<&'a str, NativeV2ImportRefusal> {
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

    fn len_hex(&mut self, field: &str) -> Result<Vec<u8>, NativeV2ImportRefusal> {
        let value = self.value(field)?;
        parse_len_hex(value).ok_or_else(|| self.refusal())
    }

    fn number(&mut self, field: &str) -> Result<usize, NativeV2ImportRefusal> {
        let value = self.value(field)?;
        parse_usize(value).ok_or_else(|| self.refusal())
    }

    fn done(&self) -> Result<(), NativeV2ImportRefusal> {
        if self.index == self.lines.len() {
            Ok(())
        } else {
            Err(self.refusal())
        }
    }
}

fn parse_usize(text: &str) -> Option<usize> {
    if text.is_empty()
        || text.len() > 1 && text.starts_with('0')
        || !text.bytes().all(|byte| byte.is_ascii_digit())
    {
        return None;
    }
    text.parse().ok()
}

fn parse_u64(text: &str) -> Option<u64> {
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
    let (pairs, remainder) = text.as_bytes().as_chunks::<2>();
    if !remainder.is_empty() {
        return None;
    }
    pairs
        .iter()
        .map(|pair| {
            let high = if pair[0].is_ascii_digit() {
                pair[0] - b'0'
            } else {
                pair[0] - b'a' + 10
            };
            let low = if pair[1].is_ascii_digit() {
                pair[1] - b'0'
            } else {
                pair[1] - b'a' + 10
            };
            Some((high << 4) | low)
        })
        .collect()
}

fn decode_digest(text: &str) -> Option<[u8; 32]> {
    decode_hex(text)?.try_into().ok()
}

fn parse_len_hex(value: &str) -> Option<Vec<u8>> {
    let (length, payload) = value.split_once(' ')?;
    let decoded = decode_hex(payload)?;
    (parse_usize(length)? == decoded.len()).then_some(decoded)
}

fn parse_text(value: &str) -> Option<String> {
    String::from_utf8(parse_len_hex(value)?).ok()
}

fn txid_for(bytes: &[u8]) -> Result<Txid, ()> {
    let transaction = TargetTransaction::decode(bytes).map_err(|_| ())?;
    if transaction.encode() != bytes {
        return Err(());
    }
    let first = tagged::sha256(&transaction.encode_without_witness());
    Ok(Txid::from_internal(tagged::sha256(&first)))
}

struct ReportSuiteFacts {
    binary_revision: String,
    intended_tip: String,
}

struct ReportExecutorFacts {
    adapter_name: String,
    adapter_version: String,
    node_name: String,
    node_version: String,
    environment: String,
    network_id: String,
    genesis_id: String,
    target_contract: String,
}

fn parse_report_suite(
    cursor: &mut LineCursor<'_>,
) -> Result<ReportSuiteFacts, NativeV2ImportRefusal> {
    cursor.exact("run-report-schema native-v2-r7-run-report 1")?;
    cursor.exact("capture-format-schema native-v2-r7-capture 1")?;
    cursor.exact(&format!("suite-commit {NATIVE_V2_R7_INPUT_SET_ADDRESS}"))?;
    let suite_tree = cursor.value("suite-tree")?.to_owned();
    if decode_digest(&suite_tree).is_none() {
        return Err(cursor.refusal());
    }
    cursor.exact("suite-clean yes")?;
    cursor.exact("rust-test-target guide13_live_native")?;
    if cursor.len_hex("cargo-argv")?.is_empty() {
        return Err(cursor.refusal());
    }
    for line in [
        "expected-test-count 40",
        "observed-test-count 40",
        "expected-ceremony-count 39",
        "observed-ceremony-count 39",
        "expected-setup-count 1",
        "observed-setup-count 1",
        "diagnostics-present yes",
        "test-passed-count 40",
        "test-failed-count 0",
        "test-ignored-count 0",
        "cargo-exit-code 0",
    ] {
        cursor.exact(line)?;
    }
    let expected_tip = cursor.value("elementsd-expected-tip")?.to_owned();
    if expected_tip.len() != 40 {
        return Err(cursor.refusal());
    }
    if decode_hex(&expected_tip).is_none() {
        return Err(cursor.refusal());
    }
    let binary_revision = parse_text(cursor.value("elementsd-binary-reported-revision")?)
        .ok_or_else(|| cursor.refusal())?;
    let intended_tip = parse_text(cursor.value("elementsd-intended-executed-tip")?)
        .ok_or_else(|| cursor.refusal())?;
    if binary_revision.len() < 12 {
        return Err(cursor.refusal());
    }
    if binary_revision.len() > expected_tip.len() {
        return Err(cursor.refusal());
    }
    if !expected_tip.starts_with(&binary_revision) {
        return Err(cursor.refusal());
    }
    if intended_tip != expected_tip {
        return Err(cursor.refusal());
    }
    Ok(ReportSuiteFacts {
        binary_revision,
        intended_tip,
    })
}

fn parse_report_executor(
    cursor: &mut LineCursor<'_>,
) -> Result<ReportExecutorFacts, NativeV2ImportRefusal> {
    let adapter_name = text_field(cursor, "executor-adapter-name")?;
    let adapter_version = text_field(cursor, "executor-adapter-version")?;
    let node_name = text_field(cursor, "node-name")?;
    let node_version = text_field(cursor, "node-version")?;
    cursor.exact("protocol-revision 7")?;
    cursor.exact("fixture-digest-algorithm forward-v2")?;
    let environment = cursor.value("deployment-environment")?.to_owned();
    let network_id = text_field(cursor, "deployment-network-id")?;
    let genesis_id = text_field(cursor, "deployment-genesis-id")?;
    let target_contract = text_field(cursor, "deployment-target-contract")?;
    Ok(ReportExecutorFacts {
        adapter_name,
        adapter_version,
        node_name,
        node_version,
        environment,
        network_id,
        genesis_id,
        target_contract,
    })
}

fn parse_report_roster(
    cursor: &mut LineCursor<'_>,
) -> Result<BTreeMap<String, [u8; 32]>, NativeV2ImportRefusal> {
    cursor.exact("ceremony-roster begin")?;
    let mut ceremony_digests = BTreeMap::new();
    for ceremony in NATIVE_V2_R7_CEREMONY_ROSTER {
        let value = cursor.value("ceremony")?;
        let (offered, digest) = value.split_once(' ').ok_or_else(|| cursor.refusal())?;
        let digest = decode_digest(digest).ok_or_else(|| cursor.refusal())?;
        if offered != ceremony {
            return Err(cursor.refusal());
        }
        if ceremony_digests
            .insert(offered.to_owned(), digest)
            .is_some()
        {
            return Err(cursor.refusal());
        }
    }
    cursor.exact("ceremony-roster end")?;
    Ok(ceremony_digests)
}

fn parse_report(bytes: &[u8]) -> Result<ReportFacts, NativeV2ImportRefusal> {
    let parsed: Result<ReportFacts, NativeV2ImportRefusal> = (|| {
        let mut cursor = LineCursor::new("RUN-REPORT", bytes)?;
        let suite = parse_report_suite(&mut cursor)?;
        let executor = parse_report_executor(&mut cursor)?;
        let ceremony_digests = parse_report_roster(&mut cursor)?;
        let manifest_hash =
            decode_digest(cursor.value("manifest-sha256")?).ok_or_else(|| cursor.refusal())?;
        cursor.exact("eligible yes")?;
        cursor.done()?;
        Ok(ReportFacts {
            binary_revision: suite.binary_revision,
            intended_tip: suite.intended_tip,
            adapter_name: executor.adapter_name,
            adapter_version: executor.adapter_version,
            node_name: executor.node_name,
            node_version: executor.node_version,
            environment: executor.environment,
            network_id: executor.network_id,
            genesis_id: executor.genesis_id,
            target_contract: executor.target_contract,
            manifest_hash,
            ceremony_digests,
        })
    })();
    parsed.map_err(|_| NativeV2ImportRefusal::RunReportGrammar)
}

fn validate_manifest(
    inputs: &CorpusInputs<'_>,
    expected_hash: &str,
) -> Result<(), NativeV2ImportRefusal> {
    if inputs.files.len() != ARCHIVE_FILES.len() {
        return Err(NativeV2ImportRefusal::CorpusCensus {
            expected: ARCHIVE_FILES.len(),
            actual: inputs.files.len(),
        });
    }
    if inputs
        .files
        .iter()
        .zip(ARCHIVE_FILES)
        .any(|(input, expected)| input.name != expected.name)
    {
        return Err(NativeV2ImportRefusal::CorpusCensus {
            expected: ARCHIVE_FILES.len(),
            actual: inputs.files.len(),
        });
    }
    let expected_hash = decode_digest(expected_hash).ok_or(NativeV2ImportRefusal::ManifestHash)?;
    if tagged::sha256(inputs.manifest) != expected_hash {
        return Err(NativeV2ImportRefusal::ManifestHash);
    }
    if inputs.manifest.contains(&b'\r') {
        return Err(NativeV2ImportRefusal::ManifestGrammar);
    }
    if !inputs.manifest.ends_with(b"\n") {
        return Err(NativeV2ImportRefusal::ManifestGrammar);
    }
    let manifest =
        std::str::from_utf8(inputs.manifest).map_err(|_| NativeV2ImportRefusal::ManifestGrammar)?;
    let lines = manifest
        .strip_suffix('\n')
        .ok_or(NativeV2ImportRefusal::ManifestGrammar)?;
    let mut previous = None;
    let mut observed = BTreeSet::new();
    for (line, input) in lines.split('\n').zip(&inputs.files) {
        let (digest, name) = line
            .split_once("  ")
            .ok_or(NativeV2ImportRefusal::ManifestGrammar)?;
        if line.matches("  ").count() != 1 {
            return Err(NativeV2ImportRefusal::ManifestGrammar);
        }
        if previous.is_some_and(|prior| prior >= name) {
            return Err(NativeV2ImportRefusal::ManifestGrammar);
        }
        if name != input.name {
            return Err(NativeV2ImportRefusal::ManifestGrammar);
        }
        if !observed.insert(name) {
            return Err(NativeV2ImportRefusal::ManifestGrammar);
        }
        previous = Some(name);
        if input.bytes.len() != input.expected_size {
            return Err(NativeV2ImportRefusal::FileSize {
                name: name.to_owned(),
                expected: input.expected_size,
                actual: input.bytes.len(),
            });
        }
        let digest = decode_digest(digest).ok_or(NativeV2ImportRefusal::ManifestGrammar)?;
        if tagged::sha256(input.bytes) != digest {
            return Err(NativeV2ImportRefusal::ManifestDigest {
                name: name.to_owned(),
            });
        }
    }
    if observed.len() != ARCHIVE_FILES.len() {
        return Err(NativeV2ImportRefusal::CorpusCensus {
            expected: ARCHIVE_FILES.len(),
            actual: lines.lines().count(),
        });
    }
    if lines.lines().count() != ARCHIVE_FILES.len() {
        return Err(NativeV2ImportRefusal::CorpusCensus {
            expected: ARCHIVE_FILES.len(),
            actual: lines.lines().count(),
        });
    }
    Ok(())
}

fn hex_bytes(bytes: &[u8]) -> String {
    let mut rendered = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(&mut rendered, "{byte:02x}");
    }
    rendered
}

fn parse_role(text: &str) -> Option<OperationRole> {
    match text {
        "auxiliary" => Some(OperationRole::Auxiliary),
        "acceptance" => Some(OperationRole::Acceptance),
        "control" => Some(OperationRole::Control),
        "refusal" => Some(OperationRole::Refusal),
        "paired-explicit" => Some(OperationRole::Paired(LivePairMember::Explicit)),
        "paired-private" => Some(OperationRole::Paired(LivePairMember::Private)),
        _ => None,
    }
}

fn parse_layer(text: &str) -> Option<ObservedOutcomeLayer> {
    match text {
        "accepted" => Some(ObservedOutcomeLayer::Accepted),
        "consensus-rejection-before-script" => {
            Some(ObservedOutcomeLayer::ConsensusRejectionBeforeScript)
        }
        "script-path-rejection" => Some(ObservedOutcomeLayer::ScriptPathRejection),
        "key-path-rejection" => Some(ObservedOutcomeLayer::KeyPathRejection),
        _ => None,
    }
}

fn parse_mutant(text: &str) -> Option<LiveMutantKind> {
    match text {
        "confidential-asset-commitment" => Some(LiveMutantKind::ConfidentialAssetCommitment),
        "empty-signature" => Some(LiveMutantKind::EmptySignature),
        "hidden-private-u-output" => Some(LiveMutantKind::HiddenPrivateUOutput),
        "key-path-escape" => Some(LiveMutantKind::KeyPathEscape),
        "malformed-rangeproof" => Some(LiveMutantKind::MalformedRangeproof),
        "malformed-signature" => Some(LiveMutantKind::MalformedSignature),
        "missing-sponsor-authorization" => Some(LiveMutantKind::MissingSponsorAuthorization),
        "no-coordinator" => Some(LiveMutantKind::NoCoordinator),
        "omitted-source" => Some(LiveMutantKind::OmittedSource),
        "output-total-one-above-input" => Some(LiveMutantKind::OutputTotalOneAboveInput),
        "output-total-one-below-input" => Some(LiveMutantKind::OutputTotalOneBelowInput),
        "private-ct-imbalance" => Some(LiveMutantKind::PrivateCtImbalance),
        "private-output-omitted" => Some(LiveMutantKind::PrivateOutputOmitted),
        "two-coordinators" => Some(LiveMutantKind::TwoCoordinators),
        "vault-control-entitlement-or-bare-u-output" => {
            Some(LiveMutantKind::VaultControlEntitlementOrBareUOutput)
        }
        "wrong-explicit-asset" => Some(LiveMutantKind::WrongExplicitAsset),
        "wrong-private-blinding-balance" => Some(LiveMutantKind::WrongPrivateBlindingBalance),
        _ => None,
    }
}

fn parse_txid(text: &str) -> Option<Txid> {
    decode_digest(text)?;
    Txid::from_target_display(text).ok()
}

fn parse_counted_usizes(text: &str) -> Option<Vec<usize>> {
    let (count, values) = text.split_once(':')?;
    let count = parse_usize(count)?;
    let parsed = if values.is_empty() {
        Vec::new()
    } else {
        values
            .split(',')
            .map(parse_usize)
            .collect::<Option<Vec<_>>>()?
    };
    (parsed.len() == count).then_some(parsed)
}

fn parse_counted_programs(text: &str) -> Option<Vec<Vec<u8>>> {
    let (count, values) = text.split_once(':')?;
    let count = parse_usize(count)?;
    let parsed = if values.is_empty() {
        Vec::new()
    } else {
        values
            .split(',')
            .map(|entry| {
                let (length, encoded) = entry.split_once(':')?;
                let bytes = decode_hex(encoded)?;
                (parse_usize(length)? == bytes.len()).then_some(bytes)
            })
            .collect::<Option<Vec<_>>>()?
    };
    (parsed.len() == count).then_some(parsed)
}

fn parse_locator(text: &str) -> Option<LiveMutationLocator> {
    let fields = text.split(' ').collect::<Vec<_>>();
    match fields.as_slice() {
        ["serialized-output-field", output, field] => {
            let field = match *field {
                "value-commitment" => SerializedOutputField::ValueCommitment,
                "rangeproof-bytes" => SerializedOutputField::RangeproofBytes,
                _ => return None,
            };
            Some(LiveMutationLocator::SerializedOutputField(
                SerializedFieldLocator::new(parse_usize(output)?, field),
            ))
        }
        ["witness-item", input, item] => Some(LiveMutationLocator::WitnessItem {
            input_index: parse_usize(input)?,
            item_index: parse_usize(item)?,
        }),
        ["witnessless-range", start, end] => Some(LiveMutationLocator::WitnesslessRange {
            start: parse_usize(start)?,
            end: parse_usize(end)?,
        }),
        [
            "transaction-shape",
            control_inputs,
            mutant_inputs,
            control_outputs,
            mutant_outputs,
        ] => Some(LiveMutationLocator::TransactionShape {
            control_inputs: parse_usize(control_inputs)?,
            mutant_inputs: parse_usize(mutant_inputs)?,
            control_outputs: parse_usize(control_outputs)?,
            mutant_outputs: parse_usize(mutant_outputs)?,
        }),
        [
            "witness-path-shape",
            "input",
            input,
            "control-stack-count",
            control_count,
            "mutant-stack-count",
            mutant_count,
            "changed-count",
            changed_count,
            "changed",
            changed,
            "control-role",
            control_role,
            "mutant-role",
            mutant_role,
            "witnessless-equal",
            witnessless_equal,
        ] => {
            let changed_positions = if *changed == "none" {
                Vec::new()
            } else {
                changed
                    .split(',')
                    .map(parse_usize)
                    .collect::<Option<Vec<_>>>()?
            };
            if changed_positions.len() != parse_usize(changed_count)? {
                return None;
            }
            let role = |value| match value {
                "key-path" => Some(LiveWitnessPathRole::KeyPath),
                "script-path" => Some(LiveWitnessPathRole::ScriptPath),
                _ => None,
            };
            Some(LiveMutationLocator::WitnessPathShape {
                input_index: parse_usize(input)?,
                control_stack_items: parse_usize(control_count)?,
                mutant_stack_items: parse_usize(mutant_count)?,
                changed_positions,
                control_role: role(control_role)?,
                mutant_role: role(mutant_role)?,
                witnessless_serialization_equal: *witnessless_equal == "true",
            })
        }
        [
            "committed-leaf-arrangement",
            "inputs",
            inputs,
            "control-coordinators",
            control_coordinators,
            "mutant-coordinators",
            mutant_coordinators,
            "control-programs",
            control_programs,
            "mutant-programs",
            mutant_programs,
        ] => Some(LiveMutationLocator::CommittedLeafArrangement {
            input_indices: parse_counted_usizes(inputs)?,
            control_coordinator_leaf_indices: parse_counted_usizes(control_coordinators)?,
            mutant_coordinator_leaf_indices: parse_counted_usizes(mutant_coordinators)?,
            control_committed_leaf_programs: parse_counted_programs(control_programs)?,
            mutant_committed_leaf_programs: parse_counted_programs(mutant_programs)?,
        }),
        _ => None,
    }
}

fn parse_projection(
    cursor: &mut LineCursor<'_>,
) -> Result<Option<ParsedProjection>, NativeV2ImportRefusal> {
    let opening = cursor.value("projection-input")?;
    if opening == "none" {
        return Ok(None);
    }
    if opening != "begin" {
        return Err(cursor.refusal());
    }
    let owner_count = cursor.number("projection-input-owner-count")?;
    let mut owners = Vec::with_capacity(owner_count);
    for index in 0..owner_count {
        let value = cursor.value("projection-input-owner")?;
        let (offered, carrier) = value.split_once(' ').ok_or_else(|| cursor.refusal())?;
        if parse_usize(offered) != Some(index) {
            return Err(cursor.refusal());
        }
        owners.push(hex_bytes(
            &parse_len_hex(carrier).ok_or_else(|| cursor.refusal())?,
        ));
    }
    let amount_count = cursor.number("projection-input-amount-count")?;
    let mut amounts = Vec::with_capacity(amount_count);
    for index in 0..amount_count {
        let value = cursor.value("projection-input-amount")?;
        let (offered, amount) = value.split_once(' ').ok_or_else(|| cursor.refusal())?;
        let amount = parse_u64(amount).ok_or_else(|| cursor.refusal())?;
        if parse_usize(offered) != Some(index) {
            return Err(cursor.refusal());
        }
        amounts.push(amount);
    }
    let destination_count = cursor.number("projection-destination-count")?;
    let mut programs = BTreeMap::new();
    let mut destination_amounts = BTreeMap::new();
    for index in 0..destination_count {
        let value = cursor.value("projection-destination")?;
        let (offered, rest) = value.split_once(' ').ok_or_else(|| cursor.refusal())?;
        let (length, rest) = rest.split_once(' ').ok_or_else(|| cursor.refusal())?;
        let (program, amount) = rest.split_once(' ').ok_or_else(|| cursor.refusal())?;
        let program = decode_hex(program).ok_or_else(|| cursor.refusal())?;
        let amount = parse_u64(amount).ok_or_else(|| cursor.refusal())?;
        if parse_usize(offered) != Some(index) {
            return Err(cursor.refusal());
        }
        if parse_usize(length) != Some(program.len()) {
            return Err(cursor.refusal());
        }
        let owner = format!("destination-{index}");
        programs.insert(owner.clone(), program);
        destination_amounts.insert(owner, amount);
    }
    cursor.exact("projection-input end")?;
    if owners.len() != amounts.len() {
        return Err(cursor.refusal());
    }
    Ok(Some(ParsedProjection {
        input_owners: owners,
        semantic_input_amounts: amounts,
        destination_programs: programs,
        semantic_destination_amounts: destination_amounts,
    }))
}

fn parse_optional_text(value: &str) -> Result<Option<String>, ()> {
    if value == "none" {
        Ok(None)
    } else {
        let bytes = parse_len_hex(value).ok_or(())?;
        Ok(Some(String::from_utf8(bytes).map_err(|_| ())?))
    }
}

struct ParsedResponse {
    response_id: String,
    verdict: &'static str,
    layer: ObservedOutcomeLayer,
    accepted_identity: Option<Txid>,
    detail: String,
}

struct ParsedAttribution {
    control_request_id: Option<String>,
    control_identity: Option<Txid>,
    mutant: Option<LiveMutantKind>,
    locator: Option<LiveMutationLocator>,
    projection: Option<ParsedProjection>,
}

fn parse_operation_response(
    cursor: &mut LineCursor<'_>,
    ceremony: &str,
    operation_id: &str,
    request_id: &str,
) -> Result<ParsedResponse, NativeV2ImportRefusal> {
    let response_id = text_field(cursor, "response-id")?;
    let response_request_id = text_field(cursor, "response-request-id")?;
    let response_operation_id = text_field(cursor, "response-operation-id")?;
    if response_request_id != request_id {
        return Err(NativeV2ImportRefusal::IdentifierLink {
            ceremony: ceremony.to_owned(),
            request: request_id.to_owned(),
        });
    }
    if response_operation_id != operation_id {
        return Err(NativeV2ImportRefusal::IdentifierLink {
            ceremony: ceremony.to_owned(),
            request: request_id.to_owned(),
        });
    }
    let verdict = match cursor.value("response-verdict")? {
        "accepted" => "accepted",
        "refused" => "refused",
        _ => return Err(cursor.refusal()),
    };
    let layer = parse_layer(cursor.value("response-layer")?).ok_or_else(|| cursor.refusal())?;
    if (verdict == "accepted") != (layer == ObservedOutcomeLayer::Accepted) {
        return Err(cursor.refusal());
    }
    let identity_text = cursor.value("response-target-identity")?;
    let accepted_identity = if identity_text == "none" {
        None
    } else {
        Some(parse_txid(identity_text).ok_or_else(|| cursor.refusal())?)
    };
    let detail = text_field(cursor, "response-detail")?;
    Ok(ParsedResponse {
        response_id,
        verdict,
        layer,
        accepted_identity,
        detail,
    })
}

fn parse_operation_attribution(
    cursor: &mut LineCursor<'_>,
) -> Result<ParsedAttribution, NativeV2ImportRefusal> {
    let control_request_id = parse_optional_text(cursor.value("attribution-control-request-id")?)
        .map_err(|()| cursor.refusal())?;
    let control_identity_text = cursor.value("attribution-control-identity")?;
    let control_identity = if control_identity_text == "none" {
        None
    } else {
        Some(parse_txid(control_identity_text).ok_or_else(|| cursor.refusal())?)
    };
    let mutant_text = cursor.value("mutation-kind")?;
    let mutant = if mutant_text == "none" {
        None
    } else {
        Some(parse_mutant(mutant_text).ok_or_else(|| cursor.refusal())?)
    };
    let locator_text = cursor.value("mutation-locator")?;
    let locator = if locator_text == "none" {
        None
    } else {
        Some(parse_locator(locator_text).ok_or_else(|| cursor.refusal())?)
    };
    Ok(ParsedAttribution {
        control_request_id,
        control_identity,
        mutant,
        locator,
        projection: parse_projection(cursor)?,
    })
}

fn operation_role_is_valid(
    role: OperationRole,
    response: &ParsedResponse,
    attribution: &ParsedAttribution,
) -> bool {
    let attribution_none = attribution.control_request_id.is_none()
        && attribution.control_identity.is_none()
        && attribution.mutant.is_none()
        && attribution.locator.is_none()
        && attribution.projection.is_none();
    let semantic_none = response.accepted_identity.is_none() && attribution_none;
    let role_matches = match role {
        OperationRole::Auxiliary => {
            attribution_none
                && (response.verdict == "accepted" || response.accepted_identity.is_none())
        }
        OperationRole::Acceptance | OperationRole::Control => {
            response.verdict == "accepted"
                && response.accepted_identity.is_some()
                && response.detail.is_empty()
                && attribution_none
        }
        OperationRole::Refusal => {
            response.verdict == "refused"
                && response.accepted_identity.is_none()
                && !response.detail.is_empty()
                && attribution.control_request_id.is_some()
                && attribution.control_identity.is_some()
                && attribution.mutant.is_some()
                && attribution.locator.is_some()
                && attribution.projection.is_none()
        }
        OperationRole::Paired(_) => {
            response.verdict == "accepted"
                && response.accepted_identity.is_some()
                && response.detail.is_empty()
                && attribution.control_request_id.is_none()
                && attribution.control_identity.is_none()
                && attribution.mutant.is_none()
                && attribution.locator.is_none()
                && attribution.projection.is_some()
        }
    };
    role_matches && (role == OperationRole::Auxiliary || !semantic_none)
}

fn parse_operation(
    cursor: &mut LineCursor<'_>,
    ordinal: usize,
    ceremony: &str,
) -> Result<ParsedOperation, NativeV2ImportRefusal> {
    cursor.exact(&format!("operation {ordinal} begin"))?;
    let operation_id =
        String::from_utf8(cursor.len_hex("operation-id")?).map_err(|_| cursor.refusal())?;
    let request_id =
        String::from_utf8(cursor.len_hex("request-id")?).map_err(|_| cursor.refusal())?;
    let role = parse_role(cursor.value("request-role")?).ok_or_else(|| cursor.refusal())?;
    let request_bytes = cursor.len_hex("request-bytes")?;
    let response = parse_operation_response(cursor, ceremony, &operation_id, &request_id)?;
    let attribution = parse_operation_attribution(cursor)?;
    cursor.exact(&format!("operation {ordinal} end"))?;
    if !operation_role_is_valid(role, &response, &attribution) {
        return Err(cursor.refusal());
    }
    Ok(ParsedOperation {
        operation_id,
        request_id,
        response_id: response.response_id,
        role,
        request_bytes,
        layer: response.layer,
        accepted_identity: response.accepted_identity,
        detail: response.detail,
        control_request_id: attribution.control_request_id,
        control_identity: attribution.control_identity,
        mutant: attribution.mutant,
        locator: attribution.locator,
        projection: attribution.projection,
    })
}

fn text_field(cursor: &mut LineCursor<'_>, field: &str) -> Result<String, NativeV2ImportRefusal> {
    String::from_utf8(cursor.len_hex(field)?).map_err(|_| cursor.refusal())
}

fn require_binding(
    ceremony: &str,
    field: &'static str,
    agrees: bool,
) -> Result<(), NativeV2ImportRefusal> {
    if agrees {
        Ok(())
    } else {
        Err(NativeV2ImportRefusal::CrossFileBinding {
            ceremony: ceremony.to_owned(),
            field,
        })
    }
}

struct CaptureHeader {
    ceremony: String,
    digest_algorithm: String,
    environment: String,
    network_id: String,
    genesis_id: String,
    target_contract: String,
    protocol_revision: usize,
    adapter_name: String,
    adapter_version: String,
    node_name: String,
    node_version: String,
    intended_tip: String,
}

fn parse_capture_topics(cursor: &mut LineCursor<'_>) -> Result<(), NativeV2ImportRefusal> {
    let topic_count = cursor.number("handshake-topic-count")?;
    for index in 0..topic_count {
        let value = cursor.value("handshake-topic")?;
        let (offered, carrier) = value.split_once(' ').ok_or_else(|| cursor.refusal())?;
        if parse_usize(offered) != Some(index) {
            return Err(cursor.refusal());
        }
        if parse_len_hex(carrier)
            .ok_or_else(|| cursor.refusal())?
            .is_empty()
        {
            return Err(cursor.refusal());
        }
    }
    Ok(())
}

fn parse_capture_environment(
    cursor: &mut LineCursor<'_>,
    ceremony: &str,
    network_id: &str,
    genesis_id: &str,
) -> Result<(), NativeV2ImportRefusal> {
    cursor.exact("environment-schema 7")?;
    cursor.exact("environment-chain 15 656c656d656e747372656774657374")?;
    if text_field(cursor, "environment-network-id")? != network_id {
        return Err(NativeV2ImportRefusal::CrossFileBinding {
            ceremony: ceremony.to_owned(),
            field: "observed-environment",
        });
    }
    if text_field(cursor, "environment-genesis-id")? != genesis_id {
        return Err(NativeV2ImportRefusal::CrossFileBinding {
            ceremony: ceremony.to_owned(),
            field: "observed-environment",
        });
    }
    for line in [
        "environment-domain-count 1",
        "environment-domain 0 tapscript supported true active true",
        "environment-leaf-count 1",
        "environment-leaf 0 196 supported true active true",
        "environment-capability-count 13",
        "environment-capability 0 failure-class-reporting",
        "environment-capability 1 transaction-context",
        "environment-capability 2 resource-observation",
        "environment-capability 3 tree-materialization",
        "environment-capability 4 confidential-conservation",
        "environment-capability 5 owner-authorized-normalization",
        "environment-capability 6 compound-prototype-fixtures",
        "environment-capability 7 fresh-process-lifecycle",
        "environment-capability 8 test-funding-ceremony",
        "environment-capability 9 target-transaction-submission",
        "environment-capability 10 test-sponsor-authorization",
        "environment-capability 11 confidential-value-test-funding",
        "environment-capability 12 confidential-value-sponsor-authorization",
        "environment-funding-count 4",
        "environment-funding 0 representation explicit-asset-confidential-value",
        "environment-funding 1 custody central-public-fixtures",
        "environment-funding 2 materializer guide-ctf-deterministic-v1",
        "environment-funding 3 reproducibility byte_identity",
    ] {
        cursor.exact(line)?;
    }
    Ok(())
}

fn parse_capture_header(
    cursor: &mut LineCursor<'_>,
    name: &str,
    report: &ReportFacts,
) -> Result<CaptureHeader, NativeV2ImportRefusal> {
    cursor.exact("native-capture-schema 1")?;
    let ceremony = cursor.value("ceremony-id")?.to_owned();
    if !NATIVE_V2_R7_CEREMONY_ROSTER.contains(&ceremony.as_str()) {
        return Err(cursor.refusal());
    }
    if name
        != format!(
            "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.{ceremony}.capture"
        )
    {
        return Err(cursor.refusal());
    }
    if text_field(cursor, "rust-test-name")?.is_empty() {
        return Err(cursor.refusal());
    }
    let run_address = cursor.value("run-address")?.to_owned();
    let semantic_identity = cursor.value("architecture-semantic-identity")?.to_owned();
    let behavioural_identity = cursor
        .value("architecture-behavioural-identity")?
        .to_owned();
    let digest_algorithm = cursor.value("fixture-digest-algorithm")?.to_owned();
    require_binding(
        &ceremony,
        "run-address",
        run_address == NATIVE_V2_R7_INPUT_SET_ADDRESS,
    )?;
    require_binding(
        &ceremony,
        "architecture-semantic-identity",
        decode_digest(&semantic_identity).is_some(),
    )?;
    require_binding(
        &ceremony,
        "architecture-behavioural-identity",
        decode_digest(&behavioural_identity).is_some(),
    )?;
    require_binding(
        &ceremony,
        "fixture-digest-algorithm",
        digest_algorithm == "forward-v2",
    )?;
    cursor.exact("run-id-input begin")?;
    let environment = cursor.value("deployment-environment")?.to_owned();
    let network_id = text_field(cursor, "deployment-network-id")?;
    let genesis_id = text_field(cursor, "deployment-genesis-id")?;
    let target_contract = text_field(cursor, "deployment-target-contract")?;
    let protocol_revision = cursor.number("handshake-protocol-schema")?;
    let adapter_name = text_field(cursor, "handshake-adapter-name")?;
    let adapter_version = text_field(cursor, "handshake-adapter-version")?;
    let framework_revision = text_field(cursor, "handshake-framework-revision")?;
    let node_name = text_field(cursor, "handshake-node-name")?;
    let node_version = text_field(cursor, "handshake-node-version")?;
    let binary_revision = text_field(cursor, "handshake-binary-reported-revision")?;
    let intended_tip = text_field(cursor, "handshake-intended-executed-tip")?;
    let _upstream_base = cursor.len_hex("handshake-upstream-base")?;
    parse_capture_topics(cursor)?;
    require_binding(&ceremony, "protocol-revision", protocol_revision == 7)?;
    for (field, agrees) in [
        ("deployment-environment", environment == report.environment),
        ("deployment-network-id", network_id == report.network_id),
        ("deployment-genesis-id", genesis_id == report.genesis_id),
        (
            "deployment-target-contract",
            target_contract == report.target_contract,
        ),
        ("executor-adapter-name", adapter_name == report.adapter_name),
        (
            "executor-adapter-version",
            adapter_version == report.adapter_version,
        ),
        ("node-name", node_name == report.node_name),
        ("node-version", node_version == report.node_version),
        (
            "binary-reported-revision",
            binary_revision == report.binary_revision,
        ),
        ("intended-executed-tip", intended_tip == report.intended_tip),
        (
            "framework-revision",
            framework_revision == report.intended_tip,
        ),
    ] {
        require_binding(&ceremony, field, agrees)?;
    }
    parse_capture_environment(cursor, &ceremony, &network_id, &genesis_id)?;
    Ok(CaptureHeader {
        ceremony,
        digest_algorithm,
        environment,
        network_id,
        genesis_id,
        target_contract,
        protocol_revision,
        adapter_name,
        adapter_version,
        node_name,
        node_version,
        intended_tip,
    })
}

fn parse_capture_digests(
    cursor: &mut LineCursor<'_>,
    digest_algorithm: &str,
) -> Result<Vec<(String, String)>, NativeV2ImportRefusal> {
    let digest_count = cursor.number("digest-count")?;
    let mut digests = Vec::with_capacity(digest_count);
    let mut digest_names = BTreeSet::new();
    for index in 0..digest_count {
        let value = cursor.value("digest")?;
        let fields = value.split(' ').collect::<Vec<_>>();
        let [offered, digest_name, algorithm, digest] = fields.as_slice() else {
            return Err(cursor.refusal());
        };
        if parse_usize(offered) != Some(index) {
            return Err(cursor.refusal());
        }
        if *algorithm != digest_algorithm {
            return Err(cursor.refusal());
        }
        if !digest_names.insert(*digest_name) {
            return Err(cursor.refusal());
        }
        if decode_digest(digest).is_none() {
            return Err(cursor.refusal());
        }
        digests.push(((*digest_name).to_owned(), (*digest).to_owned()));
    }
    Ok(digests)
}

fn parse_capture_tail(
    cursor: &mut LineCursor<'_>,
    bytes: &[u8],
    ceremony: &str,
) -> Result<[u8; 32], NativeV2ImportRefusal> {
    let _legacy = cursor.len_hex("legacy-rendering")?;
    cursor.exact("terminal-state complete")?;
    cursor.exact("run-id-input end")?;
    let prefix_length = cursor.lines[..cursor.index]
        .iter()
        .map(|line| line.len() + 1)
        .sum::<usize>();
    let content_sha256 =
        decode_digest(cursor.value("capture-content-sha256")?).ok_or_else(|| cursor.refusal())?;
    if tagged::sha256(&bytes[..prefix_length]) != content_sha256 {
        return Err(NativeV2ImportRefusal::CaptureContentHash {
            ceremony: ceremony.to_owned(),
        });
    }
    cursor.exact(&format!("native-capture-end {ceremony}"))?;
    cursor.done()?;
    Ok(content_sha256)
}

fn render_run_archive(header: &CaptureHeader, digests: &[(String, String)]) -> Vec<u8> {
    let CaptureHeader {
        ceremony,
        digest_algorithm,
        environment,
        network_id,
        genesis_id,
        target_contract,
        protocol_revision,
        adapter_name,
        adapter_version,
        node_name,
        node_version,
        intended_tip,
    } = header;
    let digest_count = digests.len();
    let mut run_archive = String::new();
    let _ = writeln!(&mut run_archive, "ceremony_id {ceremony}");
    let _ = writeln!(&mut run_archive, "protocol_revision {protocol_revision}");
    let _ = writeln!(&mut run_archive, "deployment_environment {environment}");
    let _ = writeln!(&mut run_archive, "deployment_network {network_id}");
    let _ = writeln!(&mut run_archive, "deployment_genesis {genesis_id}");
    let _ = writeln!(&mut run_archive, "deployment_target {target_contract}");
    let _ = writeln!(&mut run_archive, "executor_adapter {adapter_name}");
    let _ = writeln!(
        &mut run_archive,
        "executor_adapter_version {adapter_version}"
    );
    let _ = writeln!(&mut run_archive, "executor_node {node_name}");
    let _ = writeln!(&mut run_archive, "executor_node_version {node_version}");
    let _ = writeln!(&mut run_archive, "executor_source_tip {intended_tip}");
    let _ = writeln!(&mut run_archive, "digest_count {digest_count}");
    for (digest_name, digest) in digests {
        let _ = writeln!(
            &mut run_archive,
            "digest {digest_name} {digest_algorithm} {digest}"
        );
    }
    run_archive.into_bytes()
}

fn parse_transcript(
    name: &str,
    bytes: &[u8],
    report: &ReportFacts,
) -> Result<ParsedTranscript, NativeV2ImportRefusal> {
    let mut cursor = LineCursor::new(name, bytes)?;
    let header = parse_capture_header(&mut cursor, name, report)?;
    let digests = parse_capture_digests(&mut cursor, &header.digest_algorithm)?;
    let operation_count = cursor.number("operation-count")?;
    let mut operations = Vec::with_capacity(operation_count);
    for ordinal in 0..operation_count {
        operations.push(parse_operation(&mut cursor, ordinal, &header.ceremony)?);
    }
    let content_sha256 = parse_capture_tail(&mut cursor, bytes, &header.ceremony)?;
    Ok(ParsedTranscript {
        summary: NativeV2Transcript {
            ceremony: header.ceremony.clone(),
            operation_count,
            content_sha256,
        },
        run_archive: render_run_archive(&header, &digests),
        operations,
    })
}

fn changed_range(left: &[u8], right: &[u8]) -> (usize, usize) {
    let prefix = left.iter().zip(right).take_while(|(a, b)| a == b).count();
    let suffix = left
        .iter()
        .rev()
        .zip(right.iter().rev())
        .take_while(|(a, b)| a == b)
        .count()
        .min(left.len().saturating_sub(prefix))
        .min(right.len().saturating_sub(prefix));
    (prefix, left.len().saturating_sub(suffix))
}

fn witness_item_is_exact(
    control: &TargetTransaction,
    mutant: &TargetTransaction,
    input_index: usize,
    item_index: usize,
) -> bool {
    if control.version() != mutant.version() {
        return false;
    }
    if control.inputs() != mutant.inputs() {
        return false;
    }
    if control.outputs() != mutant.outputs() {
        return false;
    }
    if control.lock_time() != mutant.lock_time() {
        return false;
    }
    if control.output_witnesses() != mutant.output_witnesses() {
        return false;
    }
    if control.witnesses().len() != mutant.witnesses().len() {
        return false;
    }
    if control.encode_without_witness() != mutant.encode_without_witness() {
        return false;
    }
    let mut differences = 0;
    for (offered_input, pair) in control
        .witnesses()
        .iter()
        .zip(mutant.witnesses())
        .enumerate()
    {
        if pair.0.stack().len() != pair.1.stack().len() {
            return false;
        }
        for (offered_item, values) in pair.0.stack().iter().zip(pair.1.stack()).enumerate() {
            if values.0 != values.1 {
                if offered_input != input_index {
                    return false;
                }
                if offered_item != item_index {
                    return false;
                }
                differences += 1;
            }
        }
    }
    differences == 1
}

const CONTROL_BLOCK_BASE_BYTES: usize = 33;
const CONTROL_BLOCK_DIGEST_BYTES: usize = 32;
const TAPSCRIPT_LEAF_VERSION: u8 = 0xc0;

fn decoded_witness_path_role(stack: &[Vec<u8>]) -> Option<LiveWitnessPathRole> {
    if stack.len() == 1 && !stack[0].is_empty() {
        return Some(LiveWitnessPathRole::KeyPath);
    }
    let (control_block, preceding) = stack.split_last()?;
    let leaf_program = preceding.last()?;
    let valid_control = control_block.len() >= CONTROL_BLOCK_BASE_BYTES
        && (control_block.len() - CONTROL_BLOCK_BASE_BYTES)
            .is_multiple_of(CONTROL_BLOCK_DIGEST_BYTES)
        && (control_block.len() - CONTROL_BLOCK_BASE_BYTES) / CONTROL_BLOCK_DIGEST_BYTES <= 128
        && control_block[0] & 0xfe == TAPSCRIPT_LEAF_VERSION;
    (!leaf_program.is_empty() && valid_control).then_some(LiveWitnessPathRole::ScriptPath)
}

fn exact_changed_positions(control: &[Vec<u8>], mutant: &[Vec<u8>]) -> Vec<usize> {
    (0..control.len().max(mutant.len()))
        .filter(|position| control.get(*position) != mutant.get(*position))
        .collect()
}

fn witness_path_matches(
    control: &TargetTransaction,
    mutant: &TargetTransaction,
    locator: &LiveMutationLocator,
) -> bool {
    let LiveMutationLocator::WitnessPathShape {
        input_index,
        control_stack_items,
        mutant_stack_items,
        changed_positions,
        control_role,
        mutant_role,
        witnessless_serialization_equal,
    } = locator
    else {
        return false;
    };
    let Some(control_witness) = control.witnesses().get(*input_index) else {
        return false;
    };
    let Some(mutant_witness) = mutant.witnesses().get(*input_index) else {
        return false;
    };
    let control_stack = control_witness.stack();
    let mutant_stack = mutant_witness.stack();
    *witnessless_serialization_equal
        && control.encode_without_witness() == mutant.encode_without_witness()
        && control.output_witnesses() == mutant.output_witnesses()
        && control.witnesses().len() == mutant.witnesses().len()
        && control
            .witnesses()
            .iter()
            .zip(mutant.witnesses())
            .enumerate()
            .all(|(index, pair)| index == *input_index || pair.0 == pair.1)
        && control_stack.len() == *control_stack_items
        && mutant_stack.len() == *mutant_stack_items
        && decoded_witness_path_role(control_stack) == Some(*control_role)
        && decoded_witness_path_role(mutant_stack) == Some(*mutant_role)
        && exact_changed_positions(control_stack, mutant_stack).as_slice()
            == changed_positions.as_slice()
        && !changed_positions.is_empty()
        && changed_positions.windows(2).all(|pair| pair[0] < pair[1])
}

fn revealed_leaf(transaction: &TargetTransaction, input_index: usize) -> Option<Vec<u8>> {
    let stack = transaction.witnesses().get(input_index)?.stack();
    if decoded_witness_path_role(stack) != Some(LiveWitnessPathRole::ScriptPath) {
        return None;
    }
    stack.get(stack.len().checked_sub(2)?).cloned()
}

fn committed_leaves_match(
    control: &TargetTransaction,
    mutant: &TargetTransaction,
    mutant_kind: LiveMutantKind,
    locator: &LiveMutationLocator,
) -> bool {
    let LiveMutationLocator::CommittedLeafArrangement {
        input_indices,
        control_coordinator_leaf_indices,
        mutant_coordinator_leaf_indices,
        control_committed_leaf_programs,
        mutant_committed_leaf_programs,
    } = locator
    else {
        return false;
    };
    if input_indices.is_empty() {
        return false;
    }
    if input_indices.windows(2).any(|pair| pair[0] >= pair[1]) {
        return false;
    }
    if control.encode_without_witness() != mutant.encode_without_witness() {
        return false;
    }
    if control.output_witnesses() != mutant.output_witnesses() {
        return false;
    }
    if control.witnesses().len() != mutant.witnesses().len() {
        return false;
    }
    let Some(control_programs) = input_indices
        .iter()
        .map(|index| revealed_leaf(control, *index))
        .collect::<Option<Vec<_>>>()
    else {
        return false;
    };
    let Some(mutant_programs) = input_indices
        .iter()
        .map(|index| revealed_leaf(mutant, *index))
        .collect::<Option<Vec<_>>>()
    else {
        return false;
    };
    if control_programs.as_slice() != control_committed_leaf_programs.as_slice() {
        return false;
    }
    if mutant_programs.as_slice() != mutant_committed_leaf_programs.as_slice() {
        return false;
    }
    if control_programs == mutant_programs {
        return false;
    }
    let coordinator = &control_programs[0];
    let control_indices = input_indices
        .iter()
        .zip(&control_programs)
        .filter_map(|(index, program)| (program == coordinator).then_some(*index))
        .collect::<Vec<_>>();
    let mutant_indices = input_indices
        .iter()
        .zip(&mutant_programs)
        .filter_map(|(index, program)| (program == coordinator).then_some(*index))
        .collect::<Vec<_>>();
    let expected = match mutant_kind {
        LiveMutantKind::TwoCoordinators => 2,
        LiveMutantKind::NoCoordinator => 0,
        _ => return false,
    };
    control_indices.as_slice() == control_coordinator_leaf_indices.as_slice()
        && control_indices.as_slice() == [input_indices[0]]
        && mutant_indices.as_slice() == mutant_coordinator_leaf_indices.as_slice()
        && mutant_indices.len() == expected
}

fn locator_matches(
    control: &TargetTransaction,
    mutant: &TargetTransaction,
    mutant_kind: LiveMutantKind,
    locator: &LiveMutationLocator,
) -> bool {
    match locator {
        LiveMutationLocator::SerializedOutputField(locator) => {
            let Ok(control_field) = control.locate_serialized_field(*locator) else {
                return false;
            };
            let Ok(mutant_field) = mutant.locate_serialized_field(*locator) else {
                return false;
            };
            let (start, end) = changed_range(&control.encode(), &mutant.encode());
            start < end
                && start >= control_field.range().start
                && end <= control_field.range().end
                && start >= mutant_field.range().start
                && end <= mutant_field.range().end
        }
        LiveMutationLocator::WitnessItem {
            input_index,
            item_index,
        } => witness_item_is_exact(control, mutant, *input_index, *item_index),
        LiveMutationLocator::WitnesslessRange { start, end } => {
            *start < *end
                && changed_range(
                    &control.encode_without_witness(),
                    &mutant.encode_without_witness(),
                ) == (*start, *end)
        }
        LiveMutationLocator::TransactionShape {
            control_inputs,
            mutant_inputs,
            control_outputs,
            mutant_outputs,
        } => {
            control.inputs().len() == *control_inputs
                && mutant.inputs().len() == *mutant_inputs
                && control.outputs().len() == *control_outputs
                && mutant.outputs().len() == *mutant_outputs
                && (control_inputs != mutant_inputs || control_outputs != mutant_outputs)
                && control.version() == mutant.version()
                && control.lock_time() == mutant.lock_time()
        }
        LiveMutationLocator::WitnessPathShape { .. } => {
            witness_path_matches(control, mutant, locator)
        }
        LiveMutationLocator::CommittedLeafArrangement { .. } => {
            committed_leaves_match(control, mutant, mutant_kind, locator)
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RecomputedProjection {
    version: u32,
    lock_time: u32,
    input_owners: BTreeSet<String>,
    semantic_input_amounts: Vec<u64>,
    destinations: Vec<(String, u64)>,
    destination_owners: BTreeSet<String>,
    explicit_asset: Option<[u8; 32]>,
    every_input_authorized: bool,
    sponsor_inputs: usize,
    fee_outputs: usize,
    publishes_every_destination_amount: bool,
    all_outputs_classified: bool,
}

fn recompute_projection(bytes: &[u8], input: &ParsedProjection) -> Option<RecomputedProjection> {
    if input.input_owners.len() != input.semantic_input_amounts.len() {
        return None;
    }
    let decoded = TargetTransaction::decode(bytes).ok()?;
    if decoded.encode() != bytes {
        return None;
    }
    if input.input_owners.len() > decoded.inputs().len() {
        return None;
    }
    let mut semantic_input_amounts = input.semantic_input_amounts.clone();
    semantic_input_amounts.sort_unstable();
    let input_owners = input.input_owners.iter().cloned().collect::<BTreeSet<_>>();
    let mut assets = BTreeSet::new();
    let mut fee_outputs = 0;
    let mut destinations = Vec::new();
    let mut destination_owners = BTreeSet::new();
    let mut publishes_every_destination_amount = true;
    for output in decoded.outputs() {
        if let AssetField::Explicit(asset) = output.asset() {
            assets.insert(*asset.internal());
        }
        if output.is_fee() {
            fee_outputs += 1;
            continue;
        }
        let mut matches = input
            .destination_programs
            .iter()
            .filter(|(_, program)| program.as_slice() == output.program());
        let (owner, _) = matches.next()?;
        if matches.next().is_some() {
            return None;
        }
        if !destination_owners.insert(owner.clone()) {
            return None;
        }
        let amount = *input.semantic_destination_amounts.get(owner)?;
        match output.value() {
            ValueField::Explicit(published) if published == amount => {}
            ValueField::Commitment(_) => publishes_every_destination_amount = false,
            _ => return None,
        }
        destinations.push((owner.clone(), amount));
    }
    if destination_owners.len() != input.destination_programs.len() {
        return None;
    }
    if !destination_owners
        .iter()
        .eq(input.semantic_destination_amounts.keys())
    {
        return None;
    }
    destinations.sort();
    let explicit_asset = if assets.len() == 1 {
        assets.into_iter().next()
    } else {
        None
    };
    let all_outputs_classified = destination_owners.len() + fee_outputs == decoded.outputs().len();
    Some(RecomputedProjection {
        version: decoded.version(),
        lock_time: decoded.lock_time(),
        input_owners,
        semantic_input_amounts,
        destinations,
        destination_owners,
        explicit_asset,
        every_input_authorized: decoded.witnesses().iter().all(|witness| !witness.is_null()),
        sponsor_inputs: decoded.inputs().len() - input.input_owners.len(),
        fee_outputs,
        publishes_every_destination_amount,
        all_outputs_classified,
    })
}

const VALIDATED_PAIR_RELATION: &str = "all-11-terms-equal;withheld-by-private=2";

fn pair_recomputes(explicit: &ParsedOperation, private: &ParsedOperation) -> bool {
    let Some(explicit_input) = explicit.projection.as_ref() else {
        return false;
    };
    let Some(private_input) = private.projection.as_ref() else {
        return false;
    };
    let Some(explicit) = recompute_projection(&explicit.request_bytes, explicit_input) else {
        return false;
    };
    let Some(private) = recompute_projection(&private.request_bytes, private_input) else {
        return false;
    };
    let terms = [
        explicit.semantic_input_amounts == private.semantic_input_amounts,
        explicit.destinations == private.destinations,
        explicit.input_owners == private.input_owners,
        !explicit.destinations.is_empty() && !private.destinations.is_empty(),
        explicit.explicit_asset.is_some() && explicit.explicit_asset == private.explicit_asset,
        explicit.every_input_authorized && private.every_input_authorized,
        explicit.version == private.version,
        explicit.lock_time == private.lock_time,
        explicit.destination_owners == private.destination_owners,
        (explicit.sponsor_inputs, explicit.fee_outputs)
            == (private.sponsor_inputs, private.fee_outputs),
        explicit.all_outputs_classified && private.all_outputs_classified,
    ];
    let withheld = [
        explicit.publishes_every_destination_amount,
        !private.publishes_every_destination_amount,
    ]
    .into_iter()
    .filter(|term| *term)
    .count();
    terms.into_iter().all(|term| term) && withheld == 2
}

struct DecodedTranscript<'a> {
    roles: [usize; 6],
    transactions: BTreeMap<&'a str, TargetTransaction>,
}

fn decode_transcript_operations(
    transcript: &ParsedTranscript,
) -> Result<DecodedTranscript<'_>, NativeV2ImportRefusal> {
    let ceremony = transcript.summary.ceremony();
    let mut operation_ids = BTreeSet::new();
    let mut request_ids = BTreeSet::new();
    let mut response_ids = BTreeSet::new();
    let mut transactions = BTreeMap::new();
    let mut roles = [0; 6];
    for operation in &transcript.operations {
        let role_index = match operation.role {
            OperationRole::Auxiliary => 0,
            OperationRole::Acceptance => 1,
            OperationRole::Control => 2,
            OperationRole::Refusal => 3,
            OperationRole::Paired(LivePairMember::Explicit) => 4,
            OperationRole::Paired(LivePairMember::Private) => 5,
        };
        roles[role_index] += 1;
        if !operation_ids.insert(&operation.operation_id) {
            return Err(NativeV2ImportRefusal::IdentifierLink {
                ceremony: ceremony.to_owned(),
                request: operation.request_id.clone(),
            });
        }
        if !request_ids.insert(&operation.request_id) {
            return Err(NativeV2ImportRefusal::IdentifierLink {
                ceremony: ceremony.to_owned(),
                request: operation.request_id.clone(),
            });
        }
        if !response_ids.insert(&operation.response_id) {
            return Err(NativeV2ImportRefusal::IdentifierLink {
                ceremony: ceremony.to_owned(),
                request: operation.request_id.clone(),
            });
        }
        if operation.request_bytes.is_empty() {
            if operation.role != OperationRole::Auxiliary {
                return Err(NativeV2ImportRefusal::TransactionDecode {
                    ceremony: ceremony.to_owned(),
                    request: operation.request_id.clone(),
                });
            }
            continue;
        }
        let transaction = TargetTransaction::decode(&operation.request_bytes).map_err(|_| {
            NativeV2ImportRefusal::TransactionDecode {
                ceremony: ceremony.to_owned(),
                request: operation.request_id.clone(),
            }
        })?;
        if transaction.encode() != operation.request_bytes {
            return Err(NativeV2ImportRefusal::TransactionDecode {
                ceremony: ceremony.to_owned(),
                request: operation.request_id.clone(),
            });
        }
        if let Some(identity) = operation.accepted_identity {
            let recomputed = txid_for(&operation.request_bytes).map_err(|()| {
                NativeV2ImportRefusal::TransactionDecode {
                    ceremony: ceremony.to_owned(),
                    request: operation.request_id.clone(),
                }
            })?;
            if recomputed != identity {
                return Err(NativeV2ImportRefusal::TransactionIdentity {
                    ceremony: ceremony.to_owned(),
                    request: operation.request_id.clone(),
                });
            }
        }
        transactions.insert(operation.request_id.as_str(), transaction);
    }
    Ok(DecodedTranscript {
        roles,
        transactions,
    })
}

fn validate_refusal_locators(
    transcript: &ParsedTranscript,
    decoded: &DecodedTranscript<'_>,
) -> Result<(), NativeV2ImportRefusal> {
    let ceremony = transcript.summary.ceremony();
    for operation in &transcript.operations {
        if operation.role != OperationRole::Refusal {
            continue;
        }
        let control_request_id = operation.control_request_id.as_deref().ok_or_else(|| {
            NativeV2ImportRefusal::IdentifierLink {
                ceremony: ceremony.to_owned(),
                request: operation.request_id.clone(),
            }
        })?;
        let control = transcript
            .operations
            .iter()
            .find(|candidate| candidate.request_id == control_request_id)
            .ok_or_else(|| NativeV2ImportRefusal::IdentifierLink {
                ceremony: ceremony.to_owned(),
                request: operation.request_id.clone(),
            })?;
        if control.role != OperationRole::Control {
            return Err(NativeV2ImportRefusal::IdentifierLink {
                ceremony: ceremony.to_owned(),
                request: operation.request_id.clone(),
            });
        }
        if control.accepted_identity != operation.control_identity {
            return Err(NativeV2ImportRefusal::IdentifierLink {
                ceremony: ceremony.to_owned(),
                request: operation.request_id.clone(),
            });
        }
        let control_transaction = decoded
            .transactions
            .get(control.request_id.as_str())
            .ok_or_else(|| NativeV2ImportRefusal::TransactionDecode {
                ceremony: ceremony.to_owned(),
                request: control.request_id.clone(),
            })?;
        let mutant_transaction = decoded
            .transactions
            .get(operation.request_id.as_str())
            .ok_or_else(|| NativeV2ImportRefusal::TransactionDecode {
                ceremony: ceremony.to_owned(),
                request: operation.request_id.clone(),
            })?;
        if !locator_matches(
            control_transaction,
            mutant_transaction,
            operation
                .mutant
                .ok_or_else(|| NativeV2ImportRefusal::MutationLocator {
                    ceremony: ceremony.to_owned(),
                    request: operation.request_id.clone(),
                })?,
            operation
                .locator
                .as_ref()
                .ok_or_else(|| NativeV2ImportRefusal::MutationLocator {
                    ceremony: ceremony.to_owned(),
                    request: operation.request_id.clone(),
                })?,
        ) {
            return Err(NativeV2ImportRefusal::MutationLocator {
                ceremony: ceremony.to_owned(),
                request: operation.request_id.clone(),
            });
        }
    }
    Ok(())
}

fn validate_transcript_semantics(
    transcript: &ParsedTranscript,
) -> Result<[usize; 6], NativeV2ImportRefusal> {
    let decoded = decode_transcript_operations(transcript)?;
    validate_refusal_locators(transcript, &decoded)?;
    Ok(decoded.roles)
}

fn row_ceremonies(row: &str) -> Option<&'static [&'static str]> {
    match row {
        "both-commitment-parity-forms" => {
            Some(&["private-restart-control", "private-restart-parity"])
        }
        "candidate-maximum-inputs" => Some(&["explicit-maximum-inputs"]),
        "candidate-maximum-outputs" => Some(&["explicit-maximum-outputs"]),
        "canonical-input-normalization" => Some(&["explicit-normalization", "explicit-merge"]),
        "one-destination-owner" => Some(&["explicit-one-destination-owner"]),
        "one-input-split-into-two" => Some(&["explicit-split"]),
        "one-input-to-one-output" => Some(&["explicit-one-to-one"]),
        "private-many-to-many-representative" => Some(&["multi-many-to-many"]),
        "private-sponsor-values" => Some(&["sponsored-private-with-change"]),
        "private-merge" => Some(&["multi-private-merge"]),
        "private-one-to-one" => Some(&["private-restart-control"]),
        "private-several-distinct-owners" => Some(&["multi-several-owners"]),
        "private-split" => Some(&["multi-split"]),
        "repeated-owner" => Some(&["explicit-repeated-owner"]),
        "semantic-boundary-values" => Some(&["explicit-boundary-values"]),
        "several-destination-owners" => Some(&["explicit-several-destination-owners"]),
        "several-distinct-owners" => Some(&["explicit-several-owners"]),
        "several-inputs-merged-into-one" => Some(&["explicit-merge"]),
        "several-inputs-to-several-outputs" => Some(&["explicit-several-to-several"]),
        "sponsor-change-absent" | "sponsored" => Some(&["sponsored-change-absent"]),
        "sponsor-change-present" => Some(&["sponsored-change-present"]),
        "sponsorless" => Some(&["explicit-sponsorless"]),
        "target-ct-conservation"
        | "malformed-rangeproof"
        | "private-ct-imbalance"
        | "wrong-private-blinding-balance" => Some(&["conservation-negatives"]),
        "empty-signature" | "malformed-signature" => Some(&["explicit-witness-negatives"]),
        "key-path-escape" => Some(&["keypath-probe"]),
        "missing-sponsor-authorization" => Some(&["sponsored-missing-authorization"]),
        "confidential-asset-commitment"
        | "hidden-private-u-output"
        | "no-coordinator"
        | "omitted-source"
        | "output-total-one-above-input"
        | "output-total-one-below-input"
        | "private-output-omitted"
        | "two-coordinators"
        | "vault-control-entitlement-or-bare-u-output"
        | "wrong-explicit-asset" => Some(&["owner-signing-negatives"]),
        "projection-equality-with-paired-explicit" => Some(&["pairs-arc"]),
        _ => None,
    }
}

fn global_request_id(ceremony: &str, request: &str) -> String {
    format!("{ceremony}/{request}")
}

fn operation_response(operation: &ParsedOperation) -> Result<LiveTargetResponse, ()> {
    if let Some(identity) = operation.accepted_identity {
        return Ok(LiveTargetResponse::Accepted { identity });
    }
    Ok(LiveTargetResponse::Refused {
        observed_layer: operation.layer,
        detail: operation.detail.clone(),
        control_identity: operation.control_identity.ok_or(())?,
    })
}

fn build_runs(
    transcripts: &[ParsedTranscript],
) -> Result<(Vec<LiveRunBinding>, BTreeMap<String, String>), NativeV2ImportRefusal> {
    let used_ceremonies = ROW_ROSTER
        .iter()
        .flat_map(|row| row_ceremonies(row).unwrap_or(&[]).iter().copied())
        .collect::<BTreeSet<_>>();
    let mut runs = Vec::new();
    for transcript in transcripts
        .iter()
        .filter(|transcript| used_ceremonies.contains(transcript.summary.ceremony()))
    {
        let ceremony = transcript.summary.ceremony();
        let mut requests = BTreeMap::new();
        let mut facts = BTreeMap::new();
        let mut responses = BTreeMap::new();
        for operation in transcript
            .operations
            .iter()
            .filter(|operation| operation.role != OperationRole::Auxiliary)
        {
            let request_id = global_request_id(ceremony, &operation.request_id);
            requests.insert(request_id.clone(), operation.request_bytes.clone());
            let fact = match operation.role {
                OperationRole::Acceptance => LiveRequestFact::Acceptance,
                OperationRole::Control => LiveRequestFact::Control,
                OperationRole::Refusal => LiveRequestFact::Refusal {
                    mutant: operation.mutant.ok_or(NativeV2ImportRefusal::RunBinding)?,
                    control_request_id: global_request_id(
                        ceremony,
                        operation
                            .control_request_id
                            .as_deref()
                            .ok_or(NativeV2ImportRefusal::RunBinding)?,
                    ),
                    locator: operation
                        .locator
                        .clone()
                        .ok_or(NativeV2ImportRefusal::RunBinding)?,
                },
                OperationRole::Paired(member) => LiveRequestFact::Paired {
                    member,
                    projection: operation
                        .projection
                        .as_ref()
                        .ok_or(NativeV2ImportRefusal::RunBinding)?
                        .live(),
                },
                OperationRole::Auxiliary => return Err(NativeV2ImportRefusal::RunBinding),
            };
            facts.insert(request_id.clone(), fact);
            responses.insert(
                request_id,
                operation_response(operation).map_err(|()| NativeV2ImportRefusal::RunBinding)?,
            );
        }
        let run = LiveRunBinding::from_archive(
            transcript.run_archive.clone(),
            requests,
            facts,
            responses,
        )
        .map_err(|_| NativeV2ImportRefusal::RunBinding)?;
        runs.push(run);
    }
    runs.sort_by(|left, right| left.run_id().cmp(right.run_id()));
    let run_ids = runs
        .iter()
        .filter_map(|run| {
            let line = std::str::from_utf8(run.archive_bytes())
                .ok()?
                .lines()
                .next()?;
            let ceremony = line.strip_prefix("ceremony_id ")?;
            Some((ceremony.to_owned(), run.run_id().to_owned()))
        })
        .collect::<BTreeMap<_, _>>();
    if run_ids.len() != used_ceremonies.len() {
        return Err(NativeV2ImportRefusal::RunBinding);
    }
    Ok((runs, run_ids))
}

fn semantic_operation<'a>(
    transcripts: &'a BTreeMap<&str, &'a ParsedTranscript>,
    ceremony: &str,
    role: OperationRole,
) -> Result<&'a ParsedOperation, NativeV2ImportRefusal> {
    let transcript = transcripts
        .get(ceremony)
        .ok_or(NativeV2ImportRefusal::RowAttribution)?;
    let mut matching = transcript
        .operations
        .iter()
        .filter(|operation| operation.role == role);
    let operation = matching
        .next()
        .ok_or(NativeV2ImportRefusal::RowAttribution)?;
    if matching.next().is_some() {
        return Err(NativeV2ImportRefusal::RowAttribution);
    }
    Ok(operation)
}

fn boundary_for(mutant: LiveMutantKind) -> Option<EvidenceBoundary> {
    required_safety_matrix()
        .into_iter()
        .find(|candidate| candidate.name() == mutant.row())
        .and_then(crate::live_safety::LiveSafetyRow::refusing_layer)
}

fn parity_output_index(
    first: &ParsedOperation,
    second: &ParsedOperation,
) -> Result<usize, NativeV2ImportRefusal> {
    let first = TargetTransaction::decode(&first.request_bytes)
        .map_err(|_| NativeV2ImportRefusal::RowAttribution)?;
    let second = TargetTransaction::decode(&second.request_bytes)
        .map_err(|_| NativeV2ImportRefusal::RowAttribution)?;
    let matching = first
        .outputs()
        .iter()
        .zip(second.outputs())
        .enumerate()
        .filter_map(|(index, pair)| match (pair.0.value(), pair.1.value()) {
            (ValueField::Commitment(left), ValueField::Commitment(right))
                if matches!((left[0], right[0]), (0x08, 0x09) | (0x09, 0x08)) =>
            {
                Some(index)
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    let [index] = matching.as_slice() else {
        return Err(NativeV2ImportRefusal::RowAttribution);
    };
    Ok(*index)
}

fn sponsor_predicates(
    absent: &ParsedOperation,
    present: &ParsedOperation,
) -> Result<BTreeMap<&'static str, LiveRowSemanticPredicate>, NativeV2ImportRefusal> {
    let absent = TargetTransaction::decode(&absent.request_bytes)
        .map_err(|_| NativeV2ImportRefusal::RowAttribution)?;
    let present = TargetTransaction::decode(&present.request_bytes)
        .map_err(|_| NativeV2ImportRefusal::RowAttribution)?;
    let sponsor_inputs = absent
        .witnesses()
        .iter()
        .enumerate()
        .filter_map(|(index, witness)| {
            (decoded_witness_path_role(witness.stack()) == Some(LiveWitnessPathRole::KeyPath))
                .then_some(index)
        })
        .collect::<Vec<_>>();
    let fee_outputs = absent
        .outputs()
        .iter()
        .enumerate()
        .filter_map(|(index, output)| output.is_fee().then_some(index))
        .collect::<Vec<_>>();
    let [sponsor_input_index] = sponsor_inputs.as_slice() else {
        return Err(NativeV2ImportRefusal::RowAttribution);
    };
    let [fee_output_index] = fee_outputs.as_slice() else {
        return Err(NativeV2ImportRefusal::RowAttribution);
    };
    let absent_programs = absent
        .outputs()
        .iter()
        .filter(|output| !output.is_fee())
        .map(transaction::TargetOutput::program)
        .collect::<Vec<_>>();
    let change_programs = present
        .outputs()
        .iter()
        .filter(|output| {
            !output.is_fee()
                && !absent_programs
                    .iter()
                    .any(|program| *program == output.program())
        })
        .map(|output| output.program().to_vec())
        .collect::<Vec<_>>();
    let [change_program] = change_programs.as_slice() else {
        return Err(NativeV2ImportRefusal::RowAttribution);
    };
    if absent
        .outputs()
        .iter()
        .any(|output| output.program() == change_program.as_slice())
    {
        return Err(NativeV2ImportRefusal::RowAttribution);
    }
    Ok(BTreeMap::from([
        (
            "sponsor-change-absent",
            LiveRowSemanticPredicate::SponsorChangeAbsent {
                sponsor_input_index: *sponsor_input_index,
                fee_output_index: *fee_output_index,
                sponsor_change_program: change_program.clone(),
            },
        ),
        (
            "sponsored",
            LiveRowSemanticPredicate::Sponsored {
                sponsor_input_index: *sponsor_input_index,
                fee_output_index: *fee_output_index,
            },
        ),
    ]))
}

type TranscriptIndex<'a> = BTreeMap<&'a str, &'a ParsedTranscript>;
type MaterialParts = (
    Vec<LiveReportObservation>,
    Vec<ProvenNativeV2Link>,
    Vec<NativeV2RowAttribution>,
);

#[derive(Default)]
struct MaterialBuilder {
    observations: Vec<LiveReportObservation>,
    links: Vec<ProvenNativeV2Link>,
}

fn add_parity_material(
    material: &mut MaterialBuilder,
    transcripts: &TranscriptIndex<'_>,
    run_ids: &BTreeMap<String, String>,
) -> Result<(), NativeV2ImportRefusal> {
    let first = semantic_operation(
        transcripts,
        "private-restart-control",
        OperationRole::Acceptance,
    )?;
    let second = semantic_operation(
        transcripts,
        "private-restart-parity",
        OperationRole::Acceptance,
    )?;
    let parity_index = parity_output_index(first, second)?;
    let composite_member = |ceremony: &str,
                            operation: &ParsedOperation|
     -> Result<LiveCompositeAcceptanceMember, NativeV2ImportRefusal> {
        Ok(LiveCompositeAcceptanceMember::new(
            ceremony.to_owned(),
            run_ids
                .get(ceremony)
                .cloned()
                .ok_or(NativeV2ImportRefusal::RowAttribution)?,
            global_request_id(ceremony, &operation.request_id),
            operation
                .accepted_identity
                .ok_or(NativeV2ImportRefusal::RowAttribution)?,
            parity_index,
        ))
    };
    material
        .observations
        .push(LiveReportObservation::CompositeTwoAcceptance {
            row: ROW_ROSTER[0],
            acceptance: LiveCompositeTwoAcceptance::new(
                composite_member("private-restart-control", first)?,
                composite_member("private-restart-parity", second)?,
            ),
        });
    for (ceremony, operation) in [
        ("private-restart-control", first),
        ("private-restart-parity", second),
    ] {
        material.links.push(ProvenNativeV2Link {
            row: ROW_ROSTER[0],
            class: NativeV2LinkClass::Primary,
            ceremony: ceremony.to_owned(),
            run_id: run_ids
                .get(ceremony)
                .cloned()
                .ok_or(NativeV2ImportRefusal::RowAttribution)?,
            request_id: global_request_id(ceremony, &operation.request_id),
        });
    }
    Ok(())
}

fn add_positive_material(
    material: &mut MaterialBuilder,
    transcripts: &TranscriptIndex<'_>,
    run_ids: &BTreeMap<String, String>,
) -> Result<(), NativeV2ImportRefusal> {
    let positive_rows = [
        (ROW_ROSTER[1], "explicit-maximum-inputs"),
        (ROW_ROSTER[2], "explicit-maximum-outputs"),
        (ROW_ROSTER[3], "explicit-normalization"),
        (ROW_ROSTER[4], "explicit-one-destination-owner"),
        (ROW_ROSTER[5], "explicit-split"),
        (ROW_ROSTER[6], "explicit-one-to-one"),
        (ROW_ROSTER[7], "multi-many-to-many"),
        (ROW_ROSTER[8], "sponsored-private-with-change"),
        (ROW_ROSTER[9], "multi-private-merge"),
        (ROW_ROSTER[10], "private-restart-control"),
        (ROW_ROSTER[11], "multi-several-owners"),
        (ROW_ROSTER[12], "multi-split"),
        (ROW_ROSTER[13], "explicit-repeated-owner"),
        (ROW_ROSTER[14], "explicit-boundary-values"),
        (ROW_ROSTER[15], "explicit-several-destination-owners"),
        (ROW_ROSTER[16], "explicit-several-owners"),
        (ROW_ROSTER[17], "explicit-merge"),
        (ROW_ROSTER[18], "explicit-several-to-several"),
        (ROW_ROSTER[20], "sponsored-change-present"),
        (ROW_ROSTER[22], "explicit-sponsorless"),
    ];
    for (row, ceremony) in positive_rows {
        let operation = semantic_operation(transcripts, ceremony, OperationRole::Acceptance)?;
        let run_id = run_ids
            .get(ceremony)
            .cloned()
            .ok_or(NativeV2ImportRefusal::RowAttribution)?;
        let request_id = global_request_id(ceremony, &operation.request_id);
        material
            .observations
            .push(LiveReportObservation::NativeAcceptance {
                row,
                run_id: run_id.clone(),
                request_id: request_id.clone(),
                identity: operation
                    .accepted_identity
                    .ok_or(NativeV2ImportRefusal::RowAttribution)?,
            });
        material.links.push(ProvenNativeV2Link {
            row,
            class: NativeV2LinkClass::Primary,
            ceremony: ceremony.to_owned(),
            run_id,
            request_id,
        });
    }
    Ok(())
}

fn add_sponsor_material(
    material: &mut MaterialBuilder,
    transcripts: &TranscriptIndex<'_>,
    run_ids: &BTreeMap<String, String>,
) -> Result<(), NativeV2ImportRefusal> {
    let absent = semantic_operation(
        transcripts,
        "sponsored-change-absent",
        OperationRole::Acceptance,
    )?;
    let present = semantic_operation(
        transcripts,
        "sponsored-change-present",
        OperationRole::Acceptance,
    )?;
    let absent_run_id = run_ids
        .get("sponsored-change-absent")
        .cloned()
        .ok_or(NativeV2ImportRefusal::RowAttribution)?;
    let absent_request_id = global_request_id("sponsored-change-absent", &absent.request_id);
    material
        .observations
        .push(LiveReportObservation::MultiRowSemantic {
            witness: LiveMultiRowSemanticWitness::new(
                absent_run_id.clone(),
                absent_request_id.clone(),
                absent
                    .accepted_identity
                    .ok_or(NativeV2ImportRefusal::RowAttribution)?,
                BTreeSet::from([ROW_ROSTER[19], ROW_ROSTER[21]]),
                sponsor_predicates(absent, present)?,
            ),
        });
    for row in [ROW_ROSTER[19], ROW_ROSTER[21]] {
        material.links.push(ProvenNativeV2Link {
            row,
            class: NativeV2LinkClass::Primary,
            ceremony: "sponsored-change-absent".to_owned(),
            run_id: absent_run_id.clone(),
            request_id: absent_request_id.clone(),
        });
    }
    Ok(())
}

fn add_conservation_material(
    material: &mut MaterialBuilder,
    transcripts: &TranscriptIndex<'_>,
    run_ids: &BTreeMap<String, String>,
) -> Result<(), NativeV2ImportRefusal> {
    let conservation = semantic_operation(
        transcripts,
        "conservation-negatives",
        OperationRole::Control,
    )?;
    let conservation_run_id = run_ids
        .get("conservation-negatives")
        .cloned()
        .ok_or(NativeV2ImportRefusal::RowAttribution)?;
    let conservation_request_id =
        global_request_id("conservation-negatives", &conservation.request_id);
    material
        .observations
        .push(LiveReportObservation::NativeAcceptance {
            row: ROW_ROSTER[23],
            run_id: conservation_run_id.clone(),
            request_id: conservation_request_id.clone(),
            identity: conservation
                .accepted_identity
                .ok_or(NativeV2ImportRefusal::RowAttribution)?,
        });
    material.links.push(ProvenNativeV2Link {
        row: ROW_ROSTER[23],
        class: NativeV2LinkClass::Primary,
        ceremony: "conservation-negatives".to_owned(),
        run_id: conservation_run_id,
        request_id: conservation_request_id,
    });
    Ok(())
}

fn add_refusal_material(
    material: &mut MaterialBuilder,
    parsed: &[ParsedTranscript],
    transcripts: &TranscriptIndex<'_>,
    run_ids: &BTreeMap<String, String>,
) -> Result<(), NativeV2ImportRefusal> {
    for transcript in parsed {
        let ceremony = transcript.summary.ceremony();
        for operation in transcript
            .operations
            .iter()
            .filter(|operation| operation.role == OperationRole::Refusal)
        {
            let mutant = operation
                .mutant
                .ok_or(NativeV2ImportRefusal::RowAttribution)?;
            let row = mutant.row();
            let run_id = run_ids
                .get(ceremony)
                .cloned()
                .ok_or(NativeV2ImportRefusal::RowAttribution)?;
            let request_id = global_request_id(ceremony, &operation.request_id);
            let control_id = operation
                .control_request_id
                .as_deref()
                .ok_or(NativeV2ImportRefusal::RowAttribution)?;
            let control = semantic_operation(transcripts, ceremony, OperationRole::Control)?;
            if control.request_id != control_id {
                return Err(NativeV2ImportRefusal::RowAttribution);
            }
            let support_request_id = global_request_id(ceremony, control_id);
            let support = LiveSupportLink::new(
                run_id.clone(),
                support_request_id.clone(),
                control.request_bytes.clone(),
                operation_response(control).map_err(|()| NativeV2ImportRefusal::RowAttribution)?,
            );
            material
                .observations
                .push(LiveReportObservation::NativeRefusalWithSupport {
                    row,
                    run_id: run_id.clone(),
                    request_id: request_id.clone(),
                    declared_boundary: boundary_for(mutant)
                        .ok_or(NativeV2ImportRefusal::RowAttribution)?,
                    observed_layer: operation.layer,
                    detail: operation.detail.clone(),
                    support,
                });
            material.links.extend([
                ProvenNativeV2Link {
                    row,
                    class: NativeV2LinkClass::Primary,
                    ceremony: ceremony.to_owned(),
                    run_id: run_id.clone(),
                    request_id,
                },
                ProvenNativeV2Link {
                    row,
                    class: NativeV2LinkClass::Support,
                    ceremony: ceremony.to_owned(),
                    run_id,
                    request_id: support_request_id,
                },
            ]);
        }
    }
    Ok(())
}

fn add_pair_material(
    material: &mut MaterialBuilder,
    transcripts: &TranscriptIndex<'_>,
    run_ids: &BTreeMap<String, String>,
) -> Result<(), NativeV2ImportRefusal> {
    let explicit = semantic_operation(
        transcripts,
        "pairs-arc",
        OperationRole::Paired(LivePairMember::Explicit),
    )?;
    let private = semantic_operation(
        transcripts,
        "pairs-arc",
        OperationRole::Paired(LivePairMember::Private),
    )?;
    if !pair_recomputes(explicit, private) {
        return Err(NativeV2ImportRefusal::PairProjection);
    }
    let pair_run_id = run_ids
        .get("pairs-arc")
        .cloned()
        .ok_or(NativeV2ImportRefusal::RowAttribution)?;
    let explicit_request_id = global_request_id("pairs-arc", &explicit.request_id);
    let private_request_id = global_request_id("pairs-arc", &private.request_id);
    material
        .observations
        .push(LiveReportObservation::PairedRelation {
            row: ROW_ROSTER[41],
            explicit_run_id: pair_run_id.clone(),
            explicit_request_id: explicit_request_id.clone(),
            explicit_identity: explicit
                .accepted_identity
                .ok_or(NativeV2ImportRefusal::RowAttribution)?,
            private_run_id: pair_run_id.clone(),
            private_request_id: private_request_id.clone(),
            private_identity: private
                .accepted_identity
                .ok_or(NativeV2ImportRefusal::RowAttribution)?,
            relation: VALIDATED_PAIR_RELATION.to_owned(),
        });
    for request_id in [explicit_request_id, private_request_id] {
        material.links.push(ProvenNativeV2Link {
            row: ROW_ROSTER[41],
            class: NativeV2LinkClass::Primary,
            ceremony: "pairs-arc".to_owned(),
            run_id: pair_run_id.clone(),
            request_id,
        });
    }
    Ok(())
}

fn finish_material(mut material: MaterialBuilder) -> Result<MaterialParts, NativeV2ImportRefusal> {
    material.links.sort_by(|left, right| {
        ROW_ROSTER
            .iter()
            .position(|row| *row == left.row)
            .cmp(&ROW_ROSTER.iter().position(|row| *row == right.row))
            .then_with(|| left.class.cmp(&right.class))
            .then_with(|| left.ceremony.cmp(&right.ceremony))
            .then_with(|| left.request_id.cmp(&right.request_id))
    });
    let attributions = ROW_ROSTER
        .iter()
        .map(|row| {
            Ok(NativeV2RowAttribution {
                row,
                ceremonies: row_ceremonies(row)
                    .ok_or(NativeV2ImportRefusal::RowAttribution)?
                    .iter()
                    .map(ToString::to_string)
                    .collect(),
            })
        })
        .collect::<Result<Vec<_>, NativeV2ImportRefusal>>()?;
    if material.observations.len() != 41 {
        return Err(NativeV2ImportRefusal::RowAttribution);
    }
    if attributions.len() != ROW_ROSTER.len() {
        return Err(NativeV2ImportRefusal::RowAttribution);
    }
    Ok((material.observations, material.links, attributions))
}

fn build_material(
    parsed: &[ParsedTranscript],
    run_ids: &BTreeMap<String, String>,
) -> Result<MaterialParts, NativeV2ImportRefusal> {
    let transcripts = parsed
        .iter()
        .map(|transcript| (transcript.summary.ceremony(), transcript))
        .collect::<TranscriptIndex<'_>>();
    let mut material = MaterialBuilder::default();
    add_parity_material(&mut material, &transcripts, run_ids)?;
    add_positive_material(&mut material, &transcripts, run_ids)?;
    add_sponsor_material(&mut material, &transcripts, run_ids)?;
    add_conservation_material(&mut material, &transcripts, run_ids)?;
    add_refusal_material(&mut material, parsed, &transcripts, run_ids)?;
    add_pair_material(&mut material, &transcripts, run_ids)?;
    finish_material(material)
}

fn validate_inputs(
    inputs: &CorpusInputs<'_>,
    expected_manifest_hash: &str,
    expected_report_hash: &str,
) -> Result<ValidatedNativeV2R7Corpus, NativeV2ImportRefusal> {
    validate_manifest(inputs, expected_manifest_hash)?;
    let report_hash =
        decode_digest(expected_report_hash).ok_or(NativeV2ImportRefusal::RunReportGrammar)?;
    if tagged::sha256(inputs.report) != report_hash {
        return Err(NativeV2ImportRefusal::RunReportGrammar);
    }
    let report = parse_report(inputs.report)?;
    if report.manifest_hash != tagged::sha256(inputs.manifest) {
        return Err(NativeV2ImportRefusal::ReportManifestBinding);
    }
    let capture_files = inputs
        .files
        .iter()
        .filter(|file| file.name.ends_with(".capture"))
        .collect::<Vec<_>>();
    if capture_files.len() != NATIVE_V2_R7_CEREMONY_ROSTER.len() {
        return Err(NativeV2ImportRefusal::CorpusCensus {
            expected: NATIVE_V2_R7_CEREMONY_ROSTER.len(),
            actual: capture_files.len(),
        });
    }
    let mut parsed = Vec::with_capacity(capture_files.len());
    let mut role_census = [0; 6];
    for (file, ceremony) in capture_files.into_iter().zip(NATIVE_V2_R7_CEREMONY_ROSTER) {
        let expected_digest = report
            .ceremony_digests
            .get(ceremony)
            .ok_or(NativeV2ImportRefusal::RunReportGrammar)?;
        if tagged::sha256(file.bytes) != *expected_digest {
            return Err(NativeV2ImportRefusal::CrossFileBinding {
                ceremony: ceremony.to_owned(),
                field: "report-ceremony-content-address",
            });
        }
        let transcript = parse_transcript(file.name, file.bytes, &report)?;
        let local_census = validate_transcript_semantics(&transcript)?;
        for (total, local) in role_census.iter_mut().zip(local_census) {
            *total += local;
        }
        parsed.push(transcript);
    }
    if parsed
        .iter()
        .map(|transcript| transcript.summary.ceremony())
        .ne(NATIVE_V2_R7_CEREMONY_ROSTER)
    {
        return Err(NativeV2ImportRefusal::RowAttribution);
    }
    if role_census != [107, 31, 5, 17, 1, 1] {
        return Err(NativeV2ImportRefusal::RowAttribution);
    }
    if parsed
        .iter()
        .map(|transcript| transcript.summary.operation_count())
        .sum::<usize>()
        != 162
    {
        return Err(NativeV2ImportRefusal::RowAttribution);
    }
    let (runs, run_ids) = build_runs(&parsed)?;
    let (observations, links, attributions) = build_material(&parsed, &run_ids)?;
    Ok(ValidatedNativeV2R7Corpus {
        transcripts: parsed
            .into_iter()
            .map(|transcript| transcript.summary)
            .collect(),
        runs,
        observations,
        links,
        attributions,
        outcome_count: 40,
        content_address: hex_bytes(&report_hash),
    })
}

/// Admit the immutable reviewed archive once and return its content-addressed corpus.
///
/// # Errors
///
/// Returns [`NativeV2ImportRefusal`] at the first census, manifest, report,
/// transcript, transaction, linkage, mutation, projection, or attribution
/// disagreement.
pub fn run_of_record() -> Result<&'static ValidatedNativeV2R7Corpus, NativeV2ImportRefusal> {
    static CORPUS: OnceLock<Result<ValidatedNativeV2R7Corpus, NativeV2ImportRefusal>> =
        OnceLock::new();
    match CORPUS.get_or_init(|| {
        validate_inputs(
            &CorpusInputs::embedded(),
            NATIVE_V2_R7_MANIFEST_SHA256,
            NATIVE_V2_R7_RUN_ADDRESS,
        )
    }) {
        Ok(corpus) => Ok(corpus),
        Err(refusal) => Err(refusal.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct OwnedCorpus {
        files: Vec<(String, Vec<u8>, usize)>,
        manifest: Vec<u8>,
        report: Vec<u8>,
    }

    impl OwnedCorpus {
        fn embedded() -> Self {
            Self {
                files: ARCHIVE_FILES
                    .iter()
                    .map(|file| {
                        (
                            file.name.to_owned(),
                            file.bytes.to_vec(),
                            file.expected_size,
                        )
                    })
                    .collect(),
                manifest: MANIFEST_BYTES.to_vec(),
                report: RUN_REPORT_BYTES.to_vec(),
            }
        }

        fn inputs(&self) -> CorpusInputs<'_> {
            CorpusInputs {
                files: self
                    .files
                    .iter()
                    .map(|(name, bytes, expected_size)| InputFile {
                        name,
                        bytes,
                        expected_size: *expected_size,
                    })
                    .collect(),
                manifest: &self.manifest,
                report: &self.report,
            }
        }

        fn replace_capture(&mut self, name: &str, from: &str, to: &str) {
            assert_eq!(from.len(), to.len());
            let (_, bytes, _) = self
                .files
                .iter_mut()
                .find(|(candidate, _, _)| candidate == name)
                .expect("the test names an embedded capture");
            let text = std::str::from_utf8(bytes).expect("captures are UTF-8");
            assert!(text.contains(from));
            *bytes = text.replacen(from, to, 1).into_bytes();
        }

        fn rebind(&mut self, name: &str, update_content_hash: bool) -> (String, String) {
            let ceremony = name
                .strip_prefix("e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.")
                .and_then(|name| name.strip_suffix(".capture"))
                .expect("the test names a semantic capture");
            let (_, capture, _) = self
                .files
                .iter_mut()
                .find(|(candidate, _, _)| candidate == name)
                .expect("the test names an embedded capture");
            if update_content_hash {
                let marker = b"capture-content-sha256 ";
                let position = capture
                    .windows(marker.len())
                    .position(|window| window == marker)
                    .expect("the capture carries its content address");
                let digest = hex_bytes(&tagged::sha256(&capture[..position]));
                let digest_start = position + marker.len();
                capture[digest_start..digest_start + 64].copy_from_slice(digest.as_bytes());
            }
            let capture_digest = hex_bytes(&tagged::sha256(capture));
            let mut report = std::str::from_utf8(&self.report)
                .expect("the report is UTF-8")
                .to_owned();
            let roster_prefix = format!("ceremony {ceremony} ");
            let roster_start = report
                .find(&roster_prefix)
                .expect("the report names the capture")
                + roster_prefix.len();
            report.replace_range(roster_start..roster_start + 64, &capture_digest);

            let mut manifest = String::new();
            for (file_name, bytes, _) in &self.files {
                let _ = writeln!(
                    &mut manifest,
                    "{}  {file_name}",
                    hex_bytes(&tagged::sha256(bytes))
                );
            }
            self.manifest = manifest.into_bytes();
            let manifest_digest = hex_bytes(&tagged::sha256(&self.manifest));
            let manifest_prefix = "manifest-sha256 ";
            let manifest_start = report
                .find(manifest_prefix)
                .expect("the report binds the manifest")
                + manifest_prefix.len();
            report.replace_range(manifest_start..manifest_start + 64, &manifest_digest);
            self.report = report.into_bytes();
            let report_digest = hex_bytes(&tagged::sha256(&self.report));
            (manifest_digest, report_digest)
        }
    }

    #[test]
    fn reviewed_archive_admits_exactly_39_ceremonies_and_40_outcomes() {
        let corpus = run_of_record().expect("the reviewed archive admits");
        assert_eq!(corpus.transcripts().len(), 39);
        assert_eq!(corpus.outcome_count(), 40);
        assert_eq!(corpus.attributions().len(), 42);
        assert_eq!(corpus.observations().len(), 41);
        assert_eq!(corpus.content_address(), NATIVE_V2_R7_RUN_ADDRESS);
        assert!(crate::live_corpus_rerun_day::parse_rerun_day_archive().is_ok());
    }

    #[test]
    fn corrupt_transcript_content_address_refuses() {
        let mut owned = OwnedCorpus::embedded();
        let name = "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.explicit-one-to-one.capture";
        owned.replace_capture(name, "legacy-rendering 1478 65", "legacy-rendering 1478 64");
        let (manifest, report) = owned.rebind(name, false);
        assert!(matches!(
            validate_inputs(&owned.inputs(), &manifest, &report),
            Err(NativeV2ImportRefusal::CaptureContentHash { .. })
        ));
    }

    #[test]
    fn wrong_manifest_hash_refuses() {
        let mut owned = OwnedCorpus::embedded();
        owned.manifest[0] = if owned.manifest[0] == b'0' {
            b'1'
        } else {
            b'0'
        };
        assert_eq!(
            validate_inputs(
                &owned.inputs(),
                NATIVE_V2_R7_MANIFEST_SHA256,
                NATIVE_V2_R7_RUN_ADDRESS,
            ),
            Err(NativeV2ImportRefusal::ManifestHash)
        );
    }

    #[test]
    fn broken_response_request_link_refuses() {
        let mut owned = OwnedCorpus::embedded();
        let name = "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.explicit-one-to-one.capture";
        owned.replace_capture(
            name,
            "response-request-id 9 726571756573742d32",
            "response-request-id 9 726571756573742d39",
        );
        let (manifest, report) = owned.rebind(name, true);
        assert!(matches!(
            validate_inputs(&owned.inputs(), &manifest, &report),
            Err(NativeV2ImportRefusal::IdentifierLink { .. })
        ));
    }

    #[test]
    fn wrong_accepted_txid_refuses() {
        let mut owned = OwnedCorpus::embedded();
        let name = "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.explicit-one-to-one.capture";
        owned.replace_capture(
            name,
            "response-target-identity 872a2294da5ea650a7a74ffd8a5932210930ab70d6a08a991eb3ea471ee29abb",
            "response-target-identity 972a2294da5ea650a7a74ffd8a5932210930ab70d6a08a991eb3ea471ee29abb",
        );
        let (manifest, report) = owned.rebind(name, true);
        assert!(matches!(
            validate_inputs(&owned.inputs(), &manifest, &report),
            Err(NativeV2ImportRefusal::TransactionIdentity { .. })
        ));
    }

    #[test]
    fn locator_contradicted_by_decoded_bytes_refuses() {
        let mut owned = OwnedCorpus::embedded();
        let name = "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.owner-signing-negatives.capture";
        owned.replace_capture(
            name,
            "mutation-locator witnessless-range 90 122",
            "mutation-locator witnessless-range 91 122",
        );
        let (manifest, report) = owned.rebind(name, true);
        assert!(matches!(
            validate_inputs(&owned.inputs(), &manifest, &report),
            Err(NativeV2ImportRefusal::MutationLocator { .. })
        ));
    }

    #[test]
    fn perturbed_pair_projection_term_refuses() {
        let mut owned = OwnedCorpus::embedded();
        let name =
            "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.pairs-arc.capture";
        owned.replace_capture(
            name,
            "projection-input-amount 0 700000000",
            "projection-input-amount 0 700000001",
        );
        let (manifest, report) = owned.rebind(name, true);
        assert_eq!(
            validate_inputs(&owned.inputs(), &manifest, &report),
            Err(NativeV2ImportRefusal::PairProjection)
        );
    }

    #[test]
    fn wrong_protocol_tag_refuses() {
        let mut owned = OwnedCorpus::embedded();
        let name = "e8836e79b631b96420fb8006353df5b673ec7c69b830fb5f0555fb06add02517.explicit-one-to-one.capture";
        owned.replace_capture(
            name,
            "handshake-protocol-schema 7",
            "handshake-protocol-schema 6",
        );
        let (manifest, report) = owned.rebind(name, true);
        assert!(matches!(
            validate_inputs(&owned.inputs(), &manifest, &report),
            Err(NativeV2ImportRefusal::CrossFileBinding {
                field: "protocol-revision",
                ..
            })
        ));
    }

    #[test]
    fn missing_duplicate_and_extra_files_refuse_at_census() {
        let mut missing = OwnedCorpus::embedded();
        missing.files.pop();
        assert!(matches!(
            validate_inputs(
                &missing.inputs(),
                NATIVE_V2_R7_MANIFEST_SHA256,
                NATIVE_V2_R7_RUN_ADDRESS,
            ),
            Err(NativeV2ImportRefusal::CorpusCensus { .. })
        ));

        let mut duplicate = OwnedCorpus::embedded();
        duplicate.files[1].0 = duplicate.files[0].0.clone();
        assert!(matches!(
            validate_inputs(
                &duplicate.inputs(),
                NATIVE_V2_R7_MANIFEST_SHA256,
                NATIVE_V2_R7_RUN_ADDRESS,
            ),
            Err(NativeV2ImportRefusal::CorpusCensus { .. })
        ));

        let mut extra = OwnedCorpus::embedded();
        extra.files.push(("extra".to_owned(), Vec::new(), 0));
        assert!(matches!(
            validate_inputs(
                &extra.inputs(),
                NATIVE_V2_R7_MANIFEST_SHA256,
                NATIVE_V2_R7_RUN_ADDRESS,
            ),
            Err(NativeV2ImportRefusal::CorpusCensus { .. })
        ));
    }
}
