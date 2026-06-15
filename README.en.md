# UniPack Tool

> A desktop automation tool for uni-app offline packaging across Android, iOS, and HarmonyOS.

English | [中文](./README.md)

UniPack Tool is a Tauri + Vue 3 + TypeScript + Rust desktop application. It connects SDK setup, project configuration, resource import, module detection, signing settings, build logs, and artifact management so uni-app offline packaging is easier to repeat and debug.

> Current verification is based mainly on one UniApp project. Module combinations, dependency versions, and `manifest.json` settings may vary across projects. If you hit packaging issues, please report them via [Issues](https://github.com/EarlySummer2018/uniapp-packaging-tool/issues).

## Support Overview

| Platform | Project Source | Artifact | Current Status |
| --- | --- | --- | --- |
| Android | User-configured DCloud Android offline SDK | APK | 22 modules verified through offline builds |
| iOS | User-configured DCloud iOS offline SDK `HBuilder-Hello*` | IPA | 12 modules supported through automation or configuration handling |
| HarmonyOS | User-configured Harmony project template | HAP | Template-based build flow supported; module-level automation is still evolving |

## Core Capabilities

| Capability | Details |
| --- | --- |
| Project management | Create, save, and switch between multiple uni-app packaging projects |
| SDK management | Configure DCloud Android/iOS offline SDKs and Harmony templates, with local environment checks |
| Resource import | Import a local uni-app project or build resource package, read `manifest.json`, and extract app metadata |
| Platform settings | Manage Android, iOS, and HarmonyOS package IDs, Bundle IDs, signing files, output directories, and build settings |
| Module detection | Analyze common DCloud/native modules, UTS plugins, and required module parameters |
| Build center | Start selected platform builds, stream logs, and collect APK/IPA/HAP artifacts |
| Build history | Track build status, duration, version, log paths, and artifact paths |
| Secret handling | Store signing passwords through the system Keychain/credential backend instead of plain config files |

## iOS Support

The iOS build flow reads `HBuilder-Hello*` from the user-configured DCloud iOS offline SDK directory and validates the sibling `SDK/Libs` and `SDK/Bundles` support directories. During a build, the SDK project is copied into a workspace, and later library/resource lookups come from that workspace SDK link or copy.

| iOS Area | Supported Behavior |
| --- | --- |
| Project generation | Copy the DCloud iOS offline SDK `HBuilder-Hello*` project and configure the workspace copy |
| App metadata | Write app name, Bundle ID, version, `marketChannel`, and `control.xml` AppId |
| Resource import | Import UniApp resources into the iOS `Pandora/apps` layout |
| Icons and launch screen | Generate iOS AppIcon assets and register manifest storyboard launch-screen resources |
| Info.plist | Merge privacy descriptions, URL schemes, query schemes, background modes, ATS, Universal Links, and related manifest settings |
| Entitlements | Configure Associated Domains from `manifest.json` |
| Privacy manifest | Verify that the SDK `.xcprivacy` file is included in the Xcode project |
| Signing and export | Install mobileprovision files, import P12 certificates, and run Xcode archive/export to produce an IPA |

## Supported iOS Modules

The following **12 modules** are supported on iOS through automated integration or configuration handling:

| Category | Modules |
| --- | --- |
| Core features | Barcode, Bluetooth, Camera, Contacts, Fingerprint (Face ID), iBeacon, VideoPlayer, Record |
| Location | Geolocation (system/Baidu/Amap) |
| Authentication and security | FacialRecognitionVerify |
| Communication and media | Push (uniPush 2.0), LivePusher |

> Geolocation, Push, FacialRecognitionVerify, and LivePusher include native dependency wiring; Bluetooth and iBeacon include capability/background-mode configuration; the remaining modules are handled mainly through Info.plist / ATS configuration.

> Share, Login, Payment, Map, Speech, Statistic, uni-AD, UIWebview, and similar iOS modules have documentation or template references, but full end-to-end automation is not complete yet. Complex cases may still require manual Xcode configuration based on DCloud's official documentation.

## Verified Android Modules

The following **22 modules** have been verified through Android offline packaging:

| Category | Modules |
| --- | --- |
| Core features | Barcode, Bluetooth, Camera, Contacts, Fingerprint, iBeacon, VideoPlayer, Record, SQLite, Messaging, gcanvas, X5 WebView |
| Location and maps | Geolocation (system/Baidu/Amap/Tencent), Map (Amap/Baidu/Google) |
| Authentication and security | FacialRecognitionVerify (DCloud/Baidu/Aliyun) |
| Communication and media | Push (uniPush and vendor channels), LivePusher |
| Social and accounts | Share (WeChat/QQ/Weibo), Login (WeChat/QQ/Apple/one-click carrier/Xiaomi/Google/Facebook) |
| Payment and analytics | Payment (Alipay/WeChat Pay/PayPal/Stripe/Google Pay), Statistic (Umeng/Tencent MTA/Baidu/DCloud/Firebase) |
| Speech | Speech (iFlytek/Baidu/Aliyun) |

> The Advertising module (uni-AD) has template definitions in the codebase, but it has not been fully tested and may still have uncertainties.

## Tech Stack

| Layer | Technologies |
| --- | --- |
| Desktop | Tauri 2 |
| Frontend | Vue 3, TypeScript, Vite, Pinia, Vue Router, Naive UI |
| Backend | Rust, Tokio, Serde, Reqwest |
| Packaging | DCloud offline SDK, Android Gradle, Xcode, HarmonyOS project templates |

## Requirements

Base development environment:

- Node.js 18+
- npm
- Rust stable
- Tauri 2 system dependencies

Additional platform requirements:

- Android: JDK, Android SDK, SDK Build Tools, Gradle or the project Gradle Wrapper, DCloud Android offline SDK
- iOS: macOS, Xcode, Command Line Tools, DCloud iOS offline SDK; install CocoaPods if your module dependencies require it
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

## Repository Layout

```text
.
├── src/                  # Vue frontend views, components, and stores
├── src-tauri/            # Tauri/Rust backend commands and app config
├── bundled/              # Bundled templates
├── docs/                 # Android / iOS module integration notes
├── public/               # Static assets
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
