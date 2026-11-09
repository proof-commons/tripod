//! Byte-level preservation against the retained unchanged-base capture.

use std::{error::Error, fmt::Debug, num::NonZeroU64, time::Instant};

use architecture::{ARCHITECTURE, OperationId};
use realization::RealizationScope;
use sha2::{Digest, Sha256};

use crate::{
    AnalysisPolicy, CompilationScope, ProofSearchLimits, analyzed::analyze_scoped_program,
    bind_input, placement::PlacementSearchLimits,
};

const BASE_TAG: &str = "0.7.75-dev";
const TEST_ID: &str = "tests::engine_base_capture::preserve_engine_base";
const PROOF_STATES: u64 = 1_000_000;
const PROOF_CANDIDATES: u64 = 10_000;
const PLACEMENT_STATES: u64 = 10_000_000;
const PLACEMENT_CANDIDATES: u64 = 1_000_000;
const C: &[OperationId] = &[OperationId::CompactAsh];
const T: &[OperationId] = &[OperationId::TransferLive];
const A: &[OperationId] = &[OperationId::AnnounceMaturity];
const CT: &[OperationId] = &[OperationId::CompactAsh, OperationId::TransferLive];
const CTA: &[OperationId] = &[
    OperationId::CompactAsh,
    OperationId::TransferLive,
    OperationId::AnnounceMaturity,
];

struct Subject {
    name: &'static str,
    realization: &'static [OperationId],
    compilation: Option<&'static [OperationId]>,
    constructor: &'static str,
}

const SUBJECTS: &[Subject] = &[
    Subject {
        name: "realization-C",
        realization: C,
        compilation: None,
        constructor: "derive(C).project()",
    },
    Subject {
        name: "realization-T",
        realization: T,
        compilation: None,
        constructor: "derive(T).project()",
    },
    Subject {
        name: "realization-CT",
        realization: CT,
        compilation: None,
        constructor: "derive(C,T).project()",
    },
    Subject {
        name: "realization-A",
        realization: A,
        compilation: None,
        constructor: "derive(A).project()",
    },
    Subject {
        name: "realization-CTA",
        realization: CTA,
        compilation: None,
        constructor: "derive(C,T,A).project()",
    },
    Subject {
        name: "compiler-C",
        realization: CT,
        compilation: Some(C),
        constructor: "fallible bound_input(C): derive(C,T); bind_input(C); analyze; project",
    },
    Subject {
        name: "compiler-T",
        realization: CT,
        compilation: Some(T),
        constructor: "fallible bound_input(T): derive(C,T); bind_input(T); analyze; project",
    },
    Subject {
        name: "compiler-CT",
        realization: CT,
        compilation: Some(CT),
        constructor: "fallible bound_input(C,T): derive(C,T); bind_input(C,T); analyze; project",
    },
    Subject {
        name: "compiler-A",
        realization: A,
        compilation: Some(A),
        constructor: "fallible announcement_input(): derive(A); bind_input(A); analyze; project",
    },
    Subject {
        name: "compiler-CTA",
        realization: CTA,
        compilation: Some(CTA),
        constructor: "explicit derive(C,T,A); bind_input(C,T,A); analyze; project",
    },
];

struct Captured {
    result: &'static str,
    stage: &'static str,
    bytes: String,
}

fn captured(result: &'static str, stage: &'static str, value: &impl Debug) -> Captured {
    Captured {
        result,
        stage,
        bytes: format!("result: {result}\nstage: {stage}\n{value:#?}\n"),
    }
}

fn capture(
    subject: &Subject,
    proof_limits: ProofSearchLimits,
    placement_limits: PlacementSearchLimits,
) -> Captured {
    let scope = match RealizationScope::from_operations(subject.realization.iter().copied()) {
        Ok(scope) => scope,
        Err(error) => return captured("typed-refusal", "realization_scope", &error),
    };
    let realization = match realization::derive(&ARCHITECTURE, scope) {
        Ok(realization) => realization,
        Err(error) => return captured("typed-refusal", "derive", &error),
    };
    let Some(operations) = subject.compilation else {
        return captured(
            "projection",
            "realization_projection",
            &realization.project(),
        );
    };
    let scope = match CompilationScope::from_operations(operations.iter().copied()) {
        Ok(scope) => scope,
        Err(error) => return captured("typed-refusal", "compilation_scope", &error),
    };
    let input = match bind_input(
        &ARCHITECTURE,
        realization,
        scope,
        AnalysisPolicy::strict(proof_limits),
    ) {
        Ok(input) => input,
        Err(error) => return captured("typed-refusal", "bind_input", &error),
    };
    match analyze_scoped_program(&input, placement_limits) {
        Ok(program) => captured("projection", "compiler_projection", &program.project()),
        Err(error) => captured("typed-refusal", "analyze_scoped_program", &error),
    }
}

const MANIFEST: &str = include_str!("../../tests/fixtures/engine_base/MANIFEST");

#[test]
fn preserve_engine_base() -> Result<(), Box<dyn Error>> {
    let span = tracing::info_span!("engine_base_preservation", base = BASE_TAG, test = TEST_ID);
    let _entered = span.enter();
    let started = Instant::now();
    let proof_limits = ProofSearchLimits::new(
        NonZeroU64::new(PROOF_STATES).ok_or("zero proof state limit")?,
        NonZeroU64::new(PROOF_CANDIDATES).ok_or("zero proof candidate limit")?,
    );
    let placement_limits = PlacementSearchLimits::new(
        NonZeroU64::new(PLACEMENT_STATES).ok_or("zero placement state limit")?,
        NonZeroU64::new(PLACEMENT_CANDIDATES).ok_or("zero placement candidate limit")?,
    );
    let entries = MANIFEST.split("\n\n").skip(1).collect::<Vec<_>>();
    assert_eq!(entries.len(), SUBJECTS.len());
    for subject in SUBJECTS {
        let fields = entries
            .iter()
            .find_map(|entry| {
                let fields = entry
                    .lines()
                    .filter_map(|line| line.split_once('='))
                    .collect::<std::collections::BTreeMap<_, _>>();
                (fields.get("subject").copied() == Some(subject.name)).then_some(fields)
            })
            .ok_or("missing manifest subject")?;
        assert_eq!(
            fields.get("realization_scope").copied(),
            Some(format!("{:?}", subject.realization).as_str())
        );
        assert_eq!(
            fields.get("compiler_scope").copied(),
            Some(format!("{:?}", subject.compilation).as_str())
        );
        assert_eq!(
            fields.get("constructor").copied(),
            Some(subject.constructor)
        );
        assert_eq!(
            fields.get("proof_limits").copied(),
            Some(format!("states:{PROOF_STATES},candidates:{PROOF_CANDIDATES}").as_str())
        );
        assert_eq!(
            fields.get("placement_limits").copied(),
            Some(format!("states:{PLACEMENT_STATES},candidates:{PLACEMENT_CANDIDATES}").as_str())
        );
        let result = capture(subject, proof_limits, placement_limits);
        let digest = format!("{:x}", Sha256::digest(result.bytes.as_bytes()));
        let length = fields
            .get("bytes")
            .ok_or("missing byte length")?
            .parse::<usize>()?;
        assert_eq!(result.bytes.len(), length, "{} byte length", subject.name);
        assert_eq!(
            fields.get("sha256").copied(),
            Some(digest.as_str()),
            "{} digest; full base bytes: refs/evidence/w20b1-engine-base",
            subject.name
        );
        assert_eq!(fields.get("result").copied(), Some(result.result));
        assert_eq!(fields.get("stage").copied(), Some(result.stage));
        tracing::info!(
            subject = subject.name,
            result = result.result,
            bytes = length,
            sha256 = digest,
            "base projection preserved"
        );
    }
    tracing::info!(
        subjects = SUBJECTS.len(),
        elapsed_ms = started.elapsed().as_millis(),
        "base preservation complete"
    );
    Ok(())
}
