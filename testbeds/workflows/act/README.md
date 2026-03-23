# act workflow smoke tests

Place local `.actrc` or secret mappings here when exercising the thin workflows with `act`.

Use the `Medium` default image for this repo. The workflows install their own toolchains, so the large runner snapshot is usually unnecessary for local smoke tests.

Use `act` mainly for thin-workflow checks such as:

```bash
act pull_request -W .github/workflows/verify.yml
```

The full `emulated-e2e` path is more practical to run directly on the host with `company-ci e2e emulated` because it depends on nested container and cluster behavior that is awkward under `act`.
