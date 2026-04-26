//! Synthetic Assets Module
//!
//! This module provides functionality for creating and managing synthetic assets
//! backed by creator tip pools. Synthetic assets allow users to gain exposure to
//! creator performance while providing creators with upfront liquidity.

pub mod types;
pub mod minting;
pub mod redemption;
pub mod oracle;
pub mod supply;
pub mod admin;
pub mod queries;
pub mod events;

pub use types::*;
pub use events::*;
