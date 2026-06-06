//! `report` command: render a human-readable module table.

/// Generate a human-readable topology report.
pub(super) fn topology_report(path: &str, _verbose: bool) -> i32 {
    use std::path::Path;

    let topology_path = Path::new(path);
    let modules_path = topology_path.join("metrics/modules.json");

    if !modules_path.exists() {
        eprintln!("Error: No topology artifacts found at {path}");
        eprintln!("Run 'apss-dev run topology analyze' first.");
        return 1;
    }

    // Load modules and generate report
    if let Ok(content) = std::fs::read_to_string(&modules_path) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(modules) = json.get("modules").and_then(|m| m.as_array()) {
                println!("# Code Topology Report");
                println!();
                println!("## Modules ({})", modules.len());
                println!();
                println!("| Module | Functions | Avg CC | Instability |");
                println!("|--------|-----------|--------|-------------|");

                for module in modules {
                    let id = module.get("id").and_then(|v| v.as_str()).unwrap_or("?");
                    let metrics = module.get("metrics");
                    let func_count = metrics
                        .and_then(|m| m.get("function_count"))
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    let avg_cc = metrics
                        .and_then(|m| m.get("avg_cyclomatic"))
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0);
                    let instability = metrics
                        .and_then(|m| m.get("martin"))
                        .and_then(|m| m.get("instability"))
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0);

                    println!("| {id} | {func_count} | {avg_cc:.1} | {instability:.2} |");
                }

                return 0;
            }
        }
    }

    eprintln!("Error: Could not parse modules.json");
    1
}
