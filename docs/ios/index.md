# iOS UniApp SDK 集成指南 (基于 HBuilderX 5.0+)

## 1. 概述与核心概念

UniApp iOS SDK（即 DCloud App 离线开发工具包 - iOS 平台）是 DCloud 提供的官方原生开发工具包。它将 UniApp 应用的运行环境封装为原生开发接口，方便开发者在自己的 iOS 原生项目中直接集成并运行 UniApp 应用，实现了 App 本地离线打包及扩展原生能力。

**App 本地离线打包**：对应 HBuilderX 的云端打包功能，在打包时无需将 App 资源及打包要使用的签名证书等提交到云端打包服务器，直接在开发者本地配置的原生开发环境中生成安装包 ipa。

**扩展原生能力**：当 HBuilderX 中提供的能力无法满足 App 功能需求时，可以使用 App 离线 SDK 开发原生插件来扩展原生能力。

**与 unipack-tool 的关系**：本工具（unipack-tool）自动化了以下 iOS 禺线打包流程：
- 从 HBuilder-Hello 模板复制工程
- 自动修改 `project.pbxproj`（Bundle ID）
- 自动修改 `Info.plist`（Appkey、应用名称、版本号等）
- 导入 UniApp 资源到 `Pandora/apps/`
- 修改 `control.xml` 的 appid
- 处理 UTS 原生插件的 framework
- 生成并配置各尺寸 App Icon
- 安装描述文件和 P12 证书
- 执行 `xcodebuild archive` 和 `exportArchive`

---

## 2. 开发环境准备

在开始集成之前，需要确保您的本地开发环境满足以下要求：

- **开发工具**：
  - **Xcode 15 及以上版本**（从 App Store 或 Apple Developer 下载）
  - **HBuilderX**（[下载地址](https://www.dcloud.io/hbuilderx.html)）
- **操作系统**：macOS（iOS 开发仅支持 macOS）
- **Apple Developer 账号**：用于签名和真机调试 / App Store 发布
- **App 离线 SDK**：[最新 iOS 平台 SDK 下载](https://nativesupport.dcloud.net.cn/AppDocs/download/ios.html)

> **版本一致性要求（至关重要）**
>
> 请确保从 HBuilderX 导出的打包资源的 **HBuilderX 版本号** 和 **App 离线 SDK 发布的版本号是一致的**。
>
> ![](https://aka.doubaocdn.com/s/uItV1wZnFv) ![](https://aka.doubaocdn.com/s/srqn1wZnFv)
>
> **注意：如果版本不一致，app 启动时会弹出版本不一致的提示框，并且可能导致功能异常**

> **Appkey 要求（3.1.10 版本起）**
>
> 从 **3.1.10 版本起**需要申请 Appkey，具体请点击 [Appkey 申请指南](https://nativesupport.dcloud.net.cn/AppDocs/usesdk/appkey.html)

---

## 3. SDK 目录说明

从 UniApp 官方网站下载最新版的 iOS 离线 SDK 并解压。SDK 包解压后主要包含以下关键目录和文件：

```
|-- HBuilder-Hello        # 给用户打 uni-app 项目的离线打包工程（Xcode 工程）
|-- Feature-iOS.xls       # 配置表（依赖的库、资源文件、参数配置等）
|-- SDK                   # 工程需要的库文件、.h 头文件、配置文件、资源文件
```

- **HBuilder-Hello**：核心工程目录，是一个完整的 Xcode 项目，已预配置好 uni-app 运行所需的基本设置。离线打包以此工程为基础进行配置。
- **Feature-iOS.xls**：详细的模块配置表，记录了各功能模块所依赖的库文件、资源文件、plist 配置项等。在集成特定功能模块时需要参考此表。
- **SDK**：包含运行时所需的 `.framework` 动态库/静态库、`.h` 头文件、配置文件和资源文件。

> 详细说明可参考：[App 离线 SDK 内不同文件的作用](https://nativesupport.dcloud.net.cn/AppDocs/usesdk/iOSReadMe.html)

---

## 4. 工程配置与集成步骤

找到 App 离线 SDK 压缩包并解压，进入目录后找到 `HBuilder-Hello` 文件夹，使用 Xcode 打开该原生工程，然后按以下步骤逐一配置。

### 4.1 配置 Appkey（3.1.10 版本起必须）

从 3.1.10 版本起需要申请 Appkey，申请请 [参考这里](https://nativesupport.dcloud.net.cn/AppDocs/usesdk/appkey.html)。

**配置方法**：

打开工程的 `Info.plist` 文件，添加 key 为 `dcloud_appkey` 的条目，Value 类型选择 String，内容为申请到的 AppKey：

```xml
<key>dcloud_appkey</key>
<string>您申请的AppKey</string>
```

![](https://aka.doubaocdn.com/s/mIH91wZnFv)

> **重要提醒**：需要同时确保「应用标识（Bundle Identifier）」以及「导入资源教程中 control.xml 对应的 appid」已修改为正确的值，否则运行时仍会提示 appkey 错误。

### 4.2 配置应用标识（Bundle Identifier）

选择左侧应用工程根目录，选中 **TARGETS** 下的 **HBuilder** 打开工 程属性界面，在 **General** 下修改 **Identity** 区域的 **Bundle Identifier** 值：

![](https://aka.doubaocdn.com/s/hvOK1wZnFv)

| 配置项 | 说明 | 推荐值 |
|--------|------|--------|
| **Bundle Identifier** | 苹果的 AppID | 必须与应用发布时配置的 Profile 关联的 AppID 完全一致 |
| **Version**（CFBundleShortVersionString） | 应用版本名称，在 App Store 中显示 | 推荐与 manifest.json 中 `version.name` 值一致 |
| **Build**（CFBundleVersion） | 编译版本号，App Store 判断升级使用 | 推荐与 manifest.json 中 `version.code` 值一致 |

### 4.3 配置应用名称

将原生工程的 **Display Name** 与 manifest.json 中的 `name` 字段保持一致：

1. 在 Xcode 中选中工程的 Targets
2. 将 **Display Name** 修改为 manifest.json 里 `"name"` 字段的内容

manifest.json 里的 `"name"` 对应的是 HBuilderX 工程里「基础配置」中的「应用名称」。

![](https://aka.doubaocdn.com/s/aEtH1wZnFv) ![](https://aka.doubaocdn.com/s/BGFP1wZnFv)

对应的 Info.plist 键值对：

```xml
<key>CFBundleDisplayName</key>
<string>您的应用名称</string>
```

### 4.4 配置应用版本名称（Version）

将原生工程的 **Version** 与 manifest.json 中 `version.name` 保持一致：

- manifest.json 中 `version.name` 对应 HBuilderX「基础配置」里的「应用版本名称」
- Xcode 中位置：Targets → General → Identity → Version

![](https://aka.doubaocdn.com/s/cQJp1wZnFv) ![](https://aka.doubaocdn.com/s/t31D1wZnFv)

对应的 Info.plist 键值对：

```xml
<key>CFBundleShortVersionString</key>
<string>1.0.0</string>
```

### 4.5 配置应用版本号（Build）

将原生工程的 **Build** 与 manifest.json 中 `version.code` 保持一致：

- manifest.json 中 `version.code` 对应 HBuilderX「基础配置」里的「应用版本号」
- Xcode 中位置：Targets → General → Identity → Build

![](https://aka.doubaocdn.com/s/vmkL1wZnFv) ![](https://aka.doubaocdn.com/s/AsBv1wZnFv)

对应的 Info.plist 键值对：

```xml
<key>CFBundleVersion</key>
<string>100</string>
```

### 4.6 配置应用的图标

iOS 应用需要提供多种尺寸的图标以适配不同场景（主屏幕、Spotlight 搜索、设置等）。

**配置步骤**：

1. 点击 Xcode 左侧 **Project** → 选择 Target → **General** → **App Icons and Launch Images**
2. 点击 **App Icons Source** 右侧的小箭头进入 Asset Catalog

![](https://aka.doubaocdn.com/s/u0A11wZnFv)

3. 在新开的页面中，根据提示将对应尺寸的应用图标拖入到虚线框中即可

![](https://aka.doubaocdn.com/s/cyU51wZnFv)

**所需图标尺寸一览**：

| 用途 | 尺寸 | scale | 文件名示例 |
|------|------|-------|-----------|
| iPhone 通知中心 | 20pt | @2x/@3x | Icon-iphone-20@2x.png / Icon-iphone-20@3x.png |
| iPhone 设置 | 29pt | @2x/@3x | Icon-iphone-29@2x.png / Icon-iphone-29@3x.png |
| iPhone Spotlight | 40pt | @2x/@3x | Icon-iphone-40@2x.png / Icon-iphone-40@3x.png |
| iPhone App 主图标 | 60pt | @2x/@3x | Icon-iphone-60@2x.png / Icon-iphone-60@3x.png |
| iPad 通知中心 | 20pt | @1x/@2x | Icon-ipad-20.png / Icon-ipad-20@2x.png |
| iPad 设置 | 29pt | @1x/@2x | Icon-ipad-29.png / Icon-ipad-29@2x.png |
| iPad Spotlight | 40pt | @1x/@2x | Icon-ipad-40.png / Icon-ipad-40@2x.png |
| iPad App 主图标 | 76pt | @1x/@2x | Icon-ipad-76.png / Icon-ipad-76@2x.png |
| iPad Pro App | 83.5pt | @2x | Icon-ipad-83.5@2x.png |
| App Store | 1024pt | @1x | Icon-1024.png |

### 4.7 配置应用启动界面

#### 方式一：Launch Screen File（推荐）

1. 在 Xcode 中配置 **Launch Screen File**，这样配置之后启动界面就会是设置的 LaunchScreen.storyboard：

![](https://aka.doubaocdn.com/s/tMOi1wZnFv)

2. 官方提供了 2 个 storyboard 模板：
   - **图标、名称在上方的**：标准布局
   - **图标、名称在下方的**：适配用户配置广告后的场景，使启动时不会有视觉上的跳跃感

   > **注意**：在使用 Launch Screen File 方式作为启动界面时，需要将一张或几张清晰的图标拷贝到工程的根文件夹下并引入到工程中，用来给启动界面加载图标。如果拷贝过去的图标不清晰，会导致启动界面上出现图标模糊的现象。

![](https://aka.doubaocdn.com/s/IEpe1wZnFv) ![](https://aka.doubaocdn.com/s/FlxB1wZnFv)

3. 自定义 LaunchScreen.storyboard（可选）：
   - 如果想自定义 LaunchScreen.storyboard，需要具备原生开发知识
   - 需要知道怎样创建 Launch Screen File、怎样在 storyboard 中布局视图、设置约束等

> **注意**：
> - 这里的 storyboard 不是普通的 storyboard
> - 配置了广告之后，如果自定义的 LaunchScreen.storyboard 约束没设置好，会有启动页到广告页跳跃的视觉效果

### 4.8 配置开屏广告底部 Logo

如果使用了 uni-ad 广告模块并需要在开屏广告底部显示品牌 logo，操作如下：

将需要展示的图片命名为 `dcloud_logo` 并添加到项目资源（Assets.xcassets）中即可。

### 4.9 配置国际化

国际化包含两部分：

#### 第一部分：内容相关的国际化

离线打包时如果弹出提示框且内容为 "HTML5+ Runtime D" 时，需要在打包的原生工程里配置国际化。[如何配置](https://ask.dcloud.net.cn/article/35963)

#### 第二部分：Info.plist 的国际化

1. 新建一个 `.strings` 文件，命名为 `InfoPlist.strings`（**文件名必须是这个**）
2. 点击右侧的 **Localized** 进行本地化
3. 在工程导航界面选择 `InfoPlist.strings` 文件，添加 key-value 对：
   - 例如添加 key 为 `CFBundleDisplayName`，值为应用名字
   - `InfoPlist.strings (English)` 对应英文系统
   - `InfoPlist.strings (Simplified)` 对应中文简体系统

![](https://aka.doubaocdn.com/s/e4WL1wZnFv) ![](https://aka.doubaocdn.com/s/78UQ1wZnFv)

#### 隐私权限描述的国际化

对于 manifest.json 中「模块权限配置」→「iOS 隐私信息访问的许可描述」中的隐私权限描述，可以按如下方式国际化：

1. 在 HBuilderX 中切换到「模块权限配置」，在「iOS 隐私信息访问的许可描述」栏下配置需要的隐私描述信息：

![](https://aka.doubaocdn.com/s/ZnEU1wZnFv)

2. 切换到代码视图，在 `app-plus → distribute → ios → privacyDescription` 节点下可看到输入的内容：

![](https://aka.doubaocdn.com/s/KuJ21wZnFv)

3. 将 `privacyDescription` 节点下的 key（如 `NSPhotoLibraryUsageDescription`）和 value 按下图方式拷贝到 `InfoPlist.strings` 下对应的语言文件里：

![](https://aka.doubaocdn.com/s/fgv31wZnFv) ![](https://aka.doubaocdn.com/s/CITH1wZnFv)

> 完整可配置的隐私项可参考 [苹果官网](https://developer.apple.com/documentation/bundleresources/information_property_list) 中以 **NS** 开头、**Description** 结尾的项。

### 4.10 配置多渠道

在需要打包的原生工程中找到配置文件 `Info.plist`，然后添加 `marketChannel` 节点：

```xml
<key>marketChannel</key>
<string>包名|应用标识|广告标识|渠道</string>
```

**字段说明**：

| 字段 | 说明 | 示例 |
|------|------|------|
| 包名 | 对应 Xcode 里的 Bundle ID | `io.dcloud.HBuilder` |
| 应用标识 | 对应 uni-app 项目 manifest.json 中的 appid | `__UNIXXXXXX` |
| 广告标识 | DCloud 的广告标识，开通广告后在 dev.dcloud.net.cn 获取；未开通则留空 | （空字符串） |
| 渠道 | 固定填写 | `apple` |

**示例**：
```xml
<key>marketChannel</key>
<string>io.dcloud.HBuilder|__UNIXXXXXX||apple</string>
```

### 4.11 配置暗黑模式

#### 方式一：通过 UIUserInterfaceStyle 控制（全局）

在 `Info.plist` 中操作 `UIUserInterfaceStyle` 节点：

| 操作 | 效果 |
|------|------|
| **移除**该节点 | 支持跟随系统切换亮/暗模式（默认行为） |
| 添加该节点，值设为 `Light`（String 类型） | 始终显示高亮模式 |
| 添加该节点，值设为 `Dark`（String 类型） | 始终显示暗夜模式 |

#### 方式二：通过 DCloudConfig.defaultTheme 设置启动默认主题

在 `Info.plist` 中添加 `DCloudConfig` 节点（类型为 Dictionary，如已有则无需重复添加），在该节点下添加 `defaultTheme` 子节点（值类型为 String）：

```xml
<key>DCloudConfig</key>
<dict>
    <key>defaultTheme</key>
    <string>auto</string>
</dict>
```

| 可选值 | 效果 |
|--------|------|
| `light` | 高亮模式 |
| `dark` | 暗夜模式 |
| `auto` | 跟随系统 |

### 4.12 配置 IDFA（广告标识符）

#### 是否需要配置 IDFA？

如果您的应用符合下面其中 **一条** 就需要配置 IDFA，反之可以不配置：

- 应用了 **uni-AD 广告模块** → 必须开启 IDFA
- 使用 **离线 SDK 版本低于 3.2.15** 并且使用了 **新浪微博登录/分享、一键登录、友盟统计** 其中一个或多个功能模块 → 这些旧版 SDK 内会触发获取 IDFA
  - 注：HX 3.2.15 及以上版本已更新这些三方 SDK，不再获取 IDFA

#### 配置步骤

**第一步：链接库文件**

在 Xcode 工程中，进入 `TARGETS → Build Phases → Link Binary With Libraries`：

1. 点击 **+** 按钮
2. 选择 `Add Other → Add Files...`
3. 将 `SDK/Resources/Libs` 中的以下两个文件添加到工程：
   - `libAdSupport.a`
   - `AppTrackingTransparency.framework`（**系统库**，Status 选择 **Optional**）

![](https://aka.doubaocdn.com/s/rVDX1wZnFv)

**第二步：添加权限描述**

在工程的 `Info.plist` 中添加 `NSUserTrackingUsageDescription` 权限描述：

```xml
<key>NSUserTrackingUsageDescription</key>
<string>您的权限描述文字，说明为何需要追踪用户数据</string>
```

详情请参考 [iOS 平台配置应用使用广告标识（IDFA）](https://ask.dcloud.net.cn/article/36107)。

![](https://aka.doubaocdn.com/s/d4gC1wZnFv)

**第三步：App Store Connect 配置隐私**

开启 IDFA 后，提交 App Store 审核之前，需要在 **App Store Connect** 配置「App 隐私」。详情参考同上文档。

### 4.13 配置启动时是否注册 Push

#### 场景一：不希望在启动时弹出「发送通知」系统授权框

找到工程里的 `Info.plist` 文件，配置 `dcloud_push_register_mode` 字段，值为 `manual`：

```xml
<key>dcloud_push_register_mode</key>
<string>manual</string>
```

![](https://aka.doubaocdn.com/s/jG0p1wZnFv)

#### 场景二：希望在启动时弹出授权框

不需要做任何额外配置，默认行为即为启动时请求推送权限。

> 参考：[iOS 平台隐私与政策提示框实现注意问题](https://ask.dcloud.net.cn/article/36955)

---

## 5. 制作自定义调试基座

如果需要在真机上使用自定义基座进行调试（支持热更新、断点调试等），需要按以下步骤制作自定义调试基座 IPA：

### 步骤清单

1. **修改 control.xml**：在打包原生工程里找到 `control.xml` 文件，在 HBuilder 节点中确认有以下两项配置：
   ```xml
   <hbuilder debug="true" syncDebug="true">
   ```
   > **注意**：打 App Store 包的时候，这个配置需要去掉，否则会导致热更新失败！

   ![](https://aka.doubaocdn.com/s/t7ds1wZnFv)

2. **确认 Bundle Identifier**：确保 Xcode 工程的 Bundle identifier **不为** `io.dcloud.HBuilder`

3. **修改 Info.plist**：在 info.plist 文件中增加调试相关配置项：

   ![](https://aka.doubaocdn.com/s/GmII1wZnFv)

4. **确认 apps 目录**：确保原生工程里 `Pandora/apps` 文件夹下 **只有一个文件夹**，且文件夹名称和里面的 manifest 的 `id` 值相同

5. **确认 appid 一致**：确保 `control.xml` 文件里的 `appid` 的值和 `apps` 目录下的第一个文件夹的名称一致

6. **确认 HBuilderX 工程 appid**：确保 HBuilderX 里要调试的代码的 appid 和 control.xml 的 appid 值一致

7. **Archive 打包**：使用 Xcode 的 **Product → Archive** 打包，然后生成 ipa，并将 ipa 重命名为 `iOS_debug.ipa`

8. **放置 debug 包**：在 JS 工程主目录下新建 `unpackage/debug/` 文件夹（如果没有的话），把生成的 `iOS_debug.ipa` 放入该目录：

   ![](https://aka.doubaocdn.com/s/etub1wZnFv)

9. **运行调试**：在 HBuilderX 中找到之前 appid 相同的 JS 工程，点击 **运行 → 运行到手机或模拟器 → 使用自定义基座运行（iOS）**，等待连接成功即可开始调试

---

## 6. 隐私清单（Privacy Manifest）

从 **SDK 4.13 之后**的版本，示例工程中已经包含了基础模块的隐私清单（Privacy Manifest）文件。

**关键时间节点**：从 **2024 年春季**开始，所有提交到 App Store 的应用都需要包含 Privacy Manifest 文件。

Privacy Manifest 主要声明以下三类信息：

| 类别 | 说明 |
|------|------|
| **NSPrivacyCollectedDataTypes** | 收集的用户数据类型 |
| **NSPrivacyTracking** | 是否跟踪用户活动 |
| **NSPrivacyTrackingDomains** | 跟踪的域名列表 |
| **NSPrivacyAccessedAPITypes** | 使用的必需理由 API（Required Reason API） |

如果您的应用涉及以下 API 调用，需要在 Privacy Manifest 中声明使用原因：

- 文件时间戳 API
- 默认日历 API
- 系统启动时间 API
- 磁盘空间 API
- 用户默认值 API
- 其他苹果标记为 Required Reason 的 API

**PrivacyManifest.xml 示例**：

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
  "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>NSPrivacyTracking</key>
    <false/>
    <key>NSPrivacyTrackingDomains</key>
    <array/>
    <key>NSPrivacyCollectedDataTypes</key>
    <array/>
    <key>NSPrivacyAccessedAPITypes</key>
    <array>
        <dict>
            <key>NSPrivacyAccessedAPIType</key>
            <string>NSPrivacyAccessedAPICategoryFileTimestamp</string>
            <key>NSPrivacyAccessedAPITypeReasons</key>
            <array>
                <string>C617.1</string>
            </array>
        </dict>
    </array>
</dict>
</plist>
```

---

## 7. Info.plist 配置速查表

以下是 iOS 离线打包工程中所有可能涉及的 Info.plist 配置项的一站式参考：

### 7.1 基础配置（必填）

| Key | 类型 | 说明 | 来源 |
|-----|------|------|------|
| `dcloud_appkey` | String | DCloud AppKey（3.1.10+ 必填） | [Appkey 申请](https://nativesupport.dcloud.net.cn/AppDocs/usesdk/appkey.html) |
| `CFBundleDisplayName` | String | 应用显示名称 | manifest.json → name |
| `CFBundleShortVersionString` | String | 版本名称（如 1.0.0） | manifest.json → version.name |
| `CFBundleVersion` | String | 构建版本号（如 100） | manifest.json → version.code |

### 7.2 功能开关配置

| Key | 类型 | 可选值 | 说明 |
|-----|------|--------|------|
| `dcloud_push_register_mode` | String | `manual` / 不配置 | 设为 manual 则启动时不弹出推送授权框 |
| `UIUserInterfaceStyle` | String | `Light` / `Dark` / 移除 | 全局亮色/暗色/跟随系统 |
| `marketChannel` | String | `包名\|appid\|adid\|渠道` | 多渠道统计配置 |

### 7.3 DCloud 扩展配置

| Key | 类型 | 说明 |
|-----|------|------|
| `DCloudConfig` → `defaultTheme` | String | 启动默认主题：`light` / `dark` / `auto` |

### 7.4 权限与隐私配置

| Key | 类型 | 说明 |
|-----|------|------|
| `NSUserTrackingUsageDescription` | String | IDFA 使用说明（使用广告/追踪时必填） |
| `NSPhotoLibraryUsageDescription` | String | 相册访问权限说明 |
| `NSCameraUsageDescription` | String | 摄像头访问权限说明 |
| `NSLocationWhenInUseUsageDescription` | String | 使用期间定位权限说明 |
| `NSLocationAlwaysAndWhenInUseUsageDescription` | String | 始终定位权限说明 |
| `NSMicrophoneUsageDescription` | String | 麦克风访问权限说明 |
| `NSBluetoothAlwaysUsageDescription` | String | 蓝牙始终使用权限说明 |
| `NSBluetoothPeripheralUsageDescription` | String | 蓝牙外设权限说明 |
| `NSFaceIDUsageDescription` | String | Face ID 使用权限说明 |
| `NSContactsUsageDescription` | String | 通讯录访问权限说明 |

> 更多隐私权限 key 请参考 [苹果官方文档](https://developer.apple.com/documentation/bundleresources/information_property_list)，所有以 **NS** 开头、**Description** 结尾的键均为隐私权限描述。

### 7.5 网络安全配置（ATS）

| Key | 类型 | 说明 |
|-----|------|------|
| `NSAppTransportSecurity` → `NSAllowsArbitraryLoads` | Boolean | 允许所有 HTTP 请求（仅建议开发环境使用） |
| `NSAppTransportSecurity` → `NSExceptionDomains` | Dictionary | 按域名配置 HTTP 例外（生产推荐） |

**生产环境 ATS 例外配置示例**：
```xml
<key>NSAppTransportSecurity</key>
<dict>
    <key>NSExceptionDomains</key>
    <dict>
        <key>your-api-domain.com</key>
        <dict>
            <key>NSIncludesSubdomains</key>
            <true/>
            <key>NSExceptionAllowsInsecureHTTPLoads</key>
            <true/>
            <key>NSExceptionMinimumTLSVersion</key>
            <string>TLSv1.2</string>
        </dict>
    </dict>
</dict>
```

---

## 8. 注意事项与常见问题

详细的常见问题排查请参阅 [FAQ 文档](./faq.md)，以下是一些核心要点的快速索引：

| 问题类型 | 快速排查方向 |
|---------|-------------|
| 资源未更新 | 检查 `control.xml` 的 `syncDebug` 是否为 true |
| Swift 编译报错 | 工程需添加 Swift 环境（创建空 Swift 文件或配置 Build Settings） |
| 多架构 framework 报错 | Validate Workspace 设为 Yes，或使用 lipo 分离架构 |
| Xcode 15 链接错误 | Other Linker Flags 添加 `-ld_classic` |
| ATT 权限弹窗 | iOS 14.5+ 访问 IDFA 必须先请求用户授权 |
| iOS 17 审核被拒 | 检查是否包含 Privacy Manifest |
| 证书/签名错误 | 检查 Bundle ID 与证书匹配、Provisioning Profile 是否过期 |
| 内存崩溃 | 及时释放 WebView、合理管理图片缓存 |
| 网络 SSL 错误 | iOS 9+ 默认禁止 HTTP，需配置 ATS 例外或升级 HTTPS |

---

## 9. 扩展功能配置指引

以下功能的详细配置请参考各自独立的文档：

| 功能 | 配置文档 |
|------|---------|
| **uni-ad 广告模块** | [如何配置广告](https://nativesupport.dcloud.net.cn/AppDocs/usemodule/iOSModuleConfig/uniad.html) |
| **3D Touch** | [配置 3D Touch](https://nativesupport.dcloud.net.cn/AppDocs/usesdk/iosOther/3DTouch.html) |
| **平台特殊功能** | [平台功能配置](https://nativesupport.dcloud.net.cn/AppDocs/usesdk/iosOther/project.html) |
| **审核被拒处理（其他支付/隐藏功能）** | [点击查看处理办法](https://ask.dcloud.net.cn/article/36447) |

---

## 10. 各模块配置文档索引

iOS 平台各功能模块的详细配置说明：

- [Geolocation（定位）](./modules/geolocation.md) — 地理定位能力集成
- [Map（地图）](./modules/map.md) — 地图显示与交互
- [Payment（支付）](./modules/payment.md) — 支付能力集成（微信/支付宝/IAP）
- [OAuth（登录鉴权）](./modules/oauth.md) — 第三方登录（微信/Apple/Google 等）
- [Push（推送）](./modules/push.md) — 消息推送（APNs/厂商推送）
- [Share（分享）](./modules/share.md) — 社交分享能力
- [Speech（语音识别）](./modules/speech.md) — 语音识别与合成
- [Statistic（统计）](./modules/statistic.md) — 统计分析（友盟等）
- [FaceRecognitionVerify（实人认证）](./modules/facial-recognition-verify.md) — 人脸识别认证
- [LivePusher（直播推流）](./modules/livepusher.md) — 直播推流能力
- [Native Plugins（原生插件）](./modules/native-plugins.md) — 原生插件开发指南
- [Third Party Dependencies（第三方依赖）](./modules/third-party-dependencies.md) — 第三方 SDK 依赖说明
- [uni-ad（广告）](./modules/uni-ad.md) — uni-ad 广告模块配置
- [UTS Built-in Modules（UTS 内置模块）](./modules/uts-builtin-modules.md) — UTS 内置原生模块
- [UIWebView（WebView）](./modules/uiwebview.md) — WebView 相关配置

---

## 11. 更多资源

- **DCloud iOS 离线 SDK 官方文档**：https://nativesupport.dcloud.net.cn/AppDocs/usesdk/ios.html
- **iOS 离线 SDK 下载页**：https://nativesupport.dcloud.net.cn/AppDocs/download/ios.html
- **AppKey 申请指南**：https://nativesupport.dcloud.net.cn/AppDocs/usesdk/appkey.html
- **SDK 内文件作用详解**：https://nativesupport.dcloud.net.cn/AppDocs/usesdk/iOSReadMe.html
- **UniApp 资源导入教程（iOS）**：https://nativesupport.dcloud.net.cn/AppDocs/importfeproject/ios
- **FAQ 文档**：[./faq.md](./faq.md)
- **iOS 模块配置总览**：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/iOSModuleConfig/
