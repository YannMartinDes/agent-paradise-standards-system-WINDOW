# apss-core

Core engine for the Agent Paradise Standards System (APSS): diagnostics, discovery, metadata, config, lockfiles, registry, and standard resolution.

This crate is the shared library underneath the [`apss`](https://crates.io/crates/apss) CLI and APSS standard implementation crates. If you want to adopt APSS standards in a project, install the CLI instead:

```bash
cargo install apss
```

The `apss-core` public API is a stability contract: minor and patch releases must not break previously published standards.

## Documentation

Full specifications and the standards catalog live in the
[agent-paradise-standards-system repository](https://github.com/AgentParadise/agent-paradise-standards-system).

## License

MIT
