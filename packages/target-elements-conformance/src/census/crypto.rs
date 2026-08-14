//! The elliptic-curve and signature primitives.
//!
//! # Published vectors only
//!
//! The accepted relations are published scalar-multiplication and
//! key-tweak vectors and the published signature vector; the refused ones
//! are the published invalid keys, scalars, and tweaks, or a published
//! vector combined with the wrong published operand. Nothing is signed
//! here and no secret scalar appears (Guide-9 §13.9, §13.10, §17.2).
//!
//! # The verifying forms end in a literal
//!
//! A verify-form primitive consumes its operands and pushes nothing, so
//! a script that stops there leaves an empty stack and is invalid
//! whatever the relation did. Each accepted case therefore pushes a
//! single true afterwards, which is the whole point of a verify form: it
//! either continues or it does not.
//!
//! # What no static fixture can carry
//!
//! A signature over the transaction sighash depends on the transaction
//! the executor materializes, so the accepting cases of the two
//! sighash-checking primitives cannot exist as fixtures. What *can* be
//! stated is their failure behaviour, including the reviewed distinction
//! between the checking form — which consumes its operands and pushes a
//! false for an empty signature — and the verifying form, which aborts.

use tapscript::StackItem;
use target_elements::{EncodingClass, OpcodeId};

use crate::census::author::{Case, CensusAuthor, falsity, op, push, truth};
use crate::census::context::census_transaction;
use crate::census::material::{
    GENERATOR, GENERATOR_TIMES_SCALAR, GENERATOR_TIMES_TEN, KEY_UNKNOWN_PREFIX, POINT_OFF_CURVE,
    SCALAR_ABOVE_ORDER, SCALAR_ONE, SCALAR_TEN, SCALAR_VECTOR, SIGNATURE_VECTOR_KEY,
    SIGNATURE_VECTOR_MESSAGE, SIGNATURE_VECTOR_SIGNATURE, TWEAK_EVEN_INTERNAL, TWEAK_EVEN_RESULT,
    TWEAK_EVEN_TWEAK, TWEAK_ODD_INTERNAL, TWEAK_ODD_RESULT, TWEAK_ODD_TWEAK, mutated_signature,
};
use crate::fixture::NativeCaseGroup;
use crate::protocol::ObservedFailureClass;

/// Every curve and signature case.
pub fn cases(author: &mut CensusAuthor<'_>) {
    scalar_multiplication(author);
    key_tweaks(author);
    stack_message_signatures(author);
    transaction_signatures(author);
}

/// One case with no transaction context.
const fn bare<'a>(
    group: NativeCaseGroup,
    id: OpcodeId,
    script: &'a [tapscript::TapscriptInstruction],
    stack: &'a [StackItem],
) -> Case<'a> {
    Case {
        group,
        opcode: Some(id),
        script,
        stack,
        context: None,
    }
}

/// The literal that follows a verify form, so the script leaves one
/// true item.
fn continuation(author: &mut CensusAuthor<'_>) -> StackItem {
    author.item(vec![0x01])
}

/// The published scalar-multiplication relations, and the values that
/// break them.
fn scalar_multiplication(author: &mut CensusAuthor<'_>) {
    let id = OpcodeId::EcMulScalarVerify;
    let group = NativeCaseGroup::EllipticCurve;

    for (result, point, scalar) in [
        // The generator multiplied by one is the generator: an
        // even-parity result.
        (GENERATOR.to_vec(), GENERATOR.to_vec(), SCALAR_ONE),
        // Two published vectors, both odd-parity results.
        (GENERATOR_TIMES_TEN.to_vec(), GENERATOR.to_vec(), SCALAR_TEN),
        (
            GENERATOR_TIMES_SCALAR.to_vec(),
            GENERATOR.to_vec(),
            SCALAR_VECTOR,
        ),
    ] {
        let stack = [
            author.item(result),
            author.item(point),
            author.encoded(EncodingClass::EcScalar, scalar.to_vec()),
        ];
        let follow = continuation(author);
        let script = [op(id), push(follow)];
        author.accept(bare(group, id, &script, &stack), vec![truth()]);
    }

    let follow = continuation(author);
    let script = [op(id), push(follow)];

    // A relation that does not hold: the right point, the wrong scalar.
    for (result, point, scalar) in [
        (
            GENERATOR_TIMES_TEN.to_vec(),
            GENERATOR.to_vec(),
            SCALAR_ONE.to_vec(),
        ),
        (GENERATOR.to_vec(), GENERATOR.to_vec(), SCALAR_TEN.to_vec()),
        // A scalar above the group order, and one of the wrong width.
        (
            GENERATOR_TIMES_SCALAR.to_vec(),
            GENERATOR.to_vec(),
            SCALAR_ABOVE_ORDER.to_vec(),
        ),
        (
            GENERATOR_TIMES_SCALAR.to_vec(),
            GENERATOR.to_vec(),
            SCALAR_VECTOR[..31].to_vec(),
        ),
        // A point that is not on the curve, in either position.
        (
            POINT_OFF_CURVE.to_vec(),
            GENERATOR.to_vec(),
            SCALAR_ONE.to_vec(),
        ),
        (
            GENERATOR.to_vec(),
            POINT_OFF_CURVE.to_vec(),
            SCALAR_ONE.to_vec(),
        ),
    ] {
        let stack = [author.item(result), author.item(point), author.item(scalar)];
        author.reject(
            bare(group, id, &script, &stack),
            &[ObservedFailureClass::InvalidCurveRelation],
        );
    }

    // Keys the target refuses to read at all.
    for (result, point) in [
        (KEY_UNKNOWN_PREFIX.to_vec(), GENERATOR.to_vec()),
        (GENERATOR.to_vec(), KEY_UNKNOWN_PREFIX.to_vec()),
        // An x-only key is not a compressed one, and a thirty-four byte
        // value is neither.
        (GENERATOR[1..].to_vec(), GENERATOR.to_vec()),
        ([GENERATOR.as_slice(), &[0x00]].concat(), GENERATOR.to_vec()),
    ] {
        let stack = [
            author.item(result),
            author.item(point),
            author.encoded(EncodingClass::EcScalar, SCALAR_ONE.to_vec()),
        ];
        author.reject(
            bare(group, id, &script, &stack),
            &[ObservedFailureClass::InvalidPublicKeyEncoding],
        );
    }

    underflows(author, group, id, 3);
}

/// The published key-tweak relations, and the values that break them.
fn key_tweaks(author: &mut CensusAuthor<'_>) {
    let id = OpcodeId::TweakVerify;
    let group = NativeCaseGroup::EllipticCurve;

    // Both admitted parities: the published vectors state the tweaked
    // key x-only and its control block states which of the two points it
    // is, which is the parity byte these results carry.
    for (result, tweak, internal) in [
        (TWEAK_EVEN_RESULT, TWEAK_EVEN_TWEAK, TWEAK_EVEN_INTERNAL),
        (TWEAK_ODD_RESULT, TWEAK_ODD_TWEAK, TWEAK_ODD_INTERNAL),
    ] {
        let stack = [
            author.item(result.to_vec()),
            author.encoded(EncodingClass::TaprootTweak, tweak.to_vec()),
            author.encoded(EncodingClass::XOnlyPublicKey, internal.to_vec()),
        ];
        let follow = continuation(author);
        let script = [op(id), push(follow)];
        author.accept(bare(group, id, &script, &stack), vec![truth()]);
    }

    let follow = continuation(author);
    let script = [op(id), push(follow)];

    // Each vector's own operands, crossed with the other's.
    for (result, tweak, internal) in [
        (TWEAK_EVEN_RESULT, TWEAK_ODD_TWEAK, TWEAK_EVEN_INTERNAL),
        (TWEAK_EVEN_RESULT, TWEAK_EVEN_TWEAK, TWEAK_ODD_INTERNAL),
        (TWEAK_ODD_RESULT, TWEAK_EVEN_TWEAK, TWEAK_ODD_INTERNAL),
        (TWEAK_EVEN_RESULT, SCALAR_ABOVE_ORDER, TWEAK_EVEN_INTERNAL),
    ] {
        let stack = [
            author.item(result.to_vec()),
            author.item(tweak.to_vec()),
            author.item(internal.to_vec()),
        ];
        author.reject(
            bare(group, id, &script, &stack),
            &[ObservedFailureClass::InvalidCurveRelation],
        );
    }

    // Operands of the wrong shape: an x-only value where a compressed
    // key belongs, a compressed key where an x-only one belongs, and a
    // tweak of the wrong width.
    for (result, tweak, internal) in [
        (
            TWEAK_EVEN_RESULT[1..].to_vec(),
            TWEAK_EVEN_TWEAK.to_vec(),
            TWEAK_EVEN_INTERNAL.to_vec(),
        ),
        (
            TWEAK_EVEN_RESULT.to_vec(),
            TWEAK_EVEN_TWEAK.to_vec(),
            GENERATOR.to_vec(),
        ),
        (
            TWEAK_EVEN_RESULT.to_vec(),
            TWEAK_EVEN_TWEAK[..31].to_vec(),
            TWEAK_EVEN_INTERNAL.to_vec(),
        ),
    ] {
        let stack = [
            author.item(result),
            author.item(tweak),
            author.item(internal),
        ];
        author.reject(
            bare(group, id, &script, &stack),
            &[ObservedFailureClass::InvalidPublicKeyEncoding],
        );
    }

    underflows(author, group, id, 3);
}

/// The signature primitives that take their message from the stack.
fn stack_message_signatures(author: &mut CensusAuthor<'_>) {
    let group = NativeCaseGroup::Signature;

    for id in [
        OpcodeId::CheckSigFromStack,
        OpcodeId::CheckSigFromStackVerify,
    ] {
        let verifying = id == OpcodeId::CheckSigFromStackVerify;

        // The published vector, which holds.
        let stack = [
            author.encoded(
                EncodingClass::SchnorrSignature,
                SIGNATURE_VECTOR_SIGNATURE.to_vec(),
            ),
            author.item(SIGNATURE_VECTOR_MESSAGE.to_vec()),
            author.encoded(EncodingClass::XOnlyPublicKey, SIGNATURE_VECTOR_KEY.to_vec()),
        ];
        let script = if verifying {
            let follow = continuation(author);
            vec![op(id), push(follow)]
        } else {
            vec![op(id)]
        };
        author.accept(bare(group, id, &script, &stack), vec![truth()]);

        // An empty signature: the checking form consumes its operands
        // and pushes a false, the verifying form aborts.
        let stack = [
            StackItem::empty(),
            author.item(SIGNATURE_VECTOR_MESSAGE.to_vec()),
            author.encoded(EncodingClass::XOnlyPublicKey, SIGNATURE_VECTOR_KEY.to_vec()),
        ];
        if verifying {
            author.reject(
                bare(group, id, &script, &stack),
                &[
                    ObservedFailureClass::EmptySignature,
                    ObservedFailureClass::InvalidSignature,
                ],
            );
        } else {
            author.evaluated_false(bare(group, id, &script, &stack), vec![falsity()]);
        }

        // A non-empty signature that does not verify, and one that is
        // not a signature's width: both abort, in either form.
        for signature in [mutated_signature(), vec![0x01, 0x02]] {
            let stack = [
                author.item(signature),
                author.item(SIGNATURE_VECTOR_MESSAGE.to_vec()),
                author.encoded(EncodingClass::XOnlyPublicKey, SIGNATURE_VECTOR_KEY.to_vec()),
            ];
            author.reject(
                bare(group, id, &script, &stack),
                &[ObservedFailureClass::InvalidSignature],
            );
        }

        // An absent public key, which the target refuses before it looks
        // at the signature at all.
        let stack = [
            author.encoded(
                EncodingClass::SchnorrSignature,
                SIGNATURE_VECTOR_SIGNATURE.to_vec(),
            ),
            author.item(SIGNATURE_VECTOR_MESSAGE.to_vec()),
            StackItem::empty(),
        ];
        author.reject(
            bare(group, id, &script, &stack),
            &[ObservedFailureClass::InvalidPublicKeyEncoding],
        );

        underflows(author, group, id, 3);
    }
}

/// The signature primitives that take their message from the
/// transaction.
fn transaction_signatures(author: &mut CensusAuthor<'_>) {
    let group = NativeCaseGroup::Signature;

    for id in [OpcodeId::CheckSig, OpcodeId::CheckSigVerify] {
        let verifying = id == OpcodeId::CheckSigVerify;
        let script = if verifying {
            let follow = continuation(author);
            vec![op(id), push(follow)]
        } else {
            vec![op(id)]
        };

        let key = author.encoded(EncodingClass::XOnlyPublicKey, SIGNATURE_VECTOR_KEY.to_vec());

        // An empty signature. This is the reviewed distinction, and it
        // is the only path either primitive can take without a signature
        // over a transaction that does not exist yet.
        let stack = [StackItem::empty(), key.clone()];
        let case = Case {
            group,
            opcode: Some(id),
            script: &script,
            stack: &stack,
            context: Some(census_transaction()),
        };
        if verifying {
            author.reject(
                case,
                &[
                    ObservedFailureClass::EmptySignature,
                    ObservedFailureClass::InvalidSignature,
                ],
            );
        } else {
            author.evaluated_false(case, vec![falsity()]);
        }

        // A signature over something else. The published vector signs a
        // published message, which is not this transaction's sighash.
        for signature in [SIGNATURE_VECTOR_SIGNATURE.to_vec(), vec![0x01, 0x02]] {
            let stack = [author.item(signature), key.clone()];
            let case = Case {
                group,
                opcode: Some(id),
                script: &script,
                stack: &stack,
                context: Some(census_transaction()),
            };
            author.reject(case, &[ObservedFailureClass::InvalidSignature]);
        }

        // An absent public key.
        let stack = [
            author.encoded(
                EncodingClass::SchnorrSignature,
                SIGNATURE_VECTOR_SIGNATURE.to_vec(),
            ),
            StackItem::empty(),
        ];
        let case = Case {
            group,
            opcode: Some(id),
            script: &script,
            stack: &stack,
            context: Some(census_transaction()),
        };
        author.reject(case, &[ObservedFailureClass::InvalidPublicKeyEncoding]);

        underflows(author, group, id, 2);
    }
}

/// One case per stack depth below what the primitive consumes.
fn underflows(
    author: &mut CensusAuthor<'_>,
    group: NativeCaseGroup,
    id: OpcodeId,
    operands: usize,
) {
    let script = [op(id)];
    let filler = author.item(vec![0x01]);
    for depth in 0..operands {
        let stack = vec![filler.clone(); depth];
        author.reject(
            bare(group, id, &script, &stack),
            &[ObservedFailureClass::StackUnderflow],
        );
    }
}
