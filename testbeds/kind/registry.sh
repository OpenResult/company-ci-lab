#!/usr/bin/env bash
set -euo pipefail

registry_name="${COMPANY_CI_KIND_REGISTRY_NAME:-kind-registry}"
registry_port="${COMPANY_CI_KIND_REGISTRY_PORT:-5001}"

usage() {
  cat <<'EOF'
Usage: testbeds/kind/registry.sh up|down
EOF
}

ensure_registry_running() {
  if ! docker inspect "${registry_name}" >/dev/null 2>&1; then
    docker run -d --restart=always -p "127.0.0.1:${registry_port}:5000" --name "${registry_name}" registry:2 >/dev/null
  elif [ "$(docker inspect -f '{{.State.Running}}' "${registry_name}")" != "true" ]; then
    docker start "${registry_name}" >/dev/null
  fi
}

connect_registry_to_kind_network() {
  local network
  network="kind"
  if ! docker network inspect "${network}" >/dev/null 2>&1; then
    echo "kind docker network not found; create the cluster before bootstrapping the registry" >&2
    exit 1
  fi

  if [ -z "$(docker inspect -f '{{with index .NetworkSettings.Networks "kind"}}{{.NetworkID}}{{end}}' "${registry_name}")" ]; then
    docker network connect "${network}" "${registry_name}" >/dev/null
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
    if docker inspect "${registry_name}" >/dev/null 2>&1; then
      docker rm -f "${registry_name}" >/dev/null
    fi
    ;;
  *)
    usage >&2
    exit 1
    ;;
esac
