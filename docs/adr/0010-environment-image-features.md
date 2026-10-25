# ADR-0010: The Dev Container is a base image with Features and a setup script, and the work needs no containers

- **Status:** 🟡 proposed
- **Date:** 2026-10-25
- **Deciders:** Markus Brigl
- **Applies to:** `.devcontainer/`, and every change that would have the work start or reach a container

## Context

Nectar is built, tested and run with Cargo, and its tests need no database and start no
containers; Graphviz and PlantUML are installed for the process adapters of the command line. The
Dev Container ships as `mcr.microsoft.com/devcontainers/base:debian` with Features for Java and
Rust and `.devcontainer/setup.sh` for the system packages, and
[ADR-0004](0004-dev-container-runtime.md) keeps the host engine out of its reach.

## Decision

We will keep the Dev Container as a base image with Features and `.devcontainer/setup.sh`, without
a `Dockerfile` or Compose, and give the work no access to any container engine.

**Out of scope:** which packages the setup script installs; the Dev Container Feature that would
install `nectar` for other projects, which the specification names as a possible distribution.

## Alternatives considered

- **A `Dockerfile`** — everything in one place, but one more file to maintain for what the image,
  the Features and the script already give.
- **An engine inside the container or a remote one** — the catalogue's *Container access*; not
  needed while no test starts a container.

## Sources / Prior art

- Dev Container specification — <https://containers.dev/implementors/json_reference/>.

## Consequences

- Positive: nothing to maintain beyond what ships; no container engine for an agent to reach.
- Negative / trade-offs: a test that needs a container cannot be written until this is superseded.
- Follow-ups: Java stays only as the runtime of PlantUML once the Gradle plugin is gone.

## Enforcement

**Not mechanically decidable:** that the work needs no container is a statement about the tests to
come, not about a file. What holds mechanically is already checked:
[`scripts/check-devcontainer.sh`](../../scripts/check-devcontainer.sh) fails on a mounted host
socket and an unlocked Feature ([ADR-0004](0004-dev-container-runtime.md)).
