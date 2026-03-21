#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -lt 4 ] || [ "$#" -gt 5 ]; then
  echo "Usage: testbeds/kind/check-service.sh <service> <local-port> <remote-port> <path> [expected-substring]" >&2
  exit 1
fi

service_name="$1"
local_port="$2"
remote_port="$3"
path="$4"
expected_substring="${5:-}"

forward_log="$(mktemp)"
response_file="$(mktemp)"

cleanup() {
  if [ -n "${forward_pid:-}" ]; then
    kill "${forward_pid}" >/dev/null 2>&1 || true
    wait "${forward_pid}" >/dev/null 2>&1 || true
  fi
  rm -f "${forward_log}" "${response_file}"
}
trap cleanup EXIT

kubectl port-forward "service/${service_name}" "${local_port}:${remote_port}" >"${forward_log}" 2>&1 &
forward_pid=$!

for _ in $(seq 1 30); do
  if curl --fail --silent --show-error "http://127.0.0.1:${local_port}${path}" >"${response_file}"; then
    if [ -z "${expected_substring}" ] || grep -Fq "${expected_substring}" "${response_file}"; then
      echo "verified ${service_name}${path}"
      exit 0
    fi

    echo "unexpected response from ${service_name}${path}" >&2
    cat "${response_file}" >&2
    exit 1
  fi
  sleep 1
done

echo "timed out waiting for ${service_name}${path}" >&2
cat "${forward_log}" >&2 || true
exit 1
