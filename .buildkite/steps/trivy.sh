#!/bin/env bash

set -euo pipefail
source ./libs/shell/log.sh

echo "--- :information_source: Print Environment"
printenv

echo "--- :stethoscope: Check Health"
ci_toolkit ci healthcheck

echo "--- :trivy: Print Version"
trivy --version

echo "--- :trivy: Scan Images"
if trivy image --scanners vuln "$1"; then
  exit 0
else
  echo "^^^ +++"
  log error ":trivy: Scan Images failed"
  exit 1
fi
