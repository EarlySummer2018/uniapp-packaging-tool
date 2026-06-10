# Oauth 登录鉴权（Android）

> **适用版本**：HBuilderX 5.0+
> **平台**：Android
> **官方文档**：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/androidModuleConfig/oauth.html

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
<activity android:name="com.tencent.tauth.AuthActivity" android:exported="true" android:launchMode="singleTask" android:noHistory="true">
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

> `AuthActivity` 带有 `intent-filter`。面向 Android 12 及以上构建时必须显式设置 `android:exported="true"`，否则 Manifest 合并会失败。

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

### 相关模块

- [Share 分享](share.md) — 微信/QQ/微博分享与登录共用同一套SDK
- [Push 消息推送](push.md) — 一键登录依赖个推SDK
- [Payment 支付](payment.md) — 微信支付依赖微信SDK
- [Geolocation 定位](geolocation.md) — 高德定位与高德地图存在冲突
