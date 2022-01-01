#!/usr/bin/env bash

set -e # Fail if any command fails
set -x # print every command before it runs

# Requires gbafix and arm-none-eabi-objcopy to already be installed

function build_rom() {
    local GAME_NAME="$1"
    local INTERNAL_NAME="$2"

    local TARGET_FOLDER="${CARGO_TARGET_DIR:-target}"
    local GBA_FILE="$TARGET_FOLDER/$GAME_NAME.gba"

    pushd "examples/$GAME_NAME"
    cargo build --release --verbose --target thumbv4t-none-eabi

    arm-none-eabi-objcopy -O binary "$TARGET_FOLDER/thumbv4t-none-eabi/release/$GAME_NAME" "$GBA_FILE"
    gbafix -p "-t${INTERNAL_NAME:0:12}" "-c${INTERNAL_NAME:0:4}" -mGC "$GBA_FILE"

    cp -v "$GBA_FILE" "../$GAME_NAME.gba"

    popd
}

build_rom "the-purple-night" "PURPLENIGHT"
build_rom "the-hat-chooses-the-wizard" "HATWIZARD"