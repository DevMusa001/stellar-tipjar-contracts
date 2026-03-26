# Getting Started with the TipJar Contract

This tutorial walks you through deploying the TipJar contract to Stellar testnet and sending your first tip end-to-end.

## Prerequisites

- Rust toolchain (stable)
- Stellar CLI installed and authenticated
- `wasm32v1-none` target:

```bash
rustup target add wasm32v1-none
```

## Step 1: Build the Contract

```bash
cargo build -p tipjar --target wasm32v1-none --release
```

The compiled WASM lands at:

```
contracts/tipjar/target/wasm32v1-none/release/tipjar.wasm
```

## Step 2: Set Up Testnet Accounts

```bash
# Generate keys for admin, a sender, and a creator
stellar keys generate admin --network testnet
stellar keys generate sender --network testnet
stellar keys generate creator --network testnet

# Fund them via Friendbot
stellar keys fund admin --network testnet
stellar keys fund sender --network testnet
stellar keys fund creator --network testnet
```

## Step 3: Deploy the Contract

```bash
CONTRACT_ID=$(stellar contract deploy \
  --wasm contracts/tipjar/target/wasm32v1-none/release/tipjar.wasm \
  --source admin \
  --network testnet)

echo "Contract ID: $CONTRACT_ID"
```

## Step 4: Initialize the Contract

`init` is a one-time call. Pass the admin address that will control the contract.

```bash
ADMIN_ADDRESS=$(stellar keys address admin)

stellar contract invoke \
  --id $CONTRACT_ID \
  --source admin \
  --network testnet \
  -- init --admin $ADMIN_ADDRESS
```

## Step 5: Whitelist a Token

Only whitelisted tokens can be used for tips. Use a Stellar Asset Contract (SAC) address for a testnet asset.

```bash
stellar contract invoke \
  --id $CONTRACT_ID \
  --source admin \
  --network testnet \
  -- add_token --admin $ADMIN_ADDRESS --token TOKEN_ADDRESS
```

Replace `TOKEN_ADDRESS` with the SAC address of the token you want to use (e.g., the native XLM SAC or a custom asset).

## Step 6: Grant the Creator Role

Creators must hold the `Creator` role before they can withdraw tips.

```bash
CREATOR_ADDRESS=$(stellar keys address creator)

stellar contract invoke \
  --id $CONTRACT_ID \
  --source admin \
  --network testnet \
  -- grant_role \
  --caller $ADMIN_ADDRESS \
  --target $CREATOR_ADDRESS \
  --role Creator
```

## Step 7: Send Your First Tip

```bash
SENDER_ADDRESS=$(stellar keys address sender)

stellar contract invoke \
  --id $CONTRACT_ID \
  --source sender \
  --network testnet \
  -- tip \
  --sender $SENDER_ADDRESS \
  --creator $CREATOR_ADDRESS \
  --token TOKEN_ADDRESS \
  --amount 1000000
```

`amount` is in the token's smallest unit (stroops for XLM, where 1 XLM = 10,000,000 stroops).

## Step 8: Check the Creator's Balance

```bash
# Withdrawable balance (resets to 0 after each withdrawal)
stellar contract invoke \
  --id $CONTRACT_ID \
  --network testnet \
  -- get_withdrawable_balance \
  --creator $CREATOR_ADDRESS \
  --token TOKEN_ADDRESS

# All-time total (never decreases)
stellar contract invoke \
  --id $CONTRACT_ID \
  --network testnet \
  -- get_total_tips \
  --creator $CREATOR_ADDRESS \
  --token TOKEN_ADDRESS
```

## Step 9: Creator Withdraws

```bash
stellar contract invoke \
  --id $CONTRACT_ID \
  --source creator \
  --network testnet \
  -- withdraw \
  --creator $CREATOR_ADDRESS \
  --token TOKEN_ADDRESS
```

The full escrowed balance is transferred to the creator's address and the contract balance resets to zero.

## What's Next

- [Frontend Integration](./frontend-integration.md) — integrate from a web app using `@stellar/stellar-sdk`
- [Backend Integration](./backend-integration.md) — automate tips from a server
- [Testing Guide](./testing-guide.md) — write unit and integration tests
