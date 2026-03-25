# repository init

`testbeds/repository/bootstrap.sh` now handles readiness checks, captures the generated admin password into `testbeds/repository/.runtime/`, creates `npm-hosted` and `maven-snapshots` when they are missing, and validates that both repositories exist before later publish steps run.

Add extra bootstrap scripts or Groovy tasks here if the local repository profile ever needs custom repos beyond those defaults.
