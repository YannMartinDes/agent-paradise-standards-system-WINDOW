# ADR 0002: crates.io as Standard Distribution Transport

**Status:** Accepted
**Date:** 2026-06-05
**Context:** APS-V1-0000 Meta Standard, DI01 Distribution, SS01 Substandard Structure
**Supersedes:** DI01 spec section 9.4 publish scope ("standards are never published to crates.io"); bundle-as-transport model in DI01 sections 9.3 and the bundle format doc

## Context

DI01 originally specified a two-channel model: crates.io delivers the APSS tooling (`apss-core`, `apss`), and APSS bundles deliver standards as self-contained source archives. The bundle registry was never built; the interim path required consumers to clone the APSS repository, build a bundle to a temp directory, and pass `--bundle-dir` to `apss install`.

The first end-to-end deployment test (2026-06-05, issues #65, #68) demonstrated that this path works mechanically but is not turnkey: a consumer must know about the APSS repo checkout, the dev CLI, and bundle staging. The intended consumer experience is five commands with no local APSS checkout:

```bash
cargo install apss
apss init
apss install
apss run code-topology analyze .
apss run code-topology viz
```

Options evaluated:

- **A. Vendor bundles into the published `apss` crate.** Turnkey and offline, but freezes standard versions to tooling releases and keeps the bespoke bundle machinery alive.
- **B. GitHub Releases as bundle registry.** Real registry shape, but requires building and operating release/download infrastructure before any consumer succeeds.
- **C. Publish standard crates to crates.io.** Cargo becomes the registry: resolution, semver ranges, checksums, immutability, yanking, and CDN are all inherited. `apss install` generates a Cargo project with registry dependencies and builds it.

## Decision

Adopt **Option C with per-standard granularity**:

1. **crates.io is the distribution transport for standards.** Each official standard publishes as one crate (for example `apss-v1-0001-code-topology`). Experiments publish under their experiment name; on promotion, the new official crate is published fresh and the experiment crate remains on crates.io unmaintained (consistent with the existing no-backward-compat promotion decision).
2. **Substandards are not separate published crates.** Substandard code lives inside the parent standard crate as feature-gated modules (for example features `lang-rust`, `viz-dashboard`, `viz-3d`). Substandards remain first-class governed units: own `substandard.toml`, docs, version, and validation. Their isolation is enforced at module level by meta-standard validators rather than by crate boundaries.
3. **Consumer-internal standards keep internal crate layout.** The meta-standard and its substandards (CF01, DI01, CL01, SS01) are not published and may retain separate workspace crates. This carve-out may be revisited.
4. **The bundle concept is demoted from transport to catalog.** A bundle becomes a manifest describing which standards, versions, features, and alias mappings travel together. `apss.lock` records requested versus resolved IDs as before; Cargo.lock provides source-level pinning underneath.
5. **`apss install`** resolves the standards declared in the project config to crates.io dependencies with the selected substandard features, generates the composed binary project, and builds it. `--bundle-dir` and `--local-repo` remain as development and pre-publish testing paths.
6. **Release automation** publishes changed standard crates in dependency order after `apss-core`, restoring tiered publishing in the release workflow with the publish set derived from this ADR.

## Consequences

- DI01 sections 9.3 and 9.4 and the bundle format doc must be amended; DI01 version bump required.
- SS01 must be amended: substandard structure no longer mandates a per-substandard `Cargo.toml` for published standards; module layout and feature naming rules replace it. SS01 version bump required.
- Source refactor: code-topology's five substandard crates merge into the parent crate as feature-gated modules. Same pattern applies to future standards.
- Standard crates need publish metadata (description, readme, keywords, categories) and the validation suite must require it (poka-yoke, see #69).
- The release workflow regains standard-crate publishing tiers. The validation that enforces publish scope must enforce THIS scope, not the superseded section 9.4 scope.
- All published standards are public. Private standard distribution, if ever needed, requires an alternate registry and is out of scope for this ADR.
- Crate count stays bounded: tooling (2) plus one crate per official standard plus experiment crates.

## Non-Goals

- This ADR does not address composed-runtime command registration (issue #68); that gap exists under every option and is tracked separately.
- No commitment to a custom registry; crates.io suffices until a private-distribution requirement materializes.
