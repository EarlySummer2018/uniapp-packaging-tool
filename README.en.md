# UniPack Tool

> A desktop automation tool for uni-app offline packaging across Android, iOS, and HarmonyOS.

- The current tool verification is based on a single UniApp project, and there may be differences in module combinations, dependency versions, or manifest configurations across different projects. **If you encounter any packaging issues, please feel free to submit feedback via [Issues](https://github.com/EarlySummer2018/uniapp-packaging-tool/issues). The project will continue to be optimized and improved. **

English | [中文](./README.md)

UniPack Tool is a Tauri + Vue 3 + TypeScript + Rust desktop application. It brings together SDK setup, project configuration, resource import, module detection, signing settings, build logs, and artifact management so uni-app offline packaging can be easier to repeat and debug.

## Features

- Project management: create, save, and switch between multiple packaging projects.
- Multi-platform configuration: manage Android, iOS, and HarmonyOS package IDs, signing files, output directories, and build settings.
- Resource import: import local uni-app projects or resource packages, read `manifest.json`, and extract app metadata.
- Module detection: analyze common DCloud/native modules, UTS plugins, and required Android module parameters.
- SDK management: configure DCloud Android/iOS offline SDK paths and Harmony project templates, with local environment checks.
- Build center: start builds for selected platforms, stream logs, and collect APK/IPA/HAP artifacts.
- Build history: track status, duration, version, logs, and artifact paths.
- Secret handling: signing passwords are stored via the system Keychain/credential backend instead of plain project config files.

## Tech Stack

- Desktop: Tauri 2
- Frontend: Vue 3, TypeScript, Vite, Pinia, Vue Router, Naive UI
- Backend: Rust, Tokio, Serde, Reqwest
- Packaging: DCloud offline SDK, Android Gradle, Xcode, HarmonyOS project templates

## Requirements

Base development environment:

- Node.js 18+
- npm
- Rust stable
- Tauri 2 system dependencies

Additional platform requirements:

- Android: JDK, Android SDK, SDK Build Tools, Gradle or the bundled Gradle Wrapper, DCloud Android offline SDK
- iOS: macOS, Xcode, Command Line Tools, CocoaPods, DCloud iOS offline SDK
- HarmonyOS: DevEco Studio / HarmonyOS SDK and a usable Harmony project template

## Quick Start

```bash
git clone https://github.com/EarlySummer2018/uniapp-packaging-tool.git
cd uniapp-packaging-tool
npm install
npm run tauri dev
```

Useful commands:

```bash
npm run dev        # Start the Vite frontend only
npm run build      # Build frontend assets
npm run typecheck  # Run TypeScript type checking
npm run tauri dev  # Start the Tauri desktop app in development
npm run tauri build
```

## Workflow

1. Configure DCloud offline SDKs and the Harmony template in SDK Manager, then check the local environment.
2. Create a project from the home page and set its name and output directory.
3. Configure Android/iOS/HarmonyOS package IDs, certificates, signing settings, and platform options.
4. Import a uni-app project or resource package in Build Center and read `manifest.json`.
5. Fill in required module parameters, choose target platforms, and start the build.
6. Review build results, logs, and artifact paths in Build History.

## Supported Packaging Modules

The following **22 modules** have been **tested and verified on Android** and are ready for offline packaging configuration:

### Core Features

- **Barcode / QR Scanner** — scan QR codes and barcodes
- **Bluetooth Low Energy (BLE)** — connect and exchange data with BLE devices
- **Camera & Gallery** — take photos and pick images from the gallery
- **Contacts** — read and write address book entries
- **Fingerprint Recognition** — device fingerprint biometric authentication
- **iBeacon** — iBeacon device scanning and proximity detection
- **Video Player** — play local and online video content
- **Audio Recorder** — audio recording
- **Geolocation** — system location, Baidu location, Amap (Gaode) location, Tencent location (with auto permission injection and multi-provider support)
- **Maps** — Amap (Gaode) maps, Baidu maps, Google maps (with vue/nvue page type switching)
- **Speech Recognition** — iFlytek speech, Baidu speech, Aliyun speech recognition
- **gcanvas** — Weex Canvas binding graphics engine
- **SQLite Database** — local SQLite data storage
- **Messaging (SMS / MMS / Email)** — send SMS, MMS, and email messages
- **X5 WebView Kernel** — Tencent TBS kernel to replace system WebView for better compatibility

### Authentication & Security

- **Face Recognition / Real-person Verification** — face-based identity verification (DCloud / Baidu / Aliyun providers supported)

### Communication

- **Live Pusher** — audio/video live streaming push (Tencent Cloud LiteAVSDK)
- **Push Notifications (uniPush)** — push notifications (supports Xiaomi, Meizu, Huawei, OPPO, vivo, Honor, and other vendor channels)

### Social & Sharing

- **Sharing** — share to WeChat, QQ, Sina Weibo (text, images, links, mini-programs, etc.)

### Login & Authentication

- **Login** — WeChat login, QQ login, Apple Sign-In, one-click carrier login, Xiaomi login, Google login, Facebook login

### Payment

- **Payment** — Alipay payment, WeChat Pay, PayPal, Stripe, Google Pay

### Analytics & Statistics

- **Statistics & Analytics** — Umeng, Tencent MTA, Baidu Analytics, DCloud Stats, Google Firebase

> **Notes:**
>
> - All 22 modules listed above have been verified through actual Android offline build processes.
> - The Advertising module (uni-AD) has template definitions in the codebase but is **untested** and may have uncertainties.
> - **iOS and HarmonyOS module-level packaging configuration is not yet supported**; support will be added in future releases.

## Repository Layout

```text
.
├── src/                  # Vue frontend views, components, and stores
├── src-tauri/            # Tauri/Rust backend commands and app config
├── bundled/              # Bundled templates, such as the Android packaging template
├── public/               # Static assets
├── module-tutorial*.md   # Module integration and offline packaging notes
└── package.json
```

## Security Notes

- Do not commit real certificates, Keystores, P12 files, provisioning profiles, API keys, or signing passwords.
- `.gitignore` excludes common signing files, build artifacts, and local cache directories by default.
- Project config may contain local filesystem paths. Review changes before committing personal or internal paths.

## Contributing

Issues and pull requests are welcome. Before submitting changes, please run:

```bash
npm run typecheck
npm run build
```

If you changed Rust backend code or Tauri configuration, also consider running:

```bash
npm run tauri build
```

## License

This project is open source under the [MIT License](./LICENSE).
