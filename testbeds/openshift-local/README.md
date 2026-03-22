# OpenShift Local

This profile assumes OpenShift Local is already installed and running.

## Happy path

```bash
cargo run -p company-ci -- env up nexus
oc login ...
cargo run -p company-ci -- e2e openshift-local
```

`company-ci e2e openshift-local` now:

- verifies the active `oc` login
- builds the app images
- publishes those images to the local Nexus Docker hosted repo on `localhost:5002`
- deploys the OpenShift overlay
- creates or updates the `company-ci-registry` pull secret
- verifies the `next-web` and `spring-api` Routes live

OpenShift Local pulls those same images through `host.crc.testing:5002`, which is the local stand-in for the production shape of Artifactory plus OpenShift.

## Registry contract

These env vars define the reusable image contract for OpenShift-based deploy flows:

```bash
COMPANY_CI_IMAGE_PUSH_REGISTRY
COMPANY_CI_IMAGE_PULL_REGISTRY
COMPANY_CI_IMAGE_NAMESPACE
COMPANY_CI_IMAGE_TAG
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
COMPANY_CI_IMAGE_REGISTRY_USERNAME=admin
COMPANY_CI_IMAGE_REGISTRY_PASSWORD_FILE=testbeds/repo/nexus/.runtime/admin.password
```

Set those values explicitly in future external OpenShift workflows to switch from local Nexus to Artifactory without changing the CLI surface.
