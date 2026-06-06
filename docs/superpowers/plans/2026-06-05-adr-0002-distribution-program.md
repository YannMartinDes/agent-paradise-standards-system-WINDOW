# ADR-0002 Distribution Program Roadmap

> **For agentic workers:** This is a PROGRAM roadmap, not an executable task plan. Each phase below gets its own plan document before execution. Phase A's plan exists: `2026-06-05-registered-commands-validator.md`. Write each subsequent plan only when its phase starts, because details depend on the previous phase's outcome.

**Goal:** Make the consumer path turnkey per ADR-0002: `cargo install apss && apss init && apss install && apss run <standard> <command>` with crates.io as the standards transport.

**Authority:** `standards/v1/APS-V1-0000-meta/docs/adrs/0002-crates-io-distribution.md` (accepted 2026-06-05). Issues: #65 (deployment tracker), #68 (composed runtime stub), #69 (poka-yoke validator), #70 (CLI UX nits), #62 (registry, dissolved by ADR-0002).

## Current State (verified 2026-06-05)

- `apss-core` 1.0.0 and `apss` 1.0.0 are live on crates.io.
- Consumer install/validate/hooks work end to end from a local bundle.
- `apss run <standard> <cmd>` fails: every standard's `register()` is a stub (`commands: Vec::new()`, `NoopCommandHandler`). Command glue lives in `aps-cli/src/main.rs` (`dispatch_topology` at line 988, `dispatch_fitness` similarly).
- DI01 validation checks that `pub fn register` exists textually (`crates/apss-core/src/distribution/mod.rs:184`) but never that it registers commands. That is the poka-yoke hole.
- Substandards are separate workspace crates; the parent `code-topology` crate does not depend on them; viz glue lives only in aps-cli.

## Phases

### Phase A: Registered-commands validator (#69)

Plan: `2026-06-05-registered-commands-validator.md` (complete, ready to execute).

`CollectorRegistry` + `validate_registered_commands()` in `apss-core::registry`, wired into `apss-dev v1 validate distribution`. Diagnostic `CL_NO_REGISTERED_COMMANDS` (Error). Exemption only by explicit `[cli] commands = "none"` in `standard.toml`/`experiment.toml`. Unit tests use stub fixtures (permanently green); the live repo goes RED by design until Phase C.

### Phase B: Substandard merge (ADR-0002 points 2, SS01 amendment)

Merge code-topology's five substandard crates (`3D01-force-directed`, `LANG01-rust`, `VIZ01-dashboard`, `VIZ01-mermaid`, `CI01-github-actions`) into the parent crate as feature-gated modules (`viz-3d`, `lang-rust`, `viz-dashboard`, `viz-mermaid`, `ci-github-actions`). Mechanical move, no behavior change: aps-cli imports update, workspace members shrink, bundle generator simplifies. Substandards keep `substandard.toml`, docs, and versions; SS01 spec amended in the same phase (module layout + feature naming rules replace the per-substandard `Cargo.toml` mandate for published standards), SS01 version bump.

Runs in parallel with Phase A; no dependency between them.

### Phase C: CommandHandler wiring (#68)

Depends on A and B. Move `dispatch_topology` glue plus the `topology_*` helper functions from `aps-cli/src/main.rs` into the merged code-topology crate as a real `CommandHandler` (commands: analyze, validate, diff, check, comment, report, viz). Same for fitness-functions. `register()` exposes the full command list; aps-cli consumes the same registration (removes ~300 lines of monolith, advances #36). Phase A's gate flips GREEN. Exit criterion: in the tester repo (`/Users/neural/Code/AgentParadise/apss-example-repo`), `.apss/bin/apss run code-topology analyze .` and `... viz .topology --type all` work after a fresh bundle install.

### Phase D: crates.io publishing of standards + turnkey install (ADR-0002 points 1, 4, 5, 6)

Depends on C. Four work items:

1. Publish metadata for the merged standard crates (description, readme, keywords, categories) and extend `validate_publishable_standard` to require it.
2. `apss install` default path: resolve `APSS.yaml` standards to crates.io dependencies with substandard features, generate the composed project with registry deps instead of path deps. `--bundle-dir`/`--local-repo` remain as dev paths.
3. Restore tiered publishing in `release-create.yml`: tier 1 `apss-core`, tier 2 standard crates (changed only), tier 3 `apss`. Publish-scope validation enforces the ADR-0002 set.
4. DI01 spec amendment (sections 9.3, 9.4, bundle docs demoted to catalog) + DI01 version bump. First manual publish of `apss-v1-0001-code-topology`.

### Phase E: Proof and propagation

Rewrite `docs/runbooks/visualize-your-codebase.runbook.md` down to the five-command turnkey flow. Re-run full e2e in the tester repo using only published artifacts. Then: harness-app-template integration (session task 4), release infra (session task 5), README sync design (parked brainstorm).

## Branch and CI Strategy

Phase A's gate makes `v1 validate distribution` fail against today's stubs. That redness must not land on main:

- Phase B merges to main independently (always green).
- Phases A and C ride one integration branch (`feat/adr-0002-consumer-run`) and merge together. Commit history preserves the red-then-green proof; CI on main never breaks.
- Phase D is normal PR flow afterward.

## Out of Scope

- #70 UX nits (path-arg consistency, slug aliases): fix opportunistically during Phase C since the handler code moves anyway; not a phase gate.
- Topology artifact slimming (heavy `functions.json`): noted in the runbook as future work.
- Private registries.
