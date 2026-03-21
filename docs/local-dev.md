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

Dry-run output includes the required tool preflight for the selected command. Real runs verify those tools on `PATH` before starting work.

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

## Concrete slices

The most concrete local paths today are the Node and Java verification slices:

```bash
cd apps/next-web && npm run lint && npm test && npm run build
cd libs/node-lib && npm run lint && npm run typecheck && npm run build && npm test && npm run package
mvn -B -ntp -f apps/spring-api/pom.xml verify
mvn -B -ntp -f libs/java-lib/pom.xml verify
```

`libs/node-lib` uses repo-local Node scripts for type and build validation, while the Java lane uses direct Maven goals through `company-ci` so the orchestration stays in Rust rather than in helper scripts.
