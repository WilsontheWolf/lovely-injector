#!/bin/bash
set -e
export IPHONEOS_DEPLOYMENT_TARGET=16.4
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
cargo build --target aarch64-apple-ios $RELEASE --package lovely-ios

if [ -z "$PACKAGE" ]; then
	echo Done. Output file at ./target/aarch64-apple-ios/$TYPE/liblovely.dylib
else
	echo Building .deb file
	./crates/lovely-ios/build-deb.sh $TYPE aarch64-apple-ios
fi

