# Oauth（登录鉴权）（iOS）

> **适用版本**：HBuilderX 5.0+
> **平台**：iOS (iPhone/iPad)
> **官方文档**：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/iOSModuleConfig/oauth.html

---

Oauth 模块支持

- 手机号一键登录
- 新浪微博登录
- QQ登录
- 微信登录
- 苹果授权登录
- Google登录
- Facebook登录

需要到各开放平台申请帐号

## HBuilderX 5.13+ 本地 Pod 集成（推荐）

HBuilderX 5.13+ 推荐使用本地 Pod 集成登录鉴权模块：

| Pod 名称 | 用途 |
|---|---|
| `Oauth` | 登录基础模块 |
| `Oauth-Univerify` | 一键登录 |
| `Oauth-Sina` | 新浪微博登录 |
| `Oauth-QQ` | QQ 登录 |
| `Oauth-Wechat` | 微信登录 |
| `Oauth-Wechat-PaySDK` | 微信登录（含支付能力） |

> **注意**：只有同时需要微信支付能力时，才使用 `Oauth-Wechat-PaySDK`，避免不需要支付能力的应用引入 PaySDK 版本。

## 配置登录平台参数

在工程中搜索 feature.plist 文件（位于 PandoraApi.bundle 中），在 OAuth->extend 节点下添加对应平台的配置

## 一键登录（univerify）

### 添加依赖库及资源

| 依赖库 | 系统库 | 依赖资源 |
|---|---|---|
| liblibOauth.a、UniVerify.framework、GTCommonSDK.xcframework、GeYanSdk.xcframework | libz.tbd、libc++.tbd、libsqlite3.0.tbd、AdSupport.framework | TYRZResource.bundle |

### 工程配置

1. 在 info.plist 中添加 `DCloudConfig` 节点类型为 Dictionary，然后添加 `univerify` 子节点类型为 Dictionary，然后添加 `appid` 节点类型为 String，值填写您在 [DCloud开发者中心](https://dev.dcloud.net.cn/) 申请一键登录对应的 appid

2. 使用方法请参考 [一键登录 使用指南](https://uniapp.dcloud.io/univerify)

## 新浪微博登录

### 添加依赖库及资源

| 依赖库 | 系统库 | 依赖资源 |
|---|---|---|
| liblibOauth.a、libSinaWBOauth.a、libWeiboSDK.a | ImageIO.framework、libsqlite3.0.tbd | WeiboSDK.bundle |

### 工程配置

1. 在 info.plist 中添加 `sinaweibo` 字段，填入自己帐号的信息

2. 在工程的 info -> URL types 中添加配置，identifier 填写 `com.weibo`，URL Schemes 填写 `wb[后面填写appkey]`

3. 在 info.plist 添加 Schemes 白名单：

```xml
<key>LSApplicationQueriesSchemes</key>
<array>
    <string>sinaweibo</string>
    <string>sinaweibohd</string>
    <string>sinaweibosso</string>
    <string>sinaweibonotes</string>
    <string>weibosdk</string>
    <string>weibosdk2.5</string>
</array>

<key>CFBundleURLTypes</key>
<array>
    <dict>
        <key>CFBundleURLSchemes</key>
        <array>
            <string>wb%您的微博AppKey%</string>
        </array>
    </dict>
</array>
```

4. 配置 Associated Domains（域名）

填写通用链接域名

## QQ 登录

### 添加依赖库及资源

| 依赖库 | 系统库 | 依赖资源 |
|---|---|---|
| liblibOauth.a、libQQOauth.a、TencentOpenAPI.xcframework | 无 | 无 |

### 工程配置

1. 在工程的 info -> URL types 中添加配置，identifier 填写 `tencentopenapi`，URL Schemes 填写 `tencent[后面填写appid]`

2. 在 info.plist 添加 Schemes 白名单：

```xml
<key>LSApplicationQueriesSchemes</key>
<array>
    <string>mqqapi</string>
    <string>mqq</string>
    <string>mqqOpensdkSSoLogin</string>
    <string>mqqconnect</string>
    <string>mqqopensdkdataline</string>
    <string>mqqopensdkgrouptribeshare</string>
    <string>mqqopensdkfriend</string>
    <string>mqqopensdkapi</string>
    <string>mqqopensdkapiV2</string>
    <string>mqqopensdkapiV3</string>
    <string>mqqopensdkapiV4</string>
    <string>mqzoneopensdk</string>
    <string>wtloginmqq</string>
    <string>wtloginmqq2</string>
    <string>mqzone</string>
    <string>mqzonev2</string>
    <string>mqzoneshare</string>
    <string>wtloginqzone</string>
    <string>mqqwpa</string>
    <string>mqzoneopensdkapiV2</string>
    <string>mqzoneopensdkapi19</string>
    <string>mqzoneopensdkapi</string>
    <string>mqqbrowser</string>
    <string>mttbrowser</string>
</array>

<key>CFBundleURLTypes</key>
<array>
    <dict>
        <key>CFBundleURLSchemes</key>
        <array>
            <string>tencent%您的QQ AppID%</string>
        </array>
    </dict>
</array>
```

3. 在 info.plist 中添加 `qq` 字段，填入自己帐号的信息

4. 配置 Associated Domains（域名）

填写通用链接域名

## 微信登录

### 添加依赖库及资源

| 依赖库 | 系统库 | 依赖资源 |
|---|---|---|
| liblibOauth.a、libWXOauth.a、libWeChatSDK.a | libsqlite3.0.tbd、libz.tbd、CoreTelephony.framework、SystemConfiguration.framework | 无 |

注意：SDK 中的

- `libWeChatSDK_pay.a` 为带支付功能的微信SDK，支持微信分享、微信支付及微信授权登录功能
- `libWeChatSDK.a` 为不带支付功能的SDK，仅支持微信分享和授权登录，**不使用支付功能请添加此库，避免审核被拒**
- 不要同时添加到工程避免冲突

### 工程配置

1. 在工程的 info -> URL types 中添加配置，identifier 填写 `weixin`，URL Schemes 填写 `wx[后面填写appid]`

2. 在 info.plist 添加 Schemes 白名单：

```xml
<key>LSApplicationQueriesSchemes</key>
<array>
    <string>weixin</string>
    <string>weixinULAPI</string>
</array>

<key>CFBundleURLTypes</key>
<array>
    <dict>
        <key>CFBundleURLSchemes</key>
        <array>
            <string>wx%您的微信AppID%</string>
        </array>
    </dict>
</array>
```

3. 配置 Associated Domains（域名）

填写通用链接域名

4. 在 info.plist 添加 `weixin` 项，填写微信 `appid` 及 `UniversalLinks`,值和您在微信开放平台配置的一致

5. 在工程的 AppDelegate.m 系统通用链接回调方法中调用框架方法如下：

```objc
- (BOOL)application:(UIApplication *)application continueUserActivity:(NSUserActivity *)userActivity restorationHandler:(void(^)(NSArray<id<UIUserActivityRestoring>> * __nullable restorableObjects))restorationHandler {
    [PDRCore handleSysEvent:PDRCoreSysEventContinueUserActivity withObject:userActivity];
    restorationHandler(nil);
    return YES;
}
```

## 苹果登录

### 添加依赖库及资源

| 依赖库 | 系统库 | 依赖资源 |
|---|---|---|
| liblibOauth.a、libAppleOauth.a | AuthenticationServices.framework | 无 |

**注意：AuthenticationServices.framework Status 为 Optional**

### 开启 Sign in with Apple

在原生工程 -> Signing&Capabilities -> + Capability 中添加 Sign in with Apple 服务

证书配置及使用说明请参考 [文档](https://ask.dcloud.net.cn/article/36651)

## Google 登录

### 添加依赖库及资源

| 依赖库 | 系统库 | 依赖资源 |
|---|---|---|
| liblibOauth.a、libGoogleOauth.a、GoogleSignIn.xcframework、AppAuth.xcframework、GTMAppAuth.xcframework、GTMSessionFetcher.xcframework | CoreText.framework、CoreGraphics.framework、LocalAuthentication.framework、SafariServices.framework、Security.framework | GoogleSignIn.bundle |

### 工程配置

1. 在 info.plist 添加 `GIDClientID` 项，填写 Google `clientid`

2. 在工程的 info -> URL types 中添加配置，identifier 填写 `google_url`，添加您的反向 clientid 作为 URL Schemes

## Facebook 登录

### 添加依赖库及资源

| 依赖库 | 系统库 | 依赖资源 |
|---|---|---|
| liblibOauth.a、libFBOauth.a、FBSDKCoreKit.xcframework、FBAEMKit.xcframework、FBSDKCoreKit_Basics.xcframework、FBSDKLoginKit.xcframework | libc++.tbd、Accelerate.framework、Accounts.framework、AdSupport.framework、AudioToolbox.framework、CoreGraphics.framework、QuartzCore.framework、Security.framework、Social.framework、StoreKit.framework | 无 |

### 工程配置

1. 在 info.plist 添加 `FacebookAppID`、`FacebookClientToken` 项，分别填写 Facebook `appid` 和 `clientToken`

2. 在工程的 info -> URL types 中添加配置，identifier 填写 `facebook`，URL Schemes 填写 `fb[后面填写appid]`

3. 在 info.plist 添加 Schemes 白名单：

```xml
<key>LSApplicationQueriesSchemes</key>
<array>
    <string>fb</string>
    <string>fbapi</string>
    <string>fb-messenger-share-api</string>
    <string>fbshareextension</string>
    <string>fbauth2</string>
</array>
```

## 除苹果授权登录外都需要实现的方法

在 AppDelegate.m 文件的系统回调方法中调用框架的方法如下：

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

- (BOOL)application:(UIApplication *)application continueUserActivity:(NSUserActivity *)userActivity restorationHandler:(void(^)(NSArray<id<UIUserActivityRestoring>> * __nullable restorableObjects))restorationHandler{
    [PDRCore handleSysEvent:PDRCoreSysEventContinueUserActivity withObject:userActivity];
    return YES;
}
```

---

## 交叉引用

- 上一篇：[Share（分享）](share.md)
- 下一篇：[Map（地图）](map.md)
- 相关模块：[Share（分享）](share.md)（微信/QQ/微博可复用 SDK）、[Payment（支付）](payment.md)（含微信支付配置）
