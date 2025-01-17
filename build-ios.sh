#!/bin/bash
set -e

export TYPE=debug
if [[ $* == *package* ]]; then
	export PACKAGE=1
fi
if [[ $* == *rootless* ]]; then
	export ROOTLESS=1
fi
if [[ $* == *release* ]]; then
	export RELEASE="--release"
	export TYPE=release
fi

echo $ROOTLESS

echo Building arm64 $TYPE build
cargo +stage2 build --target aarch64-apple-ios $RELEASE --package lovely-ios

echo Building arm64e $TYPE build
cargo +stage2 build --target arm64e-apple-ios $RELEASE --package lovely-ios

echo Building universal lib
rm -rf target/universal/$TYPE/liblovely.dylib 2> /dev/null
mkdir -p target/universal/$TYPE

lipo -create -output target/universal/$TYPE/liblovely.dylib target/aarch64-apple-ios/$TYPE/liblovely.dylib target/arm64e-apple-ios/$TYPE/liblovely.dylib 

if [ -z "$PACKAGE" ]; then
	echo Done. Output file at ./target/universal/$TYPE/liblovely.dylib
else
	echo Building .deb file
	./crates/lovely-ios/build-deb.sh $TYPE universal
fi

