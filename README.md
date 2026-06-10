# UniPack Tool

> UniApp 离线打包自动化桌面工具，支持 Android、iOS、HarmonyOS 多端构建流程管理。

> - 当前工具验证基于单个 UniApp 项目完成，不同项目的模块组合、依赖版本或 manifest 配置可能存在差异。**如遇打包问题欢迎提交 [Issues](https://github.com/EarlySummer2018/uniapp-packaging-tool/issues) 反馈，项目将持续优化完善。**

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

## 支持的打包模块

以下 **22 个模块**已在 **Android 端完成测试验证**，可正常用于离线打包配置：

### 基础能力

- **扫码（Barcode）** — 二维码/条形码扫描
- **低功耗蓝牙（BLE）** — 蓝牙设备连接与数据交互
- **相机&相册** — 相机拍照与图片选取
- **通讯录** — 读取和写入联系人信息
- **指纹识别** — 设备指纹生物认证
- **iBeacon** — iBeacon 设备扫描与距离检测
- **视频播放** — 本地及在线视频播放
- **录音** — 音频录制
- **定位（Geolocation）** — 系统定位、百度定位、高德定位、腾讯定位（支持权限自动注入与多供应商配置）
- **地图（Map）** — 高德地图、百度地图、Google 地图（支持 vue/nvue 页面类型切换）
- **语音识别（Speech）** — 讯飞语音、百度语音、阿里云语音识别
- **gcanvas 画布** — Weex Canvas 绑定绘图引擎
- **SQLite 数据库** — 本地 SQLite 数据存储
- **短彩邮件消息** — 短信、彩信与邮件发送
- **X5 WebView 内核** — 腾讯 TBS 内核，替代系统 WebView 提升兼容性

### 认证与安全

- **实人认证 / 人脸识别** — 人脸实名核身（支持 DCloud / 百度 / 阿里云）

### 通信

- **直播推流** — 音视频直播推流（腾讯云 LiteAVSDK）
- **消息推送（uniPush）** — 推送通知（支持小米、魅族、华为、OPPO、vivo、荣耀等厂商通道）

### 社交与分享

- **分享** — 微信、QQ、新浪微博分享（文本、图片、链接、小程序等）

### 登录鉴权

- **登录** — 微信登录、QQ登录、苹果登录、一键登录、小米登录、Google登录、Facebook登录

### 支付

- **支付** — 支付宝支付、微信支付、PayPal、Stripe、Google Pay

### 统计分析

- **统计分析** — 友盟统计、腾讯MTA、百度统计、DCloud统计、Google Firebase

> **说明：**
>
> - 以上 22 个模块均已通过 Android 离线打包的实际构建验证。
> - 验证基于单个 UniApp 项目完成，不同项目的模块组合、依赖版本或 manifest 配置可能存在差异。**如遇打包问题欢迎提交 [Issues](https://github.com/EarlySummer2018/uniapp-packaging-tool/issues) 反馈，我们将持续优化完善。**
> - 广告模块（uni-AD）代码中已有模板定义，但**尚未经实测**，使用时可能存在不确定性。
> - **iOS 与 HarmonyOS 端暂不支持模块级打包配置**，后续版本将逐步完善。

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
