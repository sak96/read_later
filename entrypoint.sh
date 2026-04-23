#!/bin/bash
set -e

export CARGOFLAGS="--locked"
export RUSTFLAGS='--remap-path-prefix $PWD=. -C metadata=deadbeef'
export SOURCE_DATE_EPOCH=1700000000

if [ -f package-lock.json ] || [ -f package.json ]; then
    npm ci
fi

sed -i 's|distributionUrl=https\\://services.gradle.org/distributions/gradle-.*-bin.zip|distributionUrl=file\\:///opt/gradle-8.14.3-bin.zip|g' src-tauri/gen/android/gradle/wrapper/gradle-wrapper.properties
echo $PWD
pwd

if [ -f /tmp/keys.jks ]; then
    sed -i 's|^storeFile=.*|storeFile=/tmp/keys.jks|' src-tauri/gen/android/keystore.properties
else
    rm src-tauri/gen/android/keystore.properties
fi

cargo tauri android build --split-per-abi --target aarch64

echo "Build complete. APK available at /outputs/unsigned.apk"
