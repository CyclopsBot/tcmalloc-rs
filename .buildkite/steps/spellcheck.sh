#!/bin/env bash

set -euo pipefail
source ./libs/shell/log.sh

echo "--- :information_source: Print Environment"
printenv

echo "--- :stethoscope: Check Health"
ci_toolkit ci healthcheck

echo "--- :information_source: Versions"
typos --version

echo "--- :recycle: Spellcheck"
if typos --config .buildkite/steps/typos.toml; then
  log info "completed spellcheck."
  exit 0
else
  echo "^^^ +++"
  log error ":recycle: Spellcheck failed"
  exit 1
fi
