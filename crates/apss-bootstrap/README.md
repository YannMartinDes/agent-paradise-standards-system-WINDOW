# apss

Bootstrap CLI for the Agent Paradise Standards System (APSS): executable, versioned engineering standards for agentic codebases.

APSS uses crates.io as its distribution transport:

- **crates.io delivers the tooling**: this `apss` CLI, built on `apss-core`.
- **crates.io delivers the standards**: each official standard publishes as one crate, and its substandards ship as cargo features of that crate. `apss install` resolves standards from crates.io, no APSS checkout required. Bundles remain available as an optional offline install (`apss install --bundle-dir <path>`).

## Install

```bash
cargo install apss
```

## Usage

```bash
apss init        # generate APSS.yaml, the user-owned project manifest
apss add <standard>   # add a standard to APSS.yaml
apss install     # resolve standards from crates.io, write apss.lock, install git hooks
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
