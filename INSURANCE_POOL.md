# Tip Insurance Pool - Implementation Summary

## Overview
Implemented a decentralized insurance pool (Issue #185) that allows creators to gain coverage against failed transactions and other operational risks by contributing to a collective pool.

## Key Features

### 1. Collective Pool Mechanism
- Creators contribute whitelisted tokens to a collective insurance pool.
- Contributions earn coverage based on a configurable payout ratio.
- Automatic premium collection from tips received (optional, configurable).
- Admin can manage pool parameters (fees, ratios, limits).

### 2. Coverage and Premiums
- **Manual Contributions**: Creators can explicitly "buy" insurance by contributing tokens.
- **Automatic Premiums**: Optional small percentage (e.g., 0.1%) deducted from each tip to provide baseline coverage.
- **Payout Ratio**: Configurable ratio (e.g., 200%) that determines coverage limit relative to contributions.

### 3. Claims Management
- Creators can submit claims with proof (transaction hashes).
- Claims go through a lifecycle: `Pending` -> `Approved`/`Rejected` -> `Paid`.
- Batch processing for efficient admin management of multiple claims.
- Cooldown periods and active claim limits to prevent abuse.

### 4. Risk Mitigation
- **Min/Max Contributions**: Bounds on individual contributions.
- **Claim Cooldown**: Minimum time between claims per creator.
- **Max Active Claims**: Limit on simultaneous pending claims per creator.
- **Minimum Reserves**: Admin can only withdraw excess funds above a safety threshold (10% of total contributions).

## Technical Implementation

### Data Structures

#### `InsurancePoolConfig`
Stores global parameters like premium rates, payout ratios, and limits.

#### `InsurancePool`
Tracks total reserves, contributions, and claims paid for each token.

#### `InsuranceClaim`
Records details of individual claims, including amount, proof, and status.

### New Contract Methods

1. `insurance_set_config`: Initialize or update pool parameters (Admin only).
2. `insurance_contribute`: Manual token contribution by creator.
3. `insurance_submit_claim`: Creator submits a claim for failed TX.
4. `insurance_approve_claim`: Admin approves a pending claim.
5. `insurance_reject_claim`: Admin rejects a pending claim.
6. `insurance_pay_claim`: Payout funds for an approved claim.
7. `insurance_process_claims_batch`: Batch approve/pay multiple claims (Admin only).
8. `insurance_get_coverage`: Calculate current coverage for a creator.
9. `insurance_withdraw_excess`: Admin withdrawal of surplus reserves.

### Events
- `ins_cfg`: Pool configuration updated.
- `ins_con`: New contribution received.
- `clm_sub`: Claim submitted.
- `clm_app`: Claim approved.
- `clm_rej`: Claim rejected.
- `clm_paid`: Claim paid out.
- `clm_pro`: Batch processing results.

## Testing
- **Test File**: `contracts/tipjar/tests/insurance_tests.rs`
- **Coverage**:
  - Configuration management
  - Manual contributions and coverage calculation
  - Automatic premium accumulation from tips
  - Full claim lifecycle (Submit -> Approve -> Pay)
  - Batch processing validation
  - Security checks (unauthorized access, exceeding coverage, cooldowns)

## Documentation
- **API.md**: Added all insurance functions and error codes.
- **EVENTS.md**: Documented all new insurance-related events.
- **CONTRACT_SPEC.md**: Added insurance section to the functional spec.

## Error Codes
- `InsPoolNotCfg (52)`
- `ContributionTooLow (53)`
- `ContributionTooHigh (54)`
- `NoCoverage (55)`
- `ClaimNotApproved (56)`
- `ClaimAlreadyPaid (57)`
- `InsufficientReserves (58)`
- `ClaimCooldownActive (59)`
- `TooManyActiveClaims (60)`
- `ClaimNotFound (61)`
- `AlreadyContributed (62)`
- `InsuranceDisabled (63)`
- `PendingClaimExists (64)`
- `PayoutExceedsReserves (65)`
- `InvalidClaimAmount (66)`
- `AdmAppReq (67)`
