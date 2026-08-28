//! Historical phase-B key-path probe observations.

use target_elements_conformance::protocol::ObservedOutcomeLayer;

/// The layer the adapter filed the verdict under, corrected.
///
/// Recorded as the string the run produced, on the pattern phase A
/// set. The whole content of phase B is that this differs from
/// [`crate::live_history_v1::keypath_probe::OBSERVED_LAYER`] while the
/// words below do not: the target said the same thing and the wire stopped
/// mis-naming it.
pub const OBSERVED_LAYER: &str = "KeyPathRejection";

/// What the target said when it refused the attempt, verbatim.
pub const REFUSAL_DETAIL: &str = "mandatory-script-verify-flag-failed (Invalid Schnorr signature)";

/// The identity the target gave the accepted control.
///
/// The half of the pair a reader can check against a chain. The
/// refusal left no transaction to look up, which is what being
/// refused means.
pub const CONTROL_ACCEPTED_TXID: &str =
    "0fcf267058e83e06e87a950bbeec920a15e3641ef7f76c55df0a8fff544e64c1";

/// How many bytes the control handed to the submission wire.
pub const CONTROL_SUBMITTED_BYTES: usize = 751;

/// How many items the control's single input carried.
///
/// Three — signature, leaf script, control block — against the
/// attempt's one, which is the whole visible difference between the
/// pair.
pub const CONTROL_WITNESS_ITEMS: usize = 3;

/// Whether the pair differed in the witness ALONE, as measured.
pub const CONTROL_SHARES_THE_ATTEMPTS_WITNESSLESS_BYTES: bool = true;

/// The run's wall time.
pub const WALL_SECONDS: f64 = 5.1;

/// The layer the attempt was refused at, TYPED.
///
/// [`OBSERVED_LAYER`] above records the same fact as the STRING the
/// run produced, and phase B's whole content is that this layer is
/// its own rather than the script path phase A borrowed. The string
/// stays exactly as recorded — it is the run's own bytes — and this
/// constant carries the fact in the vocabulary a classifier can
/// compare. The two are bound to each other in this module's tests,
/// so the typed form cannot drift from the recorded one.
pub const REFUSAL_OBSERVED_LAYER: ObservedOutcomeLayer = ObservedOutcomeLayer::KeyPathRejection;
