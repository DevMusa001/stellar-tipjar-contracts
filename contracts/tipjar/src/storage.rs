use soroban_sdk::Env;

use crate::{
    AuctionKey, BridgeKey, CircuitBreakerKey, DataKey, DelegationKey, DisputeKey, FeeKey,
    InsuranceKey, LimitKey, LockedTipKey, MatchingKey, MilestoneKey, MultiSigKey, OptionKey,
    PrivateTipKey, RoleKey, SnapshotKey, StatsKey, StreamKey, SyntheticKey, VestingKey,
};

/// Default version for new contracts before any upgrade occurs.
pub const DEFAULT_CONTRACT_VERSION: u32 = 0;

/// Returns the current on-chain contract version.
pub fn get_contract_version(env: &Env) -> u32 {
    env.storage()
        .instance()
        .get(&DataKey::ContractVersion)
        .unwrap_or(DEFAULT_CONTRACT_VERSION)
}

/// Stores the current on-chain contract version.
pub fn set_contract_version(env: &Env, version: u32) {
    env.storage()
        .instance()
        .set(&DataKey::ContractVersion, &version);
}
