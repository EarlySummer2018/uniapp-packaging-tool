# DCloud UniApp Android 离线 SDK 模块配置教程

> **适用版本**：HBuilderX 5.0+
> **生成时间**：2026-05-29
> **基于官方文档**：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/androidModuleConfig/

---

## 目录

- [1. Geolocation（定位）](#1-geolocation定位)
  - [百度定位](#百度定位)
  - [高德定位](#高德定位)
  - [系统定位](#系统定位)
  - [腾讯定位](#腾讯定位)
- [2. Push（消息推送）](#2-push消息推送)
- [3. Share（分享）](#3-share分享)
- [4. Oauth（登录鉴权）](#4-oauth登录鉴权)
  - [一键登录](#一键登录)
  - [微信登录](#微信登录)
  - [QQ登录](#qq登录)
  - [新浪微博登录](#新浪微博登录)
  - [小米登录](#小米登录)
  - [Google登录](#google登录)
  - [Facebook登录](#facebook登录)
- [5. Map（地图）](#5-map地图)
  - [百度地图](#百度地图)
  - [高德地图](#高德地图)
  - [谷歌地图](#谷歌地图)
- [6. Payment（支付）](#6-payment支付)
  - [支付宝](#支付宝)
  - [微信支付](#微信支付)
  - [PayPal支付](#paypal支付)
  - [Stripe支付](#stripe支付)
  - [Google支付](#google支付)
- [7. Speech（语音输入）](#7-speech语音输入)
  - [百度语音](#百度语音)
  - [讯飞语音](#讯飞语音)
- [8. Statistic（统计）](#8-statistic统计)
  - [友盟统计](#友盟统计)
  - [友盟统计-google play](#友盟统计-google-play)
  - [谷歌统计](#谷歌统计)
- [9. FacialRecognitionVerify（实人认证）](#9-facialrecognitionverify实人认证)
- [10. uni-AD（广告）](#10-uni-ad广告)
  - [穿山甲](#穿山甲)
  - [腾讯优量汇](#腾讯优量汇)
  - [快手](#快手)
  - [Sigmob](#sigmob)
  - [百度广告](#百度广告)
  - [华为广告](#华为广告)
  - [穿山甲GroMore](#穿山甲gromore)
  - [uniMP激励视频广告](#unimp激励视频广告)
  - [其他广告平台](#其他广告平台)
- [11. Android X5 Webview](#11-android-x5-webview)
- [12. UTS 内置模块](#12-uts-内置模块)
- [13. UTS 基础模块](#13-uts-基础模块)
- [14. 其他模块及国际化配置](#14-其他模块及国际化配置)
- [15. 第三方 SDK 依赖说明](#15-第三方-sdk-依赖说明)
- [Android 注意事项](#android-注意事项)

---

# Android 模块配置

---

## 1. Geolocation（定位）

**离线打包地图模块与定位模块可以分别配置，不需要单独依赖地图模块。**

### 百度定位

#### 需要拷贝的文件

| 路径 | 文件 |
|---|---|
| SDK\libs | `baidu-libs-release.aar`, `geolocation-baidu-release.aar` |

#### application节点下配置

```xml
<meta-data android:name="com.baidu.lbsapi.API_KEY" android:value="%appkey_android%"></meta-data>
<service android:name="com.baidu.location.f" android:enabled="true" android:process=":remote"></service>
```

---

### 高德定位

#### 需要拷贝的文件

| 路径 | 文件 |
|---|---|
| SDK\libs | `geolocation-amap-release.aar` |

高德 SDK 通过 gradle 集成。

#### 通过gradle集成高德定位SDK

```groovy
android {
    xxxxxxxx
    defaultConfig {
        xxxxxxxx
    }
}
dependencies {
    xxxxxxxx
    implementation('com.amap.api:location:6.4.5')
}
```

**注意事项：**
- 版本号通过离线SDK中的demo获取相对应版本
- 本地集成的高德定位SDK需要删除相关aar文件，否则会导致sdk冲突
- 高德定位与高德地图SDK集成冲突，需要注意如果集成地图无须再配置定位

#### AndroidManifest.xml文件需要修改的项

**需要在application节点前添加权限**

```xml
<uses-permission android:name="android.permission.ACCESS_COARSE_LOCATION"/>
<uses-permission android:name="android.permission.ACCESS_FINE_LOCATION"/>
<uses-permission android:name="android.permission.ACCESS_WIFI_STATE"/>
<uses-permission android:name="android.permission.ACCESS_NETWORK_STATE"/>
<uses-permission android:name="android.permission.CHANGE_WIFI_STATE"/>
<uses-permission android:name="android.permission.READ_PHONE_STATE"/>
<uses-permission android:name="android.permission.WRITE_EXTERNAL_STORAGE"/>
<uses-permission android:name="android.permission.INTERNET"/>
<uses-permission android:name="android.permission.MOUNT_UNMOUNT_FILESYSTEMS"/>
<uses-permission android:name="android.permission.READ_LOGS"/>
<uses-permission android:name="android.permission.WRITE_SETTINGS"/>
<uses-permission android:name="android.permission.ACCESS_BACKGROUND_LOCATION"/>
<uses-permission android:name="android.permission.FOREGROUND_SERVICE"/>
```

**application节点下配置**

```xml
<meta-data android:name="com.amap.api.v2.apikey" android:value="%用户申请的APPkey%"></meta-data>
<service android:name="com.amap.api.location.APSService"></service>
```

---

### 系统定位

#### 需要拷贝的文件

**最新SDK使用系统定位已不需要引入任何文件**

#### AndroidManifest.xml文件需要修改的项

**需要在application节点前添加权限**

```xml
<uses-permission android:name="android.permission.ACCESS_COARSE_LOCATION"/>
<uses-permission android:name="android.permission.ACCESS_FINE_LOCATION"/>
<uses-permission android:name="android.permission.ACCESS_WIFI_STATE"/>
<uses-permission android:name="android.permission.ACCESS_NETWORK_STATE"/>
<uses-permission android:name="android.permission.CHANGE_WIFI_STATE"/>
<uses-permission android:name="android.permission.READ_PHONE_STATE"/>
<uses-permission android:name="android.permission.WRITE_EXTERNAL_STORAGE"/>
<uses-permission android:name="android.permission.INTERNET"/>
<uses-permission android:name="android.permission.MOUNT_UNMOUNT_FILESYSTEMS"/>
<uses-permission android:name="android.permission.READ_LOGS"/>
<uses-permission android:name="android.permission.WRITE_SETTINGS"/>
```

---

### 腾讯定位

腾讯定位依赖于UTS基础模块，请先集成[UTS基础模块](#13-uts-基础模块)。

#### 需要拷贝的文件

| 路径 | 文件 |
|---|---|
| SDK\libs | `uni-getLocation-tencent-uni1-release.aar` |

#### 通过gradle集成腾讯定位SDK

```groovy
android {
    xxxxxxxx
    defaultConfig {
        xxxxxxxx
    }
}
dependencies {
    xxxxxxxx
    implementation('com.tencent.map.geolocation:TencentLocationSdk-openplatform:xxx')
}
```

> **注意**：xxx是腾讯定位版本号

**application节点下配置**

```xml
<meta-data android:name="TencentMapSDK" android:value="您申请的腾讯定位App Key"/>
```

---

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

## 3. Share（分享）

> **适用版本**：HBuilderX 5.0+

### 微信分享

#### 需要拷贝的文件

| 路径 | 文件 |
|---|---|
| SDK\libs | `share-weixin-release.aar` |

> **注意**：微信SDK通过gradle集成，无需手动添加wechat-sdk aar文件。

#### 通过gradle集成微信SDK

```groovy
dependencies {
    implementation 'com.tencent.mm.opensdk:wechat-sdk-android-without-mta:6.8.0'
}
```

#### AndroidManifest.xml配置

**权限：**
```xml
<uses-permission android:name="android.permission.MODIFY_AUDIO_SETTINGS"/>
```

**application节点下：**
```xml
<!-- 微信分享 配置begin -->
<meta-data android:name="WX_APPID" android:value="%微信开放平台申请应用的AppID%"/>
<meta-data 
    android:name="WX_SECRET" 
    android:value="%微信开放平台申请应用的Secret%">
</meta-data>
<activity
    android:name="[包名].wxapi.WXEntryActivity"
    android:label="@string/app_name"
    android:exported="true"
    android:launchMode="singleTop">
    <intent-filter>
        <action android:name="android.intent.action.VIEW"/>
        <category android:name="android.intent.category.DEFAULT"/>
        <data android:scheme="%微信开放平台申请应用的AppID%"/>
    </intent-filter>
</activity>
<!-- 微信分享 配置end -->
```

> **重要提示**：
> - `AndroidManifest.xml`文件中声明的包名必须与申请微信AppID使用的包名一致
> - 微信分享测试需要使用在微信开放平台申请应用时使用的应用签名文件进行签名打包

#### dcloud_properties.xml配置

```xml
<feature name="Share" value="io.dcloud.share.ShareFeatureImpl">
    <module name="Weixin"/>
</feature>
```

---

### QQ分享

#### 需要拷贝的文件

| 路径 | 文件 |
|---|---|
| SDK\libs | `share-qq-release.aar`, `open_sdk_3.5.12.r2_j97423a8_lite.jar` |

#### AndroidManifest.xml配置

**权限：**
```xml
<uses-permission android:name="android.permission.MODIFY_AUDIO_SETTINGS"/>
```

**application节点下：**
```xml
<!-- Share QQ start -->
<meta-data android:value="%appId%" android:name="QQ_APPID"/>
<activity 
    android:name="com.tencent.tauth.AuthActivity" 
    android:launchMode="singleTask"
    android:noHistory="true">
    <intent-filter>
        <action android:name="android.intent.action.VIEW"/>
        <category android:name="android.intent.category.DEFAULT"/>
        <category android:name="android.intent.category.BROWSABLE"/>
        <data android:scheme="%appId%"/>
    </intent-filter>
</activity>
<activity 
    android:name="com.tencent.connect.common.AssistActivity" 
    android:theme="@android:style/Theme.Translucent.NoTitleBar"
    android:configChanges="keyboardHidden|orientation"
    android:screenOrientation="behind">
</activity>
<!-- Share QQ end -->
```

#### dcloud_properties.xml配置

```xml
<feature name="Share" value="io.dcloud.share.ShareFeatureImpl">
    <module name="QQ"/>
</feature>
```

---

### 新浪微博分享

#### 需要拷贝的文件

| 路径 | 文件 |
|---|---|
| SDK\libs | `openDefault-12.5.0.aar`, `share-sina-release.aar` |

#### AndroidManifest.xml配置

**权限：**
```xml
<uses-permission android:name="android.permission.CHANGE_WIFI_STATE"/>
<uses-permission android:name="android.permission.ACCESS_WIFI_STATE"/>
<uses-permission android:name="android.permission.ACCESS_NETWORK_STATE"/>
```

**application节点下：**
```xml
<!-- Share - 新浪微博分享 -->
<meta-data android:name="SINA_APPKEY" android:value="%新浪微博开放平台申请应用的AppKey%"/>
<meta-data android:name="SINA_SECRET" android:value="%新浪微博开放平台申请应用的Secret%"/>
<meta-data android:name="SINA_REDIRECT_URI" android:value="%新浪微博开放平台申请应用的RedirectUrl%"/>
<activity 
    android:name="com.sina.weibo.sdk.web.WeiboSdkWebActivity"
    android:configChanges="keyboardHidden|orientation"
    android:exported="false"
    android:windowSoftInputMode="adjustResize">
</activity>
<activity 
    android:name="com.sina.weibo.sdk.share.WbShareTransActivity"
    android:launchMode="singleTask"
    android:theme="@android:style/Theme.Translucent.NoTitleBar.Fullscreen">
    <intent-filter>
        <action android:name="com.sina.weibo.sdk.action.ACTION_SDK_REQ_ACTIVITY"/>
        <category android:name="android.intent.category.DEFAULT"/>
    </intent-filter>
</activity>
<!-- Share - 新浪微博分享 end -->
```

> **提示**：`AndroidManifest.xml`文件中声明的包名必须与申请新浪微博AppKey使用的包名一致

#### dcloud_properties.xml配置

```xml
<feature name="Share" value="io.dcloud.share.ShareFeatureImpl">
    <module name="Sina"/>
</feature>
```

---

## 4. Oauth（登录鉴权）

### 一键登录

#### 需要拷贝的文件

| 路径 | 文件 |
|---|---|
| SDK\libs | `oauth-univerify-release.aar` |

#### build.gradle配置

先在项目根目录build.gradle添加个推仓库源：

```groovy
allprojects {
    repositories {
        jcenter()
        google()
        maven { url 'https://mvn.getui.com/nexus/content/repositories/releases' }
    }
}
```

然后app的build.gradle中配置：

```groovy
android {
    defaultConfig {
        manifestPlaceholders = [
            "GETUI_APPID"     : "%GETUI_APPID%",
            "GY_APP_ID"       : "%GY_APP_ID%",
            "GT_INSTALL_CHANNEL":"HBuilder",
        ]
    }
}

dependencies {
    implementation 'com.getui:gtc-dcloud:3.2.16.7'
    implementation('com.getui:gysdk:3.1.7.0') { exclude(group: 'com.getui') }
}
```

> GT_INSTALL_CHANNEL 固定值 "HBuilder"。GETUI_APPID与GY_APP_ID取值相同，对应[开发者中心](https://dev.dcloud.net.cn/)一键登录->基础配置->一键登录应用ID。

#### dcloud_properties.xml配置

```xml
<feature name="OAuth" value="io.dcloud.feature.oauth.OAuthFeatureImpl">
    <module name="OAuth-IGETui" value="io.dcloud.feature.igetui.GeTuiOAuthService"/>
</feature>
```

---

### 微信登录

#### 需要拷贝的文件

| 路径 | 文件 |
|---|---|
| SDK\libs | `oauth-weixin-release.aar` |

微信 SDK 通过 gradle 集成。

#### 通过gradle集成微信SDK

```groovy
dependencies {
    implementation 'com.tencent.mm.opensdk:wechat-sdk-android-without-mta:6.8.0'
}
```

需要将`WXEntryActivity.java`引入到工程（位于离线sdk的/SDK/src/wxapi下），包名为`%应用包名%.wxapi`。

#### AndroidManifest.xml配置

**权限：**
```xml
<uses-permission android:name="android.permission.MODIFY_AUDIO_SETTINGS"/>
```

**application节点下：**
```xml
<meta-data android:value="%用户申请的微信Appcert%" android:name="WX_SECRET"/> 
<meta-data android:value="%用户申请的微信Appid%" android:name="WX_APPID"/>  
<activity android:name="%用户包名%.wxapi.WXEntryActivity" 
    android:label="@string/app_name"  
    android:exported="true" 
    android:launchMode="singleTop"> 
    <intent-filter>
        <action android:name="android.intent.action.VIEW"/>
        <category android:name="android.intent.category.DEFAULT"/> 
        <data android:scheme="%用户申请的微信Appid%"/>
    </intent-filter> 
</activity>
```

#### dcloud_properties.xml配置

```xml
<feature name="OAuth" value="io.dcloud.feature.oauth.OAuthFeatureImpl">
    <module name="OAuth-Weixin" value="io.dcloud.feature.oauth.weixin.WeiXinOAuthService"/>
</feature>
```

---

### QQ登录

#### 需要拷贝的文件

| 路径 | 文件 |
|---|---|
| SDK\libs | `oauth-qq-release.aar`, `open_sdk_3.5.12.2_r97423a8_lite.jar` |

#### AndroidManifest.xml配置

**权限：**
```xml
<uses-permission android:name="android.permission.MODIFY_AUDIO_SETTINGS"/>
```

**application节点下：**
```xml
<!-- Oauth QQ start -->
<meta-data android:value="%appid%" android:name="QQ_APPID"/> 
<activity android:name="com.tencent.tauth.AuthActivity" android:launchMode="singleTask" android:noHistory="true"> 
    <intent-filter>
        <action android:name="android.intent.action.VIEW"/> 
        <category android:name="android.intent.category.DEFAULT"/> 
        <category android:name="android.intent.category.BROWSABLE"/>
        <data android:scheme="%appid%"/> 
    </intent-filter> 
</activity> 
<activity android:name="com.tencent.connect.common.AssistActivity" android:theme="@android:style/Theme.Translucent.NoTitleBar" android:screenOrientation="portrait"/>
<!-- Oauth QQ end -->
```

#### dcloud_properties.xml配置

```xml
<feature name="OAuth" value="io.dcloud.feature.oauth.OAuthFeatureImpl">
    <module name="OAuth-QQ" value="io.dcloud.feature.oauth.qq.QQOAuthService"/>
</feature>
```

---

### 新浪微博登录

#### 需要拷贝的文件

| 路径 | 文件 |
|---|---|
| SDK\libs | `openDefault-12.5.0.aar`, `oauth-sina-release.aar` |

#### AndroidManifest.xml配置

```xml
<!-- Oauth Sina start -->
<meta-data android:value="%redirect_uri%" android:name="SINA_REDIRECT_URI"/> 
<meta-data android:value="_%appkey%" android:name="SINA_APPKEY"/> 
<activity android:name="com.sina.weibo.sdk.web.WeiboSdkWebActivity"
    android:configChanges="keyboardHidden|orientation"
    android:exported="false"
    android:windowSoftInputMode="adjustResize">
</activity>
<!-- Oauth Sina end -->
```

#### dcloud_properties.xml配置

```xml
<feature name="OAuth" value="io.dcloud.feature.oauth.OAuthFeatureImpl">
    <module name="OAuth-Sina" value="io.dcloud.feature.oauth.sina.SinaOAuthService"/>
</feature>
```

---

### 小米登录

#### 需要拷贝的文件

| 路径 | 文件 |
|---|---|
| SDK\libs | `oauth-miui-release.aar` |

#### AndroidManifest.xml配置

**权限：**
```xml
<uses-permission android:name="com.xiaomi.permission.AUTH_SERVICE"/>
```

**application节点下：**
```xml
<meta-data android:value="_%小米登陆的APPID%" android:name="MIUI_APPID"/>
<meta-data android:value="%小米登陆的APPSecret%" android:name="MIUI_APPSECRET"/>
<meta-data android:value="%小米登陆的RegURL%" android:name="MIUI_REDIRECT_URI"/>
<activity android:name="com.xiaomi.account.openauth.AuthorizeActivity"/>
```

#### dcloud_properties.xml配置

```xml
<feature name="OAuth" value="io.dcloud.feature.oauth.OAuthFeatureImpl">
    <module name="OAuth-MiUi" value="io.dcloud.feature.oauth.miui.MiUiOAuthService"/>
</feature>
```

---

### Google登录

#### 需要拷贝的文件

| 路径 | 文件 |
|---|---|
| SDK\libs | `oauth-google-release.aar` |

#### gradle配置

**project级build.gradle：**
```groovy
buildscript {
    repositories {
        google()
    }
    dependencies {
        ...
        classpath 'com.google.gms:google-services:4.2.0'
    }
}
```

**app级build.gradle：**
```groovy
dependencies {
    implementation 'com.google.android.gms:play-services-auth:19.2.0'
}
```

#### dcloud_properties.xml配置

```xml
<feature name="OAuth" value="io.dcloud.feature.oauth.OAuthFeatureImpl">
    <module name="OAuth-Google" value="io.dcloud.feature.google.GoogleOAuthService"/>
</feature>
```

---

### Facebook登录

#### 需要拷贝的文件

| 路径 | 文件 |
|---|---|
| SDK\libs | `oauth-facebook-release.aar` |

#### res/values/strings.xml 配置

```xml
<string name="facebook_app_id">xxxxxxxxxxxxxxxx</string>
<string name="fb_login_protocol_scheme">fbxxxxxxxxxxxxxxxx</string>
<string name="facebook_client_token">xxxxxxxxxxxxxxx</string>
```

#### gradle配置

```groovy
dependencies {
    implementation 'com.facebook.android:facebook-login:17.0.2'
}
```

> **注意事项**：Android端在4.31版本后Facebook登录SDK默认携带`com.google.android.gms.permission.AD_ID`权限，如未使用广告相关功能在GooglePlay上架时会遇到审核问题，需要手动删除此权限。

#### dcloud_properties.xml配置

```xml
<feature name="OAuth" value="io.dcloud.feature.oauth.OAuthFeatureImpl">
    <module name="OAuth-Facebook" value="io.dcloud.feature.facebook.FacebookOAuthService"/>
</feature>
```

---

## 5. Map（地图）

> **开发者需要修改使用的地图插件时，需要修改dcloud_properties.xml文件的features节点下Maps节点value属性的配置，高德地图和百度地图的配置只能保留一个。**

### 百度地图

#### 需要拷贝的文件

| 路径 | 文件 |
|---|---|
| SDK\libs | `baidu-libs-release.aar`, `map-baidu-release.aar` |

> 百度地图暂时不支持 nvue map 标签

#### AndroidManifest.xml配置

**权限：**
```xml
<uses-permission android:name="android.permission.ACCESS_COARSE_LOCATION"/>
<uses-permission android:name="android.permission.ACCESS_FINE_LOCATION"/>
<uses-permission android:name="android.permission.ACCESS_WIFI_STATE"/>
<uses-permission android:name="android.permission.ACCESS_NETWORK_STATE"/>
<uses-permission android:name="android.permission.CHANGE_WIFI_STATE"/>
<uses-permission android:name="android.permission.READ_PHONE_STATE"/>
<uses-permission android:name="android.permission.WRITE_EXTERNAL_STORAGE"/>
<uses-permission android:name="android.permission.INTERNET"/>
<uses-permission android:name="android.permission.MOUNT_UNMOUNT_FILESYSTEMS"/>
<uses-permission android:name="android.permission.READ_LOGS"/>
<uses-permission android:name="android.permission.WRITE_SETTINGS"/>
```

**application节点下：**
```xml
<meta-data android:name="com.baidu.lbsapi.API_KEY" android:value="%appkey_android%"></meta-data>
<service android:name="com.baidu.location.f" android:enabled="true" android:process=":remote"></service>
```

#### dcloud_properties.xml配置

**features节点：**
```xml
<feature name="Maps" value="io.dcloud.js.map.JsMapPluginImpl"></feature>
```
**services节点：**
```xml
<service name="Maps" value="io.dcloud.js.map.MapInitImpl" />
```

---

### 高德地图

#### 需要拷贝的文件

| 页面类型 | 路径 | 文件 |
|---|---|---|
| nvue页面 | SDK\libs | `weex_amap-release.aar` |
| vue页面 | SDK\libs | `map-amap-release.aar` |

高德 SDK 通过 gradle 集成。

#### 通过gradle集成高德地图SDK

```groovy
dependencies {
    implementation('com.amap.api:3dmap:xxx')
    implementation('com.amap.api:search:xxx')
}
```

> xxx是版本号，通过离线SDK中的demo获取。本地集成的需删除相关aar文件否则冲突。高德定位与高德地图SDK集成冲突，集成地图无须再配定位。

#### AndroidManifest.xml配置

**权限同百度地图（不含最后两项额外权限）。**

**application节点下：**
```xml
<meta-data android:name="com.amap.api.v2.apikey" android:value="%appkey_android%"/>
<service android:name="com.amap.api.location.APSService"></service>
```

> 高德地图使用的appkey与包名及签名文件存在对应关系，填写错误会导致地图无法正常使用。

#### dcloud_properties.xml配置

```xml
<feature name="Maps" value="io.dcloud.js.map.amap.JsMapPluginImpl"></feature>
```

> 高德地图不需要添加services节点。

---

### 谷歌地图

#### 需要拷贝的文件

| 路径 | 文件 |
|---|---|
| SDK\libs | `weex_google-map-release.aar` |

> 谷歌地图仅支持 nvue map 标签

#### app目录build.gradle添加依赖

```groovy
implementation 'com.google.android.gms:play-services-maps:18.0.1'
```

#### AndroidManifest.xml配置

**权限：**
```xml
<uses-permission android:name="android.permission.ACCESS_COARSE_LOCATION" />
<uses-permission android:name="android.permission.ACCESS_FINE_LOCATION" />
<uses-permission android:name="android.permission.ACCESS_NETWORK_STATE" />
<uses-permission android:name="android.permission.INTERNET" />
<uses-permission android:name="android.permission.WRITE_EXTERNAL_STORAGE" />
<uses-permission android:name="android.permission.ACCESS_LOCATION_EXTRA_COMMANDS" />
```

**application节点下：**
```xml
<meta-data android:name="com.google.android.geo.API_KEY" android:value="%api_key%" />
```

> api_key在[谷歌开发者](https://mapsplatform.google.com/)开通。谷歌地图不需要修改dcloud_properties.xml文件。

---

## 6. Payment（支付）

### 支付宝

#### 需要拷贝的文件

| 路径 | 文件 |
|---|---|
| SDK\libs | `payment-alipay-release.aar` |

#### 通过gradle集成支付宝SDK

```groovy
dependencies {
    implementation 'com.alipay.sdk:alipaysdk-android:15.8.11'
}
```

#### AndroidManifest.xml配置

**权限：**
```xml
<uses-permission android:name="android.permission.INTERNET" />
<uses-permission android:name="android.permission.ACCESS_NETWORK_STATE" />
<uses-permission android:name="android.permission.ACCESS_WIFI_STATE" />
<uses-permission android:name="android.permission.READ_PHONE_STATE" />
<uses-permission android:name="android.permission.WRITE_EXTERNAL_STORAGE" />
<uses-permission android:name="android.permission.ACCESS_COARSE_LOCATION" />
```

#### dcloud_properties.xml配置

```xml
<feature name="Payment" value="io.dcloud.feature.payment.PaymentFeatureImpl">
    <module name="AliPay" value="io.dcloud.feature.payment.alipay.AliPay"/>
</feature>
```

---

### 微信支付

#### 需要拷贝的文件

| 路径 | 文件 |
|---|---|
| SDK\libs | `payment-weixin-release.aar` |

微信 SDK 通过 gradle 集成。

#### 通过gradle集成微信SDK

```groovy
dependencies {
    implementation 'com.tencent.mm.opensdk:wechat-sdk-android-without-mta:6.8.0'
}
```

需将`WXPayEntryActivity.java`引入工程（位于SDK/src/wxapi），包名为`$你的包名.wxapi`。

#### AndroidManifest.xml配置

**权限：**
```xml
<uses-permission android:name="android.permission.MODIFY_AUDIO_SETTINGS"/>
```

**application节点下：**
```xml
<meta-data android:name="WX_APPID" android:value="$微信APPID" />
<activity android:name="io.dcloud.feature.payment.weixin.WXPayProcessMeadiatorActivity"
    android:exported="false"
    android:excludeFromRecents="true"
    android:theme="@style/ProjectDialogTheme" />
<activity android:name="$你的包名.wxapi.WXPayEntryActivity"
    android:exported="true"
    android:theme="@android:style/Theme.Translucent.NoTitleBar"
    android:launchMode="singleTop" />
```

#### dcloud_properties.xml配置

```xml
<feature name="Payment" value="io.dcloud.feature.payment.PaymentFeatureImpl">
    <module name="Payment-Weixin" value="io.dcloud.feature.payment.weixin.WeiXinPay"/>
</feature>
```

---

### PayPal支付

#### Gradle配置

**project级build.gradle设置PayPal私有库：**
```groovy
allprojects {
    repositories {
        maven {
            url "https://cardinalcommerceprod.jfrog.io/artifactory/android"
            credentials {
                username 'paypal_sgerritz'
                password '<YOUR_PAYPAL_JFROG_API_KEY>'
            }
        }
    }
}
```

**app级build.gradle：**
```groovy
dependencies {
    implementation('com.paypal.checkout:android-sdk:0.6.2')
}
```

#### AndroidManifest.xml配置

**权限：**
```xml
<uses-permission android:name="android.permission.INTERNET" />
```

**application节点内配置（YOUR-CUSTOM-SCHEME替换为自定义scheme）：**
```xml
<activity android:name="com.paypal.openid.RedirectUriReceiverActivity"
    android:excludeFromRecents="true" android:exported="true" android:theme="@style/PYPLAppTheme">
    <intent-filter>
        <action android:name="android.intent.action.VIEW" />
        <category android:name="android.intent.category.DEFAULT" />
        <category android:name="android.intent.category.BROWSABLE" />
        <data android:host="paypalpay" android:scheme="%YOUR-CUSTOM-SCHEME%" />
    </intent-filter>
</activity>

<activity android:name="com.paypal.pyplcheckout.home.view.activities.PYPLInitiateCheckoutActivity"
    android:exported="true" android:theme="@style/AppFullScreenTheme">
    <intent-filter android:autoVerify="true">
        <action android:name="android.intent.action.VIEW" />
        <category android:name="android.intent.category.DEFAULT" />
        <category android:name="android.intent.category.BROWSABLE" />
        <data android:host="paypalxo" android:scheme="%YOUR-CUSTOM-SCHEME%" />
    </intent-filter>
</activity>

<meta-data android:name="returnUrl" android:value="%YOUR-CUSTOM-SCHEME%://paypalpay"/>
```

#### 需要拷贝的文件 & dcloud_properties.xml

| 路径 | 文件 |
|---|---|
| SDK\libs | `payment-paypal-release.aar` |

```xml
<feature name="Payment" value="io.dcloud.feature.payment.PaymentFeatureImpl">
    <module name="Payment-Paypal" value="io.dcloud.feature.payment.paypal.PaypalPay" />
</feature>
```

---

### Stripe支付

#### Gradle配置

```groovy
android {
    defaultConfig {
        minSdkVersion 21
    }
}

dependencies {
    implementation "androidx.appcompat:appcompat:${rootProject.ext.androidxVersion}"
    implementation "androidx.legacy:legacy-support-v4:${rootProject.ext.androidxVersion}"
    implementation 'com.stripe:stripe-android:18.2.0'
}
```

#### AndroidManifest.xml配置

```xml
<activity android:name="io.dcloud.feature.payment.stripe.TransparentActivity"
    android:excludeFromRecents="true" android:exported="false" android:theme="@style/TranslucentTheme" />
```

#### 需要拷贝的文件 & dcloud_properties.xml

| 路径 | 文件 |
|---|---|
| SDK\libs | `payment-stripe-release.aar` |

```xml
<feature name="Payment" value="io.dcloud.feature.payment.PaymentFeatureImpl">
    <module name="Payment-Stripe" value="io.dcloud.feature.payment.stripe.StripePay"/>
</feature>
```

---

### Google支付

#### Gradle配置

```groovy
dependencies {
    implementation "androidx.appcompat:appcompat:${rootProject.ext.androidxVersion}"
    implementation 'com.google.android.gms:play-services-wallet:18.1.3'
}
```

#### AndroidManifest.xml配置

```xml
<meta-data android:name="com.google.android.gms.wallet.api.enabled" android:value="true" />
```

#### 需要拷贝的文件 & dcloud_properties.xml

| 路径 | 文件 |
|---|---|
| SDK\libs | `payment-google-release.aar` |

```xml
<feature name="Payment" value="io.dcloud.feature.payment.PaymentFeatureImpl">
    <module name="Payment-Google" value="io.dcloud.feature.payment.google.GooglePay"/>
</feature>
```

---

## 7. Speech（语音输入）

### 百度语音

#### 需要添加的文件

| 路径 | 文件名 |
|---|---|
| SDK\libs | `speech-release.aar`, `speech_baidu-release.aar` |

#### AndroidManifest.xml配置

**权限：**
```xml
<uses-permission android:name="android.permission.RECORD_AUDIO" />
<uses-permission android:name="android.permission.INTERNET" />
<uses-permission android:name="android.permission.ACCESS_NETWORK_STATE" />
<uses-permission android:name="android.permission.ACCESS_WIFI_STATE" />
<uses-permission android:name="android.permission.CHANGE_NETWORK_STATE" />
<uses-permission android:name="android.permission.READ_PHONE_STATE" />
<uses-permission android:name="android.permission.WRITE_EXTERNAL_STORAGE" />
```

**application节点下：**
```xml
<meta-data android:name="com.baidu.speech.APP_ID" android:value="${百度语音申请的appid}"/>
<meta-data android:name="com.baidu.speech.API_KEY" android:value="${百度语音申请的apikey}"/>
<meta-data android:name="com.baidu.speech.SECRET_KEY" android:value="${百度语音申请的secretkey}"/>
<service android:name="com.baidu.speech.VoiceRecognitionService" android:exported="false" />
```

#### dcloud_properties.xml配置

```xml
<feature name="Speech" value="io.dcloud.feature.speech.SpeechFeatureImpl">
    <module name="baidu" value="io.dcloud.feature.speech.BaiduSpeechEngine"/>
</feature>
```

---

### 讯飞语音

#### 需要添加的文件

| 路径 | 文件名 |
|---|---|
| SDK\libs | `speech-release.aar`, `speech_ifly-release.aar` |

#### AndroidManifest.xml配置

**权限同百度语音。**

**application节点下：**
```xml
<meta-data android:name="IFLY_APPKEY" android:value="${讯飞语音申请的appid}" />
```

#### dcloud_properties.xml配置

```xml
<feature name="Speech" value="io.dcloud.feature.speech.SpeechFeatureImpl">
    <module name="iFly" value="io.dcloud.feature.speech.IflySpeechEngine"/>
</feature>
```

---

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

## 9. FacialRecognitionVerify（实人认证）

> **适用版本**：HBuilderX 5.0+
> 
> **前置依赖**：实人认证依赖于 UTS 基础模块，请先集成 [UTS 基础模块](#13-uts-基础模块)

### 需要拷贝的文件

**需要引入工程的 aar 文件**（放到工程的 `libs` 目录下）：

| 路径 | 文件列表 |
|---|---|
| **SDK\libs** | `uni-facialRecognitionVerify-release.aar`<br>`aliyun-base-XXX.aar`<br>`aliyun-facade-XXX.aar`<br>`aliyun-face-XXX.aar`<br>`aliyun-faceaudio-XXX.aar`<br>`aliyun-facelanguage-XXX.aar`<br>`aliyun-photoinus-XXX.aar`<br>`aliyun-wishverify-XXX.aar`<br>`Android-7.0.1.20230914.jiagu.ar`<br>`10042.aar`<br>`APSecuritySDK-DeepSec...aar`<br>`facialRecognitionVerify-support-release.aar` |

> **注意**：`XXX` 为版本号，具体版本号以下载的 SDK 中的为准
>
> **X86 设备支持说明**：HBuilderX 新增了 `facialRecognitionVerify-support-release.aar` 库，作用是应用可以在 X86 设备上正常运行，但调用 `uni.startFacialRecognitionVerify()` 会触发错误回调。如果不支持 X86 设备，可以不用引入。

### app 级 build.gradle 配置

```groovy
dependencies {
    implementation "com.squareup.okhttp3:okhttp:3.11.0"
    implementation "com.squareup.okio:okio:1.14.0"
    implementation "Com.aliyun.dpa:oss-android-sdk:+"
}
```

### 错误处理配置

**问题**：离线 SDK 集成实人认证如果出现 `lib/*/libc++_shared.so` 报错时

**解决方案**：需要在 module 的 `build.gradle` 的 android 节点下添加如下内容：

```groovy
android {
    packagingOptions {
        pickFirst 'lib/*/libc++_shared.so'
    }
}
```

### API 调用示例

实人认证功能通过 uni-app API 调用：

```javascript
// 启动实人认证
uni.startFacialRecognitionVerify({
    success: (res) => {
        console.log('认证成功', res)
    },
    fail: (err) => {
        console.log('认证失败', err)
    }
})

// 获取实人认证 MetaInfo（用于服务端校验）
const metaInfo = uni.getFacialRecognitionMetaInfo()
console.log('MetaInfo:', metaInfo)
```

---

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

## 11. Android X5 Webview

| 适用场景 | 路径 | 文件名 |
|---|---|---|
| 5+ APP | SDK/libs | `webview-x5-release.aar` |
| uni-app项目 | SDK/libs | `webview-x5-release.aar`, `weex_webview-x5-release.aar` |

> X5不需要单独添加配置，直接拷贝上述文件到libs下即可。

**Tips**：NDK配置时请去除x86、64位cpu的配置，建议仅配置"armeabi-v7a"，否则可能无法正常使用X5内核。

详细说明参考：[DCloud App集成 X5 内核（腾讯浏览服务TBS）说明](https://ask.dcloud.net.cn/article/36806)

---

## 12. UTS 内置模块（utsPlugins）

> **UTS插件依赖于UTS基础模块，请先集成[UTS基础模块](#13-uts-基础模块)**

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

## 14. 其他模块及国际化配置

### VideoPlayer（视频播放）

| 路径 | 文件 |
|---|---|
| SDK/libs | `media-release.aar`, `weex_videoplayer-release.aar` |

**dcloud_properties.xml：**
```xml
<feature name="VideoPlayer" value="io.dcloud.media.MediaFeatureImpl"/>
```

---

### LivePusher（直播推流）

| 路径 | 文件 |
|---|---|
| SDK/libs | `weex_livepusher-release.aar` |

**AndroidManifest.xml权限：**
```xml
<uses-permission android:name="android.permission.INTERNET" />
<uses-permission android:name="android.permission.ACCESS_NETWORK_STATE" />
<uses-permission android:name="android.permission.ACCESS_WIFI_STATE" />
<uses-permission android:name="android.permission.WRITE_EXTERNAL_STORAGE" />
<uses-permission android:name="android.permission.READ_EXTERNAL_STORAGE" />
<uses-permission android:name="android.permission.RECORD_AUDIO" />
<uses-permission android:name="android.permission.MODIFY_AUDIO_SETTINGS" />
<uses-permission android:name="android.permission.BLUETOOTH" />
<uses-permission android:name="android.permission.CAMERA" />
<uses-permission android:name="android.permission.READ_PHONE_STATE" />
<uses-feature android:name="android.hardware.Camera"/>
<uses-feature android:name="android.hardware.camera.autofocus" />
```

**dcloud_properties.xml：**
```xml
<feature name="LivePusher" value="io.dcloud.media.live.LiveMediaFeatureImpl"/>
```

---

### Barcode（扫码）

**AndroidManifest.xml权限：**
```xml
<uses-permission android:name="android.permission.CAMERA" />
<uses-feature android:name="android.hardware.camera"/>
<uses-feature android:name="android.hardware.camera.autofocus"/>
<uses-permission android:name="android.permission.VIBRATE"/>
<uses-permission android:name="android.permission.FLASHLIGHT"/>
```

**dcloud_properties.xml：**
```xml
<feature name="Barcode" value="io.dcloud.feature.barcode2.BarcodeFeatureImpl"/>
```

---

### Bluetooth（低功耗蓝牙）

| 路径 | 文件 |
|---|---|
| SDK/libs | `Bluetooth-release.aar` |

**AndroidManifest.xml权限：**
```xml
<uses-permission android:name="android.permission.ACCESS_COARSE_LOCATION" />
<uses-permission android:name="android.permission.ACCESS_FINE_LOCATION" />
<uses-permission android:name="android.permission.BLUETOOTH_ADMIN" />
<uses-permission android:name="android.permission.BLUETOOTH" />
```

> targetSdkVersion 31及以上需追加：
> ```xml
> <uses-permission android:name="android.permission.BLUETOOTH_SCAN" />
> <uses-permission android:name="android.permission.BLUETOOTH_CONNECT" />
> ```

**dcloud_properties.xml：**
```xml
<feature name="Bluetooth" value="io.dcloud.feature.bluetooth.BluetoothFeature"/>
```

---

### Camera（相机/相册）

**权限：**
```xml
<uses-permission android:name="android.permission.CAMERA" />
```

**dcloud_properties.xml：**
```xml
<feature name="Camera" value="io.dcloud.js.camera.CameraFeatureImpl"/>
```

---

### iBeacon

| 路径 | 文件 |
|---|---|
| SDK/libs | `iBeacon-release.aar` |

**权限同Bluetooth模块（含targetSdkVersion 31+追加权限）。**

**dcloud_properties.xml：**
```xml
<feature name="iBeacon" value="io.dcloud.feature.iBeacon.WxBluetoothFeatureImpl"/>
```

---

### Contact（通讯录）

| 路径 | 文件 |
|---|---|
| SDK/libs | `contacts-release.aar` |

**权限：**
```xml
<uses-permission android:name="android.permission.GET_ACCOUNTS"/>
<uses-permission android:name="android.permission.WRITE_CONTACTS"/>
<uses-permission android:name="android.permission.READ_CONTACTS"/>
```

**dcloud_properties.xml：**
```xml
<feature name="Contacts" value="io.dcloud.feature.contacts.ContactsFeatureImpl"></feature>
```

---

### Fingerprint（指纹识别）

| 路径 | 文件 |
|---|---|
| SDK/libs | `fingerprint-release.aar` |

**权限：**
```xml
<uses-permission android:name="android.permission.USE_FINGERPRINT"/>
```

**dcloud_properties.xml：**
```xml
<feature name="Fingerprint" value="io.dcloud.feature.fingerprint.FingerPrintsImpl"/>
```

---

### Messaging（短彩邮件消息）

| 路径 | 文件 |
|---|---|
| SDK/libs | `messaging-release.aar` |

**权限：**
```xml
<uses-permission android:name="android.permission.RECEIVE_SMS"/>
<uses-permission android:name="android.permission.SEND_SMS"/>
<uses-permission android:name="android.permission.WRITE_SMS"/>
<uses-permission android:name="android.permission.READ_SMS"/>
```

**dcloud_properties.xml：**
```xml
<feature name="Messaging" value="io.dcloud.adapter.messaging.MessagingPluginImpl" />
```

---

### Record（录音）

**权限：**
```xml
<uses-permission android:name="android.permission.RECORD_AUDIO" />
<uses-permission android:name="android.permission.MODIFY_AUDIO_SETTINGS" />
```

---

### SQLite（数据库）

| 路径 | 文件 |
|---|---|
| SDK/libs | `sqlite-release.aar` |

**dcloud_properties.xml：**
```xml
<feature name="Sqlite" value="io.dcloud.feature.sqlite.DataBaseFeature"/>
```

---

### gcanvas

| 路径 | 文件 |
|---|---|
| SDK/libs | `weex_gcanvas-release.aar` |

---

### 拓展模块

- `audio-mp3aac-release.aar` — 录制音频时需要录制MP3格式时使用，拷贝到libs即可，不需额外配置。

---

### 隐私与政策提示框配置

#### 一级弹窗

| 字符串键名 | 字符串键值 |
|---|---|
| dcloud_privacy_prompt_title | 提示框标题，默认"服务协议和隐私政策" |
| dcloud_privacy_prompt_accept_button_text | 接受按钮文本，默认"同意" |
| dcloud_privacy_prompt_refuse_button_text | 拒绝按钮文本，默认不显示 |
| dcloud_privacy_prompt_message | 提示框内容，支持richtext |

#### 二级弹窗

| 字符串键名 | 字符串键值 |
|---|---|
| dcloud_second_privacy_prompt_title | 二级弹窗标题，默认不显示 |
| dcloud_second_privacy_prompt_accept_button_text | 确认按钮，默认"确定" |
| dcloud_second_privacy_prompt_refuse_button_text | 拒绝按钮，默认不显示 |
| dcloud_second_privacy_prompt_message | 内容，支持richtext |

> 默认不显示二级弹窗，配置后点击一级弹窗拒绝按钮时才会弹出。

---

### 国际化配置字符串

详见原文档，包括：
- html input(type=file) 选择页面国际化
- 图片选择器国际化字符串（多图）
- 应用启动时引导用户允许权限的提示语

---

## 15. 第三方 SDK 依赖说明

### 默认集成依赖库

| SDK | 版本 | 备注 |
|---|---|---|
| androidx | V1.1.0 | androidx相关依赖库 |
| fastjson | v1.2.83 | JSON解析库 |
| android-gif-drawable | V1.2.23 | gif图显示 |
| 移动安全联盟OAID | V1.0.25 | oaid获取 |
| glide | V4.9.0 | 图片预览 |
| fresco | V1.13.0 | nvue图片展示 |
| webkit | V1.3.0 | 暗黑模式支持 |

### 其他功能模块依赖库

| SDK | 版本 | 使用模块 |
|---|---|---|
| 个推push | V3.3.7.0 | unipush |
| 百度定位 | V7.5.0 | 定位 |
| 百度地图 | V5.4.1 | map |
| 高德定位 | V6.4.5 | 定位 |
| 高德地图 | V10.0.700 | map |
| 微信 | V6.8.0 | 登录/分享/支付 |
| 新浪微博 | V12.5.0 | 登录/分享 |
| QQ | V3.5.12 | 登录/分享 |
| 友盟统计 | V9.6.1 | 统计 |
| 百度语音 | V3.4.1.101 | 语音 |
| LiteAVSDK | V6.3.7089 | livepusher |
| 腾讯x5内核 | V4.3.0.1148_43697 | X5 |
| hms | V6.11.0.300 | 华为push |
| agcp | V1.9.1.301 | 华为AGC |
| 穿山甲&GroMore | V5.7.0.5 | 广告 |
| 优量汇广告 | V4.542.1412 | 广告 |
| 快手广告联盟 | V3.3.53.3 | 广告 |
| 快手内容联盟 | V3.3.53 | 广告 |
| sigmob广告 | V4.12.7 | 广告 |
| 百度广告 | V9.322 | 广告 |
| 华为广告 | V13.4.66.300 | 广告 |
| Pangle广告 | V5.0.0.3 | 广告 |
| google AdMob | V21.4.0 | 广告 |
| ijkplayer | V0.8.8 | 视频播放 |
| DanmakuFlameMaster | V0.6.2 | 弹幕 |
| lame | V3.100 | 音频录音(MP3) |
| play-services-auth | V19.2.0 | Google登录 |
| facebook-android-sdk | V16.1.3 | Facebook登录 |

---

## Android 注意事项

### 1. minSdkVersion设置的大于21时，应用启动白屏

需要在AndroidManifest.xml的application节点添加 `android:extractNativeLibs="true"`：

```xml
<application
    android:extractNativeLibs="true">
</application>
```

### 2. HBuilderX 4.41版，离线打包报 error: style attribute 'android:attr/windowOptOutEdgeToEdgeEnforcement' not found

从4.41版起，为了适配Android 15，需要配置：
- **compileSdk**: 35
- **buildToolsVersion**: 35.0.0
- **Gradle**: 8.11.1 版
- **Android Gradle 插件**: 8.7.3 版

### 3. 当设置targetSdkVersion为34时，上架 Google Play 应用白屏

如果targetSdkVersion设置为34时，需要在build.gradle的android节点下新增以下内容：

```gradle
packagingOptions {
    jniLibs {
        useLegacyPackaging true
    }
}
```

### 4. 上架 Google Play 提示：SoLoader SDK 版本有问题

需要在主模块的build.gradle的dependencies节点中添加SoLoader的依赖：

```gradle
implementation "com.facebook.soloader:soloader:0.10.5"
```

### 5. 离线SDK集成实人认证时报错：lib/*/libc++_shared.so

离线SDK集成实人认证如果出现该报错时，需要在module的build.gradle的android节点下添加如下内容：

```gradle
packagingOptions {
    pickFirst 'lib/*/libc++_shared.so'
}
```

### 6. 解决上架 Google Play 审核报 DCloud SDK 包含从未知来源下载或安装应用的问题

HBuilder X 3.8.7-alpha开始，离线打包将安装功能独立成单独的aar —— **install-apk-release.aar**，上架谷歌市场不能包含此库，非谷歌市场可酌情考量。

不包含此库，调用plus.runtime.install将无法安装apk文件。

其余上架谷歌市场注意事项可参考[文档](https://uniapp.dcloud.net.cn/tutorial/android-gp.html)。

### 7. 离线打包存在多个以uni-jsframework开头的文件

离线打包时为减少集成难度，默认会将所有框架都包含在内，如果需要去除其余框架，可参考[文档](https://nativesupport.dcloud.net.cn/AppDocs/FAQ/jsframeworkdeclare.html)配置。

### 8. 适配暗黑模式

适配暗黑模式新增了webkit依赖库，需要将如下配置添加到build.gradle中：

```gradle
dependencies {
    implementation 'androidx.webkit:webkit:1.3.0'
}
```

为适配暗黑模式，需要在AndroidManifest.xml中PandoraEntryActivity对应的android:configChanges中添加uiMode：

```xml
<activity
    android:name="io.dcloud.PandoraEntryActivity"
    android:launchMode="singleTask"
    android:configChanges="orientation|keyboardHidden|screenSize|mcc|mnc|fontScale|keyboard|smallestScreenSize|screenLayout|screenSize|uiMode"
    android:hardwareAccelerated="true"
    android:permission="com.miui.securitycenter.permission.AppPermissionsEditor"
    android:screenOrientation="user"
    android:theme="@style/DCloudTheme"
    android:windowSoftInputMode="adjustResize">
    <intent-filter>
        <category android:name="android.intent.category.DEFAULT" />
        <category android:name="android.intent.category.BROWSABLE" />
        <action android:name="android.intent.action.VIEW" />
        <data android:scheme=" " />
    </intent-filter>
</activity>
```

### 9. 离线打包设置隐私协议状态

如果离线打包需要自定义隐私协议，为了不影响SDK正常使用，需要用户在同意或拒绝隐私状态时同步到SDK：

- **SDK.setAgreePrivacy(Context context, boolean isAllow)** - 设置SDK隐私协议的状态（支持版本：3.3.1+）
- **SDK.isAgreePrivacy(Context context)** - 获取SDK隐私协议的状态（支持版本：3.3.1+）

### 10. Android 12 适配

离线打包如果将targetSdkVersion设置为31时，在Android 12设备上安装可能会报如下错误：

```
adb: failed to install XXX.apk: Failure [INSTALL_PARSE_FAILED_MANIFEST_MALFORMED: Failed parse during installPackageLI: /data/base.apk (at Binary XML file line #173): 
XXX.XXX.XXX.TestActivity: Targeting S+ (version 31 and above) requires that an explicit value for android:exported be defined when intent filters are present]
```

**解决方案**：Android 12 中要求包含 intent-filter 的 activity、service 或 receiver 必须显示声明 android:exported 属性：

```xml
<activity
    android:name="XXX.XXX.XXX.TestActivity"
    android:exported="true">
    <intent-filter>
        ......
    </intent-filter>
</activity>
```

> **注意**：Android系统默认包含 intent-filter 的组件android:exported默认值为true，所以建议将android:exported设置为true

### 11. 应用启动白屏或者提示打包时未添加ui模块

大多数是因为build.gradle中配置了混淆。如果需要使用proguard混淆代码，需确保不要混淆SDK的代码。混淆配置和混淆文件可以参考SDK中的UniPlugin-Hello-AS项目。

### 12. 编译报错 style attribute 'android:attr/forceDarkAllowed' not found.

需要将 compileSdkVersion 设置为 **29 或以上**。

### 13. breakpad 配置

离线SDK新增加了breakpad-build-release.aar，直接将这个库拷贝到libs目录下即可。详情可参考[文档](https://nativesupport.dcloud.net.cn/AppDocs/usesdk/android.html)。

### 14. gallery 冲突问题

gallery-dmcBig-release.aar相应代码被加入到lib.5plus.base-release.aar，使用时请删除gallery-dmcBig-release.aar库。

### 15. 离线打包编译报错

如果离线打包编译时提示以下错误：

```
Execution failed for task ':hbuilder:checkDebugDuplicateClasses'.
> 1 exception was raised by workers:
java.lang.IllegalStateException: java.lang.IllegalStateException: Worker finished without being first started
```

**解决方案**：

1. 将项目根目录下的build.gradle中的gradle插件版本升级：

```gradle
buildscript {
    repositories {
        jcenter()
        google()
    }
    dependencies {
        classpath 'com.android.tools.build:gradle:4.1.1'
    }
}
```

2. 修改项目根目录 gradle/gradle-wrapper.properties 下的gradle的版本到6.5：

```properties
distributionBase=GRADLE_USER_HOME
distributionPath=wrapper/dists
zipStoreBase=GRADLE_USER_HOME
zipStorePath=wrapper/dists
distributionUrl=https\://services.gradle.org/distributions/gradle-6.5-bin.zip
```

### 16. 离线打包无法调起应用安装界面

如果离线打包调用plus.runtime.install无法调起安装界面，需要在Androidmanifest.xml中添加：

**添加权限**（manifest节点下）：

```xml
<uses-permission android:name="android.permission.REQUEST_INSTALL_PACKAGES"/>
```

**添加provider节点**（application节点下）：

```xml
<provider
    android:name="io.dcloud.common.util.DCloud_FileProvider"
    android:authorities="${apk.applicationId}.dc.fileprovider"
    android:exported="false"
    android:grantUriPermissions="true">
    <meta-data
        android:name="android.support.FILE_PROVIDER_PATHS"
        android:resource="@xml/dcloud_file_provider" />
</provider>
```

> ${apk.applicationId}须替换成应用的包名。

### 17. 离线打包Android 10上无法启动相机

在application节点下添加provider节点（同第16条）：

```xml
<provider
    android:name="io.dcloud.common.util.DCloud_FileProvider"
    android:authorities="${apk.applicationId}.dc.fileprovider"
    android:exported="false"
    android:grantUriPermissions="true">
    <meta-data
        android:name="android.support.FILE_PROVIDER_PATHS"
        android:resource="@xml/dcloud_file_provider" />
</provider>
```

### 18. uni-AD业务状态异常

如果出现uni-AD业务状态异常提醒，请删除掉未申请的平台的相关配置和aar。例如广告后台添加了穿山甲广告，但没有添加360和广点通的广告，请删除掉广点通和360的相关配置和aar。

### 19. x5配置

如果需要使用x5内核，将webview-x5-release.aar拷贝到libs目录下，直接运行即可。

uni-app将webview-x5-release.aar和weex_webview-x5-release.aar拷贝到libs目录下即可。

### 20. 推送上传谷歌市场注意事项

参考文档：[Android离线SDK解决使用UniPush和个推推送违反谷歌应用商店（GooglePlay）个人和敏感信息政策的问题](https://ask.dcloud.net.cn/article/36495)

### 21. 高德地图上传谷歌市场注意事项

如需上传谷歌市场，将原来的amap-libs-release.aar替换成amap-gp-libs-release.aar即可。

### 22. uni-app离线打包注意事项

参考文档：[uni-app离线打包Android平台注意事项](https://ask.dcloud.net.cn/article/35139)

### 23. 重写Application

如果集成离线SDK时需要重写application，**必须继承自DCloudApplication**，否则会导致SDK中业务逻辑无法正常运行：

```xml
<application 
    android:name="io.dcloud.test.TestApplication" 
    android:icon="@drawable/icon" 
    android:label="@string/app_name" 
    tools:replace="android:name">
</application>
```

### 24. 添加so库

如果需要集成的第三方sdk存在so库文件，只需添加 **armeabi-v7a、arm64-v8a、x86** 三个文件夹即可，如果添加其他文件夹会导致在部分手机上无法运行。

### 25. 打包aab运行白屏

按以下配置修改：

1. 原生项目主app的AndroidManifest.xml中，application节点配置 `android:extractNativeLibs="true"`
2. 原生项目根目录 gradle.properties 配置 `android.bundle.enableUncompressedNativeLibs=false`
3. 重新编译打包

### 26. 适配Android 13 文件权限

为了兼容Android 13 新的权限要求，需要在AndroidManifest.xml 中新增下面的权限声明：

```xml
<uses-permission android:name="android.permission.READ_MEDIA_IMAGES" />
<uses-permission android:name="android.permission.READ_MEDIA_VIDEO" />
```

---

**文档生成时间**：2026-05-29  
**基于官方文档**：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/androidModuleConfig/
