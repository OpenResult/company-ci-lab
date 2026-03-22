#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${script_dir}/../../lib/container-engine.sh"

compose_file="${COMPANY_CI_NEXUS_COMPOSE_FILE:-testbeds/repo/nexus/compose.yaml}"
state_dir="${COMPANY_CI_NEXUS_STATE_DIR:-testbeds/repo/nexus/.runtime}"
base_url="${COMPANY_CI_NEXUS_URL:-http://localhost:8081}"
username="${COMPANY_CI_NEXUS_USERNAME:-admin}"

mkdir -p "${state_dir}"

refresh_repositories() {
  for _ in $(seq 1 120); do
    if curl --fail --silent --show-error -u "${username}:${password}" "${base_url}/service/rest/v1/repositories" >"${repositories_file}"; then
      return 0
    fi
    sleep 2
  done

  echo "failed to query Nexus repositories from ${base_url}" >&2
  exit 1
}

repository_exists() {
  local repository="$1"
  grep -q "\"name\"[[:space:]]*:[[:space:]]*\"${repository}\"" "${repositories_file}"
}

create_npm_hosted_repository() {
  curl --fail --silent --show-error \
    -u "${username}:${password}" \
    -H "Content-Type: application/json" \
    -X POST \
    "${base_url}/service/rest/v1/repositories/npm/hosted" \
    -d '{"name":"npm-hosted","online":true,"storage":{"blobStoreName":"default","strictContentTypeValidation":true,"writePolicy":"allow"}}' \
    >/dev/null
}

create_maven_snapshots_repository() {
  curl --fail --silent --show-error \
    -u "${username}:${password}" \
    -H "Content-Type: application/json" \
    -X POST \
    "${base_url}/service/rest/v1/repositories/maven/hosted" \
    -d '{"name":"maven-snapshots","online":true,"storage":{"blobStoreName":"default","strictContentTypeValidation":true,"writePolicy":"allow"},"maven":{"versionPolicy":"SNAPSHOT","layoutPolicy":"STRICT","contentDisposition":"ATTACHMENT"}}' \
    >/dev/null
}

create_container_hosted_repository() {
  curl --fail --silent --show-error \
    -u "${username}:${password}" \
    -H "Content-Type: application/json" \
    -X POST \
    "${base_url}/service/rest/v1/repositories/docker/hosted" \
    -d '{"name":"container-hosted","online":true,"storage":{"blobStoreName":"default","strictContentTypeValidation":true,"writePolicy":"allow"},"docker":{"v1Enabled":false,"forceBasicAuth":true,"httpPort":5002}}' \
    >/dev/null
}

for _ in $(seq 1 120); do
  if company_ci_compose -f "${compose_file}" exec -T nexus test -f /nexus-data/admin.password >/dev/null 2>&1; then
    break
  fi
  sleep 2
done

password="$(company_ci_compose -f "${compose_file}" exec -T nexus cat /nexus-data/admin.password | tr -d '\r\n')"
if [ -z "${password}" ]; then
  echo "failed to read Nexus admin password" >&2
  exit 1
fi

password_file="${state_dir}/admin.password"
printf '%s\n' "${password}" >"${password_file}"
chmod 600 "${password_file}"

repositories_file="${state_dir}/repositories.json"
refresh_repositories

if ! repository_exists "maven-snapshots"; then
  create_maven_snapshots_repository
  refresh_repositories
fi

if ! repository_exists "npm-hosted"; then
  create_npm_hosted_repository
  refresh_repositories
fi

if ! repository_exists "container-hosted"; then
  create_container_hosted_repository
  refresh_repositories
fi

sh "${script_dir}/verify-repositories.sh" "${repositories_file}" container-hosted maven-snapshots npm-hosted

echo "nexus ready at ${base_url} with Docker registry on localhost:5002"
