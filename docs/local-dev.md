# Local development

## Supported local hosts

This repo supports:

- macOS
- Windows through WSL2 with a Linux distro such as Ubuntu
- Plain Windows shells are not supported

Run the repo from the macOS terminal or from the Linux shell inside WSL. Do not run `company-ci` from PowerShell or Command Prompt.

## Required tools

- `cargo test` and all `company-ci` commands require a Rust toolchain with `cargo`.
- Component verification, packaging, and publishing require Node.js 24 with `npm`, plus Java 21. Maven comes from the repo-local `./mvnw` wrapper.
- Repository bootstrap uses `docker compose` by default or an equivalent Podman flow when `COMPANY_CI_CONTAINER_ENGINE=podman`.
- `company-ci deploy openshift` and `company-ci e2e openshift` require `oc`, plus the OpenShift auth env contract.
- `act` is only needed for local workflow smoke tests.

## Install required tools on macOS

Homebrew plus Docker Desktop is the shortest path:

```bash
xcode-select --install
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
brew install rustup node@24 openjdk@21
brew install --cask docker
rustup default stable
```

`node@24` and `openjdk@21` are versioned Homebrew formulae, so add them to your shell `PATH` if an older system version wins:

```bash
echo 'export JAVA_HOME="$(brew --prefix)/opt/openjdk@21/libexec/openjdk.jdk/Contents/Home"' >> ~/.zshrc
echo 'export PATH="$(brew --prefix)/opt/node@24/bin:$JAVA_HOME/bin:$PATH"' >> ~/.zshrc
```

Open a new shell after the install, then confirm the toolchain:

```bash
cargo --version
node --version
npm --version
java -version
./mvnw --version
docker version
```

Install `act` only if you want local workflow smoke tests:

```bash
brew install act
```

If you prefer Podman locally, install it separately and export `COMPANY_CI_CONTAINER_ENGINE=podman` before running the same commands.

## Install required tools in WSL2

These examples assume Ubuntu under WSL2. Keep the repo checkout in the Linux filesystem and run commands from the Linux shell.

Install the language toolchain inside WSL:

```bash
sudo apt-get update
sudo apt-get install -y build-essential ca-certificates curl git openjdk-21-jdk unzip zip
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.3/install.sh | bash
export NVM_DIR="$HOME/.nvm"
. "$NVM_DIR/nvm.sh"
nvm install 24
nvm alias default 24
. "$HOME/.cargo/env"
```

For the default container engine, install Docker Desktop on Windows, enable the WSL 2 backend, and turn on WSL integration for the distro that holds this repo. `docker version` should succeed inside WSL before you run repository bootstrap or any image flow.

Verify the WSL toolchain:

```bash
. "$HOME/.cargo/env"
export NVM_DIR="$HOME/.nvm"
. "$NVM_DIR/nvm.sh"
cargo --version
node --version
npm --version
java -version
./mvnw --version
docker version
```

Install `act` and `oc` only if you need the optional workflow-smoke or OpenShift paths.

## Fast verification

```bash
./scripts/bootstrap.sh
./scripts/dev-verify.sh
```

The default local model is Docker Desktop for repository and image flows. `company-ci` assumes `docker` unless `COMPANY_CI_CONTAINER_ENGINE=podman` is set. The local OpenShift-oriented profile resolves images through a repository Docker hosted endpoint on `localhost:5002`, with a local OpenShift environment such as CRC pulling those images through `host.crc.testing:5002`.

## Run the Rust CLI directly

```bash
cargo run -p company-ci -- verify --dry-run
cargo run -p company-ci -- publish npm-lib libs/node-lib --tag ci --dry-run
cargo run -p company-ci -- publish maven-lib libs/java-lib --dry-run
cargo run -p company-ci -- deploy openshift --dry-run
cargo run -p company-ci -- e2e openshift --dry-run
```

Dry-run output includes the required tool and env-contract preflight for the selected command. Real runs verify those inputs before starting work.

The first `./mvnw` execution downloads the pinned Maven distribution declared in `.mvn/wrapper/maven-wrapper.properties`.

The workflows install the same CLI onto `PATH` and then invoke `company-ci ...` directly. Locally, `cargo run -p company-ci -- ...` is just the bootstrap path before you package or install the binary yourself.

For hosted Maven publication, materialize a `settings.xml` file in the workflow and pass it to `company-ci` with `MAVEN_SETTINGS_PATH`, plus `MAVEN_DEPLOY_URL` and `MAVEN_SERVER_ID`. The wrapper still supplies Maven itself. Example:

```bash
MAVEN_SETTINGS_PATH="$RUNNER_TEMP/settings.xml" \
MAVEN_DEPLOY_URL="https://repo.example.com/repository/maven-snapshots/" \
MAVEN_SERVER_ID="github" \
cargo run -p company-ci -- publish maven-lib libs/java-lib
```

## Workflow smoke tests with act

Assets under `testbeds/workflows/act` provide a place for local `act` configuration. The happy path is:

```bash
act pull_request -W .github/workflows/verify.yml
```

## OpenShift profile

`company-ci e2e openshift` assumes an OpenShift environment and `oc` are already installed, and that the repository has already been started outside `company-ci`. The command builds and publishes app images into the repository Docker hosted repo, logs in to OpenShift through the env contract, deploys the OpenShift overlay, and verifies the exposed Routes.

The reusable OpenShift auth contract is:

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
COMPANY_CI_IMAGE_PLATFORM
COMPANY_CI_IMAGE_REGISTRY_USERNAME
COMPANY_CI_IMAGE_REGISTRY_PASSWORD
COMPANY_CI_IMAGE_REGISTRY_PASSWORD_FILE
```

For the local OpenShift profile, `company-ci` defaults these values to:

```bash
COMPANY_CI_IMAGE_PUSH_REGISTRY=localhost:5002
COMPANY_CI_IMAGE_PULL_REGISTRY=host.crc.testing:5002
COMPANY_CI_IMAGE_NAMESPACE=company-ci
COMPANY_CI_IMAGE_TAG=dev
COMPANY_CI_IMAGE_PLATFORM=linux/amd64
COMPANY_CI_IMAGE_REGISTRY_USERNAME=admin
COMPANY_CI_IMAGE_REGISTRY_PASSWORD_FILE=testbeds/repository/.runtime/admin.password
```

`company-ci e2e openshift` now defaults image builds to `linux/amd64`, which matches the common remote OpenShift worker architecture. Override `COMPANY_CI_IMAGE_PLATFORM` if your target cluster expects a different image architecture.

The direct local happy path is:

```bash
docker compose -f testbeds/repository/compose.yaml up -d
sh testbeds/repository/bootstrap.sh
COMPANY_CI_OPENSHIFT_API_URL=https://api.example.openshift.test:6443 \
COMPANY_CI_OPENSHIFT_TOKEN=... \
cargo run -p company-ci -- e2e openshift
```


## Scoping work to changed files

For local experiments that mimic CI change detection, set `COMPANY_CI_CHANGED_FILES` to a comma-separated file list before invoking `company-ci`. Example:

```bash
COMPANY_CI_CHANGED_FILES=docs/architecture.md cargo run -p company-ci -- build --dry-run
```

## Concrete slices

The most concrete local paths today are the Node and Java verification slices plus repository-backed publish flows:

```bash
cd apps/next-web && npm run lint && npm test && npm run build
cd libs/node-lib && npm run lint && npm run typecheck && npm run build && npm test && npm run package
./mvnw -B -ntp -f apps/spring-api/pom.xml verify
./mvnw -B -ntp -f libs/java-lib/pom.xml verify
docker compose -f testbeds/repository/compose.yaml up -d
sh testbeds/repository/bootstrap.sh
cargo run -p company-ci -- publish npm-lib libs/node-lib --tag ci
cargo run -p company-ci -- publish maven-lib libs/java-lib
```

`libs/node-lib` uses repo-local Node scripts for type and build validation, the Java lane uses the repo-local Maven wrapper through `company-ci`, and the local repository bootstrap captures runtime credentials in `testbeds/repository/.runtime/` so later package, image-publish, and OpenShift deploy steps can reuse them without extra workflow logic. If you need Podman locally, export `COMPANY_CI_CONTAINER_ENGINE=podman` before running the same commands.
