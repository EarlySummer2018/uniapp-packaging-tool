# UTS 插件构建流程（iOS）

> **适用版本**：HBuilderX 5.0+（普通 UniApp iOS 离线打包流程）
> **平台**：iOS (iPhone/iPad)
> **官方文档**：https://doc.dcloud.net.cn/uni-app-x/native/use/iosuts.html

---

## 概述

UTS（UniTypeScript）插件可以通过 iOS 原生产物接入离线工程。本文档面向 UniPack Tool 的普通 UniApp iOS 打包流程：自动识别导出资源中的 `uni_modules/*/utssdk/app-ios` 或 `app-iOS`，并把插件已提供的 framework、资源、系统库和 plist 配置接入主工程。

> **当前边界**：官方页面标题属于 uni-app x 文档，但其中“传统 UniApp 项目”表格才是本工具的依据。本工具不接入 `DCloudUniappRuntime.xcframework`、`DCloudUTSExtAPI.xcframework`、`UTSOC.h/mm`，也不在主工程生成 `uts-config.json`。

### 适用场景

- 需要调用 iOS 系统 API（如 CoreLocation、AVFoundation 等）
- 需要集成第三方原生 SDK（如支付宝、微信、地图等）
- 需要实现高性能的原生功能模块
- 标准 UniApp API 无法满足的业务需求

### 与传统原生插件的对比

| 特性 | 传统原生插件 | UTS 插件 |
|------|------------|---------|
| **开发语言** | Objective-C / Swift | UTS (TypeScript 超集) |
| **分发形式** | 编译好的 .framework / .a | UTS 源码 + 配置文件 |
| **构建方式** | 直接集成到工程 | 需本地编译为 xcframework |
| **跨平台能力** | 仅单平台 | 可跨平台（Android/iOS） |
| **文档参考** | [native-plugins.md](./native-plugins.md) | 本文档 |

> **重要提示**：付费 uts 插件（普通授权版）不支持原生工程接入。如果导出的资源文件不包含 `uni_modules`，或 `uni_modules` 中的插件均不包含 `app-ios` 目录，可以跳过本文档。

---

## 前置条件

### 开发环境要求

- **Xcode**：15.0 及以上版本
- **macOS**：仅支持 macOS 系统（iOS 开发限制）
- **Apple Developer 账号**：用于签名和真机调试
- **UniApp 离线 SDK**：与 HBuilderX 版本一致的 iOS SDK

### 版本一致性要求

⚠️ **至关重要**：请确保从 HBuilderX 导出的打包资源的 **HBuilderX 版本号** 和 **App 离线 SDK 发布的版本号完全一致**。

版本不一致会导致：
- 应用启动时弹出版本不一致提示框
- 功能异常或崩溃
- UTS 运行时错误

---

## 1. 新建原生插件工程

根据 HBuilderX 导出资源文件中 `uni_modules` 目录下的插件列表，为每个需要集成的 UTS 插件创建对应的 Xcode Framework 工程。

### 操作步骤

1. **启动 Xcode**，点击菜单栏 `File → New → Project...`
2. 在模板选择界面，选择 **Framework** 类型，点击 **Next**
   ![](https://aka.doubaocdn.com/s/W6Ue1wcexd)
3. **配置工程信息**：

   | 配置项 | 填写规则 |
   |--------|---------|
   | **Product Name** | `unimodule` + 插件名称（驼峰转换） |
   | **Organization Name** | 您的组织名称 |
   | **Organization Identifier** | 您的 Bundle ID 前缀 |
   | **Language** | **Objective-C**（必须选择） |

4. **Product Name 命名规则示例**：

   | 插件 ID | 工程名 |
   |---------|--------|
   | `uni-getbatteryinfo` | `unimoduleUniGetbatteryinfo` |
   | `uni-camera` | `unimoduleUniCamera` |
   | `my-custom-plugin` | `unimoduleMyCustomPlugin` |

   > **转换规则**：将插件 ID 中的 `-` 分隔符去掉，后续单词首字母大写（驼峰命名），前缀统一添加 `unimodule`

5. 选择保存路径，点击 **Create** 完成工程创建

---

## 2. 插件工程环境配置

创建完 Framework 工程后，需要对工程的 Build Settings 进行详细配置。

### 2.1 Build Settings 配置项

在 Xcode 左侧选择工程根目录 → 选择 **TARGETS** 下的插件 Target → 切换到 **Build Settings** 标签页，按以下表格逐一配置：

| 配置项 | 设置路径 | 推荐值 | 说明 |
|--------|---------|--------|------|
| **Minimum Deployments** | Target → General | `12.0` | 最低支持 iOS 版本 |
| **Mach-O Type** | Build Settings → 搜索 "Mach-O" | `Dynamic Library` | 必须设为动态库 |
| **Other Linker Flags** | Build Settings → 搜索 "Other Linker" | `-ObjC` | ⚠️ 字母 O 和 C 必须大写 |
| **Build Libraries for Distribution** | Build Settings → 搜索 "Distribution" | `YES` | 允许分发构建产物 |
| **Enable Module Verifier** | Build Settings → 搜索 "Module Verifier" | `NO` | 禁用模块验证器 |
| **Framework Search Paths** | Build Settings → 搜索 "Framework Search" | 添加 SDK/Libs 目录 | 双击展开后拖入文件夹 |

### 2.2 配置操作详解

#### Minimum Deployments
- 路径：Target → General → Deployment Info → Deployment Target
- 设置为 `12.0` 或更高（若插件要求更高版本需同步修改主工程）

#### Mach-O Type
- 路径：Build Settings → 搜索 "Mach-O Type"
- 从下拉列表选择 **Dynamic Library**
- ⚠️ 此设置影响最终产物的类型，必须正确配置

#### Other Linker Flags
- 路径：Build Settings → 搜索 "Other Linker Flags"
- 点击 **+** 号添加新条目
- 输入 `-ObjC`（注意大小写）
- **作用**：确保 Objective-C 类别（Category）正确加载

#### Framework Search Paths
- 路径：Build Settings → 搜索 "Framework Search Paths"
- 双击该配置项展开编辑器
- 将离线 SDK 的 `SDK/Libs` 文件夹直接拖入编辑器
- 或者手动输入相对路径/绝对路径

### 2.3 插件工程注册配置

如果你需要按官方文档手工新建并编译 UTS 插件工程，插件工程内可能需要维护注册配置文件：

```json
{
  "hooksClasses": [],
  "providers": [],
  "components": []
}
```

> 注意：该文件属于“插件工程”编译阶段，不属于 UniPack Tool 当前的普通 UniApp 主工程打包阶段。构建中心不会把 `uts-config.json` 写入 `HBuilder-Hello` 主工程。

### 2.4 添加 UTS 核心源文件

uni-app x 插件工程会使用 `UTSOC.h/mm`。普通 UniApp 插件工程按官方表格使用 `DCloudUTSConfig.h`、`DCloudUTSConfig.m`、`UTSCPP.h`、`UTSCPP.mm`。

| 文件路径 | 说明 |
|---------|------|
| `SDK/ExtApiSrc/DCloudUTSConfig.h` | 普通 UniApp UTS 配置头文件 |
| `SDK/ExtApiSrc/DCloudUTSConfig.m` | 普通 UniApp UTS 配置实现文件 |
| `SDK/ExtApiSrc/UTSCPP.h` | 普通 UniApp UTS C++ 桥接头文件 |
| `SDK/ExtApiSrc/UTSCPP.mm` | 普通 UniApp UTS C++ 桥接实现文件 |

**添加方法**：
1. 在 Xcode Project Navigator 中右键点击工程名
2. 选择 **Add Files to "工程名"...**
3. 导航到 SDK 的 `ExtApiSrc` 目录
4. 选择普通 UniApp 所需的 `DCloudUTSConfig.h/m` 和 `UTSCPP.h/mm`
5. 确保 **Copy items if needed** 未勾选（引用原文件）
6. 点击 **Add**

---

## 3. 配置资源文件

完成基础环境配置后，需要将 SDK 核心依赖库和 UTS 插件的资源文件分别添加到对应工程中。

### 3.1 SDK 核心文件和依赖库

根据官方文档中的“传统 UniApp 项目”分支，普通 UniApp 使用以下 SDK 文件和依赖库：

| 类别 | 文件/库 | 添加位置 | Embed 设置 |
|------|---------|---------|-----------|
| **源码文件** | `SDK/ExtApiSrc/DCloudUTSConfig.h` | 插件工程 | - |
| **源码文件** | `SDK/ExtApiSrc/DCloudUTSConfig.m` | 插件工程 | - |
| **源码文件** | `SDK/ExtApiSrc/UTSCPP.h` | 插件工程 | - |
| **源码文件** | `SDK/ExtApiSrc/UTSCPP.mm` | 插件工程 | - |
| **配置文件** | `SDK/ExtApiSrc/config.json` | 见第 4 节说明 | - |
| **基础框架** | `SDK/Libs/DCUniBase.framework` | 插件工程 | Do Not Embed |
| **UTS 基础库** | `SDK/Libs/DCloudUTSFoundation.framework` | 插件工程 | Do Not Embed |

> `DCloudUniappRuntime.xcframework`、`DCloudUTSExtAPI.xcframework`、`UTSOC.h/mm` 是 uni-app x 分支内容，普通 UniApp 打包中心不接入。

### 3.2 uni_modules 资源目录处理

HBuilderX 导出的资源文件中，`uni_modules` 目录下的每个插件都包含以下子目录，需要按照规则分别处理：

| 目录/文件名 | 用途说明 | 添加到 | 详细操作 |
|------------|---------|--------|---------|
| **Frameworks/** | 插件依赖的三方 framework / xcframework | **插件工程** | Build Phases → Link Binary With Libraries → 拖入文件，Embed 设为 Do Not Embed |
| **Libs/** | 插件依赖的三方 .a 静态库 | **插件工程** | Build Phases → Link Binary With Libraries → Add Other → 选择 .a 文件 |
| **src/** | 插件的 UTS 源代码 | **插件工程** | Build Phases → Compile Sources → 拖入源码文件 |
| **Resources/** | 插件需要的资源文件（图片、bundle 等） | **主工程** | Build Phases → Copy Bundle Resources → 拖入资源 |
| **Info.plist** | 需要合并到主工程的 plist 配置 | **主工程 Info.plist** | 手动合并键值对，注意去重 |
| **UTS.entitlements** | 权限声明配置 | **主工程 Capabilities** | 根据 entitlements 内容添加对应 Capability |
| **config.json** | 插件的系统库/pod/plist 依赖声明 | **见下节详解** | 解析后分别配置到不同位置 |

#### 操作示例

**添加 Frameworks 到插件工程**：
1. 打开插件工程的 Build Phases 标签页
2. 展开 **Link Binary With Libraries**
3. 点击 **+** 按钮
4. 点击 **Add Other...** → **Add Files...**
5. 导航到插件的 `Frameworks` 目录
6. 选择所有 .framework / .xcframework 文件
7. 在弹窗中确保 **"Copy items if needed"** 未勾选
8. 在 **Embed** 列表中设置为 **Do Not Embed**

**添加 Resources 到主工程**：
1. 打开主工程的 Build Phases 标签页
2. 展开 **Copy Bundle Resources**
3. 点击 **+** → **Add Files...**
4. 导航到插件的 `Resources` 目录
5. 选择所有资源文件
6. 点击 **Add**

---

## 4. config.json 配置详解

每个 UTS 插件的 `config.json` 文件声明了该插件所需的各种依赖和配置。这是整个集成过程中最关键的部分。

### 4.1 config.json 结构示例

```json
{
  "frameworks": [
    "CoreLocation.framework",
    "SystemConfiguration.framework",
    "Security.framework"
  ],
  "deploymentTarget": "12.0",
  "dependencies-pods": {
    "Alamofire": "~> 5.0",
    "SnapKit": "~> 5.0"
  },
  "plists": {
    "NSCameraUsageDescription": "此应用需要访问相机以拍摄照片",
    "NSLocationWhenInUseUsageDescription": "此应用需要获取您的位置信息"
  },
  "hooksClass": "MyPluginHookClass",
  "provider": "MyComponentProvider",
  "components": [
    {
      "name": "my-uts-component",
      "class": "MyUTSComponentClass"
    }
  ]
}
```

### 4.2 各字段详细说明

#### frameworks — 系统依赖框架

**用途**：声明插件依赖的 Apple 系统框架

**配置位置**：**主工程** → Build Phases → Link Binary With Libraries

**操作方法**：
1. 打开主工程设置
2. 选择 Build Phases → Link Binary With Libraries
3. 点击 **+** 按钮
4. 在搜索框中输入框架名称（如 `CoreLocation`）
5. 选择并添加

**常见系统框架列表**：

| 框架名称 | 用途 |
|---------|------|
| `CoreLocation.framework` | 定位服务 |
| `CoreMotion.framework` | 加速度计、陀螺仪 |
| `AVFoundation.framework` | 音视频捕获/播放 |
| `ImageIO.framework` | 图片编解码 |
| `SystemConfiguration.framework` | 网络状态检测 |
| `Security.framework` | 安全/加密相关 |
| `CoreTelephony.framework` | 电信网络信息 |
| `CFNetwork.framework` | 网络通信 |
| `StoreKit.framework` | 应用内购买 |
| `UserNotifications.framework` | 用户通知 |

#### deploymentTarget — 最低支持版本

**用途**：插件要求的最低 iOS 版本

**配置位置**：**插件工程** → Target → General → Minimum Deployments

**注意事项**：
- 如果插件的 `deploymentTarget` 高于主工程的最低支持版本，需要同步修改主工程的 Minimum Deployments
- 建议所有插件保持一致的最低版本要求

#### dependencies-pods — CocoaPods 依赖

**用途**：声明插件依赖的第三方 Pod 库

**配置位置**：**插件工程** → Podfile

**操作步骤**：
1. 在插件工程目录下创建或编辑 `Podfile`（如不存在）
2. 添加以下内容：

```ruby
# Podfile 示例
platform :ios, '12.0'
use_frameworks!

target 'unimoduleUniGetbatteryinfo' do
  # 插件声明的 pod 依赖
  pod 'Alamofire', '~> 5.0'
  pod 'SnapKit', '~> 5.0'

  # UTS 基础依赖（根据实际情况添加）
  # pod 'DCloudUTSFoundation', :path => '../SDK/Libs/'
end
```

3. 在终端执行安装命令：

```bash
cd <插件工程目录>
pod install
```

4. 安装完成后，使用 `.xcworkspace` 文件打开工程（而非 .xcodeproj）

> **重要**：执行 `pod install` 后必须使用 `.xcworkspace` 打开工程，否则 CocoaPods 依赖不会生效。

#### plists — Info.plist 配置项

**用途**：需要在主工程 Info.plist 中添加的配置

**配置位置**：**主工程** → Info.plist

**操作方法**：
1. 打开主工程的 `Info.plist` 文件
2. 根据 `plists` 字段的内容添加对应的键值对
3. 注意检查是否已存在相同 key（避免重复）

**示例**：
```json
"plists": {
  "NSCameraUsageDescription": "此应用需要访问相机",
  "NSLocationWhenInUseUsageDescription": "需要定位权限"
}
```

转换为 Info.plist：
```xml
<key>NSCameraUsageDescription</key>
<string>此应用需要访问相机</string>

<key>NSLocationWhenInUseUsageDescription</key>
<string>需要定位权限</string>
```

#### hooksClass — 生命周期钩子类

**用途**：用于监听 UTS 插件的应用程序生命周期事件

**配置位置**：**插件工程** → `uts-config.json` → `hooksClasses` 节点

**配置方法**：
在之前创建的 `uts-config.json` 文件中，将 `hooksClass` 的值添加到 `hooksClasses` 数组中：

```json
{
  "hooksClasses": ["MyPluginHookClass"],
  "providers": [],
  "components": []
}
```

> 多个插件如果有 hooksClass，全部追加到同一个数组中。

#### provider — 组件注册提供者

**用途**：组件的注册信息（用于自定义组件）

**配置位置**：**插件工程** → `uts-config.json` → `providers` 节点

**配置方法**：
```json
{
  "hooksClasses": [],
  "providers": ["MyComponentProvider"],
  "components": []
}
```

#### components — 组件注册信息

**用途**：声明插件提供的自定义组件列表

**配置位置**：**插件工程** → `uts-config.json` → `components` 节点

**配置方法**：
```json
{
  "hooksClasses": [],
  "providers": [],
  "components": [
    {
      "name": "my-uts-component",
      "class": "MyUTSComponentClass"
    }
  ]
}
```

### 4.3 完整 uts-config.json 示例

假设有两个 UTS 插件需要集成（plugin-a 和 plugin-b），最终的 `uts-config.json` 应该如下：

```json
{
  "hooksClasses": [
    "PluginAHookClass",
    "PluginBHookClass"
  ],
  "providers": [
    "PluginAProvider"
  ],
  "components": [
    {
      "name": "uts-component-a",
      "class": "UTSComponentAClass"
    },
    {
      "name": "uts-component-b",
      "class": "UTSComponentBClass"
    }
  ]
}
```

---

## 5. 隐私清单处理

从 **SDK 4.13+** 版本开始，以及 **2024 年春季** 之后提交 App Store 的应用都必须包含 Privacy Manifest 文件。

### 5.1 PrivacyInfo.xcprivacy 文件来源

如果 HBuilderX 导出的资源文件中的 UTS 插件包含 `PrivacyInfo.xcprivacy` 文件，说明该插件使用了需要声明的 API 或数据收集功能。

### 5.2 合并方法

1. **定位文件**：在插件的 `app-ios` 目录下查找 `PrivacyInfo.xcprivacy`
2. **打开目标文件**：在插件工程中找到同名文件（通常在工程根目录或 PrivacyInfo.xcprivacy 中）
3. **合并内容**：将插件提供的 PrivacyInfo.xcprivacy 内容**合并**到插件工程的文件中
4. **去重处理**：
   - 检查 `NSPrivacyCollectedDataTypes` 数组是否有重复的数据类型声明
   - 检查 `NSPrivacyAccessedAPITypes` 数组是否有重复的 API 声明
   - 合并时保留更详细的描述信息

### 5.3 PrivacyManifest.xml 示例结构

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
  "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <!-- 是否跟踪用户 -->
    <key>NSPrivacyTracking</key>
    <false/>

    <!-- 跟踪域名 -->
    <key>NSPrivacyTrackingDomains</key>
    <array/>

    <!-- 收集的用户数据类型 -->
    <key>NSPrivacyCollectedDataTypes</key>
    <array>
        <!-- 根据插件实际使用情况添加 -->
    </array>

    <!-- 使用的必需理由 API -->
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
        <!-- 其他 API 声明... -->
    </array>
</dict>
</plist>
```

> **更多关于隐私清单的信息**请参考 [iOS 主文档](../index.md#6-隐私清单privacy-manifest)

---

## 6. 编译导出 xcframework

完成所有配置后，需要编译插件工程并打包为 xcframework 格式，以便集成到主工程中。

### 6.1 使用 Xcode GUI 编译

#### 步骤一：编译真机版本

1. 在 Xcode 顶部工具栏，点击设备选择下拉框
2. 选择 **Any iOS Device**（用于真机构建）
   ![](https://aka.doubaocdn.com/s/uItV1wZnFv)
3. 点击菜单栏 **Product → Build**（或快捷键 **Cmd + B**）
4. 等待编译完成（状态栏显示 "Build Succeeded"）

#### 步骤二：编译模拟器版本

1. 再次点击设备选择下拉框
2. 选择 **Any iOS Simulator Device**（用于模拟器构建）
3. 点击 **Product → Build**（Cmd + B）
4. 等待编译完成

#### 步骤三：查看编译产物

1. 点击菜单栏 **Product → Show Build Folder in Finder**
2. Finder 会自动打开编译产物目录
3. 导航路径示例：
   - 真机产物：`Build/Products/Release-iphoneos/unimoduleUniGetbatteryinfo.framework`
   - 模拟器产物：`Build/Products/Release-iphonesimulator/unimoduleUniGetbatteryinfo.framework`

### 6.2 使用 xcodebuild 命令行创建 xcframework

获得真机和模拟器的 .framework 文件后，使用 `xcodebuild` 命令将其合并为 xcframework：

```bash
xcodebuild -create-xcframework \
  -framework <真机 framework 路径>/unimoduleUniGetbatteryinfo.framework \
  -framework <模拟器 framework 路径>/unimoduleUniGetbatteryinfo.framework \
  -output <导出目录>/unimoduleUniGetbatteryinfo.xcframework
```

#### 参数说明

| 参数 | 说明 |
|------|------|
| `-create-xcframework` | 创建 xcframework 的命令标志 |
| `-framework` | 指定要合并的 .framework 文件路径（可多次使用） |
| `-output` | 输出的 xcframework 文件路径 |

#### 完整示例

```bash
# 假设编译产物在以下位置
REAL_DEVICE_PATH=~/Library/Developer/Xcode/DerivedData/YourProject-xxxxx/Build/Products/Release-iphoneos
SIMULATOR_PATH=~/Library/Developer/Xcode/DerivedData/YourProject-xxxxx/Build/Products/Release-iphonesimulator
OUTPUT_PATH=~/Desktop/Output

# 执行合并命令
xcodebuild -create-xcframework \
  -framework ${REAL_DEVICE_PATH}/unimoduleUniGetbatteryinfo.framework \
  -framework ${SIMULATOR_PATH}/unimoduleUniGetbatteryinfo.framework \
  -output ${OUTPUT_PATH}/unimoduleUniGetbatteryinfo.xcframework
```

执行成功后会输出类似信息：
```
Creating xcframework...
Building frameworks: unimoduleUniGetbatteryinfo.framework
Outputting to: ~/Desktop/Output/unimoduleUniGetbatteryinfo.xcframework
```

### 6.3 Apple Silicon Mac 特殊处理（模拟器编译）

如果您使用的是 **Apple Silicon 芯片**的 Mac（M1/M2/M3 等），编译模拟器版本时需要特殊处理：

#### 问题现象
- 直接选择 iOS Simulator 编译可能会报架构不兼容错误
- 或生成的 framework 只包含 arm64 架构

#### 解决方案

1. 在 Xcode 菜单栏点击 **Product → Destination → Show All Run Destinations**
2. 在展开的目标列表中，找到 **Simulator** 部分
3. 选择带有 **(Rosetta)** 标记的模拟器选项（如 "iPhone 15 (Rosetta)"）
4. 重新执行 Product → Build

> **原理**：Rosetta 模拟器会以 x86_64 架构运行，确保编译出的模拟器 framework 包含正确的架构切片。

### 6.4 集成 xcframework 到主工程

1. 将生成的 `.xcframework` 文件拖入**主工程**的 Project Navigator
2. 在弹出的对话框中：
   - ✅ 勾选 **Copy items if needed**（复制到工程目录）
   - ❌ 不勾选 **Create folder references**
   - **Embed** 设置选择 **"Embed & Sign"**
3. 点击 **Finish**

> **验证方法**：在主工程的 General → Frameworks, Libraries, and Embedded Content 中确认该 xcframework 的 Embed 设置为 **Embed & Sign**。

---

## 7. 原生项目中调试 UTS 插件

如果需要在开发过程中调试 UTS 插件的源代码（设置断点、查看变量等），需要创建 Xcode Workspace 来同时管理主工程和插件工程。

### 7.1 创建 Xcode Workspace

1. 启动 Xcode
2. 点击菜单栏 **File → New → Workspace...**
3. 选择保存位置（建议放在项目根目录）
4. 命名为 `MyApp.xcworkspace`（或您喜欢的名称）
5. 点击 **Save**

### 7.2 添加工程到 Workspace

1. 在 Workspace 的左侧 Project Navigator 中右键点击
2. 选择 **Add Files to "MyApp"...**
3. 导航到您的**主工程**目录，选择 `.xcodeproj` 或 `.xcworkspace` 文件
4. 点击 **Add**
5. 重复上述步骤，添加 **UTS 插件工程**的 `.xcodeproj` 文件

添加完成后，Workspace 应包含两个工程：
- 主工程（如 `HBuilder-Hello`）
- UTS 插件工程（如 `unimoduleUniGetbatteryinfo`）

![](https://aka.doubaocdn.com/s/srqn1wZnFv)

### 7.3 配置主工程依赖插件工程

1. 在 Workspace 左侧点击**主工程**
2. 选择 **TARGETS** 下的主工程 Target
3. 切换到 **General** 标签页
4. 滚动到 **Frameworks, Libraries, and Embedded Content** 部分
5. 点击 **+** 按钮
6. 在弹出窗口中选择 **On Other Projects** 标签页
7. 找到并选择 UTS 插件工程生成的 framework
8. 确保 **Embed & Sign** 选项被勾选
9. 点击 **Add**

### 7.4 设置断点并进行调试

1. 在 Workspace 左侧的 **UTS 插件工程**中，打开需要调试的源代码文件（如 `.m`、`.mm`、`.swift` 文件）
2. 在代码行号左侧单击设置断点（红色圆点）
3. 确保顶部工具栏的设备选择为**真机或模拟器**
4. 点击 **Run** 按钮（▶️）或按 **Cmd + R** 运行主工程
5. 在应用中触发调用 UTS 插件的功能
6. 程序会在断点处暂停，可以查看变量值、调用栈等信息

> **调试技巧**：
> - 可以在插件工程的任何地方设置断点
> - 使用 lldb 控制台执行表达式（po、p 命令）
> - 查看 View Debugging 层级结构（适用于 UI 组件插件）

---

## 8. 与 UniPack Tool 的集成

### 8.1 当前支持情况

UniPack Tool（本项目）当前对 UTS 插件的支持情况：

| 功能 | 支持状态 | 说明 |
|------|---------|------|
| 扫描 uni_modules 目录 | ✅ 已支持 | 仅当插件包含 `utssdk/app-ios` 或 `utssdk/app-iOS` 时触发 iOS UTS 集成 |
| 解析 config.json | ✅ 已支持 | 支持系统 framework、plist、pod 依赖提示、hooks/provider/components 读取 |
| 自动接入普通 UniApp UTS 运行库 | ✅ 已支持 | 接入 `DCUniBase.framework` 与 `DCloudUTSFoundation.framework` |
| 自动添加插件依赖库 | ✅ 已支持 | 复制并接入插件 `Frameworks/`、`.framework`、`.xcframework`，并补充 `FRAMEWORK_SEARCH_PATHS` |
| 自动添加主工程资源 | ✅ 已支持 | 复制并注册 `Resources/`、`.bundle`、`PrivacyInfo.xcprivacy` |
| 自动合并 Info.plist | ✅ 已支持 | 合并插件 `config.json` 中的 `plists` 字段 |
| CocoaPods 依赖安装 | ⚠️ 提示 | 仅提示 `dependencies-pods`，当前构建中心不执行 `pod install` |
| 插件工程创建与 xcframework 编译 | ❌ 不在当前流程 | 当前是普通 UniApp 打包流程，消费插件已提供的 iOS 原生产物 |
| 主工程 uts-config.json 生成 | ❌ 不执行 | `uts-config.json` 属于插件工程注册配置，构建中心不写入主工程 |

### 8.2 当前推荐工作流

1. **确认导出资源中存在 iOS UTS 插件产物**：`uni_modules/<插件>/utssdk/app-ios` 或 `app-iOS`
2. **插件已包含可直接接入的 iOS 原生产物**：例如 `Frameworks/*.framework`、`*.xcframework`、`Resources/`、`config.json`
3. **使用 UniPack Tool 自动生成并打包 iOS 离线工程**
4. **如插件只有 UTS 源码没有 iOS 原生产物**，需要先按官方流程手工编译插件 framework/xcframework，再放入插件目录

### 8.3 未来自动化方向

计划逐步实现以下自动化功能：

- [x] 自动解析 uni_modules 中的 UTS 插件
- [x] 自动添加普通 UniApp UTS 运行库
- [x] 自动添加插件依赖库和资源文件
- [x] 自动将插件 framework/xcframework 集成到主工程
- [ ] 自动创建插件 Framework 工程
- [ ] 自动执行 xcodebuild 编译插件 xcframework
- [ ] 自动安装插件 CocoaPods 依赖

> **欢迎贡献**：如果您有相关开发经验，欢迎提交 PR 帮助完善 UTS 插件的自动化构建流程！

---

## 9. 常见问题（FAQ）

### Q1: 编译报错 "UTSCPP.h file not found" 或 "DCloudUTSConfig.h file not found"

**原因**：普通 UniApp 插件工程未正确添加 UTS 核心源文件，或 Framework Search Paths 未配置

**解决方案**：
1. 确认 `SDK/ExtApiSrc/DCloudUTSConfig.h`、`DCloudUTSConfig.m`、`UTSCPP.h`、`UTSCPP.mm` 已添加到插件工程
2. 检查 Build Settings → Framework Search Paths 是否包含 SDK/Libs 目录
3. 清理构建缓存（Product → Clean Build Folder，Cmd + Shift + K）
4. 重新编译

---

### Q2: 链接报错 "Undefined symbols for architecture xxx"

**原因**：缺少必要的依赖库或 framework

**解决方案**：
1. 检查插件的 `config.json` → `frameworks` 字段
2. 确认所有声明的系统库都已添加到 **Link Binary With Libraries**
3. 检查 `Libs/` 和 `Frameworks/` 目录下的第三方库是否已添加
4. 对于 CocoaPods 依赖，确认已执行 `pod install` 并使用 `.xcworkspace` 打开

---

### Q3: 运行时崩溃 "dyld: Library not loaded"

**原因**：动态库未正确 Embed，或签名问题

**解决方案**：
1. 确认动态 framework/xcframework 的 Embed 设置为 **Embed & Sign**（不是 Do Not Embed）
2. 检查 @rpath / @executable_path 搜索路径是否正确
3. 确保证书和描述文件有效且匹配
4. 尝试 Clean Build Folder 后重新编译运行

---

### Q4: 模拟器运行时报架构不兼容错误

**原因**：Apple Silicon Mac 编译模拟器版本时的架构问题

**解决方案**：
1. 参考 [6.3 节](#63-apple-silicon-mac-特殊处理模拟器编译) 使用 Rosetta 模拟器编译
2. 或使用命令行指定架构：
   ```bash
   xcodebuild -scheme YourScheme -destination 'platform=iOS Simulator,name=iPhone 15' -arch x86_64 build
   ```

---

### Q5: 多个插件的 config.json 有冲突怎么办

**场景**：两个插件都声明了相同的 plist key 或不同的 deploymentTarget

**解决方案**：
- **plists 冲突**：合并时保留描述文字，去重 key
- **deploymentTarget 冲突**：取最高版本作为统一的最低支持版本
- **hooksClass 冲突**：全部追加到 uts-config.json 的 hooksClasses 数组
- **pod 依赖版本冲突**：尝试找到兼容版本，或联系插件作者更新

---

### Q6: Pod 安装失败或版本冲突

**原因**：CocoaPods 依赖版本不兼容或源不可用

**解决方案**：
1. 更新 CocoaPods：`sudo gem install cocoapods`
2. 清理 Pod 缓存：`pod cache clean --all`
3. 删除 Podfile.lock 后重新安装：`rm Podfile.lock && pod install`
4. 检查网络连接（可能需要代理访问 GitHub）
5. 查看具体错误日志调整 Podfile 中的版本约束

---

### Q7: xcframework 导入后无法识别为 Module

**原因**：xcframework 内部缺少 modulemap 或 umbrella header

**解决方案**：
1. 检查 xcframework 内部是否包含 `.modulemap` 文件
2. 确认 Build Settings → Defines Module 设置为 YES
3. 如果是第三方库的问题，联系库作者确认是否支持 Module 方式导入
4. 作为临时方案，可尝试使用 Header Search Paths + Import Header 的方式

---

### Q8: 如何验证 UTS 插件是否集成成功

**验证方法**：

1. **编译验证**：主工程能成功编译无报错
2. **运行验证**：应用启动无崩溃
3. **功能验证**：在前端代码中调用插件接口：
   ```javascript
   // 示例：调用 UTS 插件
   const plugin = uni.requireNativePlugin('your-plugin-id')
   plugin.someMethod({
     success: (res) => console.log('插件调用成功', res),
     fail: (err) => console.error('插件调用失败', err)
   })
   ```
4. **日志验证**：在 Xcode Console 中查看插件相关的日志输出

---

## 相关文档

### 项目内部文档

- [iOS 离线打包主文档](../index.md) — iOS 平台完整的离线打包流程
- [原生插件集成指南](./native-plugins.md) — 传统 Objective-C/Swift 原生插件配置
- [UTS 内置模块](./uts-builtin-modules.md) — UTS 运行时和内置模块说明
- [第三方依赖说明](./third-party-dependencies.md) — 常见第三方 SDK 集成参考
- [支付模块](./payment.md) — 支付功能集成（微信/支付宝/IAP）
- [登录鉴权](./oauth.md) — 第三方登录配置

### 外部参考文档

- [DCloud 官方 iOS UTS 插件文档](https://doc.dcloud.net.cn/uni-app-x/native/use/iosuts.html)
- [DCloud UTS 内置模块集成](https://doc.dcloud.net.cn/uni-app-x/native/modules/ios/modules.html)
- [Apple Developer - Xcode Documentation](https://developer.apple.com/documentation/xcode)
- [Apple Developer - Creating Xcframeworks](https://developer.apple.com/documentation/swift_packages/bundling_resources_with_a_swift_package)
- [CocoaPods 官方文档](https://guides.cocoapods.org/)

---

## 更新记录

| 日期 | 版本 | 说明 |
|------|------|------|
| 2026-06-18 | v1.0.0 | 初始版本，基于 DCloud 官方文档整理 |
