# UTS 基础模块（Android）

> **适用版本**：HBuilderX 5.0+
> **平台**：Android
> **官方文档**：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/androidModuleConfig/

## 13. UTS 基础模块

### 需要拷贝的文件

| 路径 | 文件 |
|---|---|
| SDK\libs | `utsplugin-release.aar` |

### app级build.gradle配置

```groovy
dependencies {
    implementation "com.squareup.okhttp3:okhttp:3.12.12"
    implementation "androidx.core:core-ktx:1.6.0"
    implementation "org.jetbrains.kotlin:kotlin-stdlib:2.2.0"
    implementation "org.jetbrains.kotlin:kotlin-reflect:2.2.0"
    implementation "org.jetbrains.kotlinx:kotlinx-coroutines-core:1.8.1"
    implementation "org.jetbrains.kotlinx:kotlinx-coroutines-android:1.8.1"
    implementation "com.github.getActivity:XXPermissions:18.63"
}
```

### 项目根目录build.gradle配置（添加jitpack依赖）

```groovy
allprojects {
    ...
    repositories {
        maven { url 'https://jitpack.io' }
    }
}
```

---

### 相关模块

- [UTS 内置模块](uts-builtin-modules.md) — 依赖于此基础模块
- [FacialRecognitionVerify 实人认证](facial-recognition-verify.md) — 依赖于此基础模块
- [Geolocation 定位](geolocation.md) — 腾讯定位依赖于此基础模块
