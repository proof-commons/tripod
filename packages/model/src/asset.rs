//! Assets, receipt classes, and maturity.
//!
//! Implements `(´def:architecture:assets´)`,
//! `(´def:architecture:receipt-class´)`, and `(´def:state:maturity´)`.

use crate::scalar::Cycle;

// ´def:architecture:assets´

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Asset {
    // Open external assets.
    Lbtc,
    Foreign(u32),

    // Closed value/capability assets.
    U,
    Ent,
    DistCtl,

    // Closed singleton identity/authority assets.
    Pid,
    Pace,
    EntAuth,
    DistAuth,
}

impl Asset {
    pub fn is_open(self) -> bool {
        matches!(self, Asset::Lbtc | Asset::Foreign(_))
    }

    pub fn is_closed(self) -> bool {
        !self.is_open()
    }

    pub fn authority(self) -> Option<Asset> {
        match self {
            Asset::U => Some(Asset::Pace),
            Asset::Ent => Some(Asset::EntAuth),
            Asset::DistCtl => Some(Asset::DistAuth),
            _ => None,
        }
    }

    pub fn is_singleton_root_asset(self) -> bool {
        matches!(
            self,
            Asset::Pid | Asset::Pace | Asset::EntAuth | Asset::DistAuth
        )
    }
}

// ´def:architecture:receipt-class´

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ReceiptClass {
    Live,
    TimeLocked,
}

// ´def:state:maturity´

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Maturity {
    Unannounced,
    Announced { cycle: Cycle },
    Complete,
}
