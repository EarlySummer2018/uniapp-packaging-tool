# uni-app 本地打包教程

> 本文档系统梳理 uni-app 本地打包（离线打包）全流程，从环境准备到最终出包，按实际操作顺序组织，帮助开发者快速理解和完成各平台的本地打包配置。


## 一、概述与核心概念

uni-app 应用生成安装包主要有两种方式：**云打包**与**本地打包（离线打包）**。

- **云打包**：HBuilderX 将应用提交到 DCloud 云端完成编译签名，支持安心打包模式（不提交应用代码和证书，本地签名），操作简便，适合快速迭代。
- **本地打包（离线打包）**：开发者下载各平台原生 SDK，在 Android Studio、Xcode 或 DevEco Studio 中自行编译生成安装包，可获得更高的自定义程度。

> 本地打包的优势：完全掌控编译过程、支持私有化部署场景；局限性：无法使用付费原生插件、不支持原生代码混淆（仅 Android）。

**推荐方案选择**：
| 场景 | 推荐方式 |
|------|---------|
| 快速验证、开发调试 | 云打包（安心打包） |
| 使用付费原生插件 | 云打包 |
| 需原生代码混淆保护 JS | 云打包 |
| 企业私有化部署、深度定制 | 本地打包 |

参考官方文档：[App 打包概述](https://nativesupport.dcloud.net.cn/AppDocs/)


## 二、本地打包前置准备

### 2.1 必备工具与环境

| 平台 | 开发工具 | 最低版本要求 |
|------|---------|-------------|
| Android | Android Studio | 2022.3.1 及以上 |
| iOS | Xcode | Mac 环境，版本建议 14+ |
| 鸿蒙 | DevEco Studio | 5.0.2 及以上 |

### 2.2 获取 AppID 与 AppKey

1. **获取 AppID**：在 HBuilderX 中打开项目，查看 `manifest.json`，其中 `appid` 即为应用的唯一标识（格式如 `__UNI__xxxxxx`）。
2. **申请 AppKey**：
   - 登录 [DCloud 开发者中心](https://dev.dcloud.net.cn/)，进入“我的应用”。
   - 选择对应应用，点击“各平台信息” → “新增”，填写平台、包名、签名 SHA1/SHA256。
   - 提交后点击“离线打包 Key”→“创建”→“查看”，获取 AppKey。

### 2.3 下载 SDK

- **Android SDK**：[下载地址](https://nativesupport.dcloud.net.cn/AppDocs/download/android.html)
- **iOS SDK**：[下载地址](https://nativesupport.dcloud.net.cn/AppDocs/download/ios.html)
- **鸿蒙 SDK**：[下载地址](https://uniapp.dcloud.net.cn/harmony/dev.html)

### 2.4 预检查清单

- [ ] HBuilderX 已打开目标 uni-app 项目，能正常编译运行
- [ ] 各平台 SDK 已下载解压，原生工程能正常打开编译
- [ ] AppID 已从 DCloud 开发者中心获取
- [ ] 签名证书已准备（Android 可参考[证书生成指南](https://ask.dcloud.net.cn/article/35777)，iOS 参考[Apple 证书申请指南](https://ask.dcloud.net.cn/article/152)）


## 三、导出 uni-app 本地打包资源

在 HBuilderX 中执行以下操作：

1. 打开 uni-app 项目
2. 点击菜单 **发行** → **原生 App-本地打包** → **生成本地打包 App 资源**
3. 等待编译完成，资源文件生成于项目目录的 `unpackage/resources/` 下

> **注意**：建议使用与 SDK 版本匹配的 HBuilderX 版本，避免因版本不兼容导致编译问题。


## 四、平台原生工程配置与资源导入

### 4.1 Android 平台

#### 4.1.1 工程初始配置

1. 用 Android Studio 打开下载的 SDK 项目（`SDK/assets/apps/` 目录下含 `__UNI__xxxxxx` 模板文件夹）。
2. **配置 Gradle JDK**：`File` → `Settings` → `Build, Execution, Deployment` → `Build Tools` → `Gradle`，JDK 选择 1.8 版本。
3. **配置 AppKey**：在 `app/src/main/AndroidManifest.xml` 中找到 `dcloud_appkey` 的 `meta-data`，替换为第 2.2 步获取的 AppKey。

#### 4.1.2 配置签名

编辑 `app/build.gradle`，修改 `signingConfigs` 和 `defaultConfig`：

```gradle
signingConfigs {
    config {
        keyAlias '你的key别名'
        keyPassword '你的密码'
        storeFile file('你的签名文件.jks')
        storePassword '你的密码'
        v1SigningEnabled true
        v2SigningEnabled true
    }
}
defaultConfig {
    applicationId "你的应用包名（如 com.example.app）"
}
```

#### 4.1.3 导入前端资源

- 将第 3 步生成的资源文件夹（如 `__UNI__xxxxxx`）拷贝到 `app/src/main/assets/apps/` 目录下
- 若目录不存在则手动创建

#### 4.1.4 修改 appid

打开 `app/src/main/assets/data/dcloud_control.xml`：

```xml
<apps>
    <app appid="资源文件夹名称（如__UNI__xxxxxx）" appver=""/>
</apps>
```

#### 4.1.5 修改应用配置

| 配置项 | 文件位置 |
|--------|---------|
| 应用名称 | `res/values/strings.xml` → `app_name` |
| 桌面图标 | `res/drawable/icon.png` |
| 推送图标 | `res/drawable/push.png` |
| 启动页图标 | `res/drawable/splash.png` |

#### 4.1.6 模块与第三方 SDK 配置

若需集成定位、地图、支付、推送等原生能力，需按以下步骤配置：

1. **依赖库配置**：参考 `Feature-Android.xls` 文档，将对应模块的 jar/aar 文件拷贝到 `app/libs` 目录
2. **权限配置**：参考文档的 `AndroidManifest.xml permission` 列，拷贝权限到项目的 `AndroidManifest.xml`
3. **properties 配置**：将模块的 `features` 和 `services` 节点拷贝到 `dcloud_properties.xml`
4. **第三方应用信息**：参考文档的 `AndroidManifest.xml Application node` 列，配置对应节点

各模块的详细配置文档请见：
- [定位 (geolocation)](https://nativesupport.dcloud.net.cn/AppDocs/usemodule/androidModuleConfig/geolocation.html)
- [消息推送 (Push)](https://nativesupport.dcloud.net.cn/AppDocs/usemodule/androidModuleConfig/push.html)
- [分享 (Share)](https://nativesupport.dcloud.net.cn/AppDocs/usemodule/androidModuleConfig/share.html)
- [登录鉴权 (OAuth)](https://nativesupport.dcloud.net.cn/AppDocs/usemodule/androidModuleConfig/oauth.html)
- [地图 (Map)](https://nativesupport.dcloud.net.cn/AppDocs/usemodule/androidModuleConfig/map.html)
- [支付 (Payment)](https://nativesupport.dcloud.net.cn/AppDocs/usemodule/androidModuleConfig/pay.html)
- [语音输入 (Speech)](https://nativesupport.dcloud.net.cn/AppDocs/usemodule/androidModuleConfig/speech.html)
- [统计 (Statistic)](https://nativesupport.dcloud.net.cn/AppDocs/usemodule/androidModuleConfig/statistic.html)
- [实人认证 (FacialRecognitionVerify)](https://nativesupport.dcloud.net.cn/AppDocs/usemodule/androidModuleConfig/faceRecognitionVerify.html)
- [广告 (uni-AD)](https://nativesupport.dcloud.net.cn/AppDocs/usemodule/androidModuleConfig/uniad.html)
- [腾讯 X5 Webview](https://nativesupport.dcloud.net.cn/AppDocs/usemodule/androidModuleConfig/x5.html)
- [uts 内置模块](https://nativesupport.dcloud.net.cn/AppDocs/usemodule/androidModuleConfig/uts.html)
- [其他模块及国际化配置](https://nativesupport.dcloud.net.cn/AppDocs/usemodule/androidModuleConfig/others.html)
- [第三方 SDK 依赖说明](https://nativesupport.dcloud.net.cn/AppDocs/usemodule/androidModuleConfig/android_Library.html)

#### 4.1.7 编译打包

- 点击 Android Studio 菜单 **Build** → **Generate Signed Bundle / APK**，按提示选择签名配置并生成 APK

参考详细教程：[Android 原生工程配置](https://nativesupport.dcloud.net.cn/AppDocs/usesdk/android.html)

### 4.2 iOS 平台

#### 4.2.1 工程初始配置

1. 用 Xcode 打开下载的 SDK 项目
2. 配置 Bundle ID（应用唯一标识），格式为反写域名（如 `com.example.appname`），与 Apple 开发者账号中申请的证书关联。
3. 导入 Apple 签名证书和 Profile 文件

#### 4.2.2 导入前端资源

- 将第 3 步生成的资源文件按 SDK 文档指引导入 Xcode 工程对应位置
- 通常在 `HBuilder` 或 `Pandora` 目录下

#### 4.2.3 修改配置

- 修改应用名称、图标（`AppIcon`）、启动图（LaunchImage）
- 配置 Info.plist 中的权限描述（如定位、相机、麦克风等）

#### 4.2.4 模块配置

各 iOS 模块详细配置请见：
- [定位 (geolocation)](https://nativesupport.dcloud.net.cn/AppDocs/usemodule/iOSModuleConfig/geolocation.html)
- [消息推送 (Push)](https://nativesupport.dcloud.net.cn/AppDocs/usemodule/iOSModuleConfig/push.html)
- [分享 (Share)](https://nativesupport.dcloud.net.cn/AppDocs/usemodule/iOSModuleConfig/share.html)
- [登录鉴权 (OAuth)](https://nativesupport.dcloud.net.cn/AppDocs/usemodule/iOSModuleConfig/oauth.html)
- [地图 (Map)](https://nativesupport.dcloud.net.cn/AppDocs/usemodule/iOSModuleConfig/map.html)
- [支付 (Payment)](https://nativesupport.dcloud.net.cn/AppDocs/usemodule/iOSModuleConfig/pay.html)
- [语音输入 (Speech)](https://nativesupport.dcloud.net.cn/AppDocs/usemodule/iOSModuleConfig/speech.html)
- [直播推流 (LivePusher)](https://nativesupport.dcloud.net.cn/AppDocs/usemodule/iOSModuleConfig/livepusher.html)
- [统计 (Statistic)](https://nativesupport.dcloud.net.cn/AppDocs/usemodule/iOSModuleConfig/statistic.html)
- [实人认证 (FacialRecognitionVerify)](https://nativesupport.dcloud.net.cn/AppDocs/usemodule/iOSModuleConfig/facialRecognitionVerify.html)
- [广告 (uni-AD)](https://nativesupport.dcloud.net.cn/AppDocs/usemodule/iOSModuleConfig/uniad.html)
- [iOS UIWebView](https://nativesupport.dcloud.net.cn/AppDocs/usemodule/iOSModuleConfig/uiwebview.html)
- [uts 内置模块](https://nativesupport.dcloud.net.cn/AppDocs/usemodule/iOSModuleConfig/uts.html)
- [第三方 SDK 依赖说明](https://nativesupport.dcloud.net.cn/AppDocs/usemodule/iOSModuleConfig/dependentLibrary.html)

#### 4.2.5 编译打包

- 在 Xcode 中选择目标设备为 `Any iOS Device`（或真机）
- 点击菜单 **Product** → **Archive** 进行归档
- 通过 Organizer 导出 `.ipa` 文件（可用于分发或上架 App Store）

参考详细教程：[iOS 原生工程配置](https://nativesupport.dcloud.net.cn/AppDocs/usesdk/ios.html)

### 4.3 鸿蒙平台

#### 4.3.1 工程初始配置

1. 用 DevEco Studio 打开鸿蒙 SDK 工程（下载地址：[鸿蒙开发指南](https://uniapp.dcloud.net.cn/harmony/dev.html)）
2. 配置 `app-harmony.projectPath`：在 `manifest.json` 中设置鸿蒙工程目录路径

#### 4.3.2 安装 SDK 依赖模块

修改项目根目录的 `oh-package.json5`：

```json
{
  "dependencies": {
    "@dcloudio/uni-app-x-runtime": "版本号（如 4.71.*）"
  }
}
```

点击 Sync Now。

#### 4.3.3 导入前端资源

1. HBuilderX 点击 **发行** → **App-Harmony-本地打包** → **生成本地打包 App 资源**，资源生成于 `unpackage/resources` 目录
2. 将生成资源拷贝到 `entry/src/main/resources/resfile/uni-app-x/apps/你的APPID/www`
3. 编辑 `entry/build-profile.json5`，在 `buildOption` 中添加 `arkOptions` → `runtimeOnly` → `sources` 配置

#### 4.3.4 集成内置模块与 UTS 插件

参考文档：[集成内置模块](https://doc.dcloud.net.cn/uni-app-x/native/modules/harmony/modules.html)、[集成 UTS 插件](https://doc.dcloud.net.cn/uni-app-x/native/use/harmony.html#集成-uts-插件)

#### 4.3.5 各模块独立配置

| 模块 | 文档链接 |
|------|---------|
| 登录鉴权 (OAuth) | [配置入口](https://nativesupport.dcloud.net.cn/AppDocs/usemodule/harmonyModuleConfig/oauth.html) |
| 地图 (Map) | [配置入口](https://nativesupport.dcloud.net.cn/AppDocs/usemodule/harmonyModuleConfig/map.html) |
| 支付 (Payment) | [配置入口](https://nativesupport.dcloud.net.cn/AppDocs/usemodule/harmonyModuleConfig/pay.html) |
| 实人认证 (FacialRecognitionVerify) | [配置入口](https://nativesupport.dcloud.net.cn/AppDocs/usemodule/harmonyModuleConfig/facialRecognitionVerify.html) |

> **版本说明**：HBuilderX 4.27+ 版本可直接通过发行菜单完成打包，HBuilderX 4.26 及更早版本需手动配置项目路径。

参考详细教程：[鸿蒙原生工程配置](https://nativesupport.dcloud.net.cn/AppDocs/usesdk/harmony.html)


## 五、打包发行

完成资源导入和工程配置后，执行以下操作生成最终安装包：

| 平台 | 输出格式 | 打包方式 |
|------|---------|---------|
| Android | APK / AAB | Android Studio → Build → Generate Signed Bundle/APK |
| iOS | IPA | Xcode → Product → Archive → Organizer 导出 |
| 鸿蒙 | HAP | DevEco Studio → Build → Build HAP(s) |

**打包格式说明**：
- **APK**：国内应用市场通用格式。
- **AAB**：Google Play 要求格式，HBuilderX 4.31+ 支持，不支持 adb 直接安装。
- **渠道包**：支持华为、OPPO、VIVO、小米、荣耀、应用宝等默认渠道，也可在 manifest.json 中自定义渠道。

参考官方教程：
- [Android 打包发行](https://nativesupport.dcloud.net.cn/AppDocs/package/android.html)
- [iOS 打包发行](https://nativesupport.dcloud.net.cn/AppDocs/package/ios.html)


## 六、常见问题排查

### Android

**Q：Gradle 编译失败，提示 JDK 版本不兼容**
A：检查 Android Studio 的 Gradle JDK 是否设置为 JDK 1.8。

**Q：运行时报错“AppKey 无效”**
A：检查 AndroidManifest.xml 中的 `dcloud_appkey` 是否配置正确，AppID 与资源文件夹名称是否一致。

**Q：第三方 SDK 集成后无法正常工作**
A：确认 so 库是否仅添加了 `armeabi-v7a`、`arm64-v8a`、`x86` 三个文件夹，避免兼容性问题。

**Q：能否使用 UIWebView？**
A：官方推荐使用 WKWebView，iOS 已从 2020 年 4 月起不再接收使用 UIWebView 的新应用，建议尽早迁移到 WKWebView 内核。

### iOS

**Q：打包报错“Provisioning profile 不匹配”**
A：检查 Bundle ID 与 Apple 开发者账号中申请的证书和 Profile 是否一致。确认 Apple 开发者账号的有效期，避免证书过期。

**Q：推送功能无法收到通知**
A：确认推送证书配置正确，检查推送模块在 manifest.json 中是否已勾选。iOS 端需验证 `aps-environment` 权限是否已正确授权。

**Q：模拟器编译正常但真机启动闪退**
A：检查 Xcode 签名配置中是否勾选了真机调试的开发者证书，且设备 UDID 已注册到开发者账号。

**Q：本地打包是否支持付费原生插件？**
A：不支持，本地打包无法使用付费原生插件，付费插件仅支持云端打包。

### 鸿蒙

**Q：运行时提示 SDK 版本不匹配**
A：确保 `oh-package.json5` 中配置的 `@dcloudio/uni-app-x-runtime` 版本与 HBuilderX 导出资源时的版本一致，最低版本为 4.71。

**Q：如何触发本地打包流程的增量更新？**
A：以下操作会触发首次打包流程（即完整原生代码包下载）：修改 App 模块配置、修改应用名称/包名/证书、修改权限配置、使用 uni 原生插件、更新 HBuilderX。


## 七、平台注意事项与 FAQ

- [Android 注意事项](https://nativesupport.dcloud.net.cn/AppDocs/FAQ/android.html)
- [iOS 注意事项](https://nativesupport.dcloud.net.cn/AppDocs/FAQ/ios.html)

### 通用最佳实践

1. **版本对齐**：导出资源使用的 HBuilderX 版本应与下载的 SDK 版本相匹配，避免因版本不兼容导致功能异常。
2. **证书安全**：签名证书私钥请妥善保管，避免泄露。正式证书和开发证书分离管理，发布证书由专人掌管。
3. **调试优先云打包**：开发调试阶段优先使用云打包，正式发版阶段如需深度定制再切本地打包。
4. **隐私合规**：务必使用最新版本 SDK，老版本可能未适配最新的隐私合规要求。


## 附录：常用命令参考

### Android Studio 常用 Gradle 命令

| 命令 | 说明 |
|------|------|
| `./gradlew assembleDebug` | 编译调试版 APK |
| `./gradlew assembleRelease` | 编译正式版 APK |
| `./gradlew clean` | 清理编译缓存 |
| `./gradlew bundleRelease` | 编译正式版 AAB |

### iOS Xcode 常用命令

| 操作 | 路径/命令 |
|------|---------|
| 清理工程 | Product → Clean Build Folder |
| 编译工程 | Product → Build |
| 运行调试 | Product → Run |
| 打包归档 | Product → Archive |

### 鸿蒙 DevEco Studio 常用操作

| 操作 | 方式 |
|------|------|
| 安装依赖 | 编辑 oh-package.json5 后点击 Sync Now |
| 编译 HAP | Build → Build HAP(s) |
| 运行调试 | 连接真机后点击运行图标 |