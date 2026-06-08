# Share 分享（Android）

> **适用版本**：HBuilderX 5.0+
> **平台**：Android
> **官方文档**：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/androidModuleConfig/

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

### 相关模块

- [Oauth 登录鉴权](oauth.md) — 微信/QQ/微博登录与分享共用同一套SDK
- [Payment 支付](payment.md) — 微信支付也依赖微信SDK
