# Runbook: Adopt APSS Standards in a Repo and Enforce Them

## Purpose

Set up the Agent Paradise Standards System (APSS) in a fresh repository, declare one or more official standards, and wire them into local hooks and CI so they are enforced on every change. This runbook is written to be handed to a coding agent (Claude, Codex, Gemini) or a developer verbatim. Every step has a command and an expected result.

It ends with a section on integrating APSS into the agentic harness template (lefthook + just + CI), which is the intended target.

## The official standards

| Canonical slug (APSS.yaml key) | ID | Crate | What it enforces |
|---|---|---|---|
| `code-topology` | APS-V1-0001 | `apss-v1-0001-code-topology` | Architectural metrics, coupling, module structure; produces `.topology/` and visualizations |
| `architecture-fitness` | APS-V1-0002 | `apss-v1-0002-architecture-fitness` | Declarative fitness rules (threshold, dependency, structural) over topology artifacts |
| `documentation` | APS-V1-0003 | `apss-v1-0003-documentation` | ADR enforcement, README index validation, agent context files |

Use the canonical slug in `APSS.yaml` and in `apss run <slug>`. The composed project binary dispatches by the exact key in `APSS.yaml`, so `code-topology` works but `topology` does not (short aliases like `topology`, `fitness`, `docs` are accepted only by the development CLI `apss-dev`).

## Prerequisites

- Rust toolchain and Cargo (`cargo --version` succeeds). The standards build locally on install.
- Network access to crates.io.
- `apss` 1.1.0 or newer (1.1.0 is the first release that resolves standards from crates.io).

## 1. Install the CLI

```bash
cargo install apss
apss --version          # expect 1.1.0 or newer; `cargo install apss --force` to upgrade
```

## 2. Declare the standards

`apss init` creates `APSS.yaml`. You can scaffold one standard with `--standard <slug>` (it leaves the id as a `APS-V1-XXXX` placeholder to fill in), or write `APSS.yaml` directly. To adopt all three:

```yaml
# APSS.yaml
schema: apss.project/v1

project:
  name: my-repo
  apss_version: v1

standards:
  code-topology:        { id: APS-V1-0001, version: ">=0.2.0" }
  architecture-fitness: { id: APS-V1-0002, version: ">=1.0.0" }
  documentation:        { id: APS-V1-0003, version: ">=0.1.0" }

tool:
  offline: false
  hooks:
    pre_commit: true
```

Adopt only what you need: a repo that just wants visualizations can list `code-topology` alone.

## 3. Install (resolve and build the project binary)

```bash
apss install
```

Expected:

- Each declared standard is resolved from crates.io and pinned (exact version) in `apss.lock`.
- A project-local composed binary is built at `.apss/bin/apss`.
- A managed pre-commit hook is installed at `.git/hooks/pre-commit` (see step 6).

Commit `APSS.yaml` and `apss.lock`. Do not commit `.apss/` (generated build output; it is gitignored by the managed install). Contributors who clone the repo do not need to install the global CLI.

## 4. Run each standard once

```bash
apss run code-topology analyze .                     # writes .topology/ artifacts
apss run code-topology viz .topology --type all      # optional: opens the dashboard
apss run architecture-fitness validate .             # needs a fitness.toml (step 5)
apss run documentation validate .                    # checks docs/ADRs/README indexes
```

`code-topology analyze` must run before `architecture-fitness validate`, because fitness rules read `.topology/` artifacts.

## 5. Configure the standards you adopted

Each standard reads its own config from the repo root:

- **architecture-fitness** reads `fitness.toml` (rules, thresholds, dimensions). See `standards/v1/APS-V1-0002-architecture-fitness/examples/fitness.toml` for a starting point. Exceptions with required issue references go in `fitness-exceptions.toml`.
- **documentation** reads its docs configuration (ADR directory, naming pattern, required sections). The defaults work out of the box; see the documentation standard's `docs/02_install_contract.md` for the configurable options.
- **code-topology** needs no config; it analyzes source directly.

## 6. Two layers of enforcement

There are two distinct checks. Use both.

### Layer 1: project validation (auto-installed hook)

The managed pre-commit hook from `apss install` runs `apss validate`, which checks the project configuration and standard installation state (is `APSS.yaml` valid, is `apss.lock` consistent, is the composed binary present). It does NOT run the standards' own rules. Control it with `tool.hooks.pre_commit` in `APSS.yaml`.

### Layer 2: the standards' own rules (you wire this)

To actually enforce the standards' rules, run their commands in CI (and optionally add them to a hook). The commands return a nonzero exit code on failure, so they gate CI directly:

```bash
apss run code-topology analyze .                 # regenerate artifacts (exit 0 unless analysis fails)
apss run architecture-fitness validate .         # exit 1 if any fitness rule fails
apss run documentation validate .                # exit 1 if docs/ADR validation fails
```

Add `.topology/` to `.gitignore` and regenerate it in CI rather than committing it: the artifacts can get large (a medium Rust workspace produces a 400+ KB `functions.json`).

## 7. Minimal CI gate (GitHub Actions)

```yaml
# .github/workflows/standards.yml
name: APSS Standards
on: [pull_request]
jobs:
  standards:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo install apss
      - run: apss install --locked          # fail if apss.lock would change
      - run: apss run code-topology analyze .
      - run: apss run architecture-fitness validate .
      - run: apss run documentation validate .
```

## Appendix: Integrating into the agentic harness template

The harness template manages hooks with lefthook and tasks with just. Wire APSS into both, plus CI.

1. **Add `APSS.yaml`** (step 2) at the template root and commit it with `apss.lock` after a one-time `apss install`.

2. **A `just` recipe** as the single task surface (the template prefers `just <recipe>` over direct tool calls):

```make
# justfile
apss-check:
    apss run code-topology analyze .
    apss run architecture-fitness validate .
    apss run documentation validate .
```

3. **lefthook** entries so the standards run on commit and push:

```yaml
# lefthook.yml
pre-commit:
  commands:
    apss-validate:
      run: apss validate            # fast: project config + structure
pre-push:
  commands:
    apss-standards:
      run: just apss-check          # full: the standards' own rules
```

Put the fast project-validation check on `pre-commit` and the heavier standard rules on `pre-push` (or in CI) so the commit loop stays quick.

4. **CI**: add the step from section 7, or call `just apss-check` from the template's existing pipeline.

5. **Slots**: if the template tracks tools as manifest slots, register APSS as an external slot whose entrypoint is `apss` and whose commands are `install`, `validate`, and `run`, alongside the existing `gitleaks` and `just` external tools.

Contributors to the harness fork get enforcement for free: `apss.lock` pins exact standard versions, the hooks run on their machines, and CI is the backstop. They never need the global CLI to read or edit the repo.

## Troubleshooting

| Symptom | Cause and fix |
|---|---|
| `Standard '<x>' not found in APSS.yaml` | Use the canonical slug as the `APSS.yaml` key and in `apss run` (`code-topology`, `architecture-fitness`, `documentation`). |
| `apss install` cannot reach crates.io | Offline or proxied network. Use `apss install --bundle-dir <path>` with a locally built bundle. |
| `fitness.toml not found` | architecture-fitness needs a `fitness.toml` at the repo root (step 5). |
| fitness rules fail with missing artifacts | Run `apss run code-topology analyze .` first; fitness reads `.topology/`. |
| `apss` is too old to resolve from crates.io | `cargo install apss --force` (need 1.1.0 or newer). |

## Related

- [Visualize Your Codebase runbook](visualize-your-codebase.runbook.md): the topology-focused walkthrough
- Root `README.md`, section "Using APSS in Your Project"
- [ADR-0002: crates.io distribution](../../standards/v1/APS-V1-0000-meta/docs/adrs/0002-crates-io-distribution.md)
- [DI01 package manager lifecycle](../../standards/v1/APS-V1-0000-meta/substandards/DI01-distribution/docs/03_package_manager_lifecycle.md)
