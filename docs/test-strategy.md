# Test strategy

## Layer A: fast local checks

- Rust unit tests for command planning and runner behavior.
- CLI tool-preflight checks for required binaries before non-dry-run execution.
- Node app and library lint/test/build scripts.
- Java app and library unit tests.
- Manifest rendering and file existence checks routed through `company-ci verify`.

## Layer B: local workflow smoke

- `act` against `verify.yml` and `emulated-e2e.yml`.
- Thin workflows mean the local and hosted paths exercise the same Rust entry points.

## Layer C: emulated platform

- kind plus a local registry, using Docker by default and Podman when `COMPANY_CI_CONTAINER_ENGINE=podman`.
- Nexus via Docker Compose, with readiness checks and repo validation for the npm and Maven repositories the CLI expects.
- `company-ci e2e emulated` owns startup, orchestration, explicit local package publication, image push, deploy, live service checks, and teardown sequencing.

## Layer D: higher-fidelity OpenShift local

- Assumes an existing OpenShift Local environment.
- Uses OpenShift overlays and `oc` to verify rollout.

## Layer E: GitHub.com integration

- Hosted workflows run the same top-level commands.
- Manual sandbox deployment exists for higher-fidelity external integration.
