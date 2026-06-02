# UniApp 离线打包自动化工具 — 完整开发方案

**工具定位**：把官方离线打包教程里所有手动步骤自动化，用户配置一次，每次只需导入资源即可打包。  
**工具本身**：Tauri 桌面应用（支持 Mac + Windows 运行）  
**打包目标**：Android APK、iOS IPA（仅 macOS）、鸿蒙 HAP  
**用户角色**：开发者，本机已安装 Android Studio / Xcode / DevEco Studio  

---

## 一、用户需要做什么 vs 工具自动做什么

### 1.1 用户只需配置一次（首次创建项目时）

| 配置项 | 说明 |
|--------|------|
| **离线 SDK 路径** | 手动下载 DCloud 对应版本 SDK 解压后，填入路径 |
| **DCloud AppKey** | 在 dev.dcloud.net.cn 申请，填入 |
| **应用名称** | 桌面/桌面显示的名字 |
| **版本名称 / 版本号** | 如 1.0.0 / 1 |
| **应用图标** | 提供一张 1024×1024 PNG，工具自动生成各平台所有尺寸 |
| **Android 包名** | 如 com.example.myapp |
| **Android 签名 Keystore** | .jks 文件路径、key alias、两个密码 |
| **iOS Bundle Identifier** | 如 com.example.myapp（仅 macOS） |
| **iOS Team ID** | Apple Developer 账号的 Team ID（仅 macOS） |
| **iOS 描述文件** | .mobileprovision 文件路径（仅 macOS） |
| **iOS 证书 .p12** | 导出的发布证书路径 + 密码（仅 macOS） |
| **输出目录** | APK/IPA 最终存放位置 |

> 以上配置填写一次，持久保存。之后每次打包**不需要重新配置**。

---

### 1.2 每次打包用户只做一件事

1. 在 HBuilderX 点击「发行 → 原生App-本地打包 → 生成本地打包App资源」
2. 将生成的 `__UNI__XXXXX` 文件夹**拖入工具**
3. 勾选目标平台，点击**「开始打包」**

---

### 1.3 工具自动完成的全部操作

#### Android（对应官方教程的全部手动步骤）

| # | 官方教程要求手动做的操作 | 工具自动处理 |
|---|--------------------------|--------------|
| 1 | 将 6 个 `.aar` 文件拷贝到 `libs/` | ✅ 从 SDK 路径自动复制 |
| 2 | `build.gradle` 中添加所有 `implementation` 依赖 | ✅ 自动生成完整 `build.gradle` |
| 3 | `build.gradle` 配置 applicationId、版本号、minSdk 等 | ✅ 从项目配置自动填入 |
| 4 | `build.gradle` 配置签名 `signingConfigs` | ✅ 从项目配置自动填入 |
| 5 | `gradle.properties` 添加 AndroidX 配置 | ✅ 自动写入 |
| 6 | `AndroidManifest.xml` 写入 `dcloud_appkey` meta-data | ✅ 自动注入 |
| 7 | `AndroidManifest.xml` 配置 PandoraEntry / PandoraEntryActivity | ✅ 内置模板已包含 |
| 8 | `AndroidManifest.xml` 删除默认 MainActivity | ✅ 模板中已处理 |
| 9 | `AndroidManifest.xml` 配置 FileProvider（替换包名） | ✅ 自动替换包名占位符 |
| 10 | `strings.xml` 写入应用名称 | ✅ 自动写入 |
| 11 | 将 `SDK/assets/data/` 拷贝到工程 assets | ✅ 从 SDK 路径自动复制 |
| 12 | 将 `__UNI__XXXXX` 拷贝到 `assets/apps/` | ✅ 自动复制导入的资源 |
| 13 | 修改 `dcloud_control.xml` 的 appid | ✅ 自动替换为导入资源的 appid |
| 14 | 图标放入 6 个 drawable 目录（ldpi/mdpi/hdpi/xhdpi/xxhdpi/xxxhdpi） | ✅ 从 1024px 图标自动生成所有尺寸 |
| 15 | 执行 `./gradlew assembleRelease` | ✅ 自动执行，实时输出日志 |
| 16 | 在 build 目录深处找到 APK | ✅ 自动复制到用户指定输出目录 |
| — | **uts 插件（如有）** | |
| 17 | 检查资源是否含 uni_modules/ | ✅ 导入时自动扫描检测 |
| 18 | 复制 `utsplugin-release.aar`，追加 Kotlin/OkHttp 依赖到 build.gradle | ✅ 自动完成 |
| 19 | 追加 jitpack 仓库到根 build.gradle | ✅ 自动完成 |
| 20 | 识别内置模块，复制对应 aar，追加线上依赖，处理模块间依赖 | ✅ 自动完成 |
| 21 | 导入自定义 uts 插件 Module，解析 config.json，配置 settings/build.gradle | ✅ 自动完成 |

#### iOS（对应官方教程的全部手动步骤，仅 macOS 可执行）

| # | 官方教程要求手动做的操作 | 工具自动处理 |
|---|--------------------------|--------------|
| 1 | 解压 SDK，打开 `HBuilder-Hello` 工程 | ✅ 从 SDK 路径自动读取 |
| 2 | `Info.plist` 写入 `dcloud_appkey` | ✅ 自动注入 |
| 3 | Xcode General → 修改 Bundle Identifier | ✅ 自动修改 `project.pbxproj` |
| 4 | 修改 Display Name（应用名称） | ✅ 自动修改 |
| 5 | 修改 Version（版本名称） | ✅ 自动修改 |
| 6 | 修改 Build（版本号） | ✅ 自动修改 |
| 7 | 将各尺寸图标拖入 `Assets.xcassets/AppIcon.appiconset` | ✅ 从 1024px 图标自动生成全部 12 个尺寸并更新 Contents.json |
| 8 | 将 `__UNI__XXXXX` 拷贝到 `Pandora/apps/` | ✅ 自动复制 |
| 9 | 修改 `Pandora/control.xml` 的 appid | ✅ 自动替换 |
| 10 | 安装 `.mobileprovision` 描述文件 | ✅ 自动安装到 `~/Library/MobileDevice/Provisioning Profiles/` |
| 11 | Xcode Product → Archive | ✅ 调用 `xcodebuild archive` |
| 12 | Archive → Distribute App → 导出 IPA | ✅ 调用 `xcodebuild -exportArchive` |
| 13 | 在导出目录找到 IPA | ✅ 自动复制到用户指定输出目录 |
| — | **uts 插件（如有）** | |
| 14 | 添加 DCUniBase.framework + DCloudUTSFoundation.framework | ✅ 自动修改 project.pbxproj |
| 15 | 移除旧库（liblibPDRCore.a 等 6 个，DCUniBase 已包含） | ✅ 自动完成 |
| 16 | 若有内置模块，添加 DCloudUTSExtAPI.framework | ✅ 自动完成 |
| 17 | 将自定义插件的 .xcframework 添加为 Embed & Sign | ✅ 自动完成 |

---

## 二、uts 插件完整处理方案

uts 插件分三个层次，工具需要**全部自动处理**，无需用户额外配置。

### 2.1 三个层次的关系

```
项目是否使用了 uts 插件？
│
├─ 判断依据：导入的资源目录中是否存在 uni_modules/ 子目录
│            (HBuilderX 4.18+ 导出时自动生成)
│
└─ 是 → 需要处理以下三层
         │
         ├─ 层次1：UTS 基础运行时（必须，有 uts 插件就要集成）
         │         Android: utsplugin-release.aar + 6 条额外依赖 + jitpack 仓库
         │         iOS:     DCUniBase.framework + DCloudUTSFoundation.framework
         │                  同时移除 liblibPDRCore.a 等旧库（已包含在内）
         │
         ├─ 层次2：UTS 内置模块（按需，看 uni_modules 里有哪些）
         │         Android: 对应的 uni-xxx-release.aar + 线上依赖
         │         iOS:     DCloudUTSExtAPI.framework（13个内置模块共用一个）
         │
         └─ 层次3：自定义 uts 插件源码/产物
                   Android: uni_modules/ 目录整体导入工程（作为子模块或直接引用）
                   iOS:     uni_modules/ 中的 .framework / .xcframework 导入工程
```

### 2.2 工具如何自动检测和处理

#### 第一步：导入资源时自动扫描

```
用户拖入 __UNI__XXXXX 资源文件夹后，工具立即扫描：

资源目录结构（HBuilderX 4.18+ 导出）：
__UNI__ABCD1234/
├── manifest.json           ← 读取 appid、版本、HBuilderX 版本
├── www/                    ← HTML/JS/CSS 前端资源
└── uni_modules/            ← 【存在则说明有 uts 插件】
    ├── uni-getNetworkType/ ← UTS 内置模块
    │   └── utssdk/
    │       └── app-android/  或  app-ios/
    ├── uni-storage/
    └── my-custom-plugin/   ← 自定义 uts 插件
        └── utssdk/
            ├── app-android/
            │   ├── index.uts       ← 源码（需要 Kotlin 编译）
            │   └── config.json     ← 声明该插件的本地/线上依赖
            └── app-ios/
                └── xxx.xcframework ← 已编译的 iOS 产物
```

扫描结果数据结构：
```rust
pub struct UtsPluginScanResult {
    pub has_uts_plugins: bool,
    pub builtin_modules: Vec<UtsBuiltinModule>,  // 识别出的内置模块列表
    pub custom_plugins:  Vec<UtsCustomPlugin>,    // 自定义插件列表
}

pub struct UtsBuiltinModule {
    pub name: String,         // 如 "uni-getNetworkType"
    pub local_aar: String,    // 对应的 aar 文件名
    pub online_deps: Vec<String>, // 线上 Maven 依赖
    pub depends_on: Vec<String>,  // 依赖的其他内置模块
}

pub struct UtsCustomPlugin {
    pub id: String,           // 插件 id，如 "my-custom-plugin"
    pub android_dir: PathBuf, // app-android/ 目录路径
    pub ios_dir: PathBuf,     // app-ios/ 目录路径
    pub android_deps: Vec<String>, // 从 config.json 解析的依赖
}
```

#### 第二步：Android — uts 插件处理（自动，无需用户操作）

```
检测到 has_uts_plugins = true 时，在原有 Android 构建流程中自动追加：

Step A  注入 UTS 基础运行时
        └─ 从 SDK/libs/ 复制 utsplugin-release.aar 到工作区 app/libs/
        └─ build.gradle 追加依赖：
           implementation "com.squareup.okhttp3:okhttp:3.12.12"
           implementation "androidx.core:core-ktx:1.6.0"
           implementation "org.jetbrains.kotlin:kotlin-stdlib:2.2.0"
           implementation "org.jetbrains.kotlin:kotlin-reflect:2.2.0"
           implementation "org.jetbrains.kotlinx:kotlinx-coroutines-core:1.8.1"
           implementation "org.jetbrains.kotlinx:kotlinx-coroutines-android:1.8.1"
           implementation "com.github.getActivity:XXPermissions:18.63"
        └─ 根目录 build.gradle 追加 jitpack 仓库：
           maven { url 'https://jitpack.io' }

Step B  注入 UTS 内置模块（按 uni_modules 目录实际包含的模块）
        └─ 对照内置模块对应表（见下），自动：
           · 将对应 aar 从 SDK/libs/ 复制到工作区 app/libs/
           · 在 build.gradle 追加对应线上依赖
           · 若该模块依赖其他模块（如 uni-chooseMedia 依赖 uni-prompt），
             自动递归处理依赖模块

Step C  导入自定义 uts 插件
        └─ 将 uni_modules/{plugin-id}/utssdk/app-android/ 整目录
           复制到工作区，作为 Android library module 引入
        └─ 解析 app-android/config.json，提取本地依赖（aar）和线上依赖
        └─ 本地依赖 aar 复制到 app/libs/
        └─ 线上依赖追加到 build.gradle dependencies
        └─ 在 settings.gradle 中 include 该 module
        └─ 在 app/build.gradle 中添加 implementation project(':plugin-id')
```

**UTS 内置模块对照表（工具内置，自动匹配）：**

| uni_modules 目录名 | 需复制的 aar | 需追加的线上依赖 | 依赖模块 |
|---|---|---|---|
| uni-createRequestPermissionListener | uni-createRequestPermissionListener-release.aar | — | — |
| uni-getNetworkType | uni-getNetworkType-release.aar | — | — |
| uni-installApk | uni-installApk-release.aar | — | — |
| uni-network | uni-network-release.aar | okhttp:3.12.12 | — |
| uni-privacy | uni-privacy-release.aar | — | — |
| uni-chooseMedia | uni-chooseMedia-release.aar | appcompat:1.6.1, activity-ktx:1.9.2 | uni-prompt |
| uni-getAppBaseInfo | uni-getAppBaseInfo-release.aar | — | — |
| uni-storage | uni-storage-release.aar | — | — |
| uni-getSystemInfo | uni-getSystemInfo-release.aar | — | — |
| uni-getDeviceInfo | uni-getDeviceInfo-release.aar | — | — |
| uni-openAppAuthorizeSetting | uni-openAppAuthorizeSetting-release.aar | — | — |
| uni-exit | uni-exit-release.aar | — | — |
| uni-getAccessibilityInfo | uni-getAccessibilityInfo-release.aar | — | — |
| uni-getAppAuthorizeSetting | uni-getAppAuthorizeSetting-release.aar | — | — |
| uni-getSystemSetting | uni-getSystemSetting-release.aar | — | — |
| uni-prompt | uni-prompt-release.aar | recyclerview:1.0.0, appcompat:1.0.0 | — |
| uni-getLocation-tencent-uni1 | uni-getLocation-tencent-uni1-release.aar | TencentLocationSdk:7.5.4.8 | — |

#### 第三步：iOS — uts 插件处理（自动，无需用户操作）

```
检测到 has_uts_plugins = true 时，在原有 iOS 构建流程中自动追加：

Step A  注入 UTS 基础运行时（替换旧库）
        └─ 从 SDK 中添加：
           · DCUniBase.framework（Embed & Sign）
           · DCloudUTSFoundation.framework（Embed & Sign）
        └─ 从工程中移除（DCUniBase 内已包含，避免重复引用）：
           · liblibPDRCore.a
           · liblibWeex.a
           · libcoreSupport.a
           · storage.framework
           · libSDWebImage.a
           · KSCrash.framework
        └─ 通过修改 project.pbxproj 完成上述添加/移除

Step B  注入 UTS 内置模块（若 uni_modules 中有内置模块）
        └─ 添加 DCloudUTSExtAPI.framework（Embed & Sign）
           （iOS 侧所有 13 个内置模块共用同一个 framework，有则加，无则不加）

Step C  导入自定义 uts 插件产物
        └─ 将 uni_modules/{plugin-id}/utssdk/app-ios/ 下的
           .framework 或 .xcframework 文件复制到工作区
        └─ 在 project.pbxproj 中将其添加为 Embed & Sign 的 framework
```

### 2.3 用户界面：资源扫描结果展示

用户导入资源后，在构建中心展示扫描摘要，让用户清楚工具将做什么：

```
┌──────────────────────────────────────────────────────────────┐
│  ✅ 已导入：__UNI__ABCD1234  (HBuilderX v4.41)               │
│                                                              │
│  🔍 资源扫描结果：                                           │
│  ├─ ✅ 检测到 uts 插件，将自动处理                           │
│  ├─ 📦 UTS 基础运行时                    [自动注入]          │
│  ├─ 🧩 内置模块：uni-getNetworkType      [自动注入]          │
│  ├─ 🧩 内置模块：uni-storage             [自动注入]          │
│  └─ 🔌 自定义插件：my-custom-plugin      [自动导入]          │
└──────────────────────────────────────────────────────────────┘
```

### 2.4 核心设计：为什么完全自动，无需用户配置

- **内置模块识别**：通过 `uni_modules/` 目录名与内置对照表匹配，100% 自动
- **自定义插件依赖**：通过解析插件内的 `config.json` 自动提取所有依赖声明
- **iOS framework 处理**：uts 插件的 iOS 产物是已编译的 `.xcframework`，直接复制引入即可，无需编译
- **Android 插件模块**：插件的 `app-android/` 目录可直接作为 Gradle module 导入，工具自动处理 `settings.gradle` 和 `build.gradle` 的 include/implementation 声明

---

## 三、工具架构

```
┌─────────────────────────────────────────────────────────────────┐
│           UniApp 离线打包工具（Tauri 2.x 桌面应用）              │
│                                                                   │
│  前端 Vue3                                                        │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌────────────────┐  │
│  │ 项目管理  │  │ 资源导入  │  │ 构建中心  │  │  实时日志面板  │  │
│  └──────────┘  └──────────┘  └──────────┘  └────────────────┘  │
│                      │ Tauri Command（IPC）                      │
│  后端 Rust                                                        │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌────────────────┐  │
│  │ 项目配置  │  │ Android  │  │   iOS    │  │    鸿蒙        │  │
│  │  管理    │  │  构建器  │  │  构建器  │  │    构建器      │  │
│  └──────────┘  └────┬─────┘  └────┬─────┘  └───────┬────────┘  │
└───────────────────── ┼─────────────┼────────────────┼───────────┘
                       ▼             ▼                 ▼
                  gradlew       xcodebuild          hvigorw
                  (Gradle)      (仅 macOS)         (DevEco)
                       │             │                 │
                       ▼             ▼                 ▼
                    .apk          .ipa              .hap
```

---

## 三、工程目录结构

```
uniapp-pack-tool/
│
├── src-tauri/                        # Rust 后端
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── src/
│       ├── main.rs                   # Tauri 入口，注册所有 Command
│       ├── commands/
│       │   ├── project.rs            # 项目 CRUD
│       │   ├── build_android.rs      # Android 完整构建流程
│       │   ├── build_ios.rs          # iOS 完整构建流程（macOS only）
│       │   ├── build_harmony.rs      # 鸿蒙完整构建流程
│       │   └── env_check.rs          # 环境检测
│       └── utils/
│           ├── xml.rs                # XML 精准读写（AndroidManifest / dcloud_control.xml）
│           ├── plist.rs              # iOS plist / pbxproj 修改
│           ├── icon.rs               # 图标多尺寸生成
│           ├── process.rs            # 子进程 + 实时日志推送
│           ├── template.rs           # 模板文件变量替换
│           └── keychain.rs           # 密码存入系统 Keychain（不明文写配置文件）
│
├── src/                              # Vue3 前端
│   ├── views/
│   │   ├── ProjectList.vue           # 首页，项目列表
│   │   ├── ProjectConfig.vue         # 项目配置（4 个 Tab：基础/Android/iOS/鸿蒙）
│   │   ├── BuildCenter.vue           # 构建中心（导入资源 + 平台选择 + 执行）
│   │   └── EnvCheck.vue              # 环境检测页
│   └── components/
│       ├── LogPanel.vue              # 实时日志滚动面板
│       ├── DropZone.vue              # 拖拽导入资源
│       └── ArtifactPanel.vue         # 构建产物展示
│
└── bundled/                          # 工具内置的原生工程模板
    ├── android-template/             # 预配置好的 Android 工程骨架
    │   ├── app/
    │   │   ├── build.gradle.tpl      # 含占位变量的模板
    │   │   ├── src/main/
    │   │   │   ├── AndroidManifest.xml.tpl
    │   │   │   └── res/values/strings.xml.tpl
    │   │   └── proguard-rules.pro
    │   └── gradle.properties.tpl
    └── ios-template/                 # HBuilder-Hello 工程骨架（来自 DCloud SDK）
        └── HBuilder-Hello/
```

---

## 四、项目配置数据结构

存储在 `~/.unipack/projects/{id}/config.json`，**首次填写，永久复用**。

```jsonc
{
  "id": "uuid-xxxx",
  "name": "我的App",            // 工具内项目列表显示名，不影响打包

  // ── 公共配置（Android / iOS 共用）──────────────────────────────
  "app": {
    "name":          "我的App",          // 应用名称，写入 strings.xml / Display Name
    "appId":         "__UNI__ABCD1234",  // 由导入资源时自动读取
    "dcloudAppKey":  "xxxx",             // DCloud 开发者中心申请
    "version":       "1.0.0",            // 版本名称
    "versionCode":   1,                  // 版本号（整数）
    "icon1024":      "/path/to/icon.png" // 源图标，工具自动生成各平台所需尺寸
  },

  // ── Android 配置 ────────────────────────────────────────────────
  "android": {
    "enabled":           true,
    "sdkPath":           "/path/to/android-offline-sdk", // 解压后的 SDK 根目录
    "packageName":       "com.example.myapp",
    "minSdkVersion":     21,
    "targetSdkVersion":  30,
    "compileSdkVersion": 35,
    "keystore": {
      "path":  "/path/to/release.jks",
      "alias": "mykey"
      // storePassword / keyPassword 不存这里，走系统 Keychain
    }
  },

  // ── iOS 配置（仅 macOS）─────────────────────────────────────────
  "ios": {
    "enabled":              true,
    "sdkPath":              "/path/to/ios-offline-sdk",  // 解压后的 SDK 根目录
    "bundleId":             "com.example.myapp",
    "teamId":               "XXXXXXXXXX",
    "provisioningProfile":  "/path/to/app.mobileprovision",
    "certificate":          "/path/to/distribution.p12",
    "exportMethod":         "app-store"  // app-store / ad-hoc / enterprise
    // certificatePassword 走系统 Keychain
  },

  // ── 鸿蒙配置 ────────────────────────────────────────────────────
  "harmony": {
    "enabled":        false,
    "sdkPath":        "/path/to/harmony-offline-sdk",
    "bundleName":     "com.example.myapp",
    "runtimeVersion": "4.31.0"
  },

  // ── 输出 ────────────────────────────────────────────────────────
  "outputDir": "~/Desktop/unipack-output"
}
```

---

## 五、核心构建流程详解

### 5.1 Android 构建流程（`build_android.rs`）

```
输入：项目配置 + 导入的 __UNI__XXXXX 资源文件夹
输出：签名好的 release APK
```

```
Step 1  创建独立工作区
        └─ ~/.unipack/workspace/{project-id}/android-{timestamp}/
           （从 bundled/android-template 复制，每次构建互不干扰）

Step 2  注入 SDK 库文件
        └─ 将 {sdkPath}/SDK/libs/ 下的 6 个基础 .aar 复制到工作区 app/libs/
           · lib.5plus.base-release.aar
           · android-gif-drawable-release@1.2.23.aar
           · uniapp-v8-release.aar
           · oaid_sdk_1.0.25.aar
           · install-apk-release.aar
           · breakpad-build-release.aar

Step 2.5  【uts 插件自动处理】扫描 uni_modules/，按需注入
          └─ 若导入的资源目录中不存在 uni_modules/ → 直接跳过
          └─ 若存在 uni_modules/ →

             2.5.1  注入 UTS 基础运行时（有任何 uts 插件都必须）
                    · 复制 SDK/libs/utsplugin-release.aar 到 app/libs/
                    · 记录需追加到 build.gradle 的依赖（Step 3 统一写入）：
                        okhttp:3.12.12 / core-ktx:1.6.0 / kotlin-stdlib:2.2.0
                        kotlin-reflect:2.2.0 / coroutines-core:1.8.1
                        coroutines-android:1.8.1 / XXPermissions:18.63
                    · 记录根 build.gradle 需添加 jitpack 仓库（Step 3 写入）

             2.5.2  处理 UTS 内置模块（对照内置模块表自动匹配）
                    · 遍历 uni_modules/ 子目录名，与内置模块对照表匹配
                    · 将命中的 uni-xxx-release.aar 从 SDK/libs/ 复制到 app/libs/
                    · 记录对应线上 Maven 依赖（Step 3 写入）
                    · 自动递归处理模块间依赖关系
                      （如 uni-chooseMedia 自动补充 uni-prompt）

             2.5.3  处理自定义 uts 插件（非内置模块的其余 uni_modules 条目）
                    · 将 uni_modules/{id}/utssdk/app-android/ 整体复制到
                      工作区根目录下，作为独立 Gradle Module
                    · 解析 app-android/config.json，提取：
                        - localDependencies（本地 aar）→ 复制到 app/libs/
                        - remoteDependencies（线上库）→ 记录到 Step 3 写入
                    · 在 settings.gradle 追加 include ':{plugin-id}'
                    · 在 app/build.gradle 追加 implementation project(':{plugin-id}')

Step 3  生成 build.gradle
        └─ 渲染 build.gradle.tpl，填入：
           · applicationId ← config.android.packageName
           · compileSdkVersion / minSdkVersion / targetSdkVersion
           · versionCode / versionName
           · signingConfigs（keystore 路径、alias、两个密码从 Keychain 读取）
           · packagingOptions { jniLibs { useLegacyPackaging true } }（targetSdk≥34 时）
           · aaptOptions（uni-app 必需）
           · 全部 dependencies implementation 声明

Step 4  生成 gradle.properties
        └─ 写入 android.useAndroidX=true / android.enableJetifier=true

Step 5  配置 AndroidManifest.xml
        └─ 渲染 AndroidManifest.xml.tpl，填入：
           · package = config.android.packageName
           · dcloud_appkey meta-data = config.app.dcloudAppKey
           · PandoraEntry / PandoraEntryActivity（含 configChanges 适配折叠屏+暗黑模式）
           · FileProvider authorities = {packageName}.dc.fileprovider
           · application icon / label

Step 6  写入应用名称
        └─ strings.xml 中 app_name = config.app.name

Step 7  复制 SDK assets/data
        └─ {sdkPath}/SDK/assets/data/ → 工作区 app/src/main/assets/data/

Step 8  导入 UniApp 资源
        └─ 导入的 __UNI__XXXXX/ → 工作区 app/src/main/assets/apps/__UNI__XXXXX/

Step 9  修改 dcloud_control.xml
        └─ 将 appid 属性替换为导入资源的 appid（即文件夹名 __UNI__XXXXX）
           三者必须一致：control.xml appid = apps 文件夹名 = manifest.json id

Step 10 生成多尺寸图标
        └─ 从 config.app.icon1024 生成：
           drawable-ldpi/icon.png    (36×36)
           drawable-mdpi/icon.png    (48×48)
           drawable-hdpi/icon.png    (72×72)
           drawable-xhdpi/icon.png   (96×96)
           drawable-xxhdpi/icon.png  (144×144)
           drawable-xxxhdpi/icon.png (192×192)
           同时生成 push.png（同尺寸）和 splash.png

Step 11 执行 Gradle 构建
        └─ 工作区内执行：./gradlew assembleRelease --stacktrace
           环境变量注入：ANDROID_HOME、JAVA_HOME
           stdout/stderr 实时流式推送到前端日志面板

Step 12 收集产物
        └─ 工作区 app/build/outputs/apk/release/app-release.apk
           → 复制到 config.outputDir/{appName}-{version}.apk
```

---

### 5.2 iOS 构建流程（`build_ios.rs`，仅 macOS）

```
输入：项目配置 + 导入的 __UNI__XXXXX 资源文件夹
输出：可发布的 IPA 文件
```

```
Step 1  创建独立工作区
        └─ 从 {sdkPath}/HBuilder-Hello 整体复制到工作区
           （每次构建使用全新副本，互不干扰）

Step 2  修改 Bundle Identifier
        └─ 修改 HBuilder-Hello.xcodeproj/project.pbxproj
           将所有 PRODUCT_BUNDLE_IDENTIFIER = io.dcloud.HBuilder
           替换为 config.ios.bundleId

Step 3  修改 Info.plist
        └─ 写入 dcloud_appkey = config.app.dcloudAppKey（String 类型）
        └─ CFBundleDisplayName = config.app.name
        └─ CFBundleShortVersionString = config.app.version
        └─ CFBundleVersion = config.app.versionCode

Step 4  导入 UniApp 资源
        └─ 导入的 __UNI__XXXXX/ → 工作区 HBuilder-Hello/Pandora/apps/__UNI__XXXXX/

Step 5  修改 control.xml
        └─ 工作区 Pandora/control.xml 中 appid 替换为导入资源的 appid

Step 5.5  【uts 插件自动处理】扫描 uni_modules/，按需注入
          └─ 若导入的资源目录中不存在 uni_modules/ → 直接跳过
          └─ 若存在 uni_modules/ →

             5.5.1  注入 UTS 基础运行时（替换旧库）
                    · 在 project.pbxproj 中添加（Embed & Sign）：
                        DCUniBase.framework
                        DCloudUTSFoundation.framework
                    · 从 project.pbxproj 中移除（DCUniBase 已内含，避免重复）：
                        liblibPDRCore.a / liblibWeex.a / libcoreSupport.a
                        storage.framework / libSDWebImage.a / KSCrash.framework

             5.5.2  处理 UTS 内置模块
                    · 若 uni_modules/ 中有任何内置模块名（uni-getNetworkType 等 13 个）
                    · 在 project.pbxproj 中添加 DCloudUTSExtAPI.framework（Embed & Sign）
                      （iOS 侧所有 13 个内置模块共用同一个 framework）

             5.5.3  处理自定义 uts 插件产物
                    · 遍历 uni_modules/{id}/utssdk/app-ios/
                    · 将 .framework 或 .xcframework 文件复制到工作区
                    · 在 project.pbxproj 中将其添加为 Embed & Sign 的 framework

Step 6  生成 iOS 图标（12 个尺寸）
        └─ 从 config.app.icon1024 生成：
           Icon-20@2x.png    (40×40)
           Icon-20@3x.png    (60×60)
           Icon-29@2x.png    (58×58)
           Icon-29@3x.png    (87×87)
           Icon-40@2x.png    (80×80)
           Icon-40@3x.png    (120×120)
           Icon-60@2x.png    (120×120)
           Icon-60@3x.png    (180×180)
           Icon-76.png       (76×76)
           Icon-76@2x.png    (152×152)
           Icon-83.5@2x.png  (167×167)
           Icon-1024.png     (1024×1024)
           → 写入 Assets.xcassets/AppIcon.appiconset/
           → 同步更新 Contents.json

Step 7  安装描述文件
        └─ 解析 .mobileprovision 获取 UUID
           cp config.ios.provisioningProfile
              ~/Library/MobileDevice/Provisioning\ Profiles/{uuid}.mobileprovision

Step 8  xcodebuild archive
        └─ 执行命令：
           xcodebuild \
             -project HBuilder-Hello.xcodeproj \
             -scheme HBuilder-Hello \
             -configuration Release \
             -archivePath {workspace}/build/output.xcarchive \
             archive \
             DEVELOPMENT_TEAM={config.ios.teamId} \
             PRODUCT_BUNDLE_IDENTIFIER={config.ios.bundleId} \
             CODE_SIGN_STYLE=Manual \
             PROVISIONING_PROFILE_SPECIFIER={profile_name}
           实时流式输出日志

Step 9  生成 ExportOptions.plist
        └─ 写入：
           method = config.ios.exportMethod（app-store/ad-hoc/enterprise）
           teamID = config.ios.teamId
           signingCertificate（从 .p12 中提取名称）
           provisioningProfiles = { bundleId: profile_name }

Step 10 xcodebuild exportArchive → IPA
        └─ 执行命令：
           xcodebuild \
             -exportArchive \
             -archivePath {workspace}/build/output.xcarchive \
             -exportPath {workspace}/build/export \
             -exportOptionsPlist {workspace}/ExportOptions.plist
           实时流式输出日志

Step 11 收集产物
        └─ {workspace}/build/export/*.ipa
           → 复制到 config.outputDir/{appName}-{version}.ipa
```

---

### 5.3 版本一致性自动校验

DCloud SDK 版本**必须**与 HBuilderX 导出资源版本一致，这是离线打包最大的坑。

```
导入 __UNI__XXXXX 时：
  1. 读取资源内 manifest.json → 提取 HBuilderX 版本号（如 "4.41"）
  2. 读取用户配置的 SDK 路径下 Readme.txt → 提取 SDK 版本号
  3. 两者比对：
     ✅ 一致 → 继续
     ⚠️ 不一致 → 弹出警告框，显示两个版本号，提示用户下载匹配版本 SDK
```

---

## 六、环境检测

工具启动时自动检测，结果在界面上逐项展示：

| 检测项 | 检测方式 | 用途 |
|--------|----------|------|
| Java（JDK） | `java -version` | Android Gradle 构建 |
| Android SDK | 检查 `$ANDROID_HOME` 环境变量 + `adb version` | Gradle 需要 |
| Gradle | 工作区内 `./gradlew --version`（Wrapper） | Android 构建 |
| Xcode | `xcode-select -p` | iOS 打包（仅 macOS） |
| xcodebuild | `xcodebuild -version` | iOS 打包（仅 macOS） |
| ohpm | `ohpm --version` | 鸿蒙打包 |
| hvigorw | 检查 DevEco SDK 路径 | 鸿蒙打包 |

- 检测通过：✅ 绿色
- 未检测到：❌ 红色，附带官方安装链接
- Windows 上 iOS 检测项：灰色显示「iOS 打包仅支持 macOS」

---

## 七、界面设计

### 主页（项目列表）

```
┌──────────────────────────────────────────────────────────────┐
│  UniApp 离线打包工具                    [环境检测]            │
├──────────────┬───────────────────────────────────────────────┤
│  我的项目     │                                               │
│  ──────────  │   （右侧显示选中项目的构建中心）               │
│  > 商城App  │                                               │
│    企业OA   │                                               │
│  ──────────  │                                               │
│  [＋ 新建]   │                                               │
└──────────────┴───────────────────────────────────────────────┘
```

### 项目配置页（4 个 Tab）

```
┌──────────────────────────────────────────────────────────────┐
│  ⚙️ 项目配置 — 商城App                                        │
│  [基础信息]  [Android]  [iOS]  [鸿蒙]                        │
│  ─────────────────────────────────────────────────────────── │
│  （基础信息 Tab）                                             │
│  应用名称     [商城App                              ]         │
│  DCloud AppKey[xxxxxxxxxxxxxxxx                    ]         │
│  版本名称     [1.0.0  ]   版本号 [1    ]                     │
│  应用图标     [icon_1024.png  ✅ 1024×1024] [重新选择]       │
│  输出目录     [~/Desktop/output            ] [选择]          │
│                                                              │
│  （Android Tab）                                             │
│  离线 SDK 路径 [/path/to/android-sdk       ] [选择]          │
│  SDK 版本      自动识别：4.41 ✅                             │
│  包名          [com.example.shop            ]                │
│  Keystore 路径 [release.jks                ] [选择]          │
│  Key Alias     [mykey     ]                                  │
│  Store 密码    [●●●●●●●●  ]                                  │
│  Key 密码      [●●●●●●●●  ]                                  │
│                                                              │
│  （iOS Tab）                                                 │
│  离线 SDK 路径 [/path/to/ios-sdk            ] [选择]         │
│  Bundle ID     [com.example.shop            ]                │
│  Team ID       [XXXXXXXXXX ]                                 │
│  描述文件      [app.mobileprovision         ] [选择]         │
│  P12 证书      [distribution.p12           ] [选择]         │
│  P12 密码      [●●●●●●●●  ]                                  │
│  导出方式      [app-store ▼]                                 │
│                                                              │
│                                    [保存配置]                │
└──────────────────────────────────────────────────────────────┘
```

### 构建中心（日常使用界面）

```
┌──────────────────────────────────────────────────────────────┐
│  🚀 构建中心 — 商城App                                        │
│                                                              │
│  ① 导入 UniApp 打包资源                                      │
│  ┌────────────────────────────────────────────────────────┐  │
│  │  拖拽 HBuilderX 导出的 __UNI__XXXXX 文件夹到此处        │  │
│  │  或  [点击选择文件夹]                                   │  │
│  │                                                        │  │
│  │  ✅ 已导入：__UNI__ABCD1234  (HBuilderX v4.41)         │  │
│  │     ✅ SDK 版本匹配（Android 4.41 / iOS 4.41）          │  │
│  └────────────────────────────────────────────────────────┘  │
│                                                              │
│  ② 选择打包平台                                              │
│  ┌───────────────────┐ ┌───────────────────┐                │
│  │ 🤖 Android        │ │ 🍎 iOS             │                │
│  │ SDK 4.41 ✅       │ │ SDK 4.41 ✅        │                │
│  │ 签名已配置 ✅     │ │ 证书已配置 ✅      │                │
│  │ [✅ 选中]         │ │ [✅ 选中]          │                │
│  └───────────────────┘ └───────────────────┘                │
│                                                              │
│              [  🚀  开始打包  ]                              │
└──────────────────────────────────────────────────────────────┘
```

### 构建中（日志面板）

```
┌──────────────────────────────────────────────────────────────┐
│  📋 构建日志                               [复制] [清空]      │
│                                                              │
│  Android  [████████████████████] ✅ 完成  2分47秒            │
│  iOS      [██████████░░░░░░░░░░] 53%  xcodebuild archive... │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ [Android] 🗂  创建构建工作区                          │   │
│  │ [Android] 📦  注入 SDK 库文件（6个.aar）              │   │
│  │ [Android] ⚙️   生成 build.gradle                     │   │
│  │ [Android] 📝  配置 AndroidManifest.xml               │   │
│  │ [Android] 📂  复制 SDK assets/data                   │   │
│  │ [Android] 📲  导入 UniApp 资源 __UNI__ABCD1234        │   │
│  │ [Android] 🔧  配置 dcloud_control.xml                │   │
│  │ [Android] 🎨  生成多尺寸图标                          │   │
│  │ [Android] 🚀  执行 gradlew assembleRelease...        │   │
│  │ [Android] > Task :app:preBuild UP-TO-DATE            │   │
│  │ [Android] > Task :app:assembleRelease                │   │
│  │ [Android] BUILD SUCCESSFUL in 1m 43s                 │   │
│  │ [Android] ✅  商城App-1.0.0.apk → ~/Desktop/output/  │   │
│  │ [iOS]     🗂  创建构建工作区                          │   │
│  │ ...                                                  │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                              │
│                      [⏹ 停止构建]                           │
└──────────────────────────────────────────────────────────────┘
```

### 完成页

```
┌──────────────────────────────────────────────────────────────┐
│  🎉 打包完成   总耗时 5分22秒                                  │
│                                                              │
│  🤖  商城App-1.0.0.apk     24.3 MB                          │
│      [📂 打开所在目录]  [📋 复制路径]                        │
│                                                              │
│  🍎  商城App-1.0.0.ipa     86.7 MB                          │
│      [📂 打开所在目录]  [📋 复制路径]                        │
│                                                              │
│           [再次打包]          [返回构建中心]                  │
└──────────────────────────────────────────────────────────────┘
```

---

## 八、技术栈

| 层 | 技术 | 用途 |
|----|------|------|
| 桌面框架 | **Tauri 2.x** | 工具本身的 Mac + Windows 桌面壳 |
| 前端 UI | **Vue3 + Vite + TypeScript** | 所有界面 |
| 状态管理 | **Pinia** + tauri-plugin-store | 项目配置持久化 |
| UI 组件 | **Naive UI** | 表单、进度条、日志滚动 |
| 后端 | **Rust** | 所有构建逻辑 |
| XML 读写 | **quick-xml** | AndroidManifest.xml / dcloud_control.xml / Info.plist |
| pbxproj 修改 | 字符串正则替换 | iOS project.pbxproj 的 PRODUCT_BUNDLE_IDENTIFIER |
| 图标生成 | **image** crate | 从 1024px 源图生成所有平台所有尺寸 |
| 密码安全存储 | **security-framework**（Mac）/ **windows-credentials**（Win） | 不明文写配置文件 |
| 子进程管理 | **tokio::process** | Gradle / xcodebuild 执行 + 实时日志流 |
| 模板渲染 | 内置简单 `{{var}}` 替换 | build.gradle / AndroidManifest 等模板 |

---

## 九、关键问题说明

### Q1：为什么需要用户自己下载 SDK，而不是工具自动下载？

DCloud 的离线 SDK 需要登录 DCloud 开发者账号才能下载，工具无法代替用户登录。因此设计为：
- 用户手动下载对应版本 SDK 并解压
- 工具填写路径后自动读取和使用

### Q2：签名密码如何保证安全？

密码字段在 UI 中以 `●●●●` 显示，**不写入 `config.json` 文件**，而是存入系统安全存储：
- macOS：`security add-generic-password`（Keychain）
- Windows：Windows Credential Manager API

构建时从系统存储实时读取，传入 Gradle 命令行参数，不落地为任何文本文件。

### Q3：iOS 为什么只能在 macOS 打包？

`xcodebuild` 工具只存在于 macOS 上，Windows 版工具中 iOS 构建选项会显示为不可用，并提示「iOS 打包需要在 macOS 环境下进行」。

### Q4：每次构建不会相互干扰吗？

每次构建都在独立的时间戳子目录进行（`~/.unipack/workspace/{id}/android-20260528-102341/`），完成后自动清理只保留最近 3 次工作区，不影响任何源文件。

---

## 十、开发计划

### Phase 1 — Android 完整流程（约 4 周）

优先做最高频的需求。

- [ ] Tauri 2.x 工程搭建（Vue3 + Rust）
- [ ] 项目配置管理（新建 / 编辑 / 删除 / 持久化）
- [ ] 资源导入（拖拽 + 选择路径 + appid 自动读取 + 版本校验）
- [ ] SDK 路径识别（自动读取版本号）
- [ ] Android 构建全流程（Steps 1-12）
- [ ] 图标自动生成（Android 各密度目录）
- [ ] Gradle 实时日志推送
- [ ] APK 产物展示 + 打开目录

### Phase 2 — iOS + 环境检测（约 3 周）

- [ ] 环境检测页（Java / Android SDK / Xcode / xcodebuild）
- [ ] iOS 构建全流程（Steps 1-11）
- [ ] iOS 图标生成（全部 12 个尺寸 + Contents.json）
- [ ] 描述文件自动安装
- [ ] pbxproj Bundle ID 替换
- [ ] xcodebuild archive + exportArchive
- [ ] IPA 产物展示

### Phase 3 — 鸿蒙 + 优化（约 2 周）

- [ ] 鸿蒙构建流程
- [ ] 构建失败常见错误自动诊断提示
- [ ] 历史构建记录
- [ ] 国内 Gradle Maven 镜像一键配置（解决首次构建慢问题）
- [ ] 工作区自动清理

---

## 十一、用户完整操作流程（最终体验）

```
【首次使用，约 10 分钟完成所有配置】

1. 下载安装本工具（.dmg 或 .exe）
2. 从 DCloud 官网下载对应版本的 Android/iOS 离线 SDK，解压备用
3. 打开工具 → 新建项目
4. 填写：
   - 基础信息：应用名称、AppKey、版本、上传 1024 图标
   - Android：SDK 路径、包名、Keystore 路径及密码
   - iOS：SDK 路径、Bundle ID、Team ID、描述文件、P12 证书及密码
5. 保存 → 配置完成，之后永不需要再改


【日常每次打包，2-3 分钟操作，等待编译】

1. HBuilderX → 发行 → 原生App-本地打包 → 生成本地打包App资源（约 30 秒）
2. 将 unpackage/resources/__UNI__XXXXX 文件夹拖入工具
3. 勾选 Android / iOS，点击「开始打包」
4. 看日志滚动，完成后 APK 和 IPA 出现在输出目录
```