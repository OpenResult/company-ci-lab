# Workflows

## verify.yml

Always-on workflow for `pull_request`, pushes to `main`, and `workflow_dispatch`. It installs the language toolchains with setup actions, uses the local `setup-company-ci` action to place the `company-ci` binary on `PATH`, and runs `company-ci verify`. The workflow now uses the current `actions/checkout`, `actions/setup-node`, and `actions/setup-java` majors that are aligned with the Node 24 transition for JavaScript-based actions.

The hosted workflows are expected to install toolchains and platform CLIs up front; `company-ci` still verifies that `docker` or `podman`, `oc`, `node`, `npm`, `java`, and the repo-local `./mvnw` wrapper exist before executing the plans that need them.

That split keeps the workflow readable while the typed Rust CLI owns the actual CI logic.

## sandbox-deploy.yml

Manual deployment entry point for an external sandbox. The workflow remains thin, installs the selected platform CLI, and passes deployment intent to `company-ci`.

The reusable OpenShift deploy contract now lives in the CLI. Future external workflows should install `oc`, materialize registry credentials, and export:

- `COMPANY_CI_OPENSHIFT_API_URL`
- `COMPANY_CI_OPENSHIFT_TOKEN`
- `COMPANY_CI_OPENSHIFT_SKIP_TLS_VERIFY`

- `COMPANY_CI_IMAGE_PUSH_REGISTRY`
- `COMPANY_CI_IMAGE_PULL_REGISTRY`
- `COMPANY_CI_IMAGE_NAMESPACE`
- `COMPANY_CI_IMAGE_TAG`
- optional `COMPANY_CI_IMAGE_PLATFORM`
- `COMPANY_CI_IMAGE_REGISTRY_USERNAME`
- `COMPANY_CI_IMAGE_REGISTRY_PASSWORD` or `COMPANY_CI_IMAGE_REGISTRY_PASSWORD_FILE`

Then they can call `company-ci deploy openshift` without adding `oc login`, repository-specific, or Artifactory-specific branching in workflow YAML.

With the local action in place, the workflow-facing interface is now the `company-ci` binary itself rather than `cargo run`.

Future hosted publish workflows should stay thin as well: materialize tool-native auth files such as Maven `settings.xml`, export `MAVEN_SETTINGS_PATH`/`MAVEN_DEPLOY_URL`/`MAVEN_SERVER_ID`, and call an explicit contract-based command such as `company-ci publish maven-lib libs/java-lib`. The OpenShift e2e path remains a developer-oriented path; do not try to run it on GitHub-hosted runners unless the target cluster and repository are explicitly provisioned.
