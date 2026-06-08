# Statistic 统计（Android）

> **适用版本**：HBuilderX 5.0+
> **平台**：Android
> **官方文档**：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/androidModuleConfig/

## 8. Statistic（统计）

### 友盟统计

#### AndroidManifest.xml配置

**权限：**
```xml
<uses-permission android:name="android.permission.READ_PHONE_STATE" />,
<uses-permission android:name="android.permission.ACCESS_WIFI_STATE" />,
<uses-permission android:name="android.permission.ACCESS_NETWORK_STATE" />,
<uses-permission android:name="android.permission.INTERNET"/>
```

**application节点内：**
```xml
<meta-data android:name="UMENG_APPKEY" android:value="%appkey_android%" />
<meta-data android:name="UMENG_CHANNEL" android:value="%channelid_android%" />
```

#### 需要拷贝的文件

| 路径 | 文件 |
|---|---|
| SDK\libs | `statistic-release.aar`, `statistic-umeng-release.aar` |

#### 通过gradle集成友盟SDK

```groovy
dependencies {
    implementation 'com.umeng.umsdk:common:9.6.1'
    implementation 'com.umeng.umsdk:asms:1.8.0'
    implementation 'com.umeng.umsdk:abtest:1.0.1'
    implementation 'com.umeng.umsdk:apm:1.9.1'
}
```

#### dcloud_properties.xml配置

**features节点：**
```xml
<feature name="Statistic" value="io.dcloud.feature.statistics.StatisticsFeatureImpl">
    <module name="Statistic-Umeng" value="io.dcloud.feature.statistics.umeng.UmengStatistics" />
</feature>
```
**services节点：**
```xml
<service name="Statistic-Umeng" value="io.dcloud.feature.statistics.umeng.StatisticsBootImpl"/>
```

---

### 友盟统计-google play

#### 需要拷贝的文件

| 路径 | 文件 |
|---|---|
| SDK\libs | `statistic-release.aar`, `statistic-umeng-gp-release.aar` |

#### gradle依赖

```groovy
dependencies {
    implementation 'com.umeng.umsdk:apm:1.9.5'
}
```

dcloud_properties.xml配置同友盟统计。

---

### 谷歌统计

#### Gradle配置

**project级build.gradle：**
```groovy
buildscript {
    repositories {
        google()
    }
    dependencies {
        classpath 'com.google.gms:google-services:4.2.0'
    }
}
allprojects {
    repositories {
        google()
    }
}
```

**app级build.gradle：**
```groovy
apply plugin: 'com.google.gms.google-services'

dependencies {
    implementation 'com.google.firebase:firebase-analytics:21.3.0'
}
```

#### 需要拷贝的文件

下载`google-services.json`文件放到对应文件夹下。

| 路径 | 文件 |
|---|---|
| SDK\libs | `statistic-release.aar`, `statistic-google-release.aar` |

#### dcloud_properties.xml配置

```xml
<feature name="Statistic" value="io.dcloud.feature.statistics.StatisticsFeatureImpl">
    <module name="Statistic-Google" value="io.dcloud.feature.statistics.google.GoogleStatistics" />
</feature>
```

---

### 相关模块

- [第三方 SDK 依赖说明](third-party-dependencies.md) — 友盟统计版本信息
- [Oauth 登录鉴权](oauth.md) — Google登录也使用google-services插件
