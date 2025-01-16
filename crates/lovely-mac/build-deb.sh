#!/bin/bash

# Setup
rm -r /tmp/lovely-ios 2> /dev/null

mkdir /tmp/lovely-ios

mkdir -p /tmp/lovely-ios/files/Library/MobileSubstrate/DynamicLibraries

cd "$(dirname "$0")"

export VERSION=$(cat ../lovely-core/Cargo.toml | grep ^version\ = | cut -f 2 -d \")
export ORIG_PWD=$PWD

# Copy files
cp deb-files/liblovely.plist /tmp/lovely-ios/files/Library/MobileSubstrate/DynamicLibraries

cp deb-files/control /tmp/lovely-ios/

cp ../../target/aarch64-apple-ios/aarch64-apple-ios/release/liblovely.dylib /tmp/lovely-ios/files/Library/MobileSubstrate/DynamicLibraries

# Start building
cd /tmp/lovely-ios

echo '2.0' > debian-binary

perl -pi -e 'chomp if eof' control
echo "Version: $VERSION" >> control
echo "Installed-Size: $(du -c files | tail -1 | cut -f 1)" >> control

cp control control-rootless

echo "Architecture: iphoneos-arm" >> control
echo "Architecture: iphoneos-arm64" >> control-rootless

# Make rootful
tar -czf control.tar.gz control
cd files
tar -cf ../data.tar.lzma --lzma *
cd ..
ar cr systems.shorty.lovely-injector-$VERSION-iphoneos-arm.deb debian-binary control.tar.gz data.tar.lzma

# Rootless
mv control-rootless control
tar -czf control.tar.gz control
cd files
mkdir -p var/jb
mv * var/jb/ 2> /dev/null
tar -cf ../data.tar.lzma --lzma *
cd ..
ar cr systems.shorty.lovely-injector-$VERSION-iphoneos-arm64.deb debian-binary control.tar.gz data.tar.lzma
# Done
mv *.deb $ORIG_PWD/../../target/aarch64-apple-ios/aarch64-apple-ios/release/
rm -r /tmp/lovely-ios

echo "Created deb files in target/aarch64-apple-ios/aarch64-apple-ios/release/"
