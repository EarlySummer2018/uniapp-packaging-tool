# Android uni-app UTS 插件配置详细教程

> **适用平台**：Android uni-app（非 uni-app x）
> **生成时间**：2026-06-05
> **基于官方文档**：https://doc.dcloud.net.cn/uni-app-x/native/use/androiduts.html

---

## 目录

- [1. 前置说明与概念区分](#1-前置说明与概念区分)
- [2. 新建 Android UTS 插件模块](#2-新建-android-uts-插件模块)
- [3. 修改 build.gradle 配置](#3-修改-buildgradle-配置)
  - [3.1 添加依赖（uni-app 专用版本）](#31-添加依赖uni-app-专用版本)
  - [3.2 基础库依赖引用](#32-基础库依赖引用)
  - [3.3 插件间依赖](#33-插件间依赖)
- [4. 根据 config.json 配置应用](#4-根据-configjson-配置应用)
  - [4.1 abis（CPU 架构）](#41-abiscpu-架构)
  - [4.2 minSdkVersion](#42-minsdkversion)
  - [4.3 dependencies（依赖库）](#43-dependencies依赖库)
  - [4.4 project（Gradle 插件）](#44-projectgradle-插件)
  - [4.5 components（组件注册）](#45-components组件注册)
  - [4.6 hooksClass（生命周期监听）](#46-hookslifecycle生命周期监听)
- [5. 复制资源文件](#5-复制资源文件)
  - [5.1 libs](#51-libs)
  - [5.2 assets](#52-assets)
  - [5.3 res](#53-res)
  - [5.4 AndroidManifest.xml](#54-androidmanifestxml)
  - [5.5 src（源码）](#55-src源码)
- [6. 添加到主项目](#6-添加到主项目)
- [7. 完整配置示例](#7-完整配置示例)
- [8. 常见问题与注意事项](#8-常见问题与注意事项)
- [9. uni-app vs uni-app x 差异对照表](#9-uni-app-vs-uni-app-x-差异对照表)

---

## 1. 前置说明与概念区分

### 1.1 资源位置

资源导出成功之后，uts 插件资源位于 `unpackage/resource/app-android/uni_modules` 下。

### 1.2 重要提示

> **注意**：`普通授权版` 加密付费 UTS 插件不支持通过原生 SDK 打包。需要拿到插件源码才可以。一般推荐购买**源码授权版**。

### 1.3 概念区分

为方便区分，本文档中使用以下术语：

| 术语 | 含义 |
|------|------|
| **UTS 插件** | 前端封装的 uni_modules 插件（`.utss` 文件编写的插件） |
| **Android UTS 插件模块** | 根据编译后的 UTS 插件生成的安卓原生模块（Android Library Module） |

### 1.4 uni-app 与 uni-app x 的核心差异

本教程专门针对 **uni-app**（非 uni-app x），两者在 UTS 插件配置上存在以下关键差异：

| 配置项 | uni-app x | **uni-app（本教程）** |
|--------|-----------|----------------------|
| Gradle 插件 `io.dcloud.uts.kotlin` | 必须添加 | **可忽略，不需要添加** |
| fastjson 版本 | `1.2.83` | **`1.1.46.android`** |
| kotlin-gradle-plugin | 不需要 | **需要 `1.5.10`** |
| core-ktx 版本 | `1.10.1` | **`1.6.0`** |
| kotlin-stdlib-jdk7 | 不需要 | **需要 `1.6.0`** |
| kotlin-reflect | 不需要 | **需要 `1.6.0`** |
| 组件注册方式 | build.gradle 的 `buildConfigField "UTSRegisterComponents"` | **`dcloud_uniplugins.json`** |

---

## 2. 新建 Android UTS 插件模块

在 Android Studio 中创建一个新的 Android Library 模块来承载 UTS 插件的原生代码。

### 操作步骤

1. 点击菜单 **File -> New -> New Module...**
2. 在左侧选择 **Templates** 中的 **Android Library**
3. 配置以下选项：

| 配置项 | 推荐值 | 说明 |
|--------|--------|------|
| Language | **Kotlin** | 必须选择 Kotlin |
| Module name | 与 UTS 插件模块名称一致 | 如 `uts-pluginName` |
| Build configuration language | **Groovy DSL (build.gradle)** | 本教程均按此模式进行 |

4. 点击 **Finish** 完成创建

### 注意事项

- **Templates 一定要选择 `Android Library`**，不能选择其他类型
- **Language 一定要选择 `Kotlin`**
- **Build configuration language 建议选择 `Groovy DSL (build.gradle)`**

---

## 3. 修改 build.gradle 配置

打开新创建的 Android Library 模块下的 `build.gradle` 文件，进行如下配置。

### 3.1 添加依赖（uni-app 专用版本）

将下面内容拷贝到 build.gradle 中，替换原有的 `dependencies` 节点：

```groovy
dependencies {
    compileOnly fileTree(include: ['*.aar'], dir: '../app/libs')
    compileOnly fileTree(include: ['*.aar'], dir: './libs')
    compileOnly 'com.alibaba:fastjson:1.1.46.android'
    compileOnly 'org.jetbrains.kotlin:kotlin-gradle-plugin:1.5.10'
    compileOnly 'androidx.core:core-ktx:1.6.0'
    compileOnly 'org.jetbrains.kotlin:kotlin-stdlib-jdk7:1.6.0'
    compileOnly 'org.jetbrains.kotlin:kotlin-reflect:1.6.0'
    compileOnly 'org.jetbrains.kotlinx:kotlinx-coroutines-core:1.3.8'
    compileOnly 'org.jetbrains.kotlinx:kotlinx-coroutines-android:1.3.8'
}
```

#### 依赖说明

| 依赖 | 版本 | 用途 |
|------|------|------|
| fastjson | `1.1.46.android` | JSON 解析（uni-app 专用版本） |
| kotlin-gradle-plugin | `1.5.10` | Kotlin Gradle 插件支持 |
| core-ktx | `1.6.0` | Android KTX 扩展库 |
| kotlin-stdlib-jdk7 | `1.6.0` | Kotlin 标准库（JDK7） |
| kotlin-reflect | `1.6.0` | Kotlin 反射支持 |
| kotlinx-coroutines-core | `1.3.8` | 协程核心库 |
| kotlinx-coroutines-android | `1.3.8` | Android 协程扩展 |

> **注意**：uni-app 可以忽略 gradle 插件 `io.dcloud.uts.kotlin` 的配置，无需在 `plugins` 节点中添加。

### 3.2 基础库依赖引用

Android UTS 插件模块编译时也需要依赖基础库。建议直接使用主应用模块（如 `app`）下的 libs 目录：

```groovy
dependencies {
    compileOnly fileTree(include: ['*.aar'], dir: '../app/libs')
    // ... 其他依赖
}
```

如果项目中有专门的 `uniappx` 模块存放基础库，也可以引用该目录：

```groovy
dependencies {
    compileOnly fileTree(include: ['*.aar'], dir: '../uniappx/libs')
    // ... 其他依赖
}
```

如果插件依赖其他内置模块，可参考[模块配置文档](https://doc.dcloud.net.cn/uni-app-x/native/modules/android/others.html)，将模块对应的配置添加到 app 项目下。

### 3.3 插件间依赖

如果当前 UTS 插件依赖其他 UTS 插件，建议优先将被依赖的 **UTS 插件**也配置成 **Android UTS 插件模块**。然后在当前 Android UTS 插件模块的 build.gradle 中添加对它的依赖：

```groovy
dependencies {
    implementation project(':uts-被依赖的android-uts插件模块名')
    // ... 其他依赖
}
```

例如，如果当前插件依赖名为 `uts-basePlugin` 的插件模块：

```groovy
dependencies {
    implementation project(':uts-basePlugin')
}
```

---

## 4. 根据 config.json 配置应用

如果 UTS 插件中不包含 `config.json` 文件，可以[跳过此章节](#5-复制资源文件)。

`config.json` 是 UTS 插件的元数据配置文件，定义了插件的各项属性。[config.json 详细配置参考文档](https://doc.dcloud.net.cn/uni-app-x/plugin/uts-plugin.html#androidconfigjson)

> **重要提示**：`abis`、`minSdkVersion`、`dependencies`、`project` 这几个字段在设置 Android UTS 插件模块时，**同时也需要设置到 app 主模块中**。

### config.json 示例结构

```json
{
    "abis": [
        "armeabi-v7a",
        "arm64-v8a"
    ],
    "dependencies": [
        "androidx.core:core-ktx:1.6.0",
        {
            "id": "com.xxx.richtext:richtext",
            "source": "implementation 'com.xxx.richtext:richtext:3.0.7'"
        }
    ],
    "minSdkVersion": 21,
    "project": {
        "plugins": [
            "com.huawei.agconnect"
        ],
        "dependencies": [
            "com.huawei.agconnect:agcp:1.6.0.300"
        ]
    },
    "components": [{"name": "zl-text", "class": "uts.sdk.modules.zlText.ZlTextComponent"}],
    "hooksClass": "uts.sdk.modules.zlText.ZlTextHook"
}
```

如果 config.json 中不存在某个字段，直接忽略即可。

### 4.1 abis（CPU 架构）

`abis` 表示插件支持的 CPU 架构类型。需要将支持的 CPU 类型添加到 Android UTS 插件模块的 `build.gradle` 中：

```groovy
android {
    defaultConfig {
        ndk {
            abiFilters "armeabi-v7a", "arm64-v8a"
        }
    }
}
```

### 4.2 minSdkVersion

`minSdkVersion` 表示插件最低支持的 Android 版本。修改 Android UTS 插件模块的 `build.gradle` 中的 `minSdkVersion` 即可。

```groovy
android {
    defaultConfig {
        minSdkVersion 21
    }
}
```

> **注意**：部分 Android Studio 生成的项目中字段名为 `minSdk` 而非 `minSdkVersion`，请注意区分。

### 4.3 dependencies（依赖库）

`dependencies` 为插件依赖的第三方仓储库。需要将这些依赖添加到 Android UTS 插件模块的 `build.gradle` 中。

#### 字符串类型的依赖

对于字符串形式的内容，需要拼接 `implementation` 并添加到 `build.gradle` 的 `dependencies` 节点下：

例如 config.json 中的 `"androidx.core:core-ktx:1.6.0"`：

```groovy
dependencies {
    // ...
    implementation 'androidx.core:core-ktx:1.6.0'
}
```

#### JSON 对象类型的依赖

对于 JSON 对象形式的内容，只需要将 `source` 对应的内容添加到 `build.gradle` 的 `dependencies` 下：

例如 config.json 中的：
```json
{
    "id": "com.xxx.richtext:richtext",
    "source": "implementation 'com.xxx.richtext:richtext:3.0.7'"
}
```

对应 build.gradle 配置：
```groovy
dependencies {
    // ...
    implementation 'com.xxx.richtext:richtext:3.0.7'
}
```

### 4.4 project（Gradle 插件）

`project` 为 Gradle 插件的配置，包含两个子节点：`plugins` 和 `dependencies`。

#### plugins 节点

`plugins` 下的内容需要添加到 Android UTS 插件模块的 `build.gradle` 的 `plugins` 节点下：

例如 config.json 中的 `"com.huawei.agconnect"`：

```groovy
plugins {
    // ...
    id 'com.huawei.agconnect'
}
```

#### project.dependencies 节点

`project.dependencies` 下的内容需要添加到**项目根目录**下的 `build.gradle` 的 `buildscript > dependencies` 中：

例如 config.json 中的 `"com.huawei.agconnect:agcp:1.6.0.300"`：

```groovy
buildscript {
    dependencies {
        // ...
        classpath "com.huawei.agconnect:agcp:1.6.0.300"
    }
}
```

### 4.5 components（组件注册）

`components` 为 UTS 组件的注册信息。

> **uni-app 与 uni-app x 的重大差异**：uni-app x 使用 `build.gradle` 中的 `buildConfigField` 注册组件，而 **uni-app 使用 `dcloud_uniplugins.json` 文件注册组件**。

#### uni-app 组件注册方式

需要将 components 对应的内容添加到主模块（app 模块）的 `dcloud_uniplugins.json` 文件中：

**文件位置**：`app/src/main/assets/dcloud_uniplugins.json`

> **注意**：`dcloud_uniplugins.json` 位于项目的 `assets` 目录下。如果没有该文件，需要手动创建。

**配置格式**：

```json
{
    "nativePlugins": [{
        "plugins": [{
            "type": "component",
            "name": "zl-text",
            "class": "uts.sdk.modules.zlText.ZlTextComponent"
        }]
    }]
}
```

#### 多个组件合并

如果项目中已有其他 UTS 组件注册，需要将新的组件信息合并到现有的 `dcloud_uniplugins.json` 中：

```json
{
    "nativePlugins": [{
        "plugins": [
            {
                "type": "component",
                "name": "zl-a",
                "class": "zlA.ZlAComponent"
            },
            {
                "type": "component",
                "name": "zl-text",
                "class": "uts.sdk.modules.zlText.ZlTextComponent"
            }
        ]
    }]
}
```

### 4.6 hooksClass（生命周期监听）

`hooksClass` 为 UTS 插件的应用程序生命周期函数监听类。[详细说明参考](https://doc.dcloud.net.cn/uni-app-x/plugin/uts-plugin.html#android-%E5%B9%B3%E5%8F%B0)

需要将 hooksClass 对应的内容添加到**主模块**的 `build.gradle` 中：

```groovy
android {
    defaultConfig {
        buildConfigField 'String[]', 'UTSHooksClassArray', '{"uts.sdk.modules.zlText.ZlTextHook"}'
    }
}
```

#### 多个 hooksClass 合并

如果主模块的 `build.gradle` 已经存在 `UTSHooksClassArray`，需要将现有配置与新配置合并：

```groovy
android {
    defaultConfig {
        buildConfigField 'String[]', 'UTSHooksClassArray', '{"uts.sdk.modules.zlText.ZlTextHook","uts.sdk.modules.zla.ZLAHook"}'
    }
}
```

> **注意：转义符不能删掉，格式必须严格一致！**

---

## 5. 复制资源文件

根据 UTS 插件的资源目录结构，将对应的内容复制到 Android UTS 插件模块下。

> **说明**：不存在的目录可以不处理。

目标路径为 Android UTS 插件模块的 `src/main/` 目录下。

### 5.1 libs

将 UTS 插件 `libs` 目录下的 `.aar` 和 `.jar` 库文件拷贝到 Android UTS 插件模块的 `libs` 目录下。

拷贝完成后，需要在 Android UTS 插件模块的 `build.gradle` 中添加对这些本地库的依赖：

```groovy
dependencies {
    // ...
    compileOnly fileTree(include: ['*.aar', '*.jar'], dir: './libs')
}
```

> **注意**：UTS 插件的本地 libs 下的依赖同样也需要添加到**主模块（app）**中。

### 5.2 assets

如果 UTS 插件存在 `assets` 目录，需要将整个 `assets` 文件夹拷贝到 Android UTS 插件模块的 `src/main/` 目录下。

```
uts-plugin-module/src/main/assets/
```

### 5.3 res

如果 UTS 插件存在 `res` 目录，需要将整个 `res` 文件夹拷贝到 Android UTS 插件模块的 `src/main/` 目录下。

```
uts-plugin-module/src/main/res/
```

### 5.4 AndroidManifest.xml

如果 UTS 插件存在 `AndroidManifest.xml` 文件，需要将其拷贝到 Android UTS 插件模块的 `src/main/` 目录下。

```
uts-plugin-module/src/main/AndroidManifest.xml
```

> **重要**：如果 `AndroidManifest.xml` 中设置了 `package` 字段，**必须将此字段删除**，并将 package 的内容设置到 `build.gradle` 的 `namespace` 属性中。

### 5.5 src（源码）

将 UTS 插件 `src` 目录下的所有源代码文件（`.kt`、`.java` 等）拷贝到 Android UTS 插件模块的 `src/main/java` 目录下，保持原有的包名目录结构。

```
uts-plugin-module/src/main/java/
├── uts/
│   └── sdk/
│       └── modules/
│           └── yourPlugin/
│               ├── YourPluginComponent.kt
│               └── YourPluginHook.kt
```

---

## 6. 添加到主项目

完成以上步骤后，需要将 Android UTS 插件模块的依赖添加到**主模块（app）** 的 `build.gradle` 中：

```groovy
dependencies {
    // ...
    // 将 uts-progressNotification 替换为你实际的模块名称
    implementation project(':uts-yourPluginName')
}
```

同时确保在项目的 `settings.gradle` 中已经包含了该模块：

```groovy
include ':app'
include ':uts-yourPluginName'  // 确保包含你的 UTS 插件模块
```

---

## 7. 完整配置示例

### 7.1 Android UTS 插件模块 build.gradle 完整示例

假设我们有一个名为 `uts-zlText` 的 UTS 插件，其完整配置如下：

```groovy
plugins {
    id 'com.android.library'
    // uni-app 可以忽略 io.dcloud.uts.kotlin 插件
}

android {
    namespace "uts.sdk.modules.zlText"
    compileSdk 34

    defaultConfig {
        minSdkVersion 21
        targetSdkVersion 34

        ndk {
            abiFilters "armeabi-v7a", "arm64-v8a"
        }

        testInstrumentationRunner "androidx.test.runner.AndroidJUnitRunner"
        consumerProguardFiles "consumer-rules.pro"
    }

    buildTypes {
        release {
            minifyEnabled false
            proguardFiles getDefaultProguardFile('proguard-android-optimize.txt'), 'proguard-rules.pro'
        }
    }

    compileOptions {
        sourceCompatibility JavaVersion.VERSION_1_8
        targetCompatibility JavaVersion.VERSION_1_8
    }

    kotlinOptions {
        jvmTarget = '1.8'
    }
}

dependencies {
    // 主模块 libs 中的基础库
    compileOnly fileTree(include: ['*.aar'], dir: '../app/libs')

    // 当前插件模块 libs 中的本地库
    compileOnly fileTree(include: ['*.aar', '*.jar'], dir: './libs')

    // uni-app 专用依赖版本
    compileOnly 'com.alibaba:fastjson:1.1.46.android'
    compileOnly 'org.jetbrains.kotlin:kotlin-gradle-plugin:1.5.10'
    compileOnly 'androidx.core:core-ktx:1.6.0'
    compileOnly 'org.jetbrains.kotlin:kotlin-stdlib-jdk7:1.6.0'
    compileOnly 'org.jetbrains.kotlin:kotlin-reflect:1.6.0'
    compileOnly 'org.jetbrains.kotlinx:kotlinx-coroutines-core:1.3.8'
    compileOnly 'org.jetbrains.kotlinx:kotlinx-coroutines-android:1.3.8'

    // 如果有额外的第三方依赖（根据 config.json）
    implementation 'androidx.core:core-ktx:1.6.0'

    // 如果依赖了其他 UTS 插件模块
    // implementation project(':uts-otherPlugin')
}
```

### 7.2 主模块（app）build.gradle 中的相关配置

```groovy
android {
    defaultConfig {
        // ...

        // CPU 架构（需与插件 abis 一致）
        ndk {
            abiFilters "armeabi-v7a", "arm64-v8a"
        }

        // UTS 生命周期监听类注册
        buildConfigField 'String[]', 'UTSHooksClassArray', '{"uts.sdk.modules.zlText.ZlTextHook"}'
    }
}

dependencies {
    // ...
    // 引用 UTS 插件模块
    implementation project(':uts-zlText')
}
```

### 7.3 dcloud_uniplugins.json 示例

**文件路径**：`app/src/main/assets/dcloud_uniplugins.json`

```json
{
    "nativePlugins": [{
        "plugins": [{
            "type": "component",
            "name": "zl-text",
            "class": "uts.sdk.modules.zlText.ZlTextComponent"
        }]
    }]
}
```

### 7.4 项目根目录 build.gradle 中的额外配置（如有）

如果 config.json 中包含 `project.plugins` 或 `project.dependencies`：

```groovy
buildscript {
    repositories {
        google()
        mavenCentral()
        // 华为仓库（如果使用华为 AGC）
        maven { url 'https://developer.huawei.com/repo/' }
    }
    dependencies {
        // ...
        // 华为 AGC classpath（根据 config.json project.dependencies 配置）
        classpath "com.huawei.agconnect:agcp:1.6.0.300"
    }
}

allprojects {
    repositories {
        google()
        mavenCentral()
    }
}
```

---

## 8. 常见问题与注意事项

### 8.1 applicationIdSuffix 导致组件初始化失败

> **暂不支持在 build.gradle 中设置 `applicationIdSuffix`**，添加 `applicationIdSuffix` 会导致组件初始化失败。

### 8.2 AndroidManifest.xml 的 package 字段处理

如果从 UTS 插件复制的 `AndroidManifest.xml` 中包含 `package` 字段，**必须删除该字段**，并将其值设置到 `build.gradle` 的 `namespace` 属性中：

```groovy
android {
    namespace "uts.sdk.modules.yourPlugin"  // 原 AndroidManifest.xml 的 package 值
}
```

### 8.3 转义符问题

在 `buildConfigField` 中配置 `UTSHooksClassArray` 时，**转义符不能删掉**，格式必须严格一致。错误的转义会导致运行时解析失败。

### 8.4 多插件 components 合并

当项目中有多个 UTS 插件都包含自定义组件时，所有组件都需要合并在同一个 `dcloud_uniplugins.json` 文件的 `nativePlugins[0].plugins` 数组中。

### 8.5 编译顺序建议

建议按照以下顺序操作以避免遗漏：

1. 创建 Android Library 模块
2. 配置 build.gradle（依赖 + abis + minSdkVersion）
3. 复制资源文件（libs / assets / res / AndroidManifest.xml / src）
4. 配置主模块（添加 module dependency + UTSHooksClassArray + dcloud_uniplugins.json）
5. 配置项目根目录 build.gradle（如果有 project 级别的 plugins/dependencies）
6. Sync Project with Gradle Files
7. 编译验证

### 8.6 普通授权版插件限制

普通授权版的加密付费 UTS 插件不支持通过原生 SDK 打包。需要拿到插件源码才可以集成。推荐购买**源码授权版**。

### 8.7 依赖冲突排查

如果在编译过程中遇到依赖冲突，请检查以下几点：

- 确保 fastjson 使用的是 `1.1.46.android` 版本（而非 `1.2.83`）
- 确保 Kotlin 相关依赖版本一致（stdlib / reflect / coroutines）
- 检查是否有多个模块引入了相同的三方库但版本不同

---

## 9. uni-app vs uni-app x 差异对照表

| 配置步骤 | uni-app x | uni-app（本教程） |
|----------|-----------|------------------|
| **Gradle 插件** | 需要添加 `id 'io.dcloud.uts.kotlin'` | **不需要，可忽略** |
| **fastjson** | `compileOnly "com.alibaba:fastjson:1.2.83"` | **`compileOnly 'com.alibaba:fastjson:1.1.46.android'`** |
| **kotlin-gradle-plugin** | 无需额外配置 | **`compileOnly 'org.jetbrains.kotlin:kotlin-gradle-plugin:1.5.10'`** |
| **core-ktx** | `compileOnly "androidx.core:core-ktx:1.10.1"` | **`compileOnly 'androidx.core:core-ktx:1.6.0'`** |
| **kotlin-stdlib-jdk7** | 无需配置 | **`compileOnly 'org.jetbrains.kotlin:kotlin-stdlib-jdk7:1.6.0'`** |
| **kotlin-reflect** | 无需配置 | **`compileOnly 'org.jetbrains.kotlin:kotlin-reflect:1.6.0'`** |
| **components 注册** | `buildConfigField "String", "UTSRegisterComponents", "..."` | **`dcloud_uniplugins.json` 文件** |
| **hooksClass 注册** | `buildConfigField("String[]", "UTSHooksClassArray", "...")` | 相同，使用 `buildConfigField` |
| **基础库引用** | `fileTree(dir: '../uniappx/libs')` | **`fileTree(dir: '../app/libs')`** |

---

## 附录：文件目录结构参考

配置完成后的典型项目结构如下：

```
MyUniAppProject/
├── app/                          # 主模块
│   ├── build.gradle              # 主模块构建配置（含 UTSHooksClassArray、implementation project）
│   └── src/main/
│       ├── assets/
│       │   └── dcloud_uniplugins.json   # UTS 组件注册文件
│       └── libs/                          # 基础库 aar 文件
│           └── *.aar
├── utss-pluginName/              # Android UTS 插件模块
│   ├── build.gradle              # 插件模块构建配置
│   └── src/main/
│       ├── AndroidManifest.xml
│       ├── assets/               # （可选）插件资源
│       ├── java/                 # 插件源码
│       │   └── uts/sdk/modules/pluginName/
│       │       └── *.kt
│       ├── libs/                 # 插件本地依赖库
│       │   └── *.aar, *.jar
│       └── res/                  # （可选）插件资源
├── build.gradle                  # 项目根目录构建配置
├── settings.gradle               # 项目设置（含 include）
└── ...
```

---

**文档生成时间**：2026-06-05
**基于官方文档**：https://doc.dcloud.net.cn/uni-app-x/native/use/androiduts.html
**适用范围**：Android uni-app（非 uni-app x）离线原生打包场景
