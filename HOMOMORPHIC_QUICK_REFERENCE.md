# Homomorphic Encryption - Quick Reference

## Module Structure

```
privacy/
├── homomorphic.rs              # Core encryption operations
├── key_management.rs           # Key lifecycle management
├── encrypted_operations.rs     # Privacy-preserving tip operations
├── contract_interface.rs       # Public contract functions
├── commitment.rs               # Existing commitment scheme
├── zk_proof.rs                 # Existing ZK proofs
└── mod.rs                      # Module exports
```

## Core Data Structures

### HomomorphicPublicKey
```rust
pub struct HomomorphicPublicKey {
    pub n: BytesN<32>,           // Modulus
    pub g: BytesN<32>,           // Generator
    pub version: u32,            // Key version
}
```

### EncryptedAmount
```rust
pub struct EncryptedAmount {
    pub ciphertext: BytesN<32>,
    pub randomness_commitment: BytesN<32>,
    pub key_version: u32,
    pub bit_length: u32,
}
```

### RangeProof
```rust
pub struct RangeProof {
    pub challenge: BytesN<32>,
    pub responses: Vec<BytesN<32>>,
    pub bit_length: u32,
}
```

## Public Contract Functions

### Initialization
```rust
pub fn init_homomorphic(
    env: Env,
    admin: Address,
    n: BytesN<32>,
    g: BytesN<32>,
    key_version: u32,
    rotation_interval: u64,
    max_key_age: u64,
) -> Result<(), Symbol>
```

### Create Encrypted Tip
```rust
pub fn tip_encrypted(
    env: Env,
    sender: Address,
    creator: Address,
    token: Address,
    amount: i128,
    randomness_seed: BytesN<32>,
    range_proof: RangeProof,
) -> Result<u64, Symbol>
```

### Get Encrypted Balance
```rust
pub fn get_encrypted_balance_for(
    env: Env,
    creator: Address,
    token: Address,
) -> Result<EncryptedAmount, Symbol>
```

### Reveal Encrypted Tip
```rust
pub fn reveal_encrypted_tip_amount(
    env: Env,
    tip_id: u64,
    decrypted_amount: i128,
    creator: Address,
) -> Result<(), Symbol>
```

### Rotate Key
```rust
pub fn rotate_encryption_key(
    env: Env,
    admin: Address,
    new_n: BytesN<32>,
    new_g: BytesN<32>,
    new_version: u32,
    reason: Symbol,
) -> Result<u32, Symbol>
```

### Get Public Key
```rust
pub fn get_public_key(env: Env) -> Result<HomomorphicPublicKey, Symbol>
```

### Check Status
```rust
pub fn is_encrypted_tips_enabled(env: Env) -> bool
```

### Aggregate Tips
```rust
pub fn aggregate_tips_encrypted(
    env: Env,
    encrypted_tips: Vec<EncryptedAmount>,
) -> Result<EncryptedAmount, Symbol>
```

### Compute Fee
```rust
pub fn compute_fee_encrypted(
    env: Env,
    encrypted_amount: EncryptedAmount,
    fee_basis_points: u32,
) -> Result<EncryptedAmount, Symbol>
```

## Core Operations

### Encrypt Amount
```rust
pub fn encrypt_amount(
    env: &Env,
    amount: i128,
    public_key: &HomomorphicPublicKey,
    randomness_seed: &BytesN<32>,
) -> Result<EncryptedAmount, &'static str>
```

### Aggregate Encrypted Amounts
```rust
pub fn aggregate_encrypted_amounts(
    env: &Env,
    amounts: &Vec<EncryptedAmount>,
) -> Result<EncryptedAmount, &'static str>
```

### Scalar Multiply
```rust
pub fn scalar_multiply_encrypted(
    encrypted: &EncryptedAmount,
    scalar: u64,
) -> Result<EncryptedAmount, &'static str>
```

### Verify Range Proof
```rust
pub fn verify_range_proof(
    env: &Env,
    encrypted: &EncryptedAmount,
    proof: &RangeProof,
) -> Result<(), &'static str>
```

### Verify Decryption Proof
```rust
pub fn verify_decryption_proof(
    env: &Env,
    encrypted: &EncryptedAmount,
    decrypted_value: i128,
    proof: &DecryptionProof,
) -> Result<(), &'static str>
```

## Key Management Operations

### Initialize
```rust
pub fn initialize_homomorphic(
    env: &Env,
    admin: &Address,
    public_key: HomomorphicPublicKey,
    config: KeyManagementConfig,
) -> Result<(), &'static str>
```

### Rotate Key
```rust
pub fn rotate_key(
    env: &Env,
    admin: &Address,
    new_key: HomomorphicPublicKey,
    reason: Symbol,
) -> Result<u32, &'static str>
```

### Get Current Key
```rust
pub fn get_current_public_key(env: &Env) -> Result<HomomorphicPublicKey, &'static str>
```

### Get Key by Version
```rust
pub fn get_public_key_by_version(
    env: &Env,
    version: u32,
) -> Result<HomomorphicPublicKey, &'static str>
```

### Check Enabled
```rust
pub fn is_homomorphic_enabled(env: &Env) -> bool
```

## Encrypted Operations

### Create Encrypted Tip
```rust
pub fn create_encrypted_tip(
    env: &Env,
    sender: &Address,
    creator: &Address,
    token: &Address,
    amount: i128,
    randomness_seed: &BytesN<32>,
    range_proof: &RangeProof,
) -> Result<u64, &'static str>
```

### Get Encrypted Balance
```rust
pub fn get_encrypted_balance(
    env: &Env,
    creator: &Address,
    token: &Address,
) -> Result<EncryptedBalance, &'static str>
```

### Compute Encrypted Fee
```rust
pub fn compute_encrypted_fee(
    encrypted_amount: &EncryptedAmount,
    fee_basis_points: u32,
) -> Result<EncryptedAmount, &'static str>
```

### Aggregate Encrypted Tips
```rust
pub fn aggregate_encrypted_tips(
    env: &Env,
    encrypted_tips: &Vec<EncryptedAmount>,
) -> Result<EncryptedAmount, &'static str>
```

### Reveal Encrypted Tip
```rust
pub fn reveal_encrypted_tip(
    env: &Env,
    tip_id: u64,
    decrypted_amount: i128,
    authorizer: &Address,
) -> Result<(), &'static str>
```

## Data Keys

```rust
pub enum DataKey {
    HomomorphicConfig,                    // HomomorphicConfig
    KeyManagementConfig,                  // KeyManagementConfig
    KeyHistory,                           // Vec<HomomorphicPublicKey>
    EncryptedBalance(Address, Address),   // EncryptedBalance
    EncryptedTip(u64),                    // EncryptedTip
    EncryptedTipCounter,                  // u64
    PrivacyNullifier(BytesN<32>),        // bool
}
```

## Events

```rust
// Initialization
("homomorphic_init", admin, key_version)

// Encrypted tip creation
("encrypted_tip", sender, creator, tip_id)

// Key rotation
("key_rotated", old_version, new_version)

// Tip reveal
("encrypted_tip_revealed", tip_id, decrypted_amount)

// Feature enable/disable
("homomorphic_enabled", admin)
("homomorphic_disabled", admin)
```

## Error Messages

| Error | Meaning |
|-------|---------|
| "homomorphic encryption not initialized" | Call init_homomorphic() first |
| "homomorphic encryption not enabled" | Feature is disabled |
| "cannot aggregate empty amounts" | Vector is empty |
| "mismatched key versions in aggregation" | Different key versions |
| "cannot encrypt negative amounts" | Amount < 0 |
| "tip amount must be positive" | Amount <= 0 |
| "range proof verification failed" | Invalid range proof |
| "only creator can reveal encrypted tip" | Wrong authorizer |
| "new key version must be greater than current" | Version not incremented |
| "public key version not found" | Invalid key version |

## Common Workflows

### 1. Initialize System
```rust
let public_key = HomomorphicPublicKey { n, g, version: 1 };
let config = KeyManagementConfig {
    rotation_interval: 86400 * 30,
    max_key_age: 86400 * 90,
    key_history_size: 10,
    rotation_requires_approval: true,
};
init_homomorphic(&env, &admin, public_key, config)?;
```

### 2. Create Encrypted Tip
```rust
let tip_id = tip_encrypted(
    env,
    sender,
    creator,
    token,
    1000,
    randomness_seed,
    range_proof,
)?;
```

### 3. Get Encrypted Balance
```rust
let encrypted_balance = get_encrypted_balance_for(
    &env,
    &creator,
    &token,
)?;
```

### 4. Compute Encrypted Fee
```rust
let encrypted_fee = compute_fee_encrypted(
    &env,
    encrypted_balance,
    1000,  // 10% = 1000 basis points
)?;
```

### 5. Aggregate Multiple Tips
```rust
let tips = vec![tip1, tip2, tip3];
let aggregated = aggregate_tips_encrypted(&env, &tips)?;
```

### 6. Reveal Amount
```rust
reveal_encrypted_tip_amount(
    &env,
    tip_id,
    1000,
    &creator,
)?;
```

### 7. Rotate Key
```rust
let new_version = rotate_encryption_key(
    &env,
    &admin,
    new_n,
    new_g,
    2,
    Symbol::new(&env, "scheduled_rotation"),
)?;
```

## Performance Tips

1. **Batch Operations**: Aggregate multiple tips in single transaction
2. **Lazy Evaluation**: Defer decryption until withdrawal
3. **Ciphertext Reuse**: Cache encrypted values
4. **Key History**: Keep only necessary versions (default: 10)
5. **Proof Verification**: Use Fiat-Shamir for efficiency

## Security Checklist

- [ ] Initialize with valid RSA-like modulus
- [ ] Set appropriate key rotation interval
- [ ] Verify all range proofs before accepting tips
- [ ] Audit all decryption operations
- [ ] Monitor key rotation events
- [ ] Validate authorization for all operations
- [ ] Test with edge cases (zero, max amounts)
- [ ] Verify nullifier tracking
- [ ] Check storage efficiency
- [ ] Monitor gas usage

## Testing

Run tests with:
```bash
cargo test --test homomorphic_encryption_tests
```

Test categories:
- Encryption tests
- Aggregation tests
- Proof tests
- Key management tests
- Operation tests
- Authorization tests
- Integration tests

## Documentation

- **HOMOMORPHIC_ENCRYPTION.md** - Architecture and security model
- **IMPLEMENTATION_GUIDE.md** - Detailed implementation guide
- **HOMOMORPHIC_COMMIT.md** - Comprehensive commit message
- **HOMOMORPHIC_SUMMARY.md** - Implementation summary
- **HOMOMORPHIC_QUICK_REFERENCE.md** - This file

## Support

For issues or questions:
1. Check HOMOMORPHIC_ENCRYPTION.md for architecture
2. Check IMPLEMENTATION_GUIDE.md for detailed guide
3. Check test cases for usage examples
4. Review error messages for troubleshooting
