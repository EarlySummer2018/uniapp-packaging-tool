# UniPack Tool

> UniApp 离线打包自动化桌面工具，支持 Android、iOS、HarmonyOS 多端构建流程管理。

[English](./README.en.md) | 中文

UniPack Tool 是一个基于 Tauri + Vue 3 + TypeScript + Rust 的桌面应用，目标是把 uni-app 离线打包过程中分散的 SDK 配置、项目配置、资源导入、模块识别、签名信息、构建日志和产物管理串起来，让多端打包流程更清晰、更可复用。

## 功能特性

- 项目管理：创建、保存、切换多个 uni-app 打包项目。
- 多端配置：维护 Android、iOS、HarmonyOS 的包名、Bundle ID、签名、输出目录等配置。
- 资源导入：导入本地 uni-app 项目或构建资源，读取 `manifest.json` 并提取应用信息。
- 模块识别：分析常见 DCloud/原生模块、UTS 插件和 Android 模块必填参数。
- SDK 管理：配置 DCloud Android/iOS 离线 SDK、Harmony 工程模板，并检测本机打包环境。
- 构建中心：选择目标平台发起构建，实时查看日志，收集 APK/IPA/HAP 等产物。
- 历史记录：记录构建状态、耗时、版本、日志路径和产物路径，便于排查和追踪。
- 密钥保护：签名密码通过系统 Keychain/凭据能力保存，避免明文写入项目配置。

## 技术栈

- 桌面框架：Tauri 2
- 前端：Vue 3、TypeScript、Vite、Pinia、Vue Router、Naive UI
- 后端：Rust、Tokio、Serde、Reqwest
- 打包能力：DCloud 离线 SDK、Android Gradle、Xcode、HarmonyOS 工程模板

## 环境要求

基础开发环境：

- Node.js 18+
- npm
- Rust stable
- Tauri 2 依赖环境

按目标平台额外准备：

- Android：JDK、Android SDK、SDK Build Tools、Gradle 或项目内 Gradle Wrapper、DCloud Android 离线 SDK
- iOS：macOS、Xcode、Command Line Tools、CocoaPods、DCloud iOS 离线 SDK
- HarmonyOS：DevEco Studio / HarmonyOS SDK、可用的 Harmony 工程模板

## 快速开始

```bash
git clone https://github.com/EarlySummer2018/uniapp-packaging-tool.git
cd uniapp-packaging-tool
npm install
npm run tauri dev
```

常用命令：

```bash
npm run dev        # 仅启动 Vite 前端
npm run build      # 构建前端资源
npm run typecheck  # TypeScript 类型检查
npm run tauri dev  # 启动 Tauri 开发应用
npm run tauri build
```

## 使用流程

1. 在「SDK 管理」中配置 DCloud 离线 SDK、Harmony 工程模板，并检查本机环境。
2. 在首页创建项目，填写项目名称和输出目录。
3. 在「项目配置」中配置 Android/iOS/HarmonyOS 的包名、证书、签名和平台参数。
4. 在「构建中心」导入 uni-app 项目资源或资源包，读取 `manifest.json`。
5. 根据检测到的模块补齐必填配置，选择目标平台并开始构建。
6. 在「打包历史」中查看构建结果、日志和产物路径。

## 仓库结构

```text
.
├── src/                  # Vue 前端页面、组件、状态管理
├── src-tauri/            # Tauri/Rust 后端命令与应用配置
├── bundled/              # 内置模板，例如 Android 离线打包模板
├── public/               # 静态资源
├── module-tutorial*.md   # 模块接入与离线打包参考文档
└── package.json
```

## 安全说明

- 请不要将真实证书、Keystore、P12、Provisioning Profile、API Key 或签名密码提交到仓库。
- `.gitignore` 已默认忽略常见签名文件、安装包和本地缓存目录。
- 项目配置可能包含本机路径，请在提交前确认没有泄露个人或公司内部路径。

## 贡献

欢迎提交 Issue 和 Pull Request。建议在提交前至少运行：

```bash
npm run typecheck
npm run build
```

如果修改了 Rust 后端或 Tauri 配置，也建议运行：

```bash
npm run tauri build
```

## 许可证

本项目基于 [MIT License](./LICENSE) 开源。
