//! `diff`, `check`, and `comment` commands: compare snapshots and gate.

use super::analyze::chrono_lite_now;

/// Compare two topology snapshots.
pub(super) fn topology_diff(base: &str, target: &str, format: &str, _verbose: bool) -> i32 {
    use std::path::Path;

    let base_path = Path::new(base);
    let target_path = Path::new(target);

    // Check both paths exist
    if !base_path.exists() {
        eprintln!("Error: Base path does not exist: {base}");
        return 1;
    }
    if !target_path.exists() {
        eprintln!("Error: Target path does not exist: {target}");
        return 1;
    }

    // Load metrics from both snapshots
    let base_metrics = load_topology_metrics(base_path);
    let target_metrics = load_topology_metrics(target_path);

    // Compute diff
    let diff = compute_topology_diff(base, target, &base_metrics, &target_metrics);

    if format == "json" {
        // Output JSON format matching proto/diff.proto schema
        match serde_json::to_string_pretty(&diff) {
            Ok(json) => {
                println!("{json}");
                match diff.status.as_str() {
                    "success" => 0,
                    "error" => 1,
                    _ => 2, // warning
                }
            }
            Err(e) => {
                eprintln!("Error serializing diff: {e}");
                1
            }
        }
    } else {
        // Human-readable text format
        println!("Topology Diff: {base} → {target}");
        println!();
        println!(
            "  Functions: {} → {} ({:+})",
            base_metrics.function_count,
            target_metrics.function_count,
            target_metrics.function_count as i64 - base_metrics.function_count as i64
        );
        println!(
            "  Total CC:  {} → {} ({:+})",
            base_metrics.total_cyclomatic,
            target_metrics.total_cyclomatic,
            target_metrics.total_cyclomatic as i64 - base_metrics.total_cyclomatic as i64
        );
        println!(
            "  Avg CC:    {:.1} → {:.1} ({:+.1})",
            base_metrics.avg_cyclomatic,
            target_metrics.avg_cyclomatic,
            target_metrics.avg_cyclomatic - base_metrics.avg_cyclomatic
        );

        if !diff.hotspots.is_empty() {
            println!();
            println!("Hotspots:");
            for hotspot in &diff.hotspots {
                println!("  ⚠ {} - {}", hotspot.id, hotspot.reason);
            }
        }

        println!();
        match diff.status.as_str() {
            "success" => {
                println!("✓ No degradation detected");
                0
            }
            "error" => {
                println!("✗ Quality gate failed");
                1
            }
            _ => {
                println!("⚠ Warnings detected (review recommended)");
                2
            }
        }
    }
}

/// Aggregated topology metrics for comparison.
#[derive(Default)]
struct TopologyMetrics {
    function_count: usize,
    total_cyclomatic: u64,
    avg_cyclomatic: f64,
    total_cognitive: u64,
    avg_cognitive: f64,
    lines_of_code: u64,
}

/// Load topology metrics from a .topology/ directory.
fn load_topology_metrics(path: &std::path::Path) -> TopologyMetrics {
    let mut metrics = TopologyMetrics::default();

    // Load functions.json
    let funcs_path = path.join("metrics/functions.json");
    if let Ok(content) = std::fs::read_to_string(&funcs_path) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(funcs) = json.get("functions").and_then(|f| f.as_array()) {
                metrics.function_count = funcs.len();

                let mut total_cc = 0u64;
                let mut total_cog = 0u64;
                let mut total_loc = 0u64;

                for func in funcs {
                    if let Some(m) = func.get("metrics") {
                        total_cc += m
                            .get("cyclomatic_complexity")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0);
                        total_cog += m
                            .get("cognitive_complexity")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0);
                        total_loc += m.get("lines_of_code").and_then(|v| v.as_u64()).unwrap_or(0);
                    }
                }

                metrics.total_cyclomatic = total_cc;
                metrics.total_cognitive = total_cog;
                metrics.lines_of_code = total_loc;

                if metrics.function_count > 0 {
                    metrics.avg_cyclomatic = total_cc as f64 / metrics.function_count as f64;
                    metrics.avg_cognitive = total_cog as f64 / metrics.function_count as f64;
                }
            }
        }
    }

    metrics
}

/// Diff output matching proto/diff.proto schema.
#[derive(serde::Serialize)]
struct TopologyDiff {
    schema_version: String,
    status: String,
    timestamp: String,
    base: DiffRef,
    target: DiffRef,
    summary: DiffSummary,
    metrics: MetricDeltas,
    hotspots: Vec<DiffHotspot>,
    violations: Vec<ThresholdViolation>,
}

#[derive(serde::Serialize)]
struct DiffRef {
    path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    git_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    commit: Option<String>,
}

#[derive(serde::Serialize)]
struct DiffSummary {
    functions_added: u32,
    functions_removed: u32,
    functions_modified: u32,
    modules_added: u32,
    modules_removed: u32,
    modules_modified: u32,
}

#[derive(serde::Serialize)]
struct MetricDeltas {
    total_cyclomatic: MetricDelta,
    avg_cyclomatic: MetricDelta,
    total_cognitive: MetricDelta,
    avg_cognitive: MetricDelta,
    lines_of_code: MetricDelta,
    function_count: MetricDelta,
}

#[derive(serde::Serialize)]
struct MetricDelta {
    base: f64,
    target: f64,
    delta: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    percent_change: Option<f64>,
}

impl MetricDelta {
    fn new(base: f64, target: f64) -> Self {
        let delta = target - base;
        let percent_change = if base > 0.0 {
            Some((delta / base) * 100.0)
        } else {
            None
        };
        Self {
            base,
            target,
            delta,
            percent_change,
        }
    }
}

#[derive(serde::Serialize)]
struct DiffHotspot {
    id: String,
    #[serde(rename = "type")]
    hotspot_type: String,
    reason: String,
    severity: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    suggestion: Option<String>,
}

#[derive(serde::Serialize)]
struct ThresholdViolation {
    threshold: String,
    value: f64,
    limit: f64,
    severity: String,
    message: String,
}

/// Compute a topology diff between two snapshots.
fn compute_topology_diff(
    base_path: &str,
    target_path: &str,
    base: &TopologyMetrics,
    target: &TopologyMetrics,
) -> TopologyDiff {
    let mut hotspots = Vec::new();
    let mut violations = Vec::new();

    // Check for significant complexity increases
    let cc_delta = target.avg_cyclomatic - base.avg_cyclomatic;
    if cc_delta > 2.0 {
        hotspots.push(DiffHotspot {
            id: "aggregate".to_string(),
            hotspot_type: "INCREASED_COMPLEXITY".to_string(),
            reason: format!(
                "Average cyclomatic complexity increased by {:.1} ({:.0}%)",
                cc_delta,
                if base.avg_cyclomatic > 0.0 {
                    (cc_delta / base.avg_cyclomatic) * 100.0
                } else {
                    0.0
                }
            ),
            severity: if cc_delta > 5.0 { 3 } else { 2 },
            suggestion: Some("Review new functions for complexity".to_string()),
        });
    }

    // Determine status based on metrics
    let status = if cc_delta > 5.0 || (target.avg_cyclomatic > 15.0 && cc_delta > 0.0) {
        "error"
    } else if cc_delta > 2.0 || !hotspots.is_empty() {
        "warning"
    } else {
        "success"
    };

    // Add threshold violation if significant
    if cc_delta > 2.0 {
        violations.push(ThresholdViolation {
            threshold: "avg_cyclomatic_delta".to_string(),
            value: cc_delta,
            limit: 2.0,
            severity: if cc_delta > 5.0 {
                "ERROR".to_string()
            } else {
                "WARNING".to_string()
            },
            message: format!(
                "Average cyclomatic complexity increased by {cc_delta:.1}, exceeds threshold"
            ),
        });
    }

    // Compute function changes (simplified - just counts)
    let func_diff = target.function_count as i32 - base.function_count as i32;
    let (added, removed) = if func_diff >= 0 {
        (func_diff as u32, 0)
    } else {
        (0, (-func_diff) as u32)
    };

    TopologyDiff {
        schema_version: "1.0.0".to_string(),
        status: status.to_string(),
        timestamp: chrono_lite_now(),
        base: DiffRef {
            path: base_path.to_string(),
            git_ref: None,
            commit: None,
        },
        target: DiffRef {
            path: target_path.to_string(),
            git_ref: None,
            commit: None,
        },
        summary: DiffSummary {
            functions_added: added,
            functions_removed: removed,
            functions_modified: 0, // Would need function-level tracking
            modules_added: 0,
            modules_removed: 0,
            modules_modified: 0,
        },
        metrics: MetricDeltas {
            total_cyclomatic: MetricDelta::new(
                base.total_cyclomatic as f64,
                target.total_cyclomatic as f64,
            ),
            avg_cyclomatic: MetricDelta::new(base.avg_cyclomatic, target.avg_cyclomatic),
            total_cognitive: MetricDelta::new(
                base.total_cognitive as f64,
                target.total_cognitive as f64,
            ),
            avg_cognitive: MetricDelta::new(base.avg_cognitive, target.avg_cognitive),
            lines_of_code: MetricDelta::new(base.lines_of_code as f64, target.lines_of_code as f64),
            function_count: MetricDelta::new(
                base.function_count as f64,
                target.function_count as f64,
            ),
        },
        hotspots,
        violations,
    }
}

/// Check a diff against thresholds.
pub(super) fn topology_check(diff_file: Option<&str>, config: Option<&str>, _verbose: bool) -> i32 {
    let diff_path = match diff_file {
        Some(p) => p,
        None => {
            eprintln!("Error: diff file required");
            eprintln!("Usage: apss-dev run topology check <diff.json> [--config <file>]");
            return 1;
        }
    };

    // Load the diff
    let diff_content = match std::fs::read_to_string(diff_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading diff file: {e}");
            return 1;
        }
    };

    let diff: serde_json::Value = match serde_json::from_str(&diff_content) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error parsing diff JSON: {e}");
            return 1;
        }
    };

    // Load thresholds from config (or use defaults)
    let thresholds = load_thresholds(config);

    // Check violations
    let mut errors = 0;
    let mut warnings = 0;

    // Check avg_cyclomatic delta
    if let Some(delta) = diff
        .get("metrics")
        .and_then(|m| m.get("avg_cyclomatic"))
        .and_then(|d| d.get("delta"))
        .and_then(|v| v.as_f64())
    {
        if delta > thresholds.max_cc_delta_error {
            println!(
                "✗ ERROR: avg_cyclomatic increased by {delta:.1} (limit: {})",
                thresholds.max_cc_delta_error
            );
            errors += 1;
        } else if delta > thresholds.max_cc_delta_warning {
            println!(
                "⚠ WARNING: avg_cyclomatic increased by {delta:.1} (limit: {})",
                thresholds.max_cc_delta_warning
            );
            warnings += 1;
        }
    }

    // Check if any existing violations
    if let Some(violations) = diff.get("violations").and_then(|v| v.as_array()) {
        for v in violations {
            let severity = v
                .get("severity")
                .and_then(|s| s.as_str())
                .unwrap_or("WARNING");
            let message = v
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown violation");
            if severity == "ERROR" {
                println!("✗ ERROR: {message}");
                errors += 1;
            } else {
                println!("⚠ WARNING: {message}");
                warnings += 1;
            }
        }
    }

    // Summary
    println!();
    if errors > 0 {
        println!("✗ Check failed: {errors} error(s), {warnings} warning(s)");
        1
    } else if warnings > 0 {
        println!("⚠ Check passed with warnings: {warnings} warning(s)");
        2
    } else {
        println!("✓ All checks passed");
        0
    }
}

/// Threshold configuration.
struct Thresholds {
    max_cc_delta_warning: f64,
    max_cc_delta_error: f64,
}

impl Default for Thresholds {
    fn default() -> Self {
        Self {
            max_cc_delta_warning: 2.0,
            max_cc_delta_error: 5.0,
        }
    }
}

/// Load thresholds from config file or use defaults.
fn load_thresholds(config: Option<&str>) -> Thresholds {
    if let Some(config_path) = config {
        if let Ok(content) = std::fs::read_to_string(config_path) {
            // Simple TOML parsing for thresholds
            let mut thresholds = Thresholds::default();
            for line in content.lines() {
                let line = line.trim();
                if line.starts_with("max_cyclomatic_warning") {
                    if let Some(val) = line.split('=').nth(1) {
                        if let Ok(v) = val.trim().parse::<f64>() {
                            thresholds.max_cc_delta_warning = v;
                        }
                    }
                } else if line.starts_with("max_cyclomatic_failure") {
                    if let Some(val) = line.split('=').nth(1) {
                        if let Ok(v) = val.trim().parse::<f64>() {
                            thresholds.max_cc_delta_error = v;
                        }
                    }
                }
            }
            return thresholds;
        }
    }
    Thresholds::default()
}

/// Generate a PR comment from a diff.
pub(super) fn topology_comment(
    diff_file: Option<&str>,
    _config: Option<&str>,
    _verbose: bool,
) -> i32 {
    let diff_path = match diff_file {
        Some(p) => p,
        None => {
            eprintln!("Error: diff file required");
            eprintln!("Usage: apss-dev run topology comment <diff.json>");
            return 1;
        }
    };

    // Load the diff
    let diff_content = match std::fs::read_to_string(diff_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading diff file: {e}");
            return 1;
        }
    };

    let diff: serde_json::Value = match serde_json::from_str(&diff_content) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error parsing diff JSON: {e}");
            return 1;
        }
    };

    // Generate markdown comment
    let status = diff
        .get("status")
        .and_then(|s| s.as_str())
        .unwrap_or("unknown");
    let status_emoji = match status {
        "success" => "✅",
        "warning" => "⚠️",
        "error" => "❌",
        _ => "❓",
    };

    println!("## 🔍 Topology Analysis {status_emoji}");
    println!();

    // Metrics table
    println!("### Metrics");
    println!();
    println!("| Metric | Base | Target | Δ |");
    println!("|--------|------|--------|---|");

    if let Some(metrics) = diff.get("metrics") {
        print_metric_row(metrics, "function_count", "Functions");
        print_metric_row(metrics, "total_cyclomatic", "Total CC");
        print_metric_row(metrics, "avg_cyclomatic", "Avg CC");
        print_metric_row(metrics, "total_cognitive", "Total Cognitive");
        print_metric_row(metrics, "lines_of_code", "Lines of Code");
    }

    // Hotspots
    if let Some(hotspots) = diff.get("hotspots").and_then(|h| h.as_array()) {
        if !hotspots.is_empty() {
            println!();
            println!("### ⚠️ Hotspots");
            println!();
            for hotspot in hotspots {
                let id = hotspot.get("id").and_then(|i| i.as_str()).unwrap_or("?");
                let reason = hotspot
                    .get("reason")
                    .and_then(|r| r.as_str())
                    .unwrap_or("?");
                let suggestion = hotspot.get("suggestion").and_then(|s| s.as_str());
                println!("- **{id}**: {reason}");
                if let Some(s) = suggestion {
                    println!("  - 💡 {s}");
                }
            }
        }
    }

    // Violations
    if let Some(violations) = diff.get("violations").and_then(|v| v.as_array()) {
        if !violations.is_empty() {
            println!();
            println!("### Threshold Violations");
            println!();
            for v in violations {
                let severity = v
                    .get("severity")
                    .and_then(|s| s.as_str())
                    .unwrap_or("WARNING");
                let message = v.get("message").and_then(|m| m.as_str()).unwrap_or("?");
                let emoji = if severity == "ERROR" { "❌" } else { "⚠️" };
                println!("- {emoji} {message}");
            }
        }
    }

    // Footer
    println!();
    println!("---");
    println!(
        "*Generated by [APS Topology](https://github.com/AgentParadise/agent-paradise-standards-system) (EXP-V1-0001)*"
    );

    0
}

/// Print a metric row for the comment table.
fn print_metric_row(metrics: &serde_json::Value, key: &str, label: &str) {
    if let Some(m) = metrics.get(key) {
        let base = m.get("base").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let target = m.get("target").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let delta = m.get("delta").and_then(|v| v.as_f64()).unwrap_or(0.0);

        let delta_str = if delta >= 0.0 {
            format!("+{delta:.1}")
        } else {
            format!("{delta:.1}")
        };

        println!("| {label} | {base:.1} | {target:.1} | {delta_str} |");
    }
}
