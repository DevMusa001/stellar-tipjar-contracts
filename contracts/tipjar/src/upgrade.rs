/// Upgrade module — documents the upgrade mechanism and re-exports the
/// types callers need to invoke `TipJarContract::upgrade`.
///
/// # How Soroban upgrades work
///
/// `env.deployer().update_current_contract_wasm(new_wasm_hash)` atomically
/// swaps the executing contract's WASM bytecode for the one identified by
/// `new_wasm_hash` (which must already be uploaded to the network via
/// `stellar contract upload`).  All instance and persistent storage entries
/// are preserved by the host — no migration step is required for additive
/// changes.
///
/// # Upgrade flow
///
/// 1. Upload new WASM:
///    ```bash
///    stellar contract upload --wasm target/wasm32v1-none/release/tipjar.wasm \
///        --source <admin-key> --network testnet
///    # prints: <new_wasm_hash>
///    ```
/// 2. Invoke upgrade (Admin only):
///    ```bash
///    stellar contract invoke --id <contract_id> \
///        --source <admin-key> --network testnet \
///        -- upgrade --admin <admin_address> --new_wasm_hash <new_wasm_hash>
///    ```
/// 3. Verify:
///    ```bash
///    stellar contract invoke --id <contract_id> --network testnet \
///        -- get_version
///    # returns incremented version number
///    ```
///
/// # Backward compatibility rules
///
/// - Adding new `DataKey` variants is safe (old keys are unaffected).
/// - Adding new contract functions is safe.
/// - Removing or renaming existing `DataKey` variants will orphan stored data.
/// - Changing the type of an existing `DataKey` value requires a migration
///   function in the new WASM that reads the old layout and rewrites it.
pub use soroban_sdk::BytesN;
