//! Offline fixture acquisition and admission, compiled only into the focused test target.

use elements::hashes::sha256d;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::io::Read;
use target_elements_conformance::constructor::tagged::sha256;
use target_elements_conformance::protocol::{
    NativeOperationRequest, NativeOperationResponse, ObservedOutcomeLayer, WireOutpoint,
};
use target_elements_conformance::sponsor_parent::bind_sponsor_parent;
use transaction::{AssetField, TargetOutput, TargetTransaction, ValueField};
use vectors::keyed_fixed_capture::{
    CaptureFamily, CaptureProvenance, CaptureRole, CaptureSubject, JournalBinding, JournalRefusal,
    JournalRow, PairControl, bind_journal, expected_control, subjects, validate_journal_transport,
    validate_s8c_census,
};
use vectors::keyed_fixed_manifest::{CaptureEpoch, FreezeManifest, SourceVersion, ValidatedFreeze};
use vectors::keyed_fixed_stack_model as stack_model;

#[path = "history.rs"]
mod history;
#[path = "sponsor.rs"]
mod sponsor;

const CENSUS: &str = include_str!("../../fixtures/keyed-fixed-native/CENSUS.json");
const CENSUS_PIN: &str = "fe2345f624b912e4b50230d043f365752e7a33db45b2c4073e2fe37ddb5c51d6";
const SCALING_FULL: [usize; 17] = [
    3, 6, 9, 12, 15, 18, 21, 24, 27, 30, 33, 774, 1533, 2295, 4248, 8148, 12052,
];
const FIXTURES: [(&str, &[u8]); 7] = [
    (
        "clauses",
        include_bytes!("../../fixtures/keyed-fixed-native/clauses.json.lzma"),
    ),
    (
        "policy",
        include_bytes!("../../fixtures/keyed-fixed-native/policy.json.lzma"),
    ),
    (
        "scaling",
        include_bytes!("../../fixtures/keyed-fixed-native/scaling.json.lzma"),
    ),
    (
        "histories",
        include_bytes!("../../fixtures/keyed-fixed-native/histories.json.lzma"),
    ),
    (
        "root",
        include_bytes!("../../fixtures/keyed-fixed-native/root.json.lzma"),
    ),
    (
        "preparation",
        include_bytes!("../../fixtures/keyed-fixed-native/preparation.json.lzma"),
    ),
    (
        "sources",
        include_bytes!("../../fixtures/keyed-fixed-native/sources.json.lzma"),
    ),
];

/// Corpus-only refusal before native journal credit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CorpusRefusal {
    Index,
    Bound,
    Digest,
    Encoding,
    Path,
    Seal,
    Census,
    Record,
    Control,
    Provenance,
    Measurement,
    Witness,
    Journal(JournalRefusal),
}

/// Exact retained files in one sealed campaign family and its shared preparation.
#[derive(Clone)]
pub struct Packet {
    family: CaptureFamily,
    epoch: CaptureEpoch,
    name: &'static str,
    run: String,
    files: BTreeMap<String, Vec<u8>>,
    archive_addressed: BTreeMap<String, (String, u64)>,
}

fn hex(bytes: &[u8]) -> String {
    vectors::maturity_observer::encode_hex(bytes)
}

fn decode_hex(text: &str) -> Result<Vec<u8>, CorpusRefusal> {
    vectors::maturity_observer::decode_hex(text).map_err(|_| CorpusRefusal::Encoding)
}

fn digest_matches(bytes: &[u8], digest: &Value) -> bool {
    digest.as_str() == Some(hex(&sha256(bytes)).as_str())
}

fn raw_bytes(value: &Value) -> Result<Vec<u8>, CorpusRefusal> {
    serde_json::from_value(value.clone()).map_err(|_| CorpusRefusal::Record)
}

fn index() -> Result<Value, CorpusRefusal> {
    indexed(CENSUS.as_bytes(), CENSUS_PIN)
}

fn indexed(bytes: &[u8], pin: &str) -> Result<Value, CorpusRefusal> {
    if hex(&sha256(bytes)) != pin {
        return Err(CorpusRefusal::Index);
    }
    let index: Value = serde_json::from_slice(bytes).map_err(|_| CorpusRefusal::Index)?;
    if index["schema"] != 1 || index["preset"] != 6 {
        return Err(CorpusRefusal::Index);
    }
    Ok(index)
}

fn decode_fixture(
    name: &str,
    compressed: &[u8],
    index: &Value,
    run: &str,
    files: &mut BTreeMap<String, Vec<u8>>,
) -> Result<(), CorpusRefusal> {
    let row = &index["fixtures"][name];
    if row["bytes"] != compressed.len() || !digest_matches(compressed, &row["sha256"]) {
        return Err(CorpusRefusal::Digest);
    }
    let members = row["members"].as_array().ok_or(CorpusRefusal::Index)?;
    let mut next_offset = 0usize;
    let mut count = 0usize;
    let mut previous = String::new();
    for member in members {
        let offset = usize::try_from(member["offset"].as_u64().ok_or(CorpusRefusal::Index)?)
            .map_err(|_| CorpusRefusal::Bound)?;
        let length = usize::try_from(member["bytes"].as_u64().ok_or(CorpusRefusal::Index)?)
            .map_err(|_| CorpusRefusal::Bound)?;
        let bound = member["decoded_bytes"]
            .as_u64()
            .ok_or(CorpusRefusal::Index)?;
        if offset != next_offset || bound > 32_000_000 {
            return Err(CorpusRefusal::Bound);
        }
        next_offset = offset.checked_add(length).ok_or(CorpusRefusal::Bound)?;
        let encoded = compressed
            .get(offset..next_offset)
            .ok_or(CorpusRefusal::Bound)?;
        if !digest_matches(encoded, &member["sha256"]) {
            return Err(CorpusRefusal::Digest);
        }
        let mut decoder = lzma_rust2::LzmaReader::new_mem_limit(encoded, 16 * 1024, None)
            .map_err(|_| CorpusRefusal::Bound)?;
        decoder.set_relaxed_end_condition(false);
        let mut raw = Vec::new();
        decoder
            .take(bound + 1)
            .read_to_end(&mut raw)
            .map_err(|_| CorpusRefusal::Encoding)?;
        if raw.len() as u64 != bound || !digest_matches(&raw, &member["decoded_sha256"]) {
            return Err(CorpusRefusal::Digest);
        }
        let pairs: Vec<(String, String)> =
            serde_json::from_slice(&raw).map_err(|_| CorpusRefusal::Encoding)?;
        if member["paths"] != pairs.len() {
            return Err(CorpusRefusal::Index);
        }
        for (path, bytes) in pairs {
            if path <= previous || path.contains("..") || path.starts_with('/') {
                return Err(CorpusRefusal::Path);
            }
            let allowed = if name == "sources" {
                index["source_history"].as_array().is_some_and(|catalog| {
                    catalog.iter().any(|row| {
                        row["member"] == path
                            && row["sha256"].as_str().is_some_and(|digest| {
                                hex(&sha256(&decode_hex(&bytes).unwrap_or_default())) == digest
                            })
                    })
                })
            } else {
                path.starts_with(&format!("{run}/"))
                    && (name == "root"
                        || name == "preparation"
                        || path.starts_with(&format!("{run}/{name}/")))
            };
            if !allowed {
                return Err(CorpusRefusal::Path);
            }
            if files.contains_key(&path) {
                return Err(CorpusRefusal::Path);
            }
            files.insert(path.clone(), decode_hex(&bytes)?);
            previous = path;
            count += 1;
        }
    }
    if next_offset != compressed.len() || row["paths"] != count {
        return Err(CorpusRefusal::Index);
    }
    if name == "sources"
        && index["source_history"]
            .as_array()
            .is_none_or(|catalog| catalog.len() != count)
    {
        return Err(CorpusRefusal::Index);
    }
    tracing::info!(target: "vectors::keyed_fixed_corpus", name, members = members.len(), paths = count, encoded_bytes = compressed.len(), outcome = "completed", "indexed fixture decoded");
    Ok(())
}

fn fixture(name: &str) -> Option<&'static [u8]> {
    FIXTURES
        .iter()
        .find_map(|(candidate, bytes)| (*candidate == name).then_some(*bytes))
}

impl Packet {
    /// Check a candidate compressed fixture against the pinned acquisition index.
    /// # Errors
    /// Refuses a changed compressed byte before decoding or interpretation.
    pub fn check_fixture_candidate(name: &str, compressed: &[u8]) -> Result<(), CorpusRefusal> {
        let index = index()?;
        let run = index["run"].as_str().ok_or(CorpusRefusal::Index)?;
        decode_fixture(name, compressed, &index, run, &mut BTreeMap::new())
    }

    /// Load one family and the shared sealed preparation through bounded members.
    /// # Errors
    /// Refuses digest, path, bound, or seal drift before interpreting a record.
    pub fn load(family: CaptureFamily) -> Result<Self, CorpusRefusal> {
        let (name, expected_records, expected_exchanges) = match family {
            CaptureFamily::Clauses => ("clauses", 32, 102),
            CaptureFamily::Policy => ("policy", 3, 9),
            CaptureFamily::Scaling => ("scaling", 17, 52),
            CaptureFamily::Histories => ("histories", 168, 504),
            _ => return Err(CorpusRefusal::Census),
        };
        let _span = tracing::info_span!(target: "vectors::keyed_fixed_corpus", "load_packet", name)
            .entered();
        let index = index()?;
        let run = index["run"]
            .as_str()
            .ok_or(CorpusRefusal::Index)?
            .to_owned();
        let mut files = BTreeMap::new();
        for selection in ["root", "preparation", "sources", name] {
            decode_fixture(
                selection,
                fixture(selection).ok_or(CorpusRefusal::Index)?,
                &index,
                &run,
                &mut files,
            )?;
        }
        let packet = Self {
            family,
            epoch: CaptureEpoch::CampaignThree,
            name,
            run,
            files,
            archive_addressed: BTreeMap::new(),
        };
        packet.check_seals()?;
        packet.check_census(expected_records, expected_exchanges)?;
        Ok(packet)
    }

    /// Acquire both S8c families from one checked index and three bounded
    /// fixture members. The caller supplies literal fixture bytes and index
    /// pin when the second capture has been independently reviewed.
    /// # Errors
    /// Refuses changed index, member, seal, role, ordinal or campaign marker.
    pub fn load_s8c(
        index_bytes: &[u8],
        index_pin: &str,
        blobs: [&[u8]; 3],
    ) -> Result<(Self, Self), CorpusRefusal> {
        let index = indexed(index_bytes, index_pin)?;
        let run = index["run"]
            .as_str()
            .ok_or(CorpusRefusal::Index)?
            .to_owned();
        let mut files = BTreeMap::new();
        let source_index = indexed(CENSUS.as_bytes(), CENSUS_PIN)?;
        let source_run = source_index["run"].as_str().ok_or(CorpusRefusal::Index)?;
        decode_fixture(
            "sources",
            fixture("sources").ok_or(CorpusRefusal::Index)?,
            &source_index,
            source_run,
            &mut files,
        )?;
        for (name, bytes) in ["preparation", "sponsor", "legacy-copy"]
            .into_iter()
            .zip(blobs)
        {
            decode_fixture(name, bytes, &index, &run, &mut files)?;
        }
        let mut archive_addressed = BTreeMap::new();
        let omissions = index["archive_addressed"]
            .as_array()
            .ok_or(CorpusRefusal::Index)?;
        for item in omissions {
            let path = item["path"].as_str().ok_or(CorpusRefusal::Index)?;
            let digest = item["sha256"].as_str().ok_or(CorpusRefusal::Index)?;
            let length = item["bytes"].as_u64().ok_or(CorpusRefusal::Index)?;
            if path.starts_with('/')
                || path.contains("..")
                || digest.len() != 64
                || !digest.bytes().all(|byte| byte.is_ascii_hexdigit())
                || archive_addressed
                    .insert(path.to_owned(), (digest.to_owned(), length))
                    .is_some()
            {
                return Err(CorpusRefusal::Index);
            }
        }
        let sponsor = Self {
            family: CaptureFamily::Sponsor,
            epoch: CaptureEpoch::Sponsor,
            name: "sponsor",
            run: run.clone(),
            files: files.clone(),
            archive_addressed: archive_addressed.clone(),
        };
        let legacy = Self {
            family: CaptureFamily::LegacyCopy,
            epoch: CaptureEpoch::Sponsor,
            name: "legacy-copy",
            run,
            files,
            archive_addressed,
        };
        sponsor.check_seals()?;
        legacy.check_seals()?;
        sponsor.check_campaign_seal()?;
        sponsor.check_census(8, 24)?;
        legacy.check_census(3, 9)?;
        Ok((sponsor, legacy))
    }

    /// Read exact retained bytes by run-relative path.
    /// # Errors
    /// Refuses a path absent from this selected family.
    pub fn file(&self, path: &str) -> Result<&[u8], CorpusRefusal> {
        self.files
            .get(&format!("{}/{path}", self.run))
            .map(Vec::as_slice)
            .ok_or(CorpusRefusal::Path)
    }

    /// Historical source bytes retained outside the run namespace, addressed
    /// by both logical path and reviewed digest.
    /// # Errors
    /// Refuses any source absent from the authenticated source fixture.
    pub fn source(&self, path: &str, digest: [u8; 32]) -> Result<&[u8], CorpusRefusal> {
        let member = format!("source-history/{}/{path}", hex(&digest));
        self.files
            .get(&member)
            .map(Vec::as_slice)
            .ok_or(CorpusRefusal::Path)
    }

    /// All authenticated historical source pairs for epoch reconstruction.
    /// # Errors
    /// Refuses a malformed source-history member name or digest.
    pub fn epoch_sources(&self) -> Result<Vec<SourceVersion<'_>>, CorpusRefusal> {
        self.files
            .iter()
            .filter(|(member, _)| member.starts_with("source-history/"))
            .map(|(member, bytes)| {
                let rest = member
                    .strip_prefix("source-history/")
                    .ok_or(CorpusRefusal::Path)?;
                let (digest, path) = rest.split_once('/').ok_or(CorpusRefusal::Path)?;
                let digest: [u8; 32] = decode_hex(digest)?
                    .try_into()
                    .map_err(|_| CorpusRefusal::Path)?;
                Ok(SourceVersion {
                    path,
                    digest,
                    bytes,
                })
            })
            .collect()
    }

    /// Replace one already acquired byte subject for a focused inner-boundary test.
    /// # Errors
    /// Refuses a path not retained in the selected packet.
    pub fn replace_after_acquisition(
        &mut self,
        path: &str,
        bytes: Vec<u8>,
    ) -> Result<(), CorpusRefusal> {
        let path = format!("{}/{path}", self.run);
        let original = self.files.get_mut(&path).ok_or(CorpusRefusal::Path)?;
        *original = bytes;
        Ok(())
    }

    /// Recount an acquired family's actual subject and exchange files.
    /// # Errors
    /// Refuses a missing, extra or mistyped census entry.
    pub fn recheck_census(&self) -> Result<(), CorpusRefusal> {
        let (records, exchanges) = match self.family {
            CaptureFamily::Clauses => (32, 102),
            CaptureFamily::Policy | CaptureFamily::LegacyCopy => (3, 9),
            CaptureFamily::Scaling => (17, 52),
            CaptureFamily::Histories => (168, 504),
            CaptureFamily::Sponsor => (8, 24),
            CaptureFamily::Pilot => return Err(CorpusRefusal::Census),
        };
        self.check_census(records, exchanges)
    }

    fn family_file(&self, path: &str) -> Result<&[u8], CorpusRefusal> {
        self.file(&format!("{}/{path}", self.name))
    }

    fn json(&self, path: &str) -> Result<Value, CorpusRefusal> {
        serde_json::from_slice(self.file(path)?).map_err(|_| CorpusRefusal::Encoding)
    }

    fn family_json(&self, path: &str) -> Result<Value, CorpusRefusal> {
        serde_json::from_slice(self.family_file(path)?).map_err(|_| CorpusRefusal::Encoding)
    }

    fn check_seals(&self) -> Result<(), CorpusRefusal> {
        self.check_listing("retention.sha256", "")?;
        self.check_listing("build/retention.sha256", "build/")?;
        self.check_listing(
            &format!("{}/retention.sha256", self.name),
            &format!("{}/", self.name),
        )?;
        Ok(())
    }

    fn check_campaign_seal(&self) -> Result<(), CorpusRefusal> {
        let listing = std::str::from_utf8(self.file("campaign-retention.sha256")?)
            .map_err(|_| CorpusRefusal::Seal)?;
        let paths = [
            "retention.sha256",
            "build/retention.sha256",
            "sponsor/retention.sha256",
            "legacy-copy/retention.sha256",
        ];
        if listing.lines().count() != paths.len() {
            return Err(CorpusRefusal::Seal);
        }
        for (line, path) in listing.lines().zip(paths) {
            let (digest, listed) = line.split_once("  ").ok_or(CorpusRefusal::Seal)?;
            if listed != path || digest != hex(&sha256(self.file(path)?)) {
                return Err(CorpusRefusal::Seal);
            }
        }
        let prepared =
            std::str::from_utf8(self.file("prepare.log")?).map_err(|_| CorpusRefusal::Seal)?;
        let marker = format!("KFF_PREPARE_DONE run={} exit=0", self.run);
        if prepared.lines().last() != Some(marker.as_str())
            || prepared
                .lines()
                .filter(|line| line.starts_with("KFF_PREPARE_DONE "))
                .count()
                != 1
        {
            return Err(CorpusRefusal::Seal);
        }
        let campaign =
            std::str::from_utf8(self.file("campaign.log")?).map_err(|_| CorpusRefusal::Seal)?;
        let marker = format!("KFF_CAMPAIGN_DONE run={} exit=0", self.run);
        if campaign.lines().filter(|line| *line == marker).count() != 1 {
            return Err(CorpusRefusal::Seal);
        }
        Ok(())
    }

    fn check_listing(&self, listing: &str, prefix: &str) -> Result<(), CorpusRefusal> {
        let text = std::str::from_utf8(self.file(listing)?).map_err(|_| CorpusRefusal::Seal)?;
        for line in text.lines() {
            let (digest, relative) = line.split_once("  ").ok_or(CorpusRefusal::Seal)?;
            if digest.len() != 64 || relative.starts_with('/') || relative.contains("..") {
                return Err(CorpusRefusal::Seal);
            }
            if prefix.is_empty()
                && ["clauses", "policy", "scaling", "histories"]
                    .iter()
                    .any(|family| {
                        *family != self.name && relative == format!("{family}/capture.env")
                    })
            {
                continue;
            }
            let named = format!("{prefix}{relative}");
            if let Ok(bytes) = self.file(&named) {
                if hex(&sha256(bytes)) != digest {
                    return Err(CorpusRefusal::Seal);
                }
            } else if self.epoch == CaptureEpoch::Sponsor {
                if self
                    .archive_addressed
                    .get(&named)
                    .is_none_or(|(expected, _)| expected != digest)
                {
                    return Err(CorpusRefusal::Seal);
                }
            } else if !self.archive_addressed(relative)
                && !matches!(
                    relative,
                    "test.log"
                        | "debug.log"
                        | "node.out"
                        | "unit.log"
                        | "transport-result.json"
                        | "keyed_fixed_capture"
                )
                && !relative.ends_with("/settings.json")
            {
                return Err(CorpusRefusal::Seal);
            }
        }
        Ok(())
    }

    fn scaling_parent_only(&self, ordinal: usize) -> bool {
        self.name == "scaling"
            && ordinal <= 12048
            && ordinal % 3 == 0
            && !SCALING_FULL.contains(&ordinal)
            && self
                .family_file(&format!("subject-{ordinal:05}.transaction"))
                .is_ok()
            && self
                .family_file(&format!("subject-{ordinal:05}.json"))
                .is_err()
    }

    fn archive_addressed(&self, path: &str) -> bool {
        if self.name != "scaling" {
            return false;
        }
        if let Some(name) = path.strip_prefix("subject-") {
            let Some(ordinal) = name
                .get(..5)
                .and_then(|digits| digits.parse::<usize>().ok())
            else {
                return false;
            };
            return self.scaling_parent_only(ordinal)
                && (std::path::Path::new(name).extension() == Some(std::ffi::OsStr::new("json"))
                    || name.contains("-parent-"));
        }
        if let Some(name) = path.strip_prefix("exchange-") {
            let Some(exchange) = name
                .get(..5)
                .and_then(|digits| digits.parse::<usize>().ok())
            else {
                return false;
            };
            if exchange > 12048
                || !(name.ends_with("-request.json") || name.ends_with("-response.json"))
            {
                return false;
            }
            let submit = match exchange % 3 {
                0 => exchange,
                1 => exchange + 2,
                _ => exchange + 1,
            };
            return self.scaling_parent_only(submit);
        }
        false
    }

    fn check_census(&self, records: usize, exchanges: usize) -> Result<(), CorpusRefusal> {
        let census = self.family_json("CENSUS.json")?;
        let declared = census["subjects"].as_array().ok_or(CorpusRefusal::Census)?;
        let expected = subjects(self.family);
        if declared.len() != expected.len() {
            return Err(CorpusRefusal::Census);
        }
        for (row, subject) in declared.iter().zip(expected) {
            let actual: CaptureSubject = serde_json::from_value(row["subject"].clone())
                .map_err(|_| CorpusRefusal::Census)?;
            if actual != subject {
                return Err(CorpusRefusal::Census);
            }
        }
        if matches!(
            self.family,
            CaptureFamily::Sponsor | CaptureFamily::LegacyCopy
        ) {
            self.check_s8c_census(&census, declared, records, exchanges)?;
        }
        let record_prefix = if self.family == CaptureFamily::LegacyCopy {
            "legacy-"
        } else {
            "subject-"
        };
        let prefix = format!("{}/{}/{record_prefix}", self.run, self.name);
        let record_count = self
            .files
            .keys()
            .filter(|path| {
                path.strip_prefix(&prefix)
                    .and_then(|name| name.strip_suffix(".json"))
                    .is_some_and(|digits| {
                        digits.len() == 5 && digits.bytes().all(|byte| byte.is_ascii_digit())
                    })
            })
            .count();
        let exchange_count = self
            .files
            .keys()
            .filter(|path| {
                path.starts_with(&format!("{}/{}/exchange-", self.run, self.name))
                    && path.ends_with("-request.json")
            })
            .count();
        if record_count != records || exchange_count != exchanges {
            return Err(CorpusRefusal::Census);
        }
        if self.name == "scaling" {
            let index = index()?;
            if index["scaling_retention"]
                != serde_json::json!({
                    "full_records": 17,
                    "parent_only": 4000,
                "archive_addressed_records": 4000,
                    "retained_exchanges": 52,
                    "archive_addressed_exchanges": 12000,
                    "full_ordinals": SCALING_FULL,
                })
            {
                return Err(CorpusRefusal::Census);
            }
            let parent_only = self
                .files
                .keys()
                .filter(|path| {
                    path.strip_prefix(&format!("{}/{}/subject-", self.run, self.name))
                        .and_then(|name| name.strip_suffix(".transaction"))
                        .and_then(|digits| digits.parse::<usize>().ok())
                        .is_some_and(|ordinal| self.scaling_parent_only(ordinal))
                })
                .count();
            if parent_only != 4000 {
                return Err(CorpusRefusal::Census);
            }
        }
        tracing::info!(target: "vectors::keyed_fixed_corpus", family = self.name, records, exchanges, outcome = "completed", "packet census checked");
        Ok(())
    }

    fn check_s8c_census(
        &self,
        census: &Value,
        declared: &[Value],
        records: usize,
        exchanges: usize,
    ) -> Result<(), CorpusRefusal> {
        let rows = declared
            .iter()
            .map(|row| {
                let subject: CaptureSubject = serde_json::from_value(row["subject"].clone())
                    .map_err(|_| CorpusRefusal::Census)?;
                let role: CaptureRole = serde_json::from_value(row["role"].clone())
                    .map_err(|_| CorpusRefusal::Census)?;
                let ordinal = row["ordinal"].as_u64().ok_or(CorpusRefusal::Census)?;
                let ordinal = usize::try_from(ordinal).map_err(|_| CorpusRefusal::Census)?;
                Ok((subject, role, ordinal))
            })
            .collect::<Result<Vec<_>, CorpusRefusal>>()?;
        validate_s8c_census(self.family, &rows, exchanges).map_err(|_| CorpusRefusal::Census)?;
        let record_prefix = if self.family == CaptureFamily::LegacyCopy {
            "legacy"
        } else {
            "subject"
        };
        for (subject, _, ordinal) in &rows {
            let record = self.family_json(&format!("{record_prefix}-{ordinal:05}.json"))?;
            let recorded: CaptureSubject = serde_json::from_value(record["subject"].clone())
                .map_err(|_| CorpusRefusal::Census)?;
            if recorded != *subject {
                return Err(CorpusRefusal::Census);
            }
        }
        let primary = rows
            .iter()
            .filter(|row| row.1 == CaptureRole::Primary)
            .count();
        let member = rows
            .iter()
            .filter(|row| row.1 == CaptureRole::SupportingMember)
            .count();
        let payment = rows
            .iter()
            .filter(|row| row.1 == CaptureRole::SupportingPayment)
            .count();
        if census["family"] != serde_json::json!(self.family)
            || census["records"] != records
            || census["exchanges"] != exchanges
            || census["primary"] != primary
            || census["supporting_member"] != member
            || census["supporting_payment"] != payment
        {
            return Err(CorpusRefusal::Census);
        }
        Ok(())
    }

    fn freezes(&self) -> Result<BTreeMap<[u8; 32], ValidatedFreeze>, CorpusRefusal> {
        let historical_sources = self.epoch_sources()?;
        let mut freezes = BTreeMap::new();
        for (path, encoded) in &self.files {
            if path.starts_with(&format!("{}/{}/freeze-", self.run, self.name))
                && std::path::Path::new(path).extension() == Some(std::ffi::OsStr::new("bin"))
            {
                let freeze = FreezeManifest::reconstruct_epoch_freeze(
                    self.epoch,
                    encoded,
                    &historical_sources,
                )
                .map_err(|_| CorpusRefusal::Record)?;
                freezes.insert(freeze.identity(), freeze);
            }
        }
        if freezes.is_empty() {
            return Err(CorpusRefusal::Record);
        }
        Ok(freezes)
    }

    fn provenance(&self, identity: [u8; 32]) -> Result<CaptureProvenance, CorpusRefusal> {
        let prepared = self.json("build/prepared.json")?;
        let field = |name: &str| prepared[name].as_str().ok_or(CorpusRefusal::Provenance);
        let digest = |name: &str| -> Result<[u8; 32], CorpusRefusal> {
            decode_hex(field(name)?)?
                .try_into()
                .map_err(|_| CorpusRefusal::Provenance)
        };
        let result = CaptureProvenance {
            freeze_identity: identity,
            executable_digest: digest("capture_binary")?,
            executor_digest: digest("executor")?,
            dependency_lock_digest: digest("dependency_lock")?,
            daemon_digest: digest("daemon_digest")?,
            framework_revision: field("framework")?.to_owned(),
            zk_library_digest: digest("zk_library_digest")?,
            image_identity: field("image")?.to_owned(),
        };
        let observed = self.family_json("observed-provenance.json")?;
        for (name, digest) in [
            ("executable_digest", result.executable_digest),
            ("executor_digest", result.executor_digest),
            ("dependency_lock_digest", result.dependency_lock_digest),
            ("daemon_digest", result.daemon_digest),
            ("zk_library_digest", result.zk_library_digest),
        ] {
            if raw_bytes(&observed[name])?.as_slice() != digest.as_slice() {
                return Err(CorpusRefusal::Provenance);
            }
        }
        if observed["zk_library_loading"] != "present-not-loaded" {
            return Err(CorpusRefusal::Provenance);
        }
        let handshake = self.family_json("executor-handshake.json")?;
        if handshake["protocol_schema"] != 9
            || handshake["adapter_name"] != "elements-native-executor"
            || handshake["framework_revision"] != result.framework_revision.as_str()
            || handshake["intended_executed_tip"] != result.framework_revision.as_str()
        {
            return Err(CorpusRefusal::Provenance);
        }
        let final_state = self.family_json("container-final-state.json")?;
        let image = final_state
            .as_array()
            .and_then(|states| states.first())
            .and_then(|state| state["Image"].as_str())
            .ok_or(CorpusRefusal::Provenance)?;
        if image != result.image_identity {
            return Err(CorpusRefusal::Provenance);
        }
        Ok(result)
    }

    fn check_response_outcome(
        submitted: &[u8],
        response: &NativeOperationResponse,
    ) -> Result<(), CorpusRefusal> {
        let transaction: elements::Transaction =
            elements::encode::deserialize(submitted).map_err(|_| CorpusRefusal::Record)?;
        let weight = u64::try_from(transaction.weight()).map_err(|_| CorpusRefusal::Record)?;
        if response.resources.transaction_weight != Some(weight) {
            return Err(CorpusRefusal::Record);
        }
        if let Some(readback) = &response.mined_readback {
            if readback.witness_transaction_id != transaction.wtxid().to_string() {
                return Err(CorpusRefusal::Record);
            }
        } else {
            let detail: Value = serde_json::from_str(
                response
                    .observed_detail
                    .as_deref()
                    .ok_or(CorpusRefusal::Record)?,
            )
            .map_err(|_| CorpusRefusal::Record)?;
            let txid = transaction.txid().to_string();
            if detail["detail"]["txid"].as_str() != Some(txid.as_str()) {
                return Err(CorpusRefusal::Record);
            }
        }
        Ok(())
    }

    fn check_outcome(record: &Record) -> Result<(), CorpusRefusal> {
        Self::check_response_outcome(&record.row.submitted, &record.row.response)
    }

    fn record(&self, ordinal: usize) -> Result<Record, CorpusRefusal> {
        let name = format!("subject-{ordinal:05}");
        let value = self.family_json(&format!("{name}.json"))?;
        self.record_value(&name, &value)
    }

    fn record_value(&self, name: &str, value: &Value) -> Result<Record, CorpusRefusal> {
        let ordinal = name
            .strip_prefix("subject-")
            .ok_or(CorpusRefusal::Record)?
            .parse::<usize>()
            .map_err(|_| CorpusRefusal::Record)?;
        let request: NativeOperationRequest =
            serde_json::from_value(value["request"].clone()).map_err(|_| CorpusRefusal::Record)?;
        let response: NativeOperationResponse =
            serde_json::from_value(value["response"].clone()).map_err(|_| CorpusRefusal::Record)?;
        let subject: CaptureSubject =
            serde_json::from_value(value["subject"].clone()).map_err(|_| CorpusRefusal::Record)?;
        let key: [u8; 32] = raw_bytes(&value["key"])?
            .try_into()
            .map_err(|_| CorpusRefusal::Record)?;
        let asset: [u8; 32] = raw_bytes(&value["asset"])?
            .try_into()
            .map_err(|_| CorpusRefusal::Record)?;
        let amount = value["value"].as_u64().ok_or(CorpusRefusal::Record)?;
        if request.case.step != format!("fixed-submit-{ordinal}")
            || value["request"]
                != self.family_json(&format!("exchange-{ordinal:05}-request.json"))?
            || value["response"]
                != self.family_json(&format!("exchange-{ordinal:05}-response.json"))?
        {
            return Err(CorpusRefusal::Record);
        }
        let submitted = raw_bytes(&value["submitted"])?;
        if submitted != self.family_file(&format!("{name}.transaction"))? {
            return Err(CorpusRefusal::Record);
        }
        let parents: Vec<Vec<u8>> =
            serde_json::from_value(value["parents"].clone()).map_err(|_| CorpusRefusal::Record)?;
        for (index, parent) in parents.iter().enumerate() {
            if parent != self.family_file(&format!("{name}-parent-{index}.transaction"))? {
                return Err(CorpusRefusal::Record);
            }
        }
        let identity: [u8; 32] = raw_bytes(&value["freeze"])?
            .try_into()
            .map_err(|_| CorpusRefusal::Record)?;
        let provenance = self.provenance(identity)?;
        let readback = if value["response"]["mined_readback"].is_null() {
            None
        } else {
            Some(raw_bytes(
                &value["response"]["mined_readback"]["raw_transaction"],
            )?)
        };
        let sponsor_index = value["sponsor_index"]
            .as_u64()
            .ok_or(CorpusRefusal::Record)?;
        let sponsor_index = usize::try_from(sponsor_index).map_err(|_| CorpusRefusal::Record)?;
        let row = JournalRow {
            request,
            response,
            submitted_digest: sha256(&submitted),
            parent_digests: parents.iter().map(|parent| sha256(parent)).collect(),
            submitted,
            parent_bytes: parents,
            readback,
            expected_provenance: provenance.clone(),
            observed_provenance: provenance,
        };
        Ok(Record {
            ordinal,
            subject,
            identity,
            key,
            asset,
            amount,
            sponsor_index,
            row,
            control: value["control"].clone(),
        })
    }

    fn check_adjacency(&self, record: &Record) -> Result<(), CorpusRefusal> {
        let submit = record.ordinal;
        let sign = submit.checked_sub(1).ok_or(CorpusRefusal::Record)?;
        let fund = submit.checked_sub(2).ok_or(CorpusRefusal::Record)?;
        let fund_request = self.family_json(&format!("exchange-{fund:05}-request.json"))?;
        let fund_response = self.family_json(&format!("exchange-{fund:05}-response.json"))?;
        let sign_request = self.family_json(&format!("exchange-{sign:05}-request.json"))?;
        let sign_response = self.family_json(&format!("exchange-{sign:05}-response.json"))?;
        let expected_sign_step = if record.subject == CaptureSubject::SponsorPayment {
            format!("fixed-payment-sign-{sign}")
        } else {
            format!("fixed-sign-{sign}")
        };
        if fund_request["case"]["operation"] != "fund_sponsor"
            || sign_request["case"]["operation"] != "sign_sponsor"
            || fund_request["case"]["step"] != format!("fixed-fund-{fund}")
            || sign_request["case"]["step"] != expected_sign_step
            || fund_request["case"] != fund_response["case"]
            || sign_request["case"] != sign_response["case"]
            || sign_request["subject"]["sponsor_input_index"] != record.sponsor_index
            || sign_request["subject"]["finalized_transaction"]
                != sign_response["signature_bound_to"]
        {
            return Err(CorpusRefusal::Record);
        }
        let tx =
            TargetTransaction::decode(&record.row.submitted).map_err(|_| CorpusRefusal::Record)?;
        if raw_bytes(&sign_response["signature_bound_to"])? != tx.encode_without_witness() {
            return Err(CorpusRefusal::Record);
        }
        let sponsor = tx
            .inputs()
            .get(record.sponsor_index)
            .ok_or(CorpusRefusal::Record)?;
        let sponsor_parent = record
            .row
            .parent_bytes
            .get(record.sponsor_index)
            .ok_or(CorpusRefusal::Record)?;
        let sponsor_parent =
            TargetTransaction::decode(sponsor_parent).map_err(|_| CorpusRefusal::Record)?;
        let funded = fund_response["funded_outputs"]
            .as_array()
            .ok_or(CorpusRefusal::Record)?;
        let outpoint = &sign_request["subject"]["sponsor_outpoint"];
        if !funded.iter().any(|output| output["outpoint"] == *outpoint)
            || outpoint["txid"] != sponsor.outpoint().txid().to_target_display()
            || outpoint["vout"] != sponsor.outpoint().index()
            || sponsor_parent
                .outputs()
                .get(sponsor.outpoint().index() as usize)
                .is_none()
        {
            return Err(CorpusRefusal::Record);
        }
        let witness = tx
            .witnesses()
            .get(record.sponsor_index)
            .ok_or(CorpusRefusal::Record)?;
        let signed_witness: Vec<Vec<u8>> =
            serde_json::from_value(sign_response["sponsor_witness"].clone())
                .map_err(|_| CorpusRefusal::Record)?;
        if signed_witness != witness.stack() {
            return Err(CorpusRefusal::Record);
        }
        Ok(())
    }

    fn payment_parent_count(&self, payment: &[u8]) -> Result<usize, CorpusRefusal> {
        let prefix = format!("{}/{}/subject-", self.run, self.name);
        let mut count = 0;
        for path in self.files.keys().filter(|path| {
            path.starts_with(&prefix)
                && std::path::Path::new(path).extension() == Some(std::ffi::OsStr::new("json"))
        }) {
            let ordinal = path
                .strip_prefix(&prefix)
                .and_then(|name| name.strip_suffix(".json"))
                .ok_or(CorpusRefusal::Record)?
                .parse::<usize>()
                .map_err(|_| CorpusRefusal::Record)?;
            let record = self.record(ordinal)?;
            count += record
                .row
                .parent_bytes
                .iter()
                .enumerate()
                .filter(|(index, parent)| {
                    *index != record.sponsor_index && parent.as_slice() == payment
                })
                .count();
        }
        Ok(count)
    }

    #[allow(clippy::too_many_lines)] // One payment's fund, sign, submit, and member-parent evidence form one join.
    fn check_payment(&self, submit: usize) -> Result<(), CorpusRefusal> {
        let fund = submit.checked_sub(2).ok_or(CorpusRefusal::Record)?;
        let sign = submit.checked_sub(1).ok_or(CorpusRefusal::Record)?;
        let fund_request = self.family_json(&format!("exchange-{fund:05}-request.json"))?;
        let fund_response: NativeOperationResponse =
            serde_json::from_value(self.family_json(&format!("exchange-{fund:05}-response.json"))?)
                .map_err(|_| CorpusRefusal::Record)?;
        let sign_request = self.family_json(&format!("exchange-{sign:05}-request.json"))?;
        let sign_response: NativeOperationResponse =
            serde_json::from_value(self.family_json(&format!("exchange-{sign:05}-response.json"))?)
                .map_err(|_| CorpusRefusal::Record)?;
        let submit_request = self.family_json(&format!("exchange-{submit:05}-request.json"))?;
        let submit_response: NativeOperationResponse = serde_json::from_value(
            self.family_json(&format!("exchange-{submit:05}-response.json"))?,
        )
        .map_err(|_| CorpusRefusal::Record)?;
        if fund_request["case"]["operation"] != "fund_sponsor"
            || fund_request["case"]["step"] != format!("fixed-fund-{fund}")
            || sign_request["case"]["operation"] != "sign_sponsor"
            || sign_request["case"]["step"] != format!("fixed-payment-sign-{sign}")
            || submit_request["case"]["operation"] != "submit"
            || submit_request["case"]["step"] != format!("fixed-payment-submit-{submit}")
            || serde_json::to_value(&fund_response.case).map_err(|_| CorpusRefusal::Record)?
                != fund_request["case"]
            || serde_json::to_value(&sign_response.case).map_err(|_| CorpusRefusal::Record)?
                != sign_request["case"]
            || serde_json::to_value(&submit_response.case).map_err(|_| CorpusRefusal::Record)?
                != submit_request["case"]
        {
            return Err(CorpusRefusal::Record);
        }
        let payment = self.family_file(&format!("payment-{submit:05}.transaction"))?;
        let parent = self.family_file(&format!("payment-{submit:05}-parent.transaction"))?;
        let transaction = TargetTransaction::decode(payment).map_err(|_| CorpusRefusal::Record)?;
        let stripped = transaction.encode_without_witness();
        let accepted_txid = elements::encode::deserialize::<elements::Transaction>(payment)
            .map_err(|_| CorpusRefusal::Record)?
            .txid()
            .to_string();
        let sponsor_index = usize::try_from(
            sign_request["subject"]["sponsor_input_index"]
                .as_u64()
                .ok_or(CorpusRefusal::Record)?,
        )
        .map_err(|_| CorpusRefusal::Record)?;
        let sponsor_input = transaction
            .inputs()
            .get(sponsor_index)
            .ok_or(CorpusRefusal::Record)?;
        let funded = fund_response
            .funded_outputs
            .first()
            .ok_or(CorpusRefusal::Record)?;
        let readback = fund_response
            .mined_readback
            .as_ref()
            .ok_or(CorpusRefusal::Record)?;
        if fund_response.funded_outputs.len() != 1
            || bind_sponsor_parent(readback, funded).map_err(|_| CorpusRefusal::Record)? != parent
            || funded.outpoint.txid != sponsor_input.outpoint().txid().to_target_display()
            || funded.outpoint.vout != sponsor_input.outpoint().index()
            || sign_request["subject"]["sponsor_outpoint"]
                != serde_json::to_value(&funded.outpoint).map_err(|_| CorpusRefusal::Record)?
            || raw_bytes(&sign_request["subject"]["finalized_transaction"])? != stripped
            || sign_response.signature_bound_to.as_deref() != Some(stripped.as_slice())
            || sign_response.sponsor_witness.as_slice()
                != transaction
                    .witnesses()
                    .get(sponsor_index)
                    .ok_or(CorpusRefusal::Record)?
                    .stack()
            || raw_bytes(&submit_request["subject"]["transaction_bytes"])? != payment
            || submit_response
                .mined_readback
                .as_ref()
                .is_none_or(|readback| readback.raw_transaction.as_slice() != payment)
            || self.payment_parent_count(payment)? != 1
        {
            return Err(CorpusRefusal::Record);
        }
        let detail: Value = serde_json::from_str(
            submit_response
                .observed_detail
                .as_deref()
                .ok_or(CorpusRefusal::Record)?,
        )
        .map_err(|_| CorpusRefusal::Record)?;
        if submit_response.observed_layer != ObservedOutcomeLayer::Accepted
            || detail["route"] != "relay-and-template"
            || detail["relay_allowed"] != true
            || submit_response.accepted_txid.as_deref() != Some(accepted_txid.as_str())
        {
            return Err(CorpusRefusal::Record);
        }
        let spent_parent = TargetTransaction::decode(parent).map_err(|_| CorpusRefusal::Record)?;
        let spent = spent_parent
            .outputs()
            .get(sponsor_input.outpoint().index() as usize)
            .ok_or(CorpusRefusal::Record)?;
        let witness = transaction
            .witnesses()
            .get(sponsor_index)
            .ok_or(CorpusRefusal::Record)?;
        transaction::keyed_fixed_sponsor::verify_keyed_fixed_sponsor(
            &stripped,
            sponsor_index,
            spent,
            witness.stack(),
        )
        .map_err(|_| CorpusRefusal::Witness)?;
        Self::check_response_outcome(payment, &submit_response)
    }

    fn check_payments(&self) -> Result<(), CorpusRefusal> {
        if self.family == CaptureFamily::Clauses {
            for submit in [87, 96] {
                self.check_payment(submit)?;
            }
            tracing::info!(target: "vectors::keyed_fixed_corpus", payments = 2, outcome = "completed", "retained payment submissions admitted");
        }
        Ok(())
    }

    fn check_funded_coin_census(&self) -> Result<(), CorpusRefusal> {
        let prefix = format!("{}/{}/exchange-", self.run, self.name);
        let requests = self
            .files
            .keys()
            .filter(|path| path.starts_with(&prefix) && path.ends_with("-request.json"))
            .collect::<Vec<_>>();
        let mut funded = BTreeMap::<WireOutpoint, (usize, usize)>::new();
        for path in &requests {
            let ordinal = path
                .strip_prefix(&prefix)
                .and_then(|name| name.strip_suffix("-request.json"))
                .ok_or(CorpusRefusal::Record)?
                .parse::<usize>()
                .map_err(|_| CorpusRefusal::Record)?;
            let request = self.family_json(&format!("exchange-{ordinal:05}-request.json"))?;
            if request["case"]["operation"] == "fund_sponsor" {
                let response: NativeOperationResponse = serde_json::from_value(
                    self.family_json(&format!("exchange-{ordinal:05}-response.json"))?,
                )
                .map_err(|_| CorpusRefusal::Record)?;
                if response.funded_outputs.len() != 1 {
                    return Err(CorpusRefusal::Record);
                }
                for output in response.funded_outputs {
                    if funded.insert(output.outpoint, (ordinal, 0)).is_some() {
                        return Err(CorpusRefusal::Record);
                    }
                }
            }
        }
        for path in &requests {
            let ordinal = path
                .strip_prefix(&prefix)
                .and_then(|name| name.strip_suffix("-request.json"))
                .ok_or(CorpusRefusal::Record)?
                .parse::<usize>()
                .map_err(|_| CorpusRefusal::Record)?;
            let request = self.family_json(&format!("exchange-{ordinal:05}-request.json"))?;
            if request["case"]["operation"] == "submit" {
                let submitted = raw_bytes(&request["subject"]["transaction_bytes"])?;
                let transaction =
                    TargetTransaction::decode(&submitted).map_err(|_| CorpusRefusal::Record)?;
                for input in transaction.inputs() {
                    let outpoint = WireOutpoint {
                        txid: input.outpoint().txid().to_target_display(),
                        vout: input.outpoint().index(),
                    };
                    if let Some((_, count)) = funded.get_mut(&outpoint) {
                        *count += 1;
                    }
                }
            }
        }
        let mut spent = 0usize;
        let mut unspent = 0usize;
        for (ordinal, count) in funded.values() {
            let expected = usize::from(self.family != CaptureFamily::Scaling || *ordinal != 12049);
            if *count != expected {
                return Err(CorpusRefusal::Record);
            }
            if *count == 0 {
                unspent += 1;
            } else {
                spent += 1;
            }
        }
        tracing::info!(target: "vectors::keyed_fixed_corpus", family = self.name, funded_coins = funded.len(), spent, unspent, outcome = "completed", "funded coin census checked");
        Ok(())
    }

    fn check_key_path_attempt(
        record: &Record,
        freeze: &ValidatedFreeze,
    ) -> Result<(), CorpusRefusal> {
        if record.subject != CaptureSubject::KeyPathAttempt {
            return Ok(());
        }
        let tx =
            TargetTransaction::decode(&record.row.submitted).map_err(|_| CorpusRefusal::Witness)?;
        let program = &freeze.manifest().descriptor.output_program;
        let member = tx
            .inputs()
            .iter()
            .zip(&record.row.parent_bytes)
            .enumerate()
            .find_map(|(index, (input, parent))| {
                let parent = TargetTransaction::decode(parent).ok()?;
                parent
                    .outputs()
                    .get(input.outpoint().index() as usize)
                    .filter(|output| output.program() == program)
                    .map(|_| index)
            })
            .ok_or(CorpusRefusal::Witness)?;
        let witness = tx.witnesses().get(member).ok_or(CorpusRefusal::Witness)?;
        if witness.stack().len() != 1 || witness.stack()[0].len() != 64 {
            return Err(CorpusRefusal::Witness);
        }
        Ok(())
    }

    /// Check every retained exchange timing prefix and the sampled cgroup scope.
    /// # Errors
    /// Refuses a missing, duplicate or malformed exchange measurement.
    pub fn check_measurements(&self, timing: &[u8]) -> Result<usize, CorpusRefusal> {
        let (expected_exchanges, expected_lines) = match self.family {
            CaptureFamily::Clauses => (102, 140),
            CaptureFamily::Policy => (9, 17),
            CaptureFamily::Scaling => (12052, 16262),
            CaptureFamily::Histories => (504, 690),
            _ => return Err(CorpusRefusal::Measurement),
        };
        let text = std::str::from_utf8(timing).map_err(|_| CorpusRefusal::Measurement)?;
        let mut prefixes = BTreeSet::new();
        let mut lines = 0usize;
        for line in text.lines() {
            let event: Value =
                serde_json::from_str(line).map_err(|_| CorpusRefusal::Measurement)?;
            lines += 1;
            if event["fields"]["message"] == "fixed-form native exchange retained" {
                let prefix = event["fields"]["prefix"]
                    .as_str()
                    .ok_or(CorpusRefusal::Measurement)?;
                let wall = event["fields"]["wall_us"]
                    .as_u64()
                    .ok_or(CorpusRefusal::Measurement)?;
                if wall == 0 || !prefixes.insert(prefix.to_owned()) {
                    return Err(CorpusRefusal::Measurement);
                }
            }
        }
        if lines != expected_lines
            || prefixes.len() != expected_exchanges
            || (1..=expected_exchanges)
                .any(|ordinal| !prefixes.contains(&format!("exchange-{ordinal:05}")))
        {
            return Err(CorpusRefusal::Measurement);
        }
        let before = self.family_json("before-resources.json")?;
        let after = self.family_json("after-resources.json")?;
        let cgroup = std::str::from_utf8(self.family_file("container-cgroup.txt")?)
            .map_err(|_| CorpusRefusal::Measurement)?;
        if before["cgroup_path"] != after["cgroup_path"]
            || before["scope"] != after["scope"]
            || before["cgroup_membership"]
                .as_str()
                .is_none_or(|membership| !cgroup.contains(membership.trim()))
            || before["epoch_ns"]
                .as_u64()
                .zip(after["epoch_ns"].as_u64())
                .is_none_or(|(start, end)| end <= start)
        {
            return Err(CorpusRefusal::Measurement);
        }
        tracing::info!(target: "vectors::keyed_fixed_corpus", family = self.name, exchanges = prefixes.len(), lines, outcome = "completed", "retained measurements complete");
        Ok(prefixes.len())
    }

    /// Check the unmodified retained timing file.
    /// # Errors
    /// Refuses incomplete measurement evidence.
    pub fn check_retained_measurements(&self) -> Result<usize, CorpusRefusal> {
        self.check_measurements(self.family_file("phase-timing.jsonl")?)
    }

    /// Authenticate every scaling parent-only transaction against a retained
    /// MERGE member parent and its spent outpoint.
    /// # Errors
    /// Refuses an unused, unequal or unauthenticated parent-only byte subject.
    pub fn authenticate_scaling_parents(&self) -> Result<usize, CorpusRefusal> {
        if self.family != CaptureFamily::Scaling {
            return Err(CorpusRefusal::Census);
        }
        let mut parent_only = BTreeMap::new();
        for (path, bytes) in &self.files {
            let Some(ordinal) = path
                .strip_prefix(&format!("{}/{}/subject-", self.run, self.name))
                .and_then(|name| name.strip_suffix(".transaction"))
                .and_then(|digits| digits.parse::<usize>().ok())
            else {
                continue;
            };
            if self.scaling_parent_only(ordinal) {
                let tx = TargetTransaction::decode(bytes).map_err(|_| CorpusRefusal::Record)?;
                let id = sha256d::Hash::hash(&tx.encode_without_witness()).to_byte_array();
                if parent_only.insert(id, bytes.as_slice()).is_some() {
                    return Err(CorpusRefusal::Record);
                }
            }
        }
        if parent_only.len() != 4000 {
            return Err(CorpusRefusal::Census);
        }
        for ordinal in SCALING_FULL {
            let record = self.record(ordinal)?;
            if !matches!(
                record.subject,
                CaptureSubject::WeightMerge(_) | CaptureSubject::StructuralWeight1300
            ) {
                continue;
            }
            let tx = TargetTransaction::decode(&record.row.submitted)
                .map_err(|_| CorpusRefusal::Record)?;
            for (index, bytes) in record.row.parent_bytes.iter().enumerate() {
                if index == record.sponsor_index {
                    continue;
                }
                let parent = TargetTransaction::decode(bytes).map_err(|_| CorpusRefusal::Record)?;
                let id = sha256d::Hash::hash(&parent.encode_without_witness()).to_byte_array();
                if tx.inputs()[index].outpoint().txid().internal() != &id {
                    return Err(CorpusRefusal::Record);
                }
                if let Some(retained) = parent_only.remove(&id)
                    && retained != bytes
                {
                    return Err(CorpusRefusal::Record);
                }
            }
        }
        if !parent_only.is_empty() {
            return Err(CorpusRefusal::Record);
        }
        tracing::info!(target: "vectors::keyed_fixed_corpus", parent_only = 4000, outcome = "completed", "scaling parent-only bytes authenticated");
        Ok(4000)
    }

    /// Re-admit every retained subject record once through its own freeze and
    /// the unchanged journal binder, after corpus-specific comparisons.
    /// # Errors
    /// Returns the first missing or unequal retained join.
    pub fn admit_all(&self) -> Result<usize, CorpusRefusal> {
        let start = std::time::Instant::now();
        let _span = tracing::info_span!(target: "vectors::keyed_fixed_corpus", "admit_all", family = self.name).entered();
        let freezes = self.freezes()?;
        let mut count = 0usize;
        let prefix = format!("{}/{}/subject-", self.run, self.name);
        for path in self.files.keys().filter(|path| {
            path.starts_with(&prefix)
                && std::path::Path::new(path).extension() == Some(std::ffi::OsStr::new("json"))
        }) {
            let ordinal = path
                .strip_prefix(&prefix)
                .and_then(|name| name.strip_suffix(".json"))
                .ok_or(CorpusRefusal::Record)?
                .parse::<usize>()
                .map_err(|_| CorpusRefusal::Record)?;
            let record = self.record(ordinal)?;
            if record.ordinal != ordinal {
                return Err(CorpusRefusal::Record);
            }
            let freeze = freezes.get(&record.identity).ok_or(CorpusRefusal::Record)?;
            let triple = freeze.manifest().descriptor.parameters;
            if record.key != triple.key
                || record.asset != triple.asset
                || record.amount != triple.value.get()
            {
                return Err(CorpusRefusal::Record);
            }
            self.check_adjacency(&record)?;
            Self::check_outcome(&record)?;
            Self::check_key_path_attempt(&record, freeze)?;
            let control = if let Some(expected) = expected_control(record.subject) {
                let control_value = record.control.as_object().ok_or(CorpusRefusal::Control)?;
                let step = control_value["request"]["case"]["step"]
                    .as_str()
                    .ok_or(CorpusRefusal::Control)?;
                let control_ordinal = step
                    .strip_prefix("fixed-submit-")
                    .ok_or(CorpusRefusal::Control)?
                    .parse::<usize>()
                    .map_err(|_| CorpusRefusal::Control)?;
                if control_ordinal >= ordinal
                    || self.family_json(&format!("subject-{control_ordinal:05}.json"))?
                        != record.control
                {
                    return Err(CorpusRefusal::Control);
                }
                let control = self.record(control_ordinal)?;
                if control.subject != expected {
                    return Err(CorpusRefusal::Control);
                }
                Some(control)
            } else {
                if !record.control.is_null() {
                    return Err(CorpusRefusal::Control);
                }
                None
            };
            let control_binding = if let Some(ref control) = control {
                Some(PairControl {
                    subject: control.subject,
                    freeze: freezes
                        .get(&control.identity)
                        .ok_or(CorpusRefusal::Control)?,
                    row: &control.row,
                    sponsor_index: control.sponsor_index,
                })
            } else {
                None
            };
            let binding = JournalBinding {
                family: self.family,
                subject: record.subject,
                sponsor_index: record.sponsor_index,
                prepared_provenance: &record.row.expected_provenance,
                control: control_binding,
            };
            if self.family == CaptureFamily::Sponsor
                && record.subject == CaptureSubject::SponsorPayment
            {
                validate_journal_transport(freeze, &record.row).map_err(CorpusRefusal::Journal)?;
            } else {
                bind_journal(freeze, &record.row, &binding).map_err(CorpusRefusal::Journal)?;
            }
            if self.family == CaptureFamily::Sponsor {
                sponsor::check_sponsor_record(&record, freeze)?;
            }
            count += 1;
        }
        self.check_payments()?;
        self.check_funded_coin_census()?;
        tracing::info!(target: "vectors::keyed_fixed_corpus", family = self.name, records = count, wall_ms = start.elapsed().as_millis(), outcome = "completed", "offline admission completed");
        Ok(count)
    }
}

struct Record {
    ordinal: usize,
    subject: CaptureSubject,
    identity: [u8; 32],
    key: [u8; 32],
    asset: [u8; 32],
    amount: u64,
    sponsor_index: usize,
    row: JournalRow,
    control: Value,
}

/// A bounded-model site credit, or an explicit OPEN result with its failed premise.
#[derive(Clone, Debug)]
pub struct Attribution {
    pub subject: CaptureSubject,
    pub instruction: Option<usize>,
    pub reason: &'static str,
}

fn program_fact(program: &[u8]) -> stack_model::ProgramFact {
    if let [version, width, payload @ ..] = program {
        let decoded_version = match *version {
            0 => Some(0),
            0x51..=0x60 => Some(*version - 0x50),
            _ => None,
        };
        if let Some(version) = decoded_version
            && usize::from(*width) == payload.len()
            && (2..=40).contains(&payload.len())
        {
            return stack_model::ProgramFact::Witness {
                version,
                payload: payload.to_vec(),
            };
        }
    }
    stack_model::ProgramFact::Other {
        sha256: sha256(program),
    }
}

fn model_output(output: &TargetOutput) -> Result<stack_model::Output, CorpusRefusal> {
    let asset = match output.asset() {
        AssetField::Explicit(asset) => stack_model::Field {
            payload: asset.internal().to_vec(),
            prefix: 1,
        },
        AssetField::Commitment(bytes) => stack_model::Field {
            payload: bytes[1..].to_vec(),
            prefix: bytes[0],
        },
        _ => return Err(CorpusRefusal::Record),
    };
    let value = match output.value() {
        ValueField::Explicit(value) => stack_model::Field {
            payload: value.to_le_bytes().to_vec(),
            prefix: 1,
        },
        ValueField::Commitment(bytes) => stack_model::Field {
            payload: bytes[1..].to_vec(),
            prefix: bytes[0],
        },
        _ => return Err(CorpusRefusal::Record),
    };
    Ok(stack_model::Output {
        program: program_fact(output.program()),
        asset,
        value,
    })
}

fn model_facts(
    record: &Record,
    freeze: &ValidatedFreeze,
) -> Result<stack_model::Facts, CorpusRefusal> {
    let transaction =
        TargetTransaction::decode(&record.row.submitted).map_err(|_| CorpusRefusal::Record)?;
    let mut inputs = Vec::new();
    let mut current = None;
    for (index, (input, bytes)) in transaction
        .inputs()
        .iter()
        .zip(&record.row.parent_bytes)
        .enumerate()
    {
        let parent = TargetTransaction::decode(bytes).map_err(|_| CorpusRefusal::Record)?;
        let spent = parent
            .outputs()
            .get(input.outpoint().index() as usize)
            .ok_or(CorpusRefusal::Record)?;
        if spent.program() == freeze.manifest().descriptor.output_program {
            current = Some(u32::try_from(index).map_err(|_| CorpusRefusal::Bound)?);
        }
        inputs.push(stack_model::Input {
            txid: *input.outpoint().txid().internal(),
            birth: input.outpoint().index(),
            flags: 0,
            output: model_output(spent)?,
        });
    }
    let outputs = transaction
        .outputs()
        .iter()
        .map(model_output)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(stack_model::Facts {
        current: current.ok_or(CorpusRefusal::Record)?,
        inputs,
        outputs,
    })
}

fn changed_coordinate(
    subject: CaptureSubject,
    record: &Record,
    control: &Record,
    freeze: &ValidatedFreeze,
) -> bool {
    let Ok(subject_tx) = TargetTransaction::decode(&record.row.submitted) else {
        return false;
    };
    let Ok(control_tx) = TargetTransaction::decode(&control.row.submitted) else {
        return false;
    };
    let Some(subject_first) = subject_tx.outputs().first() else {
        return false;
    };
    let Some(control_first) = control_tx.outputs().first() else {
        return false;
    };
    let expected = freeze.manifest().descriptor.parameters;
    match subject {
        CaptureSubject::ValueMinus => {
            subject_first.value() == ValueField::Explicit(expected.value.get() - 1)
                && control_first.value() == ValueField::Explicit(expected.value.get())
        }
        CaptureSubject::ValuePlus => {
            subject_first.value() == ValueField::Explicit(expected.value.get() + 1)
                && control_first.value() == ValueField::Explicit(expected.value.get())
        }
        CaptureSubject::WrongProgram => {
            subject_first.program() != freeze.manifest().descriptor.output_program
                && control_first.program() == freeze.manifest().descriptor.output_program
        }
        CaptureSubject::WrongKey => {
            let Ok(target) = target_elements::reviewed_elements_tapscript() else {
                return false;
            };
            let foreign = realization::keyed_fixed::KeyedFixedParameters {
                key: [43; 32],
                asset: expected.asset,
                value: expected.value,
            };
            let Ok(descriptor) = vectors::keyed_fixed_constructor::construct(
                &target,
                foreign,
                &vectors::keyed_fixed_constructor::PublicKeyedCurve,
            ) else {
                return false;
            };
            subject_first.program() == descriptor.output_program
                && subject_first.program() != freeze.manifest().descriptor.output_program
                && control_first.program() == freeze.manifest().descriptor.output_program
        }
        CaptureSubject::DisplacedSuccessor => {
            subject_first.program() != freeze.manifest().descriptor.output_program
                && subject_tx.outputs().get(1).is_some_and(|output| {
                    output.program() == freeze.manifest().descriptor.output_program
                })
        }
        CaptureSubject::MalformedPaymentValue | CaptureSubject::LaterPosition => {
            let Some(raw) = record.row.parent_bytes.first() else {
                return false;
            };
            let Some(control_raw) = control.row.parent_bytes.first() else {
                return false;
            };
            let Ok(parent) = TargetTransaction::decode(raw) else {
                return false;
            };
            let Ok(control_parent) = TargetTransaction::decode(control_raw) else {
                return false;
            };
            if subject == CaptureSubject::MalformedPaymentValue {
                parent.outputs().first().is_some_and(|output| {
                    output.value() == ValueField::Explicit(expected.value.get() - 1)
                }) && control_parent.outputs().first().is_some_and(|output| {
                    output.value() == ValueField::Explicit(expected.value.get())
                })
            } else {
                parent.outputs().get(1).is_some_and(|output| {
                    output.program() == freeze.manifest().descriptor.output_program
                }) && control_parent.outputs().first().is_some_and(|output| {
                    output.program() == freeze.manifest().descriptor.output_program
                })
            }
        }
        _ => false,
    }
}

impl Packet {
    /// Attribute seven opcode-class refusals only when paired byte facts and
    /// the bounded model identify one equality instruction.
    /// # Errors
    /// Refuses malformed retained rows; uncertain sites remain OPEN.
    pub fn attribute_sites(&self) -> Result<Vec<Attribution>, CorpusRefusal> {
        if self.family != CaptureFamily::Clauses {
            return Err(CorpusRefusal::Record);
        }
        let freezes = self.freezes()?;
        let expected = [
            (CaptureSubject::ValueMinus, 12),
            (CaptureSubject::ValuePlus, 12),
            (CaptureSubject::WrongProgram, 34),
            (CaptureSubject::WrongKey, 34),
            (CaptureSubject::DisplacedSuccessor, 34),
            (CaptureSubject::MalformedPaymentValue, 21),
            (CaptureSubject::LaterPosition, 26),
        ];
        let prefix = format!("{}/{}/subject-", self.run, self.name);
        let mut results = Vec::new();
        for (subject, predicted) in expected {
            let mut selected = None;
            for path in self.files.keys().filter(|path| {
                path.starts_with(&prefix)
                    && std::path::Path::new(path).extension() == Some(std::ffi::OsStr::new("json"))
            }) {
                let ordinal = path
                    .strip_prefix(&prefix)
                    .and_then(|name| name.strip_suffix(".json"))
                    .ok_or(CorpusRefusal::Record)?
                    .parse::<usize>()
                    .map_err(|_| CorpusRefusal::Record)?;
                let record = self.record(ordinal)?;
                if record.subject == subject {
                    selected = Some(record);
                    break;
                }
            }
            let record = selected.ok_or(CorpusRefusal::Record)?;
            let freeze = freezes.get(&record.identity).ok_or(CorpusRefusal::Record)?;
            let step = record.control["request"]["case"]["step"]
                .as_str()
                .ok_or(CorpusRefusal::Control)?;
            let ordinal = step
                .strip_prefix("fixed-submit-")
                .ok_or(CorpusRefusal::Control)?
                .parse::<usize>()
                .map_err(|_| CorpusRefusal::Control)?;
            let control = self.record(ordinal)?;
            if self.family_json(&format!("subject-{ordinal:05}.json"))? != record.control
                || control.subject != expected_control(subject).ok_or(CorpusRefusal::Control)?
            {
                return Err(CorpusRefusal::Control);
            }
            let control_facts = model_facts(&control, freeze);
            let subject_facts = model_facts(&record, freeze);
            let reason = if control_facts.is_err() || subject_facts.is_err() {
                "unrepresentable facts"
            } else if stack_model::evaluate(&freeze.manifest().descriptor.leaf, &control_facts?)
                .is_err()
            {
                "paired control failed"
            } else if !changed_coordinate(subject, &record, &control, freeze) {
                "coordinate not independently established"
            } else {
                match stack_model::evaluate(&freeze.manifest().descriptor.leaf, &subject_facts?) {
                    Err(refusal)
                        if refusal.instruction == predicted && refusal.kind == "unequal" =>
                    {
                        "unique equality"
                    }
                    Err(_) => "model disagreed",
                    Ok(_) => "model accepted subject",
                }
            };
            let instruction = (reason == "unique equality").then_some(predicted);
            tracing::info!(target: "vectors::keyed_fixed_corpus", ?subject, ?instruction, reason, model_only = true, "equality site attribution");
            results.push(Attribution {
                subject,
                instruction,
                reason,
            });
        }
        Ok(results)
    }
}
