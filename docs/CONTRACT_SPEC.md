# TipJar Contract Specification

## Overview

The TipJar contract lets supporters tip creators using whitelisted Stellar tokens. Tips are held in escrow and can be withdrawn by the creator at any time. The contract supports optional memos, recurring subscriptions, split tips, conditional execution, and a leaderboard.

## Public Functions

### Initialization

#### `init(admin: Address, fee_basis_points: u32, refund_window_seconds: u64)`
One-time setup. Stores the admin address, platform fee, and refund window.

- Panics: `AlreadyInitialized` if called more than once.
- Panics: `FeeExceedsMaximum` if `fee_basis_points > 500` (5%).

#### `add_token(admin: Address, token: Address)`
Whitelists a token for use in tips. Admin only.

- Panics: `Unauthorized` if caller is not the stored admin.

### Tipping

#### `tip(sender: Address, creator: Address, token: Address, amount: i128) -> u64`
Transfers `amount` of `token` from `sender` into escrow for `creator`. Returns the tip ID.

- Requires auth from `sender`.
- Panics: `InvalidAmount` if `amount <= 0`.
- Panics: `TokenNotWhitelisted` if the token is not approved.
- Emits: `("tip", creator)` → `(sender, amount)`.

#### `tip_with_memo(sender, creator, token, amount, memo: Option<String>)`
Like `tip` but stores an optional memo (max 200 UTF-8 characters) on-chain alongside a timestamp. Uses the `CreatorStats` optimization (single storage read/write per call).

- Panics: `MemoTooLong` if memo exceeds 200 characters.
- Emits: `("tip_memo", creator)` → `(sender, amount)`.

#### `tip_with_fee(sender, creator, token, amount, congestion: u32)`
Deducts a dynamic platform fee before crediting the creator. `congestion`: 0=Low, 1=Normal, 2=High.

- Emits: `("tip", creator)` → `(sender, net_amount)` and `("fee", creator)` → `(fee_amount, fee_bps)`.

#### `tip_split(sender, token, recipients: Vec<TipRecipient>, amount)`
Splits a tip among 2–10 recipients. Each recipient's `percentage` is in basis points; all must sum to 10 000.

- Panics: `InvalidRecipientCount`, `InvalidPercentage`, `InvalidPercentageSum`.
- Emits: `("tip_splt", creator)` → `(sender, share, percentage)` per recipient.

#### `execute_conditional_tip(sender, creator, token, amount, conditions) -> bool`
Executes a tip only if all conditions evaluate to true. Returns `false` (no transfer) when conditions fail.

### Querying

#### `get_withdrawable_balance(creator: Address, token: Address) -> i128`
Returns the creator's current escrowed balance.

#### `get_total_tips(creator: Address, token: Address) -> i128`
Returns the historical total tips received by the creator.

#### `get_tips_with_memos(creator: Address, limit: u32) -> Vec<TipWithMemo>`
Returns the most recent `limit` memo-tips (capped at 50) for the creator, oldest first.

#### `get_leaderboard(period: TimePeriod, kind: ParticipantKind, limit: u32) -> Vec<LeaderboardEntry>`
Returns the top `limit` (max 100) tippers or creators sorted by total amount descending.

### Withdrawal

#### `withdraw(creator: Address, token: Address)`
Transfers the full escrowed balance to the creator.

- Requires auth from `creator`.
- Panics: `NothingToWithdraw` if balance is zero.
- Emits: `("withdraw", creator)` → `amount`.

### Subscriptions

#### `create_subscription(subscriber, creator, token, amount, interval_seconds)`
Creates a recurring tip. Minimum interval: 86 400 s (1 day).

#### `execute_subscription_payment(subscriber, creator)`
Executes a due payment. Anyone may call this.

#### `pause_subscription / resume_subscription / cancel_subscription`
Subscriber-only lifecycle management.

#### `get_subscription(subscriber, creator) -> Option<Subscription>`

### Administration

#### `pause(admin, reason: String)` / `unpause(admin)`
Halts / resumes all state-changing operations.

#### `is_paused() -> bool`

#### `upgrade(new_wasm_hash: BytesN<32>)`
Upgrades the contract WASM. Admin only. Increments the on-chain version.

#### `get_version() -> u32`

## Storage Layout

| Key | Type | Description |
|-----|------|-------------|
| `Admin` | `Address` | Contract administrator |
| `TokenWhitelist(token)` | `bool` | Whether a token is approved |
| `CreatorBalance(creator, token)` | `i128` | Withdrawable escrow balance |
| `CreatorTotal(creator, token)` | `i128` | Historical total tips |
| `CreatorStats(creator, token)` | `CreatorStats` | Combined balance+total (optimized) |
| `TipCount(creator)` | `u64` | Number of memo-tips stored |
| `TipData(creator, index)` | `TipWithMemo` | Individual memo-tip record |
| `Subscription(subscriber, creator)` | `Subscription` | Recurring tip state |
| `Paused` | `bool` | Emergency pause flag |
| `PauseReason` | `String` | Human-readable pause reason |
| `ContractVersion` | `u32` | Incremented on each upgrade |

## Error Codes

| Code | Name | Description |
|------|------|-------------|
| 1 | `AlreadyInitialized` | `init` called more than once |
| 2 | `TokenNotWhitelisted` | Token not approved for tips |
| 3 | `InvalidAmount` | Amount must be positive |
| 4 | `NothingToWithdraw` | Creator balance is zero |
| 5 | `MessageTooLong` | Tip message exceeds limit |
| 9 | `Unauthorized` | Caller is not the admin |
| 24 | `SubscriptionNotFound` | No subscription exists |
| 25 | `SubscriptionNotActive` | Subscription is paused or cancelled |
| 26 | `PaymentNotDue` | Interval has not elapsed |
| 27 | `InvalidInterval` | Interval below 1-day minimum |
| 28 | `InvalidRecipientCount` | Must have 2–10 recipients |
| 29 | `InvalidPercentageSum` | Shares must sum to 10 000 bps |
| 30 | `InvalidPercentage` | Individual share is zero |
| 31 | `ContractPaused` | Contract is paused |
| 32 | `MemoTooLong` | Memo exceeds 200 characters |

## Data Structures

```rust
pub struct TipWithMemo {
    pub sender: Address,
    pub amount: i128,
    pub memo: Option<String>,  // max 200 UTF-8 chars
    pub timestamp: u64,        // ledger timestamp
}

pub struct CreatorStats {
    pub balance: i128,  // withdrawable
    pub total: i128,    // historical
}

pub struct Subscription {
    pub subscriber: Address,
    pub creator: Address,
    pub token: Address,
    pub amount: i128,
    pub interval_seconds: u64,
    pub last_payment: u64,
    pub next_payment: u64,
    pub status: SubscriptionStatus,  // Active | Paused | Cancelled
}

pub struct TipRecipient {
    pub creator: Address,
    pub percentage: u32,  // basis points, must be > 0
}
```

## Events

| Topic | Data | Emitted by |
|-------|------|-----------|
| `("tip", creator)` | `(sender, amount)` | `tip` |
| `("tip_memo", creator)` | `(sender, amount)` | `tip_with_memo` |
| `("tip_splt", creator)` | `(sender, share, pct)` | `tip_split` |
| `("withdraw", creator)` | `amount` | `withdraw` |
| `("sub_new", creator)` | `(subscriber, amount, interval)` | `create_subscription` |
| `("sub_pay", creator)` | `(subscriber, amount)` | `execute_subscription_payment` |
| `("paused",)` | `(admin, reason)` | `pause` |
| `("unpaused",)` | `admin` | `unpause` |
| `("upgraded",)` | `version` | `upgrade` |
