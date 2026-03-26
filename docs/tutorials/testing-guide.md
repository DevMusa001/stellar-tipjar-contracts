# Testing Your TipJar Integration

This guide covers how to write unit tests for the TipJar contract using Soroban's test framework, and how to run integration tests against testnet.

## Unit Testing with Soroban Test Framework

Soroban provides an in-process test environment (`Env::default()`) that runs the contract WASM natively without a live network. All existing tests live in `contracts/tipjar/src/lib.rs`.

### Run the Tests

```bash
cargo test -p tipjar
```

### Test Setup Pattern

Every test uses a shared `setup()` helper that:

1. Creates a default `Env` with `mock_all_auths()`
2. Registers two token contracts
3. Deploys and initializes the TipJar contract
4. Whitelists token 1

```rust
fn setup() -> (Env, Address, Address, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let token_admin = Address::generate(&env);
    let token_id_1 = env
        .register_stellar_asset_contract_v2(token_admin.clone())
        .address();
    let token_id_2 = env
        .register_stellar_asset_contract_v2(token_admin.clone())
        .address();

    let admin = Address::generate(&env);
    let contract_id = env.register(TipJarContract, ());
    let client = TipJarContractClient::new(&env, &contract_id);
    client.init(&admin);
    client.add_token(&admin, &token_id_1);

    (env, contract_id, token_id_1, token_id_2, admin)
}
```

### Writing a Basic Tip Test

```rust
#[test]
fn test_tip_transfers_tokens() {
    let (env, contract_id, token_id, _, _) = setup();
    let client = TipJarContractClient::new(&env, &contract_id);
    let token_admin = token::StellarAssetClient::new(&env, &token_id);
    let token_client = token::Client::new(&env, &token_id);

    let sender = Address::generate(&env);
    let creator = Address::generate(&env);
    token_admin.mint(&sender, &1_000);

    client.tip(&sender, &creator, &token_id, &500);

    assert_eq!(token_client.balance(&sender), 500);
    assert_eq!(token_client.balance(&contract_id), 500);
    assert_eq!(client.get_total_tips(&creator, &token_id), 500);
    assert_eq!(client.get_withdrawable_balance(&creator, &token_id), 500);
}
```

### Testing Error Cases

Use `try_*` methods to assert on expected errors without panicking the test:

```rust
#[test]
fn test_tip_zero_amount_rejected() {
    let (env, contract_id, token_id, _, _) = setup();
    let client = TipJarContractClient::new(&env, &contract_id);
    let sender = Address::generate(&env);
    let creator = Address::generate(&env);

    let result = client.try_tip(&sender, &creator, &token_id, &0);
    assert_eq!(
        result.unwrap_err().unwrap(),
        TipJarError::InvalidAmount.into()
    );
}
```

### Testing Events

```rust
#[test]
fn test_tip_emits_event() {
    let (env, contract_id, token_id, _, _) = setup();
    let client = TipJarContractClient::new(&env, &contract_id);
    let token_admin = token::StellarAssetClient::new(&env, &token_id);

    let sender = Address::generate(&env);
    let creator = Address::generate(&env);
    token_admin.mint(&sender, &1_000);

    client.tip(&sender, &creator, &token_id, &100);

    let events = env.events().all();
    let last = events.last().unwrap();
    let topics: soroban_sdk::Vec<soroban_sdk::Val> = last.1;
    let topic_sym: soroban_sdk::Symbol =
        soroban_sdk::FromVal::from_val(&env, &topics.get(0).unwrap());
    assert_eq!(topic_sym, soroban_sdk::Symbol::new(&env, "tip"));
}
```

### Advancing Ledger Time

For locked tip tests, advance the ledger timestamp:

```rust
fn advance_time(env: &Env, new_ts: u64) {
    env.ledger().with_mut(|li| {
        li.timestamp = new_ts;
    });
}

#[test]
fn test_withdraw_locked_after_unlock() {
    let (env, contract_id, token_id, _, admin) = setup();
    let client = TipJarContractClient::new(&env, &contract_id);
    let token_admin = token::StellarAssetClient::new(&env, &token_id);

    let sender = Address::generate(&env);
    let creator = Address::generate(&env);
    token_admin.mint(&sender, &1_000);

    let unlock_ts = env.ledger().timestamp() + 1000;
    let tip_id = client.tip_locked(&sender, &creator, &token_id, &500, &unlock_ts);

    client.grant_role(&admin, &creator, &Role::Creator);

    // Still locked — should fail
    assert!(client.try_withdraw_locked(&creator, &tip_id).is_err());

    // Advance past unlock
    advance_time(&env, unlock_ts + 1);

    // Now succeeds
    client.withdraw_locked(&creator, &tip_id);
}
```

## Testnet Integration Testing

For end-to-end validation, use the deploy script and invoke commands manually or via a shell script.

```bash
#!/usr/bin/env bash
set -euo pipefail

CONTRACT_ID="YOUR_CONTRACT_ID"
TOKEN_ADDRESS="YOUR_TOKEN_ADDRESS"
ADMIN=$(stellar keys address admin)
SENDER=$(stellar keys address sender)
CREATOR=$(stellar keys address creator)
NETWORK="testnet"

# Initialize (skip if already done)
stellar contract invoke --id $CONTRACT_ID --source admin --network $NETWORK \
  -- init --admin $ADMIN || echo "Already initialized"

# Whitelist token
stellar contract invoke --id $CONTRACT_ID --source admin --network $NETWORK \
  -- add_token --admin $ADMIN --token $TOKEN_ADDRESS

# Grant Creator role
stellar contract invoke --id $CONTRACT_ID --source admin --network $NETWORK \
  -- grant_role --caller $ADMIN --target $CREATOR --role Creator

# Send a tip
stellar contract invoke --id $CONTRACT_ID --source sender --network $NETWORK \
  -- tip --sender $SENDER --creator $CREATOR --token $TOKEN_ADDRESS --amount 1000000

# Verify balance
BALANCE=$(stellar contract invoke --id $CONTRACT_ID --network $NETWORK \
  -- get_withdrawable_balance --creator $CREATOR --token $TOKEN_ADDRESS)
echo "Creator balance: $BALANCE"

# Withdraw
stellar contract invoke --id $CONTRACT_ID --source creator --network $NETWORK \
  -- withdraw --creator $CREATOR --token $TOKEN_ADDRESS

echo "Integration test passed"
```

## Test Checklist

Before submitting a PR, verify:

- [ ] All unit tests pass: `cargo test -p tipjar`
- [ ] Error paths are covered with `try_*` assertions
- [ ] Events are verified for state-changing calls
- [ ] Time-sensitive tests use `advance_time` rather than real delays
- [ ] Testnet smoke test runs without errors
