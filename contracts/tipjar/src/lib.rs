#![no_std]
#![deny(unsafe_code)]
#![deny(missing_docs)]

pub mod interfaces;
pub mod integrations;
pub mod security;
pub mod bridge;
pub mod privacy;

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, panic_with_error, symbol_short,
    token, Address, BytesN, Env, Map, String, Vec,
};

pub mod upgrade;

#[cfg(test)]
extern crate std;

// Advanced Event System
pub mod events;

// Automated Market Maker
pub mod amm;

// Governance System
pub mod governance;

// Staking and Rewards
pub mod staking;

// Conditional tip execution
pub mod conditions;

// Dynamic fee adjustment
pub mod fees;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TipWithMessage {
    pub sender: Address,
    pub creator: Address,
    pub amount: i128,
    pub message: String,
    pub metadata: Map<String, String>,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Milestone {
    pub id: u64,
    pub creator: Address,
    pub goal_amount: i128,
    pub current_amount: i128,
    pub description: String,
    pub deadline: Option<u64>,
    pub completed: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatchTip {
    pub creator: Address,
    pub token: Address,
    pub amount: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LockedTip {
    pub sender: Address,
    pub creator: Address,
    pub token: Address,
    pub amount: i128,
    pub unlock_timestamp: u64,
}

/// Metadata stored on-chain for each tip with an optional message.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TipMetadata {
    pub sender: Address,
    pub amount: i128,
    pub message: Option<String>,
    pub timestamp: u64,
}

/// Internal record of a tip for refund tracking.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TipRecord {
    pub id: u64,
    pub sender: Address,
    pub creator: Address,
    pub token: Address,
    pub amount: i128,
    pub timestamp: u64,
    pub refunded: bool,
    pub refund_requested: bool,
    pub refund_approved: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TimePeriod {
    AllTime,
    Monthly,
    Weekly,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LeaderboardEntry {
    pub address: Address,
    pub total_amount: i128,
    pub tip_count: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParticipantKind {
    Tipper,
    Creator,
}

/// Query parameters for tip history retrieval.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TipHistoryQuery {
    pub creator: Option<Address>,
    pub sender: Option<Address>,
    pub min_amount: Option<i128>,
    pub max_amount: Option<i128>,
    pub start_time: Option<u64>,
    pub end_time: Option<u64>,
    pub limit: u32,
    pub offset: u32,
}

/// Role enum for role-based access control.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Role {
    Admin,
    Moderator,
    Creator,
}

/// A sponsor-funded tip matching program.
///
/// `match_ratio` is in basis points: 100 = 1:1, 200 = 2:1.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MatchingProgram {
    pub id: u64,
    pub sponsor: Address,
    pub creator: Address,
    pub token: Address,
    pub match_ratio: u32,
    pub max_match_amount: i128,
    pub current_matched: i128,
    pub active: bool,
}

/// A single recipient in a split tip, with share in basis points (10 000 = 100%).
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TipRecipient {
    pub creator: Address,
    /// Share in basis points; must be > 0. All shares must sum to 10 000.
    pub percentage: u32,
}

/// Subscription status.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SubscriptionStatus {
    Active,
    Paused,
    Cancelled,
}

/// A recurring tip subscription from a subscriber to a creator.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Subscription {
    pub subscriber: Address,
    pub creator: Address,
    pub token: Address,
    pub amount: i128,
    /// Minimum seconds between payments.
    pub interval_seconds: u64,
    pub last_payment: u64,
    pub next_payment: u64,
    pub status: SubscriptionStatus,
}

/// Storage layout for persistent contract data.
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    /// Token contract address whitelist state (bool).
    TokenWhitelist(Address),
    /// Creator's currently withdrawable balance held by this contract per token.
    CreatorBalance(Address, Address), // (creator, token)
    /// Historical total tips ever received by creator per token.
    CreatorTotal(Address, Address),   // (creator, token)
    /// Emergency pause state (bool).
    Paused,
    /// Contract administrator (Address).
    Admin,
    /// Messages appended for a creator.
    CreatorMessages(Address),
    /// Current number of milestones for a creator (used for ID).
    MilestoneCounter(Address),
    /// Data for a specific milestone.
    Milestone(Address, u64),
    /// Active milestone IDs for a creator to track.
    ActiveMilestones(Address),
    /// Maps an address to its assigned Role (persistent).
    UserRole(Address),
    /// Maps a Role to the set of addresses holding it (persistent).
    RoleMembers(Role),
    /// Aggregate stats for a tipper in a specific time bucket (bucket_id: 0=AllTime, YYYYMM=Monthly, YYYYWW=Weekly).
    TipperAggregate(Address, u32),
    /// Aggregate stats for a creator in a specific time bucket.
    CreatorAggregate(Address, u32),
    /// Ordered list of all known tipper addresses for a bucket.
    TipperParticipants(u32),
    /// Ordered list of all known creator addresses for a bucket.
    CreatorParticipants(u32),
    /// Locked tip record keyed by (creator, tip_id).
    LockedTip(Address, u64),
    /// Per-creator counter for assigning tip IDs (u64).
    LockedTipCounter(Address),
    /// Global matching program counter.
    MatchingCounter,
    /// Individual matching program by ID.
    MatchingProgram(u64),
    /// Matching program IDs indexed under a creator.
    CreatorMatchingPrograms(Address),
    /// Individual tip record by global tip ID.
    TipRecord(u64),
    /// Global tip counter for assigning tip IDs.
    TipCounter,
    /// Off-chain oracle approval flag keyed by condition ID.
    OffchainCondition(BytesN<32>),
    /// Most-recently computed dynamic fee in basis points.
    CurrentFeeBps,
    /// Monotonically increasing contract version, incremented on each upgrade.
    ContractVersion,
    /// Subscription keyed by (subscriber, creator).
    Subscription(Address, Address),
    /// Human-readable reason stored when the contract is paused.
    PauseReason,
    /// TipMetadata keyed by (creator, tip_index).
    TipHistory(Address, u64),
    /// Total number of tips with metadata stored for a creator.
    TipCount(Address),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum TipJarError {
    AlreadyInitialized = 1,
    TokenNotWhitelisted = 2,
    InvalidAmount = 3,
    NothingToWithdraw = 4,
    MessageTooLong = 5,
    MilestoneNotFound = 6,
    MilestoneAlreadyCompleted = 7,
    InvalidGoalAmount = 8,
    Unauthorized = 9,
    RoleNotFound = 10,
    BatchTooLarge = 11,
    InsufficientBalance = 12,
    InvalidUnlockTime = 13,
    TipStillLocked = 14,
    LockedTipNotFound = 15,
    MatchingProgramNotFound = 16,
    MatchingProgramInactive = 17,
    InvalidMatchRatio = 18,
    DexNotConfigured = 19,
    NftNotConfigured = 20,
    SwapFailed = 21,
    ConditionFailed = 22,
    /// Caller is not the stored admin; upgrade rejected.
    UpgradeUnauthorized = 23,
    /// Subscription does not exist.
    SubscriptionNotFound = 24,
    /// Subscription is not in Active state.
    SubscriptionNotActive = 25,
    /// Payment interval has not elapsed yet.
    PaymentNotDue = 26,
    /// Interval is below the minimum allowed.
    InvalidInterval = 27,
    /// Recipient count is outside the allowed range (2–10).
    InvalidRecipientCount = 28,
    /// Basis-point shares do not sum to 10 000.
    InvalidPercentageSum = 29,
    /// An individual share is zero.
    InvalidPercentage = 30,
    /// Contract is paused; state-changing operations are blocked.
    ContractPaused = 31,
}

#[contract]
pub struct TipJarContract;

#[contractimpl]
impl TipJarContract {
    // ── pause guard ──────────────────────────────────────────────────────────

    fn require_not_paused(env: &Env) {
        if env
            .storage()
            .instance()
            .get::<DataKey, bool>(&DataKey::Paused)
            .unwrap_or(false)
        {
            panic_with_error!(env, TipJarError::ContractPaused);
        }
    }

    // ── leaderboard helpers ──────────────────────────────────────────────────

    fn update_leaderboard_stats(
        env: &Env,
        tipper: &Address,
        creator: &Address,
        amount: i128,
    ) {
        const BUCKET_ALL_TIME: u32 = 0;
        Self::update_aggregate(env, tipper, amount, BUCKET_ALL_TIME, ParticipantKind::Tipper);
        Self::update_aggregate(env, creator, amount, BUCKET_ALL_TIME, ParticipantKind::Creator);
    }

    fn update_aggregate(
        env: &Env,
        addr: &Address,
        amount: i128,
        bucket: u32,
        kind: ParticipantKind,
    ) {
        let agg_key = match kind {
            ParticipantKind::Tipper => DataKey::TipperAggregate(addr.clone(), bucket),
            ParticipantKind::Creator => DataKey::CreatorAggregate(addr.clone(), bucket),
        };
        let mut entry: LeaderboardEntry = env
            .storage()
            .persistent()
            .get(&agg_key)
            .unwrap_or(LeaderboardEntry {
                address: addr.clone(),
                total_amount: 0,
                tip_count: 0,
            });
        entry.total_amount += amount;
        entry.tip_count += 1;
        env.storage().persistent().set(&agg_key, &entry);

        let part_key = match kind {
            ParticipantKind::Tipper => DataKey::TipperParticipants(bucket),
            ParticipantKind::Creator => DataKey::CreatorParticipants(bucket),
        };
        let mut participants: Vec<Address> = env
            .storage()
            .persistent()
            .get(&part_key)
            .unwrap_or_else(|| Vec::new(env));
        if !participants.contains(addr) {
            participants.push_back(addr.clone());
            env.storage().persistent().set(&part_key, &participants);
        }
    }

    // ── initialization ───────────────────────────────────────────────────────

    /// One-time setup to choose the administrator for the TipJar.
    pub fn init(env: Env, admin: Address, fee_basis_points: u32, refund_window_seconds: u64) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic_with_error!(&env, TipJarError::AlreadyInitialized as u32);
        }
        if fee_basis_points > 500 {
            panic_with_error!(&env, TipJarError::FeeExceedsMaximum);
        }
        env.storage().instance().put(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::FeeBasisPoints, &fee_basis_points);
        env.storage().instance().set(&DataKey::RefundWindow, &refund_window_seconds);
    }

    /// Sets an off-chain condition flag that can later be referenced in
    /// conditional tip execution.
    pub fn set_offchain_condition(
        env: Env,
        oracle: Address,
        condition_id: BytesN<32>,
        approved: bool,
    ) {
        oracle.require_auth();
        conditions::evaluator::set_offchain_approval(&env, &condition_id, approved);
    }

    /// Adds a token to the whitelist. Admin only.
    pub fn add_token(env: Env, admin: Address, token: Address) {
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic_with_error!(&env, TipJarError::Unauthorized);
        }
        env.storage().instance().set(&DataKey::TokenWhitelist(token), &true);
    }

    /// Pauses all state-changing operations. Admin only.
    ///
    /// `reason` is stored on-chain for transparency.
    /// Emits `("paused",)` with data `(admin, reason)`.
    pub fn pause(env: Env, admin: Address, reason: String) {
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic_with_error!(&env, TipJarError::Unauthorized);
        }
        env.storage().instance().set(&DataKey::Paused, &true);
        env.storage().instance().set(&DataKey::PauseReason, &reason);
        env.events()
            .publish((symbol_short!("paused"),), (admin, reason));
    }

    /// Resumes normal operations. Admin only.
    ///
    /// Emits `("unpaused",)` with data `admin`.
    pub fn unpause(env: Env, admin: Address) {
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic_with_error!(&env, TipJarError::Unauthorized);
        }
        env.storage().instance().set(&DataKey::Paused, &false);
        env.storage().instance().remove(&DataKey::PauseReason);
        env.events().publish((symbol_short!("unpaused"),), admin);
    }

    /// Returns `true` when the contract is paused.
    pub fn is_paused(env: Env) -> bool {
        env.storage()
            .instance()
            .get::<DataKey, bool>(&DataKey::Paused)
            .unwrap_or(false)
    }

    /// Transfers `amount` of `token` from `sender` into escrow for `creator`.
    ///
    /// Deducts the platform fee before crediting the creator. Returns the tip ID.
    /// Emits `("tip", creator)` with data `(sender, amount)`.
    pub fn tip(env: Env, sender: Address, creator: Address, token: Address, amount: i128) -> u64 {
        sender.require_auth();
        if amount <= 0 {
            panic_with_error!(&env, TipJarError::InvalidAmount);
        }
        let whitelisted: bool = env
            .storage()
            .instance()
            .get(&DataKey::TokenWhitelist(token.clone()))
            .unwrap_or(false);
        if !whitelisted {
            panic_with_error!(&env, TipJarError::TokenNotWhitelisted);
        }
        token::Client::new(&env, &token).transfer(&sender, &env.current_contract_address(), &amount);

        let fee_bp: u32 = env.storage().instance().get(&DataKey::FeeBasisPoints).unwrap_or(0);
        let fee: i128 = (amount * fee_bp as i128) / 10000;
        let creator_amount = amount - fee;

        if fee > 0 {
            let fee_key = DataKey::PlatformFeeBalance(token.clone());
            let new_fee_bal: i128 = env.storage().instance().get(&fee_key).unwrap_or(0) + fee;
            env.storage().instance().set(&fee_key, &new_fee_bal);
        }

        let bal_key = DataKey::CreatorBalance(creator.clone(), token.clone());
        let existing_bal: i128 = env.storage().persistent().get(&bal_key)
            .unwrap_or_else(|| env.storage().instance().get(&bal_key).unwrap_or(0));
        let new_bal: i128 = existing_bal + amount;
        env.storage().persistent().set(&bal_key, &new_bal);
        let tot_key = DataKey::CreatorTotal(creator.clone(), token.clone());
        let existing_tot: i128 = env.storage().persistent().get(&tot_key)
            .unwrap_or_else(|| env.storage().instance().get(&tot_key).unwrap_or(0));
        let new_tot: i128 = existing_tot + amount;
        env.storage().persistent().set(&tot_key, &new_tot);
        Self::update_leaderboard_stats(&env, &sender, &creator, amount);
        env.events().publish((symbol_short!("tip"), creator.clone()), (sender, amount));
        tip_id
    }

    /// Withdraws the full escrowed balance for `creator` in `token`.
    ///
    /// Emits `("withdraw", creator)` with data `amount`.
    pub fn withdraw(env: Env, creator: Address, token: Address) {
        Self::require_not_paused(&env);
        creator.require_auth();
        let bal_key = DataKey::CreatorBalance(creator.clone(), token.clone());
        let amount: i128 = env.storage().persistent().get(&bal_key)
            .unwrap_or_else(|| env.storage().instance().get(&bal_key).unwrap_or(0));
        if amount == 0 {
            panic_with_error!(&env, TipJarError::NothingToWithdraw);
        }
        env.storage().persistent().set(&bal_key, &0i128);
        token::Client::new(&env, &token).transfer(&env.current_contract_address(), &creator, &amount);
        env.events().publish((symbol_short!("withdraw"), creator.clone()), amount);
    }

    /// Returns the current withdrawable balance for `creator` in `token`.
    pub fn get_withdrawable_balance(env: Env, creator: Address, token: Address) -> i128 {
        let key = DataKey::CreatorBalance(creator.clone(), token.clone());
        env.storage().persistent().get(&key)
            .unwrap_or_else(|| env.storage().instance().get(&key).unwrap_or(0))
    }

    /// Returns the historical total tips received by `creator` in `token`.
    pub fn get_total_tips(env: Env, creator: Address, token: Address) -> i128 {
        let key = DataKey::CreatorTotal(creator.clone(), token.clone());
        env.storage().persistent().get(&key)
            .unwrap_or_else(|| env.storage().instance().get(&key).unwrap_or(0))
    }

    /// Returns the top N participants (tippers or creators) for a given time period.
    ///
    /// Results are sorted by `total_amount` descending. `limit` is capped at 100.
    pub fn get_leaderboard(
        env: Env,
        period: TimePeriod,
        kind: ParticipantKind,
        limit: u32,
    ) -> Vec<LeaderboardEntry> {
        let bucket = match period {
            TimePeriod::AllTime => 0,
            _ => 0, // Monthly/Weekly not implemented; default to AllTime
        };
        let part_key = match kind {
            ParticipantKind::Tipper => DataKey::TipperParticipants(bucket),
            ParticipantKind::Creator => DataKey::CreatorParticipants(bucket),
        };
        let participants: Vec<Address> = env
            .storage()
            .persistent()
            .get(&part_key)
            .unwrap_or_else(|| Vec::new(&env));

        let cap = if limit > 100 { 100 } else { limit };
        let mut entries = Vec::new(&env);
        for addr in participants.iter() {
            let agg_key = match kind {
                ParticipantKind::Tipper => DataKey::TipperAggregate(addr.clone(), bucket),
                ParticipantKind::Creator => DataKey::CreatorAggregate(addr.clone(), bucket),
            };
            if let Some(entry) = env.storage().persistent().get::<_, LeaderboardEntry>(&agg_key) {
                entries.push_back(entry);
            }
        }

        // Build top-N by repeated linear scan (O(n*cap), cap ≤ 100, n ≤ participants).
        // Avoids in-place mutation since Soroban Vec has no set().
        let mut result = Vec::new(&env);
        let mut used = Vec::<u32>::new(&env);
        for _ in 0..cap {
            if used.len() == entries.len() {
                break;
            }
            let mut best_idx: Option<u32> = None;
            let mut best_amt: i128 = -1;
            for idx in 0..entries.len() {
                if used.contains(&idx) {
                    continue;
                }
                let amt = entries.get(idx).unwrap().total_amount;
                if amt > best_amt {
                    best_amt = amt;
                    best_idx = Some(idx);
                }
            }
            if let Some(idx) = best_idx {
                result.push_back(entries.get(idx).unwrap());
                used.push_back(idx);
            }
        }
        result
    }

    /// Executes a token tip only if all provided conditions evaluate to true.
    ///
    /// Returns true when the transfer is executed and false when conditions fail.
    pub fn execute_conditional_tip(
        env: Env,
        sender: Address,
        creator: Address,
        token: Address,
        amount: i128,
        condition_list: Vec<conditions::types::Condition>,
    ) -> bool {
        Self::require_not_paused(&env);
        sender.require_auth();

        if amount <= 0 {
            panic_with_error!(&env, TipJarError::InvalidAmount);
        }

        let is_valid = conditions::evaluator::evaluate_all(&env, &condition_list);
        if !is_valid {
            return false;
        }

        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&sender, &creator, &amount);

        env.events().publish(
            (symbol_short!("condtip"), sender.clone()),
            (creator.clone(), token, amount),
        );

        true
    }

    /// Returns the last dynamically computed fee in basis points.
    ///
    /// Defaults to the base fee (100 bps = 1%) if no tip has been processed yet.
    pub fn get_current_fee_bps(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::CurrentFeeBps)
            .unwrap_or(fees::calculator::BASE_FEE_BPS)
    }

    /// Like `tip`, but deducts a dynamic platform fee before crediting the creator.
    ///
    /// `congestion` is a `u32` mapped as: 0 = Low, 1 = Normal, 2 = High.
    /// The fee is retained in the contract; the creator receives `amount - fee`.
    ///
    /// Emits `("tip", creator)` with data `(sender, net_amount)` and
    /// `("fee", creator)` with data `(fee_amount, fee_bps)`.
    pub fn tip_with_fee(
        env: Env,
        sender: Address,
        creator: Address,
        token: Address,
        amount: i128,
        congestion: u32,
    ) {
        Self::require_not_paused(&env);
        sender.require_auth();
        if amount <= 0 {
            panic_with_error!(&env, TipJarError::InvalidAmount);
        }
        let whitelisted: bool = env
            .storage()
            .instance()
            .get(&DataKey::TokenWhitelist(token.clone()))
            .unwrap_or(false);
        if !whitelisted {
            panic_with_error!(&env, TipJarError::TokenNotWhitelisted);
        }

        let level = match congestion {
            0 => fees::CongestionLevel::Low,
            2 => fees::CongestionLevel::High,
            _ => fees::CongestionLevel::Normal,
        };
        let (fee, fee_bps) = fees::compute_fee(&env, amount, level);
        let net = amount - fee;

        token::Client::new(&env, &token).transfer(
            &sender,
            &env.current_contract_address(),
            &amount,
        );

        let bal_key = DataKey::CreatorBalance(creator.clone(), token.clone());
        let new_bal: i128 = env.storage().instance().get(&bal_key).unwrap_or(0) + net;
        env.storage().instance().set(&bal_key, &new_bal);

        let tot_key = DataKey::CreatorTotal(creator.clone(), token.clone());
        let new_tot: i128 = env.storage().instance().get(&tot_key).unwrap_or(0) + net;
        env.storage().instance().set(&tot_key, &new_tot);

        env.events()
            .publish((symbol_short!("tip"), creator.clone()), (sender, net));
        env.events()
            .publish((symbol_short!("fee"), creator.clone()), (fee, fee_bps));
    }

    /// Upgrades the contract WASM to `new_wasm_hash`. Admin only.
    ///
    /// Increments the on-chain version and emits `("upgraded",)` with the new
    /// version number.  All storage is preserved by the Soroban host.
    pub fn upgrade(env: Env, new_wasm_hash: BytesN<32>) {
        upgrade::upgrade(&env, new_wasm_hash);
    }

    /// Returns the current contract version (0 before the first upgrade).
    pub fn get_version(env: Env) -> u32 {
        upgrade::get_version(&env)
    }

    /// Creates a recurring tip subscription from `subscriber` to `creator`.
    ///
    /// The first payment becomes due immediately (at creation time).
    /// Minimum interval is 1 day (86 400 seconds).
    ///
    /// Emits `("sub_new", creator)` with data `(subscriber, amount, interval_seconds)`.
    pub fn create_subscription(
        env: Env,
        subscriber: Address,
        creator: Address,
        token: Address,
        amount: i128,
        interval_seconds: u64,
    ) {
        Self::require_not_paused(&env);
        subscriber.require_auth();
        if amount <= 0 {
            panic_with_error!(&env, TipJarError::InvalidAmount);
        }
        const MIN_INTERVAL: u64 = 86_400;
        if interval_seconds < MIN_INTERVAL {
            panic_with_error!(&env, TipJarError::InvalidInterval);
        }
        let now = env.ledger().timestamp();
        let sub = Subscription {
            subscriber: subscriber.clone(),
            creator: creator.clone(),
            token,
            amount,
            interval_seconds,
            last_payment: 0,
            next_payment: now,
            status: SubscriptionStatus::Active,
        };
        env.storage()
            .persistent()
            .set(&DataKey::Subscription(subscriber.clone(), creator.clone()), &sub);
        env.events().publish(
            (symbol_short!("sub_new"), creator),
            (subscriber, amount, interval_seconds),
        );
    }

    /// Executes a due subscription payment, transferring tokens from subscriber
    /// into escrow for the creator.
    ///
    /// Anyone may call this; the subscriber's auth is pulled via `transfer`.
    /// Emits `("sub_pay", creator)` with data `(subscriber, amount)`.
    pub fn execute_subscription_payment(env: Env, subscriber: Address, creator: Address) {
        Self::require_not_paused(&env);
        let key = DataKey::Subscription(subscriber.clone(), creator.clone());
        let mut sub: Subscription = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or_else(|| panic_with_error!(&env, TipJarError::SubscriptionNotFound));

        if sub.status != SubscriptionStatus::Active {
            panic_with_error!(&env, TipJarError::SubscriptionNotActive);
        }
        let now = env.ledger().timestamp();
        if now < sub.next_payment {
            panic_with_error!(&env, TipJarError::PaymentNotDue);
        }

        token::Client::new(&env, &sub.token).transfer(
            &subscriber,
            &env.current_contract_address(),
            &sub.amount,
        );

        let bal_key = DataKey::CreatorBalance(creator.clone(), sub.token.clone());
        let bal: i128 = env.storage().persistent().get(&bal_key).unwrap_or(0);
        env.storage().persistent().set(&bal_key, &(bal + sub.amount));

        let tot_key = DataKey::CreatorTotal(creator.clone(), sub.token.clone());
        let tot: i128 = env.storage().persistent().get(&tot_key).unwrap_or(0);
        env.storage().persistent().set(&tot_key, &(tot + sub.amount));

        sub.last_payment = now;
        sub.next_payment = now + sub.interval_seconds;
        env.storage().persistent().set(&key, &sub);

        env.events().publish(
            (symbol_short!("sub_pay"), creator),
            (subscriber, sub.amount),
        );
    }

    /// Pauses an active subscription. Only the subscriber may pause.
    ///
    /// Emits `("sub_paus", creator)` with data `subscriber`.
    pub fn pause_subscription(env: Env, subscriber: Address, creator: Address) {
        Self::require_not_paused(&env);
        subscriber.require_auth();
        let key = DataKey::Subscription(subscriber.clone(), creator.clone());
        let mut sub: Subscription = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or_else(|| panic_with_error!(&env, TipJarError::SubscriptionNotFound));
        if sub.status != SubscriptionStatus::Active {
            panic_with_error!(&env, TipJarError::SubscriptionNotActive);
        }
        sub.status = SubscriptionStatus::Paused;
        env.storage().persistent().set(&key, &sub);
        env.events()
            .publish((symbol_short!("sub_paus"), creator), subscriber);
    }

    /// Resumes a paused subscription. Only the subscriber may resume.
    ///
    /// Resets `next_payment` to now so a payment can be executed immediately.
    /// Emits `("sub_res", creator)` with data `subscriber`.
    pub fn resume_subscription(env: Env, subscriber: Address, creator: Address) {
        Self::require_not_paused(&env);
        subscriber.require_auth();
        let key = DataKey::Subscription(subscriber.clone(), creator.clone());
        let mut sub: Subscription = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or_else(|| panic_with_error!(&env, TipJarError::SubscriptionNotFound));
        if sub.status != SubscriptionStatus::Paused {
            panic_with_error!(&env, TipJarError::SubscriptionNotActive);
        }
        sub.status = SubscriptionStatus::Active;
        sub.next_payment = env.ledger().timestamp();
        env.storage().persistent().set(&key, &sub);
        env.events()
            .publish((symbol_short!("sub_res"), creator), subscriber);
    }

    /// Cancels a subscription. Only the subscriber may cancel.
    ///
    /// Emits `("sub_cncl", creator)` with data `subscriber`.
    pub fn cancel_subscription(env: Env, subscriber: Address, creator: Address) {
        Self::require_not_paused(&env);
        subscriber.require_auth();
        let key = DataKey::Subscription(subscriber.clone(), creator.clone());
        let mut sub: Subscription = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or_else(|| panic_with_error!(&env, TipJarError::SubscriptionNotFound));
        sub.status = SubscriptionStatus::Cancelled;
        env.storage().persistent().set(&key, &sub);
        env.events()
            .publish((symbol_short!("sub_cncl"), creator), subscriber);
    }

    /// Returns the subscription between `subscriber` and `creator`, if it exists.
    pub fn get_subscription(
        env: Env,
        subscriber: Address,
        creator: Address,
    ) -> Option<Subscription> {
        env.storage()
            .persistent()
            .get(&DataKey::Subscription(subscriber, creator))
    }

    /// Like `tip`, but stores an optional on-chain message and metadata.
    ///
    /// `message` is limited to 200 Unicode scalar values (character count, not
    /// byte count) so that emoji and multi-byte characters are treated fairly.
    /// Panics with `TipJarError::MessageTooLong` when the limit is exceeded.
    ///
    /// Metadata is stored in persistent storage under `TipHistory(creator, index)`
    /// and the per-creator counter `TipCount(creator)` is incremented.
    ///
    /// Emits `("tip_msg", creator)` with data `(sender, amount, message)`.
    pub fn tip_with_message(
        env: Env,
        sender: Address,
        creator: Address,
        token: Address,
        amount: i128,
        message: Option<String>,
    ) -> u64 {
        Self::require_not_paused(&env);
        sender.require_auth();

        // Validate message length by character count (not bytes) to support emoji.
        if let Some(ref msg) = message {
            // Soroban String stores raw bytes; convert to a &str slice for char counting.
            let bytes = msg.to_string();
            let char_count = bytes.chars().count();
            if char_count > 200 {
                panic_with_error!(&env, TipJarError::MessageTooLong);
            }
        }

        if amount <= 0 {
            panic_with_error!(&env, TipJarError::InvalidAmount);
        }
        let whitelisted: bool = env
            .storage()
            .instance()
            .get(&DataKey::TokenWhitelist(token.clone()))
            .unwrap_or(false);
        if !whitelisted {
            panic_with_error!(&env, TipJarError::TokenNotWhitelisted);
        }

        token::Client::new(&env, &token).transfer(
            &sender,
            &env.current_contract_address(),
            &amount,
        );

        let fee_bp: u32 = env
            .storage()
            .instance()
            .get(&DataKey::FeeBasisPoints)
            .unwrap_or(0);
        let fee: i128 = (amount * fee_bp as i128) / 10_000;
        let creator_amount = amount - fee;

        if fee > 0 {
            let fee_key = DataKey::PlatformFeeBalance(token.clone());
            let new_fee_bal: i128 =
                env.storage().instance().get(&fee_key).unwrap_or(0) + fee;
            env.storage().instance().set(&fee_key, &new_fee_bal);
        }

        let bal_key = DataKey::CreatorBalance(creator.clone(), token.clone());
        let existing_bal: i128 = env
            .storage()
            .persistent()
            .get(&bal_key)
            .unwrap_or_else(|| env.storage().instance().get(&bal_key).unwrap_or(0));
        env.storage()
            .persistent()
            .set(&bal_key, &(existing_bal + creator_amount));

        let tot_key = DataKey::CreatorTotal(creator.clone(), token.clone());
        let existing_tot: i128 = env
            .storage()
            .persistent()
            .get(&tot_key)
            .unwrap_or_else(|| env.storage().instance().get(&tot_key).unwrap_or(0));
        env.storage()
            .persistent()
            .set(&tot_key, &(existing_tot + amount));

        Self::update_leaderboard_stats(&env, &sender, &creator, amount);

        // Store metadata and increment tip count.
        let count_key = DataKey::TipCount(creator.clone());
        let tip_index: u64 = env
            .storage()
            .persistent()
            .get(&count_key)
            .unwrap_or(0u64);

        let timestamp = env.ledger().timestamp();
        let metadata = TipMetadata {
            sender: sender.clone(),
            amount,
            message: message.clone(),
            timestamp,
        };
        env.storage()
            .persistent()
            .set(&DataKey::TipHistory(creator.clone(), tip_index), &metadata);
        env.storage()
            .persistent()
            .set(&count_key, &(tip_index + 1));

        env.events().publish(
            (symbol_short!("tip_msg"), creator.clone()),
            (sender, amount, message),
        );

        tip_index
    }

    /// Returns the most recent tips (with metadata) for `creator`, newest first.
    ///
    /// `limit` is capped at 100 to bound storage reads.
    pub fn get_tip_history(env: Env, creator: Address, limit: u32) -> Vec<TipMetadata> {
        let count_key = DataKey::TipCount(creator.clone());
        let total: u64 = env
            .storage()
            .persistent()
            .get(&count_key)
            .unwrap_or(0u64);

        let cap = if limit > 100 { 100 } else { limit } as u64;
        let mut result = Vec::new(&env);

        if total == 0 {
            return result;
        }

        // Iterate from newest (total-1) down to oldest, up to `cap` entries.
        let mut idx = total;
        let mut fetched: u64 = 0;
        while idx > 0 && fetched < cap {
            idx -= 1;
            if let Some(meta) = env
                .storage()
                .persistent()
                .get::<_, TipMetadata>(&DataKey::TipHistory(creator.clone(), idx))
            {
                result.push_back(meta);
                fetched += 1;
            }
        }

        result
    }

    /// Splits a single tip among multiple recipients proportionally.
    ///
    /// `recipients` must contain 2–10 entries whose `percentage` values (basis
    /// points) sum to exactly 10 000.  The last recipient absorbs any rounding
    /// remainder so the full `amount` is always distributed.
    ///
    /// Emits `("tip_splt", creator)` with data `(sender, recipient_amount, percentage)`
    /// for every recipient.
    pub fn tip_split(
        env: Env,
        sender: Address,
        token: Address,
        recipients: Vec<TipRecipient>,
        amount: i128,
    ) {
        Self::require_not_paused(&env);
        sender.require_auth();
        if amount <= 0 {
            panic_with_error!(&env, TipJarError::InvalidAmount);
        }
        let count = recipients.len();
        if count < 2 || count > 10 {
            panic_with_error!(&env, TipJarError::InvalidRecipientCount);
        }
        let mut total_pct: u32 = 0;
        for r in recipients.iter() {
            if r.percentage == 0 {
                panic_with_error!(&env, TipJarError::InvalidPercentage);
            }
            total_pct += r.percentage;
        }
        if total_pct != 10_000 {
            panic_with_error!(&env, TipJarError::InvalidPercentageSum);
        }

        let whitelisted: bool = env
            .storage()
            .instance()
            .get(&DataKey::TokenWhitelist(token.clone()))
            .unwrap_or(false);
        if !whitelisted {
            panic_with_error!(&env, TipJarError::TokenNotWhitelisted);
        }

        token::Client::new(&env, &token).transfer(
            &sender,
            &env.current_contract_address(),
            &amount,
        );

        let last_idx = count - 1;
        let mut distributed: i128 = 0;
        for (i, r) in recipients.iter().enumerate() {
            let share = if i == last_idx as usize {
                amount - distributed
            } else {
                (amount * r.percentage as i128) / 10_000
            };

            let bal_key = DataKey::CreatorBalance(r.creator.clone(), token.clone());
            let bal: i128 = env.storage().persistent().get(&bal_key).unwrap_or(0);
            env.storage().persistent().set(&bal_key, &(bal + share));

            let tot_key = DataKey::CreatorTotal(r.creator.clone(), token.clone());
            let tot: i128 = env.storage().persistent().get(&tot_key).unwrap_or(0);
            env.storage().persistent().set(&tot_key, &(tot + share));

            distributed += share;

            env.events().publish(
                (symbol_short!("tip_splt"), r.creator.clone()),
                (sender.clone(), share, r.percentage),
            );
        }
    }
}