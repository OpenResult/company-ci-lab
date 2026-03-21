# company-ci-lab

`company-ci-lab` demonstrates a thin GitHub Actions architecture backed by a repo-local Rust CLI named `company-ci`.

## Architectural intent

- Keep GitHub Actions thin and boring.
- Put reusable decision logic and side effects in `tools/company-ci`.
- Provide sample applications and libraries that exercise build, test, package, publish, image, and deploy flows.
- Make local emulation the default path for end-to-end experimentation.

## Repository layout

- `tools/company-ci`: Rust CLI that owns orchestration.
- `apps/next-web`: Node-focused web sample with a Next-style layout and offline-buildable static artifact.
- `apps/spring-api`: minimal Spring Boot API.
- `libs/node-lib`: TypeScript library scaffold with generated ESM and declaration output.
- `libs/java-lib`: minimal Java library.
- `deploy/`: Kustomize-style deployment manifests.
- `testbeds/`: local and emulated environment assets.
- `.github/workflows/`: thin workflows that call `company-ci`.

## Quick start

```bash
cargo test
cargo run -p company-ci -- verify --dry-run
./scripts/dev-verify.sh
```

## CI contract

The always-on verification workflow builds the Rust CLI from source and delegates verification to `company-ci`. When `COMPANY_CI_CHANGED_FILES` is provided, `company-ci` can internally narrow work to impacted areas and no-op unrelated component stages:

```bash
cargo run -p company-ci -- verify
```

Each top-level `company-ci` command also performs a required-tools preflight before it executes real work. GitHub Actions is still responsible for installing language runtimes and platform CLIs, but the Rust CLI verifies that the expected tools are actually on `PATH`.

Containerized local workflows default to Docker and kind. Set `COMPANY_CI_CONTAINER_ENGINE=podman` if you want the same commands to target Podman instead.

## Local setup

Supported local hosts:

- macOS
- Windows through WSL2
- Plain Windows shells are not supported.

Install the required toolchain on the host before running non-dry-run `company-ci` commands:

- Rust toolchain with `cargo`
- Node.js 24 with `npm`
- Java 17 with `mvn`
- Docker plus `kind` and `kubectl` for the default emulated path
- `act` only for local workflow smoke tests
- `oc` only for the OpenShift Local path

See `docs/local-dev.md` for concrete install steps on macOS and WSL.

See `docs/architecture.md`, `docs/workflows.md`, `docs/local-dev.md`, and `docs/test-strategy.md` for detailed guidance.

## Current reality

The most concrete paths in the scaffold today are the verification lanes and the local emulated path:

- `apps/next-web` lint/tests/build produce a static artifact in `dist/`.
- `company-ci publish` now drives both `libs/node-lib` and `libs/java-lib` through the local Nexus helper path.
- `apps/spring-api` runs real Maven verify/package flows with Spring Boot tests.
- `company-ci e2e emulated` now brings up Nexus and kind helpers, pushes app images to a local registry, deploys to kind, and verifies live service responses.
