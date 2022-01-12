#!/usr/bin/env bash

set -e # Fail if any command fails

RELEASE_TAG=$(git tag --points-at HEAD)

PROJECT=${RELEASE_TAG/\/*/}
(cd "$PROJECT" && cargo publish)
