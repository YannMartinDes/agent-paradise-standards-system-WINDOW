# Registered-Commands Validator (#69) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** `apss-dev v1 validate distribution` fails with `CL_NO_REGISTERED_COMMANDS` when a linked standard registers zero CLI commands, unless the standard explicitly declares `[cli] commands = "none"` in its metadata.

**Architecture:** A `CollectorRegistry` in `apss-core::registry` records what each standard's `register()` actually registers. A pure validation function turns the collected entries plus an exemption set into `Diagnostics`. `aps-cli` wires it into the existing `ValidateTarget::Distribution` arm, building the exemption set from package metadata files.

**Tech Stack:** Rust 1.85 workspace, existing `apss_core::diagnostics::{Diagnostic, Diagnostics}`, existing `apss_core::registry::{StandardRegistry, RegisteredStandard, CommandHandler, CommandInfo}`.

**Branch:** `feat/adr-0002-consumer-run` (this plan plus Phase C merge together; see the program roadmap for why main must not see the red state).

---

### Task 1: CollectorRegistry and validate_registered_commands in apss-core

**Files:**
- Modify: `crates/apss-core/src/registry.rs` (360 lines today; append new section before the tests module, tests go in the existing or a new `#[cfg(test)] mod tests`)

- [ ] **Step 1: Write the failing tests**

Append to the `#[cfg(test)]` module in `crates/apss-core/src/registry.rs` (create `mod tests` at the bottom of the file if none exists):

```rust
#[cfg(test)]
mod registered_commands_tests {
    use super::*;
    use std::collections::HashSet;

    struct StubHandler {
        cmds: Vec<CommandInfo>,
    }

    impl CommandHandler for StubHandler {
        fn execute(&self, _command: &str, _args: &[String], _config: &toml::Value) -> i32 {
            0
        }
        fn commands(&self) -> Vec<CommandInfo> {
            self.cmds.clone()
        }
    }

    fn standard(id: &str, slug: &str, commands: Vec<String>) -> RegisteredStandard {
        RegisteredStandard {
            id: id.to_string(),
            slug: slug.to_string(),
            name: slug.to_string(),
            description: "test standard".to_string(),
            version: "0.1.0".to_string(),
            commands,
        }
    }

    fn handler_with(names: &[&str]) -> Box<dyn CommandHandler> {
        Box::new(StubHandler {
            cmds: names
                .iter()
                .map(|n| CommandInfo {
                    name: n.to_string(),
                    description: format!("{n} command"),
                    usage: n.to_string(),
                })
                .collect(),
        })
    }

    #[test]
    fn flags_standard_with_no_commands() {
        let mut collector = CollectorRegistry::new();
        collector.register(standard("APS-V1-9998", "stub", Vec::new()), handler_with(&[]));

        let diags = validate_registered_commands(collector.entries(), &HashSet::new());

        assert!(diags.has_errors());
        assert!(
            diags
                .iter()
                .any(|d| d.code == "CL_NO_REGISTERED_COMMANDS" && d.message.contains("APS-V1-9998"))
        );
    }

    #[test]
    fn flags_mismatch_where_info_has_commands_but_handler_has_none() {
        let mut collector = CollectorRegistry::new();
        collector.register(
            standard("APS-V1-9997", "halfstub", vec!["analyze".to_string()]),
            handler_with(&[]),
        );

        let diags = validate_registered_commands(collector.entries(), &HashSet::new());

        assert!(diags.has_errors());
    }

    #[test]
    fn passes_standard_with_commands() {
        let mut collector = CollectorRegistry::new();
        collector.register(
            standard("APS-V1-9996", "real", vec!["analyze".to_string()]),
            handler_with(&["analyze"]),
        );

        let diags = validate_registered_commands(collector.entries(), &HashSet::new());

        assert!(!diags.has_errors());
    }

    #[test]
    fn exempted_standard_passes_with_no_commands() {
        let mut collector = CollectorRegistry::new();
        collector.register(standard("APS-V1-9995", "docsonly", Vec::new()), handler_with(&[]));

        let mut exempt = HashSet::new();
        exempt.insert("APS-V1-9995".to_string());

        let diags = validate_registered_commands(collector.entries(), &exempt);

        assert!(!diags.has_errors());
    }
}
```

Note: if `Diagnostic` fields `code`/`message` are not `pub`, check `crates/apss-core/src/diagnostics.rs:54` first; they are declared on the struct and used across crates, so they are public today.

- [ ] **Step 2: Run tests to verify they fail to compile**

Run: `cargo test -p apss-core registered_commands -- --nocapture`
Expected: compile error, `CollectorRegistry` and `validate_registered_commands` not found.

- [ ] **Step 3: Implement CollectorRegistry and validate_registered_commands**

Add to `crates/apss-core/src/registry.rs`, after the `ProjectRunner` section and before the tests:

```rust
// ============================================================================
// Registration Validation (CL01 poka-yoke, see issue #69 and ADR-0002)
// ============================================================================

/// Registry that records registrations without executing anything.
///
/// Used by validation to verify that each standard's `register()` actually
/// exposes CLI commands through the CL01 contract.
pub struct CollectorRegistry {
    entries: Vec<(RegisteredStandard, Box<dyn CommandHandler>)>,
}

impl CollectorRegistry {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// The recorded registrations.
    pub fn entries(&self) -> &[(RegisteredStandard, Box<dyn CommandHandler>)] {
        &self.entries
    }
}

impl Default for CollectorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl StandardRegistry for CollectorRegistry {
    fn register(&mut self, standard: RegisteredStandard, handler: Box<dyn CommandHandler>) {
        self.entries.push((standard, handler));
    }
}

/// Validate that every collected registration exposes at least one command.
///
/// Standards in `exempt_ids` (those declaring `[cli] commands = "none"` in
/// their metadata) are skipped. Silence is never a pass: a standard with no
/// commands and no declaration is an error.
pub fn validate_registered_commands(
    entries: &[(RegisteredStandard, Box<dyn CommandHandler>)],
    exempt_ids: &std::collections::HashSet<String>,
) -> crate::diagnostics::Diagnostics {
    use crate::diagnostics::{Diagnostic, Diagnostics};

    let mut diags = Diagnostics::new();
    for (info, handler) in entries {
        if exempt_ids.contains(&info.id) {
            continue;
        }
        if info.commands.is_empty() || handler.commands().is_empty() {
            diags.push(
                Diagnostic::error(
                    "CL_NO_REGISTERED_COMMANDS",
                    format!(
                        "standard {} ({}) registers no CLI commands; the composed consumer binary cannot run it",
                        info.id, info.slug
                    ),
                )
                .with_hint(
                    "populate RegisteredStandard::commands and CommandHandler::commands(), or declare `[cli]\ncommands = \"none\"` in the standard's metadata file",
                ),
            );
        }
    }
    diags
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p apss-core registered_commands`
Expected: 4 passed.

- [ ] **Step 5: Commit**

```bash
git add crates/apss-core/src/registry.rs
git commit -m "feat(core): add CollectorRegistry and registered-commands validation (CL01 poka-yoke)"
```

---

### Task 2: Exemption metadata reader in aps-cli

**Files:**
- Create: `crates/aps-cli/src/cli_exemptions.rs`
- Modify: `crates/aps-cli/src/main.rs` (add `mod cli_exemptions;` near the other module declarations at the top of the file)

- [ ] **Step 1: Write the failing test**

Create `crates/aps-cli/src/cli_exemptions.rs` with the test first:

```rust
//! Reads `[cli] commands = "none"` declarations from standard metadata.
//!
//! A standard or experiment may declare that it intentionally ships no CLI
//! commands. Only an explicit declaration exempts it from
//! `CL_NO_REGISTERED_COMMANDS` (see ADR-0002 and issue #69).

use std::collections::HashSet;
use std::path::Path;

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
```

- [ ] **Step 2: Run test to verify it fails to compile**

Run: `cargo test -p aps-cli cli_exemptions`
Expected: compile error, `collect_cli_exemptions` not found. (Add `mod cli_exemptions;` to `main.rs` first or the module never builds.)

- [ ] **Step 3: Implement collect_cli_exemptions**

Add above the tests module in `crates/aps-cli/src/cli_exemptions.rs`:

```rust
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
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p aps-cli cli_exemptions`
Expected: 2 passed.

- [ ] **Step 5: Commit**

```bash
git add crates/aps-cli/src/cli_exemptions.rs crates/aps-cli/src/main.rs
git commit -m "feat(cli): read [cli] commands=none exemptions from package metadata"
```

---

### Task 3: Wire the check into v1 validate distribution

**Files:**
- Modify: `crates/aps-cli/src/main.rs:325-347` (the `ValidateTarget::Distribution` arm)

- [ ] **Step 1: Extend the Distribution arm**

In the `ValidateTarget::Distribution` block (currently builds `all_diags` from `validate_publishable_standard` + `validate_release_readiness` per package), append after the package loop and before `all_diags` is returned:

```rust
// CL01 poka-yoke (issue #69, ADR-0002): every linked standard must
// actually register CLI commands. Silence is never a pass.
let mut collector = apss_core::registry::CollectorRegistry::new();
code_topology::register(&mut collector);
fitness_functions::register(&mut collector);

let package_dirs: Vec<std::path::PathBuf> =
    packages.iter().map(|p| p.path.clone()).collect();
let exempt = cli_exemptions::collect_cli_exemptions(&package_dirs);

all_diags.merge(apss_core::registry::validate_registered_commands(
    collector.entries(),
    &exempt,
));
```

Notes for the implementer:
- `code_topology` and `fitness_functions` are existing renamed deps in `crates/aps-cli/Cargo.toml` (packages `apss-v1-0001-code-topology` and `apss-v1-0003-fitness-functions`).
- The list of `register()` calls is exactly the set of standards linked into the composed runtime template. When Phase B merges substandards this list does not change (substandards have no own `register()`).

- [ ] **Step 2: Build and run the gate to verify it goes RED**

Run: `cargo run -p aps-cli --bin apss-dev -- v1 validate distribution`
Expected: FAILS (exit nonzero) with exactly two errors:

```text
CL_NO_REGISTERED_COMMANDS ... APS-V1-0001 (code-topology) registers no CLI commands ...
CL_NO_REGISTERED_COMMANDS ... EXP-V1-0003 (fitness) registers no CLI commands ...
```

(The exact id/slug strings come from each crate's `register()`; confirm against `standards/v1/APS-V1-0001-code-topology/src/lib.rs:976` and the fitness equivalent. If the fitness `register()` uses a different id string, the assertion text here follows it.)

This red state is the deliverable of this plan. Do NOT exempt these standards to make it pass; Phase C makes it green by wiring real handlers.

- [ ] **Step 3: Run the full workspace tests**

Run: `cargo test --workspace`
Expected: PASS. (No committed test asserts the live repo is red; redness is gate behavior, proven by the previous step. Unit tests from Tasks 1 and 2 stay green permanently.)

- [ ] **Step 4: Run clippy and fmt**

Run: `cargo clippy --workspace --all-targets -- -D warnings && cargo fmt --all --check`
Expected: clean.

- [ ] **Step 5: Commit**

```bash
git add crates/aps-cli/src/main.rs
git commit -m "feat(cli): enforce CL_NO_REGISTERED_COMMANDS in v1 validate distribution (red until #68)"
```

---

### Task 4: Spec text for the new rule (CL01)

**Files:**
- Modify: `standards/v1/APS-V1-0000-meta/substandards/CL01-cli-contract/docs/01_spec.md` (append a section; read the file first and match its heading style)
- Modify: `standards/v1/APS-V1-0000-meta/substandards/CL01-cli-contract/substandard.toml` (bump minor version)

- [ ] **Step 1: Add the normative section to the CL01 spec**

Append, matching the document's existing section numbering (read the file to find the next section number, referred to as N here):

```markdown
## N. Registered Commands Requirement

Every standard linked into a composed consumer binary MUST register at least
one CLI command through its `register()` function: `RegisteredStandard::commands`
MUST be non-empty and the registered `CommandHandler::commands()` MUST return a
non-empty list.

A standard that intentionally ships no executable commands MUST declare it in
its metadata file:

```toml
[cli]
commands = "none"
```

Validation emits `CL_NO_REGISTERED_COMMANDS` (Error) for any linked standard
that neither registers commands nor declares the exemption. Silence is never a
pass. This check runs inside `v1 validate distribution` and therefore in QA,
CI, and the release gate.
```

- [ ] **Step 2: Bump the CL01 substandard version**

In `standards/v1/APS-V1-0000-meta/substandards/CL01-cli-contract/substandard.toml`, bump the minor version (for example `1.0.0` to `1.1.0`; read the current value first). Check whether the CL01 `Cargo.toml` version must match per repo convention and bump it identically if so.

- [ ] **Step 3: Validate the repo structure**

Run: `cargo run -p aps-cli --bin apss-dev -- v1 validate repo`
Expected: PASS (structural validation; the distribution gate is still red, which is expected on this branch).

- [ ] **Step 4: Commit**

```bash
git add standards/v1/APS-V1-0000-meta/substandards/CL01-cli-contract/
git commit -m "docs(CL01): specify registered-commands requirement and exemption; bump to 1.1.0"
```

---

## Self-Review Notes

- Spec coverage: #69 acceptance criteria map to Task 3 step 2 (red against stubs), Task 1 (validator), Task 2 (explicit exemption), Task 4 (spec text). "Scaffolded standard fails out of the box" follows because scaffolds use the stub register.
- The red gate on this branch is intentional and documented in the program roadmap's branch strategy; main never sees it.
- Type consistency: `CollectorRegistry::entries()` returns `&[(RegisteredStandard, Box<dyn CommandHandler>)]`; `validate_registered_commands` takes exactly that slice; Task 3 passes `collector.entries()`.
