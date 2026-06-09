# 原生插件（Native Plugins）（Android）

> **适用版本**：HBuilderX 5.0+
> **平台**：Android
> **官方文档**：https://ask.dcloud.net.cn/article/35763

## 概述

uni-app 原生插件（Native Plugin）是 DCloud 插件市场提供的、包含原生代码（Java/Kotlin/Objective-C/Swift）的功能扩展包。与 UTS 插件不同，原生插件以编译后的 AAR/JAR/.framework 等二进制形式分发，需要手动集成到离线打包工程中。

### UniPack Tool 自动化能力

在离线打包流程中，UniPack Tool 会自动完成以下操作：

| 步骤 | 自动化状态 | 说明 |
|------|-----------|------|
| 扫描 uni_modules 中的原生插件 | ✅ 自动 | 从导入的 UniApp 资源中识别自定义插件 |
| 复制 AAR/JAR 到工程 libs | ✅ 自动 | 将插件的 android 目录下的库文件复制到工作区 |
| 非 AAR 检测与解包 | ✅ 自动 | 检测非标准 AAR 并解包为散落文件（绕过 Jetifier 问题） |
| 生成 `dcloud_uniplugins.json` | ✅ 自动 | 根据插件的 package.json 自动生成注册文件 |
| 生成每个插件的 `build.gradle` | ✅ 自动 | 为每个插件创建独立的 Gradle 模块配置 |
| 注入 Gradle 依赖 | ✅ 自动 | 将插件声明的 dependencies 加入主工程 |
| 合并 AndroidManifest.xml | ✅ 自动 | 处理插件声明的权限、组件等 |
| **系统库依赖 (systemLibs)** | ⚠️ 需确认 | 部分系统库可能需要手动确认 |
| **parameters 参数值** | ⚠️ 需填写 | 插件所需的业务参数需用户在 UI 中填写 |

---

## 1. 插件目录结构

从 [DCloud 插件市场](https://ext.dcloud.net.cn/?cat1=5&cat2=51) 下载的原生插件具有固定的目录结构：

```
|-- 插件根目录/
    |-- android/              # Android 端原生代码和资源
    |   |-- libs/             # AAR / JAR 库文件
    |   |-- src/              # Java/Kotlin 源码
    |   |-- res/              # 资源文件
    |   |-- assets/           # 资产文件
    |   |-- jniLibs/          # .so 原生库
    |   └-- AndroidManifest.xml
    |-- ios/                  # iOS 端原生代码和资源
    |-- package.json          # ★ 插件配置文件（关键）
    └-- readme.md             # 插件说明文档
```

> **核心文件是 `package.json`**，它定义了插件的所有元数据、依赖项和配置参数。离线集成的本质就是将 package.json 中描述的内容手动映射到工程中。

---

## 2. package.json 关键字段说明

完整的 `_dp_nativeplugin.android` 配置结构如下：

```json
{
  "name": "RichAlert",
  "id": "DCloud-RichAlert",
  "version": "0.1.3",
  "_dp_type": "nativeplugin",
  "_dp_nativeplugin": {
    "android": {
      "plugins": [
        {
          "type": "module",
          "name": "DCloud-RichAlert",
          "class": "uni.dcloud.io.uniplugin_richalert.RichAlertWXModule"
        }
      ],
      "hooksClass": "io.dcloud.uniplugin.UniPluginHook",
      "integrateType": "library",
      "dependencies": [
        "com.alibaba:fastjson:1.1.46.android"
      ],
      "compileOptions": {
        "sourceCompatibility": "1.8",
        "targetCompatibility": "1.8"
      },
      "systemLibs": [
        "android.jar:${ANDROID_SDK}/platforms/android-28/android.jar"
      ],
      "parameters": {
        "appid": {
          "des": "应用 ID",
          "key": "DCloud_AppID"
        }
      }
    }
  }
}
```

### 字段详解

#### plugins — 插件组件注册

| 字段 | 类型 | 说明 |
|------|------|------|
| `type` | string | 插件类型：`module`（模块）或 `component`（组件） |
| `name` | string | 插件名称，必须与前端调用时 `requireNativePlugin()` 的参数一致 |
| `class` | string | 插件的完整类名（含包路径） |

此信息会被写入 `dcloud_uniplugins.json` 和 `data/dcloud_properties.xml`。

#### hooksClass — 生命周期钩子

指定插件的事件监听注册类。一个工程中所有插件共享同一个 hooksClass（通常为空或不配置）。

#### dependencies — Maven 依赖

插件依赖的第三方库，会自动注入到 build.gradle 的 `dependencies` 块中。

#### compileOptions — 编译选项

Java 源码兼容版本配置。

#### systemLibs — 系统库引用

需要引用的 Android 系统 JAR（如 `android.jar`），通常不需要手动处理。

#### parameters — 可配置参数

插件暴露给用户填写的业务参数。**这些参数需要在 UniPack Tool 的「模块配置」UI 中填写**，最终通过 `${占位符}` 方式写入 AndroidManifest.xml 或其他配置文件。

---

## 3. dcloud_uniplugins.json

UniPack Tool 在构建过程中会根据扫描到的原生插件自动生成此文件，位于工程的 `app/src/main/assets/dcloud_uniplugins.json`：

```json
{
  "nativePlugins": [
    {
      "plugins": [
        {
          "type": "module",
          "name": "DCloud-RichAlert",
          "class": "uni.dcloud.io.uniplugin_richalert.RichAlertWXModule"
        }
      ]
    },
    {
      "hooksClass": "io.dcloud.uniplugin.UniPluginHook",
      "plugins": [
        {
          "type": "component",
          "name": "MyCustomComponent",
          "class": "com.example.MyComponent"
        }
      ]
    }
  ]
}
```

> 工程中只包含一个 `nativePlugins` 数组，多个插件以数组元素形式并列。
> 此文件由工具自动生成，**无需手动编辑**。

---

## 4. 在 UniPack Tool 中使用原生插件

### 步骤一：下载并放置插件

1. 从 [DCloud 插件市场](https://ext.dcloud.net.cn/) 购买或下载所需的原生插件
2. 将插件解压到你的 UniApp 项目的 `uni_modules/` 目录下：
   ```
   你的UniApp项目/
   ├── uni_modules/
   │   ├── DCloud-RichAlert/          ← 原生插件目录
   │   │   ├── android/
   │   │   ├── ios/
   │   │   └── package.json
   │   └── 其他UTS插件...
   ├── manifest.json
   └── pages/
   ```

### 步骤二：在 HBuilderX 中配置 manifest.json

打开 `manifest.json` → **App原生插件配置** → 选择已安装的原生插件：

```json
// manifest.json → app-plus → nativePlugins
"nativePlugins": {
  "DCloud-RichAlert": {
    "__plugin_info": {
      "name": "RichAlert",
      "description": "丰富的弹窗提示插件",
      "platforms": "Android,iOS",
      "url": "",
      "is_custom": true,
      "is_uni_modules": true
    }
  }
}
```

### 步骤三：导出资源并在 UniPack Tool 中打包

1. HBuilderX 中选择 **发行 → 本地打包 App 资源** → 导出资源包
2. 打开 UniPack Tool → 创建/选择项目 → 进入 **构建中心**
3. 导入导出的资源目录
4. 工具会自动扫描到 `uni_modules` 中的原生插件
5. 如果插件有 `parameters` 参数，在 **Android 模块配置** 区域填写对应值
6. 选择 Android 平台 → 点击 **开始打包**

### 步骤四：验证

在前端代码中使用插件：

```javascript
// 获取原生插件实例
const richAlert = uni.requireNativePlugin('DCloud-RichAlert')

// 调用插件方法
richAlert.show(
  {
    title: '提示',
    message: '这是原生插件弹窗'
  },
  result => {
    console.log(result)
  }
)
```

---

## 5. 构建依赖说明

原生插件可能声明额外的 Gradle 依赖，UniPack Tool 会自动将其注入到构建工程中。以下是常见的基座依赖（由 DCloud SDK 要求）：

```groovy
dependencies {
    // 基座基础依赖（SDK 内置）
    implementation fileTree(dir: 'libs', include: ['*.jar'])
    implementation fileTree(dir: 'libs', include: ['*.aar'])

    // Android Support 库
    implementation 'androidx.recyclerview:recyclerview:1.3.2'
    implementation 'androidx.core:core:1.12.0'
    implementation 'androidx.appcompat:appcompat:1.6.1'

    // DCloud 组件依赖
    implementation 'com.alibaba.android:bindingx-core:1.0.3'
    implementation 'com.alibaba.android:bindingx_weex_plugin:1.0.3'
    implementation 'com.squareup.okhttp:okhttp:2.3.0'
    implementation 'com.squareup.okhttp:okhttp-ws:2.3.0'

    // 图片加载
    implementation 'com.github.bumptech.glide:glide:4.9.0'

    // JSON 解析
    implementation 'com.alibaba:fastjson:1.1.46.android'

    // 插件自有依赖（从 package.json → dependencies 自动提取）
}
```

> 以上依赖由 UniPack Tool 的 SDK 模板和插件扫描结果共同决定，通常无需手动修改。

---

## 6. 常见问题

### Q1: 加密付费插件无法使用？

> **普通授权版**的加密付费 UTS/原生插件不支持离线 SDK 打包。需要购买**源码授权版**才能在离线环境中使用。

### Q2: 多个插件有冲突怎么办？

- 检查是否有重复的 AAR 文件（不同插件引入了相同库的不同版本）
- 查看 Gradle 构建日志中的依赖冲突警告
- 尝试排除传递依赖或在 `resolutionStrategy` 中强制统一版本

### Q3: 提示找不到插件模块？

1. 确认 `manifest.json` 的 `app-plus → nativePlugins` 中已正确配置
2. 确认 `uni_modules/` 下插件目录完整（包含 `android/` 和 `package.json`）
3. 确认前端调用 `requireNativePlugin()` 的名称与 `package.json` 中的 `name` 一致
4. 检查 `dcloud_uniplugins.json` 是否包含了该插件的注册信息（查看构建产物）

### Q4: 原生插件和 UTS 插件有什么区别？

| 对比项 | 原生插件 | UTS 插件 |
|--------|---------|----------|
| 语言 | Java/Kotlin / ObjC/Swift | UTS (TypeScript 方言) |
| 分发形式 | 编译后 AAR/JAR/.framework | .utss 源码 + config.json |
| 来源 | DCloud 插件市场 | uni_modules 社区/自研 |
| 离线打包 | 需要 package.json 手动集成 | 由工具自动编译和集成 |
| 加密支持 | 源码授权版可用 | 普通授权可用（需源码） |

---

## 相关文档

- [UTS 插件配置详细教程（Android）](../../uts-plugin-config-uniapp-android.md) — 面向开发者的 UTS 插件手动配置教程
- [UTS 基础模块](uts-base-module.md) — UTS 插件的前置依赖模块
- [UTS 内置模块](uts-builtin-modules.md) — 内置 UTS 模块的 AAR/依赖列表
- [第三方依赖说明](third-party-dependencies.md) — 常见第三方库的集成参考
