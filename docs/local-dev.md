# Local development

## Fast verification

```bash
./scripts/bootstrap.sh
./scripts/dev-verify.sh
```

## Run the Rust CLI directly

```bash
cargo run -p company-ci -- verify --dry-run
cargo run -p company-ci -- e2e emulated --dry-run
```

## Workflow smoke tests with act

Assets under `testbeds/workflows/act` provide a place for local `act` configuration. The happy path is:

```bash
act pull_request -W .github/workflows/verify.yml
act workflow_dispatch -W .github/workflows/emulated-e2e.yml
```

## OpenShift Local profile

`company-ci e2e openshift-local` assumes OpenShift Local and `oc` are already installed. See `testbeds/openshift-local/README.md`.


## Scoping work to changed files

For local experiments that mimic CI change detection, set `COMPANY_CI_CHANGED_FILES` to a comma-separated file list before invoking `company-ci`. Example:

```bash
COMPANY_CI_CHANGED_FILES=docs/architecture.md cargo run -p company-ci -- build --dry-run
```

## Node slice

The most complete local path today is the Node slice:

```bash
cd apps/next-web && npm run lint && npm test && npm run build
cd libs/node-lib && npm run lint && npm run typecheck && npm run build && npm test && npm run package
```

`libs/node-lib` uses repo-local Node scripts for type and build validation, so the verification path stays self-contained.
