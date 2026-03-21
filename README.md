# company-ci-lab

`company-ci-lab` demonstrates a thin GitHub Actions architecture backed by a repo-local Rust CLI named `company-ci`.

## Architectural intent

- Keep GitHub Actions thin and boring.
- Put reusable decision logic and side effects in `tools/company-ci`.
- Provide sample applications and libraries that exercise build, test, package, publish, image, and deploy flows.
- Make local emulation the default path for end-to-end experimentation.

## Repository layout

- `tools/company-ci`: Rust CLI that owns orchestration.
<<<<<<< ours
<<<<<<< ours
<<<<<<< ours
- `apps/next-web`: minimal Next.js web application.
- `apps/spring-api`: minimal Spring Boot API.
- `libs/node-lib`: minimal TypeScript library.
=======
- `apps/next-web`: Node-focused web sample with a Next-style layout and offline-buildable static artifact.
- `apps/spring-api`: minimal Spring Boot API.
- `libs/node-lib`: real TypeScript library compiled with `tsc`.
>>>>>>> theirs
=======
- `apps/next-web`: Node-focused web sample with a Next-style layout and offline-buildable static artifact.
- `apps/spring-api`: minimal Spring Boot API.
- `libs/node-lib`: real TypeScript library compiled with `tsc`.
>>>>>>> theirs
=======
- `apps/next-web`: Node-focused web sample with a Next-style layout and offline-buildable static artifact.
- `apps/spring-api`: minimal Spring Boot API.
- `libs/node-lib`: real TypeScript library compiled with `tsc`.
>>>>>>> theirs
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

The always-on verification workflow builds the Rust CLI from source and delegates verification to. When `COMPANY_CI_CHANGED_FILES` is provided, `company-ci` can internally narrow work to impacted areas and no-op unrelated component stages:

```bash
cargo run -p company-ci -- verify
```

See `docs/architecture.md`, `docs/workflows.md`, `docs/local-dev.md`, and `docs/test-strategy.md` for detailed guidance.
<<<<<<< ours
<<<<<<< ours
<<<<<<< ours
=======
=======
>>>>>>> theirs
=======
>>>>>>> theirs

## Current reality

The Node slice is the most concrete path in the scaffold today:

- `apps/next-web` lint/tests/build produce a static artifact in `dist/`.
- `libs/node-lib` runs lint, typecheck, `tsc` build, tests against built output, and `npm pack --dry-run`.
- `company-ci verify` exercises that Node path end-to-end.
<<<<<<< ours
<<<<<<< ours
>>>>>>> theirs
=======
>>>>>>> theirs
=======
>>>>>>> theirs
