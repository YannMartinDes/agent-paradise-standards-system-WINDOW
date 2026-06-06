//! Composed CLI for the Fitness Functions experiment (ADR-0002, issue #68/#69).
//!
//! This module hosts the command implementation that backs `apss run
//! fitness-functions validate` in composed consumer binaries and `apss-dev run
//! fitness validate` in the development CLI. Both routes dispatch through
//! [`FitnessCommandHandler`], which implements
//! [`apss_core::registry::CommandHandler`].
//!
//! Impedance notes:
//! - `CommandHandler::execute` receives no repo root, so the handler resolves
//!   `repo_root = std::env::current_dir()`.
//! - verbose output is env-driven: the handler reads `APSS_VERBOSE=1`. The
//!   fitness validate command does not currently branch on verbose, but the
//!   transport is preserved for parity with the topology handler.
//! - command functions return `i32` (0 success, 1 error, 3 usage).

use apss_core::registry::{CommandHandler, CommandInfo};

use crate::{FitnessValidator, RuleStatus};

/// Handler that backs `run fitness-functions <command>` in composed binaries.
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
        let _verbose = std::env::var("APSS_VERBOSE").is_ok_and(|v| v == "1");

        dispatch(command, args, &repo_root, _verbose)
    }

    fn commands(&self) -> Vec<CommandInfo> {
        command_infos()
    }
}

/// Dispatch a fitness command to its implementation.
fn dispatch(command: &str, args: &[String], repo_root: &std::path::Path, _verbose: bool) -> i32 {
    match command {
        "--help" | "-h" | "help" => {
            print_help();
            0
        }
        "validate" => validate(args, repo_root),
        other => {
            eprintln!("Error: Unknown fitness command '{other}'");
            eprintln!("Use 'apss-dev run fitness --help' for available commands.");
            3
        }
    }
}

/// Validate fitness rules against topology artifacts.
fn validate(args: &[String], repo_root: &std::path::Path) -> i32 {
    // Parse flags and positional args separately to avoid
    // `--config custom.toml .` misinterpreting "--config" as the path
    let mut positional_path: Option<&str> = None;
    let mut config_path: Option<std::path::PathBuf> = None;
    let mut report_path: Option<&String> = None;
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

    // Resolve --config relative to target repo, not CWD
    let config_path = config_path.map(|p| if p.is_absolute() { p } else { target.join(p) });

    let validator = match FitnessValidator::load(&target, config_path.as_deref()) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error: {e}");
            return 1;
        }
    };

    let report = match validator.validate() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error during validation: {e}");
            return 1;
        }
    };

    // Print human-readable summary
    println!("Fitness Validation Report");
    println!("========================\n");
    for result in &report.results {
        let status_icon = match result.status {
            RuleStatus::Pass => "PASS",
            RuleStatus::Fail => "FAIL",
            RuleStatus::Warn => "WARN",
            RuleStatus::Skip => "SKIP",
        };
        println!(
            "  [{status_icon}] {} ({})",
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

    println!(
        "\nSummary: {} passed, {} failed, {} warned, {} violations ({} excepted), {} stale exceptions",
        report.summary.passed,
        report.summary.failed,
        report.summary.warned,
        report.summary.total_violations,
        report.summary.excepted_violations,
        report.summary.stale_exceptions,
    );

    // Write JSON report if requested
    if let Some(report_file) = report_path {
        match serde_json::to_string_pretty(&report) {
            Ok(json) => {
                if let Err(e) = std::fs::write(report_file, json) {
                    eprintln!("Error writing report: {e}");
                } else {
                    println!("\nReport written to: {report_file}");
                }
            }
            Err(e) => eprintln!("Error serializing report: {e}"),
        }
    }

    if FitnessValidator::has_failures(&report) {
        1
    } else {
        0
    }
}

/// Print the fitness CLI help text.
fn print_help() {
    println!("Architecture Fitness Functions (EXP-V1-0003) v0.1.0");
    println!();
    println!("USAGE:");
    println!("    apss-dev run fitness <COMMAND> [OPTIONS]");
    println!();
    println!("COMMANDS:");
    println!("    validate <path>    Validate fitness rules against topology artifacts");
    println!();
    println!("OPTIONS:");
    println!("    --config <file>    Path to fitness.toml (default: ./fitness.toml)");
    println!("    --report <file>    Write JSON report to file");
    println!("    --help             Show this help message");
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
