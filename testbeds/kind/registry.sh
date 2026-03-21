#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# Shared shell helpers keep docker/podman selection consistent with the Rust CLI.
source "${script_dir}/../lib/container-engine.sh"

registry_name="${COMPANY_CI_KIND_REGISTRY_NAME:-kind-registry}"
registry_port="${COMPANY_CI_KIND_REGISTRY_PORT:-5001}"
kind_network="${COMPANY_CI_KIND_NETWORK:-kind}"
container_engine="$(company_ci_container_engine_bin)"

usage() {
  cat <<'EOF'
Usage: testbeds/kind/registry.sh up|down
EOF
}

ensure_registry_running() {
  if ! "${container_engine}" inspect "${registry_name}" >/dev/null 2>&1; then
    "${container_engine}" run -d --restart=always -p "127.0.0.1:${registry_port}:5000" --name "${registry_name}" registry:2 >/dev/null
  elif [ "$("${container_engine}" inspect -f '{{.State.Running}}' "${registry_name}")" != "true" ]; then
    "${container_engine}" start "${registry_name}" >/dev/null
  fi
}

connect_registry_to_kind_network() {
  if ! "${container_engine}" network inspect "${kind_network}" >/dev/null 2>&1; then
    echo "kind network not found for ${container_engine}; create the cluster before bootstrapping the registry" >&2
    exit 1
  fi

  if [ -z "$("${container_engine}" inspect -f "{{with index .NetworkSettings.Networks \"${kind_network}\"}}{{.NetworkID}}{{end}}" "${registry_name}")" ]; then
    "${container_engine}" network connect "${kind_network}" "${registry_name}" >/dev/null
  fi
}

publish_local_registry_hint() {
  kubectl apply -f - <<EOF >/dev/null
apiVersion: v1
kind: ConfigMap
metadata:
  name: local-registry-hosting
  namespace: kube-public
data:
  localRegistryHosting.v1: |
    host: "localhost:${registry_port}"
    help: "https://kind.sigs.k8s.io/docs/user/local-registry/"
EOF
}

case "${1:-}" in
  up)
    ensure_registry_running
    connect_registry_to_kind_network
    publish_local_registry_hint
    echo "kind registry available at localhost:${registry_port}"
    ;;
  down)
    if "${container_engine}" inspect "${registry_name}" >/dev/null 2>&1; then
      "${container_engine}" rm -f "${registry_name}" >/dev/null
    fi
    ;;
  *)
    usage >&2
    exit 1
    ;;
esac
