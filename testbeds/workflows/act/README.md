# act workflow smoke tests

Place local `.actrc` or secret mappings here when exercising the thin workflows with `act`.

Use the `Medium` default image for this repo. The workflows install their own toolchains, so the large runner snapshot is usually unnecessary for local smoke tests.

Use `act` mainly for thin-workflow checks such as:

```bash
act pull_request -W .github/workflows/verify.yml
```
