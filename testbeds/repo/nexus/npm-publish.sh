#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -ne 1 ]; then
  echo "Usage: testbeds/repo/nexus/npm-publish.sh <package-dir>" >&2
  exit 1
fi

package_dir="$1"
registry_url="${NPM_REGISTRY_URL:-${COMPANY_CI_NEXUS_URL:-http://localhost:8081}/repository/npm-hosted/}"
registry_url="${registry_url%/}/"
username="${COMPANY_CI_NEXUS_USERNAME:-admin}"
email="${COMPANY_CI_NPM_EMAIL:-company-ci@example.test}"
password_file="${COMPANY_CI_NEXUS_PASSWORD_FILE:-testbeds/repo/nexus/.runtime/admin.password}"

password="${COMPANY_CI_NEXUS_PASSWORD:-}"
if [ -z "${password}" ] && [ -f "${password_file}" ]; then
  password="$(tr -d '\r\n' < "${password_file}")"
fi

if [ -z "${password}" ]; then
  echo "missing Nexus password; run company-ci env up nexus or set COMPANY_CI_NEXUS_PASSWORD" >&2
  exit 1
fi

tmp_dir="$(mktemp -d)"
publish_dir="${tmp_dir}/package"
npmrc_file="${tmp_dir}/.npmrc"
cleanup() {
  rm -rf "${tmp_dir}"
}
trap cleanup EXIT

cp -R "${package_dir}" "${publish_dir}"

suffix="${COMPANY_CI_NPM_VERSION_SUFFIX:-companyci.$(date +%Y%m%d%H%M%S)}"
node -e '
const fs = require("node:fs");
const path = process.argv[1];
const suffix = process.argv[2];
const pkg = JSON.parse(fs.readFileSync(path, "utf8"));
pkg.version = `${pkg.version}-${suffix}`;
fs.writeFileSync(path, `${JSON.stringify(pkg, null, 2)}\n`);
' "${publish_dir}/package.json" "${suffix}"

auth="$(printf '%s:%s' "${username}" "${password}" | base64 | tr -d '\r\n')"
registry_host_path="${registry_url#http://}"
registry_host_path="${registry_host_path#https://}"

cat >"${npmrc_file}" <<EOF
registry=${registry_url}
@company:registry=${registry_url}
//${registry_host_path}:_auth=${auth}
//${registry_host_path}:email=${email}
//${registry_host_path}:always-auth=true
EOF

(cd "${publish_dir}" && npm publish --registry "${registry_url}" --userconfig "${npmrc_file}")
