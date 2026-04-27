//! Royalty system for derivative content tips.
//!
//! Tracks content lineage and automatically distributes a percentage of tips
//! to original creators when derivative content receives tips.

use soroban_sdk::{contracttype, symbol_short, Address, Env};

use crate::DataKey;

/// Royalty configuration for a piece of content.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoyaltyConfig {
    /// The original creator who receives royalties.
    pub original_creator: Address,
    /// Royalty rate in basis points (e.g. 500 = 5%).
    pub rate_bps: u32,
    /// Maximum depth of lineage to traverse (capped at MAX_DEPTH).
    pub max_depth: u32,
}

/// A content lineage record linking derivative to original.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContentLineage {
    /// The derivative content creator.
    pub creator: Address,
    /// The parent content creator (one level up).
    pub parent_creator: Address,
    /// Royalty config for this lineage link.
    pub royalty_config: RoyaltyConfig,
}

/// Maximum lineage depth to prevent unbounded loops.
pub const MAX_DEPTH: u32 = 5;
/// Maximum royalty rate: 30%.
pub const MAX_ROYALTY_BPS: u32 = 3_000;

/// Register a royalty configuration for a creator's content.
pub fn register_royalty(env: &Env, creator: &Address, original_creator: &Address, rate_bps: u32, max_depth: u32) {
    let depth = if max_depth == 0 { MAX_DEPTH } else { max_depth.min(MAX_DEPTH) };
    let config = RoyaltyConfig {
        original_creator: original_creator.clone(),
        rate_bps,
        max_depth: depth,
    };
    env.storage().persistent().set(&DataKey::RoyaltyConfig(creator.clone()), &config);

    let lineage = ContentLineage {
        creator: creator.clone(),
        parent_creator: original_creator.clone(),
        royalty_config: config,
    };
    env.storage().persistent().set(&DataKey::ContentLineage(creator.clone()), &lineage);
}

/// Calculate and distribute royalties for a tip to `creator`.
///
/// Traverses the lineage chain up to `max_depth` levels, crediting royalties
/// to each ancestor's `RoyaltyBalance`. Returns the net amount after royalties.
pub fn distribute_royalties(
    env: &Env,
    creator: &Address,
    token_addr: &Address,
    tip_amount: i128,
) -> i128 {
    let mut remaining = tip_amount;
    let mut current = creator.clone();
    let mut depth = 0u32;

    loop {
        let lineage: Option<ContentLineage> = env
            .storage()
            .persistent()
            .get(&DataKey::ContentLineage(current.clone()));

        let lineage = match lineage {
            Some(l) => l,
            None => break,
        };

        if depth >= lineage.royalty_config.max_depth {
            break;
        }

        let royalty = (remaining * lineage.royalty_config.rate_bps as i128) / 10_000;
        if royalty <= 0 {
            break;
        }

        let bal_key = DataKey::RoyaltyBalance(
            lineage.royalty_config.original_creator.clone(),
            token_addr.clone(),
        );
        let current_bal: i128 = env.storage().persistent().get(&bal_key).unwrap_or(0);
        env.storage().persistent().set(&bal_key, &(current_bal + royalty));

        env.events().publish(
            (symbol_short!("royalty"),),
            (
                lineage.royalty_config.original_creator.clone(),
                creator.clone(),
                royalty,
                depth,
            ),
        );

        remaining -= royalty;
        current = lineage.royalty_config.original_creator.clone();
        depth += 1;
    }

    remaining
}
