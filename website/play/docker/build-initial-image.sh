#!/usr/bin/env bash

set -euo pipefail
cp -r ../../../template .
rm -rf template/target

cp -r ../../../agb/examples template

docker build --tag agb-build:latest . -f AgbBuild.dockerfile