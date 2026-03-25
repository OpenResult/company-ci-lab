#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -lt 2 ] || [ "$#" -gt 3 ]; then
  echo "Usage: testbeds/openshift/check-route.sh <route> <path> [expected-substring]" >&2
  exit 1
fi

route_name="$1"
path="$2"
expected_substring="${3:-}"
response_file="$(mktemp)"

cleanup() {
  rm -f "${response_file}"
}
trap cleanup EXIT

for _ in $(seq 1 60); do
  route_host="$(oc get route "${route_name}" -o jsonpath='{.spec.host}' 2>/dev/null || true)"
  if [ -n "${route_host}" ] && curl --fail --silent --show-error "http://${route_host}${path}" >"${response_file}"; then
    if [ -z "${expected_substring}" ] || grep -Fq "${expected_substring}" "${response_file}"; then
      echo "verified ${route_name}${path} via ${route_host}"
      exit 0
    fi

    echo "unexpected response from ${route_name}${path}" >&2
    cat "${response_file}" >&2
    exit 1
  fi
  sleep 2
done

echo "timed out waiting for route ${route_name}${path}" >&2
oc get route "${route_name}" -o yaml >&2 || true
exit 1
