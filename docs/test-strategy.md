# Test strategy

## Layer A: fast local checks

- Rust unit tests for command planning and runner behavior.
- Node app and library lint/test/build scripts.
- Java app and library unit tests.
- Manifest rendering and file existence checks routed through `company-ci verify`.

## Layer B: local workflow smoke

- `act` against `verify.yml` and `emulated-e2e.yml`.
- Thin workflows mean the local and hosted paths exercise the same Rust entry points.

## Layer C: emulated platform

- kind plus a local registry.
- Nexus via Docker Compose for npm/Maven/image publication experiments.
- `company-ci e2e emulated` owns startup, orchestration, health checks, and teardown sequencing.

## Layer D: higher-fidelity OpenShift local

- Assumes an existing OpenShift Local environment.
- Uses OpenShift overlays and `oc` to verify rollout.

## Layer E: GitHub.com integration

- Hosted workflows run the same top-level commands.
- Manual sandbox deployment exists for higher-fidelity external integration.
