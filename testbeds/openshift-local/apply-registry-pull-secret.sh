#!/usr/bin/env bash
set -euo pipefail

secret_name="${1:-company-ci-registry}"
registry="${COMPANY_CI_IMAGE_PULL_REGISTRY:-}"
username="${COMPANY_CI_IMAGE_REGISTRY_USERNAME:-}"
password="${COMPANY_CI_IMAGE_REGISTRY_PASSWORD:-}"
password_file="${COMPANY_CI_IMAGE_REGISTRY_PASSWORD_FILE:-}"

if [ -z "${registry}" ]; then
  echo "missing COMPANY_CI_IMAGE_PULL_REGISTRY for pull secret" >&2
  exit 1
fi

if [ -z "${username}" ]; then
  echo "missing COMPANY_CI_IMAGE_REGISTRY_USERNAME for pull secret" >&2
  exit 1
fi

if [ -z "${password}" ] && [ -n "${password_file}" ] && [ -f "${password_file}" ]; then
  password="$(tr -d '\r\n' < "${password_file}")"
fi

if [ -z "${password}" ]; then
  echo "missing registry password; set COMPANY_CI_IMAGE_REGISTRY_PASSWORD or COMPANY_CI_IMAGE_REGISTRY_PASSWORD_FILE" >&2
  exit 1
fi

oc create secret docker-registry "${secret_name}" \
  --docker-server="${registry}" \
  --docker-username="${username}" \
  --docker-password="${password}" \
  --dry-run=client \
  -o yaml | oc apply -f - >/dev/null

echo "applied pull secret ${secret_name} for ${registry}"
