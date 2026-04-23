#!/bin/bash
set -e

IMAGE_NAME="read_later:apk"
CONTAINER_NAME="read_later-build"

# Source environment to get paths
# export JAVA_HOME="$HOME/Downloads/android/jbr"
export JAVA_HOME="$HOME/Downloads/android/jdk-21.0.10+7"
export ANDROID_HOME="$HOME/Downloads/android/Sdk"
# export NDK_HOME="$HOME/Downloads/android/Sdk/ndk/android-ndk-r27d"

# Build the Docker image
echo "Building Docker image: ${IMAGE_NAME}"
docker build -t "${IMAGE_NAME}" .

# Remove any existing container
docker rm -f "${CONTAINER_NAME}" 2>/dev/null || true
mkdir -p outputs

# Run the container with mounted SDKs
echo "Running build container..."
docker run --name "${CONTAINER_NAME}" \
    -v "${JAVA_HOME}:/opt/java:ro" \
    -v "${ANDROID_HOME}:/opt/android/sdk:ro" \
    -v "$HOME/.local/share/andorid_keystore/sak96-read-laterkeystore.jks":"/tmp/keys.jks:ro" \
    -e JAVA_HOME="/opt/java" \
    -e ANDROID_HOME="/opt/android/sdk/" \
    -e NDK_HOME="/opt/android/sdk/ndk/android-ndk-r27d" \
    "${IMAGE_NAME}"

# Extract the apk
docker cp ${CONTAINER_NAME}:/home/runner/work/read_later/read_later/src-tauri/gen/android/app/build/outputs/apk/arm64/release/app-arm64-release-unsigned.apk outputs/unsigned.apk || echo "failed to get unsigned.apk"
docker cp ${CONTAINER_NAME}:/home/runner/work/read_later/read_later/src-tauri/gen/android/app/build/outputs/apk/arm64/release/app-arm64-release.apk outputs/signed.apk || echo "failed to get signed.apk"
docker cp ${CONTAINER_NAME}:/home/runner/work/read_later/read_later/src-tauri/target outputs/target || echo "failed to get target"
docker cp ${CONTAINER_NAME}:/home/runner/work/read_later/read_later/dist outputs/dist || echo "failed to get dist"
echo "Build complete. APK available at outputs/unsigned.apk"
