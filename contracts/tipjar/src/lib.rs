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
pub mod storage;

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

// Dispute resolution
pub mod dispute;

// Privacy features
pub mod privacy_tip;

// Options trading
pub mod options;

// Tip Index Funds
pub mod index_fund;

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

/// A single tip operation used in `batch_tip_v2`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TipOperation {
    pub creator: Address,
    pub token: Address,
    pub amount: i128,
}

/// A single withdrawal operation used in `batch_withdraw`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WithdrawOperation {
    pub token: Address,
    pub amount: i128,
}

/// Result for a single operation within a batch call.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatchResult {
    /// Whether this individual operation succeeded.
    pub success: bool,
    /// Zero-based index of this operation in the input vector.
    pub index: u32,
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

/// Insurance pool configuration.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InsurancePoolConfig {
    pub min_contribution: i128,
    pub max_contribution: i128,
    pub premium_rate_bps: u32,
    pub payout_ratio_bps: u32,
    pub claim_cooldown: u64,
    pub admin_fee_bps: u32,
    pub tip_premium_bps: u32,
}

/// Current state of the insurance pool for a specific token.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InsurancePool {
    pub token: Address,
    pub total_reserves: i128,
    pub total_contributions: i128,
    pub total_claims_paid: i128,
    pub active_claims: u32,
    pub total_claims: u32,
    pub last_payout_time: u64,
}

/// An insurance claim submitted by a creator for a failed transaction.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InsuranceClaim {
    pub claim_id: u64,
    pub creator: Address,
    pub token: Address,
    pub amount: i128,
    pub tx_hash: BytesN<32>,
    pub status: ClaimStatus,
    pub created_at: u64,
    pub updated_at: u64,
    pub last_claim_at: u64,
}

/// Status of an insurance claim.
#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ClaimStatus {
    Pending,
    Approved,
    Rejected,
    Paid,
}

/// Premium information for a creator's contribution.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PremiumInfo {
    pub creator: Address,
    pub token: Address,
    pub total_contributed: i128,
    pub coverage_amount: i128,
    pub last_claim_at: u64,
    pub active_claims: u32,
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

/// Subscription tier level.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SubscriptionTier {
    Bronze,
    Silver,
    Gold,
}

/// Configuration for a subscription tier set by admin.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TierConfig {
    /// Price per payment interval in token units.
    pub price: i128,
    /// Human-readable description of benefits for this tier.
    pub benefits: String,
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
    /// The tier this subscription is on.
    pub tier: SubscriptionTier,
    /// Pending tier change to apply at next payment cycle (None = no pending change).
    pub pending_tier: Option<SubscriptionTier>,
}

/// Stream status.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StreamStatus {
    Active,
    Paused,
    Cancelled,
    Completed,
}

/// A continuous tip stream where funds flow in real-time based on time elapsed.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Stream {
    pub stream_id: u64,
    pub sender: Address,
    pub creator: Address,
    pub token: Address,
    pub amount_per_second: i128,
    pub start_time: u64,
    pub end_time: u64,
    pub withdrawn: i128,
    pub status: StreamStatus,
    pub created_at: u64,
    pub updated_at: u64,
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

/// An auction for exclusive creator content or experiences.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Auction {
    pub auction_id: u64,
    pub creator: Address,
    pub token: Address,
    pub reserve_price: i128,
    pub highest_bid: i128,
    pub highest_bidder: Option<Address>,
    pub ends_at: u64,
    pub created_at: u64,
    pub settled: bool,
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
    /// List of token addresses a creator has ever received tips in.
    CreatorTokens(Address),
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
    CrMatchProgs(Address),
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
    /// Optional timestamp (unix seconds) after which the contract auto-unpauses.
    PauseUntil,
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
    DefWithdrawLimits,
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
    VestingSchedCtr,
    /// Stream record keyed by stream ID.
    Stream(u64),
    /// List of stream IDs for a creator.
    CreatorStreams(Address),
    /// List of stream IDs for a sender.
    SenderStreams(Address),
    /// Global stream counter.
    StreamCounter,
    /// Global auction counter.
    AuctionCounter,
    /// Auction record keyed by auction ID.
    Auction(u64),
    /// Creator auction IDs list.
    CreatorAuctions(Address),
    /// Time-lock record keyed by lock ID.
    TimeLock(u64),
    /// Multi-sig withdrawal request keyed by request ID.
    MultiSigRequest(u64),
    /// Global counter for multi-sig request IDs.
    MultiSigCounter,
    /// Multi-sig configuration (threshold, signers, required approvals).
    MultiSigConfig,
    /// Dispute record keyed by dispute_id.
    Dispute(u64),
    /// Global counter for dispute IDs.
    DisputeCounter,
    /// List of dispute IDs for a creator.
    CreatorDisputes(Address),
    /// Evidence records keyed by (dispute_id, evidence_index).
    DisputeEvidence(u64, u64),
    /// Evidence counter for a dispute.
    DisputeEvidenceCounter(u64),
    /// Private tip record keyed by tip_id.
    PrivateTip(u64),
    /// Global counter for private tip IDs.
    PrivateTipCounter,
    /// Revealed amount for a private tip keyed by tip_id.
    PrivateTipAmount(u64),
    /// Insurance pool configuration.
    InsPoolCfg,
    /// Insurance pool state per token.
    InsPoolToken(Address),
    /// Insurance claim record keyed by claim ID.
    InsClaim(u64),
    /// Global counter for insurance claim IDs.
    InsClaimCtr,
    /// Creator's insurance contribution per token.
    InsContrib(Address, Address),
    /// Creator's last claim timestamp per token.
    InsLastClm(Address, Address),
    /// Creator's active claim count per token.
    InsActiveClms(Address, Address),
    /// Total number of claims for a creator per token.
    InsTotalClms(Address, Address),
    /// Insurance feature enabled flag.
    InsEnabled,
    /// Max active claims per creator.
    InsMaxClms,
    /// Insurance admin address.
    InsAdmin,
    /// List of claim IDs for a creator per token.
    InsClms(Address, Address),
    /// Option contract by ID.
    Option(u64),
    /// Option counter for ID generation.
    OptionCounter,
    /// Options written by address.
    WrittenOptions(Address),
    /// Options held by address.
    HeldOptions(Address),
    /// Option position tracking for address.
    OptionPosition(Address),
    /// Option pricing parameters.
    OptionPricingParams,
    /// Active options list.
    ActiveOptions,
    /// Collateral locked per token per address for options.
    OptionCollateral(Address, Address),
    /// Bridge relayer address.
    BridgeRelayer,
    /// Bridge token address.
    BridgeToken,
    /// Bridge enabled flag.
    BridgeEnabled,
    /// Bridge fee in basis points.
    BridgeFeeBps,
    /// Index fund record by ID.
    IndexFund(u64),
    /// Global index fund counter.
    IndexFundCounter,
    /// User share position in a fund keyed by (fund_id, holder).
    IndexFundShare(u64, Address),
    /// Creator allocation within a fund keyed by (fund_id, creator).
    IndexCreatorAlloc(u64, Address),
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
    MatchProgNotFound = 16,
    MatchProgInactive = 17,
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
    MsigReqNotFound = 38,
    /// Multi-sig request has expired.
    MultiSigReqExpired = 39,
    /// Multi-sig request has already been executed or cancelled.
    MultiSigReqClosed = 40,
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
    /// Dispute not found.
    DisputeNotFound = 49,
    /// Dispute is not in Open status.
    DisputeNotOpen = 50,
    /// Only initiator or arbitrator can perform this action.
    DisputeUnauthorized = 51,
    /// Insurance pool not configured.
    InsPoolNotCfg = 52,
    /// Contribution amount below minimum.
    ContributionTooLow = 53,
    /// Contribution amount exceeds maximum.
    ContributionTooHigh = 54,
    /// No insurance coverage for this creator/token.
    NoCoverage = 55,
    /// Claim has not been approved.
    ClaimNotApproved = 56,
    /// Claim already paid out.
    ClaimAlreadyPaid = 57,
    /// Insufficient reserves in pool.
    InsufficientReserves = 58,
    /// Claim cooldown period not elapsed.
    ClaimCooldownActive = 59,
    /// Too many active claims for this creator.
    TooManyActiveClaims = 60,
    /// Claim not found.
    ClaimNotFound = 61,
    /// Already contributed to pool this period.
    AlreadyContributed = 62,
    /// Insurance feature is disabled.
    InsuranceDisabled = 63,
    /// Previous claim must be resolved first.
    PendingClaimExists = 64,
    /// Payout would exceed pool reserves.
    PayoutExceedsReserves = 65,
    /// Invalid claim amount.
    InvalidClaimAmount = 66,
    /// Admin approval required for claim.
    AdmAppReq = 67,
    /// Private tip not found.
    PrivateTipNotFound = 68,
    /// Invalid reveal - hash mismatch.
    InvalidReveal = 69,
    /// Stream not found.
    StreamNotFound = 70,
    /// Stream has already been cancelled.
    StreamAlreadyCancelled = 71,
    /// Stream has not started yet.
    StreamNotStarted = 72,
    /// Stream has already completed.
    StreamAlreadyCompleted = 73,
    /// Invalid stream amount.
    InvalidStreamAmount = 74,
    /// Invalid stream rate (amount per second).
    InvalidStreamRate = 75,
    /// No streamed amount available to withdraw.
    NoStreamedAmount = 76,
    /// Stream rate exceeds maximum allowed (1000 tokens/second).
    StrmRateMax = 77,
    /// Auction not found.
    AuctionNotFound = 78,
    /// Auction has already been settled.
    AuctionAlreadySettled = 79,
    /// Auction is not yet ended.
    AuctionNotEnded = 80,
    /// Auction bid did not meet the opening reserve price.
    AuctionReserveNotMet = 81,
    /// Auction bid is too low.
    AuctionBidTooLow = 82,
    /// Auction creator is not authorized to settle.
    AuctionUnauthorized = 83,
    /// Auction duration must be greater than zero.
    InvalidAuctionDuration = 84,
    /// Auction reserve price must be non-negative.
    InvalidReservePrice = 85,
    /// Auction has already ended and can no longer accept bids.
    AuctionEnded = 86,
    /// Option not found.
    OptionNotFound = 87,
    /// Option is not active.
    OptionNotActive = 88,
    /// Option has expired.
    OptionExpired = 89,
    /// Option is out of the money.
    OptionOutOfMoney = 90,
    /// Not the option holder.
    NotOptionHolder = 91,
    /// Not the option writer.
    NotOptionWriter = 92,
    /// Option already has a holder.
    OptionAlreadySold = 93,
    /// Insufficient collateral for option.
    InsufficientCollateral = 94,
    /// Invalid option parameters.
    InvalidOptionParams = 95,
    /// Option has not expired yet.
    OptionNotExpired = 96,
    /// Invalid bridge fee.
    InvalidBridgeFee = 97,
    /// Index fund not found.
    IndexFundNotFound = 98,
    /// Index fund is not active.
    IndexFundNotActive = 99,
    /// Component weights do not sum to 10000 bps.
    InvalidIndexWeights = 100,
    /// Index fund requires at least 2 creators.
    IndexFundTooFewCreators = 101,
    /// Insufficient fund shares for withdrawal.
    InsufficientFundShares = 102,
    /// Deposit amount is below the minimum.
    IndexDepositTooSmall = 103,
    /// Attempted to unpause a contract that is not currently paused.
    NotPaused = 104,
}

#[contract]
pub struct TipJarContract;

#[contractimpl]
impl TipJarContract {
    // ── pause guard ──────────────────────────────────────────────────────────

    fn require_not_paused(env: &Env) {
        if Self::check_is_paused(env) {
            panic_with_error!(env, TipJarError::ContractPaused);
        }
    }

    /// Core pause check: returns true if the contract is currently paused.
    /// Handles auto-unpause by clearing pause state when `PauseUntil` has elapsed.
    fn check_is_paused(env: &Env) -> bool {
        let paused: bool = env
            .storage()
            .instance()
            .get::<DataKey, bool>(&DataKey::Paused)
            .unwrap_or(false);
        if !paused {
            return false;
        }
        // Check auto-unpause
        if let Some(unpause_time) = env
            .storage()
            .instance()
            .get::<DataKey, u64>(&DataKey::PauseUntil)
        {
            if env.ledger().timestamp() >= unpause_time {
                env.storage().instance().set(&DataKey::Paused, &false);
                env.storage().instance().remove(&DataKey::PauseReason);
                env.storage().instance().remove(&DataKey::PauseUntil);
                return false;
            }
        }
        true
    }

    fn add_creator_auction(env: &Env, creator: &Address, auction_id: u64) {
        let mut auctions: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::CreatorAuctions(creator.clone()))
            .unwrap_or_else(|| Vec::new(env));
        if !auctions.contains(&auction_id) {
            auctions.push_back(auction_id);
            env.storage()
                .persistent()
                .set(&DataKey::CreatorAuctions(creator.clone()), &auctions);
        }
    }

    fn get_auction_record(env: &Env, auction_id: u64) -> Option<Auction> {
        env.storage().persistent().get(&DataKey::Auction(auction_id))
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
    pub fn init(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic_with_error!(&env, TipJarError::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::FeeBasisPoints, &0u32);
        env.storage().instance().set(&DataKey::RefundWindow, &0u64);
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

    /// Removes a token from the whitelist. Admin only.
    pub fn remove_token(env: Env, admin: Address, token: Address) {
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic_with_error!(&env, TipJarError::Unauthorized);
        }
        env.storage().instance().set(&DataKey::TokenWhitelist(token), &false);
    }

    /// Returns `true` if `token` is on the whitelist.
    pub fn is_whitelisted(env: Env, token: Address) -> bool {
        env.storage()
            .instance()
            .get::<DataKey, bool>(&DataKey::TokenWhitelist(token))
            .unwrap_or(false)
    }

    /// Pauses all state-changing operations. Admin only.
    ///
    /// `reason` is stored on-chain for transparency.
    /// `duration_seconds` optionally sets an auto-unpause time; pass `None` for indefinite pause.
    /// Emits `("paused",)` with data `(admin, reason, unpause_time_or_zero)`.
    pub fn pause(env: Env, admin: Address, reason: String, duration_seconds: Option<u64>) {
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic_with_error!(&env, TipJarError::Unauthorized);
        }
        env.storage().instance().set(&DataKey::Paused, &true);
        env.storage().instance().set(&DataKey::PauseReason, &reason);
        let unpause_time: u64 = if let Some(duration) = duration_seconds {
            let t = env.ledger().timestamp().saturating_add(duration);
            env.storage().instance().set(&DataKey::PauseUntil, &t);
            t
        } else {
            env.storage().instance().remove(&DataKey::PauseUntil);
            0
        };
        env.events()
            .publish((symbol_short!("paused"),), (admin, reason, unpause_time));
    }

    /// Resumes normal operations. Admin only. Fails if contract is not paused.
    ///
    /// Emits `("unpaused",)` with data `admin`.
    pub fn unpause(env: Env, admin: Address) {
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic_with_error!(&env, TipJarError::Unauthorized);
        }
        if !Self::check_is_paused(&env) {
            panic_with_error!(&env, TipJarError::NotPaused);
        }
        env.storage().instance().set(&DataKey::Paused, &false);
        env.storage().instance().remove(&DataKey::PauseReason);
        env.storage().instance().remove(&DataKey::PauseUntil);
        env.events().publish((symbol_short!("unpaused"),), admin);
    }

    /// Returns `true` when the contract is currently paused (respects auto-unpause).
    pub fn is_paused(env: Env) -> bool {
        Self::check_is_paused(&env)
    }

    /// Returns the pause reason if the contract is paused, or `None` otherwise.
    pub fn get_pause_reason(env: Env) -> Option<String> {
        if !Self::check_is_paused(&env) {
            return None;
        }
        env.storage().instance().get(&DataKey::PauseReason)
    }

    /// Returns the auto-unpause timestamp if set, or `None` for indefinite pause.
    pub fn get_pause_until(env: Env) -> Option<u64> {
        env.storage().instance().get(&DataKey::PauseUntil)
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

        // --- Insurance Premium Calculation ---
        let ins_enabled: bool = env.storage().instance().get(&DataKey::InsEnabled).unwrap_or(true);
        let mut ins_premium: i128 = 0;
        if ins_enabled {
            if let Some(config) = env.storage().instance().get::<DataKey, InsurancePoolConfig>(&DataKey::InsPoolCfg) {
                ins_premium = (amount * config.tip_premium_bps as i128) / 10_000;
            }
        }

        let creator_amount = amount.checked_sub(fee).and_then(|a| a.checked_sub(ins_premium)).unwrap_or(0);

        // ── state updates before external call (CEI pattern) ─────────────────
        if fee > 0 {
            let fee_key = DataKey::PlatformFeeBalance(token.clone());
            let current_fee: i128 = env.storage().instance().get(&fee_key).unwrap_or(0);
            let new_fee_bal: i128 = current_fee
                .checked_add(fee)
                .expect("fee overflow");
            env.storage().instance().set(&fee_key, &new_fee_bal);
        }

        if ins_premium > 0 {
            let pool_key = DataKey::InsPoolToken(token.clone());
            let mut pool: InsurancePool = env
                .storage()
                .persistent()
                .get(&pool_key)
                .unwrap_or_else(|| InsurancePool {
                    token: token.clone(),
                    total_reserves: 0,
                    total_contributions: 0,
                    total_claims_paid: 0,
                    active_claims: 0,
                    total_claims: 0,
                    last_payout_time: env.ledger().timestamp(),
                });
            pool.total_reserves += ins_premium;
            env.storage().persistent().set(&pool_key, &pool);
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

        // Track which tokens this creator has received
        Self::track_creator_token(&env, &creator, &token);

        // Check and award milestones
        Self::check_and_award_milestones(&env, &creator, &token, new_tot);

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

    /// Creates an auction for a creator with an optional reserve price.
    ///
    /// `duration_seconds` must be greater than zero.
    /// Emits `("auction_created",)` with data `(auction_id, creator, token, reserve_price, ends_at)`.
    pub fn create_auction(
        env: Env,
        creator: Address,
        token: Address,
        reserve_price: i128,
        duration_seconds: u64,
    ) -> u64 {
        Self::require_not_paused(&env);
        creator.require_auth();
        if reserve_price < 0 {
            panic_with_error!(&env, TipJarError::InvalidReservePrice);
        }
        if duration_seconds == 0 {
            panic_with_error!(&env, TipJarError::InvalidAuctionDuration);
        }

        let whitelisted: bool = env
            .storage()
            .instance()
            .get(&DataKey::TokenWhitelist(token.clone()))
            .unwrap_or(false);
        if !whitelisted {
            panic_with_error!(&env, TipJarError::TokenNotWhitelisted);
        }

        let auction_id: u64 = env.storage().instance().get(&DataKey::AuctionCounter).unwrap_or(0);
        env.storage().instance().set(&DataKey::AuctionCounter, &(auction_id + 1));

        let now = env.ledger().timestamp();
        let ends_at = now.saturating_add(duration_seconds);
        let auction = Auction {
            auction_id,
            creator: creator.clone(),
            token: token.clone(),
            reserve_price,
            highest_bid: 0,
            highest_bidder: None,
            ends_at,
            created_at: now,
            settled: false,
        };

        env.storage().persistent().set(&DataKey::Auction(auction_id), &auction);
        Self::add_creator_auction(&env, &creator, auction_id);

        env.events().publish(
            (symbol_short!("auc_crt"),),
            (auction_id, creator.clone(), token.clone(), reserve_price, ends_at),
        );
        auction_id
    }

    /// Places a bid on an active auction.
    ///
    /// `amount` must exceed the current highest bid and, for the first bid,
    /// must meet the reserve price.
    /// Emits `("auction_bid",)` with data `(auction_id, bidder, amount)`.
    pub fn place_bid(env: Env, bidder: Address, auction_id: u64, amount: i128) {
        Self::require_not_paused(&env);
        bidder.require_auth();
        if amount <= 0 {
            panic_with_error!(&env, TipJarError::InvalidAmount);
        }

        let mut auction: Auction = env
            .storage()
            .persistent()
            .get(&DataKey::Auction(auction_id))
            .unwrap_or_else(|| panic_with_error!(&env, TipJarError::AuctionNotFound));
        if auction.settled {
            panic_with_error!(&env, TipJarError::AuctionAlreadySettled);
        }

        let now = env.ledger().timestamp();
        if now >= auction.ends_at {
            panic_with_error!(&env, TipJarError::AuctionEnded);
        }

        if amount <= auction.highest_bid {
            panic_with_error!(&env, TipJarError::AuctionBidTooLow);
        }
        if auction.highest_bid == 0 && amount < auction.reserve_price {
            panic_with_error!(&env, TipJarError::AuctionReserveNotMet);
        }

        let previous_highest_bid = auction.highest_bid;
        let previous_highest_bidder = auction.highest_bidder.clone();

        auction.highest_bid = amount;
        auction.highest_bidder = Some(bidder.clone());
        env.storage().persistent().set(&DataKey::Auction(auction_id), &auction);

        token::Client::new(&env, &auction.token).transfer(
            &bidder,
            &env.current_contract_address(),
            &amount,
        );

        if let Some(previous_bidder) = previous_highest_bidder {
            token::Client::new(&env, &auction.token).transfer(
                &env.current_contract_address(),
                &previous_bidder,
                &previous_highest_bid,
            );
        }

        env.events().publish(
            (symbol_short!("auc_bid"),),
            (auction_id, bidder.clone(), amount),
        );
    }

    /// Settles an ended auction and credits creator earnings if successful.
    ///
    /// Emits `("auction_settled",)` on success or `("auction_failed",)` when the
    /// reserve is not met.
    pub fn settle_auction(env: Env, creator: Address, auction_id: u64) {
        Self::require_not_paused(&env);
        creator.require_auth();

        let mut auction: Auction = env
            .storage()
            .persistent()
            .get(&DataKey::Auction(auction_id))
            .unwrap_or_else(|| panic_with_error!(&env, TipJarError::AuctionNotFound));
        if auction.creator != creator {
            panic_with_error!(&env, TipJarError::AuctionUnauthorized);
        }
        if auction.settled {
            panic_with_error!(&env, TipJarError::AuctionAlreadySettled);
        }

        let now = env.ledger().timestamp();
        if now < auction.ends_at {
            panic_with_error!(&env, TipJarError::AuctionNotEnded);
        }

        auction.settled = true;
        env.storage().persistent().set(&DataKey::Auction(auction_id), &auction);

        if auction.highest_bid == 0 || auction.highest_bid < auction.reserve_price {
            if let Some(winner) = auction.highest_bidder {
                token::Client::new(&env, &auction.token).transfer(
                    &env.current_contract_address(),
                    &winner,
                    &auction.highest_bid,
                );
            }
            env.events().publish(
                (symbol_short!("auc_fail"),),
                (auction_id, auction.highest_bidder, auction.highest_bid),
            );
            return;
        }

        let fee_bp: u32 = env.storage().instance().get(&DataKey::FeeBasisPoints).unwrap_or(0);
        let fee: i128 = (auction.highest_bid * fee_bp as i128) / 10_000;
        let mut ins_premium: i128 = 0;
        let ins_enabled: bool = env.storage().instance().get(&DataKey::InsEnabled).unwrap_or(true);
        if ins_enabled {
            if let Some(config) = env
                .storage()
                .instance()
                .get::<DataKey, InsurancePoolConfig>(&DataKey::InsPoolCfg)
            {
                ins_premium = (auction.highest_bid * config.tip_premium_bps as i128) / 10_000;
            }
        }
        let creator_amount = auction
            .highest_bid
            .checked_sub(fee)
            .and_then(|a| a.checked_sub(ins_premium))
            .unwrap_or(0);

        if fee > 0 {
            let fee_key = DataKey::PlatformFeeBalance(auction.token.clone());
            let current_fee: i128 = env.storage().instance().get(&fee_key).unwrap_or(0);
            let new_fee_bal = current_fee.checked_add(fee).expect("fee overflow");
            env.storage().instance().set(&fee_key, &new_fee_bal);
        }

        if ins_premium > 0 {
            let pool_key = DataKey::InsPoolToken(auction.token.clone());
            let mut pool: InsurancePool = env
                .storage()
                .persistent()
                .get(&pool_key)
                .unwrap_or_else(|| InsurancePool {
                    token: auction.token.clone(),
                    total_reserves: 0,
                    total_contributions: 0,
                    total_claims_paid: 0,
                    active_claims: 0,
                    total_claims: 0,
                    last_payout_time: env.ledger().timestamp(),
                });
            pool.total_reserves = pool
                .total_reserves
                .checked_add(ins_premium)
                .expect("insurance reserve overflow");
            env.storage().persistent().set(&pool_key, &pool);
        }

        if creator_amount > 0 {
            let bal_key = DataKey::CreatorBalance(creator.clone(), auction.token.clone());
            let existing_bal: i128 = env.storage().persistent().get(&bal_key).unwrap_or(0);
            let new_bal = existing_bal
                .checked_add(creator_amount)
                .expect("balance overflow");
            env.storage().persistent().set(&bal_key, &new_bal);

            let tot_key = DataKey::CreatorTotal(creator.clone(), auction.token.clone());
            let existing_tot: i128 = env.storage().persistent().get(&tot_key).unwrap_or(0);
            let new_tot = existing_tot
                .checked_add(creator_amount)
                .expect("total overflow");
            env.storage().persistent().set(&tot_key, &new_tot);

            if let Some(winner) = auction.highest_bidder.clone() {
                Self::update_leaderboard_stats(&env, &winner, &creator, creator_amount);
            }
            Self::track_creator_token(&env, &creator, &auction.token);
            Self::check_and_award_milestones(&env, &creator, &auction.token, new_tot);
        }

        env.events().publish(
            (symbol_short!("auc_sett"),),
            (auction_id, creator.clone(), auction.highest_bid, auction.token.clone()),
        );
    }

    /// Returns auction details by ID.
    pub fn get_auction(env: Env, auction_id: u64) -> Option<Auction> {
        env.storage().persistent().get(&DataKey::Auction(auction_id))
    }

    /// Returns the auction IDs created by a given creator.
    pub fn get_creator_auctions(env: Env, creator: Address) -> Vec<u64> {
        env.storage()
            .persistent()
            .get(&DataKey::CreatorAuctions(creator))
            .unwrap_or_else(|| Vec::new(env))
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

    /// Records `token` in the creator's token list if not already present.
    fn track_creator_token(env: &Env, creator: &Address, token: &Address) {
        let key = DataKey::CreatorTokens(creator.clone());
        let mut tokens: Vec<Address> = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or_else(|| Vec::new(env));
        if !tokens.contains(token) {
            tokens.push_back(token.clone());
            env.storage().persistent().set(&key, &tokens);
        }
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
        let schedule_id: u64 = env.storage().instance().get(&DataKey::VestingSchedCtr).unwrap_or(0);

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
            .set(&DataKey::VestingSchedCtr, &(schedule_id + 1));

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

    /// Returns all token addresses that `creator` has ever received tips in.
    pub fn get_creator_tokens(env: Env, creator: Address) -> Vec<Address> {
        env.storage()
            .persistent()
            .get(&DataKey::CreatorTokens(creator))
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
        
        // --- Insurance Premium Calculation ---
        let ins_enabled: bool = env.storage().instance().get(&DataKey::InsEnabled).unwrap_or(true);
        let mut ins_premium: i128 = 0;
        if ins_enabled {
            if let Some(config) = env.storage().instance().get::<DataKey, InsurancePoolConfig>(&DataKey::InsPoolCfg) {
                ins_premium = (amount * config.tip_premium_bps as i128) / 10_000;
            }
        }

        let net = amount.checked_sub(fee).and_then(|a| a.checked_sub(ins_premium)).unwrap_or(0);

        token::Client::new(&env, &token).transfer(
            &sender,
            &env.current_contract_address(),
            &amount,
        );

        if ins_premium > 0 {
            let pool_key = DataKey::InsPoolToken(token.clone());
            let mut pool: InsurancePool = env
                .storage()
                .persistent()
                .get(&pool_key)
                .unwrap_or_else(|| InsurancePool {
                    token: token.clone(),
                    total_reserves: 0,
                    total_contributions: 0,
                    total_claims_paid: 0,
                    active_claims: 0,
                    total_claims: 0,
                    last_payout_time: env.ledger().timestamp(),
                });
            pool.total_reserves += ins_premium;
            env.storage().persistent().set(&pool_key, &pool);
        }

        let bal_key = DataKey::CreatorBalance(creator.clone(), token.clone());
        let current_bal: i128 = env.storage().persistent().get(&bal_key).unwrap_or(0);
        let new_bal: i128 = current_bal
            .checked_add(net).expect("balance overflow");
        env.storage().persistent().set(&bal_key, &new_bal);

        let tot_key = DataKey::CreatorTotal(creator.clone(), token.clone());
        let current_tot: i128 = env.storage().persistent().get(&tot_key).unwrap_or(0);
        let new_tot: i128 = current_tot
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

    /// Applies any required state migrations after a WASM upgrade.
    ///
    /// This is a safe, idempotent entrypoint for release-specific storage
    /// updates that must run after the host preserves storage during upgrade.
    pub fn migrate_state(env: Env) {
        let version = upgrade::get_version(&env);
        match version {
            1 => {
                // No migration is required for the current v1 release.
                // Future upgrades can add version-specific migration paths
                // here, e.g. `migrate_v1_to_v2` or `migrate_v2_to_v3`.
            }
            _ => {}
        }
    }

    /// Sets the configuration for a subscription tier. Admin only.
    ///
    /// `price` is the amount charged per payment interval.
    /// `benefits` is a human-readable description of what the tier provides.
    /// Emits `("tier_set",)` with data `(tier, price)`.
    pub fn set_tier_config(
        env: Env,
        admin: Address,
        tier: SubscriptionTier,
        price: i128,
        benefits: String,
    ) {
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic_with_error!(&env, TipJarError::Unauthorized);
        }
        if price <= 0 {
            panic_with_error!(&env, TipJarError::InvalidAmount);
        }
        let config = TierConfig { price, benefits };
        env.storage()
            .persistent()
            .set(&DataKey::TierConfig(tier.clone()), &config);
        env.events()
            .publish((symbol_short!("tier_set"),), (tier, price));
    }

    /// Returns the configuration for a tier, or `None` if not configured.
    pub fn get_tier_config(env: Env, tier: SubscriptionTier) -> Option<TierConfig> {
        env.storage()
            .persistent()
            .get(&DataKey::TierConfig(tier))
    }

    /// Returns the benefits description for a tier, or `None` if not configured.
    pub fn get_tier_benefits(env: Env, tier: SubscriptionTier) -> Option<String> {
        env.storage()
            .persistent()
            .get::<DataKey, TierConfig>(&DataKey::TierConfig(tier))
            .map(|c| c.benefits)
    }

    /// Creates a recurring tip subscription from `subscriber` to `creator` at the given tier.
    ///
    /// The tier must be configured via `set_tier_config` first.
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
            tier: SubscriptionTier::Bronze,
            pending_tier: None,
        };
        env.storage()
            .persistent()
            .set(&DataKey::Subscription(subscriber.clone(), creator.clone()), &sub);
        env.events().publish(
            (symbol_short!("sub_new"), creator),
            (subscriber, amount, interval_seconds),
        );
    }

    /// Creates a tiered subscription from `subscriber` to `creator`.
    ///
    /// The tier must be configured via `set_tier_config`. The price from the tier
    /// config is used as the payment amount.
    /// Minimum interval is 1 day (86 400 seconds).
    ///
    /// Emits `("sub_new", creator)` with data `(subscriber, amount, interval_seconds)`.
    pub fn create_tiered_subscription(
        env: Env,
        subscriber: Address,
        creator: Address,
        token: Address,
        tier: SubscriptionTier,
        interval_seconds: u64,
    ) {
        Self::require_not_paused(&env);
        subscriber.require_auth();
        let config: TierConfig = env
            .storage()
            .persistent()
            .get(&DataKey::TierConfig(tier.clone()))
            .unwrap_or_else(|| panic_with_error!(&env, TipJarError::TierNotConfigured));
        const MIN_INTERVAL: u64 = 86_400;
        if interval_seconds < MIN_INTERVAL {
            panic_with_error!(&env, TipJarError::InvalidInterval);
        }
        let now = env.ledger().timestamp();
        let amount = config.price;
        let sub = Subscription {
            subscriber: subscriber.clone(),
            creator: creator.clone(),
            token,
            amount,
            interval_seconds,
            last_payment: 0,
            next_payment: now,
            status: SubscriptionStatus::Active,
            tier: tier.clone(),
            pending_tier: None,
        };
        env.storage()
            .persistent()
            .set(&DataKey::Subscription(subscriber.clone(), creator.clone()), &sub);
        env.events().publish(
            (symbol_short!("sub_new"), creator),
            (subscriber, amount, interval_seconds),
        );
    }

    /// Upgrades an active subscription to a higher tier immediately.
    ///
    /// Executes an immediate payment at the new tier's price and updates the
    /// subscription amount for future payments.
    /// Emits `("sub_upgr", creator)` with data `(subscriber, new_tier_price)`.
    pub fn upgrade_subscription(
        env: Env,
        subscriber: Address,
        creator: Address,
        new_tier: SubscriptionTier,
    ) {
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
        let config: TierConfig = env
            .storage()
            .persistent()
            .get(&DataKey::TierConfig(new_tier.clone()))
            .unwrap_or_else(|| panic_with_error!(&env, TipJarError::TierNotConfigured));

        // Execute immediate payment at new tier price.
        token::Client::new(&env, &sub.token).transfer(
            &subscriber,
            &env.current_contract_address(),
            &config.price,
        );
        let bal_key = DataKey::CreatorBalance(creator.clone(), sub.token.clone());
        let bal: i128 = env.storage().persistent().get(&bal_key).unwrap_or(0);
        env.storage().persistent().set(&bal_key, &(bal + config.price));
        let tot_key = DataKey::CreatorTotal(creator.clone(), sub.token.clone());
        let tot: i128 = env.storage().persistent().get(&tot_key).unwrap_or(0);
        env.storage().persistent().set(&tot_key, &(tot + config.price));

        let now = env.ledger().timestamp();
        sub.tier = new_tier;
        sub.amount = config.price;
        sub.last_payment = now;
        sub.next_payment = now + sub.interval_seconds;
        sub.pending_tier = None;
        env.storage().persistent().set(&key, &sub);
        env.events().publish(
            (symbol_short!("sub_upgr"), creator),
            (subscriber, config.price),
        );
    }

    /// Schedules a downgrade to a lower tier, effective at the next payment cycle.
    ///
    /// The current tier and amount remain active until `execute_subscription_payment`
    /// is called, at which point the pending tier is applied.
    /// Emits `("sub_dngr", creator)` with data `(subscriber, new_tier_price)`.
    pub fn downgrade_subscription(
        env: Env,
        subscriber: Address,
        creator: Address,
        new_tier: SubscriptionTier,
    ) {
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
        // Validate the target tier is configured.
        let config: TierConfig = env
            .storage()
            .persistent()
            .get(&DataKey::TierConfig(new_tier.clone()))
            .unwrap_or_else(|| panic_with_error!(&env, TipJarError::TierNotConfigured));
        sub.pending_tier = Some(new_tier);
        env.storage().persistent().set(&key, &sub);
        env.events().publish(
            (symbol_short!("sub_dngr"), creator),
            (subscriber, config.price),
        );
    }

    /// Executes a due subscription payment, transferring tokens from subscriber
    /// into escrow for the creator.
    ///
    /// Applies any pending tier downgrade before charging.
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

        // Apply pending downgrade if present.
        if let Some(pending) = sub.pending_tier.clone() {
            if let Some(config) = env
                .storage()
                .persistent()
                .get::<DataKey, TierConfig>(&DataKey::TierConfig(pending.clone()))
            {
                sub.tier = pending;
                sub.amount = config.price;
            }
            sub.pending_tier = None;
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

        // --- Insurance Premium Calculation ---
        let ins_enabled: bool = env.storage().instance().get(&DataKey::InsEnabled).unwrap_or(true);
        let mut ins_premium: i128 = 0;
        if ins_enabled {
            if let Some(config) = env.storage().instance().get::<DataKey, InsurancePoolConfig>(&DataKey::InsPoolCfg) {
                ins_premium = (amount * config.tip_premium_bps as i128) / 10_000;
            }
        }

        let creator_amount = amount.checked_sub(fee).and_then(|a| a.checked_sub(ins_premium)).unwrap_or(0);

        if fee > 0 {
            let fee_key = DataKey::PlatformFeeBalance(token.clone());
            let new_fee_bal: i128 =
                env.storage().instance().get(&fee_key).unwrap_or(0) + fee;
            env.storage().instance().set(&fee_key, &new_fee_bal);
        }

        if ins_premium > 0 {
            let pool_key = DataKey::InsPoolToken(token.clone());
            let mut pool: InsurancePool = env
                .storage()
                .persistent()
                .get(&pool_key)
                .unwrap_or_else(|| InsurancePool {
                    token: token.clone(),
                    total_reserves: 0,
                    total_contributions: 0,
                    total_claims_paid: 0,
                    active_claims: 0,
                    total_claims: 0,
                    last_payout_time: env.ledger().timestamp(),
                });
            pool.total_reserves += ins_premium;
            env.storage().persistent().set(&pool_key, &pool);
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
        env.events().publish((symbol_short!("ref_wind"),), refund_window_seconds);
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
            (symbol_short!("tip_exp"), time_lock.creator.clone()),
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
            .or_else(|| env.storage().instance().get(&DataKey::DefWithdrawLimits))
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
            .set(&DataKey::DefWithdrawLimits, &defaults);

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
            .or_else(|| env.storage().instance().get(&DataKey::DefWithdrawLimits))
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
            .unwrap_or_else(|| panic_with_error!(&env, TipJarError::MsigReqNotFound));

        if request.executed || request.cancelled {
            panic_with_error!(&env, TipJarError::MultiSigReqClosed);
        }
        if env.ledger().timestamp() > request.expires_at {
            panic_with_error!(&env, TipJarError::MultiSigReqExpired);
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
            .unwrap_or_else(|| panic_with_error!(&env, TipJarError::MsigReqNotFound));

        if request.executed || request.cancelled {
            panic_with_error!(&env, TipJarError::MultiSigReqClosed);
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
            .unwrap_or_else(|| panic_with_error!(&env, TipJarError::MsigReqNotFound))
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

    // ── dispute resolution ────────────────────────────────────────────────────

    /// Creates a dispute for a tip. Only the tipper or creator can initiate.
    ///
    /// Emits `("dispute_created",)` with data `(dispute_id, tip_id, initiator)`.
    pub fn create_dispute(
        env: Env,
        tip_id: u64,
        initiator: Address,
        reason: String,
    ) -> u64 {
        Self::require_not_paused(&env);
        initiator.require_auth();

        let dispute_id: u64 = env.storage().instance().get(&DataKey::DisputeCounter).unwrap_or(0);
        env.storage().instance().set(&DataKey::DisputeCounter, &(dispute_id + 1));

        let created_at = env.ledger().timestamp();
        let dispute = dispute::Dispute {
            id: dispute_id,
            tip_id,
            initiator: initiator.clone(),
            reason,
            status: dispute::DisputeStatus::Open,
            arbitrator: None,
            resolution: None,
            created_at,
        };

        env.storage().persistent().set(&DataKey::Dispute(dispute_id), &dispute);

        let mut creator_disputes: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::CreatorDisputes(initiator.clone()))
            .unwrap_or_else(|| Vec::new(&env));
        creator_disputes.push_back(dispute_id);
        env.storage()
            .persistent()
            .set(&DataKey::CreatorDisputes(initiator.clone()), &creator_disputes);

        env.events().publish(
            (symbol_short!("disp_crt"),),
            (dispute_id, tip_id, initiator),
        );

        dispute_id
    }

    /// Assigns an arbitrator to a dispute. Admin only.
    ///
    /// Emits `("dispute_assigned",)` with data `(dispute_id, arbitrator)`.
    pub fn assign_arbitrator(env: Env, admin: Address, dispute_id: u64, arbitrator: Address) {
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic_with_error!(&env, TipJarError::Unauthorized);
        }

        let mut dispute: dispute::Dispute = env
            .storage()
            .persistent()
            .get(&DataKey::Dispute(dispute_id))
            .unwrap_or_else(|| panic_with_error!(&env, TipJarError::DisputeNotFound));

        dispute.arbitrator = Some(arbitrator.clone());
        dispute.status = dispute::DisputeStatus::UnderReview;
        env.storage().persistent().set(&DataKey::Dispute(dispute_id), &dispute);

        env.events().publish(
            (symbol_short!("disp_asgn"),),
            (dispute_id, arbitrator),
        );
    }

    /// Submits evidence for a dispute.
    ///
    /// Emits `("evidence_submitted",)` with data `(dispute_id, submitter)`.
    pub fn submit_evidence(
        env: Env,
        dispute_id: u64,
        submitter: Address,
        evidence: String,
    ) {
        Self::require_not_paused(&env);
        submitter.require_auth();

        let dispute: dispute::Dispute = env
            .storage()
            .persistent()
            .get(&DataKey::Dispute(dispute_id))
            .unwrap_or_else(|| panic_with_error!(&env, TipJarError::DisputeNotFound));

        if dispute.status != dispute::DisputeStatus::Open && dispute.status != dispute::DisputeStatus::UnderReview {
            panic_with_error!(&env, TipJarError::DisputeNotOpen);
        }

        let evidence_idx: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::DisputeEvidenceCounter(dispute_id))
            .unwrap_or(0);

        let submitted_at = env.ledger().timestamp();
        let evidence_record = dispute::DisputeEvidence {
            dispute_id,
            submitter: submitter.clone(),
            evidence,
            submitted_at,
        };

        env.storage()
            .persistent()
            .set(&DataKey::DisputeEvidence(dispute_id, evidence_idx), &evidence_record);
        env.storage()
            .persistent()
            .set(&DataKey::DisputeEvidenceCounter(dispute_id), &(evidence_idx + 1));

        env.events().publish(
            (symbol_short!("evid_sub"),),
            (dispute_id, submitter),
        );
    }

    /// Resolves a dispute. Only the arbitrator can resolve.
    ///
    /// Emits `("dispute_resolved",)` with data `(dispute_id, resolution)`.
    pub fn resolve_dispute(
        env: Env,
        dispute_id: u64,
        arbitrator: Address,
        resolution: String,
        approved: bool,
    ) {
        Self::require_not_paused(&env);
        arbitrator.require_auth();

        let mut dispute: dispute::Dispute = env
            .storage()
            .persistent()
            .get(&DataKey::Dispute(dispute_id))
            .unwrap_or_else(|| panic_with_error!(&env, TipJarError::DisputeNotFound));

        if dispute.arbitrator != Some(arbitrator.clone()) {
            panic_with_error!(&env, TipJarError::DisputeUnauthorized);
        }

        dispute.resolution = Some(resolution.clone());
        dispute.status = if approved {
            dispute::DisputeStatus::Resolved
        } else {
            dispute::DisputeStatus::Rejected
        };

        env.storage().persistent().set(&DataKey::Dispute(dispute_id), &dispute);

        env.events().publish(
            (symbol_short!("disp_res"),),
            (dispute_id, resolution),
        );
    }

    /// Returns a dispute by ID.
    pub fn get_dispute(env: Env, dispute_id: u64) -> dispute::Dispute {
        env.storage()
            .persistent()
            .get(&DataKey::Dispute(dispute_id))
            .unwrap_or_else(|| panic_with_error!(&env, TipJarError::DisputeNotFound))
    }

    /// Returns all disputes for a creator.
    pub fn get_creator_disputes(env: Env, creator: Address) -> Vec<u64> {
        env.storage()
            .persistent()
            .get(&DataKey::CreatorDisputes(creator))
            .unwrap_or_else(|| Vec::new(&env))
    }

    /// Returns evidence for a dispute.
    pub fn get_dispute_evidence(env: Env, dispute_id: u64, evidence_idx: u64) -> dispute::DisputeEvidence {
        env.storage()
            .persistent()
            .get(&DataKey::DisputeEvidence(dispute_id, evidence_idx))
            .unwrap_or_else(|| panic_with_error!(&env, TipJarError::DisputeNotFound))
    }

    // ── batch tipping ─────────────────────────────────────────────────────────

    /// Sends multiple tips in a single transaction to reduce gas costs.
    ///
    /// `tips` is a vector of (creator, amount) pairs. Returns the number of successful tips.
    /// Emits `("batch_tip",)` with data `(tipper, count, total_amount)`.
    pub fn batch_tip(
        env: Env,
        tipper: Address,
        token: Address,
        tips: Vec<BatchTip>,
    ) -> u32 {
        Self::require_not_paused(&env);
        tipper.require_auth();

        if tips.len() == 0 || tips.len() > 100 {
            panic_with_error!(&env, TipJarError::BatchTooLarge);
        }

        let whitelisted: bool = env
            .storage()
            .instance()
            .get(&DataKey::TokenWhitelist(token.clone()))
            .unwrap_or(false);
        if !whitelisted {
            panic_with_error!(&env, TipJarError::TokenNotWhitelisted);
        }

        let mut total_amount: i128 = 0;
        for tip in tips.iter() {
            if tip.amount <= 0 {
                panic_with_error!(&env, TipJarError::InvalidAmount);
            }
            total_amount = total_amount.checked_add(tip.amount).expect("total overflow");
        }

        // Transfer all tokens at once
        token::Client::new(&env, &token).transfer(&tipper, &env.current_contract_address(), &total_amount);

        let fee_bp: u32 = env.storage().instance().get(&DataKey::FeeBasisPoints).unwrap_or(0);
        let mut successful_tips: u32 = 0;

        for tip in tips.iter() {
            let fee: i128 = (tip.amount * fee_bp as i128) / 10_000;
            let creator_amount = tip.amount - fee;

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

            let bal_key = DataKey::CreatorBalance(tip.creator.clone(), token.clone());
            let existing_bal: i128 = env.storage().persistent().get(&bal_key).unwrap_or(0);
            let new_bal: i128 = existing_bal.checked_add(creator_amount).expect("balance overflow");
            env.storage().persistent().set(&bal_key, &new_bal);

            let tot_key = DataKey::CreatorTotal(tip.creator.clone(), token.clone());
            let existing_tot: i128 = env.storage().persistent().get(&tot_key).unwrap_or(0);
            let new_tot: i128 = existing_tot.checked_add(creator_amount).expect("total overflow");
            env.storage().persistent().set(&tot_key, &new_tot);

            Self::update_leaderboard_stats(&env, &tipper, &tip.creator, creator_amount);
            successful_tips += 1;
        }

        env.events().publish(
            (symbol_short!("batch_tip"),),
            (tipper, successful_tips, total_amount),
        );

        successful_tips
    }

    // ── batch withdraw ────────────────────────────────────────────────────────

    /// Withdraws balances across multiple tokens in a single transaction.
    ///
    /// Each `WithdrawOperation` specifies a `token` and the `amount` to withdraw.
    /// All operations are validated before any transfer is executed (atomic: all-or-nothing).
    /// Batch size is capped at 20 to prevent gas exhaustion.
    ///
    /// Emits `("batch_wdr",)` with data `(creator, count, operations_count)` on success.
    /// Emits `("batch_wdr_op",)` with data `(creator, token, amount, index)` for each operation.
    pub fn batch_withdraw(
        env: Env,
        creator: Address,
        operations: Vec<WithdrawOperation>,
    ) -> Vec<BatchResult> {
        Self::require_not_paused(&env);
        creator.require_auth();

        const MAX_BATCH_SIZE: u32 = 20;
        let op_count = operations.len();

        if op_count == 0 || op_count > MAX_BATCH_SIZE {
            panic_with_error!(&env, TipJarError::BatchSizeExceeded);
        }

        // ── Validation pass (checks before any state change) ─────────────────
        for op in operations.iter() {
            if op.amount <= 0 {
                panic_with_error!(&env, TipJarError::InvalidAmount);
            }

            let bal_key = DataKey::CreatorBalance(creator.clone(), op.token.clone());
            let balance: i128 = env
                .storage()
                .persistent()
                .get(&bal_key)
                .unwrap_or_else(|| env.storage().instance().get(&bal_key).unwrap_or(0));

            if op.amount > balance {
                panic_with_error!(&env, TipJarError::InsufficientBalance);
            }
        }

        // ── Effects pass (state updates before external calls) ────────────────
        for op in operations.iter() {
            let bal_key = DataKey::CreatorBalance(creator.clone(), op.token.clone());
            let balance: i128 = env
                .storage()
                .persistent()
                .get(&bal_key)
                .unwrap_or_else(|| env.storage().instance().get(&bal_key).unwrap_or(0));
            env.storage()
                .persistent()
                .set(&bal_key, &(balance - op.amount));
        }

        // ── Interactions pass (external token transfers) ──────────────────────
        let mut results: Vec<BatchResult> = Vec::new(&env);
        let contract_address = env.current_contract_address();

        for (index, op) in operations.iter().enumerate() {
            token::Client::new(&env, &op.token).transfer(
                &contract_address,
                &creator,
                &op.amount,
            );

            env.events().publish(
                (symbol_short!("btch_wdr"),),
                (creator.clone(), op.token.clone(), op.amount, index as u32),
            );

            results.push_back(BatchResult {
                success: true,
                index: index as u32,
            });
        }

        env.events().publish(
            (symbol_short!("batch_wdr"),),
            (creator.clone(), op_count),
        );

        results
    }

    /// Sends multiple tips in a single transaction, returning per-operation results.
    ///
    /// Accepts up to 20 `TipOperation` entries. All amounts are validated before any
    /// token transfer occurs (atomic: all-or-nothing). A single transfer covers the
    /// total amount, then balances are distributed to each creator.
    ///
    /// Emits `("btch_tip2",)` with data `(tipper, count, total_amount)` on success.
    /// Emits `("btch_tip_op",)` with data `(tipper, creator, token, amount, index)` per operation.
    pub fn batch_tip_v2(
        env: Env,
        tipper: Address,
        operations: Vec<TipOperation>,
    ) -> Vec<BatchResult> {
        Self::require_not_paused(&env);
        tipper.require_auth();

        const MAX_BATCH_SIZE: u32 = 20;
        let op_count = operations.len();

        if op_count == 0 || op_count > MAX_BATCH_SIZE {
            panic_with_error!(&env, TipJarError::BatchSizeExceeded);
        }

        // ── Validation pass ───────────────────────────────────────────────────
        // Group totals by token so we can do one transfer per token.
        let mut token_totals: Map<Address, i128> = Map::new(&env);

        for op in operations.iter() {
            if op.amount <= 0 {
                panic_with_error!(&env, TipJarError::InvalidAmount);
            }

            let whitelisted: bool = env
                .storage()
                .instance()
                .get(&DataKey::TokenWhitelist(op.token.clone()))
                .unwrap_or(false);
            if !whitelisted {
                panic_with_error!(&env, TipJarError::TokenNotWhitelisted);
            }

            let existing: i128 = token_totals.get(op.token.clone()).unwrap_or(0);
            token_totals.set(
                op.token.clone(),
                existing.checked_add(op.amount).expect("total overflow"),
            );
        }

        // ── Transfer total per token (one transfer per distinct token) ─────────
        let contract_address = env.current_contract_address();
        let fee_bp: u32 = env
            .storage()
            .instance()
            .get(&DataKey::FeeBasisPoints)
            .unwrap_or(0);

        for token_key in token_totals.keys() {
            let total: i128 = token_totals.get(token_key.clone()).unwrap_or(0);
            token::Client::new(&env, &token_key).transfer(&tipper, &contract_address, &total);
        }

        // ── Effects + Interactions pass ───────────────────────────────────────
        let mut results: Vec<BatchResult> = Vec::new(&env);
        let mut grand_total: i128 = 0;

        for (index, op) in operations.iter().enumerate() {
            let fee: i128 = (op.amount * fee_bp as i128) / 10_000;
            let creator_amount = op.amount - fee;

            if fee > 0 {
                let fee_key = DataKey::PlatformFeeBalance(op.token.clone());
                let new_fee_bal: i128 = env
                    .storage()
                    .instance()
                    .get(&fee_key)
                    .unwrap_or(0i128)
                    .checked_add(fee)
                    .expect("fee overflow");
                env.storage().instance().set(&fee_key, &new_fee_bal);
            }

            let bal_key = DataKey::CreatorBalance(op.creator.clone(), op.token.clone());
            let existing_bal: i128 = env.storage().persistent().get(&bal_key).unwrap_or(0);
            env.storage()
                .persistent()
                .set(&bal_key, &existing_bal.checked_add(creator_amount).expect("balance overflow"));

            let tot_key = DataKey::CreatorTotal(op.creator.clone(), op.token.clone());
            let existing_tot: i128 = env.storage().persistent().get(&tot_key).unwrap_or(0);
            env.storage()
                .persistent()
                .set(&tot_key, &existing_tot.checked_add(creator_amount).expect("total overflow"));

            Self::update_leaderboard_stats(&env, &tipper, &op.creator, creator_amount);

            env.events().publish(
                (symbol_short!("btp_op"),),
                (tipper.clone(), op.creator.clone(), op.token.clone(), creator_amount, index as u32),
            );

            results.push_back(BatchResult {
                success: true,
                index: index as u32,
            });

            grand_total = grand_total.checked_add(op.amount).expect("grand total overflow");
        }

        env.events().publish(
            (symbol_short!("btch_tip2"),),
            (tipper, op_count, grand_total),
        );

        results
    }

    // ── liquidity mining ──────────────────────────────────────────────────────

    /// Creates a new liquidity mining program.
    ///
    /// Transfers `total_rewards` of `reward_token` from `admin` into the contract.
    /// Returns the new program ID.
    ///
    /// * `reward_rate_bps`  — annual reward rate in basis points (e.g. 2000 = 20 %).
    /// * `vesting_cliff`    — seconds before any rewards unlock.
    /// * `vesting_duration` — total vesting window in seconds (>= cliff, <= 4 years).
    /// * `end_time`         — program end timestamp; pass 0 for no end.
    ///
    /// Emits `("lm_create",)` with `(program_id, lp_token, reward_token, total_rewards, rate_bps)`.
    pub fn lm_create_program(
        env: Env,
        admin: Address,
        lp_token: Address,
        reward_token: Address,
        total_rewards: i128,
        reward_rate_bps: u32,
        vesting_cliff: u64,
        vesting_duration: u64,
        end_time: u64,
    ) -> u64 {
        Self::require_not_paused(&env);
        liquidity_mining::create_program(
            &env,
            &admin,
            &lp_token,
            &reward_token,
            total_rewards,
            reward_rate_bps,
            vesting_cliff,
            vesting_duration,
            end_time,
        )
    }

    /// Stakes `amount` LP tokens into a liquidity mining program.
    ///
    /// Emits `("lm_stake",)` with `(provider, program_id, amount, new_total_staked)`.
    pub fn lm_stake(
        env: Env,
        provider: Address,
        program_id: u64,
        amount: i128,
    ) {
        Self::require_not_paused(&env);
        liquidity_mining::stake(&env, &provider, program_id, amount);
    }

    /// Unstakes `amount` LP tokens from a liquidity mining program.
    ///
    /// Accrues pending rewards before reducing the position.
    /// Emits `("lm_unstk",)` with `(provider, program_id, amount, remaining_staked)`.
    pub fn lm_unstake(
        env: Env,
        provider: Address,
        program_id: u64,
        amount: i128,
    ) {
        Self::require_not_paused(&env);
        liquidity_mining::unstake(&env, &provider, program_id, amount);
    }

    /// Claims all vested mining rewards for a provider. Returns the amount claimed.
    ///
    /// Emits `("lm_claim",)` with `(provider, program_id, amount_claimed)`.
    pub fn lm_claim_rewards(
        env: Env,
        provider: Address,
        program_id: u64,
    ) -> i128 {
        Self::require_not_paused(&env);
        liquidity_mining::claim_rewards(&env, &provider, program_id)
    }

    /// Applies a boost to a provider's position by locking rewards for `lock_duration` seconds.
    ///
    /// Boost scales linearly from 1× (no lock) to 3× (lock = 1 year).
    /// Only increasing boosts are accepted.
    /// Emits `("lm_boost",)` with `(provider, program_id, new_boost, lock_until)`.
    pub fn lm_apply_boost(
        env: Env,
        provider: Address,
        program_id: u64,
        lock_duration: u64,
    ) {
        Self::require_not_paused(&env);
        liquidity_mining::apply_boost(&env, &provider, program_id, lock_duration);
    }

    /// Deactivates a mining program. Existing positions can still claim vested rewards.
    ///
    /// Emits `("lm_deact",)` with `(program_id,)`.
    pub fn lm_deactivate_program(env: Env, admin: Address, program_id: u64) {
        Self::require_not_paused(&env);
        liquidity_mining::deactivate_program(&env, &admin, program_id);
    }

    /// Returns the vesting schedule info for a provider's position.
    pub fn lm_get_vesting_info(
        env: Env,
        provider: Address,
        program_id: u64,
    ) -> liquidity_mining::VestingInfo {
        liquidity_mining::get_vesting_info(&env, &provider, program_id)
    }

    /// Returns a mining program by ID.
    pub fn lm_get_program(env: Env, program_id: u64) -> liquidity_mining::MiningProgram {
        liquidity_mining::get_program_info(&env, program_id)
    }

    /// Returns a provider's position in a mining program (with accrued rewards).
    pub fn lm_get_position(
        env: Env,
        provider: Address,
        program_id: u64,
    ) -> liquidity_mining::MiningPosition {
        liquidity_mining::get_position_info(&env, &provider, program_id)
    }

    /// Returns all program IDs a provider has participated in.
    pub fn lm_get_provider_programs(env: Env, provider: Address) -> Vec<u64> {
        liquidity_mining::get_provider_program_ids(&env, &provider)
    }

    /// Returns the pending (not yet vested) rewards for a provider in a program.
    pub fn lm_get_pending_rewards(env: Env, provider: Address, program_id: u64) -> i128 {
        liquidity_mining::get_pending_rewards(&env, &provider, program_id)
    }

    /// Checks and awards milestones when a creator reaches specific tip thresholds.
    ///
    /// Called internally after tips are processed. Emits milestone events.
    fn check_and_award_milestones(
        env: &Env,
        creator: &Address,
        token: &Address,
        new_total: i128,
    ) {
        let milestones = Self::get_creator_milestones(env, creator);

        for (idx, milestone) in milestones.iter().enumerate() {
            if !milestone.completed && new_total >= milestone.goal_amount {
                let reward = (milestone.goal_amount * 5) / 100; // 5% reward

                let bal_key = DataKey::CreatorBalance(creator.clone(), token.clone());
                let existing_bal: i128 = env.storage().persistent().get(&bal_key).unwrap_or(0);
                let new_bal: i128 = existing_bal.checked_add(reward).expect("balance overflow");
                env.storage().persistent().set(&bal_key, &new_bal);

                let mut updated_milestone = milestone.clone();
                updated_milestone.completed = true;
                env.storage()
                    .persistent()
                    .set(&DataKey::Milestone(creator.clone(), idx as u64), &updated_milestone);

                env.events().publish(
                    (symbol_short!("milestone"),),
                    (creator.clone(), milestone.goal_amount, reward),
                );
            }
        }
    }

    /// Creates a milestone for a creator. Admin only.
    ///
    /// Emits `("milestone_created",)` with data `(creator, goal_amount)`.
    pub fn create_milestone(
        env: Env,
        admin: Address,
        creator: Address,
        goal_amount: i128,
        description: String,
    ) -> u64 {
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic_with_error!(&env, TipJarError::Unauthorized);
        }

        if goal_amount <= 0 {
            panic_with_error!(&env, TipJarError::InvalidGoalAmount);
        }

        let counter_key = DataKey::MilestoneCounter(creator.clone());
        let milestone_id: u64 = env.storage().persistent().get(&counter_key).unwrap_or(0);

        let milestone = Milestone {
            id: milestone_id,
            creator: creator.clone(),
            goal_amount,
            current_amount: 0,
            description,
            deadline: None,
            completed: false,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Milestone(creator.clone(), milestone_id), &milestone);
        env.storage()
            .persistent()
            .set(&counter_key, &(milestone_id + 1));

        let mut active: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::ActiveMilestones(creator.clone()))
            .unwrap_or_else(|| Vec::new(&env));
        active.push_back(milestone_id);
        env.storage()
            .persistent()
            .set(&DataKey::ActiveMilestones(creator.clone()), &active);

        env.events().publish(
            (symbol_short!("ms_crt"),),
            (creator, goal_amount),
        );

        milestone_id
    }

    /// Returns all milestones for a creator.
    pub fn get_creator_milestones(env: &Env, creator: &Address) -> Vec<Milestone> {
        let milestone_ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::ActiveMilestones(creator.clone()))
            .unwrap_or_else(|| Vec::new(env));

        let mut milestones = Vec::new(env);
        for id in milestone_ids.iter() {
            if let Some(milestone) = env
                .storage()
                .persistent()
                .get::<_, Milestone>(&DataKey::Milestone(creator.clone(), id))
            {
                milestones.push_back(milestone);
            }
        }
        milestones
    }

    /// Returns a specific milestone for a creator.
    pub fn get_milestone(env: Env, creator: Address, milestone_id: u64) -> Milestone {
        env.storage()
            .persistent()
            .get(&DataKey::Milestone(creator, milestone_id))
            .unwrap_or_else(|| panic_with_error!(&env, TipJarError::MilestoneNotFound))
    }

    // ── privacy features ──────────────────────────────────────────────────────

    /// Sends an anonymous or private tip. Amount is hashed for privacy.
    ///
    /// If `is_anonymous` is true, the tipper identity is not stored.
    /// Returns the private tip ID.
    /// Emits `("private_tip",)` with data `(tip_id, creator, is_anonymous)`.
    pub fn tip_private(
        env: Env,
        creator: Address,
        token: Address,
        amount: i128,
        is_anonymous: bool,
    ) -> u64 {
        Self::require_not_paused(&env);
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

        let tip_id: u64 = env.storage().instance().get(&DataKey::PrivateTipCounter).unwrap_or(0);
        env.storage().instance().set(&DataKey::PrivateTipCounter, &(tip_id + 1));

        let amount_bytes = amount.to_le_bytes();
        let amount_hash = env.crypto().sha256(&amount_bytes);

        let tipper = if is_anonymous {
            None
        } else {
            Some(env.current_contract_address())
        };

        let created_at = env.ledger().timestamp();
        let private_tip = privacy_tip::PrivateTip {
            id: tip_id,
            creator: creator.clone(),
            amount_hash,
            is_anonymous,
            tipper,
            created_at,
            revealed: false,
        };

        env.storage().persistent().set(&DataKey::PrivateTip(tip_id), &private_tip);

        env.events().publish(
            (symbol_short!("priv_tip"),),
            (tip_id, creator, is_anonymous),
        );

        tip_id
    }

    /// Reveals the amount of a private tip by providing the original amount.
    ///
    /// The amount is hashed and compared with the stored hash. If it matches,
    /// the tip is credited to the creator and marked as revealed.
    /// Emits `("tip_revealed",)` with data `(tip_id, amount)`.
    pub fn reveal_tip(
        env: Env,
        sender: Address,
        token: Address,
        tip_id: u64,
        amount: i128,
    ) {
        Self::require_not_paused(&env);
        sender.require_auth();

        if amount <= 0 {
            panic_with_error!(&env, TipJarError::InvalidAmount);
        }

        let mut private_tip: privacy_tip::PrivateTip = env
            .storage()
            .persistent()
            .get(&DataKey::PrivateTip(tip_id))
            .unwrap_or_else(|| panic_with_error!(&env, TipJarError::PrivateTipNotFound));

        let amount_bytes = amount.to_le_bytes();
        let computed_hash = env.crypto().sha256(&amount_bytes);

        if computed_hash != private_tip.amount_hash {
            panic_with_error!(&env, TipJarError::InvalidReveal);
        }

        let whitelisted: bool = env
            .storage()
            .instance()
            .get(&DataKey::TokenWhitelist(token.clone()))
            .unwrap_or(false);
        if !whitelisted {
            panic_with_error!(&env, TipJarError::TokenNotWhitelisted);
        }

        // Transfer tokens
        token::Client::new(&env, &token).transfer(&sender, &env.current_contract_address(), &amount);

        let fee_bp: u32 = env.storage().instance().get(&DataKey::FeeBasisPoints).unwrap_or(0);
        let fee: i128 = (amount * fee_bp as i128) / 10_000;
        let creator_amount = amount - fee;

        if fee > 0 {
            let fee_key = DataKey::PlatformFeeBalance(token.clone());
            let current_fee: i128 = env.storage().instance().get(&fee_key).unwrap_or(0);
            let new_fee_bal: i128 = current_fee
                .checked_add(fee)
                .expect("fee overflow");
            env.storage().instance().set(&fee_key, &new_fee_bal);
        }

        let bal_key = DataKey::CreatorBalance(private_tip.creator.clone(), token.clone());
        let existing_bal: i128 = env.storage().persistent().get(&bal_key).unwrap_or(0);
        let new_bal: i128 = existing_bal.checked_add(creator_amount).expect("balance overflow");
        env.storage().persistent().set(&bal_key, &new_bal);

        let tot_key = DataKey::CreatorTotal(private_tip.creator.clone(), token.clone());
        let existing_tot: i128 = env.storage().persistent().get(&tot_key).unwrap_or(0);
        let new_tot: i128 = existing_tot.checked_add(creator_amount).expect("total overflow");
        env.storage().persistent().set(&tot_key, &new_tot);

        private_tip.revealed = true;
        env.storage().persistent().set(&DataKey::PrivateTip(tip_id), &private_tip);
        env.storage().persistent().set(&DataKey::PrivateTipAmount(tip_id), &amount);

        env.events().publish(
            (symbol_short!("tip_rev"),),
            (tip_id, amount),
        );
    }

    /// Returns a private tip record by ID.
    pub fn get_private_tip(env: Env, tip_id: u64) -> privacy_tip::PrivateTip {
        env.storage()
            .persistent()
            .get(&DataKey::PrivateTip(tip_id))
            .unwrap_or_else(|| panic_with_error!(&env, TipJarError::PrivateTipNotFound))
    }

    /// Returns the revealed amount for a private tip (if revealed).
    pub fn get_private_tip_amount(env: Env, tip_id: u64) -> Option<i128> {
        env.storage()
            .persistent()
            .get(&DataKey::PrivateTipAmount(tip_id))
    }

    // ── streaming protocol ──────────────────────────────────────────────────────

    /// Creates a new stream from `sender` to `creator`.
    ///
    /// The stream will continuously transfer funds at `amount_per_second` until
    /// it is stopped, cancelled, or reaches its end time.
    ///
    /// Emits `("stream_created",)` with data `(stream_id, sender, creator, amount_per_second, duration)`.
    pub fn create_stream(
        env: Env,
        sender: Address,
        creator: Address,
        token: Address,
        amount_per_second: i128,
        duration: u64,
    ) -> u64 {
        Self::require_not_paused(&env);
        sender.require_auth();

        if amount_per_second <= 0 {
            panic_with_error!(&env, TipJarError::InvalidStreamRate);
        }

        // Maximum rate: 1000 tokens/second (adjust as needed)
        if amount_per_second > 1000 {
            panic_with_error!(&env, TipJarError::StrmRateMax);
        }

        if duration == 0 {
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

        let stream_id: u64 = env.storage().instance().get(&DataKey::StreamCounter).unwrap_or(0);
        let now = env.ledger().timestamp();
        let total_amount = amount_per_second * duration as i128;

        let stream = Stream {
            stream_id,
            sender: sender.clone(),
            creator: creator.clone(),
            token: token.clone(),
            amount_per_second,
            start_time: now,
            end_time: now + duration,
            withdrawn: 0,
            status: StreamStatus::Active,
            created_at: now,
            updated_at: now,
        };

        env.storage().persistent().set(&DataKey::Stream(stream_id), &stream);
        env.storage().instance().set(&DataKey::StreamCounter, &(stream_id + 1));

        // Add to sender's stream list
        let mut sender_streams: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::SenderStreams(sender.clone()))
            .unwrap_or_else(|| Vec::new(&env));
        sender_streams.push_back(stream_id);
        env.storage().persistent().set(&DataKey::SenderStreams(sender.clone()), &sender_streams);

        // Add to creator's stream list
        let mut creator_streams: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::CreatorStreams(creator.clone()))
            .unwrap_or_else(|| Vec::new(&env));
        creator_streams.push_back(stream_id);
        env.storage().persistent().set(&DataKey::CreatorStreams(creator.clone()), &creator_streams);

        // Transfer total amount into escrow
        token::Client::new(&env, &token).transfer(&sender, &env.current_contract_address(), &total_amount);

        env.events().publish(
            (symbol_short!("strm_cre"),),
            (stream_id, sender, creator, amount_per_second, duration),
        );

        stream_id
    }

    /// Calculates the amount that has been streamed up to the current time for a given stream.
    fn calculate_streamed_amount(env: &Env, stream: &Stream) -> i128 {
        let current_time = env.ledger().timestamp();

        if stream.status != StreamStatus::Active && stream.status != StreamStatus::Paused {
            return stream.withdrawn;
        }

        let elapsed = if current_time < stream.start_time {
            0
        } else if current_time > stream.end_time {
            stream.end_time - stream.start_time
        } else {
            current_time - stream.start_time
        };

        (stream.amount_per_second * elapsed as i128).min(
            stream.amount_per_second * (stream.end_time - stream.start_time) as i128
        )
    }

    /// Starts a stream (or resumes a paused stream).
    ///
    /// Only the sender can start/activate a stream.
    /// Emits `("stream_started",)` with data `(stream_id)`.
    pub fn start_stream(env: Env, sender: Address, stream_id: u64) {
        Self::require_not_paused(&env);
        sender.require_auth();

        let mut stream: Stream = env
            .storage()
            .persistent()
            .get(&DataKey::Stream(stream_id))
            .unwrap_or_else(|| panic_with_error!(&env, TipJarError::StreamNotFound));

        if stream.sender != sender {
            panic_with_error!(&env, TipJarError::Unauthorized);
        }

        if stream.status == StreamStatus::Cancelled {
            panic_with_error!(&env, TipJarError::StreamAlreadyCancelled);
        }

        if stream.status == StreamStatus::Completed {
            panic_with_error!(&env, TipJarError::StreamAlreadyCompleted);
        }

        let now = env.ledger().timestamp();

        // If starting from scratch, set start_time
        if stream.status == StreamStatus::Paused {
            // Resume from paused state
            let pause_duration = now - stream.updated_at;
            stream.start_time += pause_duration;
            stream.end_time += pause_duration;
        }

        stream.status = StreamStatus::Active;
        stream.updated_at = now;

        env.storage().persistent().set(&DataKey::Stream(stream_id), &stream);

        env.events().publish(
            (symbol_short!("strm_sta"),),
            stream_id,
        );
    }

    /// Stops (pauses) an active stream.
    ///
    /// Only the sender can stop a stream.
    /// Emits `("stream_stopped",)` with data `(stream_id, streamed_amount)`.
    pub fn stop_stream(env: Env, sender: Address, stream_id: u64) {
        Self::require_not_paused(&env);
        sender.require_auth();

        let mut stream: Stream = env
            .storage()
            .persistent()
            .get(&DataKey::Stream(stream_id))
            .unwrap_or_else(|| panic_with_error!(&env, TipJarError::StreamNotFound));

        if stream.sender != sender {
            panic_with_error!(&env, TipJarError::Unauthorized);
        }

        if stream.status != StreamStatus::Active {
            panic_with_error!(&env, TipJarError::StreamNotStarted);
        }

        if stream.status == StreamStatus::Cancelled {
            panic_with_error!(&env, TipJarError::StreamAlreadyCancelled);
        }

        let streamed_amount = Self::calculate_streamed_amount(&env, &stream);
        stream.status = StreamStatus::Paused;
        stream.withdrawn = streamed_amount;
        stream.updated_at = env.ledger().timestamp();

        env.storage().persistent().set(&DataKey::Stream(stream_id), &stream);

        env.events().publish(
            (symbol_short!("strm_sto"),),
            (stream_id, streamed_amount),
        );
    }

    /// Withdraws the currently streamed amount for a stream.
    ///
    /// The creator can withdraw the amount that has been streamed up to now.
    /// Emits `("stream_withdrawn",)` with data `(stream_id, amount, creator)`.
    pub fn withdraw_streamed(env: Env, creator: Address, stream_id: u64) {
        Self::require_not_paused(&env);
        creator.require_auth();

        let mut stream: Stream = env
            .storage()
            .persistent()
            .get(&DataKey::Stream(stream_id))
            .unwrap_or_else(|| panic_with_error!(&env, TipJarError::StreamNotFound));

        if stream.creator != creator {
            panic_with_error!(&env, TipJarError::Unauthorized);
        }

        if stream.status == StreamStatus::Cancelled {
            panic_with_error!(&env, TipJarError::StreamAlreadyCancelled);
        }

        let current_time = env.ledger().timestamp();

        if current_time < stream.start_time {
            panic_with_error!(&env, TipJarError::StreamNotStarted);
        }

        let total_streamable = stream.amount_per_second * (stream.end_time - stream.start_time) as i128;
        let streamed_amount = Self::calculate_streamed_amount(&env, &stream);
        let available_to_withdraw = streamed_amount - stream.withdrawn;

        if available_to_withdraw <= 0 {
            panic_with_error!(&env, TipJarError::NoStreamedAmount);
        }

        // Update stream state BEFORE external call
        stream.withdrawn = streamed_amount;

        // Check if stream is completed
        if current_time >= stream.end_time {
            stream.status = StreamStatus::Completed;
        }

        stream.updated_at = current_time;

        env.storage().persistent().set(&DataKey::Stream(stream_id), &stream);

        // Transfer tokens to creator
        token::Client::new(&env, &stream.token).transfer(
            &env.current_contract_address(),
            &creator,
            &available_to_withdraw,
        );

        env.events().publish(
            (symbol_short!("strm_wit"),),
            (stream_id, available_to_withdraw, creator),
        );
    }

    /// Cancels an active stream and refunds the remaining tokens to the sender.
    ///
    /// Only the sender can cancel a stream.
    /// Emits `("stream_cancelled",)` with data `(stream_id, refunded_amount)`.
    pub fn cancel_stream(env: Env, sender: Address, stream_id: u64) {
        Self::require_not_paused(&env);
        sender.require_auth();

        let mut stream: Stream = env
            .storage()
            .persistent()
            .get(&DataKey::Stream(stream_id))
            .unwrap_or_else(|| panic_with_error!(&env, TipJarError::StreamNotFound));

        if stream.sender != sender {
            panic_with_error!(&env, TipJarError::Unauthorized);
        }

        if stream.status == StreamStatus::Cancelled {
            panic_with_error!(&env, TipJarError::StreamAlreadyCancelled);
        }

        if stream.status == StreamStatus::Completed {
            panic_with_error!(&env, TipJarError::StreamAlreadyCompleted);
        }

        let current_time = env.ledger().timestamp();
        let streamed_amount = Self::calculate_streamed_amount(&env, &stream);

        // Calculate total amount that was put into escrow
        let total_amount = stream.amount_per_second * (stream.end_time - stream.start_time) as i128;
        let remaining_amount = total_amount - streamed_amount;

        // Mark stream as cancelled
        stream.status = StreamStatus::Cancelled;
        stream.updated_at = current_time;

        env.storage().persistent().set(&DataKey::Stream(stream_id), &stream);

        // Refund remaining tokens to sender
        if remaining_amount > 0 {
            token::Client::new(&env, &stream.token).transfer(
                &env.current_contract_address(),
                &sender,
                &remaining_amount,
            );
        }

        // If there's any withdrawn amount not yet claimed, it's already in the creator's balance
        // (handled by the periodic withdraw_streamed calls)

        env.events().publish(
            (symbol_short!("strm_can"),),
            (stream_id, remaining_amount),
        );
    }

    /// Returns the current stream details.
    pub fn get_stream(env: Env, stream_id: u64) -> Option<Stream> {
        env.storage().persistent().get(&DataKey::Stream(stream_id))
    }

    /// Returns all stream IDs for a creator.
    pub fn get_streams_by_creator(env: Env, creator: Address) -> Vec<u64> {
        env.storage()
            .persistent()
            .get(&DataKey::CreatorStreams(creator))
            .unwrap_or_else(|| Vec::new(&env))
    }

    /// Returns all stream IDs for a sender.
    pub fn get_streams_by_sender(env: Env, sender: Address) -> Vec<u64> {
        env.storage()
            .persistent()
            .get(&DataKey::SenderStreams(sender))
            .unwrap_or_else(|| Vec::new(&env))
    }

    /// Returns the current streamed amount for a stream.
    pub fn get_streamed_amount(env: Env, stream_id: u64) -> i128 {
        let stream: Stream = env
            .storage()
            .persistent()
            .get(&DataKey::Stream(stream_id))
            .unwrap_or_else(|| panic_with_error!(&env, TipJarError::StreamNotFound));

        Self::calculate_streamed_amount(&env, &stream)
    }

    /// Returns the available amount to withdraw for a stream.
    pub fn get_available_to_withdraw(env: Env, stream_id: u64) -> i128 {
        let stream: Stream = env
            .storage()
            .persistent()
            .get(&DataKey::Stream(stream_id))
            .unwrap_or_else(|| panic_with_error!(&env, TipJarError::StreamNotFound));

        if stream.status == StreamStatus::Cancelled || stream.status == StreamStatus::Completed {
            return 0;
        }

        let current_time = env.ledger().timestamp();
        if current_time < stream.start_time {
            return 0;
        }

        let streamed_amount = Self::calculate_streamed_amount(&env, &stream);
        streamed_amount - stream.withdrawn
    }

    /// Initialize the insurance pool configuration for the contract.
    ///
    /// Admin only. Sets up insurance pool parameters.
    /// Emits `("insurance_config_set",)` with data `(min_contrib, max_contrib, premium_rate, payout_ratio)`.
    pub fn insurance_set_config(
        env: Env,
        admin: Address,
        min_contribution: i128,
        max_contribution: i128,
        premium_rate_bps: u32,
        payout_ratio_bps: u32,
        claim_cooldown: u64,
        admin_fee_bps: u32,
        tip_premium_bps: u32,
    ) {
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic_with_error!(&env, TipJarError::Unauthorized);
        }

        if min_contribution < 0 || max_contribution <= min_contribution {
            panic_with_error!(&env, TipJarError::InvalidAmount);
        }
        if premium_rate_bps > 500 {
            panic_with_error!(&env, TipJarError::FeeExceedsMaximum);
        }
        if payout_ratio_bps > 10000 {
            panic_with_error!(&env, TipJarError::InvalidAmount);
        }
        if tip_premium_bps > 1000 {
            panic_with_error!(&env, TipJarError::FeeExceedsMaximum);
        }

        let config = InsurancePoolConfig {
            min_contribution,
            max_contribution,
            premium_rate_bps,
            payout_ratio_bps,
            claim_cooldown,
            admin_fee_bps,
            tip_premium_bps,
        };
        env.storage().instance().set(&DataKey::InsPoolCfg, &config);

        env.events().publish(
            (symbol_short!("ins_cfg"),),
            (min_contribution, max_contribution, premium_rate_bps, payout_ratio_bps, tip_premium_bps),
        );
    }

    /// Enable or disable the insurance feature.
    ///
    /// Admin only.
    pub fn insurance_set_enabled(env: Env, admin: Address, enabled: bool) {
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic_with_error!(&env, TipJarError::Unauthorized);
        }
        env.storage().instance().set(&DataKey::InsEnabled, &enabled);
        env.events().publish((symbol_short!("ins_en"),), enabled);
    }

    /// Set the maximum number of active claims a creator can have simultaneously.
    ///
    /// Admin only.
    pub fn insurance_set_max_active_claims(env: Env, admin: Address, max_claims: u32) {
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic_with_error!(&env, TipJarError::Unauthorized);
        }
        if max_claims == 0 {
            panic_with_error!(&env, TipJarError::InvalidAmount);
        }
        env.storage().instance().set(&DataKey::InsMaxClms, &max_claims);
        env.events().publish((symbol_short!("ins_max"),), max_claims);
    }

    /// Set the insurance admin address.
    ///
    /// Admin only.
    pub fn insurance_set_admin(env: Env, admin: Address, insurance_admin: Address) {
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic_with_error!(&env, TipJarError::Unauthorized);
        }
        env.storage().instance().set(&DataKey::InsAdmin, &insurance_admin);
        env.events().publish((symbol_short!("ins_adm"),), insurance_admin);
    }

    /// Contribute to the insurance pool for a specific token.
    ///
    /// Creator can contribute funds to gain insurance coverage. The contribution amount
    /// must be within configured limits. The sender must transfer the tokens to this contract.
    ///
    /// Emits `("insurance_contribution",)` with data `(creator, token, amount, coverage_amount)`.
    pub fn insurance_contribute(
        env: Env,
        creator: Address,
        token: Address,
        amount: i128,
    ) {
        Self::require_not_paused(&env);
        creator.require_auth();

        // Check if insurance is enabled
        let enabled: bool = env.storage().instance().get(&DataKey::InsEnabled).unwrap_or(true);
        if !enabled {
            panic_with_error!(&env, TipJarError::InsuranceDisabled);
        }

        let config: InsurancePoolConfig = env
            .storage()
            .instance()
            .get(&DataKey::InsPoolCfg)
            .unwrap_or_else(|| panic_with_error!(&env, TipJarError::InsPoolNotCfg));

        if amount < config.min_contribution {
            panic_with_error!(&env, TipJarError::ContributionTooLow);
        }
        if amount > config.max_contribution {
            panic_with_error!(&env, TipJarError::ContributionTooHigh);
        }

        // Check whitelist
        let whitelisted: bool = env
            .storage()
            .instance()
            .get(&DataKey::TokenWhitelist(token.clone()))
            .unwrap_or(false);
        if !whitelisted {
            panic_with_error!(&env, TipJarError::TokenNotWhitelisted);
        }

        // Transfer tokens from creator to contract
        token::Client::new(&env, &token).transfer(&creator, &env.current_contract_address(), &amount);

        // Calculate premium for this contribution period
        let premium_amount = (amount * config.premium_rate_bps as i128) / 10_000;

        // Update pool state
        let pool_key = DataKey::InsPoolToken(token.clone());
        let mut pool: InsurancePool = env
            .storage()
            .persistent()
            .get(&pool_key)
            .unwrap_or_else(|| InsurancePool {
                token: token.clone(),
                total_reserves: 0,
                total_contributions: 0,
                total_claims_paid: 0,
                active_claims: 0,
                total_claims: 0,
                last_payout_time: env.ledger().timestamp(),
            });

        pool.total_reserves += amount - premium_amount;
        pool.total_contributions += amount;
        env.storage().persistent().set(&pool_key, &pool);

        // Update creator contribution
        let contrib_key = DataKey::InsContrib(creator.clone(), token.clone());
        let current_contrib: i128 = env.storage().persistent().get(&contrib_key).unwrap_or(0);
        env.storage().persistent().set(&contrib_key, &(current_contrib + amount));

        // Add to creator's token list
        let tokens_key = DataKey::CreatorTokens(creator.clone());
        let mut tokens: Vec<Address> = env.storage().persistent().get(&tokens_key).unwrap_or_else(|| Vec::new(&env));
        if !tokens.contains(&token) {
            tokens.push_back(token.clone());
            env.storage().persistent().set(&tokens_key, &tokens);
        }

        // Calculate and add to platform fee balance
        if premium_amount > 0 {
            let fee_key = DataKey::PlatformFeeBalance(token.clone());
            let current_fee: i128 = env.storage().instance().get(&fee_key).unwrap_or(0);
            env.storage().instance().set(&fee_key, &(current_fee + premium_amount));
        }

        env.events().publish(
            (symbol_short!("ins_con"),),
            (creator.clone(), token, amount, pool.total_reserves),
        );
    }

    /// Submit an insurance claim for a failed transaction.
    ///
    /// A creator can submit a claim when they experience a failed transaction
    /// (e.g., failed tip, failed withdrawal). The claim must include proof
    /// (transaction hash) and will be subject to review.
    ///
    /// Emits `("claim_submitted",)` with data `(claim_id, creator, token, amount)`.
    pub fn insurance_submit_claim(
        env: Env,
        creator: Address,
        token: Address,
        amount: i128,
        tx_hash: BytesN<32>,
    ) -> u64 {
        Self::require_not_paused(&env);
        creator.require_auth();

        if amount <= 0 {
            panic_with_error!(&env, TipJarError::InvalidClaimAmount);
        }

        // Check if insurance is enabled
        let enabled: bool = env.storage().instance().get(&DataKey::InsEnabled).unwrap_or(true);
        if !enabled {
            panic_with_error!(&env, TipJarError::InsuranceDisabled);
        }

        // Check if pool is configured
        let config: InsurancePoolConfig = env
            .storage()
            .instance()
            .get(&DataKey::InsPoolCfg)
            .unwrap_or_else(|| panic_with_error!(&env, TipJarError::InsPoolNotCfg));

        // Check creator has coverage
        let max_payout = Self::insurance_get_coverage(env.clone(), creator.clone(), token.clone());
        if max_payout <= 0 {
            panic_with_error!(&env, TipJarError::NoCoverage);
        }

        // Check active claim limit
        let max_active: u32 = env.storage().instance().get(&DataKey::InsMaxClms).unwrap_or(3);
        let active_key = DataKey::InsActiveClms(creator.clone(), token.clone());
        let active_claims: u32 = env.storage().persistent().get(&active_key).unwrap_or(0);
        if active_claims >= max_active {
            panic_with_error!(&env, TipJarError::TooManyActiveClaims);
        }

        // Check last claim cooldown
        let last_claim_key = DataKey::InsLastClm(creator.clone(), token.clone());
        let last_claim: u64 = env.storage().persistent().get(&last_claim_key).unwrap_or(0);
        let now = env.ledger().timestamp();
        if last_claim > 0 && now < last_claim + config.claim_cooldown {
            panic_with_error!(&env, TipJarError::ClaimCooldownActive);
        }

        // Check pool has sufficient reserves
        let pool_key = DataKey::InsPoolToken(token.clone());
        let pool: InsurancePool = env
            .storage()
            .persistent()
            .get(&pool_key)
            .unwrap_or_else(|| panic_with_error!(&env, TipJarError::InsufficientReserves));

        if amount > max_payout {
            panic_with_error!(&env, TipJarError::PayoutExceedsReserves);
        }

        if amount > pool.total_reserves {
            panic_with_error!(&env, TipJarError::InsufficientReserves);
        }

        // Create claim
        let claim_id: u64 = env.storage().instance().get(&DataKey::InsClaimCtr).unwrap_or(0);
        env.storage().instance().set(&DataKey::InsClaimCtr, &(claim_id + 1));

        let claim = InsuranceClaim {
            claim_id,
            creator: creator.clone(),
            token: token.clone(),
            amount,
            tx_hash,
            status: ClaimStatus::Pending,
            created_at: now,
            updated_at: now,
            last_claim_at: last_claim,
        };
        env.storage().persistent().set(&DataKey::InsClaim(claim_id), &claim);

        // Add to creator's claims list
        let creator_claims_key = DataKey::InsClms(creator.clone(), token.clone());
        let mut creator_claims: Vec<u64> = env.storage().persistent().get(&creator_claims_key).unwrap_or_else(|| Vec::new(&env));
        creator_claims.push_back(claim_id);
        env.storage().persistent().set(&creator_claims_key, &creator_claims);

        // Update active claim count
        env.storage().persistent().set(&active_key, &(active_claims + 1));

        // Update total claims count
        let total_claims_key = DataKey::InsTotalClms(creator.clone(), token.clone());
        let total_claims: u32 = env.storage().persistent().get(&total_claims_key).unwrap_or(0);
        env.storage().persistent().set(&total_claims_key, &(total_claims + 1));

        // Update pool
        let mut updated_pool = pool.clone();
        updated_pool.active_claims += 1;
        updated_pool.total_claims += 1;
        env.storage().persistent().set(&pool_key, &updated_pool);

        env.events().publish(
            (symbol_short!("clm_sub"),),
            (claim_id, creator, token, amount),
        );

        claim_id
    }

    /// Approve an insurance claim (admin or insurance admin).
    ///
    /// Only the contract admin or insurance admin can approve claims.
    /// Once approved, the claim can be paid out.
    ///
    /// Emits `("claim_approved",)` with data `(claim_id, approver)`.
    pub fn insurance_approve_claim(
        env: Env,
        approver: Address,
        claim_id: u64,
    ) {
        approver.require_auth();

        // Check if caller is admin or insurance admin
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        let insurance_admin: Address = env.storage().instance().get(&DataKey::InsAdmin).unwrap_or(stored_admin.clone());
        if approver != stored_admin && approver != insurance_admin {
            panic_with_error!(&env, TipJarError::AdmAppReq);
        }

        let claim: InsuranceClaim = env
            .storage()
            .persistent()
            .get(&DataKey::InsClaim(claim_id))
            .unwrap_or_else(|| panic_with_error!(&env, TipJarError::ClaimNotFound));

        if claim.status != ClaimStatus::Pending {
            panic_with_error!(&env, TipJarError::ClaimNotApproved);
        }

        let mut updated_claim = claim.clone();
        updated_claim.status = ClaimStatus::Approved;
        updated_claim.updated_at = env.ledger().timestamp();
        env.storage().persistent().set(&DataKey::InsClaim(claim_id), &updated_claim);

        // Update pool
        let pool_key = DataKey::InsPoolToken(claim.token.clone());
        let pool: InsurancePool = env.storage().persistent().get(&pool_key).unwrap();
        let mut updated_pool = pool.clone();
        updated_pool.active_claims -= 1;
        env.storage().persistent().set(&pool_key, &updated_pool);

        // Update creator active claims
        let active_key = DataKey::InsActiveClms(claim.creator.clone(), claim.token.clone());
        let active_claims: u32 = env.storage().persistent().get(&active_key).unwrap_or(1);
        env.storage().persistent().set(&active_key, &(active_claims - 1));

        env.events().publish(
            (symbol_short!("clm_app"),),
            (claim_id, approver),
        );
    }

    /// Reject an insurance claim (admin or insurance admin).
    ///
    /// Only the contract admin or insurance admin can reject claims.
    ///
    /// Emits `("claim_rejected",)` with data `(claim_id, rejector)`.
    pub fn insurance_reject_claim(
        env: Env,
        rejector: Address,
        claim_id: u64,
    ) {
        rejector.require_auth();

        // Check if caller is admin or insurance admin
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        let insurance_admin: Address = env.storage().instance().get(&DataKey::InsAdmin).unwrap_or(stored_admin.clone());
        if rejector != stored_admin && rejector != insurance_admin {
            panic_with_error!(&env, TipJarError::AdmAppReq);
        }

        let claim: InsuranceClaim = env
            .storage()
            .persistent()
            .get(&DataKey::InsClaim(claim_id))
            .unwrap_or_else(|| panic_with_error!(&env, TipJarError::ClaimNotFound));

        if claim.status != ClaimStatus::Pending && claim.status != ClaimStatus::Approved {
            panic_with_error!(&env, TipJarError::ClaimNotApproved);
        }

        let mut updated_claim = claim.clone();
        updated_claim.status = ClaimStatus::Rejected;
        updated_claim.updated_at = env.ledger().timestamp();
        env.storage().persistent().set(&DataKey::InsClaim(claim_id), &updated_claim);

        if claim.status == ClaimStatus::Approved {
            // Update pool
            let pool_key = DataKey::InsPoolToken(claim.token.clone());
            let pool: InsurancePool = env.storage().persistent().get(&pool_key).unwrap();
            let mut updated_pool = pool.clone();
            updated_pool.active_claims -= 1;
            env.storage().persistent().set(&pool_key, &updated_pool);

            // Update creator active claims
            let active_key = DataKey::InsActiveClms(claim.creator.clone(), claim.token.clone());
            let active_claims: u32 = env.storage().persistent().get(&active_key).unwrap_or(1);
            env.storage().persistent().set(&active_key, &(active_claims - 1));
        }

        env.events().publish(
            (symbol_short!("clm_rej"),),
            (claim_id, rejector),
        );
    }

    /// Pay out an approved insurance claim.
    ///
    /// Transfers funds from the insurance pool to the creator.
    /// Can only be called for approved claims that haven't been paid yet.
    ///
    /// Emits `("claim_paid",)` with data `(claim_id, amount, creator)`.
    pub fn insurance_pay_claim(
        env: Env,
        caller: Address,
        claim_id: u64,
    ) {
        caller.require_auth();

        // Check if caller is admin or insurance admin
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        let insurance_admin: Address = env.storage().instance().get(&DataKey::InsAdmin).unwrap_or(stored_admin.clone());
        if caller != stored_admin && caller != insurance_admin {
            panic_with_error!(&env, TipJarError::AdmAppReq);
        }

        let claim: InsuranceClaim = env
            .storage()
            .persistent()
            .get(&DataKey::InsClaim(claim_id))
            .unwrap_or_else(|| panic_with_error!(&env, TipJarError::ClaimNotFound));

        if claim.status != ClaimStatus::Approved {
            panic_with_error!(&env, TipJarError::ClaimNotApproved);
        }

        // Check pool has sufficient reserves
        let pool_key = DataKey::InsPoolToken(claim.token.clone());
        let pool: InsurancePool = env
            .storage()
            .persistent()
            .get(&pool_key)
            .unwrap_or_else(|| panic_with_error!(&env, TipJarError::InsufficientReserves));

        if claim.amount > pool.total_reserves {
            panic_with_error!(&env, TipJarError::InsufficientReserves);
        }

        // Update creator's contribution (deduct claim amount)
        let contrib_key = DataKey::InsContrib(claim.creator.clone(), claim.token.clone());
        let current_contrib: i128 = env.storage().persistent().get(&contrib_key).unwrap_or(0);
        let new_contrib = current_contrib - claim.amount;
        env.storage().persistent().set(&contrib_key, &(new_contrib.max(0)));

        // Update pool reserves
        let mut updated_pool = pool.clone();
        updated_pool.total_reserves -= claim.amount;
        updated_pool.total_claims_paid += claim.amount;
        updated_pool.active_claims -= 1;
        updated_pool.last_payout_time = env.ledger().timestamp();
        env.storage().persistent().set(&pool_key, &updated_pool);

        // Update claim status
        let mut updated_claim = claim.clone();
        updated_claim.status = ClaimStatus::Paid;
        updated_claim.updated_at = env.ledger().timestamp();
        env.storage().persistent().set(&DataKey::InsClaim(claim_id), &updated_claim);

        // Update creator's last claim time and active claims
        let last_claim_key = DataKey::InsLastClm(claim.creator.clone(), claim.token.clone());
        env.storage().persistent().set(&last_claim_key, &(env.ledger().timestamp()));

        let active_key = DataKey::InsActiveClms(claim.creator.clone(), claim.token.clone());
        let active_claims: u32 = env.storage().persistent().get(&active_key).unwrap_or(1);
        env.storage().persistent().set(&active_key, &(active_claims - 1));

        // Transfer funds to creator
        token::Client::new(&env, &claim.token).transfer(
            &env.current_contract_address(),
            &claim.creator,
            &claim.amount,
        );

        env.events().publish(
            (symbol_short!("clm_paid"),),
            (claim_id, claim.amount, claim.creator),
        );
    }

    /// Get the insurance pool configuration.
    pub fn insurance_get_config(env: Env) -> InsurancePoolConfig {
        env.storage()
            .instance()
            .get(&DataKey::InsPoolCfg)
            .unwrap_or_else(|| panic_with_error!(&env, TipJarError::InsPoolNotCfg))
    }

    /// Check if the insurance feature is enabled.
    pub fn insurance_is_enabled(env: Env) -> bool {
        env.storage().instance().get(&DataKey::InsEnabled).unwrap_or(true)
    }

    /// Get the insurance pool state for a specific token.
    pub fn insurance_get_pool(env: Env, token: Address) -> Option<InsurancePool> {
        env.storage().persistent().get(&DataKey::InsPoolToken(token))
    }

    /// Get a specific insurance claim by ID.
    pub fn insurance_get_claim(env: Env, claim_id: u64) -> InsuranceClaim {
        env.storage()
            .persistent()
            .get(&DataKey::InsClaim(claim_id))
            .unwrap_or_else(|| panic_with_error!(&env, TipJarError::ClaimNotFound))
    }

    /// Get creator's contribution amount for a specific token.
    pub fn insurance_get_contribution(env: Env, creator: Address, token: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::InsContrib(creator, token))
            .unwrap_or(0)
    }

    /// Get creator's coverage limit based on their contribution and tips received.
    pub fn insurance_get_coverage(env: Env, creator: Address, token: Address) -> i128 {
        let config: InsurancePoolConfig = env
            .storage()
            .instance()
            .get(&DataKey::InsPoolCfg)
            .unwrap_or_else(|| panic_with_error!(&env, TipJarError::InsPoolNotCfg));

        // Manual contribution coverage
        let contrib: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::InsContrib(creator.clone(), token.clone()))
            .unwrap_or(0);

        // Automatic premium coverage estimate from tips received
        let total_received: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::CreatorTotal(creator, token))
            .unwrap_or(0);
        
        // Since CreatorTotal is net of fees and premiums, we approximate the original gross 
        // to find the premium paid. Gross = Net / (1 - fee_bps - premium_bps)
        // For simplicity, we use Net * premium_bps as a conservative estimate of coverage earned.
        let premium_earned = (total_received * config.tip_premium_bps as i128) / 10_000;

        ((contrib + premium_earned) * config.payout_ratio_bps as i128) / 10_000
    }

    /// Get creator's active claim count for a specific token.
    pub fn insurance_get_active_claims(env: Env, creator: Address, token: Address) -> u32 {
        env.storage()
            .persistent()
            .get(&DataKey::InsActiveClms(creator, token))
            .unwrap_or(0)
    }

    /// Get creator's total claim count for a specific token.
    pub fn insurance_get_total_claims(env: Env, creator: Address, token: Address) -> u32 {
        env.storage()
            .persistent()
            .get(&DataKey::InsTotalClms(creator, token))
            .unwrap_or(0)
    }

    /// Check if insurance is available for a creator/token combination.
    pub fn insurance_has_coverage(env: Env, creator: Address, token: Address) -> bool {
        let contrib: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::InsContrib(creator, token))
            .unwrap_or(0);
        contrib > 0
    }

    /// Withdraw excess funds from the insurance pool (admin only).
    ///
    /// Allows admin to withdraw funds beyond a minimum reserve threshold.
    /// Emits `("pool_withdraw",)` with data `(token, amount)`.
    pub fn insurance_withdraw_excess(
        env: Env,
        admin: Address,
        token: Address,
        amount: i128,
    ) {
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic_with_error!(&env, TipJarError::Unauthorized);
        }

        if amount <= 0 {
            panic_with_error!(&env, TipJarError::InvalidAmount);
        }

        let pool_key = DataKey::InsPoolToken(token.clone());
        let pool: InsurancePool = env
            .storage()
            .persistent()
            .get(&pool_key)
            .unwrap_or_else(|| panic_with_error!(&env, TipJarError::InsufficientReserves));

        // Keep minimum reserve (10% of total contributions)
        let min_reserve = pool.total_contributions / 10;
        if pool.total_reserves - amount < min_reserve {
            panic_with_error!(&env, TipJarError::InsufficientReserves);
        }

        let mut updated_pool = pool.clone();
        updated_pool.total_reserves -= amount;
        env.storage().persistent().set(&pool_key, &updated_pool);

        // Transfer to admin
        token::Client::new(&env, &token).transfer(
            &env.current_contract_address(),
            &admin,
            &amount,
        );

        env.events().publish(
            (symbol_short!("pol_wit"),),
            (token, amount),
        );
    }

    /// Get the insurance admin address.
    pub fn insurance_get_admin(env: Env) -> Address {
        env.storage().instance()
            .get(&DataKey::InsAdmin)
            .unwrap_or_else(|| env.storage().instance().get(&DataKey::Admin).unwrap())
    }

    /// Get the maximum active claims per creator.
    pub fn insurance_get_max_active_claims(env: Env) -> u32 {
        env.storage().instance().get(&DataKey::InsMaxClms).unwrap_or(3)
    }

    /// Process multiple insurance claims in batch (admin only).
    ///
    /// Allows efficient approval/payment of multiple claims at once.
    /// Emits `("claims_processed",)` with data `(approved_count, paid_count)`.
    pub fn insurance_process_claims_batch(
        env: Env,
        admin: Address,
        claim_ids: Vec<u64>,
        action: String,
    ) {
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic_with_error!(&env, TipJarError::Unauthorized);
        }

        let mut approved_count: u32 = 0;
        let mut paid_count: u32 = 0;

        for i in 0..claim_ids.len() {
            let claim_id = claim_ids.get(i).unwrap();
            let claim: InsuranceClaim = env
                .storage()
                .persistent()
                .get(&DataKey::InsClaim(claim_id))
                .unwrap_or_else(|| panic_with_error!(&env, TipJarError::ClaimNotFound));

            if action == String::from_str(&env, "approve") {
                if claim.status == ClaimStatus::Pending {
                    let mut updated_claim = claim.clone();
                    updated_claim.status = ClaimStatus::Approved;
                    updated_claim.updated_at = env.ledger().timestamp();
                    env.storage().persistent().set(&DataKey::InsClaim(claim_id), &updated_claim);

                    // Update pool active claims
                    let pool_key = DataKey::InsPoolToken(claim.token.clone());
                    let pool: InsurancePool = env.storage().persistent().get(&pool_key).unwrap();
                    let mut updated_pool = pool.clone();
                    updated_pool.active_claims -= 1;
                    env.storage().persistent().set(&pool_key, &updated_pool);

                    // Update creator active claims
                    let active_key = DataKey::InsActiveClms(claim.creator.clone(), claim.token.clone());
                    let active_claims: u32 = env.storage().persistent().get(&active_key).unwrap_or(1);
                    env.storage().persistent().set(&active_key, &(active_claims - 1));

                    approved_count += 1;
                }
            } else if action == String::from_str(&env, "pay") {
                if claim.status == ClaimStatus::Approved {
                    let pool_key = DataKey::InsPoolToken(claim.token.clone());
                    let pool: InsurancePool = env.storage().persistent().get(&pool_key).unwrap();

                    if claim.amount <= pool.total_reserves {
                        let mut updated_pool = pool.clone();
                        updated_pool.total_reserves -= claim.amount;
                        updated_pool.total_claims_paid += claim.amount;
                        updated_pool.active_claims -= 1;
                        updated_pool.last_payout_time = env.ledger().timestamp();
                        env.storage().persistent().set(&pool_key, &updated_pool);

                        let mut updated_claim = claim.clone();
                        updated_claim.status = ClaimStatus::Paid;
                        updated_claim.updated_at = env.ledger().timestamp();
                        env.storage().persistent().set(&DataKey::InsClaim(claim_id), &updated_claim);

                        // Update creator last claim time
                        let last_claim_key = DataKey::InsLastClm(claim.creator.clone(), claim.token.clone());
                        env.storage().persistent().set(&last_claim_key, &(env.ledger().timestamp()));

                        // Update creator active claims
                        let active_key = DataKey::InsActiveClms(claim.creator.clone(), claim.token.clone());
                        let active_claims: u32 = env.storage().persistent().get(&active_key).unwrap_or(1);
                        env.storage().persistent().set(&active_key, &(active_claims - 1));

                        // Transfer funds
                        token::Client::new(&env, &claim.token).transfer(
                            &env.current_contract_address(),
                            &claim.creator,
                            &claim.amount,
                        );

                        paid_count += 1;
                    }
                }
            }
        }

        env.events().publish(
            (symbol_short!("clm_pro"),),
            (approved_count, paid_count),
        );
    }

    /// Get all insurance claims for a specific creator and token.
    ///
    /// Returns a vector of claim IDs for the creator's claims.
    pub fn insurance_get_claims_by_creator(
        env: Env,
        creator: Address,
        token: Address,
    ) -> Vec<u64> {
        env.storage()
            .persistent()
            .get(&DataKey::InsClms(creator, token))
            .unwrap_or_else(|| Vec::new(&env))
    }

    // ── cross-chain bridge ───────────────────────────────────────────────────

    /// Sets the authorized bridge relayer and bridge token. Admin only.
    ///
    /// Emits `("bridge_cfg",)` with data `(relayer, token)`.
    pub fn set_bridge_relayer(env: Env, admin: Address, relayer: Address, token: Address) {
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic_with_error!(&env, TipJarError::Unauthorized);
        }
        env.storage().instance().set(&DataKey::BridgeRelayer, &relayer);
        env.storage().instance().set(&DataKey::BridgeToken, &token);
        env.storage().instance().set(&DataKey::BridgeEnabled, &true);
        env.events().publish(
            (symbol_short!("br_cfg"),),
            (relayer, token),
        );
    }

    /// Processes a bridged tip submitted by an authorized relayer.
    ///
    /// Validates the tip, deducts bridge fees, transfers funds from the relayer
    /// into contract escrow, and credits the creator's balance.
    /// Emits `("bridge", creator)` with data `(source_chain, source_tx_hash, amount, fee)`.
    pub fn bridge_tip(env: Env, relayer: Address, tip: bridge::BridgeTip) {
        Self::require_not_paused(&env);
        if let Err(e) = bridge::relayer::process_bridge_tip(&env, &relayer, &tip) {
            panic_with_error!(&env, e);
        }
    }

    /// Sets the bridge fee in basis points. Admin only.
    ///
    /// Maximum fee is 500 bps (5%).
    /// Emits `("bridge_fee",)` with data `fee_bps`.
    pub fn set_bridge_fee(env: Env, admin: Address, fee_bps: u32) {
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic_with_error!(&env, TipJarError::Unauthorized);
        }
        if fee_bps > 500 {
            panic_with_error!(&env, TipJarError::InvalidBridgeFee);
        }
        env.storage().instance().set(&DataKey::BridgeFeeBps, &fee_bps);
        env.events().publish((symbol_short!("br_fee"),), fee_bps);
    }

    /// Returns the current bridge fee in basis points.
    pub fn get_bridge_fee(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::BridgeFeeBps)
            .unwrap_or(0)
    }

    /// Enables or disables the bridge feature. Admin only.
    ///
    /// Emits `("bridge_en",)` with data `enabled`.
    pub fn enable_bridge(env: Env, admin: Address, enabled: bool) {
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic_with_error!(&env, TipJarError::Unauthorized);
        }
        env.storage().instance().set(&DataKey::BridgeEnabled, &enabled);
        env.events().publish((symbol_short!("br_en"),), enabled);
    }

    /// Returns `true` if the bridge feature is enabled.
    pub fn is_bridge_enabled(env: Env) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::BridgeEnabled)
            .unwrap_or(false)
    }

    // ── options trading ──────────────────────────────────────────────────────

    /// Initialize options trading system with default pricing parameters.
    ///
    /// Admin only. Emits `("opt_init",)`.
    pub fn init_options_trading(env: Env, admin: Address) {
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic_with_error!(&env, TipJarError::Unauthorized);
        }

        let params = options::PricingParams {
            volatility_bps: options::DEFAULT_VOLATILITY_BPS,
            risk_free_rate_bps: options::DEFAULT_RISK_FREE_RATE_BPS,
            min_premium_bps: options::DEFAULT_MIN_PREMIUM_BPS,
            max_premium_bps: options::DEFAULT_MAX_PREMIUM_BPS,
        };

        env.storage()
            .persistent()
            .set(&DataKey::OptionPricingParams, &params);
        
        env.storage()
            .instance()
            .set(&DataKey::OptionCounter, &0u64);

        env.events().publish((symbol_short!("opt_init"),), ());
    }

    /// Write (create) a new option contract.
    ///
    /// Writer must provide collateral which is locked until expiration or exercise.
    /// Returns the option ID.
    /// Emits `("opt_write",)` with data `(option_id, writer, option_type, strike_price, amount, expiration)`.
    pub fn write_option(
        env: Env,
        writer: Address,
        option_type: options::OptionType,
        token: Address,
        strike_price: i128,
        amount: i128,
        expiration: u64,
    ) -> u64 {
        Self::require_not_paused(&env);
        writer.require_auth();

        // Validate inputs
        if strike_price <= 0 || amount <= 0 {
            panic_with_error!(&env, TipJarError::InvalidOptionParams);
        }

        let now = env.ledger().timestamp();
        if expiration <= now {
            panic_with_error!(&env, TipJarError::InvalidOptionParams);
        }

        // Check token is whitelisted
        let whitelisted: bool = env
            .storage()
            .instance()
            .get(&DataKey::TokenWhitelist(token.clone()))
            .unwrap_or(false);
        if !whitelisted {
            panic_with_error!(&env, TipJarError::TokenNotWhitelisted);
        }

        // Calculate required collateral
        let collateral = options::calculate_collateral(option_type, strike_price, amount);

        // Generate option ID
        let option_id: u64 = env.storage().instance().get(&DataKey::OptionCounter).unwrap_or(0);
        env.storage().instance().set(&DataKey::OptionCounter, &(option_id + 1));

        // Create option contract
        let option = options::OptionContract {
            option_id,
            option_type,
            writer: writer.clone(),
            holder: None,
            token: token.clone(),
            strike_price,
            premium: 0,
            amount,
            expiration,
            created_at: now,
            status: options::OptionStatus::Active,
            collateral,
        };

        // Lock collateral
        token::Client::new(&env, &token).transfer(
            &writer,
            &env.current_contract_address(),
            &collateral,
        );

        // Update storage
        env.storage()
            .persistent()
            .set(&DataKey::Option(option_id), &option);
        
        options::add_written_option(&env, &writer, option_id);
        options::add_active_option(&env, option_id);

        // Update writer position
        let mut position = options::get_position(&env, &writer);
        position.written_count += 1;
        position.total_collateral += collateral;
        options::update_position(&env, &position);

        // Update locked collateral tracking
        let current_locked = options::get_locked_collateral(&env, &writer, &token);
        options::update_locked_collateral(&env, &writer, &token, current_locked + collateral);

        env.events().publish(
            (symbol_short!("opt_wrt"),),
            (option_id, writer, option_type, strike_price, amount, expiration),
        );

        option_id
    }

    /// Buy an option by paying the premium to the writer.
    ///
    /// Premium is calculated based on current pricing parameters.
    /// Emits `("opt_buy",)` with data `(option_id, buyer, premium)`.
    pub fn buy_option(
        env: Env,
        buyer: Address,
        option_id: u64,
        spot_price: i128,
    ) {
        Self::require_not_paused(&env);
        buyer.require_auth();

        // Get option
        let mut option: options::OptionContract = env
            .storage()
            .persistent()
            .get(&DataKey::Option(option_id))
            .unwrap_or_else(|| panic_with_error!(&env, TipJarError::OptionNotFound));

        // Verify option is available
        if option.holder.is_some() {
            panic_with_error!(&env, TipJarError::OptionAlreadySold);
        }

        if option.status != options::OptionStatus::Active {
            panic_with_error!(&env, TipJarError::OptionNotActive);
        }

        let now = env.ledger().timestamp();
        if now >= option.expiration {
            panic_with_error!(&env, TipJarError::OptionExpired);
        }

        // Calculate premium
        let params = options::get_pricing_params(&env);
        let time_to_expiry = option.expiration.saturating_sub(now);
        let premium = options::pricing::calculate_premium(
            &env,
            option.option_type,
            spot_price,
            option.strike_price,
            option.amount,
            time_to_expiry,
            &params,
        );

        // Transfer premium from buyer to writer
        token::Client::new(&env, &option.token).transfer(
            &buyer,
            &option.writer,
            &premium,
        );

        // Update option
        option.holder = Some(buyer.clone());
        option.premium = premium;
        env.storage()
            .persistent()
            .set(&DataKey::Option(option_id), &option);

        // Update positions
        let mut writer_position = options::get_position(&env, &option.writer);
        writer_position.premiums_earned += premium;
        options::update_position(&env, &writer_position);

        let mut buyer_position = options::get_position(&env, &buyer);
        buyer_position.held_count += 1;
        buyer_position.premiums_paid += premium;
        options::update_position(&env, &buyer_position);

        // Add to buyer's held options
        options::add_held_option(&env, &buyer, option_id);

        env.events().publish(
            (symbol_short!("opt_buy"),),
            (option_id, buyer, premium),
        );
    }

    /// Exercise an option contract.
    ///
    /// Only the holder can exercise. Option must be in the money.
    /// Returns the payoff amount.
    /// Emits `("opt_exer",)` with data `(option_id, holder, payoff)`.
    pub fn exercise_option(
        env: Env,
        holder: Address,
        option_id: u64,
        spot_price: i128,
    ) -> i128 {
        Self::require_not_paused(&env);
        holder.require_auth();

        let payoff = options::exercise::exercise_option(&env, &holder, option_id, spot_price);

        env.events().publish(
            (symbol_short!("opt_exer"),),
            (option_id, holder, payoff),
        );

        payoff
    }

    /// Expire an option that has passed its expiration time.
    ///
    /// Returns collateral to writer. Can be called by anyone.
    /// Emits `("opt_exp",)` with data `option_id`.
    pub fn expire_option(env: Env, option_id: u64) {
        options::exercise::expire_option(&env, option_id);

        env.events().publish(
            (symbol_short!("opt_exp"),),
            option_id,
        );
    }

    /// Cancel an unsold option (writer only).
    ///
    /// Returns collateral to writer. Only works if option has no holder yet.
    /// Emits `("opt_canc",)` with data `option_id`.
    pub fn cancel_option(env: Env, writer: Address, option_id: u64) {
        Self::require_not_paused(&env);
        writer.require_auth();

        options::exercise::cancel_option(&env, &writer, option_id);

        env.events().publish(
            (symbol_short!("opt_canc"),),
            option_id,
        );
    }

    /// Get option contract details by ID.
    pub fn get_option(env: Env, option_id: u64) -> Option<options::OptionContract> {
        options::get_option(&env, option_id)
    }

    /// Get all options written by an address.
    pub fn get_written_options(env: Env, writer: Address) -> Vec<u64> {
        env.storage()
            .persistent()
            .get(&DataKey::WrittenOptions(writer))
            .unwrap_or_else(|| Vec::new(&env))
    }

    /// Get all options held by an address.
    pub fn get_held_options(env: Env, holder: Address) -> Vec<u64> {
        env.storage()
            .persistent()
            .get(&DataKey::HeldOptions(holder))
            .unwrap_or_else(|| Vec::new(&env))
    }

    /// Get position summary for an address.
    pub fn get_option_position(env: Env, address: Address) -> options::OptionPosition {
        options::get_position(&env, &address)
    }

    /// Get all active options.
    pub fn get_active_options(env: Env) -> Vec<u64> {
        env.storage()
            .persistent()
            .get(&DataKey::ActiveOptions)
            .unwrap_or_else(|| Vec::new(&env))
    }

    /// Calculate option premium for given parameters.
    ///
    /// Useful for price discovery before writing an option.
    pub fn calculate_option_premium(
        env: Env,
        option_type: options::OptionType,
        spot_price: i128,
        strike_price: i128,
        amount: i128,
        time_to_expiry: u64,
    ) -> i128 {
        let params = options::get_pricing_params(&env);
        options::pricing::calculate_premium(
            &env,
            option_type,
            spot_price,
            strike_price,
            amount,
            time_to_expiry,
            &params,
        )
    }

    /// Update option pricing parameters (admin only).
    ///
    /// Emits `("opt_prm",)` with data `params`.
    pub fn update_option_pricing(
        env: Env,
        admin: Address,
        params: options::PricingParams,
    ) {
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic_with_error!(&env, TipJarError::Unauthorized);
        }

        options::update_pricing_params(&env, &params);

        env.events().publish(
            (symbol_short!("opt_prm"),),
            params,
        );
    }

    /// Get current option pricing parameters.
    pub fn get_option_pricing_params(env: Env) -> options::PricingParams {
        options::get_pricing_params(&env)
    }

    /// Batch expire multiple options.
    ///
    /// Useful for cleaning up expired options. Returns count of expired options.
    /// Emits `("opt_bexp",)` with data `expired_count`.
    pub fn batch_expire_options(env: Env, option_ids: Vec<u64>) -> u32 {
        let mut ids_vec: soroban_sdk::vec::Vec<u64> = soroban_sdk::vec::Vec::new(&env);
        for i in 0..option_ids.len() {
            ids_vec.push_back(option_ids.get(i).unwrap());
        }
        
        let mut expired_count = 0u32;
        for i in 0..ids_vec.len() {
            let option_id = ids_vec.get(i).unwrap();
            if let Some(option) = options::get_option(&env, option_id) {
                if option.status == options::OptionStatus::Active {
                    let now = env.ledger().timestamp();
                    if now >= option.expiration {
                        options::exercise::expire_option(&env, option_id);
                        expired_count += 1;
                    }
                }
            }
        }

        env.events().publish(
            (symbol_short!("opt_bexp"),),
            expired_count,
        );

        expired_count
    }

    // ── index funds ──────────────────────────────────────────────────────────

    /// Create a new tip index fund with a basket of creators.
    /// `components` must have at least 2 entries and weights must sum to 10_000 bps.
    pub fn create_index_fund(
        env: Env,
        manager: Address,
        token: Address,
        name: String,
        components: Vec<index_fund::IndexComponent>,
    ) -> u64 {
        manager.require_auth();
        Self::require_not_paused(&env);

        if components.len() < 2 {
            panic_with_error!(&env, TipJarError::IndexFundTooFewCreators);
        }
        if !index_fund::composition::validate_weights(&components) {
            panic_with_error!(&env, TipJarError::InvalidIndexWeights);
        }

        let fund_id = index_fund::composition::create_index_fund(
            &env, &manager, &token, name, components,
        );

        env.events().publish(
            (symbol_short!("idx_new"),),
            (manager, fund_id),
        );

        fund_id
    }

    /// Update the composition of an existing index fund (manager only).
    pub fn update_index_composition(
        env: Env,
        caller: Address,
        fund_id: u64,
        new_components: Vec<index_fund::IndexComponent>,
    ) {
        caller.require_auth();
        Self::require_not_paused(&env);

        if new_components.len() < 2 {
            panic_with_error!(&env, TipJarError::IndexFundTooFewCreators);
        }
        if !index_fund::composition::validate_weights(&new_components) {
            panic_with_error!(&env, TipJarError::InvalidIndexWeights);
        }

        index_fund::composition::update_composition(&env, fund_id, &caller, new_components);

        env.events().publish(
            (symbol_short!("idx_upd"),),
            (caller, fund_id),
        );
    }

    /// Rebalance a fund's creator allocations to match current target weights.
    pub fn rebalance_index_fund(env: Env, caller: Address, fund_id: u64) {
        caller.require_auth();
        Self::require_not_paused(&env);

        index_fund::rebalance::rebalance(&env, fund_id, &caller);

        env.events().publish(
            (symbol_short!("idx_reb"),),
            (caller, fund_id),
        );
    }

    /// Deposit tokens into an index fund and receive shares.
    /// Returns the number of shares minted.
    pub fn deposit_index_fund(
        env: Env,
        depositor: Address,
        fund_id: u64,
        amount: i128,
    ) -> i128 {
        Self::require_not_paused(&env);

        if amount < index_fund::MIN_DEPOSIT {
            panic_with_error!(&env, TipJarError::IndexDepositTooSmall);
        }

        let shares = index_fund::shares::deposit(&env, fund_id, &depositor, amount);

        env.events().publish(
            (symbol_short!("idx_dep"),),
            (depositor, fund_id, amount, shares),
        );

        shares
    }

    /// Withdraw from an index fund by redeeming shares.
    /// Returns the token amount returned to the holder.
    pub fn withdraw_index_fund(
        env: Env,
        holder: Address,
        fund_id: u64,
        shares: i128,
    ) -> i128 {
        Self::require_not_paused(&env);

        if shares <= 0 {
            panic_with_error!(&env, TipJarError::InvalidAmount);
        }

        let amount_out = index_fund::shares::withdraw(&env, fund_id, &holder, shares);

        env.events().publish(
            (symbol_short!("idx_wdr"),),
            (holder, fund_id, shares, amount_out),
        );

        amount_out
    }

    /// Get the current NAV (net asset value) per share for a fund.
    pub fn get_index_fund_nav(env: Env, fund_id: u64) -> i128 {
        index_fund::shares::get_nav(&env, fund_id)
    }

    /// Get the share balance of a holder in a fund.
    pub fn get_index_fund_shares(env: Env, fund_id: u64, holder: Address) -> i128 {
        index_fund::shares::get_shares(&env, fund_id, &holder)
    }

    /// Get the current creator allocations for a fund.
    pub fn get_index_fund_allocations(
        env: Env,
        fund_id: u64,
    ) -> Vec<(Address, i128)> {
        index_fund::rebalance::get_allocations(&env, fund_id)
    }
}
