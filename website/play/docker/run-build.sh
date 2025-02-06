#!/usr/bin/env bash

set -euo pipefail

if [ "$#" -ne 2 ]; then
    echo "Usage: $0 <empty temp directory> <rust file>"
    exit 1
fi

OUT_DIR="$1"
cp "$2" "$OUT_DIR/main.rs"
chmod -R 777 "$OUT_DIR"
timeout 30s \
    docker run \
        --cap-drop=ALL --net=none --memory=256m --memory-swap=512m --pids-limit=512 --oom-score-adj=1000 \
        --rm -v "$OUT_DIR:/out" -i \
        agb-build:latest

echo "$OUT_DIR/agb.gba.gz"
