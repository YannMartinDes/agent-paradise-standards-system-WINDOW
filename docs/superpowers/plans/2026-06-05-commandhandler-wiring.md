# CommandHandler Wiring (Phase C, #68) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Tasks are sequential.

**Goal:** `apss run code-topology <cmd>` and `apss run fitness-functions validate` work in composed consumer binaries; the Phase A gate (`CL_NO_REGISTERED_COMMANDS`) goes green; `apss-dev run ...` keeps identical behavior through the same code path.

**Architecture:** The topology command implementations (~3,300 LOC) move from `crates/aps-cli/src/main.rs` into a new `cli` module of the code-topology crate, exposed through a `TopologyCommandHandler` implementing `apss_core::registry::CommandHandler`. The fitness dispatch (~143 LOC) moves identically into the fitness-functions crate. `register()` in both crates populates real command lists. aps-cli deletes the moved code and routes `run topology ...` through the standard's own handler.

**Authority:** ADR-0002, issues #68/#69, scope map in this plan's References section.

**Branch:** `feat/adr-0002-consumer-run` (continues from Phases A and B).

## Design Decisions (locked)

1. **Module layout in code-topology:** `src/cli/mod.rs` (handler + dispatch match), `src/cli/analyze.rs` (topology_analyze + write_topology_artifacts + chrono_lite_now), `src/cli/validate.rs`, `src/cli/diff.rs` (diff + check + comment), `src/cli/report.rs`, `src/cli/viz.rs` (topology_viz + generate_vsa_placeholder), `src/cli/health.rs` (calculate_health, health_to_color, health_label, detect_layer, get_slice_from_id), `src/cli/vsa_config.rs` (moved verbatim from `crates/aps-cli/src/vsa_config.rs`; it parses the standard's own vsa.yaml and belongs to the standard).
2. **repo_root impedance:** `CommandHandler::execute` receives no repo root. The handler resolves `repo_root = std::env::current_dir()`. The dev CLI already runs commands relative to the invocation directory, so behavior is preserved.
3. **verbose impedance:** verbose becomes env-driven: handler reads `APSS_VERBOSE=1`. aps-cli sets that env var when `--verbose` is passed before delegating. No signature changes.
4. **ExitCode vs i32:** moved functions change return type from `std::process::ExitCode` to `i32` (0 success, 1 error, 3 usage). aps-cli converts at its boundary with `ExitCode::from(code as u8)`.
5. **Feature gating:** the `cli` module is unconditional; the `viz` command body is gated: `#[cfg(feature = "viz-dashboard")]` for dashboard/clusters/codecity/index generation, `#[cfg(feature = "viz-3d")]` for 3d. With a feature off, `viz` returns exit 5 with a message naming the missing feature.
6. **serde_json/walkdir deps:** the cli module needs `serde_json` (already a parent dep) and `walkdir` (currently optional behind lang-rust); make `walkdir` non-optional since analyze uses it regardless of language, OR keep analyze's walkdir usage inside the existing adapter paths; decide by compiler error, prefer non-optional.
7. **aps-cli dedup:** `dispatch_topology` body becomes: build the handler via `code_topology::cli::TopologyCommandHandler::new()`, set APSS_VERBOSE if needed, call `execute(command, args, &toml::Value::Table(Default::default()))`, convert exit code. Same for fitness. The 3,300 moved lines are deleted from main.rs.
8. **register():** topology `commands` = ["analyze", "validate", "diff", "check", "comment", "report", "viz"]; handler `commands()` returns matching `CommandInfo` entries with the usage strings from the old help text. fitness `commands` = ["validate"]. The old `--help` text becomes the handler's default branch for unknown/help commands.
9. **codegen untouched:** `apss_core::distribution::codegen::generate_main_rs` already emits `{ident}::register(&mut runner)`; no change needed in Phase C.

## References (from the Phase C scope exploration, line numbers pre-move)

- `dispatch_topology` main.rs:1018-1164; `dispatch_fitness` main.rs:1167-1309 (all logic inline)
- helpers: topology_analyze 1312-1504, write_topology_artifacts 1507-2179, chrono_lite_now 2560-2576, topology_validate 2180-2216, topology_diff 2219-2559, topology_check 2577-2710, topology_comment 2712-2833, topology_report 2835-3145, topology_viz 3147-3580, calculate_health 2888-2945, health_to_color 2948-2957, health_label 2960-2969, detect_layer 2972-3071, generate_vsa_placeholder 3074-3109, get_slice_from_id 3113-3145
- stubs: code-topology lib.rs:974-999, fitness lib.rs:665-690
- trait: apss-core registry.rs:74-101

### Task 1: Move topology CLI into the code-topology crate

**Files:**
- Create: `standards/v1/APS-V1-0001-code-topology/src/cli/` (8 files per Design Decision 1)
- Modify: `standards/v1/APS-V1-0001-code-topology/src/lib.rs` (add `pub mod cli;`, replace the stub `register()` and delete `NoopCommandHandler`)
- Modify: `standards/v1/APS-V1-0001-code-topology/Cargo.toml` (walkdir handling per Decision 6)
- Modify: `crates/aps-cli/src/main.rs` (delete moved functions, rewire `"topology" =>` arm per Decision 7, delete `mod vsa_config;`)
- Delete: `crates/aps-cli/src/vsa_config.rs` (moved)

- [ ] Step 1: Move the functions listed in References into the cli module files per Decision 1, applying Decisions 2-5 (i32 returns, env verbose, cwd repo root, feature gates in viz). Path rewrites: `code_topology::X` becomes `crate::X`; calls to viz substandards stay `crate::substandards::...`; `vsa_config::` becomes `super::vsa_config::` or `crate::cli::vsa_config::`.
- [ ] Step 2: Implement `TopologyCommandHandler` in `src/cli/mod.rs`: unit struct, `execute()` is the old dispatch match minus the help arm (help prints from a `fn print_help()` reused for unknown commands, returns 0 for explicit help, 3 for unknown), `commands()` returns the 7 CommandInfo entries.
- [ ] Step 3: Replace the `register()` stub in lib.rs: commands list per Decision 8, `Box::new(cli::TopologyCommandHandler)`. Delete `NoopCommandHandler`.
- [ ] Step 4: Rewire aps-cli per Decision 7 and delete the moved code and `vsa_config.rs`.
- [ ] Step 5: `cargo check --workspace && cargo test --workspace`. Expected: PASS.
- [ ] Step 6: Behavior parity check in the tester repo `/Users/neural/Code/AgentParadise/apss-example-repo`: `target/release/apss-dev run topology analyze .` (rebuild release first) produces the same artifact set as before; `run topology viz .topology --type all` produces the five HTML files.
- [ ] Step 7: `cargo run -p aps-cli --bin apss-dev -- v1 validate distribution`: expect exactly ONE remaining error (fitness-functions). Topology is green.
- [ ] Step 8: Commit: `refactor(topology)!: move CLI dispatch into the standard crate as TopologyCommandHandler (#68)`

### Task 2: Move fitness CLI into the fitness-functions crate

**Files:**
- Modify: `standards-experimental/v1/EXP-V1-0003-fitness-functions/src/lib.rs` (new `cli` module inline or `src/cli.rs`; move the dispatch_fitness body; real register())
- Modify: `crates/aps-cli/src/main.rs` (delete dispatch_fitness, rewire `"fitness" =>` arm)

- [ ] Step 1: Move dispatch_fitness logic into `FitnessCommandHandler::execute` (command "validate"); same Decisions 2-4 apply.
- [ ] Step 2: register(): commands = ["validate"], real handler, delete the Noop stub.
- [ ] Step 3: Rewire aps-cli, delete moved code.
- [ ] Step 4: `cargo check --workspace && cargo test --workspace`. Expected: PASS.
- [ ] Step 5: GREEN GATE: `cargo run -p aps-cli --bin apss-dev -- v1 validate distribution` passes with 0 errors. This closes #69's acceptance criteria and #68.
- [ ] Step 6: Commit: `feat(fitness): real FitnessCommandHandler; distribution gate green (#68, #69)`

### Task 3: Consumer e2e proof in the tester repo

- [ ] Step 1: Rebuild bundle: `cargo run -p aps-cli --bin apss-dev -- v1 bundle APS-V1-0001 --output /tmp/apss-phasec-bundle`
- [ ] Step 2: In `/Users/neural/Code/AgentParadise/apss-example-repo`: `apss install --bundle-dir /tmp/apss-phasec-bundle/APS-V1-0001-code-topology-0.2.0.apss` (the published crates.io `apss` 1.0.0 binary on PATH; if its install path rejects anything new, note it for Phase D rather than patching the published binary)
- [ ] Step 3: THE MOMENT: `.apss/bin/apss run code-topology analyze .` then `.apss/bin/apss run code-topology viz .topology --type all`. Expected: artifacts and five HTML files produced by the CONSUMER binary, no apss-dev involved.
- [ ] Step 4: `.apss/bin/apss list` shows code-topology with its 7 commands (if list prints commands).
- [ ] Step 5: Run full `just qa`: every step green including distribution.
- [ ] Step 6: Version bumps per repo rules: code-topology changed substantially again within the same unreleased 0.2.0; keep 0.2.0 (not yet published). fitness-functions experiment version: bump minor in experiment.toml and Cargo.toml. aps-cli/apss-core untouched pins stay.
- [ ] Step 7: Commit: `test(e2e): consumer binary runs topology analyze and viz from bundle install`

## Self-Review Notes

- The biggest mechanical risk is Step 1 of Task 1 (3,300 LOC of moves with path rewrites); the per-step workspace check catches breakage immediately, and behavior parity is verified against the tester repo before deleting anything is considered final.
- `write_topology_artifacts` uses chrono_lite_now, not chrono; no new deps for analyze.
- aps-cli keeps its own `--verbose` flag surface; only the transport changes (env var).
- If `cargo test` in aps-cli referenced moved private helpers, those tests move to the code-topology crate with the code.
