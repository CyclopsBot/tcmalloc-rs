#!/bin/env bash

set -euo pipefail
source ./libs/shell/log.sh

echo "--- :information_source: Print Environment"
printenv

echo "--- :stethoscope: Check Health"
ci_toolkit ci healthcheck

echo "--- :bazel: Info"
bazel --nosystem_rc --nohome_rc version
bazel --nosystem_rc --nohome_rc info

echo "--- :bazel: Setup RBE"
touch .aspect/bazelrc/auth.bazelrc
cat >>.aspect/bazelrc/auth.bazelrc <<EOF
build --remote_header=x-buildbuddy-api-key=${BUILDBUDDY_API_KEY:?}
EOF
log info "RBE setup with buildbuddy token ${BUILDBUDDY_API_KEY:?}"

echo "--- :bazel: Push Images"
if bazel --bazelrc=.aspect/bazelrc/ci.bazelrc run --config=release "$1"; then
  exit 0
else
  echo "^^^ +++"
  log error ":bazel: Push Images failed"
  exit 1
fi
