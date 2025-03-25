#!/usr/bin/env bash

if [ $(dirname "$0") != '.' ]; then
    echo "Must call this with ./build-initial-image.sh"
    exit 1
fi

set -euo pipefail
cp -r ../../../template .
rm -rf template/target

cp -r ../../../agb/examples template

docker build --tag ghcr.io/agbrs/playground-builder:latest . -f AgbBuild.dockerfile