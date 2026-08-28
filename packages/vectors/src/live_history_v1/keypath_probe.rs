//! Historical phase-A key-path probe observations.

/// The disposable asset the run issued.
pub const ISSUED_ASSET: &str = "d74fc8d4d85f8251aa653f5404ea646f56d34b8f506a98279ce2926d05ca93fb";

/// The witness program the funded coin paid to.
pub const FUNDED_PROGRAM: &str =
    "51208d696527d4d1517c67ba5e72b9c5b9c730cb463f20d59b84d7e813d9a188ef69";

/// The tweaked output key that program carries.
pub const OUTPUT_KEY: &str = "8d696527d4d1517c67ba5e72b9c5b9c730cb463f20d59b84d7e813d9a188ef69";

/// The taptree root the output key is tweaked by.
pub const MERKLE_ROOT: &str = "70c084e34b9e1f63b6806d999d0a2df507493b67ef1a3d8711f062d678cf6e45";

/// The x-only public key of the scalar that signed the attempt.
///
/// The first published owner's. Not [`OUTPUT_KEY`], which is the
/// whole reason the refusal below establishes nothing about the
/// internal key.
pub const SIGNING_PUBLIC_KEY: &str =
    "dff1d77f2a671c5f36183726db2341be58feae1da2deced843240f7b502ba659";

/// The candidate key-path message the signature was taken over.
///
/// Candidate-scoped. Reviewed by nobody, and observed to be the
/// target's message by nothing.
pub const CANDIDATE_KEY_PATH_MESSAGE: &str =
    "d165412ea39cc0ee067af1313c04bacdaff2f9add0db6ae895da207ab0943c05";

/// How many bytes were handed to the submission wire.
pub const SUBMITTED_BYTES: usize = 281;

/// How many items the single input's witness carried.
pub const WITNESS_ITEMS: usize = 1;

/// How wide that one item was.
pub const WITNESS_ITEM_BYTES: usize = 64;

/// What the target said, verbatim and unmapped.
pub const OBSERVED_DETAIL: &str = "mandatory-script-verify-flag-failed (Invalid Schnorr signature)";

/// The layer the adapter filed the verdict under.
///
/// Recorded as the string the run produced rather than as the enum,
/// because the name is itself the finding: the observed-layer
/// vocabulary has no key-path member, so a key-path refusal is filed
/// under a script-path name. The adapter classifies on the refusal
/// text's prefix and cannot do otherwise with the vocabulary it has.
/// Repairing that is the typed carrier the follow-up phase owns; the
/// probe reports it and changes nothing.
///
/// The repair LANDED, and this constant does not move for it. What
/// phase A observed is what phase A observed, and rewriting it to the
/// name a later vocabulary would have given it would replace a record
/// of a run with a reconstruction of one. The corrected observation
/// is phase B's own, beside this module in
/// [`crate::live_history_v1::keypath_probe_phase_b`].
pub const OBSERVED_LAYER: &str = "ScriptPathRejection";

/// The run's wall time.
pub const WALL_SECONDS: f64 = 4.1;
