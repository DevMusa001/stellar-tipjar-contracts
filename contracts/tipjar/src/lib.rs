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

/// A tip record that includes an optional memo and timestamp.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TipWithMemo {
    pub sender: Address,
    pub amount: i128,
    pub memo: Option<String>,
    pub timestamp: u64,
}

/// Combined creator stats stored in a single persistent entry to reduce storage reads/writes.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreatorStats {
    pub balance: i128,
    pub total: i128,
}

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
pub struct TipWithExpiry {
    pub tipper: Address,
    pub creator: Address,
    pub amount: i128,
    pub created_at: u64,
    pub expires_at: u64,
    pub claimed: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Delegation {
    pub creator: Address,
    pub delegate: Address,
    pub max_amount: i128,
    pub used_amount: i128,
    pub expires_at: u64,
    pub active: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VestingSchedule {
    pub id: u64,
    pub creator: Address,
    pub tipper: Address,
    pub token: Address,
    pub total_amount: i128,
    pub start_time: u64,
    pub cliff_duration: u64,
    pub vesting_duration: u64,
    pub withdrawn: i128,
    pub created_at: u64,
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

/// A time-boxed sponsor matching campaign with a budget and expiry.
///
/// `match_ratio` is in basis points: 100 = 1:1, 200 = 2:1.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MatchingCampaign {
    pub sponsor: Address,
    pub creator: Address,
    pub token: Address,
    /// Match ratio in basis points (100 = 1:1, 200 = 2:1).
    pub match_ratio: u32,
    pub total_budget: i128,
    pub remaining_budget: i128,
    pub start_time: u64,
    pub end_time: u64,
    pub active: bool,
}

/// Per-creator withdrawal rate-limit state.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WithdrawalLimits {
    /// Maximum amount withdrawable within a 24-hour window (0 = unlimited).
    pub daily_limit: i128,
    /// Minimum seconds that must elapse between withdrawals (0 = no cooldown).
    pub cooldown_seconds: u64,
    /// Ledger timestamp of the last successful withdrawal.
    pub last_withdrawal: u64,
    /// Amount already withdrawn in the current 24-hour window.
    pub withdrawn_today: i128,
    /// Ledger timestamp when the current 24-hour window started.
    pub day_start: u64,
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

/// A time-locked tip that can only be withdrawn after `unlock_time`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TimeLock {
    pub sender: Address,
    pub creator: Address,
    pub token: Address,
    pub amount: i128,
    pub unlock_time: u64,
    pub created_at: u64,
    pub expires_at: u64,
    pub cancelled: bool,
}

/// A pending multi-signature withdrawal request.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MultiSigWithdrawal {
    pub request_id: u64,
    pub creator: Address,
    pub token: Address,
    pub amount: i128,
    pub approvals: Vec<Address>,
    pub required_approvals: u32,
    pub expires_at: u64,
    pub executed: bool,
    pub cancelled: bool,
}

/// Per-contract multi-sig configuration set by admin.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MultiSigConfig {
    /// Withdrawal amount above which multi-sig is required (0 = always require).
    pub threshold: i128,
    /// Number of approvals needed to execute.
    pub required_approvals: u32,
    /// Seconds until a pending request expires.
    pub expiry_seconds: u64,
    /// Authorised signers.
    pub signers: Vec<Address>,
}

/// Leaderboard category selector.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LeaderboardType {
    TopTippers,
    TopCreators,
}

/// Immutable snapshot of key contract state for migration / audit purposes.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StateSnapshot {
    pub snapshot_id: u64,
    pub timestamp: u64,
    pub metadata: soroban_sdk::String,
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
    /// Platform fee in basis points (u32).
    FeeBasisPoints,
    /// Accumulated platform fee balance per token.
    PlatformFeeBalance(Address),
    /// Refund window in seconds (u64).
    RefundWindow,
    /// Leaderboard entries for a given LeaderboardType.
    Leaderboard(LeaderboardType),
    /// Tipper total tips sent (i128).
    TipperTotal(Address),
    /// State snapshot keyed by snapshot_id.
    Snapshot(u64),
    /// Next snapshot ID counter.
    LatestSnapshot,
    /// Per-creator withdrawal rate-limit state.
    WithdrawalLimits(Address),
    /// Platform-wide default withdrawal limits applied when no per-creator config exists.
    DefaultWithdrawalLimits,
    /// Next time-lock ID counter (u64).
    NextLockId,
    /// List of lock IDs belonging to a creator.
    CreatorLocks(Address),
    /// Active time-lock IDs for expiration processing.
    ActiveTimeLocks,
    /// Active delegations keyed by (creator, delegate).
    Delegation(Address, Address),
    /// List of active delegate addresses for a creator.
    Delegates(Address),
    /// Historical delegation snapshots for a creator.
    DelegationHistory(Address),
    /// Vesting schedule record keyed by schedule ID.
    VestingSchedule(u64),
    /// Vesting schedules for a creator keyed by (creator, schedule_id).
    CreatorVestingSchedules(Address, u64),
    /// List of vesting schedule IDs for a creator.
    CreatorVestingList(Address),
    /// Global vesting schedule counter.
    VestingScheduleCounter,
    /// Time-lock record keyed by lock ID.
    TimeLock(u64),
    /// Multi-sig withdrawal request keyed by request ID.
    MultiSigRequest(u64),
    /// Global counter for multi-sig request IDs.
    MultiSigCounter,
    /// Multi-sig configuration (threshold, signers, required approvals).
    MultiSigConfig,
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
    /// Fee exceeds the maximum allowed (500 bps).
    FeeExceedsMaximum = 32,
    /// Time-lock record not found.
    LockNotFound = 33,
    /// Unlock time has not been reached yet.
    NotUnlocked = 34,
    /// Time-lock has already been cancelled.
    LockCancelled = 35,
    /// Cooldown period between withdrawals has not elapsed yet.
    WithdrawalCooldown = 36,
    /// Withdrawal would exceed the creator's daily limit.
    DailyLimitExceeded = 37,
    /// Multi-sig request not found.
    MultiSigRequestNotFound = 38,
    /// Multi-sig request has expired.
    MultiSigRequestExpired = 39,
    /// Multi-sig request has already been executed or cancelled.
    MultiSigRequestClosed = 40,
    /// Approver is not in the authorised signer list.
    NotASigner = 41,
    /// Approver has already approved this request.
    AlreadyApproved = 42,
    /// Multi-sig config has not been set.
    MultiSigNotConfigured = 43,
    /// No delegation exists for this creator/delegate pair.
    DelegationNotFound = 44,
    /// Delegation has expired.
    DelegationExpired = 45,
    /// Delegation has been revoked or deactivated.
    DelegationInactive = 46,
    /// Requested delegate withdrawal exceeds allowed limit.
    DelegationLimitExceeded = 47,
    /// Delegation duration must be greater than zero.
    InvalidDuration = 48,
    /// Vesting schedule not found.
    VestingScheduleNotFound = 49,
    /// Vesting cliff period has not elapsed yet.
    VestingCliffNotReached = 50,
    /// No vested amount available to withdraw.
    NoVestedAmount = 51,
    /// Vesting schedule ID is invalid or zero.
    InvalidVestingId = 52,
    /// Vesting duration must be greater than zero.
    InvalidVestingDuration = 53,
    /// Cliff duration cannot exceed vesting duration.
    CliffExceedsVesting = 54,
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
            panic_with_error!(&env, TipJarError::AlreadyInitialized);
        }
        if fee_basis_points > 500 {
            panic_with_error!(&env, TipJarError::FeeExceedsMaximum);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
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

        let fee_bp: u32 = env.storage().instance().get(&DataKey::FeeBasisPoints).unwrap_or(0);
        let fee: i128 = (amount * fee_bp as i128) / 10_000;
        let creator_amount = amount.checked_sub(fee).unwrap_or(0);

        // ── state updates before external call (CEI pattern) ─────────────────
        if fee > 0 {
            let fee_key = DataKey::PlatformFeeBalance(token.clone());
            let new_fee_bal: i128 = env
                .storage()
                .instance()
                .get(&fee_key)
                .unwrap_or(0)
                .checked_add(fee)
                .expect("fee overflow");
            env.storage().instance().set(&fee_key, &new_fee_bal);
        }

        let bal_key = DataKey::CreatorBalance(creator.clone(), token.clone());
        let existing_bal: i128 = env.storage().persistent().get(&bal_key).unwrap_or(0);
        let new_bal: i128 = existing_bal.checked_add(creator_amount).expect("balance overflow");
        env.storage().persistent().set(&bal_key, &new_bal);

        let tot_key = DataKey::CreatorTotal(creator.clone(), token.clone());
        let existing_tot: i128 = env.storage().persistent().get(&tot_key).unwrap_or(0);
        let new_tot: i128 = existing_tot.checked_add(creator_amount).expect("total overflow");
        env.storage().persistent().set(&tot_key, &new_tot);

        let tip_id: u64 = env.storage().instance().get(&DataKey::TipCounter).unwrap_or(0);
        env.storage().instance().set(&DataKey::TipCounter, &(tip_id + 1));

        Self::update_leaderboard_stats(&env, &sender, &creator, creator_amount);

        // ── external call last ───────────────────────────────────────────────
        token::Client::new(&env, &token).transfer(&sender, &env.current_contract_address(), &amount);

        env.events().publish((symbol_short!("tip"), creator.clone()), (sender, creator_amount));
        tip_id
    }

    /// Withdraws the full escrowed balance for `creator` in `token`.
    ///
    /// Enforces per-creator (or default) daily limits and cooldown periods.
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
        Self::check_and_update_withdrawal_limits(&env, &creator, amount);
        env.storage().persistent().set(&bal_key, &0i128);
        token::Client::new(&env, &token).transfer(&env.current_contract_address(), &creator, &amount);
        events::emit_withdraw_event(&env, &creator, amount, &token);
    }

    /// Authorizes a delegate to withdraw on behalf of `creator`.
    ///
    /// `max_amount` is the lifetime cap and `duration` is seconds until expiry.
    /// Emits `("delegate", creator)` with data `(delegate, max_amount, expires_at)`.
    pub fn delegate_withdrawal(
        env: Env,
        creator: Address,
        delegate: Address,
        max_amount: i128,
        duration: u64,
    ) {
        Self::require_not_paused(&env);
        creator.require_auth();
        if max_amount <= 0 {
            panic_with_error!(&env, TipJarError::InvalidAmount);
        }
        if duration == 0 {
            panic_with_error!(&env, TipJarError::InvalidDuration);
        }

        let now = env.ledger().timestamp();
        let expires_at = now.saturating_add(duration);
        let delegation = Delegation {
            creator: creator.clone(),
            delegate: delegate.clone(),
            max_amount,
            used_amount: 0,
            expires_at,
            active: true,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Delegation(creator.clone(), delegate.clone()), &delegation);
        Self::add_delegate(&env, &creator, &delegate);
        Self::append_delegation_history(&env, &creator, &delegation);

        env.events().publish(
            (symbol_short!("delegate"),),
            (creator, delegate, max_amount, expires_at),
        );
    }

    /// Withdraws `amount` from `creator` balance to `delegate` when authorized.
    ///
    /// Enforces the creator's withdrawal limits and the delegation cap.
    /// Emits `("delegate_withdraw", creator)` with data `(delegate, amount, token)`.
    pub fn withdraw_as_delegate(
        env: Env,
        delegate: Address,
        creator: Address,
        token: Address,
        amount: i128,
    ) {
        Self::require_not_paused(&env);
        delegate.require_auth();
        if amount <= 0 {
            panic_with_error!(&env, TipJarError::InvalidAmount);
        }

        let mut delegation: Delegation = env
            .storage()
            .persistent()
            .get(&DataKey::Delegation(creator.clone(), delegate.clone()))
            .unwrap_or_else(|| panic_with_error!(&env, TipJarError::DelegationNotFound));

        if !delegation.active {
            panic_with_error!(&env, TipJarError::DelegationInactive);
        }
        let now = env.ledger().timestamp();
        if now > delegation.expires_at {
            delegation.active = false;
            env.storage()
                .persistent()
                .set(&DataKey::Delegation(creator.clone(), delegate.clone()), &delegation);
            Self::remove_delegate(&env, &creator, &delegate);
            Self::append_delegation_history(&env, &creator, &delegation);
            panic_with_error!(&env, TipJarError::DelegationExpired);
        }
        if delegation.used_amount.checked_add(amount).unwrap_or(i128::MAX) > delegation.max_amount {
            panic_with_error!(&env, TipJarError::DelegationLimitExceeded);
        }

        let bal_key = DataKey::CreatorBalance(creator.clone(), token.clone());
        let balance: i128 = env.storage().persistent().get(&bal_key).unwrap_or(0);
        if amount > balance || balance == 0 {
            panic_with_error!(&env, TipJarError::NothingToWithdraw);
        }

        Self::check_and_update_withdrawal_limits(&env, &creator, amount);

        env.storage().persistent().set(&bal_key, &(balance - amount));
        delegation.used_amount += amount;
        if delegation.used_amount >= delegation.max_amount {
            delegation.active = false;
            Self::remove_delegate(&env, &creator, &delegate);
        }
        env.storage()
            .persistent()
            .set(&DataKey::Delegation(creator.clone(), delegate.clone()), &delegation);
        Self::append_delegation_history(&env, &creator, &delegation);

        token::Client::new(&env, &token).transfer(
            &env.current_contract_address(),
            &delegate,
            &amount,
        );

        env.events().publish(
            (symbol_short!("del_wdr"),),
            (creator, delegate, amount, token),
        );
    }

    /// Revokes an active delegation. Only the creator may revoke.
    /// Emits `("delegate_revoked", creator)` with data `(delegate,)`.
    pub fn revoke_delegation(env: Env, creator: Address, delegate: Address) {
        Self::require_not_paused(&env);
        creator.require_auth();

        let mut delegation: Delegation = env
            .storage()
            .persistent()
            .get(&DataKey::Delegation(creator.clone(), delegate.clone()))
            .unwrap_or_else(|| panic_with_error!(&env, TipJarError::DelegationNotFound));

        if !delegation.active {
            panic_with_error!(&env, TipJarError::DelegationInactive);
        }

        delegation.active = false;
        env.storage()
            .persistent()
            .set(&DataKey::Delegation(creator.clone(), delegate.clone()), &delegation);
        Self::remove_delegate(&env, &creator, &delegate);
        Self::append_delegation_history(&env, &creator, &delegation);

        env.events().publish(
            (symbol_short!("del_rev"),),
            (creator, delegate),
        );
    }

    /// Returns the active delegation record for `creator` and `delegate`.
    pub fn get_delegation(
        env: Env,
        creator: Address,
        delegate: Address,
    ) -> Option<Delegation> {
        env.storage()
            .persistent()
            .get(&DataKey::Delegation(creator, delegate))
    }

    /// Returns the active delegate addresses for `creator`.
    pub fn get_delegates(env: Env, creator: Address) -> Vec<Address> {
        env.storage()
            .persistent()
            .get(&DataKey::Delegates(creator))
            .unwrap_or_else(|| Vec::new(&env))
    }

    /// Returns the historical delegation snapshots for `creator`.
    pub fn get_delegation_history(env: Env, creator: Address) -> Vec<Delegation> {
        env.storage()
            .persistent()
            .get(&DataKey::DelegationHistory(creator))
            .unwrap_or_else(|| Vec::new(&env))
    }

    fn add_delegate(env: &Env, creator: &Address, delegate: &Address) {
        let mut delegates: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::Delegates(creator.clone()))
            .unwrap_or_else(|| Vec::new(env));
        if !delegates.contains(delegate) {
            delegates.push_back(delegate.clone());
            env.storage().persistent().set(&DataKey::Delegates(creator.clone()), &delegates);
        }
    }

    fn remove_delegate(env: &Env, creator: &Address, delegate: &Address) {
        let delegates: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::Delegates(creator.clone()))
            .unwrap_or_else(|| Vec::new(env));
        let mut remaining = Vec::new(env);
        for d in delegates.iter() {
            if d != delegate {
                remaining.push_back(d);
            }
        }
        env.storage()
            .persistent()
            .set(&DataKey::Delegates(creator.clone()), &remaining);
    }

    fn append_delegation_history(env: &Env, creator: &Address, delegation: &Delegation) {
        let mut history: Vec<Delegation> = env
            .storage()
            .persistent()
            .get(&DataKey::DelegationHistory(creator.clone()))
            .unwrap_or_else(|| Vec::new(env));
        history.push_back(delegation.clone());
        env.storage()
            .persistent()
            .set(&DataKey::DelegationHistory(creator.clone()), &history);
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

    // ── vesting schedules ────────────────────────────────────────────────────

    /// Creates a new vesting schedule for a tip.
    ///
    /// Parameters:
    /// - `tipper`: The address that sent the tip
    /// - `creator`: The address that will receive vested amounts
    /// - `token`: The token being vested
    /// - `amount`: Total amount to vest
    /// - `cliff_duration`: Seconds until vesting begins
    /// - `vesting_duration`: Total vesting period from start_time
    ///
    /// Emits: `("vest_new",)` with data `(creator, tipper, amount, start_time, vesting_duration, cliff_duration)`.
    pub fn create_vesting_schedule(
        env: Env,
        tipper: Address,
        creator: Address,
        token: Address,
        amount: i128,
        cliff_duration: u64,
        vesting_duration: u64,
    ) -> u64 {
        Self::require_not_paused(&env);
        tipper.require_auth();

        if amount <= 0 {
            panic_with_error!(&env, TipJarError::InvalidAmount);
        }
        if vesting_duration == 0 {
            panic_with_error!(&env, TipJarError::InvalidVestingDuration);
        }
        if cliff_duration > vesting_duration {
            panic_with_error!(&env, TipJarError::CliffExceedsVesting);
        }

        let whitelisted: bool = env
            .storage()
            .instance()
            .get(&DataKey::TokenWhitelist(token.clone()))
            .unwrap_or(false);
        if !whitelisted {
            panic_with_error!(&env, TipJarError::TokenNotWhitelisted);
        }

        let now = env.ledger().timestamp();
        let schedule_id: u64 = env.storage().instance().get(&DataKey::VestingScheduleCounter).unwrap_or(0);

        let schedule = VestingSchedule {
            id: schedule_id,
            creator: creator.clone(),
            tipper: tipper.clone(),
            token: token.clone(),
            total_amount: amount,
            start_time: now,
            cliff_duration,
            vesting_duration,
            withdrawn: 0,
            created_at: now,
        };

        env.storage()
            .persistent()
            .set(&DataKey::VestingSchedule(schedule_id), &schedule);
        
        let mut schedules: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::CreatorVestingList(creator.clone()))
            .unwrap_or_else(|| Vec::new(&env));
        schedules.push_back(schedule_id);
        env.storage()
            .persistent()
            .set(&DataKey::CreatorVestingList(creator.clone()), &schedules);

        env.storage()
            .instance()
            .set(&DataKey::VestingScheduleCounter, &(schedule_id + 1));

        // Transfer tokens into contract for vesting
        token::Client::new(&env, &token).transfer(&tipper, &env.current_contract_address(), &amount);

        env.events().publish(
            (symbol_short!("vest_new"),),
            (creator, tipper, amount, now, vesting_duration, cliff_duration),
        );

        schedule_id
    }

    /// Calculates the vested amount for a schedule at the current ledger time.
    fn calculate_vested_amount(env: &Env, schedule: &VestingSchedule) -> i128 {
        let current_time = env.ledger().timestamp();

        // No vesting until cliff is reached
        if current_time < schedule.start_time + schedule.cliff_duration {
            return 0;
        }

        let elapsed = current_time - schedule.start_time;

        // Full vesting after vesting_duration has passed
        if elapsed >= schedule.vesting_duration {
            return schedule.total_amount;
        }

        // Linear vesting between cliff and end
        (schedule.total_amount * elapsed as i128) / schedule.vesting_duration as i128
    }

    /// Returns the currently vested amount for a schedule.
    pub fn get_vested_amount(env: Env, schedule_id: u64) -> i128 {
        if schedule_id == 0 {
            panic_with_error!(&env, TipJarError::InvalidVestingId);
        }

        let schedule: VestingSchedule = env
            .storage()
            .persistent()
            .get(&DataKey::VestingSchedule(schedule_id))
            .unwrap_or_else(|| panic_with_error!(&env, TipJarError::VestingScheduleNotFound));

        Self::calculate_vested_amount(&env, &schedule)
    }

    /// Returns the available vested amount that can be withdrawn (vested - already withdrawn).
    pub fn get_available_vested_amount(env: Env, schedule_id: u64) -> i128 {
        if schedule_id == 0 {
            panic_with_error!(&env, TipJarError::InvalidVestingId);
        }

        let schedule: VestingSchedule = env
            .storage()
            .persistent()
            .get(&DataKey::VestingSchedule(schedule_id))
            .unwrap_or_else(|| panic_with_error!(&env, TipJarError::VestingScheduleNotFound));

        let vested = Self::calculate_vested_amount(&env, &schedule);
        vested.saturating_sub(schedule.withdrawn)
    }

    /// Withdraws available vested amounts from a vesting schedule to the creator.
    ///
    /// Emits: `("vest_withdraw",)` with data `(creator, schedule_id, amount, token)`.
    pub fn withdraw_vested(
        env: Env,
        creator: Address,
        schedule_id: u64,
    ) -> i128 {
        Self::require_not_paused(&env);
        creator.require_auth();

        if schedule_id == 0 {
            panic_with_error!(&env, TipJarError::InvalidVestingId);
        }

        let mut schedule: VestingSchedule = env
            .storage()
            .persistent()
            .get(&DataKey::VestingSchedule(schedule_id))
            .unwrap_or_else(|| panic_with_error!(&env, TipJarError::VestingScheduleNotFound));

        if schedule.creator != creator {
            panic_with_error!(&env, TipJarError::Unauthorized);
        }

        let vested = Self::calculate_vested_amount(&env, &schedule);
        let available = vested.saturating_sub(schedule.withdrawn);

        if available <= 0 {
            panic_with_error!(&env, TipJarError::NoVestedAmount);
        }

        schedule.withdrawn = schedule.withdrawn.checked_add(available).expect("withdrawn overflow");
        env.storage()
            .persistent()
            .set(&DataKey::VestingSchedule(schedule_id), &schedule);

        token::Client::new(&env, &schedule.token).transfer(
            &env.current_contract_address(),
            &creator,
            &available,
        );

        env.events().publish(
            (symbol_short!("vest_wdr"),),
            (creator, schedule_id, available, schedule.token),
        );

        available
    }

    /// Returns the vesting schedule details.
    pub fn get_vesting_schedule(env: Env, schedule_id: u64) -> Option<VestingSchedule> {
        if schedule_id == 0 {
            return None;
        }
        env.storage()
            .persistent()
            .get(&DataKey::VestingSchedule(schedule_id))
    }

    /// Returns all vesting schedule IDs for a creator.
    pub fn get_creator_vesting_schedules(env: Env, creator: Address) -> Vec<u64> {
        env.storage()
            .persistent()
            .get(&DataKey::CreatorVestingList(creator))
            .unwrap_or_else(|| Vec::new(&env))
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
        let new_bal: i128 = env.storage().persistent().get(&bal_key).unwrap_or(0)
            .checked_add(net).expect("balance overflow");
        env.storage().persistent().set(&bal_key, &new_bal);

        let tot_key = DataKey::CreatorTotal(creator.clone(), token.clone());
        let new_tot: i128 = env.storage().persistent().get(&tot_key).unwrap_or(0)
            .checked_add(net).expect("total overflow");
        env.storage().persistent().set(&tot_key, &new_tot);

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

    // ── RBAC ─────────────────────────────────────────────────────────────────

    /// Grants `role` to `user`. Caller must be the stored admin.
    pub fn grant_role(env: Env, admin: Address, user: Address, role: Role) {
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic_with_error!(&env, TipJarError::Unauthorized);
        }
        env.storage().persistent().set(&DataKey::UserRole(user.clone()), &role);
        let mut members: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::RoleMembers(role.clone()))
            .unwrap_or_else(|| Vec::new(&env));
        if !members.contains(&user) {
            members.push_back(user.clone());
            env.storage().persistent().set(&DataKey::RoleMembers(role.clone()), &members);
        }
        env.events().publish((symbol_short!("role_grt"),), (user, role));
    }

    /// Revokes any role from `user`. Caller must be the stored admin.
    pub fn revoke_role(env: Env, admin: Address, user: Address) {
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic_with_error!(&env, TipJarError::Unauthorized);
        }
        if let Some(role) = env
            .storage()
            .persistent()
            .get::<DataKey, Role>(&DataKey::UserRole(user.clone()))
        {
            env.storage().persistent().remove(&DataKey::UserRole(user.clone()));
            let mut members: Vec<Address> = env
                .storage()
                .persistent()
                .get(&DataKey::RoleMembers(role.clone()))
                .unwrap_or_else(|| Vec::new(&env));
            members.retain(|a| a != user);
            env.storage().persistent().set(&DataKey::RoleMembers(role.clone()), &members);
            env.events().publish((symbol_short!("role_rev"),), (user, role));
        }
    }

    /// Returns `true` if `user` holds `role`.
    pub fn has_role(env: Env, user: Address, role: Role) -> bool {
        env.storage()
            .persistent()
            .get::<DataKey, Role>(&DataKey::UserRole(user))
            .map(|r| r == role)
            .unwrap_or(false)
    }

    /// Internal helper — panics with `Unauthorized` if `user` does not hold `role`.
    #[allow(dead_code)]
    fn require_role(env: &Env, user: &Address, role: Role) {
        let stored: Option<Role> = env
            .storage()
            .persistent()
            .get(&DataKey::UserRole(user.clone()));
        if stored != Some(role) {
            panic_with_error!(env, TipJarError::Unauthorized);
        }
    }

    /// Returns all addresses that currently hold `role`.
    pub fn get_role_members(env: Env, role: Role) -> Vec<Address> {
        env.storage()
            .persistent()
            .get(&DataKey::RoleMembers(role))
            .unwrap_or_else(|| Vec::new(&env))
    }

    // ── time-locked tips ──────────────────────────────────────────────────────

    /// Creates a time-locked tip for a specific `token`. Tokens are transferred
    /// immediately into escrow but can only be withdrawn by `creator` after `unlock_time`.
    ///
    /// Returns the lock ID.
    /// Emits `("lock", creator)` with data `(sender, amount, unlock_time, lock_id)`.
    pub fn tip_time_locked(
        env: Env,
        sender: Address,
        creator: Address,
        token: Address,
        amount: i128,
        unlock_time: u64,
    ) -> u64 {
        Self::require_not_paused(&env);
        sender.require_auth();
        if amount <= 0 {
            panic_with_error!(&env, TipJarError::InvalidAmount);
        }
        if unlock_time <= env.ledger().timestamp() {
            panic_with_error!(&env, TipJarError::InvalidUnlockTime);
        }

        // State updates before external call (CEI).
        let lock_id: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::NextLockId)
            .unwrap_or(0);
        let created_at = env.ledger().timestamp();
        let refund_window: u64 = env
            .storage()
            .instance()
            .get(&DataKey::RefundWindow)
            .unwrap_or(0);
        let time_lock = TimeLock {
            sender: sender.clone(),
            creator: creator.clone(),
            token: token.clone(),
            amount,
            unlock_time,
            created_at,
            expires_at: created_at.saturating_add(refund_window),
            cancelled: false,
        };
        env.storage().persistent().set(&DataKey::TimeLock(lock_id), &time_lock);
        env.storage().persistent().set(&DataKey::NextLockId, &(lock_id + 1));

        let mut creator_locks: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::CreatorLocks(creator.clone()))
            .unwrap_or_else(|| Vec::new(&env));
        creator_locks.push_back(lock_id);
        env.storage().persistent().set(&DataKey::CreatorLocks(creator.clone()), &creator_locks);

        Self::add_active_time_lock(&env, lock_id);

        // External call last.
        token::Client::new(&env, &token).transfer(&sender, &env.current_contract_address(), &amount);

        env.events().publish(
            (symbol_short!("lock"), creator),
            (sender, amount, unlock_time, lock_id),
        );
        lock_id
    }

    /// Convenience wrapper matching the public `tip_locked` API.
    pub fn tip_locked(
        env: Env,
        sender: Address,
        creator: Address,
        token: Address,
        amount: i128,
        unlock_time: u64,
    ) -> u64 {
        Self::tip_time_locked(env, sender, creator, token, amount, unlock_time)
    }

    /// Withdraws a time-locked tip after its unlock time. Only `creator` may call.
    ///
    /// Emits `("unlock", creator)` with data `(amount, lock_id)`.
    pub fn withdraw_time_locked(env: Env, creator: Address, token: Address, lock_id: u64) {
        Self::require_not_paused(&env);
        creator.require_auth();

        let time_lock: TimeLock = env
            .storage()
            .persistent()
            .get(&DataKey::TimeLock(lock_id))
            .unwrap_or_else(|| panic_with_error!(&env, TipJarError::LockNotFound));

        if time_lock.creator != creator {
            panic_with_error!(&env, TipJarError::Unauthorized);
        }
        if time_lock.cancelled {
            panic_with_error!(&env, TipJarError::LockCancelled);
        }
        if env.ledger().timestamp() < time_lock.unlock_time {
            panic_with_error!(&env, TipJarError::NotUnlocked);
        }

        // State update before external call (CEI).
        env.storage().persistent().remove(&DataKey::TimeLock(lock_id));
        Self::remove_active_time_lock(&env, lock_id);

        token::Client::new(&env, &token).transfer(
            &env.current_contract_address(),
            &creator,
            &time_lock.amount,
        );

        env.events().publish(
            (symbol_short!("unlock"), creator),
            (time_lock.amount, lock_id),
        );
    }

    /// Convenience wrapper matching the public `withdraw_locked` API.
    pub fn withdraw_locked(env: Env, creator: Address, token: Address, lock_id: u64) {
        Self::withdraw_time_locked(env, creator, token, lock_id)
    }

    /// Cancels a time-locked tip and refunds the sender. Only the original sender may call.
    ///
    /// Emits `("lk_cncl", sender)` with data `(amount, lock_id)`.
    pub fn cancel_time_lock(env: Env, sender: Address, token: Address, lock_id: u64) {
        Self::require_not_paused(&env);
        sender.require_auth();

        let mut time_lock: TimeLock = env
            .storage()
            .persistent()
            .get(&DataKey::TimeLock(lock_id))
            .unwrap_or_else(|| panic_with_error!(&env, TipJarError::LockNotFound));

        if time_lock.sender != sender {
            panic_with_error!(&env, TipJarError::Unauthorized);
        }
        if time_lock.cancelled {
            panic_with_error!(&env, TipJarError::LockCancelled);
        }

        // State update before external call (CEI).
        time_lock.cancelled = true;
        env.storage().persistent().set(&DataKey::TimeLock(lock_id), &time_lock);
        Self::remove_active_time_lock(&env, lock_id);

        token::Client::new(&env, &token).transfer(
            &env.current_contract_address(),
            &sender,
            &time_lock.amount,
        );

        env.events().publish(
            (symbol_short!("lk_cncl"), sender),
            (time_lock.amount, lock_id),
        );
    }

    /// Convenience wrapper matching the public `cancel_locked` API.
    pub fn cancel_locked(env: Env, sender: Address, token: Address, lock_id: u64) {
        Self::cancel_time_lock(env, sender, token, lock_id)
    }

    /// Returns all time-lock records for `creator`.
    pub fn get_time_locks(env: Env, creator: Address) -> Vec<TimeLock> {
        let lock_ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::CreatorLocks(creator))
            .unwrap_or_else(|| Vec::new(&env));
        let mut locks = Vec::new(&env);
        for lock_id in lock_ids.iter() {
            if let Some(lock) = env
                .storage()
                .persistent()
                .get::<DataKey, TimeLock>(&DataKey::TimeLock(lock_id))
            {
                locks.push_back(lock);
            }
        }
        locks
    }

    /// Returns a single active locked tip for `creator`.
    pub fn get_locked_tip(env: Env, creator: Address, lock_id: u64) -> LockedTip {
        let time_lock: TimeLock = env
            .storage()
            .persistent()
            .get(&DataKey::TimeLock(lock_id))
            .unwrap_or_else(|| panic_with_error!(&env, TipJarError::LockNotFound));

        if time_lock.creator != creator {
            panic_with_error!(&env, TipJarError::Unauthorized);
        }

        LockedTip {
            sender: time_lock.sender.clone(),
            creator: time_lock.creator.clone(),
            token: time_lock.token.clone(),
            amount: time_lock.amount,
            unlock_timestamp: time_lock.unlock_time,
        }
    }

    /// Returns the refund window used to compute tip expiry.
    pub fn get_refund_window(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::RefundWindow)
            .unwrap_or(0)
    }

    /// Updates the refund window used by time-locked tips.
    /// Admin only.
    pub fn set_refund_window(env: Env, admin: Address, refund_window_seconds: u64) {
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic_with_error!(&env, TipJarError::Unauthorized);
        }
        env.storage()
            .instance()
            .set(&DataKey::RefundWindow, &refund_window_seconds);
        env.events().publish((symbol_short!("refund_window"),), refund_window_seconds);
    }

    /// Returns all expired time-locked tips whose refund window has passed.
    fn get_expired_time_lock_ids(env: &Env, current_time: u64) -> Vec<u64> {
        let lock_ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::ActiveTimeLocks)
            .unwrap_or_else(|| Vec::new(env));
        let mut expired = Vec::new(env);
        for lock_id in lock_ids.iter() {
            if let Some(lock) = env
                .storage()
                .persistent()
                .get::<DataKey, TimeLock>(&DataKey::TimeLock(lock_id))
            {
                if !lock.cancelled && lock.expires_at <= current_time {
                    expired.push_back(lock_id);
                }
            }
        }
        expired
    }

    /// Processes all expired time-locked tips and refunds their senders.
    /// Returns the number of refunded tips.
    pub fn get_expired_time_locks(env: Env) -> Vec<TipWithExpiry> {
        let current_time = env.ledger().timestamp();
        let expired_ids = Self::get_expired_time_lock_ids(&env, current_time);
        let mut result = Vec::new(&env);
        for lock_id in expired_ids.iter() {
            if let Some(lock) = env
                .storage()
                .persistent()
                .get::<DataKey, TimeLock>(&DataKey::TimeLock(lock_id))
            {
                result.push_back(TipWithExpiry {
                    tipper: lock.sender.clone(),
                    creator: lock.creator.clone(),
                    amount: lock.amount,
                    created_at: lock.created_at,
                    expires_at: lock.expires_at,
                    claimed: false,
                });
            }
        }
        result
    }

    pub fn process_expired_tips(env: Env) -> u32 {
        let current_time = env.ledger().timestamp();
        let expired_locks = Self::get_expired_time_lock_ids(&env, current_time);
        let mut refunded_count = 0u32;
        for lock_id in expired_locks.iter() {
            if let Some(time_lock) = env
                .storage()
                .persistent()
                .get::<DataKey, TimeLock>(&DataKey::TimeLock(lock_id))
            {
                if !time_lock.cancelled {
                    Self::refund_time_lock(&env, lock_id, &time_lock);
                    refunded_count += 1;
                }
            }
        }
        refunded_count
    }

    fn add_active_time_lock(env: &Env, lock_id: u64) {
        let mut active: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::ActiveTimeLocks)
            .unwrap_or_else(|| Vec::new(env));
        if !active.contains(&lock_id) {
            active.push_back(lock_id);
        }
        env.storage().persistent().set(&DataKey::ActiveTimeLocks, &active);
    }

    fn remove_active_time_lock(env: &Env, lock_id: u64) {
        let active: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::ActiveTimeLocks)
            .unwrap_or_else(|| Vec::new(env));
        let mut remaining = Vec::new(env);
        for id in active.iter() {
            if id != lock_id {
                remaining.push_back(id);
            }
        }
        env.storage().persistent().set(&DataKey::ActiveTimeLocks, &remaining);
    }

    fn refund_time_lock(env: &Env, lock_id: u64, time_lock: &TimeLock) {
        env.storage().persistent().remove(&DataKey::TimeLock(lock_id));
        Self::remove_active_time_lock(env, lock_id);
        token::Client::new(&env, &time_lock.token).transfer(
            &env.current_contract_address(),
            &time_lock.sender,
            &time_lock.amount,
        );
        env.events().publish(
            (symbol_short!("tip_expired"), time_lock.creator.clone()),
            (time_lock.sender.clone(), time_lock.amount, time_lock.expires_at, lock_id),
        );
    }

    // ── withdrawal limits ─────────────────────────────────────────────────────

    /// Checks cooldown and daily limit for `creator`, then updates state.
    ///
    /// Panics with `WithdrawalCooldown` or `DailyLimitExceeded` on violation.
    fn check_and_update_withdrawal_limits(env: &Env, creator: &Address, amount: i128) {
        const DAY_SECS: u64 = 86_400;

        // Resolve per-creator config, falling back to platform default.
        let mut limits: WithdrawalLimits = env
            .storage()
            .persistent()
            .get(&DataKey::WithdrawalLimits(creator.clone()))
            .or_else(|| env.storage().instance().get(&DataKey::DefaultWithdrawalLimits))
            .unwrap_or(WithdrawalLimits {
                daily_limit: 0,
                cooldown_seconds: 0,
                last_withdrawal: 0,
                withdrawn_today: 0,
                day_start: 0,
            });

        let now = env.ledger().timestamp();

        // Cooldown check.
        if limits.cooldown_seconds > 0 && limits.last_withdrawal > 0 {
            if now < limits.last_withdrawal + limits.cooldown_seconds {
                panic_with_error!(env, TipJarError::WithdrawalCooldown);
            }
        }

        // Daily window reset.
        if now >= limits.day_start + DAY_SECS {
            limits.withdrawn_today = 0;
            limits.day_start = now;
        }

        // Daily limit check (0 = unlimited).
        if limits.daily_limit > 0 {
            if limits.withdrawn_today + amount > limits.daily_limit {
                panic_with_error!(env, TipJarError::DailyLimitExceeded);
            }
        }

        limits.withdrawn_today += amount;
        limits.last_withdrawal = now;
        env.storage()
            .persistent()
            .set(&DataKey::WithdrawalLimits(creator.clone()), &limits);
    }

    /// Sets per-creator withdrawal limits. Admin only.
    ///
    /// Pass `daily_limit = 0` for unlimited; `cooldown_seconds = 0` for no cooldown.
    /// Emits `("wl_set", creator)` with data `(daily_limit, cooldown_seconds)`.
    pub fn set_withdrawal_limits(
        env: Env,
        admin: Address,
        creator: Address,
        daily_limit: i128,
        cooldown_seconds: u64,
    ) {
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic_with_error!(&env, TipJarError::Unauthorized);
        }

        let existing: WithdrawalLimits = env
            .storage()
            .persistent()
            .get(&DataKey::WithdrawalLimits(creator.clone()))
            .unwrap_or(WithdrawalLimits {
                daily_limit: 0,
                cooldown_seconds: 0,
                last_withdrawal: 0,
                withdrawn_today: 0,
                day_start: 0,
            });

        let limits = WithdrawalLimits {
            daily_limit,
            cooldown_seconds,
            last_withdrawal: existing.last_withdrawal,
            withdrawn_today: existing.withdrawn_today,
            day_start: existing.day_start,
        };
        env.storage()
            .persistent()
            .set(&DataKey::WithdrawalLimits(creator.clone()), &limits);

        env.events().publish(
            (symbol_short!("wl_set"), creator),
            (daily_limit, cooldown_seconds),
        );
    }

    /// Sets platform-wide default withdrawal limits applied to creators without
    /// a per-creator config. Admin only.
    ///
    /// Emits `("wl_def",)` with data `(daily_limit, cooldown_seconds)`.
    pub fn set_default_withdrawal_limits(
        env: Env,
        admin: Address,
        daily_limit: i128,
        cooldown_seconds: u64,
    ) {
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic_with_error!(&env, TipJarError::Unauthorized);
        }

        let defaults = WithdrawalLimits {
            daily_limit,
            cooldown_seconds,
            last_withdrawal: 0,
            withdrawn_today: 0,
            day_start: 0,
        };
        env.storage()
            .instance()
            .set(&DataKey::DefaultWithdrawalLimits, &defaults);

        env.events()
            .publish((symbol_short!("wl_def"),), (daily_limit, cooldown_seconds));
    }

    /// Emergency withdrawal that bypasses limits. Admin only.
    ///
    /// Transfers the full escrowed balance for `creator` in `token` directly,
    /// skipping cooldown and daily-limit checks.
    /// Emits `("wl_emrg", creator)` with data `amount`.
    pub fn emergency_withdraw(env: Env, admin: Address, creator: Address, token: Address) {
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic_with_error!(&env, TipJarError::Unauthorized);
        }

        let bal_key = DataKey::CreatorBalance(creator.clone(), token.clone());
        let amount: i128 = env.storage().persistent().get(&bal_key).unwrap_or(0);
        if amount == 0 {
            panic_with_error!(&env, TipJarError::NothingToWithdraw);
        }

        env.storage().persistent().set(&bal_key, &0i128);
        token::Client::new(&env, &token).transfer(
            &env.current_contract_address(),
            &creator,
            &amount,
        );

        env.events()
            .publish((symbol_short!("wl_emrg"), creator), amount);
    }

    /// Returns the withdrawal limits for `creator`, or the platform defaults if
    /// no per-creator config exists.
    pub fn get_withdrawal_limits(env: Env, creator: Address) -> WithdrawalLimits {
        env.storage()
            .persistent()
            .get(&DataKey::WithdrawalLimits(creator))
            .or_else(|| env.storage().instance().get(&DataKey::DefaultWithdrawalLimits))
            .unwrap_or(WithdrawalLimits {
                daily_limit: 0,
                cooldown_seconds: 0,
                last_withdrawal: 0,
                withdrawn_today: 0,
                day_start: 0,
            })
    }

    // ── multi-signature withdrawals ───────────────────────────────────────────

    /// Sets the multi-sig configuration. Admin only.
    ///
    /// `threshold` — amounts strictly above this require multi-sig (0 = all withdrawals).
    /// Emits `("ms_cfg",)` with data `(threshold, required_approvals, expiry_seconds)`.
    pub fn set_multisig_config(
        env: Env,
        admin: Address,
        threshold: i128,
        required_approvals: u32,
        expiry_seconds: u64,
        signers: Vec<Address>,
    ) {
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic_with_error!(&env, TipJarError::Unauthorized);
        }
        if required_approvals == 0 || required_approvals as u32 > signers.len() {
            panic_with_error!(&env, TipJarError::Unauthorized);
        }
        let cfg = MultiSigConfig { threshold, required_approvals, expiry_seconds, signers };
        env.storage().instance().set(&DataKey::MultiSigConfig, &cfg);
        env.events().publish(
            (symbol_short!("ms_cfg"),),
            (threshold, required_approvals, expiry_seconds),
        );
    }

    /// Creates a multi-sig withdrawal request for `amount` of `token`.
    ///
    /// If `amount` is at or below the configured threshold the withdrawal is
    /// processed immediately (no multi-sig needed) and returns `0`.
    /// Otherwise a pending request is created and its ID is returned.
    ///
    /// Emits `("ms_req", creator)` with data `(request_id, amount, expires_at)`.
    pub fn request_multisig_withdrawal(
        env: Env,
        creator: Address,
        token: Address,
        amount: i128,
    ) -> u64 {
        Self::require_not_paused(&env);
        creator.require_auth();

        let bal_key = DataKey::CreatorBalance(creator.clone(), token.clone());
        let balance: i128 = env.storage().persistent().get(&bal_key).unwrap_or(0);
        if balance == 0 || amount <= 0 || amount > balance {
            panic_with_error!(&env, TipJarError::NothingToWithdraw);
        }

        let cfg: MultiSigConfig = env
            .storage()
            .instance()
            .get(&DataKey::MultiSigConfig)
            .unwrap_or_else(|| panic_with_error!(&env, TipJarError::MultiSigNotConfigured));

        // Below-or-at threshold: process immediately.
        if cfg.threshold > 0 && amount <= cfg.threshold {
            Self::check_and_update_withdrawal_limits(&env, &creator, amount);
            env.storage().persistent().set(&bal_key, &(balance - amount));
            token::Client::new(&env, &token).transfer(
                &env.current_contract_address(),
                &creator,
                &amount,
            );
            events::emit_withdraw_event(&env, &creator, amount, &token);
            return 0;
        }

        // Above threshold: create pending request.
        let request_id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::MultiSigCounter)
            .unwrap_or(0);
        env.storage().instance().set(&DataKey::MultiSigCounter, &(request_id + 1));

        let expires_at = env.ledger().timestamp() + cfg.expiry_seconds;
        let request = MultiSigWithdrawal {
            request_id,
            creator: creator.clone(),
            token,
            amount,
            approvals: Vec::new(&env),
            required_approvals: cfg.required_approvals,
            expires_at,
            executed: false,
            cancelled: false,
        };
        env.storage().persistent().set(&DataKey::MultiSigRequest(request_id), &request);

        env.events().publish(
            (symbol_short!("ms_req"), creator),
            (request_id, amount, expires_at),
        );
        request_id
    }

    /// Approves a pending multi-sig withdrawal request.
    ///
    /// Once `required_approvals` is reached the withdrawal is executed automatically.
    /// Emits `("ms_apr", approver)` with data `request_id`.
    /// Emits `("ms_exe", creator)` with data `(request_id, amount)` on execution.
    pub fn approve_withdrawal(env: Env, approver: Address, request_id: u64) {
        Self::require_not_paused(&env);
        approver.require_auth();

        let cfg: MultiSigConfig = env
            .storage()
            .instance()
            .get(&DataKey::MultiSigConfig)
            .unwrap_or_else(|| panic_with_error!(&env, TipJarError::MultiSigNotConfigured));

        if !cfg.signers.contains(&approver) {
            panic_with_error!(&env, TipJarError::NotASigner);
        }

        let mut request: MultiSigWithdrawal = env
            .storage()
            .persistent()
            .get(&DataKey::MultiSigRequest(request_id))
            .unwrap_or_else(|| panic_with_error!(&env, TipJarError::MultiSigRequestNotFound));

        if request.executed || request.cancelled {
            panic_with_error!(&env, TipJarError::MultiSigRequestClosed);
        }
        if env.ledger().timestamp() > request.expires_at {
            panic_with_error!(&env, TipJarError::MultiSigRequestExpired);
        }
        if request.approvals.contains(&approver) {
            panic_with_error!(&env, TipJarError::AlreadyApproved);
        }

        request.approvals.push_back(approver.clone());
        env.events().publish((symbol_short!("ms_apr"), approver), request_id);

        if request.approvals.len() >= request.required_approvals {
            // Execute withdrawal.
            let bal_key = DataKey::CreatorBalance(request.creator.clone(), request.token.clone());
            let balance: i128 = env.storage().persistent().get(&bal_key).unwrap_or(0);
            if balance < request.amount {
                panic_with_error!(&env, TipJarError::InsufficientBalance);
            }
            env.storage().persistent().set(&bal_key, &(balance - request.amount));
            request.executed = true;
            env.storage().persistent().set(&DataKey::MultiSigRequest(request_id), &request);

            token::Client::new(&env, &request.token).transfer(
                &env.current_contract_address(),
                &request.creator,
                &request.amount,
            );
            env.events().publish(
                (symbol_short!("ms_exe"), request.creator.clone()),
                (request_id, request.amount),
            );
        } else {
            env.storage().persistent().set(&DataKey::MultiSigRequest(request_id), &request);
        }
    }

    /// Cancels a pending multi-sig withdrawal request.
    ///
    /// Only the original creator or admin may cancel.
    /// Emits `("ms_cncl", creator)` with data `request_id`.
    pub fn cancel_multisig_withdrawal(env: Env, caller: Address, request_id: u64) {
        caller.require_auth();

        let mut request: MultiSigWithdrawal = env
            .storage()
            .persistent()
            .get(&DataKey::MultiSigRequest(request_id))
            .unwrap_or_else(|| panic_with_error!(&env, TipJarError::MultiSigRequestNotFound));

        if request.executed || request.cancelled {
            panic_with_error!(&env, TipJarError::MultiSigRequestClosed);
        }

        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if caller != request.creator && caller != stored_admin {
            panic_with_error!(&env, TipJarError::Unauthorized);
        }

        request.cancelled = true;
        env.storage().persistent().set(&DataKey::MultiSigRequest(request_id), &request);
        env.events().publish((symbol_short!("ms_cncl"), request.creator), request_id);
    }

    /// Returns a multi-sig withdrawal request by ID.
    pub fn get_multisig_request(env: Env, request_id: u64) -> MultiSigWithdrawal {
        env.storage()
            .persistent()
            .get(&DataKey::MultiSigRequest(request_id))
            .unwrap_or_else(|| panic_with_error!(&env, TipJarError::MultiSigRequestNotFound))
    }

    /// Returns the current multi-sig configuration.
    pub fn get_multisig_config(env: Env) -> MultiSigConfig {
        env.storage()
            .instance()
            .get(&DataKey::MultiSigConfig)
            .unwrap_or_else(|| panic_with_error!(&env, TipJarError::MultiSigNotConfigured))
    }

    // ── upgrade / migration ───────────────────────────────────────────────────

    /// Runs any data migration needed after an upgrade. Admin only.
    ///
    /// Match on the current version to apply version-specific migrations.
    pub fn migrate(env: Env, admin: Address) {
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic_with_error!(&env, TipJarError::Unauthorized);
        }
        let version = upgrade::get_version(&env);
        match version {
            // v1 → v2: no data migration required in this example.
            _ => {}
        }
    }
}