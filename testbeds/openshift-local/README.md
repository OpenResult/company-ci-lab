# OpenShift Local

This profile assumes OpenShift Local is already installed and running.

## Happy path

```bash
cargo run -p company-ci -- e2e openshift-local --dry-run
```

## Deferred details

- Authentication bootstrap is left to the local developer.
- Image registry configuration depends on the local cluster setup.
