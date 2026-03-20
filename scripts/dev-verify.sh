#!/usr/bin/env bash
set -euo pipefail

cargo test
cargo run -p company-ci -- verify --dry-run
