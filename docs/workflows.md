# Workflows

## verify.yml

Always-on workflow for `pull_request`, pushes to `main`, and `workflow_dispatch`. It builds `company-ci` from source and runs `company-ci verify`.

## emulated-e2e.yml

Manual or scheduled workflow that installs the base toolchain and runs `company-ci e2e emulated`.

## sandbox-deploy.yml

Manual deployment entry point for an external sandbox. The workflow remains thin and passes deployment intent to `company-ci`.

## release-company-ci.yml

Builds release artifacts for the CLI on tags or manual invocation. This prepares the repo for a future installer story without introducing reusable-workflow complexity now.
