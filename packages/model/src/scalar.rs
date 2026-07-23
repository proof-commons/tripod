//! Identifiers, exact scalar types, and checked arithmetic.
//!
//! Implements `(´def:verification:identifiers´)`, `(´def:domains:sat´)`,
//! `(´def:domains:ratio´)`, `(´rule:domains:checked-arithmetic´)`, and
//! `(´rule:domains:active-backing-cap´)`.

use std::collections::BTreeMap;

use crate::guard::Guard;

// ´def:verification:identifiers´

pub type OutPoint = u64;
pub type Cycle = u64;
pub type BlockHeight = u64;
pub type TxIndex = u32;
pub type SchemaVersion = u32;

pub const TWO_51: u64 = 1_u64 << 51;

/// Maximum simultaneously active L-BTC backing plus admitted deposit
/// escrow:
///
/// ```text
/// Ω + Q <= ACTIVE_BACKING_MAX
/// ```
///
/// This is not a cap on cumulative lifetime deposit volume. Redemptions
/// may create headroom for later deposits. `ACTIVE_BACKING_MAX` is the
/// economic/deployment cap; `TWO_51` remains the arithmetic-safety cap.
pub const ACTIVE_BACKING_MAX: u64 = 2_100_000_000_000_000;

const _: () = assert!(ACTIVE_BACKING_MAX < TWO_51);

// ´rule:domains:active-backing-cap´

pub fn require_active_backing_cap(backing: Sat) -> Result<(), Guard> {
    if backing.get() <= ACTIVE_BACKING_MAX {
        Ok(())
    } else {
        Err(Guard::ActiveBackingCapExceeded)
    }
}

/// Centralizes both the checked addition and the active-backing-cap
/// enforcement for the active backing Ω + Q.
pub fn checked_active_backing(omega: Sat, q: Sat) -> Result<Sat, Guard> {
    let backing = omega.checked_add(q)?;

    require_active_backing_cap(backing)?;

    Ok(backing)
}

/// Public abstract owner identity used by the executable model.
///
/// This value is not private key material. Callers must derive it from
/// a public identity and must never place private key bytes, seed
/// material, signing nonces, blinding factors, private openings, or
/// credentials in it (ADR-015).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OwnerKey(pub [u8; 32]);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AttestationAddress(pub [u8; 32]);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TxId(pub [u8; 32]);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BlockHash(pub [u8; 32]);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CanonicalOrder {
    pub height: BlockHeight,
    pub tx_index: TxIndex,
}

// ´def:domains:sat´

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Sat(u64);

impl Sat {
    pub const ZERO: Sat = Sat(0);
    pub const ONE: Sat = Sat(1);

    pub fn new(value: u64) -> Result<Self, Guard> {
        if value < TWO_51 {
            Ok(Self(value))
        } else {
            Err(Guard::Domain)
        }
    }

    pub fn positive(value: u64) -> Result<Self, Guard> {
        if value > 0 && value < TWO_51 {
            Ok(Self(value))
        } else {
            Err(Guard::Domain)
        }
    }

    pub fn get(self) -> u64 {
        self.0
    }

    pub fn is_zero(self) -> bool {
        self.0 == 0
    }

    pub fn checked_add(self, rhs: Sat) -> Result<Sat, Guard> {
        let value = self.0.checked_add(rhs.0).ok_or(Guard::Overflow)?;

        Sat::new(value)
    }

    pub fn checked_sub(self, rhs: Sat) -> Result<Sat, Guard> {
        let value = self.0.checked_sub(rhs.0).ok_or(Guard::Underflow)?;

        Sat::new(value)
    }

    pub fn checked_sum<I>(values: I) -> Result<Sat, Guard>
    where
        I: IntoIterator<Item = Sat>,
    {
        values
            .into_iter()
            .try_fold(Sat::ZERO, |acc, value| acc.checked_add(value))
    }

    pub fn min(self, rhs: Sat) -> Sat {
        Sat(self.0.min(rhs.0))
    }
}

// ´def:domains:ratio´
//
// Strict 0 < num < den transcribes the Attestation specification's open-interval
// f, ζ ∈ (0,1) exactly ([A-def:model:fee-and-split]): a zero rate and
// a unit rate are unrepresentable by construction, not by convention.

/// A rate in the open interval (0, 1), valid by construction.
///
/// Fields are private, so [`Ratio::new`] is the only constructor and
/// no code path — inside or outside this crate — can reach
/// [`floor_ratio`] with a zero denominator or an out-of-domain rate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ratio {
    numerator: u16,
    denominator: u16,
}

impl Ratio {
    pub fn new(numerator: u16, denominator: u16) -> Result<Self, Guard> {
        if numerator == 0 || numerator >= denominator || denominator >= 1024 {
            return Err(Guard::BadConstant);
        }

        Ok(Self {
            numerator,
            denominator,
        })
    }

    pub fn numerator(self) -> u16 {
        self.numerator
    }

    pub fn denominator(self) -> u16 {
        self.denominator
    }
}

// ´rule:domains:checked-arithmetic´

pub fn floor_mul_div(a: Sat, b: Sat, denominator: Sat) -> Result<Sat, Guard> {
    if denominator.is_zero() {
        return Err(Guard::Domain);
    }

    let numerator = u128::from(a.get())
        .checked_mul(u128::from(b.get()))
        .ok_or(Guard::Overflow)?;

    let quotient = numerator / u128::from(denominator.get());

    if quotient >= u128::from(TWO_51) {
        return Err(Guard::Domain);
    }

    Sat::new(quotient as u64)
}

pub fn floor_ratio(value: Sat, ratio: Ratio) -> Result<Sat, Guard> {
    let numerator = u128::from(value.get())
        .checked_mul(u128::from(ratio.numerator))
        .ok_or(Guard::Overflow)?;

    let quotient = numerator / u128::from(ratio.denominator);

    if quotient >= u128::from(TWO_51) {
        return Err(Guard::Domain);
    }

    Sat::new(quotient as u64)
}

pub fn checked_sum_sats<'a, I>(values: I) -> Result<Sat, Guard>
where
    I: IntoIterator<Item = &'a Sat>,
{
    Sat::checked_sum(values.into_iter().copied())
}

pub fn checked_add_to_map<K>(map: &mut BTreeMap<K, Sat>, key: K, value: Sat) -> Result<(), Guard>
where
    K: Ord,
{
    let current = map.get(&key).copied().unwrap_or(Sat::ZERO);

    let next = current.checked_add(value)?;
    map.insert(key, next);

    Ok(())
}
