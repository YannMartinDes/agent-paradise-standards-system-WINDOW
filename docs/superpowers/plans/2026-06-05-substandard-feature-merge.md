# Substandard Feature-Module Merge (Phase B) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking. Tasks 2 through 6 are sequential (they all touch the parent crate's Cargo.toml and lib.rs); do not parallelize them.

**Goal:** Merge code-topology's five substandard crates into the parent crate `apss-v1-0001-code-topology` as feature-gated modules, per ADR-0002 point 2, with no behavior change (all features default-on), and amend SS01 accordingly.

**Architecture:** Each substandard's source moves to `standards/v1/APS-V1-0001-code-topology/src/substandards/<module>/`, gated by `#[cfg(feature = "<feature>")]`. Cross-crate paths `crate::` (referring to the old substandard crate) become module-relative; references to the parent crate (`code_topology::`) become `crate::`. Substandard directories keep `substandard.toml` and `docs/` (governed units survive; only the crate boundary dissolves).

**Tech Stack:** Rust 1.85 workspace, cargo features, existing workspace deps.

**Branch:** `feat/adr-0002-consumer-run` (continues from Phase A).

## Name Mapping (single source of truth for all tasks)

| Substandard dir | Old crate (lib name) | New module path | Feature name | LOC | Extra external deps to move |
|---|---|---|---|---|---|
| `CI01-github-actions` | `code_topology_ci_github_actions` | `substandards::ci_github_actions` | `ci-github-actions` | 84 | none |
| `VIZ01-mermaid` | `code_topology_mermaid` | `substandards::viz_mermaid` | `viz-mermaid` | 462 | none |
| `3D01-force-directed` | `code_topology_3d` | `substandards::viz_3d` | `viz-3d` | 1358 | none |
| `LANG01-rust` | `code_topology_rust_adapter` | `substandards::lang_rust` | `lang-rust` | 1150 | `syn` (full, parsing, visit, extra-traits), `quote`, `proc-macro2` |
| `VIZ01-dashboard` | `code_topology_viz` | `substandards::viz_dashboard` | `viz-dashboard` | 2575 (6 files) | `chrono` (0.4, std) |

All five features go into `[features]` with `default = ["ci-github-actions", "viz-mermaid", "viz-3d", "lang-rust", "viz-dashboard"]` (no behavior change in Phase B; consumers opt out later via the composed-project generator in Phase D).

## Per-Substandard Merge Recipe (applies to Tasks 2 through 6)

For substandard S with old lib L, new module M, feature F:

1. `mkdir -p standards/v1/APS-V1-0001-code-topology/src/substandards` (first task only) and add to parent `src/lib.rs`: `pub mod substandards;` with a `src/substandards/mod.rs` declaring each module behind its feature gate as it lands: `#[cfg(feature = "F")] pub mod M;`
2. `git mv standards/v1/APS-V1-0001-code-topology/substandards/S/src/lib.rs standards/v1/APS-V1-0001-code-topology/src/substandards/M.rs` (or the whole `src/` to `M/` when S has multiple files: VIZ01-dashboard moves `lib.rs` to `M/mod.rs` and siblings alongside).
3. Inside the moved file(s): replace `code_topology::` with `crate::`; replace any `crate::` that referred to the substandard's own items with module-relative paths (single-file crates: plain `crate::` to `self`-relative is usually just deleting the prefix; check each use block). Replace `use apss_core::...` unchanged (still external).
4. Delete the substandard's `Cargo.toml` and empty `src/` and placeholder `tests/.gitkeep` (keep `substandard.toml`, `docs/`, any non-code assets).
5. Remove the member line from root `Cargo.toml` workspace members.
6. Add feature F to parent `Cargo.toml` `[features]`; move S's extra external deps (table above) into parent `[dependencies]` as `optional = true`, referenced by F: for example `syn = { version = "2", features = ["full", "parsing", "visit", "extra-traits"], optional = true }` and `lang-rust = ["dep:syn", "dep:quote", "dep:proc-macro2"]`.
7. Keep the substandard's `pub fn register(...)` as a module-level function (Phase C consumes or removes these; do not delete in Phase B).
8. Inline `#[cfg(test)]` tests move with the file and stay green.
9. `cargo check --workspace` then `cargo test -p apss-v1-0001-code-topology` must pass before moving to the next substandard.

---

### Task 1: Feature scaffolding in the parent crate

**Files:**
- Modify: `standards/v1/APS-V1-0001-code-topology/Cargo.toml` (add empty `[features]` table with `default = []` to grow per task)
- Modify: `standards/v1/APS-V1-0001-code-topology/src/lib.rs` (add `pub mod substandards;`)
- Create: `standards/v1/APS-V1-0001-code-topology/src/substandards/mod.rs` (doc comment only at first: `//! Substandard implementations as feature-gated modules (ADR-0002).`)

- [ ] Step 1: make the three edits above
- [ ] Step 2: Run `cargo check --workspace`. Expected: PASS.
- [ ] Step 3: Commit: `git commit -m "refactor(topology): scaffold feature-gated substandards module (ADR-0002 phase B)"`

### Task 2: Merge CI01-github-actions (smallest, proves the recipe)

Apply the recipe with S=`CI01-github-actions`, L=`code_topology_ci_github_actions`, M=`ci_github_actions`, F=`ci-github-actions`. No external deps. Nothing imports this crate outside the substandard itself.

- [ ] Step 1: recipe items 2 through 8
- [ ] Step 2: Run `cargo check --workspace && cargo test -p apss-v1-0001-code-topology`. Expected: PASS, the 2 moved inline tests run.
- [ ] Step 3: Run `cargo run -p aps-cli --bin apss-dev -- v1 validate repo`. If structural validation fails because the substandard lost its `Cargo.toml`/`src/`, locate the failing validator (start from the diagnostic code; check `crates/apss-core/src/discovery.rs` and meta-standard validators) and adjust it to accept module-merged substandards (a substandard directory is valid with `substandard.toml` + `docs/` and no `Cargo.toml`). Record what changed; this is part of the SS01 amendment.
- [ ] Step 4: Commit: `git commit -m "refactor(topology): merge CI01 into parent as ci-github-actions feature module"`

### Task 3: Merge VIZ01-mermaid

Recipe with S=`VIZ01-mermaid`, L=`code_topology_mermaid`, M=`viz_mermaid`, F=`viz-mermaid`. No external deps. Check parent `[dev-dependencies]` (`code-topology-mermaid` alias) and parent examples/tests that import `code_topology_mermaid::` and update them to `crate::substandards::viz_mermaid::` (tests) or `apss_v1_0001_code_topology::substandards::viz_mermaid::` (examples).

- [ ] Step 1: recipe + dev-dependency cleanup
- [ ] Step 2: `cargo check --workspace && cargo test -p apss-v1-0001-code-topology`. Expected: PASS.
- [ ] Step 3: Commit: `git commit -m "refactor(topology): merge VIZ01-mermaid into parent as viz-mermaid feature module"`

### Task 4: Merge 3D01-force-directed

Recipe with S=`3D01-force-directed`, L=`code_topology_3d`, M=`viz_3d`, F=`viz-3d`. No external deps. Importers to update:
- `crates/aps-cli/Cargo.toml` line 18: delete the `code-topology-3d` dependency line
- `crates/aps-cli/src/main.rs:3139`: `use code_topology_3d::ForceDirectedProjector;` becomes `use code_topology::substandards::viz_3d::ForceDirectedProjector;`
- parent dev-dependencies alias `code-topology-3d` and any parent tests/examples using it

- [ ] Step 1: recipe + importer updates
- [ ] Step 2: `cargo check --workspace && cargo test --workspace`. Expected: PASS.
- [ ] Step 3: Commit: `git commit -m "refactor(topology): merge 3D01 into parent as viz-3d feature module"`

### Task 5: Merge LANG01-rust

Recipe with S=`LANG01-rust`, L=`code_topology_rust_adapter`, M=`lang_rust`, F=`lang-rust`. External deps `syn`/`quote`/`proc-macro2` move to parent as optional behind `lang-rust`. Importers:
- `crates/aps-cli/Cargo.toml` line 20: delete `code-topology-rust-adapter`
- grep `code_topology_rust_adapter::` across `crates/aps-cli/src/` and parent tests/examples; update to `code_topology::substandards::lang_rust::`

- [ ] Step 1: recipe + importer updates
- [ ] Step 2: `cargo check --workspace && cargo test --workspace`. Expected: PASS.
- [ ] Step 3: Commit: `git commit -m "refactor(topology): merge LANG01 into parent as lang-rust feature module"`

### Task 6: Merge VIZ01-dashboard (multi-file)

Recipe with S=`VIZ01-dashboard`, L=`code_topology_viz`, M=`viz_dashboard` (directory module: `lib.rs` becomes `viz_dashboard/mod.rs`; `clusters.rs`, `codecity.rs`, `force_3d.rs`, `index.rs`, `vsa.rs` move alongside), F=`viz-dashboard`. `chrono` moves to parent as optional (note: aps-cli has its own chrono dep, untouched). Importers:
- `crates/aps-cli/Cargo.toml` line 19: delete `code-topology-viz`
- `crates/aps-cli/src/main.rs:3381,3395,3455,3515`: `code_topology_viz::codecity::generate(...)` etc. become `code_topology::substandards::viz_dashboard::codecity::generate(...)` (same pattern for clusters, vsa, index)

- [ ] Step 1: recipe + importer updates
- [ ] Step 2: `cargo check --workspace && cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings && cargo fmt --all`. Expected: all PASS.
- [ ] Step 3: Commit: `git commit -m "refactor(topology): merge VIZ01-dashboard into parent as viz-dashboard feature module"`

### Task 7: SS01 amendment and final verification

**Files:**
- Modify: `standards/v1/APS-V1-0000-meta/substandards/SS01-substandard-structure/docs/01_spec.md` (section 4 Package Layout)
- Modify: `standards/v1/APS-V1-0000-meta/substandards/SS01-substandard-structure/substandard.toml` (+ its `Cargo.toml` if independently versioned): minor bump
- Modify: any validator adjusted in Task 2 step 3 (already committed there; reference it in the spec text)

- [ ] Step 1: Amend SS01 section 4 to read, in the document's own style: a substandard of a published standard keeps `substandard.toml` and `docs/` as its governed-unit identity; its implementation lives in the parent crate under `src/substandards/<module>/` behind a cargo feature named after the substandard; a standalone `Cargo.toml`/`src/` per substandard is the layout for internal (unpublished) standards only. Reference ADR-0002.
- [ ] Step 2: Bump SS01 minor version (read current values first; bump both substandard.toml and Cargo.toml if they track each other).
- [ ] Step 3: Full gate: `just qa` EXCEPT expect `v1 validate distribution` to stay red with exactly the same 2 `CL_NO_REGISTERED_COMMANDS` errors from Phase A (that redness belongs to Phase C, not this phase). Everything else green. Run `cargo run -p aps-cli --bin apss-dev -- v1 validate repo` and expect PASS.
- [ ] Step 4: Bundle still works: `cargo run -p aps-cli --bin apss-dev -- v1 bundle APS-V1-0001 --output /tmp/apss-phaseb-bundle` and verify the bundle workspace has the parent crate only (no substandard members) and compiles: `cargo check` inside the bundle dir.
- [ ] Step 5: Commit: `git commit -m "docs(SS01): substandards of published standards are feature modules; bump version"`

## Self-Review Notes

- Recipe step 3 is where mechanical errors will hide (path rewriting); the per-task `cargo check --workspace` catches them immediately, which is why tasks are sequential and small-to-large.
- Parent standard.toml version: bump the standard's own version (0.1.0 to 0.2.0) in Task 7 if `v1 validate repo` or the release gate's version-bump check requires it; the parent crate changed substantially.
- The five module-level `register()` functions survive unused in Phase B; clippy may flag dead code only if they are private (they are `pub`, so no warning).
- aps-cli's `code-topology` alias for the parent remains valid throughout; only the three substandard aliases disappear.
