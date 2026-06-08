# Oauth（登录鉴权）（iOS）

> **适用版本**：HBuilderX 5.0+
> **平台**：iOS (iPhone/iPad)
> **官方文档**：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/iOSModuleConfig/

---

iOS 支持多种第三方登录方式，包括微信、QQ、微博、Apple Sign In、Google、Facebook 等。

## 3.1 一键登录（个推）

### 需要引入的系统框架

| 框架 | 说明 |
|------|------|
| CoreTelephony.framework | 用于获取运营商信息 |
| AdSupport.framework | 广告标识符（可选） |

### Info.plist 配置

```xml
<key>GETUI_APPID</key>
<string>%个推AppID%</string>
<key>GY_APP_ID</key>
<string>%一键登录AppID%</string>
```

### CocoaPods 依赖

```ruby
pod 'GySDK', '~> 3.x.x'  # 个推一键登录SDK
```

### dcloud_properties.xml 配置

```xml
<feature name="OAuth" value="io.dcloud.feature.oauth.OAuthFeatureImpl">
    <module name="OAuth-IGETui" value="io.dcloud.feature.igetui.GeTuiOAuthService"/>
</feature>
```

## 3.2 微信登录

> **注意**：如已集成微信分享，可复用微信SDK，无需重复配置。

### Info.plist 配置

同微信分享配置（见 [Share（分享）](share.md) §2.1 节）

### 需要拷贝的文件

同微信分享（见 [Share（分享）](share.md) §2.1 节）

### dcloud_properties.xml 配置

```xml
<feature name="OAuth" value="io.dcloud.feature.oauth.OAuthFeatureImpl">
    <module name="OAuth-Weixin" value="io.dcloud.feature.oauth.weixin.WeiXinOAuthService"/>
</feature>
```

## 3.3 QQ 登录

> **注意**：如已集成QQ分享，可复用QQ SDK。

### Info.plist 配置

同 QQ 分享配置（见 [Share（分享）](share.md) §2.2 节）

### dcloud_properties.xml 配置

```xml
<feature name="OAuth" value="io.dcloud.feature.oauth.OAuthFeatureImpl">
    <module name="OAuth-QQ" value="io.dcloud.feature.oauth.qq.QQOAuthService"/>
</feature>
```

## 3.4 新浪微博登录

> **注意**：如已集成微博分享，可复用微博SDK。

### dcloud_properties.xml 配置

```xml
<feature name="OAuth" value="io.dcloud.feature.oauth.OAuthFeatureImpl">
    <module name="OAuth-Sina" value="io.dcloud.feature.oauth.sina.SinaOAuthService"/>
</feature>
```

## 3.5 Apple 登录（Sign in with Apple）

> **重要**：如果应用集成了其他第三方登录方式，根据 Apple 审核指南，**必须同时提供 Apple 登录选项**。

### 需要引入的系统框架

| 框架 | 说明 |
|------|------|
| AuthenticationServices.framework | Apple 认证服务框架 |

### Info.plist 配置

无需特殊配置，但需在 Xcode 中 Signing & Capabilities 添加 **Sign in with Apple**

### Objective-C 代码示例

```objc
@import AuthenticationServices;

// 实现 Apple 登录按钮点击事件
- (void)handleAppleSignIn {
    if (@available(iOS 13.0, *)) {
        ASAuthorizationAppleIDProvider *provider = [[ASAuthorizationAppleIDProvider alloc] init];
        ASAuthorizationAppleIDRequest *request = [provider createRequest];
        request.requestedScopes = @[ASAuthorizationScopeFullName, ASAuthorizationScopeEmail];
        
        ASAuthorizationController *controller = [[ASAuthorizationController alloc] initWithAuthorizationRequests:@[request]];
        controller.delegate = self;
        controller.presentationContextProvider = self;
        [controller performRequests];
    }
}
```

### dcloud_properties.xml 配置

```xml
<feature name="OAuth" value="io.dcloud.feature.oauth.OAuthFeatureImpl">
    <module name="OAuth-Apple" value="io.dcloud.feature.oauth.apple.AppleOAuthService"/>
</feature>
```

## 3.6 Google 登录

### 需要引入的系统框架

| 框架 | 说明 |
|------|------|
| SafariServices.framework | Safari 服务框架 |

### CocoaPods 依赖

```ruby
pod 'GoogleSignIn', '~> 7.x.x'
```

### Info.plist 配置

```xml
<key>GIDClientID</key>
<string>%您的Google Client ID%</string>

<key>CFBundleURLTypes</key>
<array>
    <dict>
        <key>CFBundleURLSchemes</key>
        <array>
            <string>%您的REVERSED_CLIENT_ID%</string>
        </array>
    </dict>
</array>
```

### dcloud_properties.xml 配置

```xml
<feature name="OAuth" value="io.dcloud.feature.oauth.OAuthFeatureImpl">
    <module name="OAuth-Google" value="io.dcloud.feature.google.GoogleOAuthService"/>
</feature>
```

## 3.7 Facebook 登录

> **注意**：如已集成 Facebook 分享，可复用 Facebook SDK。

### dcloud_properties.xml 配置

```xml
<feature name="OAuth" value="io.dcloud.feature.oauth.OAuthFeatureImpl">
    <module name="OAuth-Facebook" value="io.dcloud.feature.facebook.FacebookOAuthService"/>
</feature>
```

## ⚠️ iOS 登录注意事项

1. **Apple 登录强制要求**：应用上架 App Store 时，如果使用了任何第三方登录，必须同时提供 Apple 登录
2. **隐私政策**：每个登录方式都需要在隐私政策中说明数据收集和使用情况
3. **测试环境**：部分登录方式（如 Apple 登录）需要真机测试，模拟器可能不支持
4. **回调处理**：确保正确处理各平台的 OAuth 回调

---

## 交叉引用

- 上一篇：[Share（分享）](share.md)
- 下一篇：[Map（地图）](map.md)
- 相关模块：[Share（分享）](share.md)（微信/QQ/微博/Facebook 可复用 SDK）、[Payment（支付）](payment.md)
