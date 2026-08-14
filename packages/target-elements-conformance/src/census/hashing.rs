//! The streaming hash primitives.
//!
//! # Chains, because a context has no stated bytes
//!
//! The initializing and updating primitives push a serialized hash
//! state, and this package deliberately states no expected bytes for
//! one: the serialization is the target's own, and writing down what it
//! ought to look like would be this repository inventing a target fact
//! it has not reviewed. Every positive case therefore runs a complete
//! chain and states the digest, which *is* published: the context is
//! observed by being carried through the chain rather than by being
//! spelled out (Guide-9 §13.2).
//!
//! # The digests are published vectors
//!
//! The empty, three-byte, and fifty-six byte digests are the published
//! SHA-256 vectors. The two long-chunk digests were computed
//! independently of this repository's code from the published definition
//! of the function. None of them is produced by anything under test.

use tapscript::{StackItem, TapscriptInstruction};
use target_elements::OpcodeId;

use crate::census::author::{Case, CensusAuthor, op, push};
use crate::census::material::{
    MESSAGE_ABC, MESSAGE_TWO_BLOCK, SHA256_ABC, SHA256_EMPTY, SHA256_TWO_BLOCK,
};
use crate::fixture::NativeCaseGroup;
use crate::protocol::ObservedFailureClass;

/// The largest literal the target admits, which is the widest chunk a
/// script can offer a streaming hash in one push.
const LARGEST_CHUNK: usize = 520;

/// The byte the largest-chunk vector is filled with.
const LARGEST_CHUNK_BYTE: u8 = 0x5a;

/// The digest of five hundred and twenty bytes of that byte.
const SHA256_LARGEST_CHUNK: [u8; 32] = [
    0x62, 0x4b, 0xe9, 0x84, 0xa6, 0xf1, 0x67, 0x98, 0xf4, 0xa2, 0xee, 0x7f, 0xae, 0x06, 0x99, 0x4b,
    0x30, 0x1f, 0x12, 0x86, 0xad, 0xf9, 0xc3, 0xcc, 0x7d, 0xa8, 0x3c, 0x79, 0x7d, 0x06, 0xaa, 0x46,
];

/// The digest of that chunk followed by the three-byte vector.
const SHA256_LARGEST_CHUNK_THEN_ABC: [u8; 32] = [
    0xaa, 0x14, 0xe8, 0xb5, 0xc1, 0x90, 0x45, 0x65, 0xaa, 0xfb, 0x47, 0x13, 0x9f, 0xe4, 0x49, 0x66,
    0x3b, 0x7d, 0x2e, 0x4a, 0xe8, 0x7d, 0x11, 0xea, 0x2d, 0x15, 0xe3, 0xd7, 0x23, 0x11, 0x1c, 0xa5,
];

/// The narrowest byte string the target refuses as a serialized state.
const TOO_NARROW_CONTEXT: usize = 4;

/// A byte string wider than any serialized state the target admits.
const TOO_WIDE_CONTEXT: usize = 104;

/// Every streaming-hash case.
pub fn cases(author: &mut CensusAuthor<'_>) {
    chains(author);
    malformed_contexts(author);
    underflows(author);
}

/// One complete hashing chain, attributed to one of its primitives.
///
/// `chunks` is the message split as the script offers it: the first
/// chunk initializes, the last finalizes, and every chunk between them
/// updates.
fn chain(author: &mut CensusAuthor<'_>, id: OpcodeId, chunks: &[&[u8]], digest: [u8; 32]) {
    let Some((first, rest)) = chunks.split_first() else {
        return;
    };
    let Some((last, middle)) = rest.split_last() else {
        return;
    };

    let mut script = vec![
        push(author.item(first.to_vec())),
        op(OpcodeId::Sha256Initialize),
    ];
    for chunk in middle {
        script.push(push(author.item((*chunk).to_vec())));
        script.push(op(OpcodeId::Sha256Update));
    }
    script.push(push(author.item(last.to_vec())));
    script.push(op(OpcodeId::Sha256Finalize));

    author.accept(chain_case(id, &script, &[]), vec![digest.to_vec()]);
}

/// One streaming-hash case, which never needs a transaction.
const fn chain_case<'a>(
    id: OpcodeId,
    script: &'a [TapscriptInstruction],
    stack: &'a [StackItem],
) -> Case<'a> {
    Case {
        group: NativeCaseGroup::StreamingHash,
        opcode: Some(id),
        script,
        stack,
        context: None,
    }
}

/// The complete chains, one per published vector.
fn chains(author: &mut CensusAuthor<'_>) {
    let largest = vec![LARGEST_CHUNK_BYTE; LARGEST_CHUNK];
    let (head, tail) = MESSAGE_TWO_BLOCK.split_at(32);
    let (middle, last) = tail.split_at(20);

    for (id, chunks, digest) in [
        // The empty message: nothing initializes it and nothing
        // finalizes it, and the digest is still the published one.
        (
            OpcodeId::Sha256Finalize,
            vec![&b""[..], &b""[..]],
            SHA256_EMPTY,
        ),
        // One chunk, initialized and finalized.
        (
            OpcodeId::Sha256Initialize,
            vec![MESSAGE_ABC, &b""[..]],
            SHA256_ABC,
        ),
        // The same digest from a message the finalizing primitive
        // absorbs rather than the initializing one.
        (
            OpcodeId::Sha256Finalize,
            vec![&b""[..], MESSAGE_ABC],
            SHA256_ABC,
        ),
        // Three chunks, so the updating primitive carries a state it
        // did not create and hands on one it did.
        (
            OpcodeId::Sha256Update,
            vec![&b"a"[..], &b"b"[..], &b"c"[..]],
            SHA256_ABC,
        ),
        // A message longer than one compression block, in one chunk and
        // then in three.
        (
            OpcodeId::Sha256Finalize,
            vec![MESSAGE_TWO_BLOCK, &b""[..]],
            SHA256_TWO_BLOCK,
        ),
        (
            OpcodeId::Sha256Update,
            vec![head, middle, last],
            SHA256_TWO_BLOCK,
        ),
        // The widest chunk a literal can carry, alone and then with a
        // second chunk after it.
        (
            OpcodeId::Sha256Initialize,
            vec![largest.as_slice(), &b""[..]],
            SHA256_LARGEST_CHUNK,
        ),
        (
            OpcodeId::Sha256Update,
            vec![largest.as_slice(), MESSAGE_ABC, &b""[..]],
            SHA256_LARGEST_CHUNK_THEN_ABC,
        ),
    ] {
        chain(author, id, &chunks, digest);
    }
}

/// The states the target refuses to load.
fn malformed_contexts(author: &mut CensusAuthor<'_>) {
    let narrow = author.item(vec![0x00; TOO_NARROW_CONTEXT]);
    let wide = author.item(vec![0x00; TOO_WIDE_CONTEXT]);
    let empty = StackItem::empty();

    for (id, instruction) in [
        (OpcodeId::Sha256Update, op(OpcodeId::Sha256Update)),
        (OpcodeId::Sha256Finalize, op(OpcodeId::Sha256Finalize)),
    ] {
        for context in [narrow.clone(), wide.clone()] {
            let script = [instruction.clone()];
            let stack = [context, empty.clone()];
            author.reject(
                chain_case(id, &script, &stack),
                &[ObservedFailureClass::HashContextLoad],
            );
        }
    }

    // A finalized digest is thirty-two bytes, which is not a serialized
    // state: continuing a chain after finalizing it is the ordering
    // mistake this case states.
    let script = [
        push(author.item(MESSAGE_ABC.to_vec())),
        op(OpcodeId::Sha256Initialize),
        push(StackItem::empty()),
        op(OpcodeId::Sha256Finalize),
        push(StackItem::empty()),
        op(OpcodeId::Sha256Update),
    ];
    author.reject(
        chain_case(OpcodeId::Sha256Update, &script, &[]),
        &[ObservedFailureClass::HashContextLoad],
    );
}

/// The cases with fewer operands than a primitive consumes.
fn underflows(author: &mut CensusAuthor<'_>) {
    let empty = StackItem::empty();
    let stacks: [(OpcodeId, Vec<StackItem>); 5] = [
        (OpcodeId::Sha256Initialize, Vec::new()),
        (OpcodeId::Sha256Update, Vec::new()),
        (OpcodeId::Sha256Update, vec![empty.clone()]),
        (OpcodeId::Sha256Finalize, Vec::new()),
        (OpcodeId::Sha256Finalize, vec![empty]),
    ];
    for (id, stack) in stacks {
        let script: [TapscriptInstruction; 1] = [op(id)];
        author.reject(
            chain_case(id, &script, &stack),
            &[ObservedFailureClass::StackUnderflow],
        );
    }
}
