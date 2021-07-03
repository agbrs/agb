#!/bin/bash

MGBA_VERSION=$1
OUT_DIRECTORY=$2

cd ${OUT_DIRECTORY}
curl -L https://github.com/mgba-emu/mgba/archive/refs/tags/${MGBA_VERSION}.tar.gz -o mgba-${MGBA_VERSION}.tar.gz
tar -xvf mgba-${MGBA_VERSION}.tar.gz
cd mgba-${MGBA_VERSION}
rm -rf build
mkdir -p build
cd build
cmake .. -DBUILD_STATIC=ON -DBUILD_SHARED=OFF -DDISABLE_FRONTENDS=ON -DBUILD_GL=OFF -DBUILD_GLES2=OFF -DUSE_DISCORD_RPC=OFF -DUSE_PNG=OFF
make

cp libmgba.a ../../libmgba-cycle.a
