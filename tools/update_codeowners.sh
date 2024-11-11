#!/usr/bin/env bash

# target to the generate_codeowners, eg "//.github:gen_codeowners"
readonly target=$1

readonly BAZEL=$(which bazel)
readonly BUILDOZER=$(which buildozer)

readonly new_owners=$(
  # Query for all codeowners() rules (anchor at the front to avoid match on generate_codeowners rule)
  $BAZEL query --output=label 'kind("^codeowners rule", //...)' |
    # Print the length of each label at the front of the line
    awk '{ print length, $0 }' |
    # Sort shortest-first, so that the root //:OWNERS ends up first in CODEOWNERS
    sort -n -s |
    # 43 label -> "label"
    awk '{ print "\x22" $2 "\x22" }' |
    # comma-separated
    tr '\n' ','
)

readonly command="set owners [${new_owners}]|${target}"

echo "$command" | $BUILDOZER -f -
