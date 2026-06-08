# UTS 内置模块（Android）

> **适用版本**：HBuilderX 5.0+
> **平台**：Android
> **官方文档**：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/androidModuleConfig/

## 12. UTS 内置模块（utsPlugins）

> **UTS插件依赖于UTS基础模块，请先集成[UTS基础模块](uts-base-module.md)**

### API与对应的库参照表

| 模块名称 | 本地依赖库 | 线上依赖库 | 依赖的模块 |
|---|---|---|---|
| uni-createRequestPermissionListener | uni-createRequestPermissionListener-release.aar | - | - |
| uni-getNetworkType | uni-getNetworkType-release.aar | - | - |
| uni-installApk | uni-installApk-release.aar | - | - |
| uni-network | uni-network-release.aar | com.squareup.okhttp3:okhttp:3.12.12 | - |
| uni-privacy | uni-privacy-release.aar | - | - |
| uni-chooseMedia | uni-chooseMedia-release.aar | androidx.appcompat:appcompat:1.6.1, androidx.activity:activity-ktx:1.9.2 | uni-prompt |
| uni-getAppBaseInfo | uni-getAppBaseInfo-release.aar | - | - |
| uni-storage | uni-storage-release.aar | - | - |
| uni-getSystemInfo | uni-getSystemInfo-release.aar | - | - |
| uni-getDeviceInfo | uni-getDeviceInfo-release.aar | - | - |
| uni-openAppAuthorizeSetting | uni-openAppAuthorizeSetting-release.aar | - | - |
| uni-exit | uni-exit-release.aar | - | - |
| uni-getAccessibilityInfo | uni-getAccessibilityInfo-release.aar | - | - |
| uni-getAppAuthorizeSetting | uni-getAppAuthorizeSetting-release.aar | - | - |
| uni-getSystemSetting | uni-getSystemSetting-release.aar | - | - |
| uni-prompt | uni-prompt-release.aar | androidx.recyclerview:recyclerview:1.0.0, androidx.appcompat:appcompat:1.0.0 | - |
| uni-getLocation-tencent-uni1 | uni-getLocation-tencent-uni1-release.aar | com.tencent.map.geolocation:TencentLocationSdk-openplatform:7.5.4.8 | - |

### 配置方法

**本地依赖库**：将上表中本地依赖库对应的aar拷贝到app模块的libs目录下。

**线上依赖库**：添加到app模块的build.gradle中，例如：
```groovy
dependencies {
    implementation 'com.squareup.okhttp3:okhttp:3.12.12'
}
```

> **注意**：部分插件会依赖其他插件模块（如uni-chooseMedia依赖uni-prompt），需要将依赖的插件也集成到项目中。

---

### 相关模块

- [UTS 基础模块](uts-base-module.md) — UTS内置模块的前置依赖
- [Geolocation 定位](geolocation.md) — 腾讯定位使用UTS内置模块中的uni-getLocation-tencent-uni1
