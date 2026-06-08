# Payment 支付（Android）

> **适用版本**：HBuilderX 5.0+
> **平台**：Android
> **官方文档**：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/androidModuleConfig/

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

<meta-data android:name="returnUrl" android:value="%YOUR-CUSTOM-SCHEME%//paypalpay"/>
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

### 相关模块

- [Oauth 登录鉴权](oauth.md) — 微信登录与微信支付共用微信SDK
- [Share 分享](share.md) — 微信分享与微信支付共用微信SDK
- [第三方 SDK 依赖说明](third-party-dependencies.md) — 微信/QQ等版本信息
