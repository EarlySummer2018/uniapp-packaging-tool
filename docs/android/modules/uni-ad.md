# uni-AD 广告（Android）

> **适用版本**：HBuilderX 5.0+
> **平台**：Android
> **官方文档**：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/androidModuleConfig/

## 10. uni-AD（广告）

> 配置离线广告之前，需先在[dcloud广告联盟](https://uniad.dcloud.net.cn)申请账号。

**公共配置（AndroidManifest.xml的application节点）：**

```xml
<meta-data android:name="DCLOUD_AD_SPLASH" android:value="true"/><!--如果不开启开屏广告则不设置此字段或者值设置为false--> 
<meta-data android:name="DCLOUD_STREAMAPP_CHANNEL" android:value="包名|应用标识|广告标识|渠道，如io.dcloud.appid|appid|adid|google" />
```

字段说明：
- **包名**：对应Android项目中build.gradle中的applicationId
- **应用标识**：对应5+ APP或uni-app项目manifest.json中appid
- **广告标识**：联盟ID，开通后可在uniad.dcloud.net.cn获取
- **渠道**：参考[渠道包制作指南](https://ask.dcloud.net.cn/article/35974)

---

### 穿山甲

> **穿山甲GroMore广告与穿山甲广告互斥，集成时必须二选一。**

| 路径 | 文件名 |
|---|---|
| SDK/libs | `ads-release.aar`, `ads-csj-release.aar`, `open_ad_sdk.aar` |

**AndroidManifest.xml（application节点下，替换`${applicationId}`为实际包名）：**
```xml
<provider android:name="com.bytedance.sdk.openadsdk.TTFileProvider"
    android:authorities="${applicationId}.TTFileProvider"
    android:exported="false" android:grantUriPermissions="true">
    <meta-data android:name="android.support.FILE_PROVIDER_PATHS"
        android:resource="@xml/file_paths" tools:replace="android:resource"/>
</provider>
<provider android:name="com.bytedance.sdk.openadsdk.multipro.TTMultiProvider"
    android:authorities="${applicationId}.TTMultiProvider" android:exported="false" />
```

**dcloud_properties.xml：**
```xml
<feature name="Ad" value="io.dcloud.feature.ad.AdFlowFeatureImpl">
    <module name="csj" value="io.dcloud.feature.ad.csj.ADCsjModule"/>
</feature>
```

---

### 腾讯优量汇

| 路径 | 文件名 |
|---|---|
| SDK/libs | `ads-release.aar`, `ads-gdt-release.aar`, `GDTSDK.unionNormal.aar` |

**dcloud_properties.xml：**
```xml
<feature name="Ad" value="io.dcloud.feature.ad.AdFlowFeatureImpl">
    <module name="gdt" value="io.dcloud.feature.ad.gdt.ADGdtModule"/>
</feature>
```

---

### 快手

> 快手广告联盟跟快手内容联盟只能二选一。

| 类型 | 路径 | 文件名 |
|---|---|---|
| 快手广告联盟 | SDK/libs | `ads-release.aar`, `ads-ks-release.aar`, `ks_adsdk-ad.aar` |
| 快手内容联盟 | SDK/libs | `ads-release.aar`, `ads-ks-content-release.aar`, `kssdk-allad-content.aar` |

**AndroidManifest.xml（manifest节点下）：**
```xml
<uses-permission android:name="android.permission.ACCESS_NETWORK_STATE" />
<uses-permission android:name="android.permission.READ_PHONE_STATE" />
<uses-permission android:name="android.permission.ACCESS_WIFI_STATE" />
<permission android:name="${applicationId}.permission.KW_SDK_BROADCAST" android:protectionLevel="signature" />
<uses-permission android:name="${applicationId}.permission.KW_SDK_BROADCAST" />
```

**dcloud_properties.xml：**
```xml
<feature name="Ad" value="io.dcloud.feature.ad.AdFlowFeatureImpl">
    <module name="ks" value="io.dcloud.feature.ad.ks.ADKsModule"/>
</feature>
```

---

### Sigmob

| 路径 | 文件名 |
|---|---|
| SDK/libs | `ads-release.aar`, `ads-sigmob-release.aar`, `windAd.aar`, `wind-common.aar` |

**AndroidManifest.xml权限：**
```xml
<uses-permission android:name="android.permission.INTERNET" />
<uses-permission android:name="android.permission.ACCESS_NETWORK_STATE" />
<uses-permission android:name="android.permission.ACCESS_WIFI_STATE" />
<uses-permission android:name="android.permission.READ_PHONE_STATE" />
<uses-permission android:name="android.permission.ACCESS_COARSE_LOCATION" />
<uses-permission android:name="android.permission.ACCESS_FINE_LOCATION" />
<uses-permission android:name="android.permission.WRITE_EXTERNAL_STORAGE" tools:node="replace"/>
<uses-permission android:name="android.permission.READ_EXTERNAL_STORAGE"/>
<uses-permission android:name="android.permission.CHANGE_WIFI_STATE" />
<uses-permission android:name="android.permission.REQUEST_INSTALL_PACKAGES" />
```

**dcloud_properties.xml：**
```xml
<feature name="Ad" value="io.dcloud.feature.ad.AdFlowFeatureImpl">
    <module name="sgm" value="io.dcloud.feature.ad.sigmob.ADSMModule"/>
</feature>
```

---

### 百度广告（最低支持版本：离线sdk 3.4.1）

| 路径 | 文件名 |
|---|---|
| SDK/libs | `ads-release.aar`, `ads-bd-release.aar`, `Baidu_MobAds_SDK.aar` |

**dcloud_properties.xml：**
```xml
<feature name="Ad" value="io.dcloud.feature.ad.AdFlowFeatureImpl">
    <module name="bd" value="io.dcloud.feature.bd.ADBDModule" />
</feature>
```

---

### 华为广告（最低支持版本：离线sdk 3.4.1）

| 路径 | 文件名 |
|---|---|
| SDK/libs | `ads-release.aar`, `ads-hw-release.aar` |

**Gradle配置：**
```groovy
// project级
classpath 'com.huawei.agconnect:agcp:1.6.0.300'
maven {url 'https://developer.huawei.com/repo/'}

// app级
implementation 'com.huawei.hms:ads-lite:13.4.56.302'
implementation 'com.huawei.hms:ads-omsdk:1.3.35'
```

**dcloud_properties.xml：**
```xml
<feature name="Ad" value="io.dcloud.feature.ad.AdFlowFeatureImpl">
    <module name="hw" value="io.dcloud.feature.hw.AdHwModule" />
</feature>
```

---

### 穿山甲GroMore（最低支持版本：离线sdk 3.5.2）

> 与穿山甲广告互斥，必须二选一。

| 路径 | 文件名 |
|---|---|
| SDK/libs | `ads-release.aar`, `ads-gromore-release.aar`, `open_ad_sdk.aar` |

**dcloud_properties.xml：**
```xml
<feature name="Ad" value="io.dcloud.feature.ad.AdFlowFeatureImpl">
    <module name="gm" value="io.dcloud.feature.ad.gm.AdGMModule"/>
</feature>
```

---

### uniMP激励视频广告（最低支持版本：离线sdk 3.7.13）

| 路径 | 文件名 |
|---|---|
| SDK/libs | `ads-release.aar`, `ads-wm-release.aar`, `wechat-sdk-android-without-mta-6.8.0.aar` |

> 不需要配置dcloud_properties.xml文件

需引入`WXEntryActivity.java`（如已集成微信登录/分享可跳过），并配置AndroidManifest.xml（同微信登录配置）。

---

### 其他广告平台

| 广告平台 | 所需文件 |
|---|---|
| **章鱼** | `ads-release.aar`, `octopus_ad_sdk_XXXX.aar`, `uniad-zy-release.aar` |
| **倍孜** | `ads-release.aar`, `uniad_bz_adapter_5.2.2.0.aar`, `beizi_fusion_sdk_5.2.3.2.aar` |
| **聚力阅盟** | `YmDCloudymSdk20240617.aar` |
| **泛连** | （见原文档） |

---

### 相关模块

- [Share 分享](share.md) — 微信分享/登录与激励视频广告共用微信SDK
- [Push 消息推送](push.md) — 华为push也依赖华为AGC
- [第三方 SDK 依赖说明](third-party-dependencies.md) — 各广告平台版本信息
- [FAQ](../faq.md) — FAQ第18条关于uni-AD业务状态异常的处理
