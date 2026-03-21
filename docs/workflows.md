# Workflows

## verify.yml

Always-on workflow for `pull_request`, pushes to `main`, and `workflow_dispatch`. It builds `company-ci` from source, installs the language toolchains with setup actions, and runs `company-ci verify`. The workflow now uses the current `actions/checkout`, `actions/setup-node`, and `actions/setup-java` majors that are aligned with the Node 24 transition for JavaScript-based actions.

The hosted workflows are expected to install toolchains and platform CLIs up front; `company-ci` still verifies that `docker` or `podman`, `kind`, `kubectl`, `oc`, `node`, `npm`, `java`, and `mvn` exist before executing the plans that need them.

## emulated-e2e.yml

Manual or scheduled workflow that installs the base toolchain and runs `company-ci e2e emulated`. The command now owns env bootstrap, library publication, image push, deploy, and post-rollout health checks.

## sandbox-deploy.yml

Manual deployment entry point for an external sandbox. The workflow remains thin and passes deployment intent to `company-ci`; any required platform tooling is expected from the runner image or setup steps.

## release-company-ci.yml

Builds release artifacts for the CLI on tags or manual invocation. This prepares the repo for a future installer story without introducing reusable-workflow complexity now.
