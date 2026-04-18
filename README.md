# Read It Later

Read it later app done using Tauri. Mainly for android.

![cover](fastlane/metadata/android/en-US/images/featureGraphic.png)

## Install

[<img src="https://gitlab.com/IzzyOnDroid/repo/-/raw/master/assets/IzzyOnDroidButtonGreyBorder_nofont.png" height="80px">](https://apt.izzysoft.de/packages/io.github.sak.read.it.later)

<!--
[<img src="https://play.google.com/intl/en_us/badges/static/images/badges/en_badge_web_generic.png" height="80px">](https://play.google.com/store/apps/details?id=io.github.sak.read.it.later)
-->

[<img src="https://img.shields.io/badge/Get%20it%20on-GitHub-black?logo=github" style="height:80px;" height="80px">](https://github.com/sak96/read_later/releases/latest)

## Setup

```bash
npm install --package-lock
cargo install tauri-cli@2.9.5
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

- image handling during offline.
- keep awake during tts.
- create [issue](https://github.com/sak96/read_later/issues) for more features

## License

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Distributed under the MIT License. See `LICENSE.md` for more information.

## Acknowledgment

- [Pico Css](https://picocss.com/)
- [Lucide icons](https://lucide.dev/icons/)
- [Readability Library](https://github.com/theiskaa/readabilityrs)

## Works well with

- [SherpaTTS](https://f-droid.org/packages/org.woheller69.ttsengine/)
