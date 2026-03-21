#!/usr/bin/env bash

company_ci_container_engine() {
  printf '%s\n' "${COMPANY_CI_CONTAINER_ENGINE:-docker}"
}

company_ci_container_engine_bin() {
  case "$(company_ci_container_engine)" in
    docker|podman)
      company_ci_container_engine
      ;;
    *)
      echo "unsupported COMPANY_CI_CONTAINER_ENGINE: $(company_ci_container_engine)" >&2
      return 1
      ;;
  esac
}

company_ci_compose() {
  local engine
  engine="$(company_ci_container_engine_bin)"
  "${engine}" compose "$@"
}
