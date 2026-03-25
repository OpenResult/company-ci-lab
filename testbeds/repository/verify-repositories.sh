#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -lt 2 ]; then
  echo "Usage: testbeds/repository/verify-repositories.sh <repositories-json> <repository> [repository...]" >&2
  exit 1
fi

repositories_file="$1"
shift

if [ ! -f "${repositories_file}" ]; then
  echo "repository manifest not found: ${repositories_file}" >&2
  exit 1
fi

for repository in "$@"; do
  if ! grep -q "\"name\"[[:space:]]*:[[:space:]]*\"${repository}\"" "${repositories_file}"; then
    echo "required repository not found: ${repository}" >&2
    exit 1
  fi
done

echo "verified repositories: $*"
