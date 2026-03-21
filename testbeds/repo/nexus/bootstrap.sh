#!/usr/bin/env bash
set -euo pipefail

compose_file="${COMPANY_CI_NEXUS_COMPOSE_FILE:-testbeds/repo/nexus/compose.yaml}"
state_dir="${COMPANY_CI_NEXUS_STATE_DIR:-testbeds/repo/nexus/.runtime}"
base_url="${COMPANY_CI_NEXUS_URL:-http://localhost:8081}"
username="${COMPANY_CI_NEXUS_USERNAME:-admin}"

mkdir -p "${state_dir}"

for _ in $(seq 1 120); do
  if docker compose -f "${compose_file}" exec -T nexus test -f /nexus-data/admin.password >/dev/null 2>&1; then
    break
  fi
  sleep 2
done

password="$(docker compose -f "${compose_file}" exec -T nexus cat /nexus-data/admin.password | tr -d '\r\n')"
if [ -z "${password}" ]; then
  echo "failed to read Nexus admin password" >&2
  exit 1
fi

password_file="${state_dir}/admin.password"
printf '%s\n' "${password}" >"${password_file}"
chmod 600 "${password_file}"

repositories_file="${state_dir}/repositories.json"
for _ in $(seq 1 120); do
  if curl --fail --silent --show-error -u "${username}:${password}" "${base_url}/service/rest/v1/repositories" >"${repositories_file}"; then
    break
  fi
  sleep 2
done

for repository in maven-snapshots npm-hosted; do
  if ! grep -q "\"name\"[[:space:]]*:[[:space:]]*\"${repository}\"" "${repositories_file}"; then
    echo "required Nexus repository not found: ${repository}" >&2
    exit 1
  fi
done

echo "nexus ready at ${base_url}"
