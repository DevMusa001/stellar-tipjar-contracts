# TipJar Contract — Integration Guide

This guide covers everything you need to integrate the TipJar Soroban smart contract into your application.

## Table of Contents

- [Overview](#overview)
- [Prerequisites](#prerequisites)
- [Quick Start](#quick-start)
- [Contract Setup](#contract-setup)
- [Core Workflows](#core-workflows)
- [Role System](#role-system)
- [Multi-Token Support](#multi-token-support)
- [Advanced Features](#advanced-features)
- [Further Reading](#further-reading)

---

## Overview

The TipJar contract escrows Stellar token tips for creators. Key capabilities:

- **Token whitelisting** — only admin-approved tokens can be used for tips
- **Role-based access control** — Admin, Moderator, and Creator roles
- **Batch tipping** — up to 50 tips in a single transaction
- **Tipping with messages** — attach a note and metadata to a tip
- **Locked tips** — time-locked escrow released after a deadline
- **Leaderboards** — ranked tippers and creators across AllTime, Monthly, and Weekly periods
- **Emergency pause** — Admin or Moderator can halt all state-changing operations

---

## Prerequisites

- [Rust toolchain](https://rustup.rs/) (stable)
- [Stellar CLI](https://developers.stellar.org/docs/tools/developer-tools/cli/install-cli)
- Soroban WASM target:

```bash
rustup target add wasm32v1-none
```

- A funded Stellar testnet account (use `stellar keys generate` and the Friendbot faucet)

---

## Quick Start

```bash
# 1. Build
cargo build -p tipjar --target wasm32v1-none --release

# 2. Deploy
CONTRACT_ID=$(stellar contract deploy \
  --wasm contracts/tipjar/target/wasm32v1-none/release/tipjar.wasm \
  --source alice \
  --network testnet)

# 3. Initialize (sets the admin; run once)
stellar contract invoke --id $CONTRACT_ID --source alice --network testnet \
  -- init --admin ADMIN_ADDRESS

# 4. Whitelist a token
stellar contract invoke --id $CONTRACT_ID --source alice --network testnet \
  -- add_token --admin ADMIN_ADDRESS --token TOKEN_ADDRESS

# 5. Send a tip
stellar contract invoke --id $CONTRACT_ID --source alice --network testnet \
  -- tip \
  --sender SENDER_ADDRESS \
  --creator CREATOR_ADDRESS \
  --token TOKEN_ADDRESS \
  --amount 1000000
```

---

## Contract Setup

### Initialization

`init` is a one-time call that sets the contract administrator and grants them the `Admin` role. It also sets `Paused = false`.

```bash
stellar contract invoke --id $CONTRACT_ID --source alice --network testnet \
  -- init --admin ADMIN_ADDRESS
```

Calling `init` a second time panics with `AlreadyInitialized`.

### Token Whitelisting

Only whitelisted tokens can be used for tips. The admin manages the whitelist.

```bash
# Add a token
stellar contract invoke --id $CONTRACT_ID --source alice --network testnet \
  -- add_token --admin ADMIN_ADDRESS --token TOKEN_ADDRESS

# Remove a token
stellar contract invoke --id $CONTRACT_ID --source alice --network testnet \
  -- remove_token --admin ADMIN_ADDRESS --token TOKEN_ADDRESS

# Check whitelist status
stellar contract invoke --id $CONTRACT_ID --network testnet \
  -- is_whitelisted --token TOKEN_ADDRESS
```

---

## Core Workflows

### Sending a Tip

The sender must have sufficient token balance and the token must be whitelisted.

```bash
stellar contract invoke --id $CONTRACT_ID --source alice --network testnet \
  -- tip \
  --sender SENDER_ADDRESS \
  --creator CREATOR_ADDRESS \
  --token TOKEN_ADDRESS \
  --amount 1000000
```

Emits event: topics `("tip", creator, token)`, data `(sender, amount)`.

### Querying Balances

```bash
# Total historical tips (never decreases after withdrawals)
stellar contract invoke --id $CONTRACT_ID --network testnet \
  -- get_total_tips --creator CREATOR_ADDRESS --token TOKEN_ADDRESS

# Current withdrawable balance
stellar contract invoke --id $CONTRACT_ID --network testnet \
  -- get_withdrawable_balance --creator CREATOR_ADDRESS --token TOKEN_ADDRESS
```

### Creator Withdrawal

The creator must hold the `Creator` role (granted by an Admin) before withdrawing.

```bash
# Admin grants Creator role
stellar contract invoke --id $CONTRACT_ID --source alice --network testnet \
  -- grant_role --caller ADMIN_ADDRESS --target CREATOR_ADDRESS --role Creator

# Creator withdraws
stellar contract invoke --id $CONTRACT_ID --source creator --network testnet \
  -- withdraw --creator CREATOR_ADDRESS --token TOKEN_ADDRESS
```

Emits event: topics `("withdraw", creator, token)`, data `amount`.

---

## Role System

The contract uses three roles:

| Role | Capabilities |
|---|---|
| `Admin` | Whitelist tokens, grant/revoke roles, pause/unpause |
| `Moderator` | Pause/unpause |
| `Creator` | Withdraw escrowed tips and locked tips |

```bash
# Grant a role
stellar contract invoke --id $CONTRACT_ID --source alice --network testnet \
  -- grant_role --caller ADMIN_ADDRESS --target TARGET_ADDRESS --role Moderator

# Revoke a role
stellar contract invoke --id $CONTRACT_ID --source alice --network testnet \
  -- revoke_role --caller ADMIN_ADDRESS --target TARGET_ADDRESS

# Check a role
stellar contract invoke --id $CONTRACT_ID --network testnet \
  -- has_role --target TARGET_ADDRESS --role Creator
```

---

## Multi-Token Support

Balances and totals are tracked per `(creator, token)` pair. A creator can receive tips in multiple whitelisted tokens and withdraw each independently.

```bash
# Tip with token A
stellar contract invoke --id $CONTRACT_ID --source alice --network testnet \
  -- tip --sender SENDER --creator CREATOR --token TOKEN_A --amount 500000

# Tip with token B
stellar contract invoke --id $CONTRACT_ID --source alice --network testnet \
  -- tip --sender SENDER --creator CREATOR --token TOKEN_B --amount 250000

# Withdraw token A balance
stellar contract invoke --id $CONTRACT_ID --source creator --network testnet \
  -- withdraw --creator CREATOR --token TOKEN_A

# Withdraw token B balance
stellar contract invoke --id $CONTRACT_ID --source creator --network testnet \
  -- withdraw --creator CREATOR --token TOKEN_B
```

---

## Advanced Features

### Batch Tipping

Send up to 50 tips in one transaction. Each entry is processed independently — failures do not roll back successful entries.

```bash
stellar contract invoke --id $CONTRACT_ID --source alice --network testnet \
  -- tip_batch \
  --sender SENDER_ADDRESS \
  --tips '[{"creator":"CREATOR_A","token":"TOKEN_ADDRESS","amount":100000},{"creator":"CREATOR_B","token":"TOKEN_ADDRESS","amount":200000}]'
```

Returns a result vector with `Ok` or an error code per entry.

### Tipping with a Message

Attach a note (max 280 chars) and arbitrary metadata to a tip.

```bash
stellar contract invoke --id $CONTRACT_ID --source alice --network testnet \
  -- tip_with_message \
  --sender SENDER_ADDRESS \
  --creator CREATOR_ADDRESS \
  --token TOKEN_ADDRESS \
  --amount 1000000 \
  --message "Great content, keep it up!" \
  --metadata '{}'
```

Retrieve stored messages:

```bash
stellar contract invoke --id $CONTRACT_ID --network testnet \
  -- get_messages --creator CREATOR_ADDRESS
```

### Locked Tips

Lock tokens in escrow until a future timestamp. The creator can only withdraw after the unlock time.

```bash
# Lock a tip (returns tip_id)
stellar contract invoke --id $CONTRACT_ID --source alice --network testnet \
  -- tip_locked \
  --sender SENDER_ADDRESS \
  --creator CREATOR_ADDRESS \
  --token TOKEN_ADDRESS \
  --amount 1000000 \
  --unlock_timestamp 1800000000

# Check a locked tip
stellar contract invoke --id $CONTRACT_ID --network testnet \
  -- get_locked_tip --creator CREATOR_ADDRESS --tip_id 0

# Withdraw after unlock time (Creator role required)
stellar contract invoke --id $CONTRACT_ID --source creator --network testnet \
  -- withdraw_locked --creator CREATOR_ADDRESS --tip_id 0
```

### Leaderboards

Query ranked tippers and creators for AllTime, Monthly, or Weekly periods.

```bash
# Top 10 tippers all-time
stellar contract invoke --id $CONTRACT_ID --network testnet \
  -- get_top_tippers --period AllTime --offset 0 --limit 10

# Top 10 creators this month
stellar contract invoke --id $CONTRACT_ID --network testnet \
  -- get_top_creators --period Monthly --offset 0 --limit 10
```

Ranking is by `total_amount` descending; `tip_count` is the tiebreaker.

---

## Further Reading

- [API Reference](./API.md)
- [Events Reference](./EVENTS.md)
- [Storage Model](./STORAGE.md)
- [Getting Started Tutorial](./tutorials/getting-started.md)
- [Frontend Integration](./tutorials/frontend-integration.md)
- [Backend Integration](./tutorials/backend-integration.md)
- [Testing Guide](./tutorials/testing-guide.md)
- [Best Practices](./BEST_PRACTICES.md)
- [Troubleshooting](./TROUBLESHOOTING.md)
- [Security Considerations](./SECURITY.md)
