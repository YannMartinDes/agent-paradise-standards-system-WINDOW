//! Reads `[cli] commands = "none"` declarations from standard metadata.
//!
//! A standard or experiment may declare that it intentionally ships no CLI
//! commands. Only an explicit declaration exempts it from
//! `CL_NO_REGISTERED_COMMANDS` (see ADR-0002 and issue #69).

use std::collections::HashSet;
use std::path::Path;

/// Scan package directories for metadata declaring `[cli] commands = "none"`.
///
/// Returns the set of exempted standard IDs. Looks for `standard.toml` or
/// `experiment.toml` directly inside each directory.
pub fn collect_cli_exemptions(package_dirs: &[std::path::PathBuf]) -> HashSet<String> {
    let mut exempt = HashSet::new();
    for dir in package_dirs {
        for meta_name in ["standard.toml", "experiment.toml"] {
            let meta_path = dir.join(meta_name);
            if let Some(id) = exemption_id_from_file(&meta_path) {
                exempt.insert(id);
            }
        }
    }
    exempt
}

fn exemption_id_from_file(meta_path: &Path) -> Option<String> {
    let content = std::fs::read_to_string(meta_path).ok()?;
    let value: toml::Value = content.parse().ok()?;

    let declares_none = value
        .get("cli")
        .and_then(|cli| cli.get("commands"))
        .and_then(|c| c.as_str())
        == Some("none");
    if !declares_none {
        return None;
    }

    for table in ["standard", "experiment"] {
        if let Some(id) = value
            .get(table)
            .and_then(|t| t.get("id"))
            .and_then(|i| i.as_str())
        {
            return Some(id.to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collects_exemption_from_standard_toml() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("standard.toml"),
            r#"
schema = "aps.standard/v1"

[standard]
id = "APS-V1-9999"
name = "Docs Only"
slug = "docs-only"
version = "0.1.0"

[cli]
commands = "none"
"#,
        )
        .unwrap();

        let exempt = collect_cli_exemptions(&[dir.path().to_path_buf()]);
        assert!(exempt.contains("APS-V1-9999"));
    }

    #[test]
    fn no_declaration_means_no_exemption() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("standard.toml"),
            r#"
schema = "aps.standard/v1"

[standard]
id = "APS-V1-9998"
name = "Normal"
slug = "normal"
version = "0.1.0"
"#,
        )
        .unwrap();

        let exempt = collect_cli_exemptions(&[dir.path().to_path_buf()]);
        assert!(exempt.is_empty());
    }
}
