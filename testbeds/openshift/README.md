# OpenShift

This profile assumes an OpenShift environment is already installed or reachable.

## Happy path

```bash
docker compose -f testbeds/repository/compose.yaml up -d
sh testbeds/repository/bootstrap.sh
COMPANY_CI_OPENSHIFT_API_URL=https://api.example.openshift.test:6443 \
COMPANY_CI_OPENSHIFT_TOKEN=... \
cargo run -p company-ci -- e2e openshift
```

`company-ci e2e openshift` now:

- validates the required OpenShift auth env contract
- logs in to OpenShift with `oc`
- builds the app images
- publishes those images to the local repository Docker hosted repo on `localhost:5002`
- deploys the OpenShift overlay
- creates or updates the `company-ci-registry` pull secret
- verifies the `next-web` and `spring-api` Routes live

For local CRC-style testing, OpenShift pulls those same images through `host.crc.testing:5002`, which is the local stand-in for the production shape of Artifactory plus OpenShift.

## Auth contract

OpenShift auth also flows through the CLI:

```bash
COMPANY_CI_OPENSHIFT_API_URL
COMPANY_CI_OPENSHIFT_TOKEN
COMPANY_CI_OPENSHIFT_SKIP_TLS_VERIFY
```

## Registry contract

These env vars define the reusable image contract for OpenShift-based deploy flows:

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

Local defaults are:

```bash
COMPANY_CI_IMAGE_PUSH_REGISTRY=localhost:5002
COMPANY_CI_IMAGE_PULL_REGISTRY=host.crc.testing:5002
COMPANY_CI_IMAGE_NAMESPACE=company-ci
COMPANY_CI_IMAGE_TAG=dev
COMPANY_CI_IMAGE_PLATFORM=linux/amd64
COMPANY_CI_IMAGE_REGISTRY_USERNAME=admin
COMPANY_CI_IMAGE_REGISTRY_PASSWORD_FILE=testbeds/repository/.runtime/admin.password
```

Set those values explicitly in future external OpenShift workflows to switch from a local repository to Artifactory without changing the CLI surface.

`company-ci e2e openshift` defaults image builds to `linux/amd64` so Apple Silicon developer machines can still target the common `amd64` OpenShift worker shape. Override `COMPANY_CI_IMAGE_PLATFORM` when your cluster expects something else.
