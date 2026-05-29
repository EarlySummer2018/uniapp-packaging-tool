# UniApp 离线打包自动化工具 — 完整开发方案

工具本身：Tauri（Mac + Windows）
打包目标：Android APK / iOS IPA / 鸿蒙 HAP
用户：开发者，本机已装 Android Studio / Xcode / DevEco Studio

---

## 一、官方教程 vs 本工具的对照

官方离线打包教程要求用户每次打包都手动完成以下操作：

### Android（每次打包需手动做的事）

| # | 手动操作 | 本工具自动完成 |
|---|----------|----------------|
| 1 | 下载对应版本 Android 离线 SDK，解压 | ✅ 自动按版本下载并缓存 SDK |
| 2 | 将 6 个 .aar 文件复制到 `libs/` | ✅ 自动复制 |
| 3 | 手写 `build.gradle`（依赖、包名、签名、SDK 版本） | ✅ 自动生成 |
| 4 | 手改 `AndroidManifest.xml`（AppKey、Activity、FileProvider） | ✅ 自动注入 |
| 5 | 手改 `strings.xml`（应用名称） | ✅ 自动写入 |
| 6 | 将 `SDK/assets/data/` 复制进工程 | ✅ 自动复制 |
| 7 | 将 UniApp 导出的 `__UNI__XXXXX` 复制到 `assets/apps/` | ✅ 自动复制 |
| 8 | 手改 `dcloud_control.xml` 的 appid | ✅ 自动替换 |
| 9 | 把图标放到各 `drawable-xxx/` 文件夹（7个密度） | ✅ 从 1024 图标自动生成 |
| 10 | 在 `build.gradle` 写签名配置（密码明文） | ✅ 自动注入，密码走系统 Keychain |
| 11 | 命令行执行 `./gradlew assembleRelease` | ✅ 自动执行 |
| 12 | 在深层 build 目录找 APK | ✅ 自动复制到输出目录 |

### iOS（每次打包需手动做的事）

| # | 手动操作 | 本工具自动完成 |
|---|----------|----------------|
| 1 | 下载对应版本 iOS 离线 SDK，解压 | ✅ 自动按版本下载并缓存 |
| 2 | Xcode 打开工程，修改 Bundle Identifier | ✅ 自动修改 project.pbxproj |
| 3 | 修改 `info.plist` 写入 dcloud_appkey | ✅ 自动写入 |
| 4 | 配置应用名称、版本名称、版本号 | ✅ 自动写入 |
| 5 | 12 个尺寸图标拖入 Assets.xcassets | ✅ 自动生成全尺寸 |
| 6 | 将 UniApp 资源复制到 `Pandora/apps/` | ✅ 自动复制 |
| 7 | 修改 `control.xml` 的 appid | ✅ 自动替换 |
| 8 | Xcode Archive → 导出 IPA | ✅ 调用 xcodebuild 自动完成 |

### 鸿蒙（每次打包需手动做的事）

| # | 手动操作 | 本工具自动完成 |
|---|----------|----------------|
| 1 | DevEco Studio 新建/打开鸿蒙工程 | ✅ 使用内置模板工程 |
| 2 | 修改 `oh-package.json5` 填入 runtime 版本 | ✅ 自动写入 |
| 3 | Sync Now 同步依赖 | ✅ 调用 ohpm install |
| 4 | 修改 `EntryAbility.ets` 注入初始化代码 | ✅ 自动注入 |
| 5 | 导入 UniApp 资源 | ✅ 自动复制 |
| 6 | 编译打包 | ✅ 调用 hvigorw 构建 |

**用户只需做两件事：**
1. HBuilderX → 发行 → 生成本地打包 App 资源（约 30 秒）
2. 拖入本工具 → 选平台 → 点击打包

---

## 二、整体架构

```
┌─────────────────────────────────────────────────────────────────┐
│              UniApp 离线打包工具（Tauri 桌面应用）                 │
│                                                                   │
│   Vue3 前端                                                       │
│  ┌────────────┐ ┌────────────┐ ┌────────────┐ ┌──────────────┐  │
│  │  项目管理   │ │  资源导入   │ │  构建中心   │ │  实时日志    │  │
│  └────────────┘ └────────────┘ └────────────┘ └──────────────┘  │
│                        │ Tauri IPC                               │
│   Rust 后端                                                       │
│  ┌────────────┐ ┌────────────┐ ┌────────────┐ ┌──────────────┐  │
│  │  SDK管理   │ │ Android构建 │ │  iOS构建   │ │  鸿蒙构建    │  │
│  │ (下载/缓存) │ │  (Gradle)  │ │(xcodebuild)│ │  (hvigorw)   │  │
│  └────────────┘ └─────┬──────┘ └─────┬──────┘ └──────┬───────┘  │
└────────────────────────┼─────────────┼───────────────┼───────────┘
                         ▼             ▼               ▼
                      app.apk       app.ipa         app.hap
```

---

## 三、工程目录结构

```
uniapp-pack-tool/
│
├── src-tauri/                          # Rust 后端
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── src/
│       ├── main.rs
│       ├── commands/                   # 暴露给前端的 Tauri Command
│       │   ├── mod.rs
│       │   ├── project.rs              # 项目 CRUD
│       │   ├── resource.rs             # UniApp 资源导入与版本检测
│       │   ├── sdk.rs                  # SDK 下载 / 缓存管理
│       │   ├── build_android.rs        # Android 完整构建流程
│       │   ├── build_ios.rs            # iOS 完整构建流程
│       │   ├── build_harmony.rs        # 鸿蒙完整构建流程
│       │   └── env.rs                  # 环境检测
│       └── utils/
│           ├── xml.rs                  # XML 精准读写（AndroidManifest / control.xml）
│           ├── plist.rs                # iOS plist 读写
│           ├── icon.rs                 # 图标多尺寸生成
│           ├── process.rs              # 子进程执行 + 实时日志流
│           ├── keychain.rs             # 系统 Keychain / Credential 存取密码
│           └── fs.rs                   # 文件操作工具
│
├── src/                                # Vue3 前端
│   ├── main.ts
│   ├── App.vue
│   ├── router/index.ts
│   ├── stores/
│   │   ├── projects.ts                 # 项目列表（pinia + 持久化）
│   │   └── build.ts                    # 构建状态
│   ├── views/
│   │   ├── ProjectList.vue             # 主页：项目列表
│   │   ├── ProjectConfig.vue           # 项目配置（基本信息 / Android / iOS / 鸿蒙）
│   │   ├── BuildCenter.vue             # 构建中心（导入资源 + 选平台 + 执行）
│   │   └── SdkManager.vue              # SDK 管理
│   └── components/
│       ├── LogPanel.vue                # 实时日志滚动面板
│       ├── EnvChecker.vue              # 环境检测状态
│       ├── DropZone.vue                # 拖拽导入资源
│       ├── PlatformCard.vue            # 平台选择卡片
│       └── ArtifactList.vue            # 构建产物列表
│
└── bundled/                            # 内置工程模板（随工具分发）
    ├── android-template/               # 预配置好的 Android 原生工程骨架
    │   └── app/
    │       ├── build.gradle.tmpl       # 变量占位的模板
    │       ├── src/main/
    │       │   ├── AndroidManifest.xml.tmpl
    │       │   ├── assets/data/        # DCloud 基础 data 资源（内置，随 SDK 更新）
    │       │   └── res/values/strings.xml.tmpl
    │       └── proguard-rules.pro
    ├── ios-template/                   # iOS 工程骨架（来自 HBuilder-Hello）
    │   ├── HBuilder-Hello.xcodeproj/
    │   └── Pandora/
    └── harmony-template/               # 鸿蒙工程骨架
        ├── oh-package.json5.tmpl
        └── entry/src/main/ets/entryability/EntryAbility.ets.tmpl
```

---

## 四、数据模型

### 项目配置（`~/.unipack/projects/{id}/config.json`）

填一次，永久复用。

```jsonc
{
  "id": "uuid-xxxx",
  "name": "我的App",

  // 应用基本信息（从 UniApp manifest.json 可自动读取）
  "app": {
    "name": "我的App",             // 应用名称
    "appId": "__UNI__ABCD1234",   // UniApp AppId
    "dcloudAppKey": "xxx",         // DCloud 开发者中心申请的 AppKey
    "version": "1.0.0",
    "versionCode": 1,
    "icon1024": "/path/to/icon.png"  // 1024x1024 源图标，工具自动生成各尺寸
  },

  // Android 配置
  "android": {
    "enabled": true,
    "packageName": "com.example.myapp",
    "minSdkVersion": 21,
    "targetSdkVersion": 30,
    "compileSdkVersion": 35,
    "keystore": {
      "path": "/path/to/release.jks",
      "alias": "mykey"
      // 密码不存这里，走系统 Keychain
    }
  },

  // iOS 配置（仅 macOS 可打包）
  "ios": {
    "enabled": true,
    "bundleId": "com.example.myapp",
    "teamId": "XXXXXXXXXX",
    "provisioningProfile": "/path/to/app.mobileprovision",
    "certificate": "/path/to/distribution.p12",
    "exportMethod": "app-store"    // app-store / ad-hoc / enterprise / development
  },

  // 鸿蒙配置
  "harmony": {
    "enabled": true,
    "bundleName": "com.example.myapp",
    "runtimeVersion": "4.31.0"
  },

  // SDK 版本锁定（工具自动从导入的资源中检测并记录）
  "sdkVersion": "4.41",

  // 输出目录
  "outputDir": "~/Desktop/unipack-output"
}
```

---

## 五、核心模块详细设计

### 5.1 资源导入与版本检测（`resource.rs`）

用户拖入 HBuilderX 导出的文件夹后，工具自动完成：

```rust
pub struct ImportedResource {
    pub app_id: String,         // __UNI__ABCD1234
    pub hbuilderx_version: String, // "4.41"，来自资源内的 manifest.json
    pub resource_path: PathBuf,
}

pub fn import_resource(dropped_path: &Path) -> Result<ImportedResource> {
    // 1. 找到 __UNI__XXXXX 文件夹（用户可能拖入父目录或直接拖入该文件夹）
    let app_folder = find_uniapp_resource_folder(dropped_path)?;
    let app_id = app_folder.file_name().unwrap().to_str().unwrap().to_string();

    // 2. 读取其中的 manifest.json，提取版本号和 appid
    let manifest_path = app_folder.join("manifest.json");
    let manifest: Value = serde_json::from_str(&fs::read_to_string(manifest_path)?)?;
    let hbuilderx_version = manifest["versionName"].as_str()
        .unwrap_or("").to_string();

    // 3. 校验 appId 是否与项目配置一致，不一致则警告
    // 4. 检查 SDK 缓存，若版本不匹配则提示需要下载对应 SDK

    Ok(ImportedResource { app_id, hbuilderx_version, resource_path: app_folder })
}
```

### 5.2 SDK 管理（`sdk.rs`）

SDK 版本必须与 HBuilderX 版本严格一致，这是离线打包最大的坑。工具按版本缓存，一次下载永久复用。

```
~/.unipack/
├── sdk-cache/
│   ├── android/
│   │   ├── 4.41/           # 该版本 SDK 完整内容
│   │   │   ├── .complete   # 标记：下载完整
│   │   │   └── SDK/        # 解压内容
│   │   └── 4.38/
│   └── ios/
│       └── 4.41/
└── projects/
    └── {id}/
        ├── config.json
        └── workspace/      # 构建工作区（每次构建独立子目录，构建完可清理）
            └── android-20260528-102341/
            └── ios-20260528-102341/
```

```rust
impl SdkManager {
    // 根据资源版本号，确保本地有对应 SDK
    pub async fn ensure_sdk(
        &self,
        platform: Platform,
        version: &str,
        progress_tx: Sender<SdkDownloadProgress>,
    ) -> Result<PathBuf> {
        if self.is_cached(platform, version) {
            return Ok(self.sdk_path(platform, version));
        }
        // 从 DCloud CDN 下载对应版本
        let url = sdk_download_url(platform, version);
        self.download_and_extract(&url, platform, version, progress_tx).await
    }

    fn sdk_download_url(platform: Platform, version: &str) -> String {
        match platform {
            Platform::Android => format!(
                "https://native-res.dcloud.net.cn/native/android/{version}/android_sdk.zip"
            ),
            Platform::Ios => format!(
                "https://native-res.dcloud.net.cn/native/ios/{version}/ios_sdk.zip"
            ),
        }
    }
}
```

### 5.3 Android 构建全流程（`build_android.rs`）

```rust
pub async fn build(cfg: &ProjectConfig, resource: &ImportedResource, win: &Window) -> Result<PathBuf> {

    // ── 准备工作区 ──────────────────────────────────────────────────
    emit(win, "🗂  准备构建工作区...");
    // 以时间戳命名，每次构建互不干扰
    let workspace = cfg.workspace_dir().join(format!("android-{}", timestamp()));
    // 将内置 android-template 复制到工作区
    copy_dir(&BUNDLED_ANDROID_TEMPLATE, &workspace)?;

    // ── 注入 SDK 库文件 ─────────────────────────────────────────────
    emit(win, "📦 注入 DCloud SDK 库文件 (6 个 .aar)...");
    let sdk_dir = sdk_manager.sdk_path(Platform::Android, &resource.hbuilderx_version);
    // 将以下 aar 文件复制到 workspace/app/libs/
    for aar in REQUIRED_AARS {  // lib.5plus.base-release.aar 等 6 个
        fs::copy(sdk_dir.join("SDK/libs").join(aar), workspace.join("app/libs").join(aar))?;
    }
    // 同时将 SDK/assets/data/ 复制到工程 assets/data/
    copy_dir(sdk_dir.join("SDK/assets/data"), workspace.join("app/src/main/assets/data"))?;

    // ── 生成 build.gradle ───────────────────────────────────────────
    emit(win, "⚙️  生成 build.gradle...");
    render_template(
        "android-template/app/build.gradle.tmpl",
        workspace.join("app/build.gradle"),
        &BuildGradleVars {
            package_name:        &cfg.android.package_name,
            compile_sdk:         cfg.android.compile_sdk_version,
            target_sdk:          cfg.android.target_sdk_version,
            min_sdk:             cfg.android.min_sdk_version,
            version_code:        cfg.app.version_code,
            version_name:        &cfg.app.version,
            keystore_path:       &cfg.android.keystore.path,
            key_alias:           &cfg.android.keystore.alias,
            key_password:        &keychain::get(&cfg.id, "android-key-password")?,
            store_password:      &keychain::get(&cfg.id, "android-store-password")?,
        },
    )?;

    // ── 修改 AndroidManifest.xml ────────────────────────────────────
    emit(win, "📝 配置 AndroidManifest.xml...");
    let manifest = workspace.join("app/src/main/AndroidManifest.xml");
    xml::set_meta_data(&manifest, "dcloud_appkey", &cfg.app.dcloud_app_key)?;
    xml::set_attr(&manifest, "/manifest", "package", &cfg.android.package_name)?;
    // FileProvider authorities 必须与包名一致
    xml::set_attr(
        &manifest,
        "//provider[@android:name='io.dcloud.common.util.DCloud_FileProvider']",
        "android:authorities",
        &format!("{}.dc.fileprovider", cfg.android.package_name),
    )?;

    // ── 写应用名称 ──────────────────────────────────────────────────
    emit(win, "✏️  写入应用名称...");
    xml::set_text(
        &workspace.join("app/src/main/res/values/strings.xml"),
        "//string[@name='app_name']",
        &cfg.app.name,
    )?;

    // ── 导入 UniApp 资源 ────────────────────────────────────────────
    emit(win, "📲 导入 UniApp 应用资源...");
    let apps_dir = workspace.join("app/src/main/assets/apps").join(&resource.app_id);
    copy_dir(&resource.resource_path, &apps_dir)?;

    // ── 修改 dcloud_control.xml ─────────────────────────────────────
    // appid 必须与文件夹名 和 manifest.json 中的 id 三者一致
    emit(win, "🔧 配置 dcloud_control.xml...");
    xml::set_attr(
        &workspace.join("app/src/main/assets/data/dcloud_control.xml"),
        "/hbuilder",
        "appid",
        &resource.app_id,
    )?;

    // ── 生成图标 ────────────────────────────────────────────────────
    emit(win, "🎨 生成多尺寸图标...");
    icon::generate_android(&cfg.app.icon_1024, &workspace)?;
    // 从 1024px 生成以下所有目录的 icon.png / push.png / splash.png：
    // drawable-ldpi(36) / mdpi(48) / hdpi(72) / xhdpi(96) / xxhdpi(144) / xxxhdpi(192)

    // ── 执行 Gradle 构建 ────────────────────────────────────────────
    emit(win, "🚀 执行 Gradle 构建（首次可能需要下载依赖，请耐心等待）...");
    process::run_streaming(
        if cfg!(windows) { "gradlew.bat" } else { "./gradlew" },
        &["assembleRelease", "--stacktrace"],
        &workspace,
        &[("ANDROID_HOME", &cfg.android_sdk_home), ("JAVA_HOME", &cfg.java_home)],
        win,
        "android-log",
    ).await?;

    // ── 收集产物 ────────────────────────────────────────────────────
    let apk_src = workspace.join("app/build/outputs/apk/release/app-release.apk");
    let apk_dst = cfg.output_dir().join(format!("{}-{}.apk", cfg.app.name, cfg.app.version));
    fs::copy(&apk_src, &apk_dst)?;

    emit(win, &format!("✅ Android 打包完成！→ {}", apk_dst.display()));
    Ok(apk_dst)
}
```

**`build.gradle.tmpl` 完整模板：**

```groovy
apply plugin: 'com.android.application'

android {
    compileSdkVersion {{compile_sdk}}
    buildToolsVersion "{{build_tools}}"
    defaultConfig {
        applicationId "{{package_name}}"
        minSdkVersion {{min_sdk}}
        targetSdkVersion {{target_sdk}}
        versionCode {{version_code}}
        versionName "{{version_name}}"
        multiDexEnabled true
        ndk {
            abiFilters 'x86', 'armeabi-v7a', 'arm64-v8a'
        }
    }
    signingConfigs {
        config {
            keyAlias '{{key_alias}}'
            keyPassword '{{key_password}}'
            storeFile file('{{keystore_path}}')
            storePassword '{{store_password}}'
            v1SigningEnabled true
            v2SigningEnabled true
        }
    }
    buildTypes {
        release {
            minifyEnabled false
            signingConfig signingConfigs.config
        }
    }
    aaptOptions {
        additionalParameters '--auto-add-overlay'
        ignoreAssetsPattern "!.svn:!.git:.*:!CVS:!thumbs.db:!picasa.ini:!*.scc:*~"
    }
    packagingOptions {
        jniLibs { useLegacyPackaging true }
    }
    compileOptions {
        sourceCompatibility JavaVersion.VERSION_1_8
        targetCompatibility JavaVersion.VERSION_1_8
    }
}

dependencies {
    implementation fileTree(include: ['*.jar', '*.aar'], dir: 'libs')
    implementation 'androidx.appcompat:appcompat:1.1.0'
    implementation 'androidx.localbroadcastmanager:localbroadcastmanager:1.0.0'
    implementation 'androidx.core:core:1.6.0'
    implementation "androidx.fragment:fragment:1.1.0"
    implementation 'androidx.recyclerview:recyclerview:1.1.0'
    implementation "com.facebook.fresco:fresco:3.4.0"
    implementation "com.facebook.fresco:middleware:3.4.0"
    implementation "com.facebook.fresco:animated-gif:3.4.0"
    implementation "com.facebook.fresco:webpsupport:3.4.0"
    implementation "com.facebook.fresco:animated-webp:3.4.0"
    implementation 'com.github.bumptech.glide:glide:4.9.0'
    implementation 'com.alibaba:fastjson:1.2.83'
    implementation 'androidx.webkit:webkit:1.5.0'
    annotationProcessor 'com.github.bumptech.glide:compiler:4.9.0'
    implementation "net.lingala.zip4j:zip4j:2.11.5"
}
```

### 5.4 iOS 构建全流程（`build_ios.rs`，仅 macOS 可执行）

```rust
pub async fn build(cfg: &ProjectConfig, resource: &ImportedResource, win: &Window) -> Result<PathBuf> {

    emit(win, "🗂  准备 iOS 构建工作区...");
    let workspace = cfg.workspace_dir().join(format!("ios-{}", timestamp()));
    // 复制内置 ios-template（即 HBuilder-Hello 工程骨架）
    copy_dir(&BUNDLED_IOS_TEMPLATE, &workspace)?;

    // ── 修改 Bundle Identifier ──────────────────────────────────────
    emit(win, "🔧 配置 Bundle Identifier...");
    // 修改 project.pbxproj 中所有 PRODUCT_BUNDLE_IDENTIFIER 字段
    pbxproj::set_bundle_id(
        &workspace.join("HBuilder-Hello.xcodeproj/project.pbxproj"),
        &cfg.ios.bundle_id,
    )?;

    // ── 修改 info.plist ─────────────────────────────────────────────
    emit(win, "📝 配置 info.plist...");
    let info_plist = workspace.join("HBuilder-Hello/Info.plist");
    plist::set_string(&info_plist, "dcloud_appkey",           &cfg.app.dcloud_app_key)?;
    plist::set_string(&info_plist, "CFBundleDisplayName",     &cfg.app.name)?;
    plist::set_string(&info_plist, "CFBundleShortVersionString", &cfg.app.version)?;
    plist::set_string(&info_plist, "CFBundleVersion",         &cfg.app.version_code.to_string())?;

    // ── 导入 UniApp 资源 ────────────────────────────────────────────
    emit(win, "📲 导入 UniApp 应用资源...");
    let apps_dir = workspace.join("HBuilder-Hello/Pandora/apps").join(&resource.app_id);
    copy_dir(&resource.resource_path, &apps_dir)?;

    // ── 修改 control.xml ────────────────────────────────────────────
    emit(win, "🔧 配置 control.xml...");
    xml::set_attr(
        &workspace.join("HBuilder-Hello/Pandora/control.xml"),
        "/hbuilder",
        "appid",
        &resource.app_id,
    )?;

    // ── 生成图标 ────────────────────────────────────────────────────
    emit(win, "🎨 生成 iOS 多尺寸图标...");
    icon::generate_ios(&cfg.app.icon_1024, &workspace)?;
    // 生成 AppIcon.appiconset 下全部 12 个尺寸：
    // 20@2x(40) / 20@3x(60) / 29@2x(58) / 29@3x(87) / 40@2x(80) / 40@3x(120)
    // 60@2x(120) / 60@3x(180) / 76(76) / 76@2x(152) / 83.5@2x(167) / 1024(1024)
    // 并更新 Contents.json

    // ── 安装描述文件 ────────────────────────────────────────────────
    emit(win, "🔑 安装 Provisioning Profile...");
    provision::install(&cfg.ios.provisioning_profile)?;
    // cp xxx.mobileprovision ~/Library/MobileDevice/Provisioning\ Profiles/{uuid}.mobileprovision

    // ── xcodebuild archive ──────────────────────────────────────────
    emit(win, "🏗  xcodebuild archive...");
    let archive_path = workspace.join("build/Archive.xcarchive");
    process::run_streaming(
        "xcodebuild",
        &[
            "-project",     "HBuilder-Hello.xcodeproj",
            "-scheme",      "HBuilder-Hello",
            "-configuration", "Release",
            "-archivePath", archive_path.to_str().unwrap(),
            "archive",
            &format!("DEVELOPMENT_TEAM={}", cfg.ios.team_id),
            &format!("PRODUCT_BUNDLE_IDENTIFIER={}", cfg.ios.bundle_id),
            "CODE_SIGN_STYLE=Manual",
        ],
        &workspace.join("HBuilder-Hello"),
        &[],
        win,
        "ios-log",
    ).await?;

    // ── 生成 ExportOptions.plist ────────────────────────────────────
    let export_options = workspace.join("ExportOptions.plist");
    plist::write_export_options(&export_options, &cfg.ios)?;

    // ── xcodebuild exportArchive → IPA ──────────────────────────────
    emit(win, "📦 导出 IPA...");
    let export_dir = workspace.join("build/export");
    process::run_streaming(
        "xcodebuild",
        &[
            "-exportArchive",
            "-archivePath",      archive_path.to_str().unwrap(),
            "-exportPath",       export_dir.to_str().unwrap(),
            "-exportOptionsPlist", export_options.to_str().unwrap(),
        ],
        &workspace,
        &[],
        win,
        "ios-log",
    ).await?;

    // ── 收集产物 ────────────────────────────────────────────────────
    let ipa_src = find_ipa(&export_dir)?;
    let ipa_dst = cfg.output_dir().join(format!("{}-{}.ipa", cfg.app.name, cfg.app.version));
    fs::copy(&ipa_src, &ipa_dst)?;

    emit(win, &format!("✅ iOS 打包完成！→ {}", ipa_dst.display()));
    Ok(ipa_dst)
}
```

### 5.5 鸿蒙构建全流程（`build_harmony.rs`）

```rust
pub async fn build(cfg: &ProjectConfig, resource: &ImportedResource, win: &Window) -> Result<PathBuf> {

    emit(win, "🗂  准备鸿蒙构建工作区...");
    let workspace = cfg.workspace_dir().join(format!("harmony-{}", timestamp()));
    copy_dir(&BUNDLED_HARMONY_TEMPLATE, &workspace)?;

    // ── 修改 oh-package.json5 ───────────────────────────────────────
    emit(win, "📝 配置 oh-package.json5...");
    // 将 "@dcloudio/uni-app-runtime" 版本设为与 HBuilderX 一致
    json5::set_dependency(
        &workspace.join("oh-package.json5"),
        "@dcloudio/uni-app-runtime",
        &cfg.harmony.runtime_version,
    )?;

    // ── 修改 EntryAbility.ets ───────────────────────────────────────
    emit(win, "⚙️  注入 SDK 初始化代码...");
    render_template(
        "harmony-template/entry/src/main/ets/entryability/EntryAbility.ets.tmpl",
        workspace.join("entry/src/main/ets/entryability/EntryAbility.ets"),
        &HarmonyVars { app_id: &resource.app_id },
    )?;
    // 生成内容：
    // import { UniEntryAbility } from "@dcloudio/uni-app-runtime";
    // export default class EntryAbility extends UniEntryAbility {
    //   constructor() { super("{{app_id}}", { debug: BuildProfile.DEBUG }); }
    // }

    // ── ohpm install ─────────────────────────────────────────────────
    emit(win, "📦 同步鸿蒙依赖（ohpm install）...");
    process::run_streaming("ohpm", &["install"], &workspace, &[], win, "harmony-log").await?;

    // ── 导入 UniApp 资源 ─────────────────────────────────────────────
    emit(win, "📲 导入 UniApp 应用资源...");
    copy_dir(
        &resource.resource_path,
        &workspace.join("entry/src/main/resources/rawfile/apps").join(&resource.app_id),
    )?;

    // ── hvigorw 构建 ─────────────────────────────────────────────────
    emit(win, "🚀 执行鸿蒙构建（hvigorw assembleRelease）...");
    let hvigorw = if cfg!(windows) { "hvigorw.bat" } else { "./hvigorw" };
    process::run_streaming(
        hvigorw,
        &["assembleRelease"],
        &workspace,
        &[("deveco_sdk", &cfg.deveco_sdk_path)],
        win,
        "harmony-log",
    ).await?;

    // ── 收集产物 ─────────────────────────────────────────────────────
    let hap_src = find_hap(&workspace)?;
    let hap_dst = cfg.output_dir().join(format!("{}-{}.hap", cfg.app.name, cfg.app.version));
    fs::copy(&hap_src, &hap_dst)?;

    emit(win, &format!("✅ 鸿蒙打包完成！→ {}", hap_dst.display()));
    Ok(hap_dst)
}
```

### 5.6 实时日志流（`process.rs`）

```rust
// 执行子进程并将 stdout/stderr 实时推送到前端
pub async fn run_streaming(
    cmd: &str,
    args: &[&str],
    cwd: &Path,
    envs: &[(&str, &str)],
    window: &Window,
    event: &str,
) -> Result<()> {
    let mut child = Command::new(cmd)
        .args(args)
        .current_dir(cwd)
        .envs(envs.iter().copied())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    // 并发读取 stdout 和 stderr
    let stdout = BufReader::new(child.stdout.take().unwrap());
    let stderr = BufReader::new(child.stderr.take().unwrap());
    let win_clone = window.clone();
    let event_clone = event.to_string();

    let handle = tokio::spawn(async move {
        let mut lines = stdout.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            win_clone.emit(&event_clone, LogLine { level: "info", text: line }).ok();
        }
    });

    let mut err_lines = stderr.lines();
    while let Ok(Some(line)) = err_lines.next_line().await {
        window.emit(event, LogLine { level: "warn", text: line }).ok();
    }

    handle.await?;
    let status = child.wait().await?;
    if !status.success() {
        return Err(anyhow!("命令退出码: {}", status.code().unwrap_or(-1)));
    }
    Ok(())
}
```

### 5.7 环境检测（`env.rs`）

```rust
#[derive(Serialize)]
pub struct EnvReport {
    pub java:        EnvItem,   // java -version
    pub android_sdk: EnvItem,   // $ANDROID_HOME/platform-tools/adb
    pub gradle:      EnvItem,   // ./gradlew --version（工程内）
    pub xcode:       EnvItem,   // xcode-select -p（仅 macOS）
    pub xcodebuild:  EnvItem,   // xcodebuild -version（仅 macOS）
    pub ohpm:        EnvItem,   // ohpm --version
    pub hvigorw:     EnvItem,   // 鸿蒙构建工具
}

// 每项结构
pub struct EnvItem {
    pub found: bool,
    pub version: Option<String>,
    pub path: Option<String>,
    pub install_guide: &'static str,  // 未找到时显示的安装指引 URL
}
```

---

## 六、前端界面设计

### 6.1 整体布局

```
┌──────────────────────────────────────────────────────────────────┐
│  UniApp 离线打包工具                       [环境检测]  [SDK管理]  │
├──────────────┬───────────────────────────────────────────────────┤
│              │                                                    │
│  📁 我的项目  │              右侧内容区                           │
│  ──────────  │                                                    │
│  > 项目A ●  │                                                    │
│    项目B    │                                                    │
│  ──────────  │                                                    │
│  [＋ 新建]   │                                                    │
│              │                                                    │
└──────────────┴───────────────────────────────────────────────────┘

● 有未读构建结果
```

### 6.2 构建中心（最常用的核心界面）

```
┌─────────────────────────────────────────────────────────────────┐
│  🚀 构建中心  ·  项目A  ·  v1.0.0 (1)                           │
│                                                                   │
│  ┌── Step 1：导入 UniApp 打包资源 ───────────────────────────┐   │
│  │                                                           │   │
│  │   ┌─────────────────────────────────────────────────┐    │   │
│  │   │                                                 │    │   │
│  │   │   拖拽 HBuilderX 导出的资源文件夹到此处           │    │   │
│  │   │   或  [选择文件夹]                               │    │   │
│  │   │                                                 │    │   │
│  │   └─────────────────────────────────────────────────┘    │   │
│  │                                                           │   │
│  │   ✅ 上次导入：__UNI__ABCD1234  ·  HBuilderX 4.41        │   │
│  │      2026-05-28 10:23  ·  [重新导入]                     │   │
│  └───────────────────────────────────────────────────────────┘   │
│                                                                   │
│  ┌── Step 2：选择打包平台 ────────────────────────────────────┐   │
│  │                                                            │   │
│  │  ┌──────────────────┐ ┌──────────────────┐ ┌───────────┐  │   │
│  │  │  🤖  Android     │ │  🍎  iOS          │ │ 🌸  鸿蒙  │  │   │
│  │  │  SDK 4.41 ✅     │ │  SDK 4.41 ✅     │ │  已配置 ✅ │  │   │
│  │  │  签名已配置 ✅   │ │  证书已配置 ✅   │ │           │  │   │
│  │  │                  │ │  ⚠️ 仅 macOS     │ │           │  │   │
│  │  │  [✅ 已选中]     │ │  [✅ 已选中]     │ │  [ 选择 ] │  │   │
│  │  └──────────────────┘ └──────────────────┘ └───────────┘  │   │
│  └────────────────────────────────────────────────────────────┘   │
│                                                                   │
│                    [  🚀  开始打包  ]                             │
└─────────────────────────────────────────────────────────────────┘
```

### 6.3 构建执行中

```
┌─────────────────────────────────────────────────────────────────┐
│  📋 构建日志                                   [复制全部] [清空]  │
│                                                                   │
│  Android  [████████████████████] 完成 ✅  3分12秒               │
│  iOS      [██████████░░░░░░░░░░]  51%   xcodebuild archive...   │
│                                                                   │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │ 10:23:41  [Android]  🗂  准备构建工作区...               │    │
│  │ 10:23:42  [Android]  📦  注入 DCloud SDK 库文件...       │    │
│  │ 10:23:43  [Android]  ⚙️   生成 build.gradle...           │    │
│  │ 10:23:43  [Android]  📝  配置 AndroidManifest.xml...     │    │
│  │ 10:23:44  [Android]  🎨  生成多尺寸图标...               │    │
│  │ 10:23:44  [Android]  📲  导入 UniApp 资源...             │    │
│  │ 10:23:44  [Android]  🔧  配置 dcloud_control.xml...      │    │
│  │ 10:23:45  [Android]  🚀  执行 Gradle 构建...             │    │
│  │ 10:23:51  [Android]  > Task :app:preBuild UP-TO-DATE     │    │
│  │ 10:24:10  [Android]  > Task :app:assembleRelease         │    │
│  │ 10:24:11  [Android]  BUILD SUCCESSFUL in 26s             │    │
│  │ 10:24:11  [Android]  ✅ 完成 → ~/Desktop/项目A-1.0.0.apk │    │
│  │ 10:24:12  [iOS]      🗂  准备构建工作区...               │    │
│  │ ...                                                      │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                   │
│                        [⏹ 停止构建]                              │
└─────────────────────────────────────────────────────────────────┘
```

### 6.4 构建完成

```
┌─────────────────────────────────────────────────────────────────┐
│  🎉 打包完成   总耗时 6分44秒                                     │
│                                                                   │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                                                         │    │
│  │  🤖  项目A-1.0.0.apk          24.3 MB   ✅             │    │
│  │      [📂 打开目录]  [📋 复制路径]  [📱 adb 安装]       │    │
│  │                                                         │    │
│  │  🍎  项目A-1.0.0.ipa          87.1 MB   ✅             │    │
│  │      [📂 打开目录]  [📋 复制路径]  [⬆️ 上传 TestFlight] │    │
│  │                                                         │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                   │
│  [再次打包]                                  [查看历史构建]       │
└─────────────────────────────────────────────────────────────────┘
```

### 6.5 项目配置页（首次填写，之后自动复用）

```
┌─────────────────────────────────────────────────────────────────┐
│  ⚙️ 项目配置  ·  项目A                                           │
│  ─── 基本信息 ───────────────────────────────────────────────── │
│  应用名称    [我的App                                ]           │
│  UniApp AppId  [__UNI__ABCD1234  （从导入资源自动读取）]         │
│  DCloud AppKey [在 dev.dcloud.net.cn 申请            ]          │
│  版本号      [1.0.0]  Build [1]                                 │
│  应用图标    [icon_1024.png  1024×1024   ] [选择]               │
│  输出目录    [~/Desktop/unipack-output  ] [选择]                 │
│                                                                   │
│  ─── Android ────────────────────────────────────────────────── │
│  包名        [com.example.myapp                      ]           │
│  签名文件    [release.jks                            ] [选择]    │
│  Key 别名    [mykey     ]                                        │
│  Store 密码  [●●●●●●●●  ]（保存到系统 Keychain，不写入配置文件） │
│  Key 密码    [●●●●●●●●  ]                                        │
│                                                                   │
│  ─── iOS（仅 macOS）─────────────────────────────────────────── │
│  Bundle ID   [com.example.myapp                      ]           │
│  Team ID     [XXXXXXXXXX ]                                       │
│  描述文件    [app.mobileprovision                    ] [选择]    │
│  P12 证书    [distribution.p12                       ] [选择]    │
│  P12 密码    [●●●●●●●●  ]                                        │
│  导出方式    [app-store ▼]                                       │
│                                                                   │
│  ─── 鸿蒙 ──────────────────────────────────────────────────── │
│  Bundle Name [com.example.myapp                      ]           │
│                                                                   │
│                                           [保存配置]             │
└─────────────────────────────────────────────────────────────────┘
```

---

## 七、技术栈

| 层 | 选型 | 说明 |
|----|------|------|
| 桌面框架 | Tauri 2.x | 工具本身的 Mac/Windows 桌面壳 |
| 前端 | Vue 3 + Vite + TypeScript | 配置界面 / 日志面板 |
| 状态管理 | Pinia + tauri-plugin-store | 项目配置持久化到本地 |
| UI 组件 | Naive UI | 表单 / 进度 / 日志滚动 |
| 后端 | Rust | 所有构建逻辑 |
| XML 读写 | quick-xml | AndroidManifest / dcloud_control.xml |
| plist 读写 | plist crate | iOS Info.plist / ExportOptions.plist |
| 图标生成 | image crate | 一张图生成所有平台所有尺寸 |
| HTTP 下载 | reqwest | SDK 自动下载 |
| 模板渲染 | 内置简单替换（`{{var}}`） | build.gradle / EntryAbility.ets |
| 密码存储 | security-framework (Mac) / windows-credentials (Win) | Keychain / Credential Manager |
| 进程管理 | tokio::process | Gradle / xcodebuild / hvigorw 实时日志流 |

---

## 八、关键问题与解决方案

### 问题 1：SDK 版本必须与 HBuilderX 完全一致

**现象**：版本不一致会导致 App 启动时弹出"版本不一致"提示框，功能可能异常。

**方案**：
1. 导入资源时，自动从资源内 `manifest.json` 读取 HBuilderX 版本
2. 查询本地 SDK 缓存，命中则直接使用
3. 未命中则弹出提示，一键触发该版本 SDK 下载
4. SDK 按版本隔离缓存，多个版本可并存

### 问题 2：签名密码不能明文写进配置文件

**方案**：密码字段在配置文件中留空，实际存取走系统安全存储：
- macOS：`security add-generic-password` / `security find-generic-password`
- Windows：Windows Credential Manager API

### 问题 3：iOS 只能在 macOS 打包

**方案**：在 Windows 上运行工具时，iOS 平台卡片显示为灰色不可选，并说明"iOS 打包需要在 macOS 上进行"。

### 问题 4：Gradle 首次构建需下载大量依赖（国内网络慢）

**方案**：
1. 日志面板明确提示"首次构建需下载 Gradle 依赖，耗时约 5-20 分钟，取决于网络"
2. 提供国内镜像配置选项（在 `build.gradle` 的 repositories 中注入阿里云 maven 镜像）

### 问题 5：工作区清理

**方案**：每次构建使用时间戳子目录，构建完成后提供"清理旧构建工作区"按钮（保留最近 3 次，清理更早的）。

---

## 九、开发路线图

### Phase 1 — MVP（4 周）
优先完成最高频的 Android 打包。

- [ ] Tauri 工程搭建（Vue3 + Rust）
- [ ] 项目配置 CRUD + 持久化
- [ ] 资源导入（拖拽 + 路径选择 + 版本检测）
- [ ] Android SDK 下载 + 缓存管理
- [ ] Android 构建全流程（模板 → 注入 → Gradle → APK）
- [ ] 图标自动生成（Android 各密度）
- [ ] 实时日志面板
- [ ] 构建产物管理 + 打开目录

### Phase 2（3 周）
iOS 打包 + 环境检测完善。

- [ ] 环境检测页（Java / Android SDK / Xcode）
- [ ] iOS SDK 下载 + 缓存
- [ ] iOS 构建全流程（project.pbxproj / info.plist / xcodebuild）
- [ ] iOS 图标生成（全尺寸 xcassets）
- [ ] 描述文件安装辅助

### Phase 3（2 周）
鸿蒙 + 体验优化。

- [ ] 鸿蒙构建全流程
- [ ] 历史构建记录
- [ ] 构建失败常见错误自动诊断
- [ ] 国内 Gradle 镜像一键配置
- [ ] 工作区自动清理

---

## 十、用户使用流程（最终体验）

```
首次使用（一次性配置，约 10 分钟）：
  1. 安装工具（.dmg / .exe）
  2. 新建项目，填写：应用名称、DCloud AppKey、包名、签名证书、Bundle ID...
  3. 上传一张 1024×1024 PNG 图标
  4. 保存 → 完成

日常每次打包（约 2 分钟操作 + 等待编译）：
  1. HBuilderX 点击"生成本地打包 App 资源"（30 秒）
  2. 将导出的文件夹拖入工具
  3. 选择平台（Android / iOS / 鸿蒙），点击"开始打包"
  4. 看着日志滚动，完成后 APK / IPA / HAP 出现在输出目录
```