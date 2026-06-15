# APSS Bundle Format

## Purpose

An APSS bundle is the OFFLINE and catalog format for a standard or
substandard. The required distribution transport for official standards is
crates.io (see ADR-0002 and DI01 spec Section 2.1); bundles are an optional
mechanism for development, offline, and air-gapped installation through the
`apss install --bundle-dir <path>` path, and a catalog format that records
which standards, versions, and features travel together.

Tooling package managers install APSS tools. crates.io delivers standards;
bundles deliver the same standards in the offline and catalog case.

## Bundle Directory

The source form of a bundle is a directory named:

```text
<id>-<slug>-<version>.apss/
```

Example:

```text
APS-V1-0001-code-topology-1.0.0.apss/
```

A registry MAY store this directory as an archive, but the archive transport
format is separate from the bundle content format.

## Required Files

Every bundle MUST include:

- `bundle.toml`
- The package metadata file named by `bundle.toml.metadata_file`
- All source files needed to build or execute the standard implementation
- All standard documentation that defines install, validation, and usage
  behavior

Bundles MUST NOT include generated build output such as `target/`,
`.apss/build/`, or project-specific lockfiles.

## Managed Files

Files generated from an APSS bundle into a consumer repository MUST include a
clear managed-file notice when they are validator, hook, scaffold, generated
standard, or generated runtime files.

The notice MUST say:

```text
Managed by APSS. Do not edit this generated file directly.
```

Operator-owned configuration files, including `apss.yaml`, MUST NOT use that
notice because they are expected to be manually edited.

## Manifest

`bundle.toml` is the bundle manifest.

```toml
schema = "apss.bundle/v1"
id = "APS-V1-0001"
name = "Code Topology"
slug = "code-topology"
version = "1.0.0"
kind = "standard"
metadata_file = "standard.toml"

[source]
package_path = "standards/v1/APS-V1-0001-code-topology"
repository = "https://github.com/AgentParadise/agent-paradise-standards-system"

[payload]
metadata = "standard.toml"
docs = "docs"
implementation = "."
```

### Fields

- `schema` MUST be `apss.bundle/v1`.
- `id` MUST match the package metadata ID.
- `slug` MUST match the package metadata slug.
- `version` MUST match the package metadata version.
- `kind` MUST be `standard`, `substandard`, or `experiment`.
- `metadata_file` MUST name the included metadata file.
- `source.package_path` SHOULD record the source repository path used to
  produce the bundle.
- `source.repository` SHOULD record the source repository URL when known.
- `payload.metadata` MUST point to the included metadata file.
- `payload.docs` SHOULD point to included documentation when present.
- `payload.implementation` SHOULD point to the implementation payload root.

## Local Installation

For development, offline, and air-gapped installation,
`apss install --bundle-dir <path>` consumes a bundle directory directly. This
path MUST behave like installing the same standard from the crates.io
transport, except that checksum and registry resolution may be skipped because
the source is local.

## Registry Installation

Registry installation is the default path and resolves against crates.io
(ADR-0002). It MUST resolve:

- The selected standard crate version
- A content checksum
- A registry source descriptor (`registry+https://crates.io`)
- The implementation crate source, with the selected substandard features

Unresolved registry lockfiles MUST be refused in release-ready locked
installs. The offline `--bundle-dir` and `--local-repo` paths supply a local
source when crates.io resolution is not desired or not available.
