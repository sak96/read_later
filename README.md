# Read It Later

Read it later app done using Tauri. Mainly for android.

![cover](fastlane/metadata/android/en-US/images/featureGraphic.png)

## Install

Use APK from [releases.](https://github.com/sak96/read_later/releases)

## Setup

```bash
cargo install wasm-bindgen-cli@0.2.106 wasm-opt@0.116.1 tauri-cli@2.9.5 trunk@0.21.14
```

```bash
rustup target add wasm32-unknown-unknown
```

## Development

```bash
cargo tauri dev
```

## Build

```bash
cargo tauri build
```

## Android

Setup environment as per [tauri](https://v2.tauri.app/start/prerequisites/#android).

- `JAVA_HOME`: can point to jbr folder from android studio.
- `ANDROID_HOME`: Possibly in `~/Android/Sdk`.
- `NDK_HOME`: Extracted NDK package folder. (e.g.: `android-ndk-r27d`)

To develop use (needs to be in same network):

```bash
cargo tauri android dev
```

To build use:

Follow code signing as per [here](https://tauri.app/distribute/sign/android/).

```bash
cargo tauri android build  --split-per-abi
```

## Screenshots

|![share](fastlane/metadata/android/en-US/images/phoneScreenshots/1_share.png)|![home](fastlane/metadata/android/en-US/images/phoneScreenshots/2_home.png)|
|---|---|
|![tts](fastlane/metadata/android/en-US/images/phoneScreenshots/3_tts.png)|![settings](fastlane/metadata/android/en-US/images/phoneScreenshots/4_settings.png)|
|![light](fastlane/metadata/android/en-US/images/phoneScreenshots/5_light.png)|![dark](fastlane/metadata/android/en-US/images/phoneScreenshots/6_dark.png)|

## Roadmap

- export and import need confirmation pages.
- rework css for code block.
- rework tts parsing logic.
- image handling during offline.
- keep awake during tts.

## Acknowledgment

- [Pico Css](https://picocss.com/)
- [Tabler icons](https://tabler.io/icons)
- [Readability Library](https://github.com/theiskaa/readabilityrs)

## Works well with

- [SherpaTTS](f-droid.org/packages/org.woheller69.ttsengine/)
