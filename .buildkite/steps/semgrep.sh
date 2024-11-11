#!/bin/env bash

set -euo pipefail
source ./libs/shell/log.sh

echo "--- :information_source: Print Environment"
printenv

echo "--- :information_source: Versions"
semgrep --version

echo "--- :semgrep: Semgrep"
if semgrep scan --config=auto --experimental; then
  exit 0
else
  echo "^^^ +++"
  log error ":semgrep: Semgrep failed"
  exit 1
fi
