# Push 消息推送（Android）

> **适用版本**：HBuilderX 5.0+
> **平台**：Android
> **官方文档**：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/androidModuleConfig/

## 2. Push（消息推送 / uniPush）

当前版本使用仓储方式集成。

### 需要拷贝的文件

| 路径 | 文件 |
|---|---|
| SDK\libs | `aps-release.aar`, `aps-unipush-release.aar` |

### gradle配置

打开build.gradle，在defaultConfig添加manifestPlaceholders节点：

- 项目根目录下的build.gradle（添加个推仓库地址）：

```groovy
allprojects {
    repositories {
        jcenter()
        google()
        // 个推的Maven仓地址。
        maven { 
            url 'https://mvn.getui.com/nexus/content/repositories/releases' 
        }
    }
}
```

- 项目应用下的build.gradle：

```groovy
android {
    defaultConfig {
        manifestPlaceholders = [
            "GETUI_APPID": "unipush的appid",
            "plus.unipush.appid" : "unipush的appid",
            "plus.unipush.appkey" : "unipush的key",
            "plus.unipush.appsecret": "unipush的secret",
            "apk.applicationId":"io.dcloud.HBuilder",
            // 根据所需厂商选择集成
            "XIAOMI_APP_ID": "",
            "XIAOMI_APP_KEY": "",
            "MEIZU_APP_ID": "",
            "MEIZU_APP_KEY": "",
            "HUAWEI_APP_ID": "",
            "OPPO_APP_KEY": "",
            "OPPO_APP_SECRET": "",
            "VIVO_APP_ID": "",
            "VIVO_APP_KEY": "",
            "HONOR_APP_ID": ""
        ]
    }
}

dependencies {
    implementation('com.getui:gtsdk:3.3.7.0'){ exclude(group: 'com.getui') } //个推SDK
    implementation 'com.getui:gtc-dcloud:3.2.16.7' //个推核心组件
    // 根据所需厂商选择集成
    implementation 'com.getui.opt:hwp:3.1.1' // 华为
    implementation 'com.huawei.hms:push:6.11.0.300' // 华为
    implementation 'com.getui.opt:xmp:3.3.1' // 小米
    implementation 'com.assist-v3:oppo:3.3.0' // oppo
    implementation 'com.google.code.gson:gson:2.6.2' // oppo
    implementation 'commons-codec:commons-codec:1.6' // oppo
    implementation 'androidx.annotation:annotation:1.1.0' // oppo
    implementation 'com.assist-v3:vivo:3.1.1' // vivo
    implementation 'com.getui.opt:mzp:3.2.3' // 魅族
    implementation 'com.getui.opt:honor:3.6.0' // 荣耀
    implementation 'com.hihonor.mcs:push:7.0.61.303' // 荣耀
}
```

> 应用的app id/app key等信息，从开发者后台->unipush->配置管理->应用管理界面查看。

### AndroidManifest.xml配置

在`io.dcloud.PandoraEntry`的Activity标签下追加intent-filter（**注意不能和其他的intent-filter内容合并到一起**）：

```xml
<intent-filter>
    <action android:name="android.intent.action.VIEW"/>
    <category android:name="android.intent.category.DEFAULT" />
    <category android:name="android.intent.category.BROWSABLE" />
    <data android:host="io.dcloud.unipush" android:path="/" android:scheme="unipush" /> 
</intent-filter>
```

### dcloud_properties.xml配置

在properties中添加如下配置，features节点与services节点必须同时配置！

```xml
<properties>
    <features>
        <feature name="Push" value="io.dcloud.feature.aps.APSFeatureImpl">
            <module name="unipush" value="io.dcloud.feature.unipush.GTPushService"/>
        </feature>
    </features>    
    <services>
        <service name="push" value="io.dcloud.feature.aps.APSFeatureImpl"/>
    </services>
</properties>
```

### OPPO推送特殊配置

OPPO集成uniPush时需在AndroidManifest.xml的入口activity中添加：

```xml
<intent-filter>
    <action android:name="android.intent.action.oppopush" />
    <category android:name="android.intent.category.DEFAULT" />
</intent-filter>
```

在app目录下的build.gradle内添加：

```groovy
dependencies {
    implementation 'com.google.code.gson:gson:2.6.2' 
    implementation 'commons-codec:commons-codec:1.6' 
    implementation 'androidx.annotation:annotation:1.1.0'
}
```

### 华为推送特殊配置

**项目根目录下的build.gradle**（添加华为推送仓库地址）：

```groovy
buildscript {
    repositories {
        jcenter()
        google()
        maven {url 'https://developer.huawei.com/repo/'}
    }
    dependencies {
        classpath 'com.huawei.agconnect:agcp:1.9.1.301'
    }
}
allprojects {
    repositories {
        jcenter()
        google()
        maven {url 'https://developer.huawei.com/repo/'}
    }
}
```

**项目应用下的build.gradle**（添加agcp插件和依赖）：

```groovy
apply plugin: 'com.android.application'
apply plugin: 'com.huawei.agconnect'

dependencies {
    implementation 'com.huawei.hms:push:6.11.0.300'
}
```

**添加华为推送的配置文件**：登录华为AppGallery Connect网站，下载`agconnect-services.json`文件到应用级根目录下。

### 荣耀推送

项目根目录下的build.gradle添加荣耀仓库地址：

```groovy
maven { url 'https://developer.hihonor.com/repo/' }
```

---

### 相关模块

- [Oauth 登录鉴权](oauth.md) — 一键登录也依赖个推SDK
- [第三方 SDK 依赖说明](third-party-dependencies.md) — 个推push版本信息
