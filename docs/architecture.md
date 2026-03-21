# Architecture

## Thin workflow model

GitHub Actions only performs checkout, language/tool setup, cache preparation, and a top-level `company-ci` invocation. Real orchestration lives in Rust so that the same entry points work locally, under `act`, and on GitHub.com.

## CLI design

`tools/company-ci` separates pure planning from command execution:

- `plan.rs` builds ordered command plans for each subcommand.
- Each plan declares the external tools it requires so the CLI can fail fast on missing `mvn`, `node`, `podman`, `oc`, and similar binaries.
- `runner.rs` abstracts side effects so unit tests can verify planning without shelling out, and performs tool preflight before non-dry-run execution.
- `commands.rs` maps CLI inputs to plans.

## Monorepo conventions

- Apps live under `apps/`.
- Libraries live under `libs/`.
- Deployment manifests live under `deploy/`.
- Local environment assets live under `testbeds/`.

## Deferred items

- Production-grade release promotion is intentionally deferred.
- OpenShift-specific routes and image streams are kept minimal for the first scaffold.
- Real artifact repository integration is environment-driven and documented, not fully automated here.
