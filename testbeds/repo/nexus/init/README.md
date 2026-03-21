# nexus init

`testbeds/repo/nexus/bootstrap.sh` now handles readiness checks, captures the generated admin password into `testbeds/repo/nexus/.runtime/`, and validates that the default `npm-hosted` and `maven-snapshots` repositories exist.

Add extra repository bootstrap scripts or Groovy tasks here if the emulated Nexus profile ever needs custom repos beyond those defaults.
