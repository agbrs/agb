#!/usr/bin/env bash

set -e # Fail if any command fails

PROJECTS_TO_RELEASE_IN_ORDER="agb-fixnum agb-macros agb-image-converter agb-sound-converter agb"

for PROJECT in $PROJECTS_TO_RELEASE_IN_ORDER; do
   pushd "$PROJECT"
   echo "Publishing $PROJECT"
   cargo publish
   popd
done
