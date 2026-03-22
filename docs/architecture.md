# Architecture

## Thin workflow model

GitHub Actions only performs checkout, language/tool setup, cache preparation, installation of the `company-ci` binary, and a top-level `company-ci` invocation. Real orchestration lives in Rust so that the same entry points work locally, under `act`, and on GitHub.com.

This is intentional because workflow YAML is not treated as the long-term CI DSL. The maintainable API is the Rust command surface.

## CLI design

`tools/company-ci` separates pure planning from command execution:

- `plan.rs` builds ordered command plans for each subcommand.
- The command palette is the public CI API for both local shells and GitHub workflows.
- Each plan declares the external tools it requires so the CLI can fail fast on missing `mvn`, `node`, `docker` or `podman`, `oc`, and similar binaries.
- `runner.rs` abstracts side effects so unit tests can verify planning without shelling out, and performs tool preflight before non-dry-run execution.
- `commands.rs` maps CLI inputs to plans.
- `container_engine.rs` centralizes `COMPANY_CI_CONTAINER_ENGINE` parsing so plans and helper scripts can target either Docker or Podman with the same top-level commands.

## Monorepo conventions

- Apps live under `apps/`.
- Libraries live under `libs/`.
- Deployment manifests live under `deploy/`.
- Local environment assets live under `testbeds/`.

## Deferred items

- Production-grade release promotion is intentionally deferred.
- OpenShift-specific routes and image streams are kept minimal for the first scaffold.
- Real artifact repository integration is environment-driven and documented, not fully automated here.
