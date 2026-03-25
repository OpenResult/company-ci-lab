# company-ci-lab

`company-ci-lab` demonstrates a thin GitHub Actions architecture backed by a repo-local Rust CLI named `company-ci`.

The point of the repo is not "how to write more YAML". It is the opposite: GitHub Actions workflow YAML is a poor DSL for maintainable CI logic once the behavior becomes non-trivial. This repo keeps workflows thin and moves the real CI API into typed, testable Rust code that can run locally, under `act`, and on GitHub-hosted runners.

## Architectural intent

- Keep GitHub Actions thin and boring.
- Put reusable decision logic and side effects in `tools/company-ci`.
- Treat `company-ci` as the CI command palette and workflow-facing API.
- Keep orchestration in Rust where it is type-safe, testable, and refactorable.
- Provide sample applications and libraries that exercise build, test, package, publish, image, and deploy flows.
- Make local experimentation straightforward through the same CLI contract.

## Repository layout

- `tools/company-ci`: Rust CLI that owns orchestration.
- `apps/next-web`: Node-focused web sample with a Next-style layout and offline-buildable static artifact.
- `apps/spring-api`: minimal Spring Boot API.
- `libs/node-lib`: TypeScript library scaffold with generated ESM and declaration output.
- `libs/java-lib`: minimal Java library.
- `deploy/`: Kustomize-style deployment manifests.
- `testbeds/`: local environment assets.
- `.github/actions/setup-company-ci`: local action that installs the `company-ci` binary onto `PATH`.
- `.github/workflows/`: thin workflows that install and call `company-ci`.

## Quick start

```bash
cargo test
cargo run -p company-ci -- verify --dry-run
./scripts/dev-verify.sh
```

## CI contract

The always-on verification workflow installs `company-ci` onto `PATH` and delegates verification to that binary. When `COMPANY_CI_CHANGED_FILES` is provided, `company-ci` can internally narrow work to impacted areas and no-op unrelated component stages:

```bash
company-ci verify
```

Each top-level `company-ci` command also performs a required-tools and required-env preflight before it executes real work. GitHub Actions is still responsible for installing language runtimes, platform CLIs, and injecting credentials, but the Rust CLI verifies that the expected tools and contract inputs are actually present before side effects begin.

That split is deliberate:

- Workflows only describe triggers, runners, tool installation, and one top-level command.
- `company-ci` owns the maintainable logic: impact detection, sequencing, environment contracts, and platform-specific orchestration.
- The same command surface stays usable locally, in workflow runners, and in future installer-based entry points.

In other words, the maintainability story lives in Rust code, not in YAML conditionals.

## Command palette

The reusable CI API is the `company-ci` command palette:

```text
company-ci verify [--dry-run]
company-ci build [--dry-run]
company-ci test [--dry-run]
company-ci package [--dry-run]
company-ci publish maven-lib <path> [--dry-run]
company-ci publish npm-lib <path> [--dry-run] [--tag <dist-tag>]
company-ci image build [--dry-run]
company-ci image publish [--dry-run]
company-ci deploy openshift [--dry-run]
company-ci e2e openshift [--dry-run]
```

Containerized image workflows default to Docker. Set `COMPANY_CI_CONTAINER_ENGINE=podman` if you want the same commands to target Podman instead.

Container image flows now resolve from a generic env contract so the same CLI can target a local repository or a future external Artifactory-backed workflow:

- `COMPANY_CI_IMAGE_PUSH_REGISTRY`
- `COMPANY_CI_IMAGE_PULL_REGISTRY`
- `COMPANY_CI_IMAGE_NAMESPACE`
- `COMPANY_CI_IMAGE_TAG`
- `COMPANY_CI_IMAGE_PLATFORM`
- `COMPANY_CI_IMAGE_REGISTRY_USERNAME`
- `COMPANY_CI_IMAGE_REGISTRY_PASSWORD`
- `COMPANY_CI_IMAGE_REGISTRY_PASSWORD_FILE`

OpenShift deploy flows also use an auth contract:

- `COMPANY_CI_OPENSHIFT_API_URL`
- `COMPANY_CI_OPENSHIFT_TOKEN`
- `COMPANY_CI_OPENSHIFT_SKIP_TLS_VERIFY`

## Local setup

Supported local hosts:

- macOS
- Windows through WSL2
- Plain Windows shells are not supported.

Install the required toolchain on the host before running non-dry-run `company-ci` commands:

- Rust toolchain with `cargo`
- Node.js 24 with `npm`
- Java 21, with the repo-local Maven wrapper via `./mvnw`
- Docker or Podman for image and repository flows
- `act` only for local workflow smoke tests
- `oc` plus the OpenShift auth env contract for the OpenShift path

See `docs/local-dev.md` for concrete install steps on macOS and WSL.

See `docs/architecture.md`, `docs/workflows.md`, `docs/local-dev.md`, and `docs/test-strategy.md` for detailed guidance.

## Current reality

The most concrete paths in the scaffold today are the verification lanes, repository-backed publish flows, and the OpenShift deployment path:

- `apps/next-web` lint/tests/build produce a static artifact in `dist/`.
- `company-ci publish npm-lib libs/node-lib --tag ci` and `company-ci publish maven-lib libs/java-lib` drive the local library publish paths through the repository helper flow.
- `apps/spring-api` runs real Maven verify/package flows with Spring Boot tests through the repo-local wrapper.
- `company-ci deploy openshift` and `company-ci e2e openshift` model the OpenShift deployment path, with a repository and cluster expected to exist outside `company-ci`.

That is the core claim of the repo: maintainable CI should be real code with types, tests, and local execution, while GitHub Actions stays a thin transport for invoking that code.
