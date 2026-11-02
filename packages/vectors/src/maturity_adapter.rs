//! Exact request comparisons admitted from an embedded Python reading.
//!
//! Derivation reads only embedded data and constructs its own operands. The
//! ordinary Python gate separately compares that data with the live adapter.
//! No interpreter, filesystem access or subprocess participates in derivation.

use std::sync::LazyLock;

use target_elements_conformance::constructor::tagged::sha256;

use crate::maturity_observer::decode_hex;

/// Exact source identity of the adapter whose request entries were compared.
pub const ADAPTER_SOURCE_SHA256: &str =
    "a0edebf53cdfa4c95e4658711ced62be280b5891ea49140edfce32776a768b00";
const MAXIMUM: usize = 4 * 1024 * 1024;
const FIXTURE: &[u8] = include_bytes!("../fixtures/adapter-request-comparisons/COMPARISONS");
static ADMITTED: LazyLock<Result<Vec<MaturityAdapterComparison>, MaturityAdapterRefusal>> =
    LazyLock::new(|| admit(FIXTURE));

/// The five request-side coordinates, in the fixture's fixed order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MaturityAdapterCase {
    /// Replace the nonempty body with whitespace.
    Blank,
    /// Extend trailing padding beyond the read bound.
    Oversized,
    /// Remove the final newline.
    Unterminated,
    /// Replace the closing object delimiter.
    MalformedJson,
    /// Add one undeclared execution-request field.
    UnknownField,
}

impl MaturityAdapterCase {
    /// Complete ordered comparison inventory.
    pub const ALL: [Self; 5] = [
        Self::Blank,
        Self::Oversized,
        Self::Unterminated,
        Self::MalformedJson,
        Self::UnknownField,
    ];

    /// Safety row name, within section 16.12.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Blank => "blank-request",
            Self::Oversized => "oversized-request",
            Self::Unterminated => "unterminated-request",
            Self::MalformedJson => "malformed-json",
            Self::UnknownField => "unknown-request-field",
        }
    }

    /// Exact Python entry invoked for both operands.
    #[must_use]
    pub const fn entry(self) -> &'static str {
        match self {
            Self::Blank | Self::Oversized | Self::Unterminated => "RequestReader.read",
            Self::MalformedJson => "RequestReader.read_object",
            Self::UnknownField => "require_execution_request_fields",
        }
    }

    /// Request-side refinement of the matrix's semantic request boundary.
    #[must_use]
    pub const fn boundary(self) -> &'static str {
        match self {
            Self::Blank | Self::Oversized | Self::Unterminated => "RequestFraming",
            Self::MalformedJson => "RequestJson",
            Self::UnknownField => "RequestFields",
        }
    }

    /// Exact refusal spelling emitted by the adapter.
    #[must_use]
    pub const fn refusal(self) -> &'static str {
        match self {
            Self::Blank => "blank_record",
            Self::Oversized => "oversized_record",
            Self::Unterminated => "unterminated_record",
            Self::MalformedJson => "malformed_record",
            Self::UnknownField => "EXECUTION_REQUEST_FIELD_CENSUS_FAILED",
        }
    }

    fn operands(self) -> (Vec<u8>, Vec<u8>) {
        let control = b"{}\n".to_vec();
        match self {
            Self::Blank => (control, b"  \n".to_vec()),
            Self::Oversized => {
                let mut subject = b"{}".to_vec();
                subject.extend(std::iter::repeat_n(b' ', MAXIMUM - 1));
                subject.push(b'\n');
                (control, subject)
            }
            Self::Unterminated => (control, b"{}".to_vec()),
            Self::MalformedJson => (control, b"{]\n".to_vec()),
            Self::UnknownField => (
                b"{\"schema\":8,\"case\":{},\"subject\":{}}\n".to_vec(),
                b"{\"schema\":8,\"case\":{},\"subject\":{},\"unknown\":0}\n".to_vec(),
            ),
        }
    }
}

/// Why a Python comparison reading cannot be credited.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MaturityAdapterRefusal {
    /// Malformed encoding, field inventory or line termination.
    Grammar,
    /// The fixture does not contain exactly five lines.
    RowCensus,
    /// A key, entry or boundary differs from its ordered expectation.
    RowBinding,
    /// An operand's exact bytes, length or digest differs.
    OperandBinding,
    /// The control did not accept.
    InvalidControl,
    /// The subject did not refuse.
    AcceptedSubject,
    /// The refusal does not name the expected class.
    WrongRefusal,
    /// The source digest differs from the independently pinned adapter.
    SourceDigest,
    /// The adapter name, version or protocol differs.
    AdapterIdentity,
}

/// One exact operand; the large padding subject is retained by length and digest.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MaturityAdapterOperand {
    bytes: Option<Vec<u8>>,
    length: usize,
    digest: [u8; 32],
}

impl MaturityAdapterOperand {
    fn new(bytes: Vec<u8>, retain_bytes: bool) -> Self {
        Self {
            length: bytes.len(),
            digest: sha256(&bytes),
            bytes: retain_bytes.then_some(bytes),
        }
    }
}

/// A retained exact pair and its source-bound, validated Python reading.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MaturityAdapterComparison {
    case: MaturityAdapterCase,
    language: &'static str,
    entry: &'static str,
    boundary: &'static str,
    phase: &'static str,
    maximum: usize,
    allowed: Vec<&'static str>,
    source_digest: &'static str,
    interpreter: String,
    control: MaturityAdapterOperand,
    subject: MaturityAdapterOperand,
}

impl MaturityAdapterComparison {
    /// Admit a pair from the embedded fixture, without external execution.
    ///
    /// # Errors
    /// Returns the exact fixture-admission failure, never comparison credit.
    pub fn admitted(case: MaturityAdapterCase) -> Result<Self, MaturityAdapterRefusal> {
        ADMITTED
            .as_ref()
            .map_err(|error| *error)?
            .iter()
            .find(|pair| pair.case == case)
            .cloned()
            .ok_or(MaturityAdapterRefusal::RowBinding)
    }

    /// Recheck every retained field against the admitted pair.
    ///
    /// # Errors
    /// Refuses altered operands or bindings rather than trusting a copied outcome.
    pub fn validate(&self) -> Result<MaturityAdapterCase, MaturityAdapterRefusal> {
        let expected = Self::admitted(self.case)?;
        if self.control != expected.control || self.subject != expected.subject {
            return Err(MaturityAdapterRefusal::OperandBinding);
        }
        if self != &expected {
            return Err(MaturityAdapterRefusal::RowBinding);
        }
        Ok(self.case)
    }
}

fn number(text: &str) -> Result<usize, MaturityAdapterRefusal> {
    if text.is_empty()
        || (text.len() > 1 && text.starts_with('0'))
        || !text.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(MaturityAdapterRefusal::Grammar);
    }
    text.parse().map_err(|_| MaturityAdapterRefusal::Grammar)
}

fn text(length: &str, hex: &str) -> Result<String, MaturityAdapterRefusal> {
    let bytes = decode_hex(hex).map_err(|_| MaturityAdapterRefusal::Grammar)?;
    if number(length)? != bytes.len() {
        return Err(MaturityAdapterRefusal::Grammar);
    }
    String::from_utf8(bytes).map_err(|_| MaturityAdapterRefusal::Grammar)
}

fn digest(text: &str) -> Result<[u8; 32], MaturityAdapterRefusal> {
    decode_hex(text)
        .map_err(|_| MaturityAdapterRefusal::Grammar)?
        .try_into()
        .map_err(|_| MaturityAdapterRefusal::Grammar)
}

pub(crate) fn admit(
    bytes: &[u8],
) -> Result<Vec<MaturityAdapterComparison>, MaturityAdapterRefusal> {
    use MaturityAdapterRefusal as Refusal;
    if bytes.len() > 8192 || bytes.contains(&b'\r') || !bytes.ends_with(b"\n") {
        return Err(Refusal::Grammar);
    }
    let source = std::str::from_utf8(bytes).map_err(|_| Refusal::Grammar)?;
    let lines: Vec<_> = source
        .strip_suffix('\n')
        .ok_or(Refusal::Grammar)?
        .split('\n')
        .collect();
    if lines.len() != MaturityAdapterCase::ALL.len() {
        return Err(Refusal::RowCensus);
    }
    lines
        .into_iter()
        .zip(MaturityAdapterCase::ALL)
        .map(|(line, case)| admit_row(line, case))
        .collect()
}

fn admit_row(
    line: &str,
    case: MaturityAdapterCase,
) -> Result<MaturityAdapterComparison, MaturityAdapterRefusal> {
    use MaturityAdapterRefusal as Refusal;
    let fields: Vec<_> = line.split(' ').collect();
    let [
        "comparison",
        key_len,
        key,
        boundary,
        entry_len,
        entry,
        control,
        subject,
        refusal_len,
        refusal,
        control_len,
        control_digest,
        subject_len,
        subject_digest,
        name_len,
        name,
        version_len,
        version,
        protocol,
        source_digest,
        interpreter_len,
        interpreter,
    ] = fields.as_slice()
    else {
        return Err(Refusal::Grammar);
    };
    if text(key_len, key)? != format!("16.12 / {}", case.name())
        || *boundary != case.boundary()
        || text(entry_len, entry)? != case.entry()
    {
        return Err(Refusal::RowBinding);
    }
    if *control != "accepted" {
        return Err(Refusal::InvalidControl);
    }
    if *subject != "refused" {
        return Err(Refusal::AcceptedSubject);
    }
    if text(refusal_len, refusal)? != case.refusal() {
        return Err(Refusal::WrongRefusal);
    }
    if text(name_len, name)? != "elements-native-executor"
        || text(version_len, version)? != "2.1.0"
        || *protocol != "8"
    {
        return Err(Refusal::AdapterIdentity);
    }
    if *source_digest != ADAPTER_SOURCE_SHA256 {
        return Err(Refusal::SourceDigest);
    }
    let interpreter = text(interpreter_len, interpreter)?;
    if interpreter.is_empty() {
        return Err(Refusal::Grammar);
    }
    let (control, subject) = case.operands();
    let control = MaturityAdapterOperand::new(control, true);
    let subject = MaturityAdapterOperand::new(subject, case != MaturityAdapterCase::Oversized);
    if number(control_len)? != control.length
        || digest(control_digest)? != control.digest
        || number(subject_len)? != subject.length
        || digest(subject_digest)? != subject.digest
    {
        return Err(Refusal::OperandBinding);
    }
    Ok(MaturityAdapterComparison {
        case,
        language: "Python",
        entry: case.entry(),
        boundary: case.boundary(),
        phase: "request",
        maximum: MAXIMUM,
        allowed: if case == MaturityAdapterCase::UnknownField {
            vec!["schema", "case", "subject"]
        } else {
            Vec::new()
        },
        source_digest: ADAPTER_SOURCE_SHA256,
        interpreter,
        control,
        subject,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::maturity_observer::encode_hex;

    #[test]
    fn adapter_request_comparisons_bind_exact_pairs() {
        let pairs = admit(FIXTURE).expect("admitted comparisons");
        assert_eq!(pairs.len(), 5);
        for (pair, case) in pairs.iter().zip(MaturityAdapterCase::ALL) {
            assert_eq!(pair.validate(), Ok(case));
            assert_ne!(pair.control.digest, pair.subject.digest);
            assert_eq!(
                pair.subject.bytes.is_none(),
                case == MaturityAdapterCase::Oversized
            );
        }
    }

    #[test]
    fn adapter_source_matches_the_admitted_digest() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../scripts/elements-native-executor.py");
        let source = std::fs::read(path).expect("adapter source");
        assert_eq!(encode_hex(&sha256(&source)), ADAPTER_SOURCE_SHA256);
    }

    fn changed(field: usize, value: &str) -> Vec<u8> {
        let mut lines: Vec<_> = std::str::from_utf8(FIXTURE)
            .unwrap()
            .lines()
            .map(str::to_owned)
            .collect();
        let mut fields: Vec<_> = lines[0].split(' ').map(str::to_owned).collect();
        fields[field] = value.to_owned();
        lines[0] = fields.join(" ");
        (lines.join("\n") + "\n").into_bytes()
    }

    #[test]
    fn adapter_admission_refuses_each_changed_reading() {
        use MaturityAdapterRefusal as R;
        for (field, value, refusal) in [
            (0, "unexpected", R::Grammar),
            (1, "021", R::Grammar),
            (2, "ff", R::Grammar),
            (3, "RequestJson", R::RowBinding),
            (5, &"78".repeat(18), R::RowBinding),
            (6, "refused", R::InvalidControl),
            (7, "accepted", R::AcceptedSubject),
            (9, "787878787878787878787878", R::WrongRefusal),
            (10, "4", R::OperandBinding),
            (11, &"00".repeat(32), R::OperandBinding),
            (12, "4", R::OperandBinding),
            (13, &"00".repeat(32), R::OperandBinding),
            (15, &"78".repeat(24), R::AdapterIdentity),
            (17, &"78".repeat(5), R::AdapterIdentity),
            (18, "7", R::AdapterIdentity),
            (19, &"00".repeat(32), R::SourceDigest),
            (21, "", R::Grammar),
        ] {
            assert_eq!(admit(&changed(field, value)), Err(refusal));
        }
        let source = std::str::from_utf8(FIXTURE).unwrap();
        assert_eq!(
            admit(source.replacen("6 332e31322e33\n", "0 \n", 1).as_bytes()),
            Err(R::Grammar)
        );
        let lines: Vec<_> = source.lines().collect();
        assert_eq!(
            admit((lines[1..].join("\n") + "\n").as_bytes()),
            Err(R::RowCensus)
        );
        assert_eq!(
            admit(format!("{source}{}\n", lines[0]).as_bytes()),
            Err(R::RowCensus)
        );
        let mut duplicate = lines.clone();
        duplicate[1] = duplicate[0];
        assert_eq!(
            admit((duplicate.join("\n") + "\n").as_bytes()),
            Err(R::RowBinding)
        );
        assert_eq!(admit(source.trim_end().as_bytes()), Err(R::Grammar));
        assert_eq!(
            admit(source.replace('\n', "\r\n").as_bytes()),
            Err(R::Grammar)
        );
    }

    #[test]
    fn adapter_pairs_refuse_copied_reversed_and_misbound_operands() {
        let pair = MaturityAdapterComparison::admitted(MaturityAdapterCase::Blank).unwrap();
        let mut copied = pair.clone();
        copied.subject = copied.control.clone();
        assert_eq!(
            copied.validate(),
            Err(MaturityAdapterRefusal::OperandBinding)
        );
        let mut reversed = pair.clone();
        std::mem::swap(&mut reversed.control, &mut reversed.subject);
        assert_eq!(
            reversed.validate(),
            Err(MaturityAdapterRefusal::OperandBinding)
        );
        let mut misbound = pair;
        misbound.maximum -= 1;
        assert_eq!(misbound.validate(), Err(MaturityAdapterRefusal::RowBinding));
    }
}
