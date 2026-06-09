# 原生插件（Native Plugins）（iOS）

> **适用版本**：HBuilderX 5.0+
> **平台**：iOS (iPhone/iPad)
> **官方文档**：https://ask.dcloud.net.cn/article/35764

## 概述

iOS 端原生插件包含编译后的 Objective-C / Swift 代码，以 `.framework`、`.a`、`.dylib` 等二进制形式分发。离线集成时需要根据插件的 `package.json` 配置文件，将各节点描述的内容手动映射到 Xcode 工程的对应位置。

### 当前支持状态

| 功能 | 支持情况 |
|------|---------|
| 扫描 uni_modules 中的原生插件 | ✅ 支持 |
| 识别 package.json 配置 | ✅ 支持 |
| Info.plist 参数写入 | ⚠️ 部分支持（parameters 节点） |
| Framework 库链接 | ⚠️ 需在 Xcode 工程中手动配置 |
| 资源文件拷贝 | ⚠️ 需确认资源路径 |

---

## 1. 插件目录结构

从 [DCloud 插件市场](https://ext.dcloud.net.cn/?cat1=5&cat2=51) 下载的原生插件：

```
|-- 插件根目录/
    |-- android/              # Android 端（本文不涉及）
    |-- ios/                  # ★ iOS 端原生代码和资源
    |   |-- *.framework/      # 框架库（静态或动态）
    |   |-- *.a               # 静态库
    |   |-- *.dylib           # 动态库
    |   |-- Headers/          # 公开头文件
    |   |-- Resources/        # 资源文件（bundle、图片等）
    |   └-- 其他文件...
    |-- package.json          # ★ 插件配置文件（关键）
    └-- readme.md
```

---

## 2. package.json 关键字段说明

完整的 `_dp_nativeplugin.ios` 配置结构：

```json
{
  "name": "RichAlert",
  "id": "DCloud-RichAlert",
  "version": "0.1.3",
  "_dp_type": "nativeplugin",
  "_dp_nativeplugin": {
    "ios": {
      "plugins": [
        {
          "type": "module",
          "name": "DCloud-RichAlert",
          "class": "DCRichAlertModule"
        }
      ],
      "hooksClass": "DCRichAlertHook",
      "integrateType": "library",
      "deploymentTarget": "9.0",
      "frameworks": [
        "CoreLocation.framework",
        "SystemConfiguration.framework"
      ],
      "embedFrameworks": [
        "MyDynamicFramework.framework"
      ],
      "resources": [
        "Resources/RichAlertBundle.bundle"
      ],
      "privacies": [
        "NSLocationWhenInUseUsageDescription"
      ],
      "parameters": {
        "getui:appid": {
          "des": "个推 AppID",
          "key": "appid"
        }
      },
      "validArchitectures": ["arm64", "armv7"]
    }
  }
}
```

### 字段详解

#### plugins — 插件注册信息

| 字段 | 说明 |
|------|------|
| `type` | 插件类型：`module`（模块）或 `component`（组件）|
| `name` | 插件名称，与前端 `requireNativePlugin()` 参数一致 |
| `class` | 插件的 Objective-C/Swift 完整类名 |

#### hooksClass — 生命周期钩子

插件的事件监听注册类名。一个工程中所有插件共享同一个 `dcloud_uniplugins` 节点下的 `hooksClass`。

#### frameworks — 系统依赖框架

插件依赖的**系统静态库**。需要添加到 Xcode 的 **Link Binary With Libraries** 中。

支持的格式：
- `.framework` — 如 `CoreLocation.framework`
- `.tbd` — 如 `libz.tbd`
- `.dylib` — 如 `libsqlite3.dylib`

#### embedFrameworks — 动态框架

插件依赖的**动态 framework 库**。必须以 **Embed & Sign** 方式引入，不能弄错为静态链接。

> 动态库和静态库的区别：动态库以 `.framework` 形式存在但运行时由系统加载；静态库在编译时直接合并到可执行文件中。

#### resources — 资源文件

插件所需的资源文件列表（相对于 ios 目录的路径）。需添加到 Xcode 的 **Copy Bundle Resources** 中。

#### privacies — 隐私权限

插件使用到的系统隐私权限。需要在 `Info.plist` 中添加对应的 Usage Description，否则 App 运行时会崩溃或功能不可用。

#### parameters — 可配置参数

插件的业务配置参数。键名为 Info.plist 中的嵌套路径（使用 `:` 分隔层级）。

#### validArchitectures — 有效架构

插件支持的 CPU 架构，通常为 `arm64`。

---

## 3. 各节点的离线工程配置映射

以下是 package.json 各节点到 Xcode 工程 / Info.plist 的完整映射表：

| package.json 节点 | Xcode 操作位置 | 具体操作 |
|---|---|---|
| `plugins` + `hooksClass` | **Info.plist** | 添加 `dcloud_uniplugs` 节点（见下方示例） |
| `frameworks` | **Build Phases → Link Binary With Libraries** | 点击 `+` → 选择系统库 |
| `embedFrameworks` | **Build Phases → Embed & Sign (Embed Frameworks)** | 点击 `+` → 从插件 ios 目录选择 .framework |
| `.a` 静态库（ios 目录下） | **Build Phases → Link Binary With Libraries** | 点击 `+` → Add Other → 选择 .a 文件 |
| `.framework` 静态库（ios 目录下） | **Build Phases → Link Binary With Libraries** | 点击 `+` → Add Other → 选择 .framework |
| `resources` | **Build Phases → Copy Bundle Resources** | 点击 `+` → 选择资源文件 |
| `headers` | 直接拖入工程 | 将头文件拖入 Project Navigator |
| `privacies` | **Info.plist** | 添加对应的 Usage Description 键值对 |
| `parameters` | **Info.plist** | 添加嵌套 dict 结构的配置项 |

---

## 4. Info.plist 完整配置

### 4.1 dcloud_uniplugs 节点（plugins 注册）

根据插件的 `plugins` 和 `hooksClass` 配置，在 `Info.plist` 中添加：

```xml
<key>dcloud_uniplugs</key>
<array>
    <!-- 第一个插件 -->
    <dict>
        <key>hooksClass</key>
        <string>DCRichAlertHook</string>
        <key>plugins</key>
        <array>
            <dict>
                <key>type</key>
                <string>module</string>
                <key>name</key>
                <string>DCloud-RichAlert</string>
                <key>class</key>
                <string>DCRichAlertModule</string>
            </dict>
        </array>
    </dict>
    <!-- 第二个插件（如有）追加新的 dict -->
    <dict>
        <key>hooksClass</key>
        <string></string>
        <key>plugins</key>
        <array>
            <dict>
                <key>type</key>
                <string>component</string>
                <key>name</key>
                <string>MyComponent</string>
                <key>class</key>
                <string>MyComponentClass</string>
            </dict>
        </array>
    </dict>
</array>
```

> **重要**：工程中只能包含一个 `dcloud_uniplugs` 节点。多个插件在该节点下配置多个 `<dict>` 元素即可。

### 4.2 parameters 嵌套参数

如果插件声明了 parameters：

```json
"parameters": {
  "getui:appid": {
    "des": "个推 AppID",
    "key": "appid"
  },
  "jpush:appKey": {
    "des": "极光 AppKey",
    "key": "appKey"
  }
}
```

在 Info.plist 中对应添加嵌套结构：

```xml
<!-- getui:appid → 嵌套 dict -->
<key>getui</key>
<dict>
    <key>appid</key>
    <string>你的个推AppID</string>
</dict>

<!-- jpush:appKey → 嵌套 dict -->
<key>jpush</key>
<dict>
    <key>appKey</key>
    <string>你的极光AppKey</string>
</dict>
```

> 键名中 `:` 左侧为顶层 key，右侧为内层 key。

### 4.3 privacies 权限描述

如果插件声明了隐私权限：

```json
"privacies": [
  "NSLocationWhenInUseUsageDescription",
  "NSCameraUsageDescription",
  "NSPhotoLibraryUsageDescription"
]
```

在 Info.plist 中添加：

```xml
<key>NSLocationWhenInUseUsageDescription</key>
<string>此应用需要获取您的位置信息以提供服务</string>

<key>NSCameraUsageDescription</key>
<string>此App需要访问相机以拍摄照片</string>

<key>NSPhotoLibraryUsageDescription</key>
<string>此App需要访问相册以选择图片</string>
```

> 描述文字应清晰说明为何需要该权限，否则 App Store 审核可能被拒。

---

## 5. 静态库与动态库区分与处理

### 判断方法

| 特征 | 静态库 | 动态库 |
|------|--------|--------|
| 文件扩展名 | `.a`、`.framework`（内部含 .a） | `.framework`（内部含二进制）、`.dylib` |
| package.json 位置 | 不在 `embedFrameworks` 中 | 在 `embedFrameworks` 中 |
| Xcode 操作 | **Link Binary With Libraries** | **Embed & Sign (Embed Frameworks)** |
| 运行时行为 | 编译时合并到主程序 | 运行时由 dyld 加载 |
| 是否需要 Embed | 否 | 是（必须 Embed & Sign） |

### 处理步骤

1. **`.a` 静态库**：
   - Build Phases → Link Binary With Libraries → `+` → Add Other → 选择 `.a` 文件

2. **`.framework` 静态库**（不在 embedFrameworks 中）：
   - Build Phases → Link Binary With Libraries → `+` → Add Other → 选择 `.framework`

3. **`.framework` 动态库**（在 embedFrameworks 中）：
   - Build Phases → **Embed & Sign (Embed Frameworks)** → `+` → Add Other → 从插件 ios 目录选择
   - 在弹窗中选择 **"Copy items if needed"**
   - 确保 **"Create folder references"** 未选中
   - Embed 设置为 **"Embed & Sign"**

---

## 6. 集成示例（以 RichAlert 为例）

### 步骤一：下载插件

从 [DCloud 插件市场](https://ext.dcloud.net.cn/plugin?id=36) 下载 RichAlert 示例插件。

### 步骤二：查看 package.json

```json
{
  "name": "RichAlert",
  "id": "DCloud-RichAlert",
  "version": "0.1.3",
  "_dp_type": "nativeplugin",
  "_dp_nativeplugin": {
    "ios": {
      "plugins": [
        {
          "type": "module",
          "name": "DCloud-RichAlert",
          "class": "DCRichAlertModule"
        }
      ],
      "integrateType": "library",
      "deploymentTarget": "8.0"
    }
  }
}
```

此插件配置较简单，只有 plugins 节点，无额外 frameworks 或 dependencies。

### 步骤三：配置离线工程

1. 将 ios 目录下的 `.a` 库文件添加到 **Link Binary With Libraries**
2. 在 **Info.plist** 中添加 dcloud_uniplugs 节点：

```xml
<key>dcloud_uniplugs</key>
<array>
    <dict>
        <key>hooksClass</key>
        <string></string>
        <key>plugins</key>
        <array>
            <dict>
                <key>type</key>
                <string>module</string>
                <key>name</key>
                <string>DCloud-RichAlert</string>
                <key>class</key>
                <string>DCRichAlertModule</string>
            </dict>
        </array>
    </dict>
</array>
```

3. 如果 ios 目录下有头文件（Headers），将 `.h` 文件拖入工程

### 步骤四：前端调用验证

```javascript
const richAlert = uni.requireNativePlugin('DCloud-RichAlert')

richAlert.show(
  { title: '提示', message: '原生弹窗' },
  result => console.log(result)
)
```

---

## 7. 在 UniPack Tool 中使用 iOS 原生插件

### 当前流程

1. 从插件市场下载原生插件，放入 UniApp 项目的 `uni_modules/` 目录
2. 在 HBuilderX `manifest.json` → **App原生插件配置** 中勾选插件
3. 导出本地打包资源（HBuilderX → 发行 → 本地打包 App 资源）
4. 在 UniPack Tool 中导入资源并选择 iOS 平台打包
5. 打包完成后，在生成的 Xcode 工程中完成以下手动操作（如插件有声明）：
   - 添加 frameworks 到 Link Binary With Libraries
   - 添加动态 frameworks 到 Embed & Sign
   - 配置 Info.plist（dcloud_uniplugs、parameters、privacies）
   - 添加资源文件到 Copy Bundle Resources

> **注意**：iOS 离线打包目前需要在生成的 Xcode 工程中完成部分手动配置。后续版本将逐步自动化这些步骤。

---

## 8. 常见问题

### Q1: 控制台提示 `[warn] No component config for XXX`？

- 确认 Info.plist 中 `dcloud_uniplugs` 节点的 `plugins` 数组包含了该组件
- 检查 `name` 和 `class` 是否与 package.json 完全一致
- 确认 `.a` / `.framework` 已正确链接（不是仅拷贝文件）

### Q2: 通过 CocoaPods 引入的插件提示找不到？

CocoaPods 集成的插件需要确保：
1. Podfile 中已正确声明 pod 依赖
2. 执行了 `pod install` 并使用 `.xcworkspace` 打开工程
3. CocoaPods 的 frameworks 搜索路径设置正确

### Q3: 多插件的 hooksClass 冲突？

一个工程只有一个 `dcloud_uniplugs` 节点，只有一个 `hooksClass` 键。如果不同插件需要不同的 hooksClass：
- 通常只保留非空的那个 hooksClass
- 大部分情况下留空字符串即可

### Q4: 动态库报 `dyld: Library not loaded`？

- 确认动态 framework 已添加到 **Embed & Sign**（而非 Link Binary With Libraries）
- Embed 设置必须是 **"Embed & Sign"**（不能是 "Do Not Embed"）
- 检查 `@rpath` / `@executable_path` 的搜索路径是否正确

### Q5: ios 目录下没有 .a 文件怎么办？

有些纯 Swift 插件可能只提供 `.framework`，此时按 framework 类型处理即可。如果没有需要链接的二进制文件，只需配置好 Info.plist 的 `dcloud_uniplugs` 节点即可。

---

## 相关文档

- [UTS 内置模块（iOS）](uts-builtin-modules.md) — UTS 运行时和内置模块的框架/CocoaPods 配置
- [第三方依赖说明](third-party-dependencies.md) — iOS 端常见第三方库的集成参考
- [Android 原生插件指南](../android/modules/native-plugins.md) — Android 端原生插件配置
