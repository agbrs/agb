#!/usr/bin/env bash

set -euo pipefail

curl http://localhost:3000/build --json @test.json --output out.gba.gz
if gunzip out.gba.gz; then
    mgba-qt out.gba
    rm -f out.gba out.gba.sav
else
    jq -r .error out.gba.gz
    rm out.gba.gz
fi
