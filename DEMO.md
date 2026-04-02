# DEMO

`company-ci-lab` is easiest to present as a portability demo for thin CI. The main claim is simple: the workflow engine should not own the real delivery logic. The engine should own triggers, agents, tool setup, and credentials. The reusable CI API should live in code.

This demo uses the Spring Boot service in `apps/spring-api` as the Java workload and OpenShift as the deployment target.

The snippets below are illustrative examples that show the intended future `company-ci` contract across GitHub Actions, Jenkins, GitLab CI, and Buildkite.

## Demo thesis

If the deploy logic is hidden inside GitHub Actions YAML, Jenkins Groovy, or another pipeline DSL, portability is expensive because every engine migration becomes a reimplementation.

If the deploy logic lives behind a typed CLI, portability is mostly a translation problem:

- checkout the repo
- install a prebuilt `company-ci` binary and the required platform CLIs
- inject credentials
- run the same top-level `company-ci` commands

For this repo, the portable command sequence is:

```bash
company-ci verify
company-ci image build
company-ci image publish
company-ci deploy openshift
```

Those commands operate on the repo's application set, including the Java app in `apps/spring-api`.

## OpenShift deploy contract

Before running `company-ci deploy openshift`, the engine needs to do four things:

1. Install `company-ci`.
2. Make `oc` available on `PATH`.
3. Provide OpenShift authentication credentials through environment variables.
4. Provide the image contract through environment variables.

The reusable OpenShift auth contract can look like this:

```bash
COMPANY_CI_OPENSHIFT_API_URL
COMPANY_CI_OPENSHIFT_TOKEN
COMPANY_CI_OPENSHIFT_SKIP_TLS_VERIFY
```

The reusable image contract for OpenShift-based deploys is:

```bash
COMPANY_CI_IMAGE_PUSH_REGISTRY
COMPANY_CI_IMAGE_PULL_REGISTRY
COMPANY_CI_IMAGE_NAMESPACE
COMPANY_CI_IMAGE_TAG
COMPANY_CI_IMAGE_REGISTRY_USERNAME
COMPANY_CI_IMAGE_REGISTRY_PASSWORD
COMPANY_CI_IMAGE_REGISTRY_PASSWORD_FILE
```

At deploy time, `company-ci deploy openshift` is responsible for the repo-specific orchestration:

- verify that `oc` exists
- perform or verify the OpenShift login using the provided auth contract
- create or update the image pull secret
- apply the OpenShift overlay
- set the resolved image references
- wait for rollout completion
- verify the live routes

That is the portability boundary. The engine handles setup and secrets. `company-ci` handles delivery logic.

## Engine examples

### GitHub Actions

Assume `./.github/actions/setup-company-ci` installs a prebuilt `company-ci` release binary onto `PATH`, and the selected runner image already includes `oc`.

```yaml
name: demo-openshift-deploy

on:
  workflow_dispatch:

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v5
      - uses: actions/setup-node@v6
        with:
          node-version: '24'
      - uses: actions/setup-java@v5
        with:
          distribution: temurin
          java-version: '21'
      - uses: ./.github/actions/setup-company-ci
      - name: Deploy with company-ci
        env:
          COMPANY_CI_OPENSHIFT_API_URL: ${{ secrets.OPENSHIFT_API_URL }}
          COMPANY_CI_OPENSHIFT_TOKEN: ${{ secrets.OPENSHIFT_TOKEN }}
          COMPANY_CI_IMAGE_PUSH_REGISTRY: ${{ vars.IMAGE_PUSH_REGISTRY }}
          COMPANY_CI_IMAGE_PULL_REGISTRY: ${{ vars.IMAGE_PULL_REGISTRY }}
          COMPANY_CI_IMAGE_NAMESPACE: demo
          COMPANY_CI_IMAGE_TAG: ${{ github.sha }}
          COMPANY_CI_IMAGE_REGISTRY_USERNAME: ${{ secrets.REGISTRY_USERNAME }}
          COMPANY_CI_IMAGE_REGISTRY_PASSWORD: ${{ secrets.REGISTRY_PASSWORD }}
        run: |
          company-ci verify
          company-ci image build
          company-ci image publish
          company-ci deploy openshift
```

If you standardize on a prebuilt CI image such as `registry.example.com/company-ci/company-ci:1.0.0` that already contains `company-ci`, `oc`, the container client, and the language runtimes, the workflow can shrink further. In that model the image exposes tool-version selection through the CLI, so GitHub Actions only needs one checkout step plus the top-level `company-ci` calls.

```yaml
name: demo-openshift-deploy-container-image

on:
  workflow_dispatch:

jobs:
  deploy:
    runs-on: ubuntu-latest
    container:
      image: registry.example.com/company-ci/company-ci:1.0.0
    steps:
      - uses: actions/checkout@v5
      - name: Select runtime versions
        run: |
          company-ci java 21
          company-ci node 24
      - name: Deploy with company-ci
        env:
          COMPANY_CI_OPENSHIFT_API_URL: ${{ secrets.OPENSHIFT_API_URL }}
          COMPANY_CI_OPENSHIFT_TOKEN: ${{ secrets.OPENSHIFT_TOKEN }}
          COMPANY_CI_IMAGE_PUSH_REGISTRY: ${{ vars.IMAGE_PUSH_REGISTRY }}
          COMPANY_CI_IMAGE_PULL_REGISTRY: ${{ vars.IMAGE_PULL_REGISTRY }}
          COMPANY_CI_IMAGE_NAMESPACE: demo
          COMPANY_CI_IMAGE_TAG: ${{ github.sha }}
          COMPANY_CI_IMAGE_REGISTRY_USERNAME: ${{ secrets.REGISTRY_USERNAME }}
          COMPANY_CI_IMAGE_REGISTRY_PASSWORD: ${{ secrets.REGISTRY_PASSWORD }}
        run: |
          company-ci verify
          company-ci image build
          company-ci image publish
          company-ci deploy openshift
```

### Jenkins

```groovy
pipeline {
  agent any

  environment {
    COMPANY_CI_OPENSHIFT_API_URL = credentials('openshift-api-url')
    COMPANY_CI_OPENSHIFT_TOKEN = credentials('openshift-token')
    COMPANY_CI_IMAGE_PUSH_REGISTRY = 'registry.build.example.com'
    COMPANY_CI_IMAGE_PULL_REGISTRY = 'registry.apps.example.com'
    COMPANY_CI_IMAGE_NAMESPACE = 'demo'
    COMPANY_CI_IMAGE_TAG = "${env.BUILD_NUMBER}"
  }

  stages {
    stage('Checkout') {
      steps {
        checkout scm
      }
    }

    stage('Install company-ci') {
      steps {
        sh '''
          curl -fsSL https://downloads.example.com/company-ci/install.sh | sh
          export PATH="$HOME/.company-ci/bin:$PATH"
          company-ci --version
        '''
      }
    }

    stage('Deploy') {
      steps {
        withCredentials([
          usernamePassword(
            credentialsId: 'image-registry-creds',
            usernameVariable: 'COMPANY_CI_IMAGE_REGISTRY_USERNAME',
            passwordVariable: 'COMPANY_CI_IMAGE_REGISTRY_PASSWORD'
          )
        ]) {
          sh '''
            export PATH="$HOME/.company-ci/bin:$PATH"
            company-ci verify
            company-ci image build
            company-ci image publish
            company-ci deploy openshift
          '''
        }
      }
    }
  }
}
```

### GitLab CI

Assume `COMPANY_CI_OPENSHIFT_API_URL`, `COMPANY_CI_OPENSHIFT_TOKEN`, `COMPANY_CI_IMAGE_REGISTRY_USERNAME`, and `COMPANY_CI_IMAGE_REGISTRY_PASSWORD` are stored as GitLab CI/CD variables.

```yaml
stages:
  - deploy

deploy_openshift:
  stage: deploy
  tags:
    - linux
    - java21
    - node24
    - oc
  variables:
    COMPANY_CI_OPENSHIFT_SKIP_TLS_VERIFY: "false"
    COMPANY_CI_IMAGE_PUSH_REGISTRY: registry.build.example.com
    COMPANY_CI_IMAGE_PULL_REGISTRY: registry.apps.example.com
    COMPANY_CI_IMAGE_NAMESPACE: demo
    COMPANY_CI_IMAGE_TAG: $CI_COMMIT_SHORT_SHA
  before_script:
    - curl -fsSL https://downloads.example.com/company-ci/install.sh | sh
    - export PATH="$HOME/.company-ci/bin:$PATH"
  script:
    - company-ci verify
    - company-ci image build
    - company-ci image publish
    - company-ci deploy openshift
```

### Buildkite

Assume `COMPANY_CI_OPENSHIFT_API_URL`, `COMPANY_CI_OPENSHIFT_TOKEN`, `COMPANY_CI_IMAGE_REGISTRY_USERNAME`, and `COMPANY_CI_IMAGE_REGISTRY_PASSWORD` are injected by the agent environment or a secrets plugin.

```yaml
steps:
  - label: "Deploy Java app to OpenShift"
    agents:
      queue: "linux"
    env:
      COMPANY_CI_OPENSHIFT_SKIP_TLS_VERIFY: "false"
      COMPANY_CI_IMAGE_PUSH_REGISTRY: "registry.build.example.com"
      COMPANY_CI_IMAGE_PULL_REGISTRY: "registry.apps.example.com"
      COMPANY_CI_IMAGE_NAMESPACE: "demo"
      COMPANY_CI_IMAGE_TAG: "${BUILDKITE_COMMIT}"
    commands:
      - curl -fsSL https://downloads.example.com/company-ci/install.sh | sh
      - export PATH="$HOME/.company-ci/bin:$PATH"
      - company-ci verify
      - company-ci image build
      - company-ci image publish
      - company-ci deploy openshift
```

## Portability at a glance

| Concern | Owned by the engine | Owned by `company-ci` |
| --- | --- | --- |
| Triggers and approvals | Yes | No |
| Agent or runner selection | Yes | No |
| Installing prebuilt `company-ci`, Java, Node, `oc` | Yes | No |
| Secret injection | Yes | No |
| OpenShift authentication flow | No | Yes |
| Build, publish, deploy ordering | No | Yes |
| OpenShift overlay selection | No | Yes |
| Image reference resolution | No | Yes |
| Rollout and route verification | No | Yes |

That split is what makes the examples above look similar even though the pipeline syntax changes.

## Why this is maintainable

- The real orchestration lives in `tools/company-ci/src/plan.rs`, not in four different pipeline DSLs.
- CLI input mapping is centralized in `tools/company-ci/src/commands.rs`.
- Execution is separated from planning in `tools/company-ci/src/runner.rs`, which keeps the orchestration testable.
- Tool preflight is built into the CLI, so missing `oc`, `java`, `npm`, or `./mvnw` fail at the command boundary instead of producing partial pipeline behavior.
- OpenShift auth can live behind one CLI contract instead of being rewritten as engine-specific `oc login` fragments.
- Change impact stays in one place instead of being duplicated as YAML or Groovy conditionals per engine.

This is the core architectural point for the demo: portability is a side effect of maintainability.

## Why this is testable

- The repo has unit coverage around planning and runner behavior in `tools/company-ci/src/plan.rs` and `tools/company-ci/src/runner.rs`.
- The CLI has dry-run contract tests in `tools/company-ci/tests/cli_dry_run.rs`.
- OpenShift deploy behavior is testable at the command-plan layer, including pull-secret setup and route checks.
- The same top-level command surface is usable locally, under `act`, and in hosted CI.

A useful demo line here is: "We are not testing YAML branches. We are testing the CI API."

## What else to include in the demo

- A one-slide architecture summary showing "engine shell" around `company-ci`.
- A portability slide with the four snippets side by side.
- A short OpenShift auth contract slide beside the image contract slide.
- A failure-mode example showing that missing OpenShift credentials or registry credentials fail at a predictable contract boundary.
- A short note that the repo's current OpenShift path is oriented around the local profile today, while the env contract is designed to carry forward to external registries and clusters.

## Suggested live flow

1. Start with the thesis: thin pipelines, real CI logic in code.
2. Show the GitHub Actions workflow and point out how little logic is actually there.
3. Show the four-engine slide and highlight that only the wrapper syntax changes.
4. Show the OpenShift contract and explain that this is the stable integration surface.
5. Close on maintainability and testability: one command palette, one implementation, one place to evolve.

## Honest framing

- The snippets above are intentionally illustrative rather than copy-paste complete.
- They describe the intended future `company-ci` integration model, including prebuilt installation and CLI-owned OpenShift authentication.
- The point of the demo is the architecture pattern and the target interface, not a claim that every implementation detail already exists in this repository today.
