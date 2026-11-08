//! Card spec types live in the `card-domain` crate: one V3-shaped envelope
//! (`Card`) that round-trips as both V2 and V3, with unknown-field
//! preservation via `serde(flatten)`. Re-exported here so `cardgen` keeps a
//! single import site.

pub use card_domain::Card as CardFile;
