# UniPack Tool

> A desktop automation tool for uni-app offline packaging across Android, iOS, and HarmonyOS.

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
