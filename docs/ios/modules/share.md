# Share（分享）（iOS）

> **适用版本**：HBuilderX 5.0+
> **平台**：iOS (iPhone/iPad)
> **官方文档**：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/iOSModuleConfig/

---

iOS 分享模块支持微信、QQ、微博、Facebook 等主流社交平台。

## 2.1 微信分享

### 需要引入的系统框架

无额外系统框架要求（微信SDK已包含）

### Info.plist 配置

```xml
<!-- 微信URL Scheme -->
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
            <string>%您的微信AppID%</string>
        </array>
    </dict>
</array>
```

### CocoaPods 依赖

```ruby
pod 'WechatOpenSDK', '1.9.2'  # 或最新版本
```

### 需要拷贝的文件

| 路径 | 文件 |
|------|------|
| SDK/libs | `libWeChatSDK.a`, `WXApi.h`, `WXApiObject.h` 等 |

### Objective-C 代码集成

```objc
#import "WXApi.h"

// 在 AppDelegate 中注册微信
- (BOOL)application:(UIApplication *)application didFinishLaunchingWithOptions:(NSDictionary *)launchOptions {
    
    // 注册微信SDK
    [WXApi registerApp:@"您的微信AppID" universalLink:@"https://您的UniversalLink.com/app/"];
    
    return YES;
}

// 处理微信回调
- (BOOL)application:(UIApplication *)app openURL:(NSURL *)url options:(NSDictionary<NSString *,id> *)options {
    return [WXApi handleOpenURL:url delegate:self];
}
```

### dcloud_properties.xml 配置

```xml
<feature name="Share" value="io.dcloud.feature.share.ShareFeatureImpl">
    <module name="Share-Weixin" value="io.dcloud.feature.share.weixin.WeiXinShareService"/>
</feature>
```

## 2.2 QQ 分享

### Info.plist 配置

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

### 需要拷贝的文件

| 路径 | 文件 |
|------|------|
| SDK/libs | `TencentOpenAPI.framework`, `TencentOpenApi_IOS_Bundle.bundle` |

### dcloud_properties.xml 配置

```xml
<feature name="Share" value="io.dcloud.feature.share.ShareFeatureImpl">
    <module name="Share-QQ" value="io.dcloud.feature.share.qq.QQShareService"/>
</feature>
```

## 2.3 新浪微博分享

### Info.plist 配置

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

### 需要拷贝的文件

| 路径 | 文件 |
|------|------|
| SDK/libs | `WeiboSDK.framework` 或 `libWeiboSDK.a` |

### dcloud_properties.xml 配置

```xml
<feature name="Share" value="io.dcloud.feature.share.ShareFeatureImpl">
    <module name="Share-Sina" value="io.dcloud.feature.share.sina.SinaShareService"/>
</feature>
```

## 2.4 Facebook 分享（可选）

### Info.plist 配置

```xml
<key>LSApplicationQueriesSchemes</key>
<array>
    <string>fb</string>
    <string>fbapi</string>
    <string>fb-messenger-share-api</string>
    <string>fbshareextension</string>
    <string>fbauth2</string>
</array>

<key>CFBundleURLTypes</key>
<array>
    <dict>
        <key>CFBundleURLSchemes</key>
        <array>
            <string>fb%您的Facebook App ID%</string>
        </array>
    </dict>
</array>
```

### CocoaPods 依赖

```ruby
pod 'FBSDKCoreKit'
pod 'FBSDKLoginKit'
pod 'FBSDKShareKit'
```

### dcloud_properties.xml 配置

```xml
<feature name="Share" value="io.dcloud.feature.share.ShareFeatureImpl">
    <module name="Share-Facebook" value="io.dcloud.feature.share.facebook.FacebookShareService"/>
</feature>
```

## ⚠️ Universal Links 配置（iOS 9+ 必须）

从 iOS 9 开始，应用间跳转需要配置 Universal Links：

1. 在 Apple Developer 后台配置 Associated Domains
2. 创建 `apple-app-site-association` 文件并上传到服务器
3. 在 Xcode 中 Signing & Capabilities 添加 Associated Domains

---

## 交叉引用

- 上一篇：[Push（消息推送）](push.md)
- 下一篇：[Oauth（登录鉴权）](oauth.md)
- 相关模块：[Payment（支付）](payment.md)（含微信支付配置）
