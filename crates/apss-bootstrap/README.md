# apss

Bootstrap CLI for the Agent Paradise Standards System (APSS): executable, versioned engineering standards for agentic codebases.

APSS delivers through two channels:

- **crates.io delivers the tooling**: this `apss` CLI, built on `apss-core`.
- **APSS bundles deliver the standards**: standards are distributed as versioned bundles, never as crates.io crates.

## Install

```bash
cargo install apss
```

## Usage

```bash
apss init        # generate APSS.yaml, the user-owned project manifest
apss install     # resolve standards, write apss.lock, install git hooks
apss validate    # validate the project (also runs from the pre-commit hook)
apss status      # show project configuration and status
apss run <standard> <command>   # run a standard's command
```

Commit `APSS.yaml` and `apss.lock`. The generated `.apss/` runtime is build output and stays out of git. Contributors to your repo can read, edit, and commit without installing this CLI.

## Documentation

Full specifications, standards catalog, and the distribution lifecycle live in the
[agent-paradise-standards-system repository](https://github.com/AgentParadise/agent-paradise-standards-system).

## License

MIT
