#!/usr/bin/env bash

set -euxo pipefail

if [ "$1" == "" ]; then
    echo "Must pass an argument (the rust file to execute)"
    exit 1
fi

OUT_DIR=$(mktemp -d --tmpdir playagbrsdev.XXXXXXXXX)
cp "$1" "$OUT_DIR/main.rs"
chmod -R 777 "$OUT_DIR"
timeout 30s \
    docker run \
        --cap-drop=ALL --net=none --memory=256m --memory-swap=512m --pids-limit=512 --oom-score-adj=1000 \
        --rm -v "$OUT_DIR:/out" -i \
        agb-build:latest

echo "$OUT_DIR/agb.gba.gz"
