#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -ne 1 ]; then
  echo "Usage: testbeds/repository/maven-deploy.sh <pom-path>" >&2
  exit 1
fi

pom_path="$1"
repo_root="$(cd "$(dirname "$0")/../../.." && pwd)"
maven_cmd="${repo_root}/mvnw"
repo_url="${MAVEN_DEPLOY_URL:-${COMPANY_CI_REPOSITORY_URL:-${COMPANY_CI_NEXUS_URL:-http://localhost:8081}}/repository/maven-snapshots/}"
server_id="${MAVEN_SERVER_ID:-local}"
username="${COMPANY_CI_REPOSITORY_USERNAME:-${COMPANY_CI_NEXUS_USERNAME:-admin}}"
password_file="${COMPANY_CI_REPOSITORY_PASSWORD_FILE:-${COMPANY_CI_NEXUS_PASSWORD_FILE:-testbeds/repository/.runtime/admin.password}}"

if [ -n "${MAVEN_SETTINGS_PATH:-}" ]; then
  "${maven_cmd}" -B -ntp -s "${MAVEN_SETTINGS_PATH}" -f "${pom_path}" -DskipTests -DaltDeploymentRepository="${server_id}::${repo_url}" deploy
  exit 0
fi

password="${COMPANY_CI_REPOSITORY_PASSWORD:-${COMPANY_CI_NEXUS_PASSWORD:-}}"
if [ -z "${password}" ] && [ -f "${password_file}" ]; then
  password="$(tr -d '\r\n' < "${password_file}")"
fi

if [ -z "${password}" ]; then
  echo "missing repository password; initialize the repository or set COMPANY_CI_REPOSITORY_PASSWORD/MAVEN_SETTINGS_PATH" >&2
  exit 1
fi

work_dir="$(mktemp -d)"
settings_file="${work_dir}/settings.xml"
cleanup() {
  rm -rf "${work_dir}"
}
trap cleanup EXIT

cat >"${settings_file}" <<EOF
<settings>
  <servers>
    <server>
      <id>${server_id}</id>
      <username>${username}</username>
      <password>${password}</password>
    </server>
  </servers>
</settings>
EOF

"${maven_cmd}" -B -ntp -s "${settings_file}" -f "${pom_path}" -DskipTests -DaltDeploymentRepository="${server_id}::${repo_url}" deploy
