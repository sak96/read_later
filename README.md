# Read It Later

Read it later app done using Tauri.

## Setup

```bash
cargo install trunk tauri-cli
cargo install wasm-bindgen-cli
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

```bash
cargo tauri android apk
```
