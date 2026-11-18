//! Canonical binary grammar for the fixed-form capture freeze.
//!
//! ASCII recipe domain followed by a zero byte;
//! u32-BE source count; each source is u32-BE path length, ASCII path, digest32;
//! u32-BE computed count and explicit u16-BE codes. Then K32, A32, v-u64-BE,
//! transcript42, pair32, framed leaf, version-u8, internal32, root32, tweak32,
//! output-key32, parity-u8, framed program and control, schema/target u32-BE,
//! envelope: optional maximum (0, or 1 plus u64-BE), five u64-BE counts,
//! four boolean bytes. Every byte string frame uses u32-BE length; booleans are
//! 0/1. Paths admit only [A-Za-z0-9/._-], without normalization or escaping.
//! Stored source/computed order is content and validation requires the inventory.

use super::{ComputedItem, Envelope, FreezeManifest, FreezeRefusal, RECIPE, SourceIdentity};
use crate::keyed_fixed_constructor::KeyedFixedDescriptor;
use realization::keyed_fixed::KeyedFixedParameters;

#[allow(clippy::too_many_lines)] // Keep exhaustive field coverage and its canonical order together.
pub(super) fn encode(manifest: &FreezeManifest) -> Result<Vec<u8>, FreezeRefusal> {
    let FreezeManifest {
        sources,
        computed,
        descriptor,
        recipe_schema,
        target_version,
        envelope,
    } = manifest;
    let mut out = RECIPE.as_bytes().to_vec();
    out.push(0);
    count(&mut out, sources.len())?;
    for SourceIdentity { path, digest } in sources {
        if !path.bytes().all(
            |b| matches!(b, b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'/' | b'.' | b'_' | b'-'),
        ) {
            return Err(FreezeRefusal::Alphabet);
        }
        frame(&mut out, path.as_bytes())?;
        out.extend(digest);
    }
    count(&mut out, computed.len())?;
    for item in computed {
        out.extend(code(*item).to_be_bytes());
    }
    let KeyedFixedDescriptor {
        parameters,
        transcript,
        pair_digest,
        leaf,
        leaf_version,
        internal_key,
        root,
        tweak,
        output_key,
        parity,
        output_program,
        control,
    } = descriptor;
    let KeyedFixedParameters { key, asset, value } = parameters;
    out.extend(key);
    out.extend(asset);
    out.extend(value.get().to_be_bytes());
    out.extend(transcript);
    out.extend(pair_digest);
    frame(&mut out, leaf)?;
    out.push(*leaf_version);
    out.extend(internal_key);
    out.extend(root);
    out.extend(tweak);
    out.extend(output_key);
    out.push(u8::from(*parity));
    frame(&mut out, output_program)?;
    frame(&mut out, control)?;
    out.extend(recipe_schema.to_be_bytes());
    out.extend(target_version.to_be_bytes());
    let Envelope {
        maximum_members,
        script_bytes,
        script_witness_bytes,
        input_witness_bytes,
        control_depth,
        argument_items,
        annex,
        issuance,
        peg_in,
        empty_nonce,
    } = envelope;
    match maximum_members {
        None => out.push(0),
        Some(value) => {
            out.push(1);
            out.extend(value.to_be_bytes());
        }
    }
    for value in [
        script_bytes,
        script_witness_bytes,
        input_witness_bytes,
        control_depth,
        argument_items,
    ] {
        out.extend(value.to_be_bytes());
    }
    out.extend([*annex, *issuance, *peg_in, *empty_nonce].map(u8::from));
    Ok(out)
}

fn count(out: &mut Vec<u8>, value: usize) -> Result<(), FreezeRefusal> {
    out.extend(
        u32::try_from(value)
            .map_err(|_| FreezeRefusal::Framing)?
            .to_be_bytes(),
    );
    Ok(())
}

fn frame(out: &mut Vec<u8>, value: &[u8]) -> Result<(), FreezeRefusal> {
    count(out, value.len())?;
    out.extend(value);
    Ok(())
}

const fn code(item: ComputedItem) -> u16 {
    match item {
        ComputedItem::Triple => 1,
        ComputedItem::Transcript => 2,
        ComputedItem::PairDigest => 3,
        ComputedItem::Leaf => 4,
        ComputedItem::Root => 5,
        ComputedItem::Tweak => 6,
        ComputedItem::Program => 7,
        ComputedItem::Control => 8,
        ComputedItem::EmissionAndClosure => 9,
        ComputedItem::MemberCoverage => 10,
        ComputedItem::DerivedResources => 11,
        ComputedItem::TransactionShape => 12,
        ComputedItem::CaptureSubjectCensus => 13,
        ComputedItem::ReaderRecurrence => 14,
        ComputedItem::MemberMaximum => 15,
    }
}

#[cfg(test)]
mod tests;
