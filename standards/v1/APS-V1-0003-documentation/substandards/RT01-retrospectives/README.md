# Retrospectives

**ID:** `APS-V1-0003.RT01`
**Type:** Substandard
**Slug:** `retrospectives`
**Version:** `0.1.0`
**Parent:** `APS-V1-0003` (Documentation and Context Engineering)

Validates the presence and structure of project retrospective documents.
Implemented as a feature-module of the parent crate behind the `RT01` cargo
feature (ADR-0002).

## Index

- [substandard.toml](substandard.toml)
- [Overview](docs/00_overview.md)
- [Specification](docs/01_spec.md)

## Implementation

The validator and scaffold logic live in the parent crate at
`src/substandards/retrospectives.rs` behind the `RT01` feature.

## Validation

```bash
cargo run -p aps-cli --bin apss-dev -- v1 validate substandard APS-V1-0003.RT01
```

Run the full repository validation with:

```bash
cargo run -p aps-cli --bin apss-dev -- v1 validate repo
```
