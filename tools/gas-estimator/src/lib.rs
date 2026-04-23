//! Gas estimation library for TipJar contract operations.
//!
//! Provides types and helpers shared between the CLI binary and the
//! integration-test harness that actually runs the Soroban budget measurements.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ── Core types ────────────────────────────────────────────────────────────────

/// Raw budget numbers captured from `env.budget()` after a single invocation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasEstimate {
    /// Contract function name.
    pub function_name: String,
    /// Storage access pattern: "cold" (first write) or "warm" (subsequent).
    pub storage_variant: String,
    /// CPU instructions consumed by the invocation.
    pub cpu_instructions: u64,
    /// Memory bytes consumed by the invocation.
    pub memory_bytes: u64,
    /// Estimated cost in stroops (1 XLM = 10,000,000 stroops).
    ///
    /// Derived from the Stellar fee model:
    ///   fee_stroops = ceil(cpu_instructions / CPU_PER_STROOP)
    ///               + ceil(memory_bytes    / MEM_PER_STROOP)
    pub estimated_cost_stroops: i128,
    /// Human-readable XLM equivalent.
    pub estimated_cost_xlm: f64,
}

/// A complete estimation report covering all measured functions.
#[derive(Debug, Serialize, Deserialize)]
pub struct EstimationReport {
    /// ISO-8601 timestamp of when the report was generated.
    pub timestamp: DateTime<Utc>,
    /// Stellar network the estimates target (informational).
    pub network: String,
    /// All individual function estimates.
    pub estimates: Vec<GasEstimate>,
    /// Batch operation estimates (batch size → aggregate estimate).
    pub batch_estimates: Vec<BatchEstimate>,
    /// Comparison table between related operations.
    pub comparisons: Vec<Comparison>,
    /// Optimisation suggestions derived from the measurements.
    pub suggestions: Vec<Suggestion>,
}

/// Aggregate cost for a batch of N identical operations.
#[derive(Debug, Serialize, Deserialize)]
pub struct BatchEstimate {
    pub operation: String,
    pub batch_size: u32,
    pub total_cpu_instructions: u64,
    pub total_memory_bytes: u64,
    pub total_cost_stroops: i128,
    pub total_cost_xlm: f64,
    pub cost_per_item_stroops: i128,
    pub cost_per_item_xlm: f64,
}

/// Side-by-side comparison of two operations.
#[derive(Debug, Serialize, Deserialize)]
pub struct Comparison {
    pub label: String,
    pub baseline: String,
    pub candidate: String,
    pub baseline_cpu: u64,
    pub candidate_cpu: u64,
    /// Positive = candidate is more expensive; negative = cheaper.
    pub delta_cpu: i64,
    pub delta_pct: f64,
}

/// A single optimisation recommendation.
#[derive(Debug, Serialize, Deserialize)]
pub struct Suggestion {
    pub function: String,
    pub severity: Severity,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity {
    Info,
    Warning,
    Critical,
}

// ── Fee model constants ───────────────────────────────────────────────────────

/// Stellar fee model: CPU instructions per stroop.
/// Based on Stellar Core's resource fee schedule (approximate).
pub const CPU_PER_STROOP: u64 = 10_000;

/// Stellar fee model: memory bytes per stroop.
pub const MEM_PER_STROOP: u64 = 1_024;

/// Stroops per XLM.
pub const STROOPS_PER_XLM: i128 = 10_000_000;

/// CPU threshold above which a warning is emitted.
pub const WARN_CPU: u64 = 1_000_000;

/// CPU threshold above which a critical alert is emitted.
pub const CRITICAL_CPU: u64 = 5_000_000;

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Convert raw budget numbers to an estimated stroop cost.
pub fn compute_cost_stroops(cpu: u64, mem: u64) -> i128 {
    let cpu_fee = (cpu as i128 + CPU_PER_STROOP as i128 - 1) / CPU_PER_STROOP as i128;
    let mem_fee = (mem as i128 + MEM_PER_STROOP as i128 - 1) / MEM_PER_STROOP as i128;
    cpu_fee + mem_fee
}

/// Convert stroops to XLM.
pub fn stroops_to_xlm(stroops: i128) -> f64 {
    stroops as f64 / STROOPS_PER_XLM as f64
}

/// Build a `GasEstimate` from raw budget numbers.
pub fn make_estimate(function_name: &str, storage_variant: &str, cpu: u64, mem: u64) -> GasEstimate {
    let cost = compute_cost_stroops(cpu, mem);
    GasEstimate {
        function_name: function_name.to_string(),
        storage_variant: storage_variant.to_string(),
        cpu_instructions: cpu,
        memory_bytes: mem,
        estimated_cost_stroops: cost,
        estimated_cost_xlm: stroops_to_xlm(cost),
    }
}

/// Derive optimisation suggestions from a list of estimates.
pub fn generate_suggestions(estimates: &[GasEstimate]) -> Vec<Suggestion> {
    let mut suggestions = Vec::new();

    for e in estimates {
        let severity = if e.cpu_instructions >= CRITICAL_CPU {
            Some(Severity::Critical)
        } else if e.cpu_instructions >= WARN_CPU {
            Some(Severity::Warning)
        } else {
            None
        };

        if let Some(sev) = severity {
            suggestions.push(Suggestion {
                function: e.function_name.clone(),
                severity: sev,
                message: format!(
                    "CPU usage ({} instructions) is high. Consider caching storage reads \
                     or splitting the operation into smaller steps.",
                    e.cpu_instructions
                ),
            });
        }

        // Memory-specific hint
        if e.memory_bytes >= 50_000 {
            suggestions.push(Suggestion {
                function: e.function_name.clone(),
                severity: Severity::Warning,
                message: format!(
                    "Memory usage ({} bytes) is elevated. Avoid allocating large \
                     Vecs/Maps inside the contract; prefer pagination.",
                    e.memory_bytes
                ),
            });
        }

        // Function-specific hints
        if e.function_name.contains("leaderboard") {
            suggestions.push(Suggestion {
                function: e.function_name.clone(),
                severity: Severity::Info,
                message: "Leaderboard queries iterate over all participants. \
                          Maintain a pre-sorted index in storage to avoid O(n) scans."
                    .to_string(),
            });
        }

        if e.function_name.contains("split") {
            suggestions.push(Suggestion {
                function: e.function_name.clone(),
                severity: Severity::Info,
                message: "tip_split iterates over recipients and writes one storage entry \
                          per recipient. Batch writes are bounded by the 2–10 recipient limit."
                    .to_string(),
            });
        }

        if e.function_name.contains("subscription") && e.cpu_instructions >= WARN_CPU {
            suggestions.push(Suggestion {
                function: e.function_name.clone(),
                severity: Severity::Info,
                message: "Subscription operations read and write the full Subscription struct. \
                          Ensure the struct size stays small to minimise serialisation cost."
                    .to_string(),
            });
        }
    }

    // Cold vs warm comparison hint
    let cold = estimates.iter().find(|e| e.function_name == "tip" && e.storage_variant == "cold");
    let warm = estimates.iter().find(|e| e.function_name == "tip" && e.storage_variant == "warm");
    if let (Some(c), Some(w)) = (cold, warm) {
        let overhead_pct = (c.cpu_instructions as f64 - w.cpu_instructions as f64)
            / w.cpu_instructions as f64
            * 100.0;
        if overhead_pct > 50.0 {
            suggestions.push(Suggestion {
                function: "tip (cold)".to_string(),
                severity: Severity::Info,
                message: format!(
                    "Cold-storage tip is {overhead_pct:.0}% more expensive than warm. \
                     First-time creator tips allocate new ledger entries; this is expected \
                     but worth communicating to users."
                ),
            });
        }
    }

    suggestions
}

/// Build comparison entries from a slice of estimates.
pub fn generate_comparisons(estimates: &[GasEstimate]) -> Vec<Comparison> {
    let mut comparisons = Vec::new();

    let pairs: &[(&str, &str, &str, &str, &str)] = &[
        ("tip: cold vs warm", "tip", "cold", "tip", "warm"),
        ("tip vs tip_with_fee", "tip", "cold", "tip_with_fee", "cold"),
        ("tip vs tip_split (3 recipients)", "tip", "cold", "tip_split", "cold"),
        ("withdraw vs get_withdrawable_balance", "withdraw", "warm", "get_withdrawable_balance", "warm"),
        ("create_subscription vs execute_subscription_payment", "create_subscription", "cold", "execute_subscription_payment", "warm"),
    ];

    for (label, base_fn, base_var, cand_fn, cand_var) in pairs {
        let base = estimates.iter().find(|e| e.function_name == *base_fn && e.storage_variant == *base_var);
        let cand = estimates.iter().find(|e| e.function_name == *cand_fn && e.storage_variant == *cand_var);
        if let (Some(b), Some(c)) = (base, cand) {
            let delta = c.cpu_instructions as i64 - b.cpu_instructions as i64;
            let delta_pct = delta as f64 / b.cpu_instructions as f64 * 100.0;
            comparisons.push(Comparison {
                label: label.to_string(),
                baseline: format!("{} ({})", b.function_name, b.storage_variant),
                candidate: format!("{} ({})", c.function_name, c.storage_variant),
                baseline_cpu: b.cpu_instructions,
                candidate_cpu: c.cpu_instructions,
                delta_cpu: delta,
                delta_pct,
            });
        }
    }

    comparisons
}
