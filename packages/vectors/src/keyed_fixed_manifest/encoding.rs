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

use super::{
    COMPUTED, ComputedItem, Envelope, FreezeManifest, FreezeRefusal, RECIPE, SourceIdentity,
};
use crate::keyed_fixed_constructor::KeyedFixedDescriptor;
use realization::keyed_fixed::{KeyedFixedParameters, KeyedFixedValue};

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

struct Decoder<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Decoder<'a> {
    fn take(&mut self, length: usize) -> Result<&'a [u8], FreezeRefusal> {
        let end = self
            .offset
            .checked_add(length)
            .ok_or(FreezeRefusal::Framing)?;
        let part = self
            .bytes
            .get(self.offset..end)
            .ok_or(FreezeRefusal::Framing)?;
        self.offset = end;
        Ok(part)
    }

    fn array<const N: usize>(&mut self) -> Result<[u8; N], FreezeRefusal> {
        self.take(N)?.try_into().map_err(|_| FreezeRefusal::Framing)
    }

    fn u8(&mut self) -> Result<u8, FreezeRefusal> {
        Ok(self.array::<1>()?[0])
    }

    fn u16(&mut self) -> Result<u16, FreezeRefusal> {
        Ok(u16::from_be_bytes(self.array()?))
    }

    fn u32(&mut self) -> Result<u32, FreezeRefusal> {
        Ok(u32::from_be_bytes(self.array()?))
    }

    fn u64(&mut self) -> Result<u64, FreezeRefusal> {
        Ok(u64::from_be_bytes(self.array()?))
    }

    fn frame(&mut self) -> Result<Vec<u8>, FreezeRefusal> {
        let length = usize::try_from(self.u32()?).map_err(|_| FreezeRefusal::Framing)?;
        if length > 32_000_000 {
            return Err(FreezeRefusal::Framing);
        }
        Ok(self.take(length)?.to_vec())
    }

    fn boolean(&mut self) -> Result<bool, FreezeRefusal> {
        match self.u8()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(FreezeRefusal::Framing),
        }
    }
}

/// Decode the exact campaign grammar without trusting its source inventory.
/// Historical authentication is performed by the manifest owner after this parse.
pub(super) fn decode(bytes: &[u8], source_count: usize) -> Result<FreezeManifest, FreezeRefusal> {
    let mut reader = Decoder { bytes, offset: 0 };
    if reader.take(RECIPE.len())? != RECIPE.as_bytes() || reader.u8()? != 0 {
        return Err(FreezeRefusal::Recipe);
    }
    if source_count > 256
        || reader.u32()? != u32::try_from(source_count).map_err(|_| FreezeRefusal::Inventory)?
    {
        return Err(FreezeRefusal::Inventory);
    }
    let mut sources = Vec::with_capacity(source_count);
    for _ in 0..source_count {
        let path = reader.frame()?;
        if path.is_empty()
            || !path.iter().all(|byte| {
                matches!(byte, b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'/' | b'.' | b'_' | b'-')
            })
        {
            return Err(FreezeRefusal::Alphabet);
        }
        let path = String::from_utf8(path).map_err(|_| FreezeRefusal::Alphabet)?;
        if sources
            .last()
            .is_some_and(|last: &SourceIdentity| last.path.as_str() >= path.as_str())
        {
            return Err(FreezeRefusal::Inventory);
        }
        sources.push(SourceIdentity {
            path,
            digest: reader.array()?,
        });
    }
    if reader.u32()? != u32::try_from(COMPUTED.len()).map_err(|_| FreezeRefusal::Inventory)? {
        return Err(FreezeRefusal::Inventory);
    }
    for item in COMPUTED {
        if reader.u16()? != code(item) {
            return Err(FreezeRefusal::Inventory);
        }
    }
    let key = reader.array()?;
    let asset = reader.array()?;
    let value = KeyedFixedValue::new(reader.u64()?).map_err(|_| FreezeRefusal::Triple)?;
    let parameters = KeyedFixedParameters { key, asset, value };
    let descriptor = KeyedFixedDescriptor {
        parameters,
        transcript: reader.array()?,
        pair_digest: reader.array()?,
        leaf: reader.frame()?,
        leaf_version: reader.u8()?,
        internal_key: reader.array()?,
        root: reader.array()?,
        tweak: reader.array()?,
        output_key: reader.array()?,
        parity: reader.boolean()?,
        output_program: reader.frame()?,
        control: reader.frame()?,
    };
    let recipe_schema = reader.u32()?;
    let target_version = reader.u32()?;
    let maximum_members = match reader.u8()? {
        0 => None,
        1 => Some(reader.u64()?),
        _ => return Err(FreezeRefusal::Framing),
    };
    let envelope = Envelope {
        maximum_members,
        script_bytes: reader.u64()?,
        script_witness_bytes: reader.u64()?,
        input_witness_bytes: reader.u64()?,
        control_depth: reader.u64()?,
        argument_items: reader.u64()?,
        annex: reader.boolean()?,
        issuance: reader.boolean()?,
        peg_in: reader.boolean()?,
        empty_nonce: reader.boolean()?,
    };
    if reader.offset != bytes.len() {
        return Err(FreezeRefusal::Framing);
    }
    Ok(FreezeManifest {
        sources,
        computed: COMPUTED.to_vec(),
        descriptor,
        recipe_schema,
        target_version,
        envelope,
    })
}

#[cfg(test)]
mod tests;
