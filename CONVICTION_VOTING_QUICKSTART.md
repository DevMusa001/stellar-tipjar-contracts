# Conviction Voting - Quick Start Guide

## What is Conviction Voting?

Conviction voting is a governance mechanism where your voting power increases the longer you lock tokens. This encourages long-term commitment to governance decisions.

**Key Idea**: The longer you hold your vote, the more powerful it becomes.

## Quick Facts

| Aspect | Value |
|--------|-------|
| **Max Multiplier** | 3x (after 30 days) |
| **Conviction Period** | 30 days |
| **Min Voting Power** | 0.1 tokens |
| **Decay on Vote Change** | 0.01% per second |
| **Proposal Threshold Reduction** | Up to 50% |

## Getting Started

### 1. Initialize Conviction Voting

```rust
contract.init_conviction_voting();
```

This sets up the conviction voting system with default parameters.

### 2. Cast Your First Conviction Vote

```rust
contract.cast_conviction_vote(
    my_address,
    proposal_id,
    VoteChoice::For,
    1_000_000_000  // 1000 tokens
);
```

Your voting power starts at 1x (1000 tokens) and grows over time.

### 3. Check Your Voting Power

```rust
let power = contract.get_conviction_voting_power(proposal_id, my_address);
// Day 0: 1000 tokens (1x)
// Day 15: 2000 tokens (2x)
// Day 30+: 3000 tokens (3x)
```

### 4. Get Detailed Information

```rust
let details = contract.get_conviction_voting_details(proposal_id, my_address);

println!("Base Power: {}", details.base_voting_power);           // 1000
println!("Multiplier: {}", details.conviction_multiplier);       // 2_000_000 (2x)
println!("Effective Power: {}", details.effective_voting_power); // 2000
println!("Time Locked: {} seconds", details.time_locked);        // 1,296,000 (15 days)
```

## Common Scenarios

### Scenario 1: Vote on a Proposal

```rust
// Day 1: Cast vote with 1000 tokens
contract.cast_conviction_vote(voter, proposal_1, VoteChoice::For, 1_000_000_000);

// Day 15: Your voting power is now 2000 tokens
let power = contract.get_conviction_voting_power(proposal_1, voter);
assert_eq!(power, 2_000_000_000);

// Day 30: Your voting power reaches maximum 3000 tokens
let power = contract.get_conviction_voting_power(proposal_1, voter);
assert_eq!(power, 3_000_000_000);
```

### Scenario 2: Change Your Vote

```rust
// Initial vote: 1000 tokens for 15 days
contract.cast_conviction_vote(voter, proposal_1, VoteChoice::For, 1_000_000_000);

// Change to Against with 2000 tokens
contract.change_conviction_vote(
    voter,
    proposal_1,
    VoteChoice::Against,
    2_000_000_000
);

// Your accumulated conviction is reduced due to decay penalty
let details = contract.get_conviction_voting_details(proposal_1, voter);
// accumulated_conviction < 1_000_000_000 (due to decay)
```

### Scenario 3: Build Conviction for Proposal Creation

```rust
// Vote on multiple proposals to build conviction
for proposal_id in 1..=10 {
    contract.cast_conviction_vote(
        voter,
        proposal_id,
        VoteChoice::For,
        1_000_000_000  // 1000 tokens each
    );
}

// Check total conviction
let total = contract.get_voter_total_conviction(voter);
// total = 10_000_000_000 (10,000 tokens)

// Check if you can create a proposal
let can_propose = contract.can_create_proposal_with_conviction(voter);
// true (with reduced threshold)

// Get your adjusted proposal threshold
let threshold = contract.get_adjusted_proposal_threshold(voter);
// threshold < DEFAULT_PROPOSAL_THRESHOLD (50% reduction possible)
```

## Understanding Multipliers

### How Multipliers Work

Your effective voting power = base voting power × multiplier

```
Day 0:  1000 × 1.0 = 1000 tokens
Day 15: 1000 × 2.0 = 2000 tokens
Day 30: 1000 × 3.0 = 3000 tokens
```

### Multiplier Growth

The multiplier grows linearly from 1x to 3x over 30 days:

```
Multiplier = 1 + (days_locked / 30) × 2
```

Examples:
- 0 days: 1.0x
- 7.5 days: 1.5x
- 15 days: 2.0x
- 22.5 days: 2.5x
- 30+ days: 3.0x (max)

## Understanding Decay

### What is Decay?

When you change your vote, your accumulated conviction is reduced as a penalty.

```
decay = accumulated_conviction × 0.01% × seconds_since_vote
```

### Example

```
Initial vote: 1000 tokens for 10 days
Accumulated conviction: ~1000 (10 days of accumulation)

Change vote after 1 day:
Decay: 1000 × 0.01% × 86400 = ~0.864
New accumulated: 999.136
```

### Why Decay?

- Discourages frequent vote changes
- Rewards conviction holders
- Prevents gaming the system

## Proposal Thresholds

### How Thresholds Work

Voters with high conviction can create proposals with lower thresholds.

```
adjusted_threshold = base_threshold × (1 - conviction_bonus)
conviction_bonus = min(total_conviction / (base_threshold × 10), 50%)
```

### Example

```
Base threshold: 1000 tokens
Your total conviction: 5000 tokens

Conviction bonus: min(5000 / 100, 50%) = 5%
Adjusted threshold: 1000 × (1 - 0.05) = 950 tokens

You can create proposals with only 950 tokens!
```

### Maximum Reduction

The maximum threshold reduction is 50%, achieved when:
```
total_conviction >= base_threshold × 10
```

## Configuration

### Default Configuration

```rust
ConvictionConfig {
    conviction_period: 2_592_000,        // 30 days
    max_conviction_multiplier: 3_000_000, // 3x
    conviction_decay_rate_bps: 100,      // 0.01% per second
    min_conviction_threshold: 100_000,   // 0.1 tokens
}
```

### Updating Configuration (Admin Only)

```rust
let new_config = ConvictionConfig {
    conviction_period: 5_184_000,        // 60 days
    max_conviction_multiplier: 5_000_000, // 5x
    conviction_decay_rate_bps: 50,       // 0.005% per second
    min_conviction_threshold: 50_000,    // 0.05 tokens
};

contract.update_conviction_config(new_config);
```

## API Reference

### Core Functions

| Function | Purpose |
|----------|---------|
| `init_conviction_voting()` | Initialize system |
| `cast_conviction_vote()` | Cast a conviction vote |
| `change_conviction_vote()` | Change existing vote |
| `get_conviction_voting_power()` | Get effective voting power |
| `get_conviction_voting_details()` | Get detailed voting info |
| `get_voter_total_conviction()` | Get total conviction |
| `get_conviction_config()` | Get current configuration |
| `update_conviction_config()` | Update configuration |
| `can_create_proposal_with_conviction()` | Check proposal eligibility |
| `get_adjusted_proposal_threshold()` | Get reduced threshold |

## Tips & Tricks

### Tip 1: Build Conviction Early

Start voting early to build conviction for future proposals:
```rust
// Vote on early proposals to build conviction
// This reduces your threshold for creating proposals later
```

### Tip 2: Don't Change Votes Frequently

Changing votes applies decay penalties:
```rust
// Good: Vote once and hold
contract.cast_conviction_vote(voter, proposal, VoteChoice::For, power);

// Avoid: Changing votes frequently
contract.change_conviction_vote(voter, proposal, VoteChoice::Against, power);
contract.change_conviction_vote(voter, proposal, VoteChoice::For, power);
```

### Tip 3: Use Conviction for Proposal Creation

Build conviction to reduce proposal creation thresholds:
```rust
// Build conviction across multiple proposals
// Then use reduced threshold to create your own proposals
```

### Tip 4: Monitor Your Voting Power

Check your voting power regularly:
```rust
let details = contract.get_conviction_voting_details(proposal_id, voter);
println!("Your voting power: {}", details.effective_voting_power);
println!("Time locked: {} days", details.time_locked / 86400);
```

## Troubleshooting

### Issue: "Voting power below minimum conviction threshold"

**Cause**: Your voting power is too low (< 0.1 tokens)

**Solution**: Use at least 0.1 tokens (100,000 in base units)

```rust
// Wrong: Too low
contract.cast_conviction_vote(voter, proposal, VoteChoice::For, 50_000);

// Correct: Meets minimum
contract.cast_conviction_vote(voter, proposal, VoteChoice::For, 100_000);
```

### Issue: "Already voted on this proposal"

**Cause**: You already cast a conviction vote on this proposal

**Solution**: Use `change_conviction_vote()` instead of `cast_conviction_vote()`

```rust
// Wrong: Already voted
contract.cast_conviction_vote(voter, proposal, VoteChoice::For, power);
contract.cast_conviction_vote(voter, proposal, VoteChoice::Against, power);

// Correct: Use change function
contract.cast_conviction_vote(voter, proposal, VoteChoice::For, power);
contract.change_conviction_vote(voter, proposal, VoteChoice::Against, power);
```

### Issue: "Proposal is not active"

**Cause**: Voting period has ended

**Solution**: Check proposal status before voting

```rust
let proposal = contract.get_proposal(proposal_id);
if proposal.end_time > env.ledger().timestamp() {
    // Voting is still active
    contract.cast_conviction_vote(voter, proposal_id, VoteChoice::For, power);
}
```

## Next Steps

1. **Read Full Documentation**: See `CONVICTION_VOTING.md` for complete details
2. **Review Implementation**: See `CONVICTION_VOTING_IMPLEMENTATION.md` for technical details
3. **Run Tests**: See `tests/conviction_voting_tests.rs` for test examples
4. **Deploy**: Integrate into your governance system

## Support

For questions or issues:
1. Check the full documentation: `CONVICTION_VOTING.md`
2. Review implementation details: `CONVICTION_VOTING_IMPLEMENTATION.md`
3. Look at test examples: `tests/conviction_voting_tests.rs`
4. Check the commit message: `CONVICTION_VOTING_COMMIT.txt`
