//! Operation bodies — pure `World -> Result<World, Guard>` transitions
//! built exclusively through the transition kernel.
//!
//! Each submodule carries the spec labels it implements:
//!
//! - [`request`] — `´protocol:operations:create-request´`,
//!   `´branch:operations:cancel-request´`.
//! - [`admission`] — `´branch:operations:admit-deposits´`.
//! - [`cycle`] — `´def:operations:cycle-caller´`, `´branch:operations:cycle´`.
//! - [`settlement`] — `´branch:operations:settle-distribution´`.
//! - [`transfer`] — `´protocol:operations:transfer´`.
//! - [`redeem`] — `´branch:operations:redeem´`.
//! - [`relabel`] — `´branch:operations:receipt-relabel´`.
//! - [`burn`] — `´branch:operations:burn´`.
//! - [`ash`] — `´branch:operations:compact-ash´`, `´branch:operations:clear´`.
//! - [`maturity`] — `´branch:operations:announce-maturity´`.
//! - [`inject`] — `´def:verification:open-object-injection´`.

pub mod admission;
pub mod ash;
pub mod burn;
pub mod cycle;
pub mod inject;
pub mod maturity;
pub mod redeem;
pub mod relabel;
pub mod request;
pub mod settlement;
pub mod transfer;

pub use admission::AdmitDeposits;
pub use ash::{ClearAsh, CompactAsh};
pub use burn::BurnReceipts;
pub use cycle::{CycleCaller, RunCycle};
pub use inject::inject_open_object;
pub use maturity::AnnounceMaturity;
pub use redeem::RedeemReceipt;
pub use relabel::RelabelReceipts;
pub use request::{CancelRequest, CreateRequest};
pub use settlement::SettleDistribution;
pub use transfer::{ReceiptDestination, TransferReceipts};
