#!/bin/bash

# Setup
export arch=$2
export type=$3
if [ -z "$arch" ]; then
	export arch="aarch64-apple-ios"
fi
if [ -z "$type" ]; then
	export type="release"
fi

rm -r /tmp/lovely-ios 2> /dev/null

mkdir /tmp/lovely-ios

mkdir -p /tmp/lovely-ios/files/Library/MobileSubstrate/DynamicLibraries

cd "$(dirname "$0")"

export VERSION=$(cat ../lovely-core/Cargo.toml | grep ^version\ = | cut -f 2 -d \")
export ORIG_PWD=$PWD

# Copy files
cp deb-files/liblovely.plist /tmp/lovely-ios/files/Library/MobileSubstrate/DynamicLibraries

cp deb-files/control /tmp/lovely-ios/

cp ../../target/$arch/$type/liblovely.dylib /tmp/lovely-ios/files/Library/MobileSubstrate/DynamicLibraries

# Start building
cd /tmp/lovely-ios

echo '2.0' > debian-binary

perl -pi -e 'chomp if eof' control
echo "Version: $VERSION" >> control
echo "Installed-Size: $(du -c files | tail -1 | cut -f 1)" >> control

if [ ! -z "$ROOTLESS" ]; then
	export ARCH=arm64
	echo "Architecture: iphoneos-arm64" >> control
	mkdir -p files/var/jb
	mv files/* files/var/jb/ 2> /dev/null
else
	export ARCH=arm
	echo "Architecture: iphoneos-arm" >> control
fi
# Make
tar -czf control.tar.gz control
cd files
tar -cf ../data.tar.lzma --lzma *
cd ..
export name=systems.shorty.lovely-injector-$VERSION-iphoneos-$ARCH.deb
ar cr $name debian-binary control.tar.gz data.tar.lzma

# Done
mv $name $ORIG_PWD/../../target/$arch/$type/
rm -r /tmp/lovely-ios

echo "Created deb file in target/$arch/$type/$name"
