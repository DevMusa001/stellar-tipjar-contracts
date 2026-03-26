# Best Practices

Guidelines for building reliable integrations on top of the TipJar contract.

## Validate Inputs Before Calling the Contract

Contract calls cost fees. Reject bad inputs client-side before submitting a transaction.

```typescript
function validateTipAmount(amount: bigint): void {
  if (amount <= 0n) {
    throw new Error('Tip amount must be greater than zero');
  }
}

function validateMessage(message: string): void {
  if (message.length > 280) {
    throw new Error('Message must be 280 characters or fewer');
  }
}
```

## Always Simulate Before Submitting

Simulation catches errors (insufficient balance, wrong role, paused contract) without spending fees.

```typescript
const simResult = await server.simulateTransaction(tx);
if (SorobanRpc.Api.isSimulationError(simResult)) {
  // Surface the error to the user before submitting
  throw new Error(`Transaction would fail: ${simResult.error}`);
}
```

## Handle Transaction Failures Gracefully

Transactions can fail after submission due to network conditions or ledger state changes. Always poll for the final status and handle `FAILED` explicitly.

```typescript
const result = await server.getTransaction(hash);
if (result.status === SorobanRpc.Api.GetTransactionStatus.FAILED) {
  // Log the failure, notify the user, and do not update local state
}
```

## Use Batch Tips for Fan-Out Scenarios

When tipping multiple creators at once, `tip_batch` is more efficient than individual `tip` calls — it uses a single transaction and a single authorization.

- Keep batches at or below 50 entries (the contract enforces this limit).
- Check each result entry individually; a failed entry does not roll back successful ones.

## Cache Read-Only Data Appropriately

`get_total_tips` and `get_withdrawable_balance` are cheap reads, but avoid polling them on every render. Cache the values and invalidate on confirmed tip or withdraw events.

## Monitor Events for Confirmations

Do not rely solely on transaction status for UI updates. Subscribe to `("tip", creator, token)` and `("withdraw", creator, token)` events to confirm state changes and drive notifications.

## Manage Roles Before Onboarding Creators

Grant the `Creator` role to a creator's address before they attempt to withdraw. A withdrawal from an address without the `Creator` role will fail with `Unauthorized`.

```bash
stellar contract invoke --id $CONTRACT_ID --source admin --network testnet \
  -- grant_role --caller ADMIN_ADDRESS --target CREATOR_ADDRESS --role Creator
```

## Use Locked Tips for Milestone-Based Payouts

When you want to release funds only after a condition is met (e.g., a project deadline), use `tip_locked` with an appropriate `unlock_timestamp`. This keeps funds in escrow until the time passes, without requiring any off-chain coordination.

## Keep the Token Whitelist Minimal

Only whitelist tokens your application actively supports. A large whitelist increases the attack surface and makes it harder to reason about which assets are in escrow.

## Test on Testnet Before Mainnet

Always validate the full flow — deploy, init, whitelist, tip, withdraw — on testnet before deploying to mainnet. Use the helper script in `scripts/deploy.sh` as a starting point.
