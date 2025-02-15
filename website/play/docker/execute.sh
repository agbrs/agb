#!/usr/bin/env bash

set -euo pipefail

cp /out/main.rs src/main.rs
cargo build --color always
agb-gbafix target/thumbv4t-none-eabi/debug/agb_template -o agb.gba
rm -f agb.gba.gz
gzip agb.gba
mv agb.gba.gz /out/agb.gba.gz
