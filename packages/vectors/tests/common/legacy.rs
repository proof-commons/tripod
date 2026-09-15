impl CeremonyCaptureFacts {
    fn with_projection(mut self, role: RequestRole, projection: ProjectionInput) -> Self {
        if let Some(operation) = self
            .operations
            .iter_mut()
            .find(|operation| operation.role == role)
        {
            operation.projection = Some(projection);
        }
        self
    }
    fn with_locator(mut self, step: &str, locator: LiveMutationLocator) -> Self {
        // Origin records also carry ceremony-only controls such as
        // `missing-rangeproof`, which has no row in the closed report-mutant
        // vocabulary. A locator completes an already typed mutation; it does
        // not promote every origin negative into a report mutation.
        if let Some(operation) = self
            .operations
            .iter_mut()
            .find(|operation| operation.case_step == step && operation.mutant.is_some())
        {
            operation.locator = Some(locator);
        }
        self
    }
}


impl CaptureDestination {
    fn from_environment() -> Result<Option<Self>, String> {
        let Some(directory) = environment("TRIPOD_LIVE_REPORT_DIR") else {
            return Ok(None);
        };
        let short_sha = environment("TRIPOD_LIVE_SUITE_SHORT_SHA").ok_or_else(|| {
            "TRIPOD_LIVE_SUITE_SHORT_SHA is required for enhanced capture".to_owned()
        })?;
        let suite_commit = environment("TRIPOD_LIVE_SUITE_COMMIT")
            .ok_or_else(|| "TRIPOD_LIVE_SUITE_COMMIT is required".to_owned())?;
        let suite_tree = environment("TRIPOD_LIVE_SUITE_TREE")
            .ok_or_else(|| "TRIPOD_LIVE_SUITE_TREE is required".to_owned())?;
        Self::new(
            PathBuf::from(directory),
            short_sha,
            suite_commit,
            suite_tree,
        )
        .map(Some)
    }
}


impl CaptureGuard {
    fn new(ceremony: CeremonyId) -> Self {
        let capture = capture_path(ceremony).expect("the enhanced capture path is valid");
        let (destination, path) = capture.unzip();
        Self {
            ceremony,
            destination,
            path,
            transcript_written: false,
        }
    }
}


fn pair_projection_input(programs: &[Vec<u8>]) -> Result<ProjectionInput, String> {
    let fixture = vectors::live_pair_arc::pair_arc_fixture();
    let owners = fixture
        .sources()
        .iter()
        .map(|source| vectors::live_pair_arc::published_scalar(source.owner()).to_vec())
        .collect();
    let amounts = fixture
        .sources()
        .iter()
        .map(|source| source.amount())
        .collect();
    let destinations = fixture
        .destinations()
        .iter()
        .map(|destination| {
            programs
                .get(destination.owner())
                .cloned()
                .map(|program| {
                    (
                        vectors::live_pair_arc::published_scalar(destination.owner()).to_vec(),
                        (program, destination.amount()),
                    )
                })
                .ok_or_else(|| "the pair projection program census differs".to_owned())
        })
        .collect::<Result<_, _>>()?;
    Ok(ProjectionInput {
        owners,
        amounts,
        destinations,
    })
}


fn sponsor_capture_facts(
    ceremony: CeremonyId,
    capture: &NativeOperationCapture,
    predecessor_digest: Option<[u8; 32]>,
) -> CeremonyCaptureFacts {
    CeremonyCaptureFacts::from_capture(ceremony, capture)
        .with_digest("predecessor", predecessor_digest)
}


fn capture_path(ceremony: CeremonyId) -> Result<Option<(CaptureDestination, PathBuf)>, String> {
    let Some(destination) = CaptureDestination::from_environment()? else {
        return Ok(None);
    };
    let thread = std::thread::current();
    let current = thread
        .name()
        .and_then(|name| name.rsplit("::").next())
        .ok_or_else(|| "the current Rust test has no name".to_owned())?;
    let path = destination.capture_path(ceremony, current)?;
    Ok(Some((destination, path)))
}


fn capture_diagnostics(ceremony: CeremonyId, legacy_report: Option<&Path>) -> PathBuf {
    if let Some(directory) = environment("TRIPOD_LIVE_REPORT_DIR") {
        return Path::new(&directory)
            .join("diagnostics")
            .join(ceremony.as_str());
    }
    legacy_report
        .and_then(Path::parent)
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf()
}


fn legacy_report(extension: Option<&str>) -> Option<PathBuf> {
    let enhanced_directory = environment("TRIPOD_LIVE_REPORT_DIR").map(PathBuf::from);
    let legacy_base = environment("TRIPOD_LIVE_REPORT").map(PathBuf::from);
    legacy_report_for(
        enhanced_directory.as_deref(),
        legacy_base.as_deref(),
        extension,
    )
}


fn legacy_report_for(
    enhanced_directory: Option<&Path>,
    legacy_base: Option<&Path>,
    extension: Option<&str>,
) -> Option<PathBuf> {
    if enhanced_directory.is_some() {
        return None;
    }
    let base = legacy_base?;
    Some(extension.map_or_else(|| base.to_path_buf(), |value| base.with_extension(value)))
}


fn execute_and_capture(
    target: &target_elements::ReviewedElementsTapscriptDefinition,
    binding: &target_elements::ReviewedDevelopmentBinding,
    configuration: &ExecutorConfiguration,
    planner: &mut dyn TargetOperationPlanner,
) -> (
    Result<
        target_elements_conformance::executor::ExecutionTranscript,
        target_elements_conformance::error::NativeConformanceError,
    >,
    NativeOperationCapture,
) {
    let mut capture = NativeOperationCapture::default();
    let outcome =
        execute_operations_captured(target, binding, configuration, planner, &mut capture);
    (outcome, capture)
}

