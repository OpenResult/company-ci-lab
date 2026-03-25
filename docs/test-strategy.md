# Test strategy

## Layer A: fast local checks

- Rust unit tests for command planning and runner behavior.
- CLI tool-preflight checks for required binaries before non-dry-run execution.
- Node app and library lint/test/build scripts.
- Java app and library unit tests.
- Manifest rendering and file existence checks routed through `company-ci verify`.

## Layer B: local workflow smoke

- `act` against `verify.yml`.
- Thin workflows mean the local and hosted paths exercise the same Rust entry points and the same `company-ci` command palette.

## Layer C: local repository

- Docker Compose can stand up the local repository service used for npm, Maven, and Docker hosted repositories.
- The repository bootstrap helper validates readiness and captures runtime credentials for later publish and image flows.

## Layer D: OpenShift integration

- Assumes an existing OpenShift environment plus the OpenShift auth env contract.
- Uses the same CLI registry contract with a local repository-backed image repo, can pin image builds to a target platform such as `linux/amd64`, applies OpenShift overlays with Routes and pull-secret wiring, and verifies live HTTP responses through `oc` plus route checks.

## Layer E: GitHub.com integration

- Hosted workflows run the same top-level commands.
- Manual sandbox deployment exists for higher-fidelity external integration.
