# North Star (Mission, Vision, Position)

**ID:** `APS-V1-0003.PV01`
**Type:** Substandard
**Slug:** `north-star`
**Version:** `0.1.0`
**Parent:** `APS-V1-0003` (Documentation and Context Engineering)

Validates the presence and structure of the project's single North Star
document that agents read during plan and design to stay aligned with the
project's intent. Implemented as a feature-module of the parent crate behind
the `PV01` cargo feature (ADR-0002).

## Index

- [substandard.toml](substandard.toml)
- [Overview](docs/00_overview.md)
- [Specification](docs/01_spec.md)

## Implementation

The validator and scaffold logic live in the parent crate at
`src/substandards/purpose_and_vision.rs` behind the `PV01` feature.

## Validation

```bash
cargo run -p aps-cli --bin apss-dev -- v1 validate substandard APS-V1-0003.PV01
```

Run the full repository validation with:

```bash
cargo run -p aps-cli --bin apss-dev -- v1 validate repo
```
