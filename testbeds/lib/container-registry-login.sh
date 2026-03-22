#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${script_dir}/container-engine.sh"

registry="${COMPANY_CI_IMAGE_PUSH_REGISTRY:-}"
username="${COMPANY_CI_IMAGE_REGISTRY_USERNAME:-}"
password="${COMPANY_CI_IMAGE_REGISTRY_PASSWORD:-}"
password_file="${COMPANY_CI_IMAGE_REGISTRY_PASSWORD_FILE:-}"
container_engine="$(company_ci_container_engine_bin)"
tls_verify="${COMPANY_CI_IMAGE_TLS_VERIFY:-false}"

if [ -z "${registry}" ]; then
  echo "missing COMPANY_CI_IMAGE_PUSH_REGISTRY for registry login" >&2
  exit 1
fi

if [ -z "${username}" ]; then
  echo "missing COMPANY_CI_IMAGE_REGISTRY_USERNAME for registry login" >&2
  exit 1
fi

if [ -z "${password}" ] && [ -n "${password_file}" ] && [ -f "${password_file}" ]; then
  password="$(tr -d '\r\n' < "${password_file}")"
fi

if [ -z "${password}" ]; then
  echo "missing registry password; set COMPANY_CI_IMAGE_REGISTRY_PASSWORD or COMPANY_CI_IMAGE_REGISTRY_PASSWORD_FILE" >&2
  exit 1
fi

case "${container_engine}" in
  docker)
    printf '%s' "${password}" | docker login "${registry}" --username "${username}" --password-stdin >/dev/null
    ;;
  podman)
    printf '%s' "${password}" | podman login "${registry}" --username "${username}" --password-stdin --tls-verify="${tls_verify}" >/dev/null
    ;;
  *)
    echo "unsupported container engine: ${container_engine}" >&2
    exit 1
    ;;
esac

echo "authenticated ${container_engine} to ${registry}"
