#!/usr/bin/env bash

set -e # Fail if any command fails
set -x # print every command before it runs

# Requires gbafix and arm-none-eabi-objcopy to already be installed

function build_rom() {
    local GAME_FOLDER="$1"
    local INTERNAL_NAME="$2"

    local GAME_NAME
    GAME_NAME="$(basename "$GAME_FOLDER")"

    local TARGET_FOLDER="${CARGO_TARGET_DIR:-$GAME_FOLDER/target}"
    local GBA_FILE="$TARGET_FOLDER/$GAME_NAME.gba"

    (cd "$GAME_FOLDER" && cargo build --release --verbose --target thumbv4t-none-eabi)

    arm-none-eabi-objcopy -O binary "$TARGET_FOLDER/thumbv4t-none-eabi/release/$GAME_NAME" "$GBA_FILE"
    gbafix -p "-t${INTERNAL_NAME:0:12}" "-c${INTERNAL_NAME:0:4}" -mGC "$GBA_FILE"

    cp -v "$GBA_FILE" "examples/$GAME_NAME.gba"
}

mkdir -p examples/target

build_rom "examples/the-purple-night" "PURPLENIGHT"
build_rom "examples/the-hat-chooses-the-wizard" "HATWIZARD"

build_rom "book/games/pong" "PONG"

zip examples/target/examples.zip examples/*.gba