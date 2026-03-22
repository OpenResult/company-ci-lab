# nexus init

`testbeds/repo/nexus/bootstrap.sh` now handles readiness checks, captures the generated admin password into `testbeds/repo/nexus/.runtime/`, creates `npm-hosted` and `maven-snapshots` when they are missing, and validates that both repositories exist before later publish steps run.

Add extra repository bootstrap scripts or Groovy tasks here if the emulated Nexus profile ever needs custom repos beyond those defaults.
