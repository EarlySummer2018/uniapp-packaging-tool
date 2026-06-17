# Share（分享）（iOS）

> **适用版本**：HBuilderX 5.0+
> **平台**：iOS (iPhone/iPad)
> **官方文档**：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/iOSModuleConfig/share.html

---

目前分享功能支持新浪微博分享、QQ分享、微信分享，分享功能首先需要到各开放平台申请帐号，参考 [文档](http://ask.dcloud.net.cn/article/36)

## HBuilderX 5.13+ 本地 Pod 集成（推荐）

HBuilderX 5.13+ 推荐使用本地 Pod 集成分享模块：

| Pod 名称 | 用途 |
|---|---|
| `Share` | 分享基础模块 |
| `Share-Sina` | 新浪微博分享 |
| `Share-QQ` | QQ 分享 |
| `Share-Wechat` | 微信分享 |
| `Share-Wechat-PaySDK` | 微信分享（含支付能力） |

> **注意**：只有同时需要微信支付能力时，才使用 `Share-Wechat-PaySDK`，避免不需要支付能力的应用引入 PaySDK 版本。

## 新浪微博分享

### 添加依赖库及资源

| 依赖库 | 系统库 | 依赖资源 |
|---|---|---|
| liblibShare.a、libSinaShare.a、libWeiboSDK.a | ImageIO.framework、libsqlite3.0.tbd | WeiboSDK.bundle |

### 工程配置

1. 在 info.plist 中添加 `sinaweibo` 字段，填入自己帐号的信息

#### 注意 SDK 3.2.0+ 必须按照下图填写

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

#### 注意 SDK 3.2.0+ 必须按照下图填写

4. 配置 Associated Domains（域名）

填写通用链接域名

## QQ 分享

### 添加依赖库及资源

| 依赖库 | 系统库 | 依赖资源 |
|---|---|---|
| liblibShare.a、libQQShare.a、TencentOpenAPI.xcframework | 无 | 无 |

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

#### 注意 SDK 3.2.0+ 必须按照下图填写

3. 在 info.plist 中添加 `qq` 字段，填入自己帐号的信息

4. 配置 Associated Domains（域名）

填写通用链接域名

## 微信分享

### 添加依赖库及资源

| 依赖库 | 系统库 | 依赖资源 |
|---|---|---|
| liblibShare.a、libweixinShare.a、libWeChatSDK.a | libsqlite3.0.tbd、libz.tbd、CoreTelephony.framework、SystemConfiguration.framework | 无 |

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

## 所有分享都需要实现的方法

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
```

---

## 交叉引用

- 上一篇：[Push（消息推送）](push.md)
- 下一篇：[Oauth（登录鉴权）](oauth.md)
- 相关模块：[Payment（支付）](payment.md)（含微信支付配置）
