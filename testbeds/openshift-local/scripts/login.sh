#!/usr/bin/env bash
set -euo pipefail

if ! oc whoami >/dev/null 2>&1; then
  echo "Log in to OpenShift with oc before invoking company-ci." >&2
  exit 1
fi

current_user="$(oc whoami)"
current_project="$(oc project -q)"
current_server="$(oc whoami --show-server)"

echo "using OpenShift context ${current_user} on ${current_project} via ${current_server}"
