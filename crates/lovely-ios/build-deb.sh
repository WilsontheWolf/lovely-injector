#!/bin/bash

# Setup
export arch=$2
export type=$1
if [ -z "$arch" ]; then
	export arch="aarch64-apple-ios"
fi
if [ -z "$type" ]; then
	export type="release"
fi

rm -r /tmp/lovely-ios 2> /dev/null

mkdir /tmp/lovely-ios

mkdir -p /tmp/lovely-ios/Library/MobileSubstrate/DynamicLibraries

mkdir -p /tmp/lovely-ios/DEBIAN

cd "$(dirname "$0")"

export VERSION=$(cat ../lovely-core/Cargo.toml | grep ^version\ = | cut -f 2 -d \")
export ORIG_PWD=$PWD

# Copy files
cp deb-files/liblovely.plist /tmp/lovely-ios/Library/MobileSubstrate/DynamicLibraries

cp deb-files/control /tmp/lovely-ios/DEBIAN

cp ../../target/$arch/$type/liblovely.dylib /tmp/lovely-ios/Library/MobileSubstrate/DynamicLibraries

# Start building
cd /tmp/lovely-ios

ldid -S Library/MobileSubstrate/DynamicLibraries/liblovely.dylib

perl -pi -e 'chomp if eof' DEBIAN/control
echo "Version: $VERSION" >> DEBIAN/control
echo "Installed-Size: $(du -c Library | tail -1 | cut -f 1)" >> DEBIAN/control

if [ ! -z "$ROOTLESS" ]; then
	export ARCH=arm64
	echo "Architecture: iphoneos-arm64" >> DEBIAN/control
	mkdir -p var/jb
	mv Library var/jb/ 2> /dev/null
else
	export ARCH=arm
	echo "Architecture: iphoneos-arm" >> DEBIAN/control
fi
# Make
export name=systems.shorty.lovely-injector-$VERSION-iphoneos-$ARCH.deb
fakeroot $THEOS/bin/dm.pl -Zxz -z9 . $ORIG_PWD/../../target/$arch/$type/$name

# Done
rm -r /tmp/lovely-ios

echo "Created deb file in target/$arch/$type/$name"
