# UniPack Tool

> UniApp 离线打包自动化桌面工具，面向 Android、iOS、HarmonyOS 多端构建流程管理。

[English](./README.en.md) | 中文

UniPack Tool 是一个基于 Tauri + Vue 3 + TypeScript + Rust 的桌面应用。它把 uni-app 离线打包中分散的 SDK 配置、项目配置、资源导入、模块识别、签名信息、构建日志和产物管理串联起来，让多端打包流程更清晰、更可复用。

> 当前验证主要基于单个 UniApp 项目完成，不同项目的模块组合、依赖版本或 `manifest.json` 配置可能存在差异。如遇打包问题，欢迎通过 [Issues](https://github.com/EarlySummer2018/uniapp-packaging-tool/issues) 反馈。

## 支持概览

| 平台 | 工程来源 | 构建产物 | 当前状态 |
| --- | --- | --- | --- |
| Android | 用户配置的 DCloud Android 离线 SDK | APK | 22 个模块已完成离线构建验证 |
| iOS | 用户配置的 DCloud iOS 离线 SDK `HBuilder-Hello*` | IPA | 22 个模块已完成离线构建验证 |
| HarmonyOS | 用户配置的 Harmony 工程模板 | HAP | 已支持模板工程构建流程，模块级配置持续完善 |

## 核心能力

| 能力 | 说明 |
| --- | --- |
| 项目管理 | 创建、保存、切换多个 uni-app 打包项目 |
| SDK 管理 | 配置 DCloud Android/iOS 离线 SDK、Harmony 工程模板，并检测本机环境 |
| 资源导入 | 导入本地 uni-app 项目或构建资源，读取 `manifest.json` 并提取应用信息 |
| 平台配置 | 维护 Android、iOS、HarmonyOS 的包名、Bundle ID、签名、输出目录等配置 |
| 模块识别 | 分析常见 DCloud/原生模块、UTS 插件和模块必填参数 |
| 构建中心 | 选择目标平台发起构建，实时查看日志，收集 APK/IPA/HAP 等产物 |
| 历史记录 | 记录构建状态、耗时、版本、日志路径和产物路径 |
| 密钥保护 | 签名密码通过系统 Keychain/凭据能力保存，避免明文写入项目配置 |

## iOS 支持情况

当前 iOS 构建流程已从用户配置的 DCloud iOS 离线 SDK 目录读取 `HBuilder-Hello*` 工程，并校验同级 `SDK/Libs`、`SDK/Bundles` 支持目录。构建时会复制 SDK 工程到工作区，后续所有库、资源和 bundle 查找都来自该工作区的 SDK 链接或副本。

| iOS 能力 | 支持内容 |
| --- | --- |
| 工程生成 | 复制 DCloud iOS 离线 SDK 自带 `HBuilder-Hello*`，配置 workspace 副本 |
| 应用信息 | 写入应用名称、Bundle ID、版本号、`marketChannel`、`control.xml` AppId |
| 资源导入 | 导入 UniApp 资源到 iOS `Pandora/apps` 布局 |
| 图标与启动图 | 生成 iOS AppIcon，支持 manifest storyboard 启动界面资源注册 |
| Info.plist | 合并隐私权限、URL Schemes、白名单、后台模式、ATS、Universal Links 等配置 |
| Entitlements | 根据 manifest 配置 Associated Domains |
| 隐私清单 | 校验 SDK 工程中的 `.xcprivacy` 是否纳入 Xcode 工程 |
| 签名导出 | 安装 mobileprovision，导入 P12，执行 Xcode archive/export 生成 IPA |

## iOS 已验证模块

以下 **22 个模块**已在 iOS 端完成离线打包验证：

| 分类 | 模块 |
| --- | --- |
| 基础能力 | Barcode、Bluetooth、Camera、Contacts、Fingerprint、iBeacon、VideoPlayer、Record、SQLite、Messaging、gcanvas |
| 位置与地图 | Geolocation（系统/百度/高德/腾讯）、Map（高德/百度/Google） |
| 认证与安全 | FacialRecognitionVerify（DCloud/百度/阿里云） |
| 通信与媒体 | Push（uniPush 与厂商通道）、LivePusher |
| 社交与账号 | Share（微信/QQ/微博）、Login（微信/QQ/苹果/一键登录/小米/Google/Facebook） |
| 支付与统计 | Payment（支付宝/微信/PayPal/Stripe/Google Pay）、Statistic（友盟/腾讯MTA/百度/DCloud/Firebase） |
| 语音 | Speech（讯飞/百度/阿里云） |

> 广告模块（uni-AD）代码中已有模板定义，但尚未经完整实测，使用时可能存在不确定性。

## Android 已验证模块

以下 **22 个模块**已在 Android 端完成离线打包验证：

| 分类 | 模块 |
| --- | --- |
| 基础能力 | Barcode、Bluetooth、Camera、Contacts、Fingerprint、iBeacon、VideoPlayer、Record、SQLite、Messaging、gcanvas、X5 WebView |
| 位置与地图 | Geolocation（系统/百度/高德/腾讯）、Map（高德/百度/Google） |
| 认证与安全 | FacialRecognitionVerify（DCloud/百度/阿里云） |
| 通信与媒体 | Push（uniPush 与厂商通道）、LivePusher |
| 社交与账号 | Share（微信/QQ/微博）、Login（微信/QQ/苹果/一键登录/小米/Google/Facebook） |
| 支付与统计 | Payment（支付宝/微信/PayPal/Stripe/Google Pay）、Statistic（友盟/腾讯MTA/百度/DCloud/Firebase） |
| 语音 | Speech（讯飞/百度/阿里云） |

> 广告模块（uni-AD）代码中已有模板定义，但尚未经完整实测，使用时可能存在不确定性。

## 技术栈

| 层 | 技术 |
| --- | --- |
| 桌面框架 | Tauri 2 |
| 前端 | Vue 3、TypeScript、Vite、Pinia、Vue Router、Naive UI |
| 后端 | Rust、Tokio、Serde、Reqwest |
| 打包能力 | DCloud 离线 SDK、Android Gradle、Xcode、HarmonyOS 工程模板 |

## 环境要求

基础开发环境：

- Node.js 18+
- npm
- Rust stable
- Tauri 2 依赖环境

按目标平台额外准备：

- Android：JDK、Android SDK、SDK Build Tools、Gradle 或项目内 Gradle Wrapper、DCloud Android 离线 SDK
- iOS：macOS、Xcode、Command Line Tools、DCloud iOS 离线 SDK；如模块依赖 CocoaPods，请额外安装 CocoaPods
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
├── bundled/              # 内置模板
├── docs/                 # Android / iOS 模块接入参考文档
├── public/               # 静态资源
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

本项目基于 [Apache License 2.0](./LICENSE) 开源。
