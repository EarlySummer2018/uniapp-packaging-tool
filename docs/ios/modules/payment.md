# Payment（支付）（iOS）

> **适用版本**：HBuilderX 5.0+
> **平台**：iOS (iPhone/iPad)
> **官方文档**：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/iOSModuleConfig/pay.html

---

HBuilderX 5.13+ 推荐使用本地 Pod 集成支付模块。支付基础模块使用 `Payment`，支付宝支付使用 `Payment-AliPay`，微信支付使用 `Payment-Wechat`，Apple IAP 使用 `Payment-IAP`，PayPal 使用 `Payment-Paypal`，Stripe 使用 `Payment-Stripe`。 手动集成时再参考下方"依赖库 / 系统库 / 资源文件"表格。

目前支持支付宝、微信支付、苹果内购支付、paypal支付、stripe支付：

支付插件首先需要到各开放平台申请帐号，查看该[文档](http://ask.dcloud.net.cn/article/71)

## 配置支付平台参数

在工程中搜索 feature.plist 文件（位于 PandoraApi.bundle 中），在 Payment->extend 节点下添加对应平台的配置

**注意：如果用不到的不要配置，以免影响审核**

## 支付宝

### 添加依赖库及资源

| 依赖库 | 系统库 | 资源文件 |
|---|---|---|
| liblibPayment.a、libalixpayment.a、AlipaySDK.framework | Security.framework、CoreMotion.framework、SystemConfiguration.framework、CFNetwork.framework、libc++.tbd | AlipaySDK.bundle |

### 工程配置

1. 在 URL Types 中添加配置：identifier 填写 `alixpay`，URL Schemes 填写 `alix[后面是您在支付宝平台申请的appid]`，如果没有该项按照图中的格式创建

2. 在 info.plist 添加 Schemes 白名单配置：

```xml
<key>LSApplicationQueriesSchemes</key>
<array>
    <string>alipay</string>
    <string>alipays</string>
</array>
```

## 微信支付

### 添加依赖库及资源

| 依赖库 | 系统库 | 资源文件 |
|---|---|---|
| liblibPayment.a、libwxpay.a、libWeChatSDK_pay.a | libsqlite3.0.tbd、libz.tbd、CoreTelephony.framework、SystemConfiguration.framework | 无 |

注意：SDK 中的

- `libWeChatSDK_pay.a` 为带支付功能的微信SDK，支持微信分享、微信支付及微信授权登录功能
- `libWeChatSDK.a` 为不带支付功能的SDK，仅支持微信分享和授权登录，**不使用支付功能请添加此库，避免审核被拒**
- 不要同时添加到工程避免冲突

### 工程配置

1. 在 URL Types 中添加配置：identifier 填写 `weixin`，URL Schemes 填写 `wx[后面是您在微信平台申请的appkey]`，如果没有该项按照图中的格式创建

2. 在 info.plist 添加 Schemes 白名单配置：

```xml
<key>LSApplicationQueriesSchemes</key>
<array>
    <string>weixin</string>
    <string>weixinULAPI</string>
</array>
```

3. 配置 Associated Domains（域名）

填写通用链接域名

4. 在 info.plist root 节点添加 `UniversalLinks` 字段，值和您在微信开放平台配置的一致（SDK 3.2.0版本以后 此项已废弃，仅保留字段,配置参数已经位置如步骤5所示）

5. 在 info.plist 添加 `weixin`(3.2.0 以前为 `weixinoauth`) 项，填写微信 `appid` 及 `UniversalLinks`,值和您在微信开放平台配置的一致

6. 在工程的 AppDelegate.m 系统通用链接回调方法中调用框架方法如下：

```objc
- (BOOL)application:(UIApplication *)application continueUserActivity:(NSUserActivity *)userActivity restorationHandler:(void(^)(NSArray<id<UIUserActivityRestoring>> * __nullable restorableObjects))restorationHandler {
    [PDRCore handleSysEvent:PDRCoreSysEventContinueUserActivity withObject:userActivity];
    restorationHandler(nil);
    return YES;
}
```

## 苹果应用内购支付

### 添加依赖库及资源

| 依赖库 | 系统库 | 资源文件 |
|---|---|---|
| liblibPayment.a、libIAPPay.a | StoreKit.framework | 无 |

## paypal 支付

> 注：SDK 3.3.7+、iOS 13.0+

### 添加依赖库及资源

| 依赖库 | 系统库 | 资源文件 |
|---|---|---|
| liblibPayment.a、libpaypalpay.a、PayPalCheckout.xcframework | 无 | 无 |

### 工程配置

1. 在 info.plist 添加 `paypal` 项，填写 `returnUrl`，参考如下：

```xml
<key>paypal</key>
<dict>
    <key>returnUrl</key>
    <string>%您的returnUrl%</string>
</dict>
```

## stripe 支付

> 注：SDK 3.3.7+、iOS 13.0+

### 添加依赖库及资源

| 依赖库 | 系统库 | 资源文件 |
|---|---|---|
| liblibPayment.a、libstripepay.a、StripeApplePay.xcframework、StripeCore.xcframework、StripeUICore.xcframework、Stripe3DS2.xcframework、StripePayments.xcframework、StripePaymentsUI.xcframework、StripePaymentSheet.xcframework | 无 | 无 |

### 工程配置

1. 在 URL Types 中添加当前应用的自定义 URL Schemes

2. 在 info.plist 添加 `stripe` 项，填写 `returnUrl`，returnUrl 为当前应用的自定义 URL Schemes，参考如下：

```xml
<key>stripe</key>
<dict>
    <key>returnUrl</key>
    <string>%您的自定义URL Schemes%</string>
</dict>
```

## 除苹果支付外都需要实现的方法

**注意：以上支付方式都需要配置支付平台参数**

除苹果支付外，其他支付需在 AppDelegate.m 文件的系统回调方法中调用框架的方法如下：

```objc
- (BOOL)application:(UIApplication *)application handleOpenURL:(NSURL *)url
{
    [PDRCore handleSysEvent:PDRCoreSysEventOpenURL withObject:url];
    return YES;
}

- (BOOL)application:(UIApplication *)application openURL:(nonnull NSURL *)url options:(nonnull NSDictionary<UIApplicationOpenURLOptionsKey,id> *)options {
    [PDRCore handleSysEvent:PDRCoreSysEventOpenURLWithOptions withObject:@[url,options]];
    return YES;
}
```

---

## 交叉引用

- 上一篇：[Oauth（登录鉴权）](oauth.md)
- 下一篇：[Geolocation（定位）](geolocation.md)
- 相关模块：[Share（分享）](share.md)（微信 SDK 可复用）、[Oauth（登录鉴权）](oauth.md)（微信 SDK 可复用）
