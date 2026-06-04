# Android UniApp SDK 集成指南 (基于 HBuilderX 4.87+)

## 📚 1. 概述与核心概念

UniApp Android SDK（即 DCloud App 离线开发工具包）是 DCloud 提供的官方原生开发工具包。它将 UniApp 应用的运行环境封装为原生开发接口，方便开发者在自己的 Android 原生项目中直接集成并运行 UniApp 应用，实现了 App 本地离线打包及扩展原生能力。

**App 本地离线打包**：对应 HBuilderX 的云端打包功能，在打包时无需将 App 资源及打包要使用的签名证书等提交到云端打包服务器，直接在开发者本地配置的原生开发环境中生成安装包 apk/ipa。

**扩展原生能力**：当 HBuilderX 中提供的能力无法满足 App 功能需求时，可以使用 App 离线 SDK 开发原生插件来扩展原生能力。

---

## 🔧 2. 开发环境准备

在开始集成之前，需要确保您的本地开发环境满足以下要求：

- **开发工具**：
  - HBuilderX 4.87+（正式版或 alpha 均可）
  - Android Studio（最新稳定版）
- **Java 开发工具包**：JDK 8 或 11（需配置 `JAVA_HOME` 环境变量）
- **Android SDK**：通过 Android Studio 的 SDK Manager 安装最新版本的 Android SDK（如 Android 12+）
- **Gradle 配置要求（针对 4.81 及以上版本）**：从 HBuilderX 4.81 版起，为了适配 16KB 内存页模式，需配置 `compileSdk` 为 36、`buildToolsVersion` 为 36.0.0、Gradle 版本为 8.14.3、Android Gradle 插件版本为 8.12.0。

---

## 📥 3. SDK 下载与目录说明

从 UniApp 官方网站下载最新版的 Android 离线 SDK。
> ⚠️ **重要提示**：从 **3.1.10 版本起**，使用 App 离线 SDK 需要申请 Appkey。

SDK 包解压后主要包含以下关键目录和文件：

```
|-- HBuilder-Hello                # App 离线打包演示应用
|-- HBuilder-Integrate-AS         # 集成 UniApp 的最简示例（推荐直接导入）
|-- SDK                          # SDK 库文件目录
|-- Feature-Android.xls           # Android 平台各扩展 Feature API 对应的详细配置说明
|-- Readme.txt                    # 版本说明文件及注意事项
|-- UniPlugin-Hello-AS            # Uni 原生插件开发示例工程
```

---

## 🛠️ 4. 工程配置与集成步骤

从 2.7.0 版本起，SDK 提供了 `HBuilder-Integrate-AS` 工程，可以直接导入 Android Studio，然后直接运行其中的 `simpleDemo` 项目。具体操作如下：

1.  将解压后的 SDK 目录中的 `HBuilder-Integrate-AS` 工程文件夹复制到一个**没有中文路径**的文件夹中。
2.  打开 Android Studio，选择 **File → Open**，然后定位到刚刚复制的 `HBuilder-Integrate-AS` 文件夹，点击 **OK** 导入工程。
3.  等待 Android Studio 完成 Gradle 同步。
4.  在导入的工程中找到 `simpleDemo` 模块，直接运行即可看到 UniApp 示例应用。

### 4.1 现有项目中的集成参考

如果您需要在已有的 Android 项目中集成 UniApp，可以参考 `HBuilder-Integrate-AS` 中的工程结构、build.gradle 配置、AndroidManifest.xml 中的 Application 和 Appkey 配置，以及资源文件的放置方式。

### 4.2 build.gradle 关键配置（参考）

在应用模块的 `build.gradle` 文件中通常需要进行以下配置：

**① 添加资源引用**（HBuilderX 3.2.5 版本之后已适配 AndroidX）：

```groovy
dependencies {
    implementation fileTree(include: ['*.jar'], dir: 'libs')
    implementation fileTree(include: ['*.aar'], dir: 'libs')
    implementation 'androidx.appcompat:appcompat:1.1.0'
    implementation 'androidx.localbroadcastmanager:localbroadcastmanager:1.0.0'
    implementation 'androidx.core:core:1.6.0'
    implementation "androidx.fragment:fragment:1.1.0"
    implementation 'androidx.recyclerview:recyclerview:1.1.0'
    implementation "com.facebook.fresco:fresco:3.4.0"
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

**② aaptOptions 配置**：

```groovy
aaptOptions {
    additionalParameters '--auto-add-overlay'
    ignoreAssetsPattern "!.svn:!.git:.*:!CVS:!thumbs.db:!picasa.ini:!*.scc:*~"
}
```

**③ UTS 插件支持**（如项目包含 UTS 原生插件）：

- 将 `utsplugin-release.aar` 拷贝到项目的 `libs` 目录下。
- 在 `dependencies` 中添加 `implementation "com.squareup.okhttp3:okhttp:3.12.12"`。

**④ targetSdkVersion 配置**：

建议将 `targetSdkVersion` 设置为 **30 或以上**。
> 如果 targetSdkVersion 设置为 34 时，需要在 `build.gradle` 的 android 节点下新增以下内容：
> ```groovy
> packagingOptions {
>     jniLibs {
>         useLegacyPackaging true
>     }
> }
> ```

### 4.3 配置 Application 和 Appkey

① **配置 Appkey**（3.1.10 版本起必须执行）：
打开 `AndroidManifest.xml`，找到 Application 节点，创建 `meta-data` 节点：
```xml
<meta-data
    android:name="dcloud_appkey"
    android:value="您申请的AppKey" />
```

② **配置 Application**：
如果需要自定义 Application，必须继承自 `DCloudApplication`，否则会导致 SDK 业务逻辑无法正常运行。
```xml
<application
    android:name=".MyDCloudApplication"
    ... >
```

### 4.4 生成 UniApp 资源

在 HBuilderX 中：
1.  打开您的 UniApp 项目。
2.  点击菜单栏的 **“发行” → “原生App-本地打包” → “生成本地打包App资源”**。
3.  打包完成后，会生成一个名为 `__UNI__XXX` 的资源文件夹（含 `www` 目录，存放编译后的前端资源）。
4.  将该文件夹中的资源文件复制到 Android 项目的 `assets/apps/` 目录下。

---

## 🔌 5. 原生插件开发与集成

为了满足个性化业务需求，UniApp 支持将原生能力封装成“原生插件”，然后在前端用 JS 调用。整个流程分为：**准备 → 开发 → 集成**。

### 5.1 创建插件

在 uni-app 项目根目录执行命令，创建原生插件：

```shell
hx --createPlugin android my-native-plugin
```
这会在 `uni_modules/my-native-plugin` 目录下生成 Android 原生代码模板。

### 5.2 编写原生代码

在 `android/src/main/java/...` 目录下新建类：
- 继承 `StandardFeature`（对应 Module 模块，用于调用方法）或 `Component`（对应 Component 组件，用于创建视图）。
- 按模板实现业务方法。

### 5.3 声明接口与前端调用

① **声明接口**：在 `android/assets/dcloud_uniplugins.json` 文件中，将编写的类声明为 `module` 或 `component`，并配置好方法名和参数类型。

② **前端调用**：在 UniApp 的 JS 代码中通过以下方式调用：

```javascript
const myPlugin = uni.requireNativePlugin('my-native-plugin');
myPlugin.hello({ name: 'uniapp' }, res => {
    console.log(res);
});
```

### 5.4 调试与发布

- **调试**：在 HBuilderX 中真机运行自定义基座，HBuilderX 会自动把插件编进去。如果需要打断点，可以用 Android Studio 打开插件的 `android` 目录进行调试。
- **隐私合规**：如果插件涉及敏感权限或采集用户信息，必须在插件 `package.json` 的 `privacy` 字段里声明，并在宿主 App 的隐私政策中同步说明，否则上架会被拒。
- **发布**：开发完成后可将插件上传到 DCloud 插件市场，供其他开发者一键安装；也可以私用，直接放在自己项目的 `uni_modules` 里。

---

## ✅ 6. 集成验证与运行

1.  连接 Android 手机或启动模拟器。
2.  在 Android Studio 中点击运行按钮（Run）。
3.  检查 App 是否能正常启动，并验证前端页面是否能正常调用原生插件功能。

---

## ⚠️ 7. 注意事项与常见问题

- **Kotlin 支持**：App 离线 SDK **不支持 Kotlin**。
- **版本对齐**：**SDK 版本必须严格与 HBuilderX 版本一一对应**。如果版本不一致，运行时会弹窗报警“当前自定义基座的SDK与HBuilderX自带的基座SDK版本不一致”。请注意离线 SDK 是独立于 HBuilderX 下载的，HBuilderX 的版本决定了 `uniCompileVersion`，必须与离线 SDK 的 `uniRuntimeVersion` 匹配。
- **M1/M2 芯片 Mac**：如果使用的是搭载 M1/M2 芯片的 Mac 电脑，请注意 Gradle 和 NDK 的兼容性问题。
- **5+ SDK 废弃提醒**：原 5+ SDK 的 Widget 方式集成和 WebView 方式集成已停止维护，功能已迁移到 **Uni 小程序 SDK**，建议新项目统一采用 UniApp 模式开发。

---

## 🔗 8. 扩展阅读与资源

- **UniApp 原生插件开发总览**：[https://nativesupport.dcloud.net.cn/NativePlugin/README](https://nativesupport.dcloud.net.cn/NativePlugin/README)
- **UTS 插件 Android 运行配置**：[https://uniapp.dcloud.net.cn/tutorial/run/uts-development-android.html](https://uniapp.dcloud.net.cn/tutorial/run/uts-development-android.html)
- **Android 离线 SDK 下载页**：[https://nativesupport.dcloud.net.cn/AppDocs/download/android.html](https://nativesupport.dcloud.net.cn/AppDocs/download/android.html)
- **AppKey 申请指南**：[https://nativesupport.dcloud.net.cn/AppDocs/usesdk/appkey.html](https://nativesupport.dcloud.net.cn/AppDocs/usesdk/appkey.html)
- **FAQ 文档**：[https://nativesupport.dcloud.net.cn/AppDocs/FAQ/android.html](https://nativesupport.dcloud.net.cn/AppDocs/FAQ/android.html)