#!/bin/env bash

set -euo pipefail

echo "--- :information_source: Print Environment"
printenv

echo "--- :stethoscope: Check Health"
ci_toolkit ci healthcheck

echo "--- :buildkite: Annotate Job"
ci_toolkit ci annotate
