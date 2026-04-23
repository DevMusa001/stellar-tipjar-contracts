//! gas-estimator — CLI tool for estimating TipJar contract gas costs.
//!
//! Reads a gas report produced by the companion integration test
//! (`cargo test -p gas-estimator --test estimate`) and presents it in a
//! human-readable table, JSON, or Markdown format.  It also accepts a
//! baseline report for regression detection.
//!
//! # Usage
//!
//! ```text
//! # Run the estimator tests to produce a fresh report, then analyse it:
//! cargo test -p gas-estimator --test estimate -- --nocapture
//! cargo run -p gas-estimator -- --report gas-estimates.json
//!
//! # Compare against a saved baseline:
//! cargo run -p gas-estimator -- --report gas-estimates.json --baseline baseline.json
//!
//! # Output as JSON:
//! cargo run -p gas-estimator -- --report gas-estimates.json --format json
//!
//! # Output as Markdown:
//! cargo run -p gas-estimator -- --report gas-estimates.json --format markdown
//! ```

use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use gas_estimator::{EstimationReport, Severity};

#[derive(Parser)]
#[command(
    name = "gas-estimator",
    about = "Estimate and analyse TipJar contract gas costs",
    version
)]
struct Cli {
    /// Path to the gas estimation report JSON (produced by `cargo test -p gas-estimator --test estimate`)
    #[arg(long, default_value = "gas-estimates.json")]
    report: String,

    /// Optional baseline report to compare against for regression detection
    #[arg(long)]
    baseline: Option<String>,

    /// Output format
    #[arg(long, value_enum, default_value = "table")]
    format: OutputFormat,

    /// Fail with exit code 1 if any function exceeds the CPU warning threshold
    #[arg(long)]
    strict: bool,
}

#[derive(Clone, ValueEnum)]
enum OutputFormat {
    Table,
    Json,
    Markdown,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let json = std::fs::read_to_string(&cli.report)
        .with_context(|| format!("Cannot read report file: {}", cli.report))?;
    let report: EstimationReport = serde_json::from_str(&json)
        .with_context(|| format!("Failed to parse report: {}", cli.report))?;

    match cli.format {
        OutputFormat::Table => print_table(&report),
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        OutputFormat::Markdown => print_markdown(&report),
    }

    // Baseline comparison
    if let Some(baseline_path) = cli.baseline {
        let baseline_json = std::fs::read_to_string(&baseline_path)
            .with_context(|| format!("Cannot read baseline: {baseline_path}"))?;
        let baseline: EstimationReport = serde_json::from_str(&baseline_json)?;
        let regression = compare_baseline(&baseline, &report);
        if regression && cli.strict {
            std::process::exit(1);
        }
    }

    // Strict mode: fail if any critical suggestion exists
    if cli.strict {
        let has_critical = report
            .suggestions
            .iter()
            .any(|s| matches!(s.severity, Severity::Critical));
        if has_critical {
            eprintln!("\n❌ Critical gas issues detected. Failing (--strict mode).");
            std::process::exit(1);
        }
    }

    Ok(())
}

// ── Table output ──────────────────────────────────────────────────────────────

fn print_table(report: &EstimationReport) {
    println!("╔══════════════════════════════════════════════════════════════════════════════╗");
    println!("║              TipJar Gas Cost Estimation Report                              ║");
    println!("╚══════════════════════════════════════════════════════════════════════════════╝");
    println!("  Generated : {}", report.timestamp.format("%Y-%m-%d %H:%M:%S UTC"));
    println!("  Network   : {}", report.network);
    println!();

    // ── Per-function estimates ────────────────────────────────────────────────
    println!("┌─────────────────────────────────────┬─────────┬──────────────────┬──────────────┬──────────────────┐");
    println!("│ Function                            │ Variant │ CPU Instructions │ Memory Bytes │ Est. Cost (XLM)  │");
    println!("├─────────────────────────────────────┼─────────┼──────────────────┼──────────────┼──────────────────┤");

    for e in &report.estimates {
        let marker = severity_marker(e.cpu_instructions);
        println!(
            "│ {}{:<36} │ {:<7} │ {:>16} │ {:>12} │ {:>16.8} │",
            marker,
            e.function_name,
            e.storage_variant,
            e.cpu_instructions,
            e.memory_bytes,
            e.estimated_cost_xlm,
        );
    }
    println!("└─────────────────────────────────────┴─────────┴──────────────────┴──────────────┴──────────────────┘");
    println!("  🟢 < 1M CPU   🟡 1M–5M CPU   🔴 > 5M CPU");
    println!();

    // ── Batch estimates ───────────────────────────────────────────────────────
    if !report.batch_estimates.is_empty() {
        println!("Batch Operation Estimates");
        println!("─────────────────────────────────────────────────────────────────────────────");
        println!(
            "  {:<30} {:>5}  {:>18}  {:>16}  {:>16}",
            "Operation", "N", "Total CPU", "Total XLM", "Per-item XLM"
        );
        println!("  {}", "─".repeat(90));
        for b in &report.batch_estimates {
            println!(
                "  {:<30} {:>5}  {:>18}  {:>16.8}  {:>16.8}",
                b.operation, b.batch_size, b.total_cpu_instructions,
                b.total_cost_xlm, b.cost_per_item_xlm,
            );
        }
        println!();
    }

    // ── Comparisons ───────────────────────────────────────────────────────────
    if !report.comparisons.is_empty() {
        println!("Cost Comparisons");
        println!("─────────────────────────────────────────────────────────────────────────────");
        for c in &report.comparisons {
            let arrow = if c.delta_cpu > 0 { "▲" } else { "▼" };
            let sign = if c.delta_cpu > 0 { "+" } else { "" };
            println!(
                "  {:<50}  {}{}{:.1}%  ({}{} CPU)",
                c.label, arrow, sign, c.delta_pct, sign, c.delta_cpu
            );
        }
        println!();
    }

    // ── Suggestions ───────────────────────────────────────────────────────────
    if !report.suggestions.is_empty() {
        println!("Optimisation Suggestions");
        println!("─────────────────────────────────────────────────────────────────────────────");
        for s in &report.suggestions {
            let icon = match s.severity {
                Severity::Info => "ℹ️ ",
                Severity::Warning => "⚠️ ",
                Severity::Critical => "🔴",
            };
            println!("  {} [{}] {}", icon, s.function, s.message);
        }
        println!();
    } else {
        println!("✅ No optimisation suggestions — all functions are within acceptable limits.");
        println!();
    }
}

// ── Markdown output ───────────────────────────────────────────────────────────

fn print_markdown(report: &EstimationReport) {
    println!("# TipJar Gas Cost Estimation Report");
    println!();
    println!("**Generated:** {}  ", report.timestamp.format("%Y-%m-%d %H:%M:%S UTC"));
    println!("**Network:** {}  ", report.network);
    println!();

    println!("## Per-Function Estimates");
    println!();
    println!("| Function | Variant | CPU Instructions | Memory Bytes | Est. Cost (XLM) |");
    println!("|---|---|---:|---:|---:|");
    for e in &report.estimates {
        println!(
            "| `{}` | {} | {} | {} | {:.8} |",
            e.function_name, e.storage_variant,
            e.cpu_instructions, e.memory_bytes, e.estimated_cost_xlm,
        );
    }
    println!();

    if !report.batch_estimates.is_empty() {
        println!("## Batch Operation Estimates");
        println!();
        println!("| Operation | Batch Size | Total CPU | Total XLM | Per-item XLM |");
        println!("|---|---:|---:|---:|---:|");
        for b in &report.batch_estimates {
            println!(
                "| `{}` | {} | {} | {:.8} | {:.8} |",
                b.operation, b.batch_size, b.total_cpu_instructions,
                b.total_cost_xlm, b.cost_per_item_xlm,
            );
        }
        println!();
    }

    if !report.comparisons.is_empty() {
        println!("## Cost Comparisons");
        println!();
        println!("| Comparison | Baseline CPU | Candidate CPU | Delta | Delta % |");
        println!("|---|---:|---:|---:|---:|");
        for c in &report.comparisons {
            let sign = if c.delta_cpu > 0 { "+" } else { "" };
            println!(
                "| {} | {} | {} | {}{} | {}{:.1}% |",
                c.label, c.baseline_cpu, c.candidate_cpu,
                sign, c.delta_cpu, sign, c.delta_pct,
            );
        }
        println!();
    }

    if !report.suggestions.is_empty() {
        println!("## Optimisation Suggestions");
        println!();
        for s in &report.suggestions {
            let level = match s.severity {
                Severity::Info => "INFO",
                Severity::Warning => "WARNING",
                Severity::Critical => "CRITICAL",
            };
            println!("- **[{}] `{}`** — {}", level, s.function, s.message);
        }
        println!();
    }
}

// ── Baseline comparison ───────────────────────────────────────────────────────

/// Returns `true` if any regression (>10% CPU increase) was detected.
fn compare_baseline(baseline: &EstimationReport, current: &EstimationReport) -> bool {
    println!("\n=== Gas Regression Report ===");
    println!(
        "  {:<40} {:>15} {:>15} {:>10}",
        "Function (variant)", "Baseline CPU", "Current CPU", "Delta %"
    );
    println!("  {}", "─".repeat(85));

    let mut regression = false;
    for cur in &current.estimates {
        let key = format!("{}/{}", cur.function_name, cur.storage_variant);
        let base = baseline.estimates.iter().find(|b| {
            b.function_name == cur.function_name && b.storage_variant == cur.storage_variant
        });
        if let Some(b) = base {
            let delta_pct = (cur.cpu_instructions as f64 - b.cpu_instructions as f64)
                / b.cpu_instructions as f64
                * 100.0;
            let flag = if delta_pct > 10.0 {
                regression = true;
                " ⚠ REGRESSION"
            } else {
                ""
            };
            println!(
                "  {:<40} {:>15} {:>15} {:>9.1}%{}",
                key, b.cpu_instructions, cur.cpu_instructions, delta_pct, flag
            );
        }
    }

    if regression {
        eprintln!("\n❌ Gas regression detected (>10% CPU increase).");
    } else {
        println!("\n✅ No gas regressions detected.");
    }

    regression
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn severity_marker(cpu: u64) -> &'static str {
    if cpu >= gas_estimator::CRITICAL_CPU {
        "🔴"
    } else if cpu >= gas_estimator::WARN_CPU {
        "🟡"
    } else {
        "🟢"
    }
}
