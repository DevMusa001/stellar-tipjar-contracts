//! Gas estimation integration tests for TipJar contract operations.
//!
//! Each test function measures the CPU instructions and memory bytes consumed
//! by a single contract invocation using the Soroban test environment's
//! `env.budget()` API.  Setup overhead is excluded by resetting the budget
//! to unlimited during setup and to default immediately before the measured call.
//!
//! After all measurements are collected the results are written to
//! `gas-estimates.json` in the workspace root so the `gas-estimator` CLI can
//! read and display them.
//!
//! Run with:
//!   cargo test -p gas-estimator --test estimate -- --nocapture

extern crate std;

use chrono::Utc;
use gas_estimator::{
    generate_comparisons, generate_suggestions, make_estimate, BatchEstimate,
    EstimationReport, GasEstimate, compute_cost_stroops, stroops_to_xlm,
};
use soroban_sdk::{
    testutils::{Address as _, Ledger as _},
    token, Address, Env, Vec as SorobanVec,
};
use tipjar::{TipJarContract, TipJarContractClient, TipRecipient};

// ── Shared setup ──────────────────────────────────────────────────────────────

/// Registers the TipJar contract and a whitelisted mock token.
/// Returns `(env, contract_id, token_id, admin)`.
///
/// Budget is reset to unlimited so setup costs do not pollute measurements.
fn setup() -> (Env, Address, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let token_admin = Address::generate(&env);
    let token_id = env
        .register_stellar_asset_contract_v2(token_admin.clone())
        .address();

    let admin = Address::generate(&env);
    let contract_id = env.register(TipJarContract, ());
    let client = TipJarContractClient::new(&env, &contract_id);
    client.init(&admin);
    client.add_token(&admin, &token_id);

    (env, contract_id, token_id, admin)
}

/// Mint `amount` tokens to `recipient` using the stellar asset admin client.
fn mint(env: &Env, token_id: &Address, recipient: &Address, amount: i128) {
    token::StellarAssetClient::new(env, token_id).mint(recipient, &amount);
}

// ── Individual measurements ───────────────────────────────────────────────────

fn measure_tip_cold() -> GasEstimate {
    let (env, contract_id, token_id, _) = setup();
    let client = TipJarContractClient::new(&env, &contract_id);
    let sender = Address::generate(&env);
    let creator = Address::generate(&env);
    mint(&env, &token_id, &sender, 1_000_000);

    env.budget().reset_default();
    client.tip(&sender, &creator, &token_id, &1_000_000);
    let cpu = env.budget().cpu_instruction_count();
    let mem = env.budget().memory_bytes_count();

    println!("[GAS] tip (cold)  cpu={cpu}  mem={mem}");
    make_estimate("tip", "cold", cpu, mem)
}

fn measure_tip_warm() -> GasEstimate {
    let (env, contract_id, token_id, _) = setup();
    let client = TipJarContractClient::new(&env, &contract_id);
    let sender = Address::generate(&env);
    let creator = Address::generate(&env);
    mint(&env, &token_id, &sender, 2_000_000);

    // Warm up storage — not measured.
    client.tip(&sender, &creator, &token_id, &1_000);

    env.budget().reset_default();
    client.tip(&sender, &creator, &token_id, &1_000);
    let cpu = env.budget().cpu_instruction_count();
    let mem = env.budget().memory_bytes_count();

    println!("[GAS] tip (warm)  cpu={cpu}  mem={mem}");
    make_estimate("tip", "warm", cpu, mem)
}

fn measure_tip_with_fee_cold() -> GasEstimate {
    let (env, contract_id, token_id, _) = setup();
    let client = TipJarContractClient::new(&env, &contract_id);
    let sender = Address::generate(&env);
    let creator = Address::generate(&env);
    mint(&env, &token_id, &sender, 1_000_000);

    env.budget().reset_default();
    // congestion = 0 (low congestion)
    client.tip_with_fee(&sender, &creator, &token_id, &1_000_000, &0u32);
    let cpu = env.budget().cpu_instruction_count();
    let mem = env.budget().memory_bytes_count();

    println!("[GAS] tip_with_fee (cold)  cpu={cpu}  mem={mem}");
    make_estimate("tip_with_fee", "cold", cpu, mem)
}

fn measure_withdraw_warm() -> GasEstimate {
    let (env, contract_id, token_id, _) = setup();
    let client = TipJarContractClient::new(&env, &contract_id);
    let sender = Address::generate(&env);
    let creator = Address::generate(&env);
    mint(&env, &token_id, &sender, 1_000_000);

    // Pre-state: creator has a balance — not measured.
    client.tip(&sender, &creator, &token_id, &1_000_000);

    env.budget().reset_default();
    client.withdraw(&creator, &token_id);
    let cpu = env.budget().cpu_instruction_count();
    let mem = env.budget().memory_bytes_count();

    println!("[GAS] withdraw (warm)  cpu={cpu}  mem={mem}");
    make_estimate("withdraw", "warm", cpu, mem)
}

fn measure_get_withdrawable_balance_warm() -> GasEstimate {
    let (env, contract_id, token_id, _) = setup();
    let client = TipJarContractClient::new(&env, &contract_id);
    let sender = Address::generate(&env);
    let creator = Address::generate(&env);
    mint(&env, &token_id, &sender, 1_000);
    client.tip(&sender, &creator, &token_id, &1_000);

    env.budget().reset_default();
    client.get_withdrawable_balance(&creator, &token_id);
    let cpu = env.budget().cpu_instruction_count();
    let mem = env.budget().memory_bytes_count();

    println!("[GAS] get_withdrawable_balance (warm)  cpu={cpu}  mem={mem}");
    make_estimate("get_withdrawable_balance", "warm", cpu, mem)
}

fn measure_get_total_tips_warm() -> GasEstimate {
    let (env, contract_id, token_id, _) = setup();
    let client = TipJarContractClient::new(&env, &contract_id);
    let sender = Address::generate(&env);
    let creator = Address::generate(&env);
    mint(&env, &token_id, &sender, 1_000);
    client.tip(&sender, &creator, &token_id, &1_000);

    env.budget().reset_default();
    client.get_total_tips(&creator, &token_id);
    let cpu = env.budget().cpu_instruction_count();
    let mem = env.budget().memory_bytes_count();

    println!("[GAS] get_total_tips (warm)  cpu={cpu}  mem={mem}");
    make_estimate("get_total_tips", "warm", cpu, mem)
}

fn measure_tip_split_3() -> GasEstimate {
    let (env, contract_id, token_id, _) = setup();
    let client = TipJarContractClient::new(&env, &contract_id);
    let sender = Address::generate(&env);
    mint(&env, &token_id, &sender, 1_000_000);

    let mut recipients = SorobanVec::new(&env);
    recipients.push_back(TipRecipient {
        creator: Address::generate(&env),
        percentage: 5_000, // 50%
    });
    recipients.push_back(TipRecipient {
        creator: Address::generate(&env),
        percentage: 3_000, // 30%
    });
    recipients.push_back(TipRecipient {
        creator: Address::generate(&env),
        percentage: 2_000, // 20%
    });

    env.budget().reset_default();
    client.tip_split(&sender, &token_id, &recipients, &1_000_000);
    let cpu = env.budget().cpu_instruction_count();
    let mem = env.budget().memory_bytes_count();

    println!("[GAS] tip_split (3 recipients, cold)  cpu={cpu}  mem={mem}");
    make_estimate("tip_split", "cold", cpu, mem)
}

fn measure_get_leaderboard_10() -> GasEstimate {
    let (env, contract_id, token_id, _) = setup();
    let client = TipJarContractClient::new(&env, &contract_id);
    let sender = Address::generate(&env);
    mint(&env, &token_id, &sender, 100_000);

    // Seed 10 creators — not measured.
    for _ in 0..10 {
        let creator = Address::generate(&env);
        client.tip(&sender, &creator, &token_id, &1_000);
    }

    env.budget().reset_default();
    client.get_leaderboard(
        &tipjar::TimePeriod::AllTime,
        &tipjar::ParticipantKind::Creator,
        &10u32,
    );
    let cpu = env.budget().cpu_instruction_count();
    let mem = env.budget().memory_bytes_count();

    println!("[GAS] get_leaderboard (10 creators)  cpu={cpu}  mem={mem}");
    make_estimate("get_leaderboard", "warm", cpu, mem)
}

fn measure_create_subscription_cold() -> GasEstimate {
    let (env, contract_id, token_id, _) = setup();
    let client = TipJarContractClient::new(&env, &contract_id);
    let subscriber = Address::generate(&env);
    let creator = Address::generate(&env);

    env.budget().reset_default();
    client.create_subscription(
        &subscriber,
        &creator,
        &token_id,
        &1_000,
        &86_400u64, // 1 day interval
    );
    let cpu = env.budget().cpu_instruction_count();
    let mem = env.budget().memory_bytes_count();

    println!("[GAS] create_subscription (cold)  cpu={cpu}  mem={mem}");
    make_estimate("create_subscription", "cold", cpu, mem)
}

fn measure_execute_subscription_payment_warm() -> GasEstimate {
    let (env, contract_id, token_id, _) = setup();
    let client = TipJarContractClient::new(&env, &contract_id);
    let subscriber = Address::generate(&env);
    let creator = Address::generate(&env);
    mint(&env, &token_id, &subscriber, 10_000);

    // Create subscription — not measured.
    client.create_subscription(&subscriber, &creator, &token_id, &1_000, &86_400u64);

    // Advance ledger time so payment is due.
    env.ledger().with_mut(|l| {
        l.timestamp += 86_400;
    });

    env.budget().reset_default();
    client.execute_subscription_payment(&subscriber, &creator);
    let cpu = env.budget().cpu_instruction_count();
    let mem = env.budget().memory_bytes_count();

    println!("[GAS] execute_subscription_payment (warm)  cpu={cpu}  mem={mem}");
    make_estimate("execute_subscription_payment", "warm", cpu, mem)
}

fn measure_is_paused() -> GasEstimate {
    let (env, contract_id, _, _) = setup();
    let client = TipJarContractClient::new(&env, &contract_id);

    env.budget().reset_default();
    client.is_paused();
    let cpu = env.budget().cpu_instruction_count();
    let mem = env.budget().memory_bytes_count();

    println!("[GAS] is_paused  cpu={cpu}  mem={mem}");
    make_estimate("is_paused", "warm", cpu, mem)
}

fn measure_get_current_fee_bps() -> GasEstimate {
    let (env, contract_id, _, _) = setup();
    let client = TipJarContractClient::new(&env, &contract_id);

    env.budget().reset_default();
    client.get_current_fee_bps();
    let cpu = env.budget().cpu_instruction_count();
    let mem = env.budget().memory_bytes_count();

    println!("[GAS] get_current_fee_bps  cpu={cpu}  mem={mem}");
    make_estimate("get_current_fee_bps", "warm", cpu, mem)
}

// ── Batch aggregate helpers ───────────────────────────────────────────────────

fn make_batch_estimate(operation: &str, size: u32, estimate: &GasEstimate) -> BatchEstimate {
    let total_cost = compute_cost_stroops(estimate.cpu_instructions, estimate.memory_bytes);
    let per_item = total_cost / size as i128;
    BatchEstimate {
        operation: operation.to_string(),
        batch_size: size,
        total_cpu_instructions: estimate.cpu_instructions,
        total_memory_bytes: estimate.memory_bytes,
        total_cost_stroops: total_cost,
        total_cost_xlm: stroops_to_xlm(total_cost),
        cost_per_item_stroops: per_item,
        cost_per_item_xlm: stroops_to_xlm(per_item),
    }
}

// ── Main test entry point ─────────────────────────────────────────────────────

#[test]
fn run_all_estimates() {
    println!("\n╔══════════════════════════════════════════════════════╗");
    println!("║         TipJar Gas Estimation Suite                 ║");
    println!("╚══════════════════════════════════════════════════════╝\n");

    // Collect all estimates
    let estimates: std::vec::Vec<GasEstimate> = std::vec![
        measure_tip_cold(),
        measure_tip_warm(),
        measure_tip_with_fee_cold(),
        measure_withdraw_warm(),
        measure_get_withdrawable_balance_warm(),
        measure_get_total_tips_warm(),
        measure_tip_split_3(),
        measure_get_leaderboard_10(),
        measure_create_subscription_cold(),
        measure_execute_subscription_payment_warm(),
        measure_is_paused(),
        measure_get_current_fee_bps(),
    ];

    // Batch estimates: simulate N individual tips to estimate batch cost
    // (tip_batch is not yet implemented in the contract; we extrapolate from single-tip cost)
    let tip_cold = estimates.iter().find(|e| e.function_name == "tip" && e.storage_variant == "cold").unwrap();
    let tip_warm = estimates.iter().find(|e| e.function_name == "tip" && e.storage_variant == "warm").unwrap();
    // Batch of N: first tip is cold, remaining N-1 are warm
    let batch_10_cpu = tip_cold.cpu_instructions + 9 * tip_warm.cpu_instructions;
    let batch_10_mem = tip_cold.memory_bytes + 9 * tip_warm.memory_bytes;
    let batch_50_cpu = tip_cold.cpu_instructions + 49 * tip_warm.cpu_instructions;
    let batch_50_mem = tip_cold.memory_bytes + 49 * tip_warm.memory_bytes;

    let batch_estimates = std::vec![
        make_batch_estimate("tip (extrapolated)", 10,
            &make_estimate("tip_batch_extrapolated", "batch-10", batch_10_cpu, batch_10_mem)),
        make_batch_estimate("tip (extrapolated)", 50,
            &make_estimate("tip_batch_extrapolated", "batch-50", batch_50_cpu, batch_50_mem)),
    ];

    // Comparisons and suggestions
    let comparisons = generate_comparisons(&estimates);
    let suggestions = generate_suggestions(&estimates);

    let report = EstimationReport {
        timestamp: Utc::now(),
        network: "Stellar Testnet / Mainnet (Soroban)".to_string(),
        estimates,
        batch_estimates,
        comparisons,
        suggestions,
    };

    // Print summary to stdout
    println!("\n{:<40} {:>18} {:>14} {:>16}", "Function (variant)", "CPU Instructions", "Memory Bytes", "Est. Cost (XLM)");
    println!("{}", "─".repeat(92));
    for e in &report.estimates {
        println!(
            "{:<40} {:>18} {:>14} {:>16.8}",
            format!("{} ({})", e.function_name, e.storage_variant),
            e.cpu_instructions,
            e.memory_bytes,
            e.estimated_cost_xlm,
        );
    }

    println!("\nBatch Estimates:");
    println!("{:<30} {:>5} {:>18} {:>16} {:>16}", "Operation", "N", "Total CPU", "Total XLM", "Per-item XLM");
    println!("{}", "─".repeat(90));
    for b in &report.batch_estimates {
        println!(
            "{:<30} {:>5} {:>18} {:>16.8} {:>16.8}",
            b.operation, b.batch_size, b.total_cpu_instructions,
            b.total_cost_xlm, b.cost_per_item_xlm,
        );
    }

    if !report.suggestions.is_empty() {
        println!("\nOptimisation Suggestions:");
        for s in &report.suggestions {
            println!("  [{:?}] {}: {}", s.severity, s.function, s.message);
        }
    }

    // Write JSON report
    let json = serde_json::to_string_pretty(&report).expect("serialise report");
    std::fs::write("gas-estimates.json", &json).expect("write gas-estimates.json");
    println!("\n✅ Report written to gas-estimates.json");
}
