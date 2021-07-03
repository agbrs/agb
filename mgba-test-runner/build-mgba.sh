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
cmake .. \
    -DBUILD_STATIC=ON \
    -DBUILD_SHARED=OFF \
    -DDISABLE_FRONTENDS=ON \
    -DBUILD_GL=OFF \
    -DBUILD_GLES2=OFF \
    -DUSE_GDB_STUB=OFF \
	-DUSE_FFMPEG=OFF \
	-DUSE_ZLIB=OFF \
	-DUSE_MINIZIP=OFF \
	-DUSE_PNG=OFF \
	-DUSE_LIBZIP=OFF \
	-DUSE_SQLITE3=OFF \
	-DUSE_ELF=ON \
	-DM_CORE_GBA=ON \
	-DM_CORE_GB=OFF \
	-DUSE_LZMA=OFF \
	-DUSE_DISCORD_RPC=OFF \
	-DENABLE_SCRIPTING=OFF \
    -DUSE_EPOXY=OFF
make

cp libmgba.a ../../libmgba-cycle.a
