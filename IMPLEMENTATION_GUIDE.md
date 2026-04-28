# Homomorphic Encryption Implementation Guide

## Overview

This guide provides a comprehensive walkthrough of the homomorphic encryption implementation for privacy-preserving tip computations in the Stellar TipJar contract.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    Contract Interface                        │
│              (contract_interface.rs)                         │
│  - init_homomorphic()                                        │
│  - tip_encrypted()                                           │
│  - get_encrypted_balance_for()                               │
│  - reveal_encrypted_tip_amount()                             │
│  - rotate_encryption_key()                                   │
└────────────────────┬────────────────────────────────────────┘
                     │
        ┌────────────┴────────────┐
        │                         │
┌───────▼──────────────┐  ┌──────▼──────────────┐
│ Encrypted Operations │  │  Key Management    │
│ (encrypted_ops.rs)   │  │ (key_management.rs)│
│                      │  │                    │
│ - create_encrypted   │  │ - initialize       │
│   _tip()             │  │ - rotate_key()     │
│ - get_encrypted      │  │ - get_public_key() │
│   _balance()         │  │ - verify_validity()│
│ - compute_encrypted  │  │                    │
│   _fee()             │  │                    │
│ - aggregate_encrypted│  │                    │
│   _tips()            │  │                    │
│ - reveal_encrypted   │  │                    │
│   _tip()             │  │                    │
└───────┬──────────────┘  └──────┬─────────────┘
        │                        │
        └────────────┬───────────┘
                     │
        ┌────────────▼────────────┐
        │  Homomorphic Encryption │
        │   (homomorphic.rs)      │
        │                         │
        │ - encrypt_amount()      │
        │ - aggregate_encrypted   │
        │   _amounts()            │
        │ - scalar_multiply       │
        │   _encrypted()          │
        │ - verify_range_proof()  │
        │ - verify_decryption     │
        │   _proof()              │
        └─────────────────────────┘
```

## Module Breakdown

### 1. Homomorphic Encryption Module (`homomorphic.rs`)

**Purpose**: Core cryptographic operations for additive homomorphic encryption.

**Key Structures**:

```rust
HomomorphicPublicKey {
    n: BytesN<32>,      // Modulus
    g: BytesN<32>,      // Generator
    version: u32,       // Key version
}

EncryptedAmount {
    ciphertext: BytesN<32>,
    randomness_commitment: BytesN<32>,
    key_version: u32,
    bit_length: u32,
}

RangeProof {
    challenge: BytesN<32>,
    responses: Vec<BytesN<32>>,
    bit_length: u32,
}

DecryptionProof {
    value_commitment: BytesN<32>,
    challenge: BytesN<32>,
    response: BytesN<32>,
}
```

**Core Functions**:

1. **`encrypt_amount()`**
   - Input: plaintext amount, public key, randomness seed
   - Output: EncryptedAmount
   - Process:
     - Determine bit-length from amount
     - Generate randomness from seed
     - Compute ciphertext: hash(n || amount || randomness)
     - Return encrypted structure

2. **`aggregate_encrypted_amounts()`**
   - Input: vector of EncryptedAmount
   - Output: aggregated EncryptedAmount
   - Process:
     - Verify all amounts use same key version
     - XOR ciphertexts (simplified homomorphic operation)
     - XOR randomness commitments
     - Return aggregated result

3. **`scalar_multiply_encrypted()`**
   - Input: EncryptedAmount, scalar
   - Output: scaled EncryptedAmount
   - Process:
     - Multiply ciphertext bytes by scalar
     - Multiply randomness commitment by scalar
     - Return scaled result

4. **`verify_range_proof()`**
   - Input: EncryptedAmount, RangeProof
   - Output: Result<(), &str>
   - Process:
     - Verify bit-length consistency
     - Verify bit-length bounds (32-128)
     - Recompute Fiat-Shamir challenge
     - Verify challenge matches proof

5. **`verify_decryption_proof()`**
   - Input: EncryptedAmount, decrypted_value, DecryptionProof
   - Output: Result<(), &str>
   - Process:
     - Verify value commitment
     - Recompute challenge
     - Verify challenge matches proof

### 2. Key Management Module (`key_management.rs`)

**Purpose**: Manage encryption keys with versioning and rotation.

**Key Structures**:

```rust
KeyManagementConfig {
    rotation_interval: u64,
    max_key_age: u64,
    key_history_size: u32,
    rotation_requires_approval: bool,
}

KeyRotationEvent {
    old_version: u32,
    new_version: u32,
    timestamp: u64,
    reason: Symbol,
}
```

**Core Functions**:

1. **`initialize_homomorphic()`**
   - Initializes homomorphic encryption system
   - Stores public key and configuration
   - Creates initial key history
   - Emits initialization event

2. **`rotate_key()`**
   - Rotates to new public key
   - Maintains key history
   - Trims history if needed
   - Emits rotation event

3. **`get_current_public_key()`**
   - Returns active public key
   - Used for new encryptions

4. **`get_public_key_by_version()`**
   - Retrieves historical key by version
   - Used for decryption of old ciphertexts

5. **`is_homomorphic_enabled()`**
   - Checks if feature is active
   - Used for feature gating

### 3. Encrypted Operations Module (`encrypted_operations.rs`)

**Purpose**: Implement privacy-preserving tip operations.

**Key Structures**:

```rust
EncryptedTip {
    id: u64,
    sender: Address,
    creator: Address,
    token: Address,
    encrypted_amount: EncryptedAmount,
    created_at: u64,
    revealed: bool,
}

EncryptedBalance {
    creator: Address,
    token: Address,
    encrypted_amount: EncryptedAmount,
    last_updated: u64,
    tip_count: u32,
}
```

**Core Functions**:

1. **`create_encrypted_tip()`**
   - Creates encrypted tip with range proof
   - Stores encrypted tip record
   - Updates encrypted balance
   - Emits event

2. **`update_encrypted_balance()`**
   - Aggregates new amount into balance
   - Creates new balance if needed
   - Updates tip count and timestamp

3. **`get_encrypted_balance()`**
   - Retrieves encrypted balance for creator/token
   - Returns full EncryptedBalance record

4. **`compute_encrypted_fee()`**
   - Scales encrypted amount by fee percentage
   - Maintains encryption properties

5. **`aggregate_encrypted_tips()`**
   - Batch aggregation of multiple tips
   - Useful for leaderboard computations

6. **`reveal_encrypted_tip()`**
   - Authorized decryption of tip amount
   - Only creator can reveal
   - Marks tip as revealed

### 4. Contract Interface Module (`contract_interface.rs`)

**Purpose**: Expose public contract functions.

**Public Functions**:

```rust
pub fn init_homomorphic(env, admin, n, g, key_version, rotation_interval, max_key_age)
pub fn tip_encrypted(env, sender, creator, token, amount, randomness_seed, range_proof)
pub fn get_encrypted_balance_for(env, creator, token)
pub fn reveal_encrypted_tip_amount(env, tip_id, decrypted_amount, creator)
pub fn rotate_encryption_key(env, admin, new_n, new_g, new_version, reason)
pub fn get_public_key(env)
pub fn is_encrypted_tips_enabled(env)
pub fn aggregate_tips_encrypted(env, encrypted_tips)
pub fn compute_fee_encrypted(env, encrypted_amount, fee_basis_points)
```

## Data Flow

### Creating an Encrypted Tip

```
1. Client generates randomness seed
2. Client creates range proof for amount
3. Client calls tip_encrypted(sender, creator, token, amount, seed, proof)
4. Contract verifies authorization (sender.require_auth())
5. Contract retrieves current public key
6. Contract encrypts amount: encrypt_amount(amount, pubkey, seed)
7. Contract verifies range proof: verify_range_proof(encrypted, proof)
8. Contract stores encrypted tip: EncryptedTip { ... }
9. Contract updates encrypted balance: aggregate_encrypted_amounts(...)
10. Contract emits event: ("encrypted_tip", sender, creator, tip_id)
11. Return tip_id to client
```

### Aggregating Encrypted Balances

```
1. Client calls get_encrypted_balance_for(creator, token)
2. Contract retrieves EncryptedBalance from storage
3. Contract returns encrypted_amount (no decryption)
4. Client can aggregate multiple balances without decryption
5. Client can compute fees on encrypted amounts
6. Client can verify range proofs
```

### Revealing Encrypted Amount

```
1. Creator calls reveal_encrypted_tip_amount(tip_id, amount, creator)
2. Contract verifies authorization (creator.require_auth())
3. Contract retrieves encrypted tip
4. Contract verifies creator matches
5. Contract marks tip as revealed
6. Contract emits event: ("encrypted_tip_revealed", tip_id, amount)
7. Return success
```

### Rotating Encryption Key

```
1. Admin calls rotate_encryption_key(admin, new_n, new_g, new_version, reason)
2. Contract verifies authorization (admin.require_auth())
3. Contract verifies new_version > current_version
4. Contract updates HomomorphicConfig with new key
5. Contract appends new key to KeyHistory
6. Contract trims history if needed
7. Contract emits event: ("key_rotated", old_version, new_version)
8. Return new_version
```

## Storage Layout

### Instance Storage (Fast, Limited)

```
DataKey::HomomorphicConfig -> HomomorphicConfig
DataKey::KeyManagementConfig -> KeyManagementConfig
DataKey::KeyHistory -> Vec<HomomorphicPublicKey>
DataKey::EncryptedTipCounter -> u64
```

### Persistent Storage (Slower, Unlimited)

```
DataKey::EncryptedTip(tip_id) -> EncryptedTip
DataKey::EncryptedBalance(creator, token) -> EncryptedBalance
DataKey::PrivacyNullifier(nullifier) -> bool
```

## Security Considerations

### 1. Key Management

- **Rotation**: Keys should be rotated periodically (e.g., every 30 days)
- **History**: Keep last 10 key versions for decryption of old ciphertexts
- **Expiration**: Enforce maximum key age (e.g., 90 days)
- **Access Control**: Only admin can rotate keys

### 2. Encryption

- **Randomness**: Use cryptographically secure randomness for each encryption
- **Bit-Length**: Enforce reasonable bit-length bounds (32-128 bits)
- **Range Proofs**: Verify all encrypted amounts have valid range proofs
- **Nullifiers**: Track nullifiers to prevent double-spend

### 3. Decryption

- **Authorization**: Only authorized parties can decrypt
- **Proofs**: Require zero-knowledge proofs of correct decryption
- **Audit Trail**: Emit events for all decryption operations
- **Logging**: Log all decryption attempts

### 4. Aggregation

- **Key Version**: Verify all amounts use same key version before aggregation
- **Overflow**: Ensure aggregation doesn't overflow
- **Consistency**: Maintain consistency across multiple aggregations

## Testing Strategy

### Unit Tests

1. **Encryption Tests**
   - Test encrypt_amount() with various inputs
   - Test edge cases (zero, max, negative)
   - Verify ciphertext uniqueness

2. **Aggregation Tests**
   - Test aggregate_encrypted_amounts()
   - Verify homomorphic property
   - Test with multiple amounts

3. **Proof Tests**
   - Test verify_range_proof()
   - Test verify_decryption_proof()
   - Test invalid proofs

4. **Key Management Tests**
   - Test initialize_homomorphic()
   - Test rotate_key()
   - Test key history

### Integration Tests

1. **Full Flow Tests**
   - Initialize -> Encrypt -> Aggregate -> Reveal
   - Verify all state changes

2. **Concurrent Tests**
   - Multiple creators creating tips
   - Concurrent balance updates
   - Concurrent key rotations

3. **Privacy Tests**
   - Verify ciphertexts don't leak information
   - Verify aggregation doesn't reveal amounts
   - Verify range proofs work correctly

## Performance Optimization

### Gas Optimization

1. **Batch Operations**: Aggregate multiple tips in single transaction
2. **Lazy Evaluation**: Defer decryption until withdrawal
3. **Ciphertext Reuse**: Cache encrypted values
4. **Proof Verification**: Use Fiat-Shamir for efficiency

### Storage Optimization

1. **Compact Representation**: Use 32-byte ciphertexts
2. **History Trimming**: Keep only last 10 key versions
3. **Lazy Loading**: Load encrypted balances only when needed

## Deployment Checklist

- [ ] All tests pass
- [ ] Code review completed
- [ ] Security audit passed
- [ ] Documentation reviewed
- [ ] Key generation completed
- [ ] Initial public key configured
- [ ] Key management config set
- [ ] Feature flag enabled
- [ ] Monitoring configured
- [ ] Rollback plan prepared

## Troubleshooting

### Issue: "homomorphic encryption not initialized"

**Solution**: Call `init_homomorphic()` with valid parameters before using encrypted tips.

### Issue: "mismatched key versions in aggregation"

**Solution**: Ensure all encrypted amounts use the same key version. Rotate keys before aggregating amounts from different versions.

### Issue: "range proof verification failed"

**Solution**: Verify that the range proof was generated correctly for the encrypted amount. Check bit-length bounds.

### Issue: "only creator can reveal encrypted tip"

**Solution**: Only the creator address can reveal encrypted tips. Use the correct creator address.

## References

- Paillier, P. (1999). "Public-Key Cryptosystems Based on Composite Degree Residuosity Classes"
- Fiat, A., & Shamir, A. (1986). "How to Prove Yourself: Practical Solutions to Identification and Signature Problems"
- Soroban SDK: https://developers.stellar.org/docs/learn/soroban
