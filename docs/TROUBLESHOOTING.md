# Troubleshooting

Common issues and how to resolve them.

---

## Contract Errors

### `AlreadyInitialized` (code 1)

**Cause:** `init` was called more than once on the same contract instance.

**Fix:** `init` is a one-time setup. If you need to redeploy, deploy a new contract instance and call `init` on it.

---

### `TokenNotWhitelisted` (code 2)

**Cause:** The token address passed to `tip`, `tip_with_message`, `tip_batch`, or `tip_locked` has not been added to the whitelist.

**Fix:** Have the admin whitelist the token first:

```bash
stellar contract invoke --id $CONTRACT_ID --source admin --network testnet \
  -- add_token --admin ADMIN_ADDRESS --token TOKEN_ADDRESS
```

---

### `InvalidAmount` (code 3)

**Cause:** `amount` is zero or negative.

**Fix:** Validate the amount client-side before submitting. Amounts must be `> 0`.

---

### `NothingToWithdraw` (code 4)

**Cause:** The creator's withdrawable balance for the given token is zero.

**Fix:** Verify the balance before calling `withdraw`:

```bash
stellar contract invoke --id $CONTRACT_ID --network testnet \
  -- get_withdrawable_balance --creator CREATOR_ADDRESS --token TOKEN_ADDRESS
```

---

### `MessageTooLong` (code 5)

**Cause:** The message passed to `tip_with_message` exceeds 280 characters.

**Fix:** Truncate or validate the message length before calling the contract.

---

### `Unauthorized` (code 9)

**Cause:** The caller does not hold the required role for the operation.

Common scenarios:
- A creator tries to `withdraw` without the `Creator` role.
- A non-admin tries to call `add_token`, `grant_role`, or `revoke_role`.
- A non-admin/moderator tries to `pause` or `unpause`.

**Fix:** Have the admin grant the appropriate role:

```bash
stellar contract invoke --id $CONTRACT_ID --source admin --network testnet \
  -- grant_role --caller ADMIN_ADDRESS --target TARGET_ADDRESS --role Creator
```

---

### `BatchTooLarge` (code 11)

**Cause:** `tip_batch` was called with more than 50 entries.

**Fix:** Split the batch into chunks of 50 or fewer.

---

### `InsufficientBalance` (code 12)

**Cause:** The sender does not have enough tokens for one of the entries in a `tip_batch`.

**Fix:** Check the sender's token balance before building the batch. In a batch, this error is returned per-entry and does not affect other entries.

---

### `InvalidUnlockTime` (code 13)

**Cause:** The `unlock_timestamp` passed to `tip_locked` is not strictly in the future (i.e., it is ≤ the current ledger timestamp).

**Fix:** Use a timestamp that is at least a few seconds ahead of the current ledger time to account for submission latency.

---

### `TipStillLocked` (code 14)

**Cause:** `withdraw_locked` was called before the `unlock_timestamp` has elapsed.

**Fix:** Check the locked tip's `unlock_timestamp` and wait until the ledger time has passed it:

```bash
stellar contract invoke --id $CONTRACT_ID --network testnet \
  -- get_locked_tip --creator CREATOR_ADDRESS --tip_id TIP_ID
```

---

### `LockedTipNotFound` (code 15)

**Cause:** No locked tip exists for the given `(creator, tip_id)` pair. Either the ID is wrong or the tip has already been withdrawn.

**Fix:** Verify the `tip_id` returned by `tip_locked`. Once `withdraw_locked` succeeds, the record is deleted.

---

## Network and CLI Issues

### "Contract not found"

**Cause:** The `CONTRACT_ID` is incorrect or the contract was deployed to a different network.

**Fix:** Confirm the contract ID and network flag match:

```bash
stellar contract invoke --id $CONTRACT_ID --network testnet -- get_total_tips ...
```

### Transaction times out / not confirmed

**Cause:** Network congestion or an RPC node issue.

**Fix:** Increase the transaction fee and retry. In SDK code, poll `server.getTransaction(hash)` for up to 30 seconds before giving up.

### Simulation succeeds but submission fails

**Cause:** Ledger state changed between simulation and submission (e.g., another transaction consumed the sender's balance).

**Fix:** Re-simulate immediately before submitting. Do not cache simulation results across user interactions.

---

## Build Issues

### `error[E0463]: can't find crate for 'std'`

**Cause:** Building without the correct target.

**Fix:**

```bash
rustup target add wasm32v1-none
cargo build -p tipjar --target wasm32v1-none --release
```

### Tests fail with stale snapshots

**Cause:** Test snapshots in `test_snapshots/` are stale after a contract change.

**Fix:** Delete the snapshot directory and re-run tests to regenerate:

```bash
rm -rf contracts/tipjar/test_snapshots
cargo test -p tipjar
```
