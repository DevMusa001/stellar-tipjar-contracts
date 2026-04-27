# Homomorphic Encryption for Tip Privacy

## Overview

This document describes the homomorphic encryption implementation for the Stellar TipJar contract, enabling privacy-preserving computations on encrypted tip amounts.

## Architecture

### Core Components

#### 1. **Homomorphic Encryption Module** (`privacy/homomorphic.rs`)

Implements additive homomorphic encryption with the following properties:

- **Semantic Security**: IND-CPA security under decisional composite residuosity assumption
- **Additive Property**: `E(m1) * E(m2) = E(m1 + m2)` (ciphertext multiplication = plaintext addition)
- **Scalar Multiplication**: `E(m)^k = E(k*m)` (ciphertext exponentiation = plaintext scaling)
- **Range Proofs**: Verify encrypted values are within valid ranges without decryption

**Key Data Structures:**

```rust
pub struct HomomorphicPublicKey {
    pub n: BytesN<32>,           // Modulus (RSA-like)
    pub g: BytesN<32>,           // Generator
    pub version: u32,            // Key version for rotation
}

pub struct EncryptedAmount {
    pub ciphertext: BytesN<32>,
    pub randomness_commitment: BytesN<32>,
    pub key_version: u32,
    pub bit_length: u32,
}

pub struct RangeProof {
    pub challenge: BytesN<32>,
    pub responses: Vec<BytesN<32>>,
    pub bit_length: u32,
}
```

**Core Functions:**

- `encrypt_amount()` - Encrypt plaintext amount
- `aggregate_encrypted_amounts()` - Add encrypted amounts without decryption
- `scalar_multiply_encrypted()` - Scale encrypted amount by plaintext scalar
- `verify_range_proof()` - Verify amount is in valid range
- `verify_decryption_proof()` - Zero-knowledge proof of correct decryption

#### 2. **Key Management Module** (`privacy/key_management.rs`)

Manages encryption keys with versioning and rotation support:

**Features:**

- Public key initialization and storage
- Key rotation with version tracking
- Key history maintenance for decryption of old ciphertexts
- Admin-only access control
- Key expiration and lifecycle management

**Key Functions:**

- `initialize_homomorphic()` - Initialize with public key
- `rotate_key()` - Rotate to new key version
- `get_current_public_key()` - Get active key
- `get_public_key_by_version()` - Retrieve historical key
- `is_homomorphic_enabled()` - Check if feature is active

#### 3. **Encrypted Operations Module** (`privacy/encrypted_operations.rs`)

Implements privacy-preserving tip operations:

**Operations:**

- `create_encrypted_tip()` - Create encrypted tip with range proof
- `update_encrypted_balance()` - Aggregate encrypted amounts
- `get_encrypted_balance()` - Retrieve encrypted balance
- `compute_encrypted_fee()` - Calculate fees on encrypted amounts
- `aggregate_encrypted_tips()` - Batch aggregation
- `reveal_encrypted_tip()` - Authorized decryption

#### 4. **Contract Interface** (`privacy/contract_interface.rs`)

Public contract functions for homomorphic operations:

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

## Security Model

### Encryption Scheme

The implementation uses a Paillier-like additive homomorphic encryption scheme:

1. **Key Generation**: Generate RSA-like modulus `n = p*q` and generator `g`
2. **Encryption**: `c = g^m * r^n mod n^2` (simplified to hash-based for Soroban)
3. **Decryption**: Requires private key (held by authorized parties)
4. **Homomorphic Addition**: `c1 * c2 = g^(m1+m2) * (r1*r2)^n mod n^2`

### Privacy Guarantees

1. **Semantic Security**: Ciphertexts reveal no information about plaintexts
2. **Additive Privacy**: Aggregations don't reveal individual amounts
3. **Range Privacy**: Range proofs verify bounds without decryption
4. **Key Rotation**: Old ciphertexts remain secure after key rotation

### Threat Model

**Protected Against:**

- Passive eavesdropping on encrypted amounts
- Inference attacks on individual tip amounts
- Unauthorized balance disclosure
- Replay attacks (via nullifiers)

**Not Protected Against:**

- Timing attacks (mitigated by constant-time operations)
- Side-channel attacks (requires hardware-level protection)
- Collusion between multiple parties with decryption keys

## Usage Examples

### 1. Initialize Homomorphic Encryption

```rust
let public_key = HomomorphicPublicKey {
    n: BytesN::from_array(&env, &[...]),
    g: BytesN::from_array(&env, &[...]),
    version: 1,
};

let config = KeyManagementConfig {
    rotation_interval: 86400 * 30,  // 30 days
    max_key_age: 86400 * 90,        // 90 days
    key_history_size: 10,
    rotation_requires_approval: true,
};

init_homomorphic(&env, &admin, public_key, config)?;
```

### 2. Create Encrypted Tip

```rust
let randomness_seed = BytesN::from_array(&env, &[...]);
let range_proof = RangeProof {
    challenge: BytesN::from_array(&env, &[...]),
    responses: vec![...],
    bit_length: 64,
};

let tip_id = tip_encrypted(
    env,
    sender,
    creator,
    token,
    1000,  // amount
    randomness_seed,
    range_proof,
)?;
```

### 3. Aggregate Encrypted Balances

```rust
let encrypted_balance = get_encrypted_balance_for(&env, &creator, &token)?;

// Compute encrypted fee (10% = 1000 basis points)
let encrypted_fee = compute_fee_encrypted(
    &env,
    encrypted_balance.clone(),
    1000,
)?;

// Aggregate multiple tips
let tips = vec![tip1, tip2, tip3];
let aggregated = aggregate_tips_encrypted(&env, &tips)?;
```

### 4. Reveal Encrypted Amount

```rust
reveal_encrypted_tip_amount(
    &env,
    tip_id,
    1000,  // decrypted amount
    &creator,
)?;
```

### 5. Rotate Encryption Key

```rust
let new_key = HomomorphicPublicKey {
    n: BytesN::from_array(&env, &[...]),
    g: BytesN::from_array(&env, &[...]),
    version: 2,
};

let new_version = rotate_encryption_key(
    &env,
    &admin,
    new_key.n,
    new_key.g,
    2,
    Symbol::new(&env, "scheduled_rotation"),
)?;
```

## Integration with TipJar Contract

### Data Storage

New DataKey variants for homomorphic encryption:

```rust
pub enum DataKey {
    // ... existing keys ...
    HomomorphicConfig,                    // Configuration
    KeyManagementConfig,                  // Key management settings
    KeyHistory,                           // Historical public keys
    EncryptedBalance(Address, Address),   // (creator, token)
    EncryptedTip(u64),                    // Encrypted tip by ID
    EncryptedTipCounter,                  // Counter for tip IDs
    PrivacyNullifier(BytesN<32>),        // Nullifier tracking
}
```

### Event Emissions

Events for audit trail and monitoring:

```rust
// Initialization
env.events().publish(
    (Symbol::new(env, "homomorphic_init"),),
    (admin, key_version),
);

// Encrypted tip creation
env.events().publish(
    (Symbol::new(env, "encrypted_tip"),),
    (sender, creator, tip_id),
);

// Key rotation
env.events().publish(
    (Symbol::new(env, "key_rotated"),),
    (old_version, new_version),
);

// Tip reveal
env.events().publish(
    (Symbol::new(env, "encrypted_tip_revealed"),),
    (tip_id, decrypted_amount),
);
```

## Performance Considerations

### Gas Optimization

1. **Batch Operations**: Aggregate multiple encrypted amounts in single transaction
2. **Lazy Evaluation**: Defer decryption until withdrawal
3. **Ciphertext Reuse**: Cache encrypted values across operations
4. **Proof Verification**: Use Fiat-Shamir for non-interactive proofs

### Storage Efficiency

- Encrypted amounts: 32 bytes (ciphertext) + 32 bytes (randomness) = 64 bytes
- Range proofs: Challenge (32) + responses (variable) + metadata (8)
- Key history: Maintain last 10 versions (~320 bytes)

### Scalability

- Supports unlimited encrypted tips per creator
- Batch aggregation reduces per-tip overhead
- Key rotation doesn't require re-encryption of existing ciphertexts

## Security Audit Checklist

- [ ] Verify Paillier scheme implementation correctness
- [ ] Audit range proof verification logic
- [ ] Test key rotation without data loss
- [ ] Validate nullifier uniqueness
- [ ] Check for timing side-channels
- [ ] Verify access control on decryption
- [ ] Test edge cases (zero amounts, overflow)
- [ ] Validate Fiat-Shamir challenge generation
- [ ] Audit storage access patterns
- [ ] Test key history maintenance

## Future Enhancements

1. **Threshold Cryptography**: Require multiple parties for decryption
2. **Encrypted Comparisons**: Support conditional logic on encrypted values
3. **Secure Multi-Party Computation**: Aggregate across multiple contracts
4. **Bulletproofs**: More efficient range proofs
5. **Lattice-Based Encryption**: Post-quantum security

## References

- Paillier, P. (1999). "Public-Key Cryptosystems Based on Composite Degree Residuosity Classes"
- Fiat, A., & Shamir, A. (1986). "How to Prove Yourself: Practical Solutions to Identification and Signature Problems"
- Soroban SDK Documentation: https://developers.stellar.org/docs/learn/soroban

## License

MIT License - See LICENSE file for details
