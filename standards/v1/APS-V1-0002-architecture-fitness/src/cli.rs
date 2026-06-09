//! Composed CLI for the Architecture Fitness Functions standard (APS-V1-0002).
//!
//! This module hosts the command implementation that backs `apss run
//! architecture-fitness validate` in composed consumer binaries and `apss-dev
//! run architecture-fitness validate` (plus the `fitness` alias) in the
//! development CLI. Both routes dispatch through [`FitnessCommandHandler`],
//! which implements [`apss_core::registry::CommandHandler`].
//!
//! Impedance notes:
//! - `CommandHandler::execute` receives no repo root, so the handler resolves
//!   `repo_root = std::env::current_dir()`.
//! - verbose output is env-driven: the handler reads `APSS_VERBOSE=1`. The
//!   transport is preserved for parity with the topology handler.
//! - command functions return `i32` (0 success, 1 error, 3 usage/unknown).

use apss_core::registry::{CommandHandler, CommandInfo};

use crate::{FitnessReport, FitnessValidator, RuleStatus};

/// Handler that backs `run architecture-fitness <command>` in composed
/// binaries and `run architecture-fitness <command>` in the dev CLI.
pub struct FitnessCommandHandler;

impl FitnessCommandHandler {
    /// Create a new handler instance.
    pub fn new() -> Self {
        Self
    }
}

impl Default for FitnessCommandHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandHandler for FitnessCommandHandler {
    fn execute(&self, command: &str, args: &[String], _config: &toml::Value) -> i32 {
        // repo_root: the dev CLI runs commands relative to the invocation
        // directory, so resolving the current directory preserves behavior.
        let repo_root = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        // verbose: env-driven so the trait boundary stays signature-stable.
        let verbose = std::env::var("APSS_VERBOSE").is_ok_and(|v| v == "1");

        dispatch(command, args, &repo_root, verbose)
    }

    fn commands(&self) -> Vec<CommandInfo> {
        command_infos()
    }
}

/// Dispatch a fitness command to its implementation.
fn dispatch(command: &str, args: &[String], repo_root: &std::path::Path, verbose: bool) -> i32 {
    match command {
        "--help" | "-h" | "help" => {
            print_help();
            0
        }
        "validate" => validate(args, repo_root, verbose),
        other => {
            eprintln!("Error: Unknown architecture-fitness command '{other}'");
            eprintln!("Use 'apss-dev run architecture-fitness --help' for available commands.");
            3
        }
    }
}

/// Validate fitness rules against topology artifacts.
fn validate(args: &[String], repo_root: &std::path::Path, _verbose: bool) -> i32 {
    // Parse flags and positional args separately to avoid
    // `--config custom.toml .` misinterpreting "--config" as the path.
    let mut positional_path: Option<&str> = None;
    let mut config_path: Option<std::path::PathBuf> = None;
    let mut report_path: Option<&String> = None;
    let mut previous_path: Option<&String> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--config" => {
                config_path = args.get(i + 1).map(std::path::PathBuf::from);
                i += 2;
            }
            "--report" => {
                report_path = args.get(i + 1);
                i += 2;
            }
            // `--previous` is accepted as a back-compat alias of `--previous-report`.
            "--previous-report" | "--previous" => {
                previous_path = args.get(i + 1);
                i += 2;
            }
            arg if !arg.starts_with('-') && positional_path.is_none() => {
                positional_path = Some(arg);
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }
    let path = positional_path.unwrap_or(".");

    let target = if std::path::Path::new(path).is_absolute() {
        std::path::PathBuf::from(path)
    } else {
        repo_root.join(path)
    };

    // Resolve --config relative to target repo, not CWD.
    let config_path = config_path.map(|p| if p.is_absolute() { p } else { target.join(p) });

    // Missing inputs (no fitness.toml, missing topology_dir) surface as a clear
    // message + nonzero exit, mirroring how topology handles absent artifacts.
    let validator = match FitnessValidator::load(&target, config_path.as_deref()) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error: {e}");
            return 1;
        }
    };

    // Attach a previous report for trend deltas when requested. The path is
    // resolved relative to the validate target (per the spec). A requested but
    // unreadable or malformed previous report is a hard error: a silently
    // dropped trend would mislead.
    let validator = match previous_path {
        None => validator,
        Some(prev) => {
            let prev_path = {
                let p = std::path::Path::new(prev);
                if p.is_absolute() {
                    p.to_path_buf()
                } else {
                    target.join(p)
                }
            };
            match std::fs::read_to_string(&prev_path)
                .map_err(|e| e.to_string())
                .and_then(|s| serde_json::from_str::<FitnessReport>(&s).map_err(|e| e.to_string()))
            {
                Ok(prev_report) => validator.with_previous_report(prev_report),
                Err(e) => {
                    eprintln!(
                        "Error reading previous report '{}': {e}",
                        prev_path.display()
                    );
                    return 1;
                }
            }
        }
    };

    let report = match validator.validate() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error during validation: {e}");
            return 1;
        }
    };

    // Print human-readable summary.
    println!("Fitness Validation Report");
    println!("========================\n");
    for result in &report.results {
        let status_icon = match result.status {
            RuleStatus::Pass => "PASS",
            RuleStatus::Fail => "FAIL",
            RuleStatus::Warn => "WARN",
            RuleStatus::Skip => "SKIP",
        };
        let dim = result
            .dimension
            .as_deref()
            .map(|d| format!(" [{d}]"))
            .unwrap_or_default();
        println!(
            "  [{status_icon}] {} ({}){dim}",
            result.rule_name, result.rule_id
        );
        for v in &result.violations {
            let exc = if v.excepted { " (excepted)" } else { "" };
            println!(
                "         {} = {} (threshold: {} {:?}){exc}",
                v.entity, v.actual, v.threshold, v.direction
            );
        }
    }

    if !report.stale_exceptions.is_empty() {
        println!("\nStale Exceptions:");
        for s in &report.stale_exceptions {
            println!("  {} [{}]: {:?}", s.entity, s.rule_id, s.reason);
        }
    }

    // Surface the system-level fitness composite (new in APS-V1-0002).
    if let Some(sf) = &report.system_fitness {
        let verdict = if sf.passing { "PASS" } else { "FAIL" };
        println!(
            "\nSystem Fitness: {:.3} (min {:.3}) [{verdict}]",
            sf.score, sf.min_score
        );
        if let Some(note) = &sf.weights_note {
            println!("  note: {note}");
        }
    }

    println!(
        "\nSummary: {} passed, {} failed, {} warned, {} skipped, {} violations ({} excepted), {} stale exceptions",
        report.summary.passed,
        report.summary.failed,
        report.summary.warned,
        report.summary.skipped,
        report.summary.total_violations,
        report.summary.excepted_violations,
        report.summary.stale_exceptions,
    );

    // Write JSON report if requested. A failed write or serialize is a hard
    // error: returning success with a missing report artifact would let CI pass
    // while the expected output is absent.
    let mut report_write_failed = false;
    if let Some(report_file) = report_path {
        match serde_json::to_string_pretty(&report) {
            Ok(json) => {
                if let Err(e) = std::fs::write(report_file, json) {
                    eprintln!("Error writing report to '{report_file}': {e}");
                    report_write_failed = true;
                } else {
                    println!("\nReport written to: {report_file}");
                }
            }
            Err(e) => {
                eprintln!("Error serializing report: {e}");
                report_write_failed = true;
            }
        }
    }

    if report_write_failed || FitnessValidator::has_failures(&report) {
        1
    } else {
        0
    }
}

/// Print the fitness CLI help text.
fn print_help() {
    println!(
        "Architecture Fitness Functions ({}) v{}",
        crate::ID,
        env!("CARGO_PKG_VERSION")
    );
    println!();
    println!("USAGE:");
    println!("    apss-dev run architecture-fitness <COMMAND> [OPTIONS]");
    println!();
    println!("COMMANDS:");
    println!("    validate <path>    Validate fitness rules against topology artifacts");
    println!();
    println!("OPTIONS:");
    println!("    --config <file>            Path to fitness.toml (default: ./fitness.toml)");
    println!("    --report <file>            Write JSON report to file");
    println!(
        "    --previous-report <file>   Prior JSON report for trend deltas (alias: --previous)"
    );
    println!("    --help                     Show this help message");
}

/// The command list returned by `commands()` and used by `register()`.
fn command_infos() -> Vec<CommandInfo> {
    vec![CommandInfo {
        name: "validate".to_string(),
        description: "Validate fitness rules against topology artifacts".to_string(),
        usage: "validate <path>".to_string(),
    }]
}

/// Command names registered by `register()`; kept in sync with [`command_infos`].
pub(crate) const COMMAND_NAMES: [&str; 1] = ["validate"];
