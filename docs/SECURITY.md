# Security Considerations

This document covers security properties of the TipJar contract and guidance for integrators.

## Contract Security Properties

### Role-Based Access Control

All privileged operations are gated by roles stored in persistent contract storage:

| Operation | Required Role |
|---|---|
| `add_token` / `remove_token` | Admin |
| `grant_role` / `revoke_role` | Admin |
| `pause` / `unpause` | Admin or Moderator |
| `withdraw` | Creator |
| `withdraw_locked` | Creator |

The admin address is set once at `init` time and cannot be changed without revoking and re-granting roles. Protect the admin key accordingly.

### Token Whitelisting

Only tokens explicitly added by the admin can be used for tips. This prevents griefing attacks where an attacker tips with a worthless or malicious token contract.

### Authorization Requirements

Every state-changing call that moves funds requires `require_auth()` from the relevant address:

- `tip`, `tip_with_message`, `tip_batch`, `tip_locked` — sender must authorize
- `withdraw`, `withdraw_locked` — creator must authorize
- `add_token`, `remove_token`, `grant_role`, `revoke_role` — admin must authorize
- `pause`, `unpause` — admin or moderator must authorize

This means no party can move funds on behalf of another without an explicit on-chain authorization.

### Emergency Pause

The `pause` function halts all state-changing operations (`tip`, `tip_with_message`, `tip_batch`, `tip_locked`, `withdraw`, `withdraw_locked`). Read-only queries (`get_total_tips`, `get_withdrawable_balance`, `get_messages`, `get_locked_tip`, `get_top_tippers`, `get_top_creators`) remain available while paused.

Use pause as a circuit breaker if a vulnerability is discovered. Have a clear runbook for who can trigger it and under what conditions.

### Locked Tips

`tip_locked` enforces that `unlock_timestamp > current ledger timestamp` at the time of the call. The contract does not allow early withdrawal — `withdraw_locked` will fail with `TipStillLocked` until the ledger time has passed the unlock timestamp.

---

## Integrator Guidance

### Protect the Admin Key

The admin key controls token whitelisting and role management. Compromise of this key allows an attacker to whitelist malicious tokens or revoke creator roles.

- Store the admin secret key in a hardware security module (HSM) or secrets manager.
- Never commit secret keys to source control.
- Consider using a multisig account as the admin for high-value deployments.

### Validate Token Addresses

Before whitelisting a token, verify it is a legitimate Stellar Asset Contract (SAC) or a trusted custom token. A malicious token contract could behave unexpectedly during `transfer` calls.

### Do Not Trust Client-Supplied Amounts Blindly

Always validate `amount > 0` and that the sender has sufficient balance before submitting a tip transaction. The contract enforces these checks, but client-side validation avoids wasted fees.

### Avoid Storing Secret Keys in Frontend Code

Never embed a Stellar secret key in browser-side JavaScript. Use a wallet (e.g., Freighter) to sign transactions in the browser. For backend services, load keys from environment variables or a secrets manager at runtime.

### Monitor for Unexpected Pauses

If your application depends on the contract being active, monitor for `pause` events and alert your team immediately. Have a contingency plan for serving users while the contract is paused.

### Audit Before Mainnet Deployment

Have the contract code reviewed by a Soroban security specialist before deploying to mainnet with real funds. Pay particular attention to:

- Role assignment logic in `init` and `grant_role`
- Token transfer paths in `tip`, `withdraw`, and `withdraw_locked`
- The batch processing loop in `tip_batch`
