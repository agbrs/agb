#!/usr/bin/env bash

set -euo pipefail

if [ "$#" != 1 ]; then
    echo "Usage: $0 <rust file to compile>"
    exit 1
fi

curl -v http://localhost:5409/build --json "$(jq -Rs '{code: .}' < "$1")" --output out.gba.gz
if gunzip out.gba.gz; then
    mgba-qt out.gba
    rm -f out.gba out.gba.sav
else
    jq -r .error out.gba.gz
    rm out.gba.gz
fi
