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

See `docs/architecture.md`, `docs/workflows.md`, `docs/local-dev.md`, and `docs/test-strategy.md` for detailed guidance.

## Current reality

The most concrete paths in the scaffold today are the Node and Java verification lanes:

- `apps/next-web` lint/tests/build produce a static artifact in `dist/`.
- `libs/node-lib` runs lint, contract type checks, build, tests against built output, and `npm pack --dry-run`.
- `apps/spring-api` runs real Maven verify/package flows with Spring Boot tests.
- `libs/java-lib` runs real Maven verify/package/publish flows against a Maven-style target.
- `company-ci verify` exercises both the Node and Java lanes end-to-end.
