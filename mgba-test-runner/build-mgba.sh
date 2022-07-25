#!/usr/bin/env bash

MGBA_VERSION=$1
OUT_DIRECTORY=$2
CURRENT_DIRECTORY=$(pwd)

cd "${OUT_DIRECTORY}" || exit

if [[ ! -f "mgba-${MGBA_VERSION}.tar.gz" ]]; then
	curl -L "https://github.com/mgba-emu/mgba/archive/refs/tags/${MGBA_VERSION}.tar.gz" -o "mgba-${MGBA_VERSION}.tar.gz"
fi

if [[ -f libmgba-cycle.a ]]; then
	exit 0
fi

curl -L "https://github.com/mgba-emu/mgba/archive/refs/tags/${MGBA_VERSION}.tar.gz" -o "mgba-${MGBA_VERSION}.tar.gz"
tar -xvf "mgba-${MGBA_VERSION}.tar.gz"
cd "mgba-${MGBA_VERSION}" || exit
rm -rf build
patch --strip=1 < "${CURRENT_DIRECTORY}/add_cycles_register.patch"
mkdir -p build
cd build || exit
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
	-DCMAKE_BUILD_TYPE=Debug \
    -DUSE_EPOXY=OFF
make

cp libmgba.a ../../libmgba-cycle.a
