# Conviction Voting System

## Overview

The conviction voting system implements a time-weighted voting mechanism where voting power accumulates over time. This encourages long-term commitment to governance decisions and prevents short-term manipulation of voting outcomes.

## Key Concepts

### Conviction Accumulation

Voting power is based on two factors:
1. **Base Voting Power**: The initial amount of tokens locked for voting
2. **Conviction Multiplier**: A time-based multiplier that increases as tokens remain locked

The effective voting power is calculated as:
```
Effective Voting Power = Base Voting Power × Conviction Multiplier
```

### Conviction Multiplier

The conviction multiplier grows linearly from 1x to a maximum over a configurable period:

- **At lock time**: 1x multiplier (base voting power)
- **At conviction period**: Maximum multiplier (default 3x)
- **Formula**: `multiplier = 1 + (time_locked / conviction_period) × (max_multiplier - 1)`

### Accumulated Conviction

Conviction accumulates over time as a measure of total voting commitment:
```
Accumulated Conviction = Base Voting Power × (time_locked / conviction_period)
```

This metric can be used to:
- Reduce proposal creation thresholds
- Determine governance participation rewards
- Track voter engagement over time

## Configuration

### Default Parameters

| Parameter | Value | Description |
|-----------|-------|-------------|
| `conviction_period` | 30 days (2,592,000 seconds) | Time to reach maximum conviction |
| `max_conviction_multiplier` | 3x (3,000,000) | Maximum voting power multiplier |
| `conviction_decay_rate_bps` | 0.01% (100 bps) | Decay rate per second when vote changes |
| `min_conviction_threshold` | 0.1 tokens (100,000) | Minimum voting power required |

### Updating Configuration

Only contract admins can update conviction voting configuration:

```rust
let new_config = ConvictionConfig {
    conviction_period: 2592000,
    max_conviction_multiplier: 3_000_000,
    conviction_decay_rate_bps: 100,
    min_conviction_threshold: 100_000,
};

contract.update_conviction_config(new_config);
```

## Usage

### 1. Initialize Conviction Voting

Before using conviction voting, initialize the system:

```rust
contract.init_conviction_voting();
```

### 2. Cast a Conviction Vote

Vote on a proposal with conviction voting:

```rust
contract.cast_conviction_vote(
    voter_address,
    proposal_id,
    VoteChoice::For,
    base_voting_power  // Amount of tokens to lock
);
```

**Effects:**
- Records conviction vote with current timestamp
- Calculates effective voting power with 1x multiplier (at start)
- Updates proposal vote totals
- Emits `("conv_vote",)` event

### 3. Change a Conviction Vote

Update an existing conviction vote (e.g., increase voting power):

```rust
contract.change_conviction_vote(
    voter_address,
    proposal_id,
    VoteChoice::Against,  // New choice
    new_base_voting_power  // New amount
);
```

**Effects:**
- Applies decay penalty to accumulated conviction
- Updates vote choice and base voting power
- Recalculates effective voting power
- Updates proposal vote totals
- Emits `("conv_chg",)` event

### 4. Query Voting Power

Get the effective voting power (with conviction multiplier):

```rust
let effective_power = contract.get_conviction_voting_power(proposal_id, voter_address);
```

### 5. Get Detailed Voting Information

Retrieve comprehensive conviction voting details:

```rust
let details = contract.get_conviction_voting_details(proposal_id, voter_address);
// Returns:
// - base_voting_power: Initial voting power
// - conviction_start: When conviction started accumulating
// - conviction_multiplier: Current time-based multiplier
// - accumulated_conviction: Total conviction accumulated
// - effective_voting_power: Base power × multiplier
// - time_locked: Seconds since conviction started
```

### 6. Track Total Conviction

Get total conviction accumulated across all proposals:

```rust
let total_conviction = contract.get_voter_total_conviction(voter_address);
```

## Advanced Features

### Proposal Threshold Reduction

Voters with high conviction can create proposals with reduced thresholds:

```rust
let adjusted_threshold = contract.get_adjusted_proposal_threshold(voter_address);
let can_propose = contract.can_create_proposal_with_conviction(voter_address);
```

**Formula:**
```
adjusted_threshold = base_threshold × (1 - conviction_bonus)
conviction_bonus = min(total_conviction / (base_threshold × 10), 0.5)
```

Maximum threshold reduction is 50%.

### Vote Change Decay

When a voter changes their vote, accumulated conviction decays as a penalty:

```
decay_amount = accumulated_conviction × decay_rate_bps × time_since_vote / 10_000 / 1_000_000
new_accumulated = accumulated_conviction - decay_amount
```

This discourages frequent vote changes and rewards conviction holders.

### Conviction History

All conviction votes are tracked for auditing:

```rust
let history = contract.get_conviction_history(proposal_id, voter_address);
```

## Data Structures

### ConvictionVote

```rust
pub struct ConvictionVote {
    pub voter: Address,
    pub proposal_id: u64,
    pub base_voting_power: i128,
    pub conviction_start: u64,
    pub last_updated: u64,
    pub accumulated_conviction: i128,
}
```

### ConvictionConfig

```rust
pub struct ConvictionConfig {
    pub conviction_period: u64,
    pub max_conviction_multiplier: i128,
    pub conviction_decay_rate_bps: u32,
    pub min_conviction_threshold: i128,
}
```

### ConvictionVotingDetails

```rust
pub struct ConvictionVotingDetails {
    pub base_voting_power: i128,
    pub conviction_start: u64,
    pub conviction_multiplier: i128,
    pub accumulated_conviction: i128,
    pub effective_voting_power: i128,
    pub time_locked: u64,
}
```

## Storage

Conviction voting data is stored in persistent storage with the following keys:

| Key | Purpose |
|-----|---------|
| `ConvictionVote(proposal_id, voter)` | Current conviction vote record |
| `ConvictionConfig` | Global conviction voting configuration |
| `ConvictionHistory(proposal_id, voter)` | Historical conviction vote records |
| `VoterConvictionTotal(voter)` | Total conviction across all proposals |

## Events

### Conviction Vote Cast

```
Event: ("conv_vote",)
Data: (voter, proposal_id, effective_voting_power)
```

Emitted when a voter casts a conviction vote.

### Conviction Vote Changed

```
Event: ("conv_chg",)
Data: (voter, proposal_id, new_effective_voting_power)
```

Emitted when a voter changes their conviction vote.

## Integration with Existing Governance

Conviction voting integrates seamlessly with the existing governance system:

1. **Proposals**: Uses existing proposal structure and voting periods
2. **Vote Recording**: Stores both conviction votes and standard votes
3. **Vote Totals**: Updates proposal vote counts with effective voting power
4. **Timelock**: Works with existing timelock mechanism for execution

## Examples

### Example 1: Basic Conviction Voting

```rust
// Initialize
contract.init_conviction_voting();

// Voter locks 1000 tokens for 30 days
contract.cast_conviction_vote(
    voter,
    proposal_1,
    VoteChoice::For,
    1_000_000_000  // 1000 tokens
);

// After 15 days, effective voting power is ~2000 (2x multiplier)
let details = contract.get_conviction_voting_details(proposal_1, voter);
assert_eq!(details.conviction_multiplier, 2_000_000);  // 2x
assert_eq!(details.effective_voting_power, 2_000_000_000);  // 2000 tokens
```

### Example 2: Vote Change with Decay

```rust
// Initial vote: 1000 tokens
contract.cast_conviction_vote(voter, proposal_1, VoteChoice::For, 1_000_000_000);

// After 10 days, voter wants to increase to 2000 tokens
contract.change_conviction_vote(
    voter,
    proposal_1,
    VoteChoice::For,
    2_000_000_000  // 2000 tokens
);

// Accumulated conviction is reduced due to decay penalty
let details = contract.get_conviction_voting_details(proposal_1, voter);
// accumulated_conviction < 1_000_000_000 (due to decay)
```

### Example 3: Reduced Proposal Threshold

```rust
// Voter has high conviction across multiple proposals
let total_conviction = contract.get_voter_total_conviction(voter);  // 10,000 tokens

// Proposal threshold is reduced
let adjusted_threshold = contract.get_adjusted_proposal_threshold(voter);
// adjusted_threshold < DEFAULT_PROPOSAL_THRESHOLD

// Voter can now create proposals with less voting power
let can_propose = contract.can_create_proposal_with_conviction(voter);
assert!(can_propose);
```

## Security Considerations

1. **Minimum Threshold**: Prevents spam voting with negligible amounts
2. **Decay Penalty**: Discourages frequent vote changes
3. **Time-Based Accumulation**: Prevents instant voting power acquisition
4. **Admin Controls**: Configuration updates require admin authorization
5. **Persistent Storage**: All votes are permanently recorded for auditing

## Performance

- **Storage**: O(1) per vote (composite key lookup)
- **Calculation**: O(1) for multiplier and conviction calculations
- **Events**: Emitted for all conviction voting actions
- **Scalability**: Supports unlimited voters and proposals

## Future Enhancements

1. **Delegation**: Allow conviction voting power delegation
2. **Quadratic Conviction**: Implement quadratic conviction scaling
3. **Conviction Decay**: Automatic conviction decay over time
4. **Conviction Rewards**: Reward voters for maintaining conviction
5. **Conviction Bonds**: Require conviction bonds for proposal creation
6. **Conviction Slashing**: Slash conviction for malicious voting

## References

- [Polkadot Conviction Voting](https://wiki.polkadot.network/docs/learn-governance#conviction-voting)
- [Cosmos Governance](https://docs.cosmos.network/main/modules/gov)
- [Quadratic Voting](https://en.wikipedia.org/wiki/Quadratic_voting)
