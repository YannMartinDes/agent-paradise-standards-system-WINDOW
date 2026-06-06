# Runbook: Visualize Your Codebase with the Topology Standard

## Purpose

Install APSS in a target repository, run the Code Topology standard (APS-V1-0001) against its source code, and generate interactive visualizations: a 3D coupling graph, CodeCity, cluster map, VSA matrix, and a combined dashboard. Optionally wire validation into a git pre-commit hook.

This runbook is written to be handed to a coding agent (for example Claude Code) verbatim. Every step has a command and an expected result. Run steps in order.

## Audience and Prerequisites

- A target repository containing Rust (`.rs`) or Python (`.py`/`.pyi`) source files. Other languages are not yet supported by the analyzer.
- Rust toolchain and Cargo installed (`cargo --version` succeeds).
- Network access to crates.io for step 1.
- Until the public bundle registry ships, a local checkout of the APSS repository is required for steps 2 and 5: `git clone https://github.com/AgentParadise/agent-paradise-standards-system`. Set `APSS_REPO` to its path.

> **Current limitation (issue #68):** the composed consumer binary cannot yet execute standard commands (`apss run code-topology ...` reports no registered commands). Until #68 lands, analysis and visualization run through the `apss-dev` CLI built from the APSS repository checkout. Steps 5 and 6 use that path. Everything else uses the published `apss` CLI.

## 1. Install the APSS CLI

```bash
cargo install apss
apss --version
```

Expected: `apss 1.0.0` (or newer) on PATH.

## 2. Build the Topology Bundle

From the APSS repository checkout:

```bash
cd "$APSS_REPO"
mkdir -p /tmp/apss-bundles
cargo run -p aps-cli --bin apss-dev -- v1 bundle APS-V1-0001 --output /tmp/apss-bundles
```

Expected: `Created APSS bundle: /tmp/apss-bundles/APS-V1-0001-code-topology-<version>.apss`. The bundle is a self-contained source workspace: `apss-core`, the standard crate, and all substandard crates. Nothing else is downloaded.

## 3. Initialize and Install in the Target Repository

From the target repository root:

```bash
apss init
apss install --bundle-dir /tmp/apss-bundles/APS-V1-0001-code-topology-<version>.apss
```

Expected:

- `APSS.yaml` created (user-owned, edit freely) and `apss.lock` written.
- A composed binary compiled to `.apss/bin/apss`.
- `Installed Git hook: .git/hooks/pre-commit` (see step 7).

Commit `APSS.yaml` and `apss.lock`. Do not commit `.apss/`; it is generated output.

## 4. Validate the Installation

```bash
apss validate
apss status
```

Expected: `Validation passed.` and a status listing `code-topology  APS-V1-0001 <version> [all]` with the binary marked installed.

## 5. Run the Topology Analysis

From the target repository root (note: `apss-dev` path until #68 lands):

```bash
"$APSS_REPO/target/release/apss-dev" run topology analyze . --output .topology
```

If the binary does not exist yet, build it first: `cargo build --release -p aps-cli` in `$APSS_REPO`.

Expected output shape:

```text
Found N source file(s):
  rust: N files
✓ Analyzed M functions (0 warnings)
✓ Wrote artifacts to .topology
```

Artifacts produced: `.topology/manifest.toml`, `metrics/modules.json`, `metrics/functions.json`, `metrics/slices.json`, `graphs/dependencies.json`, `graphs/coupling-matrix.json`.

> **Warning: topology JSON can get heavy.** Artifact size grows with codebase size, and `functions.json` dominates: a 14-file demo produces ~176 KB of artifacts, while a medium Rust workspace already produces a 400+ KB `functions.json`. Large monorepos can reach many megabytes. Do not load these files into an agent context wholesale; query specific fields instead. Consider adding `.topology/` to `.gitignore` and regenerating in CI rather than committing artifacts. Artifact slimming/sharding is a known future optimization.

## 6. Generate Visualizations

3D coupling graph only:

```bash
"$APSS_REPO/target/release/apss-dev" run topology viz .topology --type 3d
```

Full dashboard (recommended):

```bash
"$APSS_REPO/target/release/apss-dev" run topology viz .topology --type all
```

Expected: HTML files under `.topology/viz/` (`index.html`, `topology-3d.html`, `codecity.html`, `clusters.html`, `vsa.html`); the dashboard opens in the default browser. The viz command takes the `.topology` directory path, not the repo root (issue #70 tracks this inconsistency).

Visualization types: `3d` (force-directed coupling), `codecity` (buildings = modules), `clusters` (2D package graph), `vsa` (vertical slice matrix), `all`.

## 7. Git Pre-Commit Hook (Optional but Recommended)

`apss install` already installed a pre-commit hook that runs `apss validate` on every commit. Control it via `APSS.yaml`:

```yaml
tool:
  hooks:
    pre_commit: true   # set false to disable, then rerun: apss install
```

Verify it works:

```bash
git commit --allow-empty -m "test: hook check"
```

Expected: validation output before the commit completes. Hook failures block the commit.

## Troubleshooting

| Symptom | Cause and Fix |
|---|---|
| `No supported source files found` | Target repo has no `.rs`/`.py` files at or below the analyzed path. Point `analyze` at the source root. |
| `Standard 'topology' not found in APSS.yaml` | The consumer config uses the slug `code-topology`; `topology` is a dev-CLI alias only (issue #70). |
| `No modules.json found at ./metrics/modules.json` | `viz` was given the repo root. Pass the `.topology` directory. |
| `No composed CLI commands are registered` | Known gap, issue #68. Use the `apss-dev` path shown in steps 5 and 6 until it lands. |
| `cargo install apss` fails on publish metadata | Update Rust; the workspace requires Rust 1.85+. |

## Related

- Consumer flow overview: root `README.md`, section "Using APSS in Your Project"
- Distribution lifecycle: `standards/v1/APS-V1-0000-meta/substandards/DI01-distribution/docs/03_package_manager_lifecycle.md`
- Release acceptance testing: `docs/testing/apss-package-manual-acceptance-testing.runbook.md`
- Tracking: #67 (runbooks), #68 (composed runtime commands), #70 (CLI UX)
